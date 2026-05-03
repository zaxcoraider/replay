# Day 12 — TypeScript SDK

**Date:** 2026-05-03
**Prompt:** [`prompts/day-12-ts-sdk.md`](../prompts/day-12-ts-sdk.md)
**Result:** Done in one session. Build clean, published to npm.

## Goal

Ship a first-class TypeScript SDK so JS/TS developers can replay transactions, fork sessions,
and run historical regression batches from Node.js or browser code.

## What landed

### `packages/replay-sdk-ts/` — new package

Published as `@zaxcoraider/replay-sdk@0.1.0` (note: `@replay` scope is owned by someone else on npm).

**`package.json`:**
- Name: `@zaxcoraider/replay-sdk`, version: `0.1.0`
- `"types"` export condition must come BEFORE `"import"` / `"require"` in exports map (fixed after warning)
- `tsup` build: ESM + CJS + `.d.ts` / `.d.mts`

**`src/types.ts`:**
- `Trace`, `CpiFrame`, `AccountDelta`, `TraceDiff`, `TxResult`, `LogDivergence`
- `Mutation` discriminated union: `{ type: "field", path, new_value }` | `{ type: "raw_bytes", offset, bytes, extend }` | `{ type: "lamports", new_value }`

**`src/index.ts`:**
- `ReplayClient({ apiUrl })` — wraps fetch calls to the HTTP API
  - `.replay(sig)` → `Trace`
  - `.fork(sig)` → `Session`
- `Session` — holds `session_id`, wraps mutate/execute/diff calls
  - `.mutate(pubkey, mutation)` → void
  - `.execute()` → `Trace`
  - `.diff()` → `TraceDiff`
- `parseResponse<T>()` helper — reads text first, parses JSON, throws readable error on failure

**`src/testing.ts`:**
- `replayHistorical(config)` — replays a list of sigs, reports failures + CU regressions
- `loadSignatures(filePath)` — reads newline-delimited `.txt` file (Node.js only)

## Dist output

```
dist/index.mjs, dist/index.js, dist/index.d.ts, dist/index.d.mts
dist/testing.mjs, dist/testing.js, dist/testing.d.ts, dist/testing.d.mts
dist/chunk-*.mjs (shared types chunk)
```

## Key decisions

- `@replay` scope → 404 on publish (owned by another user). Renamed to `@zaxcoraider/replay-sdk`.
- Used npm Automation classic token to bypass 2FA on publish.
- `"types"` export condition must be first in exports map (before `"import"` / `"require"`), otherwise bundlers warn it's unreachable.

## Tests / build

```bash
cd packages/replay-sdk-ts && pnpm build   # ESM + CJS + DTS all green
```

## Commits

```
(part of Day 12 session — see git log)
```

## Next session bootstrap (Day 13)

Day 13 = Rust SDK. Read:
- `memory/day-12.md` (this file)
- `prompts/day-13-rust-sdk.md`

Key deliverables:
1. `crates/replay-sdk/` — `replay-sdk` crate on crates.io
2. `ReplayClient`, `Session` structs
3. `replay_historical()` batch helper
4. Publish `replay-core` first, then `replay-sdk`
