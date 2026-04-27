//! The litesvm wrapper. Responsible for seeding reconstructed state,
//! setting sysvars, and running the replay transaction.
//!
//! Key choices:
//! - `with_sigverify(false)` so we don't need real signer keys.
//! - `with_blockhash_check(false)` so expired blockhashes don't block us.
//! - Clock sysvar set from the target slot's block time.

use crate::error::ReplayError;
use crate::types::{ReconstructedState, TxContext, TxResult};
use litesvm::LiteSVM;
use solana_sdk::{
    account::Account,
    clock::Clock,
    pubkey::Pubkey,
};
use std::collections::HashMap;
use tracing::{debug, info};

pub struct SvmRunner {
    svm: LiteSVM,
}

pub struct ExecutionResult {
    pub logs: Vec<String>,
    pub result: TxResult,
    pub cu_consumed: u64,
    pub accounts_after: HashMap<Pubkey, Account>,
    pub return_data: Option<Vec<u8>>,
}

impl Default for SvmRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl SvmRunner {
    pub fn new() -> Self {
        let svm = LiteSVM::new()
            .with_sigverify(false)
            .with_blockhash_check(false);
        Self { svm }
    }

    /// Seed the SVM with reconstructed state. Called once per session.
    ///
    /// Order matters: program-data accounts are seeded BEFORE their owning
    /// program accounts. litesvm's `set_account` eagerly links upgradeable
    /// programs against their `programdata_address` and rejects with
    /// `Instruction(MissingAccount)` if the program-data isn't already in
    /// the SVM. We do non-program accounts + all program-data first, then
    /// the program stubs last.
    #[tracing::instrument(skip(self, state))]
    pub fn seed(&mut self, state: &ReconstructedState) -> Result<(), ReplayError> {
        // Pass 1: every non-program account.
        for (pubkey, account) in &state.accounts {
            self.svm
                .set_account(*pubkey, account.clone())
                .map_err(|e| ReplayError::Execution(format!("set_account {pubkey}: {e:?}")))?;
        }

        // Pass 2: program-data accounts (the bytecode side of upgradeable
        // programs). Must precede the program account itself.
        for (program_id, info) in &state.programs {
            if let (Some(pda), Some(data_acc)) =
                (info.program_data_address, &info.program_data_account)
            {
                self.svm.set_account(pda, data_acc.clone()).map_err(|e| {
                    ReplayError::Execution(format!(
                        "set_account program_data {pda} (for program {program_id}): {e:?}"
                    ))
                })?;
            }
        }

        // Pass 3: program accounts. With program-data already present, the
        // upgradeable loader linkage resolves cleanly.
        for (program_id, info) in &state.programs {
            self.svm
                .set_account(*program_id, info.program_account.clone())
                .map_err(|e| {
                    ReplayError::Execution(format!("set_account program {program_id}: {e:?}"))
                })?;
        }

        debug!(
            accounts = state.accounts.len(),
            programs = state.programs.len(),
            "svm seeded"
        );
        Ok(())
    }

    /// Apply a fresh clock matching the target tx's slot + block time.
    pub fn set_clock_for_slot(&mut self, slot: u64, block_time: Option<i64>) {
        let unix_timestamp = block_time.unwrap_or(0);
        let clock = Clock {
            slot,
            epoch_start_timestamp: unix_timestamp,
            epoch: slot / 432_000,
            leader_schedule_epoch: slot / 432_000,
            unix_timestamp,
        };
        self.svm.set_sysvar::<Clock>(&clock);
    }

    /// Execute the reconstructed transaction. Returns logs and post-state.
    #[tracing::instrument(skip(self, ctx), fields(slot = ctx.slot))]
    pub fn execute(&mut self, ctx: &TxContext) -> Result<ExecutionResult, ReplayError> {
        // NOTE on signature: with `sigverify(false)`, litesvm accepts a tx
        // whose signatures don't match. We pass the original versioned tx
        // directly — no need to resign or swap the fee payer for v1.
        let tx = ctx.original_tx.clone();

        // simulate_transaction runs the tx but doesn't commit. For v1 we
        // prefer send_transaction so account state updates are visible
        // afterward via get_account.
        let result = self.svm.send_transaction(tx);

        match result {
            Ok(meta) => {
                let logs = meta.logs.clone();
                let cu_consumed = meta.compute_units_consumed;
                let return_data = if meta.return_data.data.is_empty() {
                    None
                } else {
                    Some(meta.return_data.data.clone())
                };

                // Gather post-state for every key the tx touched.
                let mut accounts_after = HashMap::new();
                for pk in &ctx.resolved_account_keys {
                    if let Some(acc) = self.svm.get_account(pk) {
                        accounts_after.insert(*pk, acc);
                    }
                }

                info!(cu = cu_consumed, "replay succeeded");
                Ok(ExecutionResult {
                    logs,
                    result: TxResult::Success,
                    cu_consumed,
                    accounts_after,
                    return_data,
                })
            }
            Err(failed) => {
                // litesvm returns structured failure info; we convert to our
                // TxResult but still capture logs from the meta. A failing
                // tx is NOT a tool error — it's often the correct outcome.
                let logs = failed.meta.logs.clone();
                let cu_consumed = failed.meta.compute_units_consumed;

                let mut accounts_after = HashMap::new();
                for pk in &ctx.resolved_account_keys {
                    if let Some(acc) = self.svm.get_account(pk) {
                        accounts_after.insert(*pk, acc);
                    }
                }

                let err_string = format!("{:?}", failed.err);
                info!(err = %err_string, "replay completed with failure (expected)");

                Ok(ExecutionResult {
                    logs,
                    result: TxResult::Failure {
                        error: err_string,
                        error_code: None,
                    },
                    cu_consumed,
                    accounts_after,
                    return_data: None,
                })
            }
        }
    }

    /// Escape hatch: apply an arbitrary account override on an already-seeded
    /// SVM. Used by `ForkedSession::apply_mutation`.
    pub fn override_account(
        &mut self,
        pubkey: Pubkey,
        account: Account,
    ) -> Result<(), ReplayError> {
        self.svm
            .set_account(pubkey, account)
            .map_err(|e| ReplayError::Execution(format!("override_account {pubkey}: {e:?}")))
    }

    /// Reset the SVM to a fresh state. Used between re-runs in a session.
    pub fn reset(&mut self) {
        self.svm = LiteSVM::new()
            .with_sigverify(false)
            .with_blockhash_check(false);
    }

    /// Raw account read (for mutation application logic).
    pub fn get_account(&self, pubkey: &Pubkey) -> Option<Account> {
        self.svm.get_account(pubkey)
    }

    /// Re-expose for advanced callers. Keep narrow.
    #[doc(hidden)]
    pub fn inner_mut(&mut self) -> &mut LiteSVM {
        &mut self.svm
    }
}

/// A replay trivially re-runnable: wipe SVM, re-seed, re-apply clock, re-execute.
pub fn replay_from_scratch(
    state: &ReconstructedState,
    ctx: &TxContext,
    mutations: &[(Pubkey, Account)],
) -> Result<ExecutionResult, ReplayError> {
    let mut runner = SvmRunner::new();
    runner.seed(state)?;

    for (pk, acct) in mutations {
        runner.override_account(*pk, acct.clone())?;
    }

    runner.set_clock_for_slot(ctx.slot, ctx.block_time);
    runner.execute(ctx)
}
