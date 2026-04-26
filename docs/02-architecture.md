# 02 — Architecture

## High-level diagram

```
┌────────────────────────────────────────────────────────────────────────┐
│                         Web UI  (Next.js 14)                           │
│                                                                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────────┐    │
│  │ Signature input │  │ Timeline        │  │ Account inspector    │    │
│  │ + "Fork" button │  │ scrubber        │  │ + mutate UI          │    │
│  └────────┬────────┘  └────────┬────────┘  └──────────┬───────────┘    │
│           │                    │                      │                │
│           └────────────────────┴──────────────────────┘                │
│                                │                                       │
│                       HTTP (REST) + WebSocket                          │
└────────────────────────────────┼───────────────────────────────────────┘
                                 │
┌────────────────────────────────▼───────────────────────────────────────┐
│                    replay-api  (axum, Rust)                            │
│                                                                        │
│  POST /replay        → Replay a tx, return full trace                  │
│  POST /fork          → Create a session seeded from a tx               │
│  POST /session/:id/mutate → Change account state                       │
│  POST /session/:id/execute → Re-run the tx in the session              │
│  GET  /session/:id/diff    → Diff against baseline                     │
│  WS   /session/:id/stream  → Live trace events as tx executes          │
│                                                                        │
└────────────────────────────────┼───────────────────────────────────────┘
                                 │
┌────────────────────────────────▼───────────────────────────────────────┐
│                    replay-core  (library, Rust)                        │
│                                                                        │
│  ┌─────────────┐   ┌──────────────────┐   ┌────────────────────────┐   │
│  │ TxFetcher   │──▶│ StateRecon-      │──▶│ SvmRunner              │   │
│  │ (Helius)    │   │ structor         │   │ (litesvm wrapper)      │   │
│  │             │   │                  │   │ • set_account          │   │
│  │ • get_tx    │   │ • resolve LUTs   │   │ • add_program          │   │
│  │ • get_accts │   │ • fetch program  │   │ • set_sysvar(Clock)    │   │
│  │ • get_prog  │   │   bytecode at    │   │ • execute              │   │
│  │   bytecode  │   │   historical slot│   │ • capture logs + CU    │   │
│  │ (at slot-1) │   │ • compute budget │   │                        │   │
│  │             │   │   reapply        │   │                        │   │
│  └─────────────┘   └──────────────────┘   └────────────┬───────────┘   │
│                                                        │               │
│                                ┌───────────────────────▼──────────┐    │
│                                │ TraceBuilder                     │    │
│                                │ • parse "Program X invoke [N]"   │    │
│                                │ • tree of CPI frames             │    │
│                                │ • per-frame CU consumption       │    │
│                                │ • per-account delta              │    │
│                                └───────────────────────┬──────────┘    │
│                                                        │               │
│                                ┌───────────────────────▼──────────┐    │
│                                │ IdlDecoder                       │    │
│                                │ • on-chain IDL fetch             │    │
│                                │ • Borsh + discriminator decode   │    │
│                                │ • fallback: hex + "bring your    │    │
│                                │   own IDL" paste                 │    │
│                                └──────────────────────────────────┘    │
└────────────────────────────────────────────────────────────────────────┘
```

## Module boundaries

### `replay-core` (the engine, pure Rust library)

Zero HTTP. Zero CLI. Just a library with one public entry point:

```rust
pub async fn replay(
    signature: &str,
    rpc: &HeliusClient,
    options: ReplayOptions,
) -> Result<Trace, ReplayError>;

pub async fn fork(
    signature: &str,
    rpc: &HeliusClient,
) -> Result<ForkedSession, ReplayError>;

impl ForkedSession {
    pub fn mutate(&mut self, pubkey: &Pubkey, mutation: AccountMutation) -> Result<(), _>;
    pub fn execute(&mut self) -> Result<Trace, _>;
    pub fn diff_against_baseline(&self) -> TraceDiff;
}
```

Keep this crate testable without a network — mock `HeliusClient` as a trait.

### `replay-api` (HTTP + WebSocket server)

Thin wrapper around `replay-core`. Owns:
- Session state (in-memory `DashMap<SessionId, ForkedSession>`). **No database for v1.** Sessions expire after 1 hour.
- Request validation. Signature format, mutation bounds checking.
- CORS (wide open for the hackathon; tighten later).
- Rate limiting (simple per-IP token bucket).

### `replay-cli` (binary)

Thin wrapper around `replay-core`. Subcommands:
- `replay <signature>` — one-shot replay, print trace as JSON or pretty-printed.
- `replay fork <signature>` — open an interactive REPL (use `rustyline`).
- `replay diff <sig1> <sig2>` — cross-replay two txs, diff.
- `replay serve` — spawn the API server (convenience).

### `web/` (Next.js 14 app)

- Server Components for everything static. Client Components only for the mutator UI and timeline scrubber.
- API calls via a typed client generated from a shared OpenAPI spec (`spec/openapi.yaml`). Simpler: hand-write a thin client in `web/lib/api.ts`.
- State management: Zustand (simpler than Redux, plenty for this scope). One store per session.

## Data flow: a full replay

1. User pastes `5xY...` into the web UI.
2. `POST /replay { signature: "5xY..." }` hits the API.
3. API calls `replay_core::replay(sig, ...)`.
4. **TxFetcher** calls Helius `getTransaction` with `maxSupportedTransactionVersion: 0` and `encoding: "jsonParsed"`. Returns `EncodedConfirmedTransactionWithStatusMeta`.
5. From the returned tx, extract: `slot`, `account_keys` (including LUT-resolved ones from `meta.loaded_addresses`), `instructions`, `pre_balances`, `post_balances`, `log_messages`.
6. **StateReconstructor** builds a list of `(Pubkey, slot)` tuples for every account touched. For each one, call `getAccountInfo` with `minContextSlot: slot - 1` and `commitment: "confirmed"`. Cache aggressively.
7. For each program ID in the tx, fetch both the program account AND the program-data account (for upgradeable BPF loader). This gives you the bytecode that was live at that slot.
8. **SvmRunner** constructs a `LiteSVM::new()`, calls `.set_account()` for every fetched account, `.add_program()` for every fetched program, and `.set_sysvar::<Clock>()` with the target slot's unix timestamp.
9. Re-apply compute budget instructions from the original tx (these are consumed by the compute-budget program, not stored on-chain).
10. Build a `VersionedTransaction` from the original instructions. Sign it with a dummy fee payer (you `airdrop` it and substitute the fee payer's account, since you don't have the real signer). **This is a key trick** — see `04-solana-gotchas.md`.
11. Call `svm.send_transaction(tx)`. Capture result: logs, returned data, per-instruction CU (from log parsing), post-account-states (diff via `get_account`).
12. **TraceBuilder** parses the logs into a tree of CPI frames, cross-references with **IdlDecoder** to render human-readable account names and values.
13. Return the full `Trace` as JSON. UI renders.

## Data flow: a fork + mutate + re-run

1. User clicks "Fork this transaction" in UI.
2. `POST /fork { signature }` creates a session, does steps 1–8 above but **does not execute**. Returns `session_id`.
3. User clicks an account in the inspector, edits a field, clicks "Apply."
4. `POST /session/:id/mutate { pubkey, path: "config.fee_bps", new_value: 9999 }` hits the API.
5. API looks up the session's `LiteSVM`, finds the account, decodes with IDL, applies the mutation at the right byte offset (re-serializing with Borsh), calls `svm.set_account()` with the new data.
6. User clicks "Re-run."
7. `POST /session/:id/execute` runs step 11 above with the mutated state.
8. Response includes the new trace.
9. UI calls `GET /session/:id/diff` which computes a structured diff between the baseline trace (captured at fork time) and the latest execution.

## Error handling philosophy

- **Fail loudly, fail specifically.** Every error type carries the tx signature, the step name, and a suggested next action. Silent degradation is how debugging tools lose trust.
- **Typed errors per module.** `replay-core` uses `thiserror`. `replay-api` maps to HTTP status codes with a uniform `{ error: { code, message, context } }` JSON shape.
- **Three classes of failure to handle explicitly:**
  1. `HeliusError` — RPC rate limit, transient. Retry with backoff.
  2. `StateReconstructionError` — missing account, pruned slot, unknown program. These are the *interesting* errors; surface them prominently in UI so user understands why replay is impossible.
  3. `ExecutionError` — SVM returned an error. Often the correct answer for a failed mainnet tx; don't treat as a tool failure. Render it alongside the mainnet error for comparison.

## Non-goals for v1

- Persistent sessions. In-memory only.
- Multi-user auth. Everyone shares one namespace.
- Replaying multiple txs in sequence. One tx per session.
- Sub-instruction opcode-level stepping. Per-instruction + per-CPI only.
- Replaying non-mainnet clusters. Mainnet-only for v1 (devnet if trivially easy).
- Program upgrade history visualization. Out of scope.
