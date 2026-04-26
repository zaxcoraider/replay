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
use solana_sdk::{account::Account, bpf_loader, bpf_loader_deprecated, bpf_loader_upgradeable, native_loader, pubkey::Pubkey};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// LoaderV4 program ID (stable; hardcoded so we don't depend on the newer
/// SDK constant that might not be in every release).
const LOADER_V4_ID: Pubkey = solana_sdk::pubkey!("LoaderV411111111111111111111111111111111111");

#[tracing::instrument(skip(client, ctx), fields(slot = ctx.slot))]
pub async fn reconstruct_state<C: HeliusClient>(
    client: &C,
    ctx: &TxContext,
) -> Result<ReconstructedState, ReplayError> {
    let mut accounts: HashMap<Pubkey, Account> = HashMap::new();
    let mut programs: HashMap<Pubkey, ProgramInfo> = HashMap::new();

    // Collect program ids referenced in the instructions.
    let mut program_ids: Vec<Pubkey> = Vec::new();
    for ci in ctx.original_tx.message.instructions() {
        let idx = ci.program_id_index as usize;
        if idx < ctx.resolved_account_keys.len() {
            let pid = ctx.resolved_account_keys[idx];
            if !program_ids.contains(&pid) {
                program_ids.push(pid);
            }
        }
    }
    info!(program_count = program_ids.len(), "programs referenced");

    // --- 1. Fetch all non-program accounts at the historical slot --- //
    let fetch_slot = ctx.slot.saturating_sub(1);
    for pubkey in &ctx.resolved_account_keys {
        if program_ids.contains(pubkey) {
            continue; // handled separately
        }
        match client.get_account_info_at_slot(pubkey, fetch_slot).await? {
            Some(account) => {
                accounts.insert(*pubkey, account);
            }
            None => {
                // An account being absent at pre-slot is legitimate — the tx
                // may create it. Note it but don't fail.
                debug!(?pubkey, "account absent at pre-slot (may be created by tx)");
            }
        }
    }
    info!(account_count = accounts.len(), "reconstructed non-program accounts");

    // --- 2. Fetch each program's bytecode at the historical slot --- //
    for program_id in &program_ids {
        let info = fetch_program_info(client, program_id, fetch_slot).await?;
        if let Some(i) = info {
            programs.insert(*program_id, i);
        }
    }
    info!(program_count = programs.len(), "reconstructed programs");

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

async fn fetch_program_info<C: HeliusClient>(
    client: &C,
    program_id: &Pubkey,
    slot: u64,
) -> Result<Option<ProgramInfo>, ReplayError> {
    let Some(program_account) = client.get_account_info_at_slot(program_id, slot).await? else {
        return Err(ReplayError::MissingProgramBytecode {
            program_id: program_id.to_string(),
            slot,
        });
    };

    let loader = classify_loader(&program_account.owner);

    match loader {
        // Native programs: litesvm has them built in; nothing to fetch.
        ProgramLoader::Native => {
            debug!(?program_id, "native program; skipping bytecode fetch");
            Ok(None)
        }

        // Legacy BPF loader: bytecode lives in the program account's data.
        ProgramLoader::BpfLoader | ProgramLoader::BpfLoaderDeprecated => {
            Ok(Some(ProgramInfo {
                program_account,
                program_data_address: None,
                program_data_account: None,
                loader,
            }))
        }

        // Upgradeable loader: parse the program-data pointer and fetch that.
        ProgramLoader::BpfLoaderUpgradeable => {
            let data = &program_account.data;
            // Layout: enum tag (4 bytes) + pubkey (32 bytes)
            if data.len() < 36 {
                return Err(ReplayError::StateReconstruction {
                    step: "parse_program_data_address".into(),
                    detail: format!(
                        "upgradeable program account too short: len={}",
                        data.len()
                    ),
                });
            }
            let program_data_address = Pubkey::try_from(&data[4..36]).map_err(|e| {
                ReplayError::StateReconstruction {
                    step: "parse_program_data_address".into(),
                    detail: format!("pubkey parse: {e:?}"),
                }
            })?;

            let program_data_account = client
                .get_account_info_at_slot(&program_data_address, slot)
                .await?
                .ok_or_else(|| ReplayError::MissingProgramBytecode {
                    program_id: program_data_address.to_string(),
                    slot,
                })?;

            Ok(Some(ProgramInfo {
                program_account,
                program_data_address: Some(program_data_address),
                program_data_account: Some(program_data_account),
                loader,
            }))
        }

        // LoaderV4: bytecode lives after a header in the program account
        // itself. Handled at seed time by litesvm.
        ProgramLoader::LoaderV4 => {
            warn!(?program_id, "LoaderV4 support is experimental — verify replay fidelity");
            Ok(Some(ProgramInfo {
                program_account,
                program_data_address: None,
                program_data_account: None,
                loader,
            }))
        }
    }
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
