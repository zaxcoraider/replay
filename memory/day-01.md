# Day 1 — Spike Part 1: Fetch and parse a mainnet transaction

**Date:** 2026-04-26
**Prompt:** [`prompts/day-01-spike-fetch-tx.md`](../prompts/day-01-spike-fetch-tx.md)
**Result:** Done. All Day-1 acceptance items satisfied except the real
Helius-captured fixture (deferred — needs an API key).

## Goal

Given a mainnet tx signature, produce a structured `TxContext` containing
every account the tx touches (static + LUT-resolved + program + program-data)
plus the slot, block-time, mainnet logs, mainnet result, compute-budget
instructions extracted separately, and pre/post balances. **No replay yet.**

## Pre-Day-1 setup (this session)

Before Day-1 work, the project layout was reorganized:

- Extracted `replay-plan.zip` into the repo root (was un-extracted).
- Promoted `scaffolding/*` to the project root so `crates/`, `docs/`,
  `prompts/` sit at the top level.
- Moved the working spike to `spikes/spike-replay.rs` for reference.
- Initialized a fresh git repo at `Desktop/replay/` (was previously nested
  under the home-dir git repo) and pushed to
  `https://github.com/zaxcoraider/replay.git` on `main`.
- Added `.claude/settings.local.json` to `.gitignore`.

## What landed

### Compile fixups (`KNOWN-FIXUPS.md`)

Only one compile error surfaced, not the eight predicted:

- **`replay-core/Cargo.toml`** — added `ulid = { workspace = true }`.
  `session.rs:33` uses `ulid::Ulid::new()` but the dep wasn't pulled in.
- **`replay-core/src/svm.rs`** — dropped unused
  `transaction::VersionedTransaction` import.
- **`replay-core/src/session.rs`** — dropped unused `ExecutionResult` import.

The other 7 fixups (`meta.logs` vs `log_messages`, `bincode 1.x` vs `2.x`,
`solana_sdk::pubkey!` macro, etc.) all turned out correct as written
against the resolved versions.

### Day-1 deliverables

- **`replay-core/src/types.rs`** — `TxContext` now derives `Serialize`.
  Custom `serialize_account_snapshots` emits `HashMap<Pubkey, Account>` as
  base58-keyed objects with base64-encoded `data`. Required so the CLI
  `--json` output isn't a JSON array of u8 per account.
- **`replay-core/src/test_support.rs` (new)** — `MockHeliusClient` extracted
  here, gated `#[cfg(test)] pub(crate)`. Removed the duplicate that lived
  in `rpc.rs::tests`.
- **`replay-core/src/fetch.rs`** — added 4 mock-backed tests + 1 live test:
  - `fetch_full_tx_context_resolves_luts_and_extracts_compute_budget`
  - `fetch_full_tx_context_maps_failure` (asserts `Custom(6001)` extraction)
  - `fetch_full_tx_context_returns_tx_not_found`
  - `tx_context_serializes_to_json`
  - `live_fetch_jupiter_swap` (`#[ignore]`, gated on `REPLAY_LIVE_TESTS=1`,
    skips with message if `tests/fixtures/demo-signature.txt` is a
    placeholder)
  Synthetic V0 tx is built in-memory with bincode/base64 round-trip, so
  the mocked tests are hermetic.
- **`replay-cli/src/main.rs`** — `Fetch` arm honors `--json` flag; without
  it shows pretty summary including `cb_ix=` count. Added context to the
  signature parse error.

### Tooling

- **`scripts/capture-fixture.sh`** — captures a real Helius
  `getTransaction` response into
  `crates/replay-core/tests/fixtures/jupiter-swap-response.json` and
  writes the sig to `demo-signature.txt`. Needs `HELIUS_API_KEY`.
- **`crates/replay-core/tests/fixtures/demo-signature.txt`** — placeholder
  signature; live test recognizes "PLACEHOLDER" and skips.

## Tests

```text
cargo test -p replay-core
  9 passed; 0 failed; 1 ignored
```

| Test | Status |
|------|--------|
| `fetch::tests::extract_custom_error_code_*` (×2) | green |
| `fetch::tests::fetch_full_tx_context_resolves_luts_and_extracts_compute_budget` | green |
| `fetch::tests::fetch_full_tx_context_maps_failure` | green |
| `fetch::tests::fetch_full_tx_context_returns_tx_not_found` | green |
| `fetch::tests::tx_context_serializes_to_json` | green |
| `rpc::tests::mock_client_returns_canned_tx` | green |
| `trace::tests::*` (×2, scaffolding) | green |
| `fetch::tests::live_fetch_jupiter_swap` | ignored (needs `REPLAY_LIVE_TESTS=1` + real fixture sig) |

## Decisions worth remembering

- **Dep versions stay where the scaffolding had them.** `litesvm = "0.6.1"`
  + `solana-sdk = "2.2.1"` actually compile together. The spike uses
  `litesvm 0.11` + `solana-sdk` 3.x sub-crates — that's a *future* bump,
  not a Day-1 requirement. Stay on 0.6/2.2 unless something actively
  forces an upgrade. **Cold compile takes ~36 minutes**; incremental ~10s.
- **`HeliusClient::get_transaction` returns `Result<Option<FetchedTx>, _>`,
  not `Result<FetchedTx, _>`** as the Day-1 prompt sketched. The Option
  is more correct (None = tx not found); `fetch_full_tx_context` maps
  None → `ReplayError::TxNotFound`.
- **`meta.err` shape:** `{"InstructionError": [ix_index, {"Custom": code}]}`
  for program errors. Anchor errors are `Custom(N)` here. Other err shapes
  (`InsufficientFundsForRent`, `BlockhashNotFound`) yield `code: None`.
- **Account ordering after LUT resolution:**
  `static_account_keys ++ loaded_writable ++ loaded_readonly`. The runtime
  uses this exact order for `program_id_index`. Get this off-by-one and
  compute-budget extraction silently misclassifies instructions.
- **No `add_program_from_file` for replay.** Use `set_account` on both
  the program account and program-data account; let litesvm's loader
  wire them up. (See `docs/04-solana-gotchas.md` §1.) This is Day-2's
  problem; noted here so we don't forget.

## Follow-ups (deferred)

1. **Capture a real Helius fixture.** Run
   `HELIUS_API_KEY=... ./scripts/capture-fixture.sh <jupiter-sig>` once a
   key is available. Replaces the synthetic in-memory test setup with a
   recorded mainnet response.
2. **Replace `demo-signature.txt` placeholder** with a real recent
   Jupiter v6 swap sig so the live test can run.
3. **`replay()` / `fork()` in `lib.rs`** still depend on Day-2+ modules
   (svm/reconstruct/trace/idl) which compile but aren't validated against
   real litesvm semantics yet — Day-2 work.
4. The `rpc.rs::HeliusRpcClient::get_account_info_at_slot` impl is a
   placeholder that calls `get_account_info` — `minContextSlot` doesn't
   give true historical state (see gotcha §12). Day-2 will revisit:
   either Helius Enhanced API or accept current-state replays.

## Next session bootstrap (Day 2)

```bash
# 1. Pull
git pull origin main

# 2. Read this snapshot first
cat memory/day-01.md

# 3. Open the Day-2 prompt + system prompt in Claude Code
#    Files to paste at session start:
#      prompts/SYSTEM-PROMPT.md
#      prompts/day-02-spike-replay.md
#    Plus context-on-demand:
#      docs/01-project-brief.md, 02-architecture.md, 03-technical-spec.md, 04-solana-gotchas.md
#      memory/day-01.md  ← this file

# 4. Smoke-check before writing code
cargo check --workspace
cargo test -p replay-core
```

Day-2 north-star: a real mainnet Jupiter swap replays in litesvm and
produces logs that match mainnet character-for-character (modulo
clock-dependent output). The standalone spike at
`spikes/spike-replay.rs` already proves this is achievable — the Day-2
work is integrating it into `replay-core/svm.rs` + `reconstruct.rs`.

## Commits

```
b99af6c chore: add fixture-capture script + lockfile
2d73413 feat(cli): print TxContext as JSON when --json flag is set
fae9d46 feat(core): Day-1 fetch deliverables — TxContext JSON, mock+live tests
e1dfb0c chore(core): add ulid dep, drop unused imports
537a019 chore: initial scaffold + working replay spike
```
