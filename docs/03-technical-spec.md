# 03 — Technical Specification

Concrete types, API contracts, and error taxonomy. Treat this as the contract between `replay-core`, `replay-api`, and `web/`.

## Core types (Rust, in `replay-core`)

```rust
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub mainnet_result: TxResult,
    pub replay_result: TxResult,
    pub frames: Vec<CpiFrame>,
    pub account_deltas: Vec<AccountDelta>,
    pub total_cu: u64,
    pub log_divergence: Option<LogDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxResult {
    Success,
    Failure { error: String, error_code: Option<i64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpiFrame {
    pub depth: u32,
    pub program_id: String,        // base58
    pub program_name: Option<String>, // from IDL cache, e.g. "Jupiter Aggregator v6"
    pub instruction_index: usize,
    pub instruction_name: Option<String>, // from IDL, e.g. "swap"
    pub accounts: Vec<FrameAccount>,
    pub data_hex: String,
    pub decoded_args: Option<serde_json::Value>, // IDL-decoded
    pub logs: Vec<String>,
    pub cu_consumed: u64,
    pub cu_remaining_after: u64,
    pub children: Vec<CpiFrame>,
    pub result: TxResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameAccount {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
    pub role: Option<String>, // from IDL, e.g. "user_token_account"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDelta {
    pub pubkey: String,
    pub owner_before: String,
    pub owner_after: String,
    pub lamports_before: u64,
    pub lamports_after: u64,
    pub data_before_hex: String,   // truncated to first 256 bytes for payload size
    pub data_after_hex: String,
    pub decoded_before: Option<serde_json::Value>,
    pub decoded_after: Option<serde_json::Value>,
    pub idl_type_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDivergence {
    pub first_divergent_line: usize,
    pub mainnet_line: String,
    pub replay_line: String,
    pub suspected_cause: Option<String>, // e.g. "clock sysvar drift"
}
```

## API contracts (replay-api)

All responses are JSON. All errors follow the shape:
```json
{ "error": { "code": "STATE_RECONSTRUCTION_FAILED", "message": "...", "context": {...} } }
```

### `POST /replay`

Request:
```json
{ "signature": "5xY..." }
```

Response (200):
```json
{ "trace": Trace }
```

Errors:
- `400 INVALID_SIGNATURE` — not a valid base58 sig.
- `404 TX_NOT_FOUND` — Helius returned null.
- `422 STATE_RECONSTRUCTION_FAILED` — missing pre-state (pruned slot, etc.).
- `500 EXECUTION_ERROR` — litesvm crashed (not a normal tx failure; this is a tool bug).
- `429 RATE_LIMITED` — too many requests from this IP.

### `POST /fork`

Request:
```json
{ "signature": "5xY..." }
```

Response (200):
```json
{
  "session_id": "ulid_here",
  "baseline_trace": Trace,
  "expires_at": "2026-04-24T15:00:00Z"
}
```

### `POST /session/:id/mutate`

Request:
```json
{
  "pubkey": "...",
  "mutation": {
    "type": "field",
    "path": "config.fee_bps",
    "new_value": 9999
  }
}
```

Alternate mutation types:
```json
{ "type": "raw_bytes", "offset": 8, "bytes_hex": "ff00ff00" }
{ "type": "lamports", "new_value": 1000000000 }
{ "type": "owner", "new_value": "11111111111111111111111111111112" }
```

Response (200):
```json
{ "applied": true, "account_after": AccountDelta }
```

### `POST /session/:id/execute`

No body. Re-runs the tx in the session with current (possibly mutated) state.

Response (200):
```json
{ "trace": Trace }
```

### `GET /session/:id/diff`

Response (200):
```json
{
  "baseline": Trace,
  "latest": Trace,
  "diff": {
    "result_changed": true,
    "account_deltas_diff": [...],
    "frames_diff": [...],
    "total_cu_delta": -12345
  }
}
```

### `WS /session/:id/stream`

Bi-directional. Server pushes events as the session evolves:
```json
{ "type": "mutation_applied", "pubkey": "...", "mutation": {...} }
{ "type": "execution_started" }
{ "type": "frame_completed", "frame": CpiFrame }
{ "type": "execution_completed", "trace": Trace }
{ "type": "error", "error": {...} }
```

## Error taxonomy (Rust, in `replay-core`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("transaction not found on Helius")]
    TxNotFound,

    #[error("rpc error: {0}")]
    Rpc(#[from] HeliusError),

    #[error("state reconstruction failed: {step}: {detail}")]
    StateReconstruction { step: &'static str, detail: String },

    #[error("historical slot {slot} may be pruned; try a more recent tx")]
    SlotPruned { slot: u64 },

    #[error("program {program_id} has no known bytecode at slot {slot}")]
    MissingProgramBytecode { program_id: String, slot: u64 },

    #[error("LUT resolution failed for {lut}: {detail}")]
    LutResolution { lut: String, detail: String },

    #[error("execution error: {0}")]
    Execution(String),

    #[error("IDL parse error: {0}")]
    Idl(String),
}
```

## IDL cache

A small local disk cache for fetched IDLs:
```
~/.replay/idl-cache/
├── JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4.json
├── whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc.json
└── ...
```

Structure: standard Anchor IDL JSON. Check `~/.replay/idl-cache/` before fetching. TTL: 7 days (programs upgrade their IDLs sometimes).

IDL fetch order:
1. Local disk cache (if not expired).
2. On-chain IDL account (derived via `anchor_lang::idl::IdlAccount::address`).
3. Anchor IDL registry (github.com/coral-xyz/anchor-idl-registry — if it exists at time of build; verify).
4. User-provided paste in UI ("This program has no on-chain IDL — paste one here").

## Logging + observability

- `tracing` crate. Every function that touches the network gets a `#[tracing::instrument]` attribute.
- Log level: `info` for session lifecycle events, `debug` for RPC calls, `error` for anything that becomes an error response.
- No OpenTelemetry/Jaeger for v1. `tracing_subscriber::fmt()` to stderr is enough.
- In the UI, expose a "copy debug bundle" button that bundles the session ID, tx signature, and last 100 log lines for easy bug reporting.

## Configuration

Env-based. No config files for v1.

```
HELIUS_API_KEY=...              # required
REPLAY_RPC_URL=...              # optional override, defaults to Helius
REPLAY_BIND_ADDR=0.0.0.0:8787   # API server
REPLAY_IDL_CACHE_DIR=~/.replay/idl-cache
REPLAY_SESSION_TTL_SECS=3600
REPLAY_MAX_SESSIONS=100
RUST_LOG=replay=info
```
