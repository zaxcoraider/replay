//! # replay-core
//!
//! The engine for Replay — a time-travel debugger for Solana transactions.
//!
//! The public surface is intentionally small:
//! - [`replay`] — one-shot replay of a mainnet transaction.
//! - [`fork`] — create a session you can mutate and re-execute.
//!
//! Everything else is a private implementation detail. If you find yourself
//! reaching for internals from outside this crate, that's a signal to widen
//! the public API here rather than bypass it.

pub mod error;
pub mod types;
pub mod rpc;
pub mod fetch;
pub mod reconstruct;
pub mod svm;
pub mod idl;
pub mod trace;
pub mod session;
pub mod laserstream;

#[cfg(test)]
pub(crate) mod test_support;

pub use error::ReplayError;
pub use types::{
    AccountDelta, AccountMutation, CpiFrame, FetchedTx, FrameAccount, LogDivergence,
    ProgramInfo, ProgramLoader, ReconstructedState, Trace, TraceDiff, TxContext, TxResult,
};
pub use rpc::{HeliusClient, HeliusRpcClient};
pub use session::ForkedSession;
pub use laserstream::{
    connect as laserstream_connect, LaserStreamConfig, LaserStreamStatus,
    LiveReplayEvent, LiveSource,
};

use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;

/// Replay a mainnet transaction against litesvm with historical state.
///
/// Equivalent to [`fork`] followed by [`ForkedSession::execute`], but without
/// retaining the session. Use this when you just want to inspect what
/// happened; use [`fork`] when you want to mutate state and ask what-if.
pub async fn replay<C: HeliusClient>(
    signature: &str,
    client: &C,
) -> Result<Trace, ReplayError> {
    let sig = Signature::from_str(signature)
        .map_err(|_| ReplayError::InvalidSignature(signature.to_string()))?;

    let mut ctx = fetch::fetch_full_tx_context(client, &sig).await?;
    let state = reconstruct::reconstruct_state(client, &ctx).await?;
    ctx.pre_account_snapshots = reconstruct::snapshot_pre_state(&state);

    let mut runner = svm::SvmRunner::new();
    runner.seed(&state)?;
    runner.set_clock_for_slot(ctx.slot, ctx.block_time);

    let execution = runner.execute(&ctx)?;

    let idl_cache = idl::IdlCache::default();
    let decoder = idl::AccountDecoder::new(&idl_cache);

    let mut trace = trace::build_trace(&ctx, &execution, &decoder).await;
    decode_account_deltas(&mut trace, &ctx, &execution, &decoder, client).await;

    Ok(trace)
}

/// Walk `trace.account_deltas` and populate `decoded_before` / `decoded_after`
/// (and `idl_type_name` when available) by running the decoder against each
/// pre/post snapshot. Idempotent and side-effect-free outside the trace.
pub async fn decode_account_deltas<C: HeliusClient>(
    trace: &mut Trace,
    ctx: &TxContext,
    execution: &svm::ExecutionResult,
    decoder: &idl::AccountDecoder<'_>,
    client: &C,
) {
    for delta in &mut trace.account_deltas {
        let Ok(pubkey) = Pubkey::from_str(&delta.pubkey) else {
            continue;
        };
        if let Some(before) = ctx.pre_account_snapshots.get(&pubkey) {
            let dec = decoder.decode(&pubkey, before, client).await;
            if let idl::DecodedAccount::Decoded { type_name, .. }
            | idl::DecodedAccount::Native { type_name, .. } = &dec
            {
                delta.idl_type_name.get_or_insert_with(|| type_name.clone());
            }
            delta.decoded_before = serde_json::to_value(&dec).ok();
        }
        if let Some(after) = execution.accounts_after.get(&pubkey) {
            let dec = decoder.decode(&pubkey, after, client).await;
            if let idl::DecodedAccount::Decoded { type_name, .. }
            | idl::DecodedAccount::Native { type_name, .. } = &dec
            {
                delta.idl_type_name.get_or_insert_with(|| type_name.clone());
            }
            delta.decoded_after = serde_json::to_value(&dec).ok();
        }
    }
}

/// Create a forked session seeded from a mainnet transaction.
///
/// The session holds all reconstructed state in-memory. Mutations apply
/// on top; [`ForkedSession::execute`] re-runs the transaction against the
/// mutated state. A baseline trace is captured at fork time so you can
/// diff any subsequent run against the original outcome.
pub async fn fork<C: HeliusClient>(
    signature: &str,
    client: &C,
) -> Result<ForkedSession, ReplayError> {
    let sig = Signature::from_str(signature)
        .map_err(|_| ReplayError::InvalidSignature(signature.to_string()))?;

    let mut ctx = fetch::fetch_full_tx_context(client, &sig).await?;
    let state = reconstruct::reconstruct_state(client, &ctx).await?;
    ctx.pre_account_snapshots = reconstruct::snapshot_pre_state(&state);

    ForkedSession::new(ctx, state).await
}
