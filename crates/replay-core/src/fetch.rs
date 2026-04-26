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
    use crate::test_support::MockHeliusClient;
    use crate::types::{FetchedTx, FetchedTxMeta, LoadedAddresses};
    use base64::Engine;
    use solana_sdk::{
        hash::Hash,
        message::{v0, MessageHeader, VersionedMessage},
        signature::Signature as SdkSignature,
    };

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

    /// Build a synthetic V0 transaction that exercises the three things
    /// fetch_full_tx_context has to get right:
    /// - LUT resolution (loaded_addresses concatenated after static keys)
    /// - compute-budget instruction extraction
    /// - mainnet result mapping (Success/Failure)
    fn synth_v0_tx() -> (VersionedTransaction, Vec<Pubkey>, Vec<Pubkey>, Vec<Pubkey>) {
        let payer = Pubkey::new_unique();
        let cb_program = compute_budget::id();
        let target_program = Pubkey::new_unique();

        // Static keys, in the order the runtime indexes them.
        let static_keys = vec![payer, cb_program, target_program];

        // LUT-loaded keys we'll splice in via meta.loaded_addresses.
        let loaded_writable = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let loaded_readonly = vec![Pubkey::new_unique()];

        // SetComputeUnitLimit(400_000) — discriminator 2, then little-endian u32.
        let cb_data = {
            let mut v = vec![2u8];
            v.extend_from_slice(&400_000u32.to_le_bytes());
            v
        };
        let cb_ix = CompiledInstruction {
            program_id_index: 1,
            accounts: vec![],
            data: cb_data,
        };
        let user_ix = CompiledInstruction {
            program_id_index: 2,
            accounts: vec![0],
            data: vec![0xAA],
        };

        let header = MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 2,
        };
        let msg = v0::Message {
            header,
            account_keys: static_keys.clone(),
            recent_blockhash: Hash::default(),
            instructions: vec![cb_ix, user_ix],
            address_table_lookups: vec![],
        };

        let tx = VersionedTransaction {
            signatures: vec![SdkSignature::default()],
            message: VersionedMessage::V0(msg),
        };

        (tx, static_keys, loaded_writable, loaded_readonly)
    }

    fn fetched_tx_for(
        tx: &VersionedTransaction,
        loaded_writable: &[Pubkey],
        loaded_readonly: &[Pubkey],
        err: Option<serde_json::Value>,
    ) -> FetchedTx {
        let bytes = bincode::serialize(tx).unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

        FetchedTx {
            slot: 240_000_000,
            block_time: Some(1_700_000_000),
            transaction_base64: b64,
            meta: FetchedTxMeta {
                err,
                log_messages: vec![
                    "Program ComputeBudget111111111111111111111111111111 invoke [1]".to_string(),
                    "Program ComputeBudget111111111111111111111111111111 success".to_string(),
                ],
                pre_balances: vec![10_000_000_000, 1, 1],
                post_balances: vec![9_999_995_000, 1, 1],
                loaded_addresses: Some(LoadedAddresses {
                    writable: loaded_writable.iter().map(|p| p.to_string()).collect(),
                    readonly: loaded_readonly.iter().map(|p| p.to_string()).collect(),
                }),
                compute_units_consumed: Some(150),
                inner_instructions: None,
            },
        }
    }

    #[tokio::test]
    async fn fetch_full_tx_context_resolves_luts_and_extracts_compute_budget() {
        let (tx, static_keys, loaded_w, loaded_r) = synth_v0_tx();
        let sig = Signature::new_unique();
        let fetched = fetched_tx_for(&tx, &loaded_w, &loaded_r, None);

        let mut mock = MockHeliusClient::default();
        mock.txs.insert(sig.to_string(), fetched);

        let ctx = fetch_full_tx_context(&mock, &sig).await.unwrap();

        // Slot + block time threaded through.
        assert_eq!(ctx.slot, 240_000_000);
        assert_eq!(ctx.block_time, Some(1_700_000_000));

        // LUT-resolved keys are appended in [static, writable, readonly] order.
        let mut expected: Vec<Pubkey> = static_keys.clone();
        expected.extend(loaded_w.iter().copied());
        expected.extend(loaded_r.iter().copied());
        assert_eq!(ctx.resolved_account_keys, expected);
        assert_eq!(ctx.resolved_account_keys.len(), 6);

        // Compute-budget instruction extracted, exactly one.
        assert_eq!(ctx.compute_budget_instructions.len(), 1);
        assert_eq!(ctx.compute_budget_instructions[0].program_id_index, 1);

        // Mainnet result reflects success.
        assert!(matches!(ctx.mainnet_result, TxResult::Success));

        // Logs + balances passed through verbatim.
        assert_eq!(ctx.mainnet_logs.len(), 2);
        assert_eq!(ctx.pre_balances.len(), 3);
        assert_eq!(ctx.post_balances.len(), 3);
    }

    #[tokio::test]
    async fn fetch_full_tx_context_maps_failure() {
        let (tx, _, loaded_w, loaded_r) = synth_v0_tx();
        let sig = Signature::new_unique();
        let err = serde_json::json!({"InstructionError": [1, {"Custom": 6001}]});
        let fetched = fetched_tx_for(&tx, &loaded_w, &loaded_r, Some(err));

        let mut mock = MockHeliusClient::default();
        mock.txs.insert(sig.to_string(), fetched);

        let ctx = fetch_full_tx_context(&mock, &sig).await.unwrap();

        match ctx.mainnet_result {
            TxResult::Failure { error_code, .. } => assert_eq!(error_code, Some(6001)),
            TxResult::Success => panic!("expected failure"),
        }
    }

    #[tokio::test]
    async fn fetch_full_tx_context_returns_tx_not_found() {
        let mock = MockHeliusClient::default();
        let sig = Signature::new_unique();
        let err = fetch_full_tx_context(&mock, &sig).await.unwrap_err();
        assert!(matches!(err, ReplayError::TxNotFound), "got {err:?}");
    }

    #[tokio::test]
    async fn tx_context_serializes_to_json() {
        let (tx, _, loaded_w, loaded_r) = synth_v0_tx();
        let sig = Signature::new_unique();
        let mut mock = MockHeliusClient::default();
        mock.txs
            .insert(sig.to_string(), fetched_tx_for(&tx, &loaded_w, &loaded_r, None));

        let ctx = fetch_full_tx_context(&mock, &sig).await.unwrap();
        let json = serde_json::to_value(&ctx).expect("TxContext must serialize");
        assert!(json.is_object());
        assert_eq!(json["slot"], 240_000_000);
        assert!(json["resolved_account_keys"].is_array());
    }

    /// Live integration test against real Helius. Run with:
    ///   REPLAY_LIVE_TESTS=1 HELIUS_API_KEY=... cargo test -p replay-core -- --ignored live_
    /// Reads the demo signature from tests/fixtures/demo-signature.txt.
    #[tokio::test]
    #[ignore]
    async fn live_fetch_jupiter_swap() {
        if std::env::var("REPLAY_LIVE_TESTS").is_err() {
            eprintln!("REPLAY_LIVE_TESTS not set; skipping");
            return;
        }
        let api_key =
            std::env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY must be set for live tests");
        let sig_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo-signature.txt");
        let sig_str = std::fs::read_to_string(&sig_path)
            .expect("tests/fixtures/demo-signature.txt missing")
            .trim()
            .to_string();
        if sig_str.contains("PLACEHOLDER") {
            eprintln!(
                "demo-signature.txt is a placeholder; \
                 set it to a real Jupiter swap sig (see scripts/capture-fixture.sh) — skipping"
            );
            return;
        }
        let sig = Signature::from_str(&sig_str).expect("invalid demo signature");

        let client = crate::rpc::HeliusRpcClient::from_api_key(&api_key).unwrap();
        let ctx = fetch_full_tx_context(&client, &sig).await.unwrap();

        // Jupiter swaps always reference >10 accounts after LUT resolution.
        assert!(
            ctx.resolved_account_keys.len() > 10,
            "expected >10 resolved accounts for a Jupiter swap, got {}",
            ctx.resolved_account_keys.len()
        );
        assert!(!ctx.mainnet_logs.is_empty(), "expected non-empty mainnet logs");
    }
}
