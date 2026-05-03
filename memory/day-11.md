# Day 11 — CLI Polish

**Date:** 2026-05-03
**Prompt:** [`prompts/day-11-cli-polish.md`](../prompts/day-11-cli-polish.md)
**Result:** Done in one session. Build clean.

## Goal

Polish the `replay-cli` binary: better output formatting with a spinner, a CPI frame table,
and a new `inspect` subcommand for per-account IDL-decoded inspection.

## What landed

### `crates/replay-cli/src/main.rs` — full rewrite

**Spinner helper:**
```rust
fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .unwrap().tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"]));
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
```
Used on three steps: "Fetching transaction…", "Reconstructing state (N accounts)…", "Replaying…"

**CPI frame table** — `print_frames()` recursive function:
- Prints each `CpiFrame` with depth-indented program name
- Three-column table: Program, CU Used, Depth
- Uses `indicatif`-style formatted output with `┌─┬─┐` borders
- Recurses into `frame.inner` for nested CPI calls

**`replay` subcommand:**
- Calls `replay_core::replay()` (NOT `svm::execute()` — ExecutionResult lacks frames)
- Prints summary: `✓ Replayed in Xms  ·  CU: Y  ·  Success/Failed`
- Prints full CPI frame table
- `--diff-logs` flag: shows lines added/removed vs mainnet

**`inspect` subcommand:**
- Args: `<SIGNATURE> --account <PUBKEY>`
- Fetches tx context, reconstructs state, finds the account
- Prints IDL-decoded JSON if available, raw base64 data otherwise

**`fetch` subcommand:**
- `--json` flag dumps raw `TxContext` as pretty JSON

**Tracing default changed** from `"info"` to `"warn"` to reduce noise.

### `crates/replay-cli/Cargo.toml`

Added `solana-sdk = { workspace = true }` for `Pubkey` type in `inspect`.

## Key decision

Must use `replay_core::replay()` for the full trace — `svm::execute()` returns `ExecutionResult`
which has no `frames` field. This trips you up if you read only the types.

## Tests / build

```bash
cargo build -p replay-cli   # clean
cargo clippy -p replay-cli  # clean
```

## Commits

```
(part of Day 11 session — see git log)
```

## Next session bootstrap (Day 12)

Day 12 = TypeScript SDK. Read:
- `memory/day-11.md` (this file)
- `prompts/day-12-ts-sdk.md`

Key deliverables:
1. `packages/replay-sdk-ts/` — `@replay/sdk` npm package
2. `ReplayClient` class, `Session` class
3. `replayHistorical()` CI helper
4. Build with tsup (ESM + CJS + .d.ts)
