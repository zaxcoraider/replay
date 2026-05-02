# Day 5 — Fork Sessions + HTTP API

**Date:** 2026-05-02
**Prompt:** [`prompts/day-05-fork-sessions.md`](../prompts/day-05-fork-sessions.md)
**Result:** Done in one session (scaffold was pre-built; completed rate limiting, testability refactor, integration tests).

## Goal

Wrap the engine in an axum HTTP server with session semantics. This is the
contract the web UI consumes.

## What landed

### `crates/replay-api/` — complete HTTP server

All 7 routes live and returning correct HTTP status codes:
- `GET  /health` → `"ok"`
- `GET  /version` → `{"name":"replay-api","version":"0.1.0"}`
- `POST /replay` → one-shot `Trace` JSON
- `POST /fork` → `{session_id, baseline_trace, expires_at}`
- `POST /session/:id/mutate` → apply `AccountMutation` (lamports/owner/raw_bytes; field is stub)
- `POST /session/:id/execute` → re-run with mutations, return new `Trace`
- `GET  /session/:id/diff` → `TraceDiff` (baseline + latest + changed_accounts + cu_delta)

**Rate limiting:** `tower_governor` 20 req/min per IP (burst 20). Background task
prunes expired sessions + rate-limiter entries every 60s.

**`AppState` refactor:** `client: Arc<dyn HeliusClient>` + blanket impl
`HeliusClient for Arc<dyn HeliusClient>` in `rpc.rs`. Enables mock injection
in tests without changing the generic bounds on `replay`/`fork`/`execute`.

Added `src/lib.rs` exposing `build_app(state)` so integration tests can
construct an in-process router without spawning a real server.

### `crates/replay-api/tests/api_integration.rs`

7 integration tests using `tower::ServiceExt::oneshot` + `EmptyClient` mock:
- `health_returns_ok`
- `version_returns_json_with_name_and_version`
- `replay_invalid_signature_returns_400`
- `replay_valid_sig_not_found_returns_404`
- `session_not_found_returns_404`
- `mutate_missing_session_returns_404`
- `execute_missing_session_returns_404`

### `docs/API-EXAMPLES.md`

Full curl workflow: health → fork → mutate (lamports + raw_bytes) → execute → diff.
Error shape documented.

## Mutation types supported

```rust
AccountMutation::Lamports  { new_value: u64 }
AccountMutation::Owner     { new_value: String }   // base58 pubkey
AccountMutation::RawBytes  { offset, bytes, extend }
AccountMutation::Field     { path, new_value }     // stub — Day-8 work
```

## Session lifecycle

- `POST /fork` → creates `ForkedSession`, runs baseline replay, stores in `DashMap`
- `POST /session/:id/mutate` → appends `(Pubkey, AccountMutation)` to log (no re-execute)
- `POST /session/:id/execute` → re-seeds SVM from reconstructed state + mutations, runs tx
- `GET  /session/:id/diff` → requires at least one execute; diffs baseline vs latest
- Sessions expire after 1h (TTL configurable via `REPLAY_SESSION_TTL_SECS`)
- Hard cap: 100 sessions (configurable via `REPLAY_MAX_SESSIONS`)

## Commits

```
f5000dd feat(api): Day-5 HTTP API — rate limiting, background pruner, integration tests
```

## Next session bootstrap (Day 6 → already done)

Day 6 was also completed in the same session. See `memory/day-06.md`.
