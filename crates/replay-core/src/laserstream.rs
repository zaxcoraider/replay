//! Helius LaserStream integration for the live-replay endpoint.
//!
//! LaserStream is Helius's gRPC streaming product (built on the Yellowstone
//! gRPC spec). It pushes slot, transaction, and account updates in real time
//! over a single long-lived connection — the right transport for a "live
//! replay" mode where the user wants to see a transaction reconstruct itself
//! the moment it lands on mainnet.
//!
//! ## Layout of this module
//!
//! - [`LaserStreamConfig`] reads `LASERSTREAM_GRPC_URL` and
//!   `LASERSTREAM_X_TOKEN` from the environment. When the URL is unset the
//!   feature is considered disabled and the live-replay endpoint falls back
//!   to driving progress events from standard Helius RPC. This is the
//!   intentional "stub" mode: code, types, and wiring are real, but the
//!   gRPC client never opens because LaserStream is a paid Helius tier
//!   ($499/mo Business plan as of 2026-05) and we don't gate the demo on it.
//! - [`LiveReplayEvent`] is the wire shape streamed back to the browser as
//!   Server-Sent Events. The same type is emitted whether the source is
//!   LaserStream or a progressive walk over the standard RPC pipeline; the
//!   `source` field on [`LiveReplayEvent::Mode`] records which.
//! - [`connect`] is the entry point that becomes a real gRPC subscription
//!   when LaserStream credentials are present. Today it returns
//!   [`LaserStreamStatus::NotConfigured`] for the unconfigured path and a
//!   `Configured` marker carrying the parsed config otherwise — enough for
//!   the API layer to decide which pipeline to drive without pulling in the
//!   heavy `yellowstone-grpc-client` dependency on every build. See
//!   `docs/blog/time-travel-debugger-on-laserstream.md` for the full gRPC
//!   client sketch we'd land once a paid key is available.

use crate::types::{CpiFrame, Trace};
use serde::Serialize;

/// Environment variable that opts the binary into LaserStream-backed live
/// replay. When unset, [`LaserStreamConfig::from_env`] returns `None` and
/// the SSE endpoint sources progress events from standard RPC instead.
pub const LASERSTREAM_GRPC_URL_ENV: &str = "LASERSTREAM_GRPC_URL";

/// Optional gRPC `x-token` header. Helius LaserStream requires this; the
/// value comes from the dashboard once the workspace is upgraded to the
/// Business plan.
pub const LASERSTREAM_X_TOKEN_ENV: &str = "LASERSTREAM_X_TOKEN";

/// Parsed LaserStream connection settings.
#[derive(Debug, Clone)]
pub struct LaserStreamConfig {
    pub grpc_url: String,
    pub x_token: Option<String>,
}

impl LaserStreamConfig {
    /// Read connection settings from the process environment.
    ///
    /// Returns `None` when `LASERSTREAM_GRPC_URL` is unset — the signal the
    /// API layer uses to fall back to the standard-RPC progress driver.
    pub fn from_env() -> Option<Self> {
        let grpc_url = std::env::var(LASERSTREAM_GRPC_URL_ENV).ok()?;
        if grpc_url.trim().is_empty() {
            return None;
        }
        let x_token = std::env::var(LASERSTREAM_X_TOKEN_ENV)
            .ok()
            .filter(|s| !s.trim().is_empty());
        Some(Self { grpc_url, x_token })
    }
}

/// Outcome of attempting to bring up a LaserStream connection.
///
/// We return a status (not a typed client) so the rest of the code can be
/// written without committing to a specific gRPC crate. Once a paid key is
/// available we can extend `Configured` with a real client handle without
/// changing any caller.
#[derive(Debug)]
pub enum LaserStreamStatus {
    Configured(LaserStreamConfig),
    NotConfigured,
}

/// Inspect the environment and report whether LaserStream is available.
///
/// Today this is purely an env-var check — opening a real gRPC channel
/// happens lazily inside the live-replay handler when the source is set
/// to LaserStream. Keeping `connect` cheap means we can call it on every
/// SSE request without contention.
pub fn connect() -> LaserStreamStatus {
    match LaserStreamConfig::from_env() {
        Some(cfg) => LaserStreamStatus::Configured(cfg),
        None => LaserStreamStatus::NotConfigured,
    }
}

/// Source the live-replay events were derived from. Surfaced in the
/// first SSE event so the UI can label the stream honestly.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveSource {
    /// Sourced from a real Helius LaserStream gRPC subscription.
    Laserstream,
    /// Sourced from a progressive walk over standard Helius RPC.
    /// Functionally equivalent for historical txs; the only difference
    /// is that LaserStream pushes new slots within ~hundreds of ms
    /// instead of waiting on getTransaction polling.
    Rpc,
}

/// Wire shape of one event in the `/replay-live/:signature` SSE stream.
///
/// Tagged-union JSON so the browser can switch on `type` without
/// peeking at fields. Order in practice:
/// `mode → slot_observed → account_fetched* → all_accounts_fetched →
///  execution_started → frame_completed* → done` (plus `error` to terminate
/// abnormally).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveReplayEvent {
    /// First event on every stream. Tells the UI which transport drove
    /// the rest of the events — useful for the "LaserStream live" badge
    /// vs the fallback note.
    Mode { source: LiveSource },

    /// The slot that owns this transaction has been resolved.
    SlotObserved { slot: u64, block_time: Option<i64> },

    /// One account has been fetched (or pushed by LaserStream). The UI
    /// renders these as a streaming list of pubkeys lighting up.
    AccountFetched {
        pubkey: String,
        size: usize,
        is_program: bool,
    },

    /// Convenience aggregate so the UI can show a "fetched N accounts"
    /// pill without counting events itself.
    AllAccountsFetched { count: usize },

    /// litesvm is about to seed and execute. The UI uses this as the
    /// boundary between "loading" and "running".
    ExecutionStarted,

    /// One CPI frame finished. The UI appends frames to the trace tree
    /// as they arrive. For RPC-source mode these all arrive in a tight
    /// burst after `execute()` returns; for LaserStream they arrive as
    /// the validator emits them. Boxed so the enum stays compact —
    /// `CpiFrame` carries decoded args + child frames and easily exceeds
    /// 256 bytes inline.
    FrameCompleted { frame: Box<CpiFrame> },

    /// Terminal success. Carries the full assembled trace so the UI can
    /// hand off to the same renderer used by the non-live path.
    Done { trace: Box<Trace> },

    /// Terminal failure. `code` matches `ReplayError::code()` so the UI
    /// can render specific guidance (e.g. SLOT_PRUNED).
    Error { code: String, message: String },
}
