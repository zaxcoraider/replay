# Local memory

One file per working day. Each file is a self-contained snapshot of where
the project stands after that day's session, written so a fresh Claude
Code session can recover full state without reading every commit.

## Convention

Each `day-XX.md` file should answer:

1. **Goal** — what the day's prompt asked for.
2. **What landed** — files touched, key code added, with line refs where
   useful.
3. **Tests** — what runs green; what's gated; what's skipped and why.
4. **Decisions worth remembering** — non-obvious calls (dep versions,
   API quirks, things that surprised us).
5. **Follow-ups** — work explicitly deferred, in priority order.
6. **Next session bootstrap** — exact commands + prompt files to open.

## Files

- [`day-01.md`](day-01.md) — fetch path complete (HeliusClient, fetch_full_tx_context, mock+live tests, CLI --json).
- [`day-02.md`](day-02.md) — replay in LiteSVM, historical state reconstruction.
- [`day-03.md`](day-03.md) — IDL-aware account decoder, bundled IDLs (Jupiter, Whirlpool, Drift, Kamino).
- [`day-04.md`](day-04.md) — CPI trace tree (nesting, per-frame CU, decoded args).
- [`day-05.md`](day-05.md) — fork sessions, HTTP API (axum, rate limiting, 7 integration tests).
- [`day-06.md`](day-06.md) — web UI scaffold (landing page, 3-panel replay view).
- [`day-07.md`](day-07.md) — Timeline scrubber (CU bar, clickable segments).
- [`day-08.md`](day-08.md) — Account mutator UI (fork → edit fields → re-run).
- [`day-09.md`](day-09.md) — Diff view (baseline vs forked, side-by-side).
- [`day-10.md`](day-10.md) — Demo preload, Dockerfile, deployment config, landing page polish.
- [`day-11.md`](day-11.md) — CLI polish: indicatif spinner, CPI frame table, `inspect` subcommand.
- [`day-12.md`](day-12.md) — TypeScript SDK (`@zaxcoraider/replay-sdk`), published to npm.
- [`day-13.md`](day-13.md) — Rust SDK (`replay-sdk`), docs site, published to npm + crates.io.
