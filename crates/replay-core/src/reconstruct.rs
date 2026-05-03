//! Reconstruct the pre-slot state of every account a transaction touches.
//!
//! This is the heart of faithful replay. For each referenced account, we
//! fetch its state as close to `slot - 1` as we can, and for each invoked
//! program, we fetch the exact bytecode that was live at that slot (handling
//! the upgradeable BPF loader's two-account layout).
//!
//! See `docs/04-solana-gotchas.md` items #1, #8, and #12 for the why.

use crate::error::ReplayError;
use crate::rpc::HeliusClient;
use crate::types::{ProgramInfo, ProgramLoader, ReconstructedState, TxContext};
use solana_sdk::{
    account::Account,
    bpf_loader,
    bpf_loader_deprecated,
    bpf_loader_upgradeable,
    message::VersionedMessage,
    native_loader,
    pubkey::Pubkey,
};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// LoaderV4 program ID (stable; hardcoded so we don't depend on the newer
/// SDK constant that might not be in every release).
const LOADER_V4_ID: Pubkey = solana_sdk::pubkey!("LoaderV411111111111111111111111111111111111");

/// Progress signal emitted by [`reconstruct_state_with_progress`] as it
/// fetches each account. The live-replay SSE endpoint maps these into
/// `account_fetched` wire events; the standard `reconstruct_state` wrapper
/// passes a no-op closure and discards them.
#[derive(Debug, Clone)]
pub enum ReconstructProgress {
    AccountFetched {
        pubkey: Pubkey,
        size: usize,
        is_program: bool,
    },
}

#[tracing::instrument(skip(client, ctx), fields(slot = ctx.slot))]
pub async fn reconstruct_state<C: HeliusClient>(
    client: &C,
    ctx: &TxContext,
) -> Result<ReconstructedState, ReplayError> {
    reconstruct_state_with_progress(client, ctx, |_| {}).await
}

/// Same as [`reconstruct_state`] but invokes `on_progress` once for every
/// account the reconstruction touches (pre-slot fetches, ALT fetches, and
/// program-data fetches for upgradeable programs).
///
/// The callback is synchronous and `Send`; pushing into a `tokio::sync::mpsc`
/// sender via `try_send` (or queuing into a `VecDeque` for batched dispatch)
/// is the expected usage. We deliberately don't make it async to keep the
/// lifetime story simple — the SSE handler is the only caller and it is
/// happy to drop events if the receiver is gone.
#[tracing::instrument(skip(client, ctx, on_progress), fields(slot = ctx.slot))]
pub async fn reconstruct_state_with_progress<C, F>(
    client: &C,
    ctx: &TxContext,
    mut on_progress: F,
) -> Result<ReconstructedState, ReplayError>
where
    C: HeliusClient,
    F: FnMut(ReconstructProgress) + Send,
{
    let fetch_slot = ctx.slot.saturating_sub(1);
    let mut accounts: HashMap<Pubkey, Account> = HashMap::new();
    let mut programs: HashMap<Pubkey, ProgramInfo> = HashMap::new();

    // --- Pass 1: fetch every resolved account at slot-1 --- //
    //
    // Don't pre-classify by walking instructions — that misses programs
    // invoked via CPI (Token-2022 from a Jupiter swap is the canonical
    // example: it's never a top-level instruction's program_id_index but
    // appears in resolved_account_keys because the runtime needs it
    // accessible to satisfy CPI account constraints). Post-classification
    // by `account.executable && owner == loader` catches all of them.
    for pubkey in &ctx.resolved_account_keys {
        match client.get_account_info_at_slot(pubkey, fetch_slot).await? {
            Some(account) => {
                on_progress(ReconstructProgress::AccountFetched {
                    pubkey: *pubkey,
                    size: account.data.len(),
                    is_program: account.executable,
                });
                accounts.insert(*pubkey, account);
            }
            None => {
                // An account being absent at pre-slot is legitimate — the tx
                // may create it. Note it but don't fail.
                debug!(?pubkey, "account absent at pre-slot (may be created by tx)");
            }
        }
    }
    info!(fetched = accounts.len(), "fetched accounts at slot-1");

    // --- Pass 1b: fetch every Address Lookup Table account itself --- //
    //
    // resolved_account_keys covers what the LUTs *resolve to*; it does NOT
    // include the ALT accounts that hold those addresses. litesvm's V0-message
    // path resolves lookups against its own account store at execute time,
    // so each `address_table_lookups[].account_key` must be seeded too. The
    // spike (spikes/spike-replay.rs) catches this with `needed.insert(account_key)`.
    if let VersionedMessage::V0(v0) = &ctx.original_tx.message {
        for atl in &v0.address_table_lookups {
            if accounts.contains_key(&atl.account_key) {
                continue;
            }
            match client
                .get_account_info_at_slot(&atl.account_key, fetch_slot)
                .await?
            {
                Some(account) => {
                    on_progress(ReconstructProgress::AccountFetched {
                        pubkey: atl.account_key,
                        size: account.data.len(),
                        is_program: false,
                    });
                    accounts.insert(atl.account_key, account);
                }
                None => {
                    return Err(ReplayError::LutResolution {
                        lut: atl.account_key.to_string(),
                        detail: format!("ALT account not found at slot {fetch_slot}"),
                    });
                }
            }
        }
        if !v0.address_table_lookups.is_empty() {
            info!(
                lut_count = v0.address_table_lookups.len(),
                "fetched LUT lookup tables"
            );
        }
    }

    // --- Pass 2: classify each fetched account; pull program-data when
    //             upgradeable; route programs to their own map. --- //
    let candidates: Vec<(Pubkey, Account)> = accounts
        .iter()
        .filter(|(_, a)| a.executable)
        .map(|(k, v)| (*k, v.clone()))
        .collect();

    for (program_id, program_account) in candidates {
        let loader = classify_loader(&program_account.owner);

        // Native programs are built into litesvm; remove from accounts and
        // don't insert into programs (litesvm rejects overrides on builtins).
        if matches!(loader, ProgramLoader::Native) {
            accounts.remove(&program_id);
            continue;
        }

        // Move from accounts → programs
        accounts.remove(&program_id);

        let info = match loader {
            ProgramLoader::BpfLoader | ProgramLoader::BpfLoaderDeprecated => ProgramInfo {
                program_account,
                program_data_address: None,
                program_data_account: None,
                loader,
            },

            ProgramLoader::BpfLoaderUpgradeable => {
                let data = &program_account.data;
                if data.len() < 36 {
                    return Err(ReplayError::StateReconstruction {
                        step: "parse_program_data_address".into(),
                        detail: format!(
                            "upgradeable program {} account too short: len={}",
                            program_id,
                            data.len()
                        ),
                    });
                }
                let pda = Pubkey::try_from(&data[4..36]).map_err(|e| {
                    ReplayError::StateReconstruction {
                        step: "parse_program_data_address".into(),
                        detail: format!("pubkey parse for {program_id}: {e:?}"),
                    }
                })?;
                let pda_account = client
                    .get_account_info_at_slot(&pda, fetch_slot)
                    .await?
                    .ok_or_else(|| ReplayError::MissingProgramBytecode {
                        program_id: pda.to_string(),
                        slot: fetch_slot,
                    })?;
                on_progress(ReconstructProgress::AccountFetched {
                    pubkey: pda,
                    size: pda_account.data.len(),
                    is_program: true,
                });
                ProgramInfo {
                    program_account,
                    program_data_address: Some(pda),
                    program_data_account: Some(pda_account),
                    loader,
                }
            }

            ProgramLoader::LoaderV4 => {
                warn!(?program_id, "LoaderV4 support is experimental — verify replay fidelity");
                ProgramInfo {
                    program_account,
                    program_data_address: None,
                    program_data_account: None,
                    loader,
                }
            }

            ProgramLoader::Native => unreachable!("filtered out above"),
        };

        programs.insert(program_id, info);
    }

    info!(
        accounts = accounts.len(),
        programs = programs.len(),
        "reconstructed state"
    );

    Ok(ReconstructedState { accounts, programs })
}

/// Flatten reconstructed state into a single `Pubkey -> Account` map, including
/// program accounts and (for upgradeable programs) their program-data accounts.
///
/// Used to populate `TxContext::pre_account_snapshots` after reconstruction so
/// `trace::build_deltas` can produce non-empty `account_deltas` without
/// changing the prompt-mandated `&TxContext` signature on `reconstruct_state`.
pub fn snapshot_pre_state(state: &ReconstructedState) -> HashMap<Pubkey, Account> {
    let mut map = state.accounts.clone();
    for (program_id, info) in &state.programs {
        map.insert(*program_id, info.program_account.clone());
        if let (Some(pda), Some(da)) =
            (info.program_data_address, info.program_data_account.as_ref())
        {
            map.insert(pda, da.clone());
        }
    }
    map
}

fn classify_loader(owner: &Pubkey) -> ProgramLoader {
    if *owner == native_loader::id() {
        ProgramLoader::Native
    } else if *owner == bpf_loader::id() {
        ProgramLoader::BpfLoader
    } else if *owner == bpf_loader_deprecated::id() {
        ProgramLoader::BpfLoaderDeprecated
    } else if *owner == bpf_loader_upgradeable::id() {
        ProgramLoader::BpfLoaderUpgradeable
    } else if *owner == LOADER_V4_ID {
        ProgramLoader::LoaderV4
    } else {
        // Unknown owner for a "program" — probably a data account mis-classified
        // as a program by the caller. Treat as non-native and let the SVM reject.
        ProgramLoader::BpfLoader
    }
}
