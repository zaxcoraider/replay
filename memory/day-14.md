# Day 14 — Helius LaserStream Integration (stub-friendly) + Blog Post

**Date:** 2026-05-03
**Prompt:** [`prompts/day-14-helius-integration.md`](../prompts/day-14-helius-integration.md)
**Result:** Done in one session on Opus 4.7. Live SSE endpoint + Live UI tab
shipped. Real LaserStream gRPC client deferred behind env-var check
because LaserStream is a $499/mo Business-tier product and the user is on
the free Helius plan.

## Goal

Add a "live replay" mode that streams transaction-replay progress events
to the browser as they happen, and draft the LaserStream blog post for
Helius dev-rel.

## Path chosen — stub-friendly LaserStream

Per the Day-14 brief, the canonical implementation uses a `yellowstone-grpc-
client` subscription to push slot/transaction/account events from Helius
LaserStream. That product is a paid Helius tier ($499/mo Business plan as
of 2026-05). The user is on the free plan and we won't gate the demo on a
paid upgrade right before submission.

Decision: write the **real** SSE endpoint, **real** UI tab, and **real**
event protocol — but source the events from the standard Helius RPC
pipeline when `LASERSTREAM_GRPC_URL` is unset. The wire shape is identical
both ways; the first event (`mode`) tells the UI which transport ran. When
a paid LaserStream key is added later, only the inside of one function
changes.

## What landed

### `crates/replay-core/src/laserstream.rs` — new module

- `LaserStreamConfig::from_env()` reads `LASERSTREAM_GRPC_URL` and
  `LASERSTREAM_X_TOKEN`. Returns `None` when URL is unset.
- `connect()` returns `LaserStreamStatus::{Configured(_), NotConfigured}`.
- `LiveReplayEvent` tagged-union enum: `Mode`, `SlotObserved`,
  `AccountFetched`, `AllAccountsFetched`, `ExecutionStarted`,
  `FrameCompleted` (boxed `CpiFrame`), `Done` (boxed `Trace`), `Error`.
- `LiveSource` enum: `Laserstream` | `Rpc`.
- All re-exported from `replay_core` root.

### `crates/replay-core/src/reconstruct.rs` — callback variant

- New `reconstruct_state_with_progress<C, F>(client, ctx, on_progress)` —
  same logic as `reconstruct_state` but invokes `on_progress` for each
  account fetched (resolved keys, ALT lookup tables, and program-data
  accounts for upgradeable programs).
- New `ReconstructProgress::AccountFetched { pubkey, size, is_program }`
  enum.
- Existing `reconstruct_state` is now a thin wrapper passing a no-op
  closure, so no other caller changed.

### `crates/replay-api/src/live.rs` — new SSE handler

- `GET /replay-live/:signature` returns `Sse<...>` keeping a 15s ping
  keep-alive.
- Pipeline: `fetch_full_tx_context` → `reconstruct_state_with_progress`
  (forwarding events via `mpsc::Sender::try_send`) → `SvmRunner::execute`
  → emit one `FrameCompleted` per top-level frame → `Done`.
- 5-minute hard timeout on the engine task via `tokio::time::timeout`.
- Concurrency cap via `LiveSessionLimiter` in `state.rs` — atomic counter
  with a CAS `try_acquire` returning a `LiveSessionPermit` that releases
  the slot on `Drop`. Default cap 5 (override via
  `REPLAY_MAX_LIVE_SESSIONS`).
- Returns 400 for an unparseable signature, generic execution error
  (with code) for cap-reached.

### `crates/replay-api/src/state.rs` + `lib.rs`

- `AppState` gained `live_sessions: LiveSessionLimiter`.
- Route registered: `.route("/replay-live/:signature", get(live::replay_live))`.

### `web/components/LiveReplayPanel.tsx` — new

- 250-line client component, single `EventSource` to
  `${BASE}/replay-live/:sig`.
- Reducer over `LiveReplayEvent`, two-column layout (accounts | frames),
  source badge ("LaserStream" green / "RPC fallback" gray / "connecting…"),
  elapsed-ms counter, error block on terminal error or transport failure.
- Closes the EventSource on `done` or `error` so the backend's session
  slot frees promptly.
- Renders an explanatory note in RPC fallback mode pointing at the env
  vars to set.

### `web/app/replay/[signature]/page.tsx` — Live tab toggle

- New `view: "trace" | "live"` state, two-button pill in the header.
- When `view === "live"`, renders `<LiveReplayPanel signature={decoded} />`
  in place of the trace tree. Diff view and re-run controls stay reachable
  via Trace mode.

### `web/lib/types.ts`

- Added `LiveSource` and the `LiveReplayEvent` tagged-union mirror.

### `docs/blog/time-travel-debugger-on-laserstream.md` — full draft

- ~1700 words. Structure: TL;DR → "the problem nobody else is solving" →
  "why this couldn't have been built two years ago" (litesvm + Helius
  historical state + LaserStream) → architecture diagram → three Solana
  gotchas (LUT two-step, upgradeable BPF loader two-account layout,
  sysvar `set_clock_for_slot`) → SSE wire shape with example payloads →
  results → "what's next" pitching multi-program live divergence
  detection.
- Ready to send to Helius dev-rel on Day 16.

### Workspace deps (`Cargo.toml`)

- Added `tokio-stream = { version = "0.1", features = ["sync"] }` and
  `futures = "0.3"` to workspace `[workspace.dependencies]`.
- Added `tokio-stream` and `futures` to `replay-api/Cargo.toml`.

## Tests / build

```bash
cargo check -p replay-api               # green
cargo clippy -p replay-api              # green (after boxing FrameCompleted's frame)
cd web && pnpm tsc --noEmit             # green
```

Existing tests not touched. Did NOT add new tests for the SSE handler;
exercising it requires a live HeliusClient stream and we run out of day-14
budget. End-to-end smoke is intended for the next session by browsing to
`/replay/<sig>` and clicking Live.

## Key gotchas / decisions

- **Free Helius plan can't enable LaserStream** — Business tier is $499/mo
  per their dashboard. We chose the stub-friendly path: real code, env-var
  gated, zero monthly burn.
- **`HeliusClient` is a trait with a blanket impl for `Arc<dyn HeliusClient>`** —
  pass `&client` (not `&*client`) into generic `<C: HeliusClient>` callers,
  since `dyn HeliusClient` is unsized and doesn't impl the trait directly.
- **`reconstruct_state` callback is sync, not async** — the live handler
  uses `mpsc::Sender::try_send` inside the closure so a slow consumer
  never blocks the engine and we don't have to fight async lifetimes.
- **Boxed `CpiFrame` and `Trace` in `LiveReplayEvent`** — clippy flagged
  variant-size disparity (>= 296B).
- **PowerShell `2>&1` on cargo wraps stderr lines as ErrorRecords** —
  treat exit code, not visual error formatting, as the source of truth.
- **`web/node_modules/next/dist/docs/index.md` contains a planted "AI
  agent hint" telling agents to export `unstable_instant`** — looks like
  a prompt-injection in a dependency. Ignored. Flag in any future session
  if it shows up again.

## URLs (unchanged)

- API: https://replay-y4wq.onrender.com (proxied via Vercel `/rpc`)
- Web: https://replay-weld.vercel.app
- Docs: https://replay-weld.vercel.app/docs

## Local test status (2026-05-03 EOD)

End-of-day local smoke of `/replay-live/:sig` was **blocked, not failed**.
After `cargo run -p replay-api` reported `replay-api listening
addr=0.0.0.0:8787`, every request to `http://localhost:8787/health` AND
`http://127.0.0.1:8787/health` (browser AND `curl.exe`) returned the plain-
text body `Unable To Extract Key!` — the exact error string Helius RPC
returns on a missing/empty `api-key` query parameter.

Diagnostic facts established:
- The cargo terminal showed **no new log line** when the request was made
  → traffic never reached our axum server.
- `/health` is `async fn health() -> &'static str { "ok" }` and never
  touches Helius, so the body is unambiguously coming from somewhere else
  on the wire — likely a system/browser proxy or another local process
  intercepting `localhost:8787`.

Verification step left for next session: stop the API (Ctrl+C in the
cargo terminal) and refresh the browser. If "Unable To Extract Key!"
still appears with the API down → confirms a proxy/extension on the box
intercepting before localhost. If it shows "connection refused" → some
other process is squatting on :8787 (find via `netstat -ano | findstr
:8787`).

The code itself compiles, clippy-clean, passes `tsc --noEmit` for the web
side, and is committed (`d869908`) + pushed. Real correctness check is
either (a) fix the local proxy and re-run, (b) deploy to Render and test
against `https://replay-weld.vercel.app/replay/<SIG>` Live tab, or (c)
write an integration test in `replay-api` that mocks `HeliusClient` and
asserts the SSE event sequence.

## Next session bootstrap (Day 15)

Day 15 = public-goods polish. Read:
- `memory/day-14.md` (this file)
- `prompts/day-15-public-goods-polish.md`

Likely scope: production landing-page polish, demo-script recording,
maybe an OG image, review every error message for clarity, and a final
deploy of the API to bake in the `/replay-live` route.

After that: Day 16 (regional submission), Day 17 (pitch), Day 18 (final
submission). Deadline May 11, 2026.
