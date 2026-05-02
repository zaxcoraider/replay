//! ForkedSession — a replayable, mutable sandbox around a single transaction.
//!
//! Mutations are stored as a log and re-applied on every `execute()` call, so
//! you can always reset to baseline by clearing the mutation list. This keeps
//! the semantics simple and makes the diff computation trivial.

use crate::error::ReplayError;
use crate::idl::{AccountDecoder, IdlCache};
use crate::rpc::HeliusClient;
use crate::svm::replay_from_scratch;
use crate::trace::build_trace;
use crate::types::{AccountMutation, ReconstructedState, Trace, TraceDiff, TxContext};
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::collections::HashMap;

pub struct ForkedSession {
    pub id: String,
    pub ctx: TxContext,
    pub state: ReconstructedState,
    pub baseline_trace: Trace,
    pub mutations: Vec<(Pubkey, AccountMutation)>,
    pub latest_trace: Option<Trace>,
    pub created_at: std::time::Instant,
}

impl ForkedSession {
    /// Create a new session. Runs the baseline replay to capture the
    /// un-mutated trace — this is the reference point for future diffs.
    pub async fn new(
        ctx: TxContext,
        state: ReconstructedState,
    ) -> Result<Self, ReplayError> {
        let id = ulid::Ulid::new().to_string();

        // Run the baseline replay.
        let execution = replay_from_scratch(&state, &ctx, &[])?;
        let idl_cache = IdlCache::default();
        let decoder = AccountDecoder::new(&idl_cache);
        let baseline_trace = build_trace(&ctx, &execution, &decoder).await;

        Ok(Self {
            id,
            ctx,
            state,
            baseline_trace,
            mutations: Vec::new(),
            latest_trace: None,
            created_at: std::time::Instant::now(),
        })
    }

    /// Append a mutation to the session's mutation log. Does NOT re-execute.
    pub fn mutate(
        &mut self,
        pubkey: Pubkey,
        mutation: AccountMutation,
    ) -> Result<(), ReplayError> {
        // For v1 we don't pre-validate mutations against the IDL here; the
        // apply-logic during execute() handles errors. A future version
        // should return preview info (byte-level diff) synchronously.
        self.mutations.push((pubkey, mutation));
        Ok(())
    }

    /// Clear all applied mutations. Next execute() will match baseline.
    pub fn reset(&mut self) {
        self.mutations.clear();
        self.latest_trace = None;
    }

    /// Execute the transaction with the current mutations applied on top of
    /// reconstructed state. Caches the result in `latest_trace`.
    pub async fn execute<C: HeliusClient>(
        &mut self,
        client: &C,
    ) -> Result<Trace, ReplayError> {
        let resolved = self.resolve_mutations(client).await?;
        let execution = replay_from_scratch(&self.state, &self.ctx, &resolved)?;

        let idl_cache = IdlCache::default();
        let decoder = AccountDecoder::new(&idl_cache);
        let trace = build_trace(&self.ctx, &execution, &decoder).await;

        self.latest_trace = Some(trace.clone());
        Ok(trace)
    }

    /// Produce a diff between baseline and the most recent execution.
    pub fn diff(&self) -> Option<TraceDiff> {
        let latest = self.latest_trace.clone()?;
        let result_changed = self.baseline_trace.replay_result != latest.replay_result;

        let mut changed_accounts: Vec<String> = Vec::new();
        let baseline_deltas: HashMap<_, _> = self
            .baseline_trace
            .account_deltas
            .iter()
            .map(|d| (d.pubkey.clone(), d))
            .collect();
        let latest_deltas: HashMap<_, _> = latest
            .account_deltas
            .iter()
            .map(|d| (d.pubkey.clone(), d))
            .collect();
        for key in latest_deltas.keys() {
            match baseline_deltas.get(key) {
                None => changed_accounts.push(key.clone()),
                Some(b) => {
                    if b.data_after_hex != latest_deltas[key].data_after_hex
                        || b.lamports_after != latest_deltas[key].lamports_after
                    {
                        changed_accounts.push(key.clone());
                    }
                }
            }
        }

        Some(TraceDiff {
            baseline: self.baseline_trace.clone(),
            total_cu_delta: latest.total_cu as i64 - self.baseline_trace.total_cu as i64,
            result_changed,
            changed_accounts,
            latest,
        })
    }

    /// Resolve the mutation log into concrete Account overrides.
    async fn resolve_mutations<C: HeliusClient>(
        &self,
        _client: &C,
    ) -> Result<Vec<(Pubkey, Account)>, ReplayError> {
        let idl_cache = IdlCache::default();
        let mut out: HashMap<Pubkey, Account> = HashMap::new();

        for (pk, mutation) in &self.mutations {
            let base = out
                .get(pk)
                .cloned()
                .or_else(|| self.state.accounts.get(pk).cloned())
                .ok_or_else(|| ReplayError::StateReconstruction {
                    step: "mutation_target_missing".into(),
                    detail: format!("account {pk} not in reconstructed state"),
                })?;

            let mutated = apply_mutation(base, mutation, &idl_cache)?;
            out.insert(*pk, mutated);
        }

        Ok(out.into_iter().collect())
    }
}

fn apply_mutation(
    mut account: Account,
    mutation: &AccountMutation,
    idl_cache: &IdlCache,
) -> Result<Account, ReplayError> {
    match mutation {
        AccountMutation::Lamports { new_value } => {
            account.lamports = *new_value;
        }
        AccountMutation::Owner { new_value } => {
            account.owner = new_value
                .parse()
                .map_err(|e| ReplayError::Decoder(format!("bad owner pubkey: {e:?}")))?;
        }
        AccountMutation::RawBytes { offset, bytes, extend } => {
            let required = *offset + bytes.len();
            if required > account.data.len() {
                if *extend {
                    account.data.resize(required, 0);
                } else {
                    return Err(ReplayError::Decoder(format!(
                        "raw_bytes mutation exceeds account data length ({}+{} > {})",
                        offset,
                        bytes.len(),
                        account.data.len()
                    )));
                }
            }
            account.data[*offset..*offset + bytes.len()].copy_from_slice(bytes);
        }
        AccountMutation::Field { path, new_value } => {
            let idl = idl_cache
                .get_local(&account.owner)
                .ok_or_else(|| ReplayError::InvalidMutationPath {
                    path: path.clone(),
                    type_name: format!("no IDL for owner {}", account.owner),
                })?;
            account.data =
                crate::idl::apply_field_mutation(&idl, &account.data, path, new_value)?;
        }
    }
    Ok(account)
}
