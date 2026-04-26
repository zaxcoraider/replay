# Day 11 — CLI Polish + Install

## Goal

Ship `replay-cli` as a first-class product, not an afterthought. `cargo install replay-cli` works. Every subcommand has `--help` and docs.

## Deliverables

1. `crates/replay-cli/src/main.rs` with `clap`-based subcommands:
   - `replay <signature>` — one-shot replay. Flags: `--json`, `--diff-logs`, `--rpc <url>`, `--verbose`.
   - `replay fork <signature>` — interactive REPL.
   - `replay diff <sig1> <sig2>` — cross-replay and diff two txs.
   - `replay serve [--port <p>]` — spawn the API server.
   - `replay inspect <signature> --account <pubkey>` — print decoded state of one account at the tx's pre-slot.

2. REPL mode (`rustyline`):
   ```
   replay> list-accounts
   [0] JUP6Lk... (Jupiter v6) — 36 bytes
   [1] 8sLbNZ... (Whirlpool Config) — 108 bytes
   ...
   replay> show 1
   Whirlpool Config {
     whirlpools_config: ...,
     fee_rate: 30,
     protocol_fee_rate: 300,
     ...
   }
   replay> set 1.fee_rate 9999
   Mutation applied.
   replay> execute
   Replay failed: SwapAmountTooSmall (Whirlpool error 0x1780)
   replay> diff
   [shows before/after]
   replay> quit
   ```

3. Installation:
   - Publish to crates.io: `cargo publish -p replay-sdk && cargo publish -p replay-cli`
   - Homebrew tap (optional, Day 15 if time): `brew install replay`
   - `curl | sh` installer that grabs the right prebuilt binary for the OS from GitHub Releases.
   - Release action on GitHub: tag-triggered workflow that builds `linux-x86_64`, `linux-aarch64`, `darwin-x86_64`, `darwin-aarch64`, and uploads.

4. Docs:
   - `crates/replay-cli/README.md` — copy-pasteable usage examples.
   - Man page (optional): `clap_mangen` to auto-generate.
   - `replay --help` shows full subcommand tree.

## Interaction polish

- Colorized output with `owo-colors`. Dim for addresses, bold for program names, red for errors.
- Progress bars on long operations: `indicatif`.
- `--json` flag everywhere for CI use.
- Sensible defaults: `--rpc` defaults to `$HELIUS_API_KEY` env → Helius URL.

## End-of-day

- `cargo install --path crates/replay-cli` then `replay <signature>` works from any directory.
- Write `docs/CLI-EXAMPLES.md` with 10 real usage examples.
- Commit a demo terminal session (asciinema recording) to the repo.
