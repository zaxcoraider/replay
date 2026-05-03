//! # replay-sdk
//!
//! Stable Rust SDK for Replay — the Solana time-travel debugger.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use replay_sdk::{ReplayClient, Error};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // Reads HELIUS_API_KEY or REPLAY_RPC_URL from the environment.
//!     let client = ReplayClient::from_env()?;
//!
//!     // One-shot replay
//!     let trace = client.replay("5xYourSigHere...").await?;
//!     println!("CU consumed: {}", trace.total_cu);
//!
//!     // Fork → mutate → re-run → diff
//!     let mut session = client.fork("5xYourSigHere...").await?;
//!     session.mutate_field(
//!         "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".parse().unwrap(),
//!         "feeRate",
//!         serde_json::json!(9999),
//!     )?;
//!     let _new_trace = session.execute().await?;
//!     let diff = session.diff();
//!     println!("Result changed: {}", diff.map(|d| d.result_changed).unwrap_or(false));
//!     Ok(())
//! }
//! ```

pub use replay_core::{
    AccountDelta, AccountMutation, CpiFrame, ForkedSession as CoreSession,
    FrameAccount, HeliusClient, HeliusRpcClient, LogDivergence, ProgramInfo,
    ProgramLoader, ReconstructedState, ReplayError, Trace, TraceDiff, TxContext,
    TxResult,
};

use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

// ── Error ────────────────────────────────────────────────────────────────────

/// Top-level error type for the SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Replay(#[from] ReplayError),
    #[error("invalid pubkey: {0}")]
    Pubkey(String),
}

// ── ReplayClient ─────────────────────────────────────────────────────────────

/// High-level entry point for the Replay SDK.
pub struct ReplayClient {
    inner: Arc<HeliusRpcClient>,
}

impl ReplayClient {
    /// Build a client from a Helius RPC URL.
    pub fn new(rpc_url: impl Into<String>) -> Result<Self, Error> {
        Ok(Self { inner: Arc::new(HeliusRpcClient::from_url(rpc_url)?) })
    }

    /// Build a client from `HELIUS_API_KEY` or `REPLAY_RPC_URL` env var.
    pub fn from_env() -> Result<Self, Error> {
        let inner = if let Ok(url) = std::env::var("REPLAY_RPC_URL") {
            HeliusRpcClient::from_url(url)?
        } else {
            let key = std::env::var("HELIUS_API_KEY").map_err(|_| {
                ReplayError::Rpc("set HELIUS_API_KEY or REPLAY_RPC_URL".into())
            })?;
            HeliusRpcClient::from_api_key(&key)?
        };
        Ok(Self { inner: Arc::new(inner) })
    }

    /// One-shot replay: fetch, reconstruct, execute, return trace.
    pub async fn replay(&self, signature: &str) -> Result<Trace, Error> {
        Ok(replay_core::replay(signature, &*self.inner).await?)
    }

    /// Fork a transaction into a mutable [`Session`].
    pub async fn fork(&self, signature: &str) -> Result<Session, Error> {
        let session = replay_core::fork(signature, &*self.inner).await?;
        Ok(Session { inner: session, client: Arc::clone(&self.inner) })
    }
}

// ── Session ───────────────────────────────────────────────────────────────────

/// A forked, mutable replay session.
pub struct Session {
    inner: CoreSession,
    client: Arc<HeliusRpcClient>,
}

impl Session {
    /// Mutate an IDL field by dot-path (e.g. `"feeRate"` or `"config.fee_bps"`).
    pub fn mutate_field(
        &mut self,
        pubkey: Pubkey,
        path: &str,
        value: serde_json::Value,
    ) -> Result<(), Error> {
        self.inner.mutate(pubkey, AccountMutation::Field {
            path: path.to_string(),
            new_value: value,
        })?;
        Ok(())
    }

    /// Overwrite raw bytes starting at `offset` in an account's data.
    pub fn mutate_raw(
        &mut self,
        pubkey: Pubkey,
        offset: usize,
        bytes: Vec<u8>,
    ) -> Result<(), Error> {
        self.inner.mutate(pubkey, AccountMutation::RawBytes {
            offset,
            bytes,
            extend: false,
        })?;
        Ok(())
    }

    /// Set the lamport balance of an account.
    pub fn set_lamports(&mut self, pubkey: Pubkey, lamports: u64) -> Result<(), Error> {
        self.inner.mutate(pubkey, AccountMutation::Lamports { new_value: lamports })?;
        Ok(())
    }

    /// Re-execute the transaction with all applied mutations.
    pub async fn execute(&mut self) -> Result<Trace, Error> {
        Ok(self.inner.execute(&*self.client).await?)
    }

    /// Diff baseline against the latest execution. Returns `None` if
    /// `execute()` has not been called yet.
    pub fn diff(&self) -> Option<TraceDiff> {
        self.inner.diff()
    }

    /// Clear all mutations and reset to baseline state.
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

// ── Historical regression helper ──────────────────────────────────────────────

/// Per-signature outcome.
pub struct SignatureOutcome {
    pub signature: String,
    pub trace: Option<Trace>,
    pub error: Option<String>,
}

/// Report from [`replay_historical`].
pub struct ReplayHistoricalReport {
    pub total: usize,
    pub passed: usize,
    pub outcomes: Vec<SignatureOutcome>,
}

impl ReplayHistoricalReport {
    pub fn failures(&self) -> impl Iterator<Item = &SignatureOutcome> {
        self.outcomes.iter().filter(|o| o.error.is_some())
    }
    pub fn has_failures(&self) -> bool {
        self.outcomes.iter().any(|o| o.error.is_some())
    }
}

/// Replay a batch of historical transactions and collect pass/fail outcomes.
///
/// Useful in CI to assert that a program upgrade doesn't break old transactions.
///
/// ```rust,no_run
/// # async fn example() -> Result<(), replay_sdk::Error> {
/// let client = replay_sdk::ReplayClient::from_env()?;
/// let report = replay_sdk::replay_historical(&client, &[
///     "5xYourSigHere...",
///     "3aBanotherSig...",
/// ]).await?;
/// assert!(!report.has_failures(), "historical replay regressed");
/// # Ok(()) }
/// ```
pub async fn replay_historical(
    client: &ReplayClient,
    signatures: &[&str],
) -> Result<ReplayHistoricalReport, Error> {
    let total = signatures.len();
    let mut outcomes = Vec::with_capacity(total);
    let mut passed = 0;

    for &sig in signatures {
        match client.replay(sig).await {
            Ok(trace) => {
                passed += 1;
                outcomes.push(SignatureOutcome {
                    signature: sig.to_string(),
                    trace: Some(trace),
                    error: None,
                });
            }
            Err(e) => {
                outcomes.push(SignatureOutcome {
                    signature: sig.to_string(),
                    trace: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(ReplayHistoricalReport { total, passed, outcomes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_exports_compile() {
        let _ = ReplayError::TxNotFound;
    }

    #[test]
    fn error_display() {
        let e = Error::Pubkey("bad".to_string());
        assert!(e.to_string().contains("invalid pubkey"));
    }
}
