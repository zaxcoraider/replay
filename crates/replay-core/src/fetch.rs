//! Fetch a transaction and turn the raw Helius response into a structured
//! `TxContext` ready for state reconstruction and replay.
//!
//! The hard parts handled here:
//! - Deserializing a `VersionedTransaction` from base64
//! - Resolving Address Lookup Tables (LUTs): concatenating
//!   `static_account_keys ++ loaded_writable ++ loaded_readonly`
//!   in the order the runtime uses.
//! - Splitting compute-budget instructions out from the regular ones so
//!   the replay can preserve them verbatim.
//! - Extracting the mainnet result (`meta.err`) into our typed `TxResult`.

use crate::error::ReplayError;
use crate::rpc::HeliusClient;
use crate::types::{TxContext, TxResult};
use base64::Engine;
use solana_sdk::{
    compute_budget,
    instruction::CompiledInstruction,
    pubkey::Pubkey,
    signature::Signature,
    transaction::VersionedTransaction,
};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, info};

#[tracing::instrument(skip(client), fields(signature = %signature))]
pub async fn fetch_full_tx_context<C: HeliusClient>(
    client: &C,
    signature: &Signature,
) -> Result<TxContext, ReplayError> {
    let fetched = client
        .get_transaction(signature)
        .await?
        .ok_or(ReplayError::TxNotFound)?;

    info!(slot = fetched.slot, "fetched transaction from rpc");

    // --- 1. Deserialize the versioned transaction --- //
    let tx_bytes = base64::engine::general_purpose::STANDARD
        .decode(&fetched.transaction_base64)
        .map_err(|e| ReplayError::StateReconstruction {
            step: "base64_decode_tx".into(),
            detail: e.to_string(),
        })?;

    let original_tx: VersionedTransaction = bincode::deserialize(&tx_bytes).map_err(|e| {
        ReplayError::StateReconstruction {
            step: "deserialize_versioned_tx".into(),
            detail: e.to_string(),
        }
    })?;

    // --- 2. Resolve account keys (static + LUT-loaded) --- //
    let static_keys = original_tx.message.static_account_keys().to_vec();
    debug!(count = static_keys.len(), "static account keys");

    let mut resolved: Vec<Pubkey> = static_keys.clone();

    if let Some(loaded) = fetched.meta.loaded_addresses.as_ref() {
        for s in &loaded.writable {
            let pk = Pubkey::from_str(s).map_err(|e| ReplayError::LutResolution {
                lut: s.clone(),
                detail: format!("invalid pubkey: {e}"),
            })?;
            resolved.push(pk);
        }
        for s in &loaded.readonly {
            let pk = Pubkey::from_str(s).map_err(|e| ReplayError::LutResolution {
                lut: s.clone(),
                detail: format!("invalid pubkey: {e}"),
            })?;
            resolved.push(pk);
        }
        info!(
            writable = loaded.writable.len(),
            readonly = loaded.readonly.len(),
            total = resolved.len(),
            "resolved LUT addresses"
        );
    }

    // --- 3. Split compute-budget instructions from the rest --- //
    let compute_budget_program_id = compute_budget::id();
    let mut cb_instructions: Vec<CompiledInstruction> = Vec::new();

    for ci in original_tx.message.instructions() {
        let program_idx = ci.program_id_index as usize;
        if program_idx >= resolved.len() {
            return Err(ReplayError::StateReconstruction {
                step: "compute_budget_extraction".into(),
                detail: format!(
                    "program_id_index {} out of bounds (resolved={})",
                    program_idx,
                    resolved.len()
                ),
            });
        }
        if resolved[program_idx] == compute_budget_program_id {
            cb_instructions.push(ci.clone());
        }
    }
    debug!(count = cb_instructions.len(), "compute budget instructions");

    // --- 4. Map the mainnet result --- //
    let mainnet_result = match &fetched.meta.err {
        None => TxResult::Success,
        Some(err_val) => TxResult::Failure {
            error: err_val.to_string(),
            error_code: extract_custom_error_code(err_val),
        },
    };

    // --- 5. Return the context --- //
    Ok(TxContext {
        signature: *signature,
        slot: fetched.slot,
        block_time: fetched.block_time,
        original_tx,
        resolved_account_keys: resolved,
        mainnet_logs: fetched.meta.log_messages.clone(),
        mainnet_result,
        compute_budget_instructions: cb_instructions,
        pre_balances: fetched.meta.pre_balances.clone(),
        post_balances: fetched.meta.post_balances.clone(),
        pre_account_snapshots: HashMap::new(), // populated by reconstruct.rs
    })
}

/// Attempt to extract a program-defined error code from an `InstructionError::Custom(N)`.
fn extract_custom_error_code(err: &serde_json::Value) -> Option<i64> {
    // Typical shape: {"InstructionError": [0, {"Custom": 6001}]}
    let ix_err = err.get("InstructionError")?.as_array()?;
    let inner = ix_err.get(1)?;
    inner.get("Custom").and_then(|v| v.as_i64())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_custom_error_code_parses_shape() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"InstructionError":[0,{"Custom":6001}]}"#).unwrap();
        assert_eq!(extract_custom_error_code(&v), Some(6001));
    }

    #[test]
    fn extract_custom_error_code_handles_non_custom() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"InsufficientFundsForRent":{"account_index":0}}"#).unwrap();
        assert_eq!(extract_custom_error_code(&v), None);
    }
}
