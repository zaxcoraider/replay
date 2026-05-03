# Day 13 — Rust SDK + Docs Site + Publishing

**Date:** 2026-05-03
**Prompt:** [`prompts/day-13-rust-sdk.md`](../prompts/day-13-rust-sdk.md)
**Result:** Done in one session. All packages published. Docs live.

## Goal

Ship a stable Rust SDK, publish all packages (npm + crates.io), and deploy a docs site.

## What landed

### `crates/replay-sdk/src/lib.rs` — full rewrite

**`ReplayClient`** struct wrapping `Arc<HeliusRpcClient>`:
- `new(rpc_url)` — build from URL
- `from_env()` — reads `HELIUS_API_KEY` or `REPLAY_RPC_URL` env var
- `replay(sig)` → `Result<Trace, Error>`
- `fork(sig)` → `Result<Session, Error>`

**`Session`** struct wrapping `CoreSession + Arc<HeliusRpcClient>`:
- `mutate_field(pubkey, path, value)` — IDL field dot-path mutation
- `mutate_raw(pubkey, offset, bytes)` — raw byte splice
- `set_lamports(pubkey, lamports)` — lamport override
- `execute()` → `Result<Trace, Error>`
- `diff()` → `Option<TraceDiff>`
- `reset()` — clear mutations

**`replay_historical(client, signatures)`** — batch replay, returns `ReplayHistoricalReport`
with `passed`, `has_failures()`, `failures()` iterator.

**`Error`** enum via `thiserror`:
```rust
pub enum Error {
    Replay(#[from] ReplayError),
    Pubkey(String),
}
```

### `crates/replay-sdk/Cargo.toml`

Added `solana-sdk = { workspace = true }` for `Pubkey`.
Added `version = "0.1"` alongside `path = "../replay-core"` (required for crates.io publish).

### `Cargo.toml` (workspace)

Fixed placeholder metadata before publish:
- `authors = ["zaxcoraider"]`
- `repository = "https://github.com/zaxcoraider/replay"`
- `homepage = "https://github.com/zaxcoraider/replay"`
- `documentation = "https://replay-weld.vercel.app/docs"`

### `web/app/docs/page.tsx` — new docs site

Full docs page at `/docs` route on Vercel. Sections:
- Overview (6 feature cards)
- API Reference (all endpoints with request/response shapes)
- TypeScript SDK (install, one-shot, fork/mutate/diff, CI helper, types)
- Rust SDK (install, one-shot, fork/mutate/diff, historical batch)
- CLI (all subcommands + example output)
- Self-hosting (Docker + from-source + env var table)
- Architecture (ASCII diagram + crate layout + key decisions)

Sticky sidebar nav on desktop. Header with back link to main app.
Link added to landing page footer.

## Published packages

| Package | Registry | Version |
|---------|----------|---------|
| `@zaxcoraider/replay-sdk` | npm | 0.1.0 |
| `replay-core` | crates.io | 0.1.0 |
| `replay-sdk` | crates.io | 0.1.0 |

## Key decisions / gotchas

- `ForkedSession::execute()` requires `&HeliusClient` arg — Session must hold `Arc<HeliusRpcClient>`
- `ForkedSession::diff()` returns `Option<TraceDiff>` (None before first execute)
- Must verify email on crates.io before first publish (crates.io rejects with 400 otherwise)
- crates.io requires `version` alongside `path` dep when publishing
- Workspace `Cargo.toml` must have real metadata (not `YOUR_HANDLE` placeholders) before publish

## Tests / build

```bash
cargo test -p replay-sdk --lib   # re_exports_compile, error_display — green
cargo clippy -p replay-sdk       # clean
cd web && pnpm tsc --noEmit      # clean (docs page)
```

## Commits

```
(see git log — multiple commits across session)
```

## Next session bootstrap (Day 14)

Day 14 = Helius LaserStream integration. **Use Opus model** (complex streaming architecture).

Read:
- `memory/day-13.md` (this file)
- `prompts/day-14-laserstream.md`

Key deliverables:
1. WebSocket / SSE stream from Helius LaserStream for real-time slot updates
2. Stream new transactions matching a program filter into the replay engine
3. Surface in the web UI as a live "incoming tx" feed
4. API endpoint: `GET /stream?program=<PUBKEY>` (SSE)

Current state:
- API: https://replay-y4wq.onrender.com (Render, proxied via Vercel `/rpc`)
- Web: https://replay-weld.vercel.app
- Docs: https://replay-weld.vercel.app/docs
- All 3 demo signatures filled and working
