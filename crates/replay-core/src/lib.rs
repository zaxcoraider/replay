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

#[cfg(test)]
pub(crate) mod test_support;

pub use error::ReplayError;
pub use types::{
    AccountDelta, AccountMutation, CpiFrame, FetchedTx, FrameAccount, LogDivergence,
    ProgramInfo, ProgramLoader, ReconstructedState, Trace, TraceDiff, TxContext, TxResult,
};
pub use rpc::{HeliusClient, HeliusRpcClient};
pub use session::ForkedSession;

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

    let ctx = fetch::fetch_full_tx_context(client, &sig).await?;
    let state = reconstruct::reconstruct_state(client, &ctx).await?;

    let mut runner = svm::SvmRunner::new();
    runner.seed(&state)?;
    runner.set_clock_for_slot(ctx.slot, ctx.block_time);

    let execution = runner.execute(&ctx)?;

    let idl_cache = idl::IdlCache::default();
    let decoder = idl::AccountDecoder::new(&idl_cache);

    Ok(trace::build_trace(&ctx, &execution, &decoder).await)
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

    let ctx = fetch::fetch_full_tx_context(client, &sig).await?;
    let state = reconstruct::reconstruct_state(client, &ctx).await?;

    ForkedSession::new(ctx, state).await
}
