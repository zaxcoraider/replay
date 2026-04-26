# Day 5 — Fork Sessions + HTTP API

## Goal

Wrap the engine in an axum HTTP server with session semantics. This is the contract the web UI will consume tomorrow.

## Deliverables

1. `crates/replay-api/src/main.rs`: axum server binding to `REPLAY_BIND_ADDR` (default `0.0.0.0:8787`).

2. Session store — `crates/replay-api/src/session.rs`:
   ```rust
   pub struct SessionStore {
       sessions: DashMap<SessionId, Session>,
       ttl: Duration,
       max_sessions: usize,
   }
   
   pub struct Session {
       pub id: SessionId,
       pub signature: Signature,
       pub runner: SvmRunner,
       pub reconstructed_state: ReconstructedState,  // for re-seed between runs
       pub baseline_trace: Trace,
       pub latest_trace: Option<Trace>,
       pub mutations: Vec<AppliedMutation>,
       pub created_at: Instant,
       pub last_accessed_at: Instant,
   }
   ```
   
   A background task prunes expired sessions every 60s.

3. Endpoints (per `docs/03-technical-spec.md`):
   - `POST /replay`
   - `POST /fork`
   - `POST /session/:id/mutate`
   - `POST /session/:id/execute`
   - `GET /session/:id/diff`
   - `GET /health`
   - `GET /version`
   
   Skip WebSocket for now — add on Day 9 if time permits, otherwise the UI polls.

4. Mutations supported:
   - `type: "field"` — IDL-aware field edit. Decode the account, modify the field, re-encode to bytes, `set_account`.
   - `type: "raw_bytes"` — splice `bytes_hex` at `offset` into account data. Bounds check: offset + len ≤ data.len() (unless `extend: true`).
   - `type: "lamports"` — set `account.lamports`.
   - `type: "owner"` — set `account.owner`.

5. CORS — `tower_http::cors::CorsLayer::very_permissive()` for the hackathon.

6. Rate limit — per-IP token bucket, 20 req/min. Use `tower_governor` crate.

## Mutation semantics

- Applying a mutation does NOT re-execute. It only updates SVM account state.
- `POST /session/:id/execute` re-seeds the SVM from the reconstructed state PLUS applied mutations, then executes. This is idempotent across calls.
- Think of mutations as a replayable log layered on top of baseline reconstructed state.

## Session diff

`GET /session/:id/diff` computes:
```rust
pub struct TraceDiff {
    pub result_changed: bool,
    pub account_deltas_diff: Vec<AccountDeltaDiff>,
    pub frames_diff: FramesDiff,
    pub total_cu_delta: i64,
}
```

Keep the diff computation simple for v1: pubkey-keyed account diffs, CU delta per frame. Don't over-engineer the log diff — a unified line-by-line diff is enough.

## Tests

Integration test using `axum::Router` in-process with a mock `HeliusClient`. Full flow: fork → mutate → execute → diff.

## What NOT to do

- No auth. No user model. No persistence.
- No streaming. REST only.
- No TLS. Let the deploy target terminate TLS.

## End-of-day

- Run `curl -X POST localhost:8787/replay -d '{"signature":"..."}'` and get a real trace back.
- Run the full fork → mutate → execute → diff via curl; paste the commands into `docs/API-EXAMPLES.md`.
