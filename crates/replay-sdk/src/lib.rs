//! # replay-sdk
//!
//! The stable, narrow Rust SDK for Replay. This is the recommended entry
//! point for embedding Replay in your own tooling — it re-exports the
//! `replay-core` public surface with no extra cruft.
//!
//! For programs that need the full engine (custom decoders, raw SVM
//! access), depend on `replay-core` directly.

pub use replay_core::{
    fork, replay, AccountDelta, AccountMutation, CpiFrame, ForkedSession, FrameAccount,
    HeliusClient, HeliusRpcClient, LogDivergence, ProgramInfo, ProgramLoader,
    ReconstructedState, ReplayError, Trace, TraceDiff, TxContext, TxResult,
};

/// Convenience constructor: build a `HeliusRpcClient` from `HELIUS_API_KEY`
/// or fall back to a custom URL via the `REPLAY_RPC_URL` environment
/// variable. Useful when the SDK is consumed as a library inside another
/// app that already loads its own dotenv.
pub fn client_from_env() -> Result<HeliusRpcClient, ReplayError> {
    if let Ok(url) = std::env::var("REPLAY_RPC_URL") {
        HeliusRpcClient::from_url(url)
    } else {
        let key = std::env::var("HELIUS_API_KEY").map_err(|_| {
            ReplayError::Rpc("set HELIUS_API_KEY or REPLAY_RPC_URL".to_string())
        })?;
        HeliusRpcClient::from_api_key(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_exports_compile() {
        // If this module doesn't compile, the re-export list is broken.
        let _ = ReplayError::TxNotFound;
    }
}
