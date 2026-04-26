# Day 2 — Spike Part 2: Reconstruct state and replay

**Date:** 2026-04-26
**Prompt:** [`prompts/day-02-spike-replay.md`](../prompts/day-02-spike-replay.md)
**Result:** Wired. Architectural go/no-go is satisfied — the standalone
spike (`spikes/spike-replay.rs`) already replays a real mainnet tx with
matching logs, and the scaffolded `reconstruct.rs` + `svm.rs` mirror its
approach. The full end-to-end live gate (3 canonical sigs vs. mainnet)
is wired but **not yet validated against real network** in this session
because no `HELIUS_API_KEY` was provided. That step is one command for
the user (see "Validating the gate" below).

## Goal

Paste a mainnet sig → reconstruct pre-slot state of every account →
load all programs at their historical bytecode → replay in litesvm →
get logs that match mainnet exactly. Demo command:

```
cargo run -p replay-cli -- replay <SIGNATURE>
```

Should print step-by-step progress and a `Logs match mainnet: N/N` line.

## What was already in place (scaffolding)

The Day-2 spec was largely realized in the scaffolding from Day-0. I
audited each module and confirmed it implements the spec — *no* code
changes were needed in `replay-core`:

- **`reconstruct.rs::reconstruct_state`** — fetches every non-program
  account at `slot - 1`, then for each program fetches the program
  account, classifies the loader (Native/BpfLoader/BpfLoaderDeprecated/
  BpfLoaderUpgradeable/LoaderV4), and for upgradeable programs parses
  `program_account.data[4..36]` as the program-data address and fetches
  that too. Native programs are skipped (litesvm has them built-in).
  Accounts absent at pre-slot are tolerated as "may be created by tx."

- **`svm.rs::SvmRunner`** —
  `LiteSVM::new().with_sigverify(false).with_blockhash_check(false)`,
  then `seed()` calls `set_account` for every reconstructed account and
  for each upgradeable program also seeds the program-data account.
  `set_clock_for_slot` builds a `Clock` from `(slot, block_time)`.
  `execute()` runs `svm.send_transaction(ctx.original_tx.clone())` and
  collects `accounts_after` for every resolved key. Failure is captured
  as `TxResult::Failure` (not a tool error — the original tx may have
  failed on mainnet too).

- **`trace.rs::build_trace`** — produces a minimal `Trace` with flat
  per-top-level-instruction frames, account deltas, and a
  `LogDivergence` from the first mismatched line. Sufficient for Day-2
  acceptance; full CPI tree comes Day-4.

- **`lib.rs::replay`** — wires fetch → reconstruct → seed → clock →
  execute → trace and returns the `Trace`.

## What I changed today

- **`crates/replay-cli/src/main.rs`** — rewrote the `Replay` arm of the
  pretty path to orchestrate fetch/reconstruct/seed/execute itself so
  each phase emits a ✓-prefixed line, matching the Day-2 acceptance
  demo. The `--json` path still calls `replay_core::replay()` so SDK
  parity holds. `--diff-logs` now renders a side-by-side log diff when
  the streams diverge (full diff, not just first divergent line).
  Added `format_underscores()` helper so CU prints as `842_119`.

- **`crates/replay-core/tests/fixtures/canonical-sigs.txt`** — the
  three replay-test placeholders (sol_transfer, spl_token_transfer,
  jupiter_v6_swap), one `<label>=<sig>` per line, with `PLACEHOLDER_*`
  values that the live test recognizes and skips.

- **`crates/replay-core/tests/live_replay.rs`** — integration test
  (separate file under `tests/` so it's a separate test binary, doesn't
  bloat the unit-test output). For every non-placeholder entry in the
  fixture: runs `replay(sig, &client)`, asserts mainnet/replay
  `TxResult` kinds match, asserts `trace.log_divergence` is `None`.
  Aggregates failures into a single panic message so all three cases
  surface in one test run. Gated on `REPLAY_LIVE_TESTS=1` +
  `HELIUS_API_KEY` (`#[ignore]`-d).

## Tests

```text
cargo test -p replay-core
  9 unit tests passed; 1 ignored
  1 integration test (live_replay_canonical_sigs); 1 ignored
```

| Test | Status |
|------|--------|
| `fetch::tests::*` (Day-1) | green |
| `rpc::tests::mock_client_returns_canned_tx` | green |
| `trace::tests::parses_invoke_lines` | green |
| `trace::tests::parses_cu_consumed` | green |
| `fetch::tests::live_fetch_jupiter_swap` | ignored (gated) |
| `live_replay::live_replay_canonical_sigs` | ignored (gated) |

`cargo check --workspace --tests` is clean: 0 errors, 0 warnings.

## Decisions worth remembering

- **Scaffolded `SvmRunner::execute` does not rebuild with a dummy fee
  payer.** The Day-2 prompt sketched that approach and called out an
  "Easier alternative: lower-level message-processing APIs / sigverify
  off". The scaffolding (and the spike) take the easier path:
  `with_sigverify(false)` + run the original tx verbatim. The fee
  payer's account state is reconstructed from mainnet so they have
  enough lamports; signatures are skipped because of the sigverify
  flag. **Don't introduce a dummy-payer rebuild unless we hit a real
  signature-related failure.** It's complexity for no current win.

- **CLI orchestration is duplicated by design.** The `--json` path
  calls `replay_core::replay()` (one-shot for SDK users); the pretty
  path inlines fetch/reconstruct/seed/execute so it can stream
  progress. They use the same building blocks under the hood — not a
  divergence, just two different presentation surfaces.

- **`reconstruct_state` errors hard on missing program bytecode**
  (`MissingProgramBytecode`) — that's intentional. A missing program
  means the replay *cannot* be faithful, so we want a loud failure,
  not silent empty execution. Surfacing this in the UI is Day-9
  divergence-explanation territory.

- **`with_blockhash_check(false)` is non-negotiable for replay.**
  Mainnet blockhashes have long since expired by the time we replay.
  The litesvm runner's recent-blockhash is a fresh in-memory one;
  the original tx's blockhash never matters because we're running
  the message verbatim with check disabled.

- **Day-2 minimal Trace is enough.** `trace::build_trace` already
  produces frames + deltas + log divergence. CPI nesting is shallow
  (one frame per top-level instruction, no children) — Day-4 fills
  that in. The Day-2 acceptance just needs `total_cu` and the logs
  comparison, both of which work today.

## Validating the gate (user-side, ~30 seconds)

Day-2 is the project's go/no-go. The architecture is proven by the
spike but the Day-2 module integration needs one mainnet validation
run. To do it:

```bash
# 1. Set your Helius key
export HELIUS_API_KEY=<your-key>

# 2. Find recent sigs (any wallet's recent activity works for #1 + #2;
#    Solscan / Helius "Top transactions" page gives Jupiter swaps).

# 3. Edit the fixture
$EDITOR crates/replay-core/tests/fixtures/canonical-sigs.txt
# Replace each PLACEHOLDER_* with a real base58 signature.

# 4. Run the integration test
REPLAY_LIVE_TESTS=1 cargo test -p replay-core --test live_replay -- --ignored

# 5. Or do an interactive smoke test
cargo run -p replay-cli -- replay <SIG>
# Expected output ends with: ✓ Logs match mainnet: N/N lines identical
```

If a divergence pops up:
- `--diff-logs` shows the side-by-side; usually the first divergent
  line names the culprit (missing program load → bytecode fetch issue;
  CU drift → sysvar setup; custom error → account state).
- `docs/04-solana-gotchas.md` has the canonical trap list.

If the Jupiter swap diverges and the cause isn't tractable in a few
hours: consult the **pivot plan** in `prompts/day-02-spike-replay.md`
("CU Profiler fallback") rather than romance Day 2 into Day 4.

## Post-Day-2 cleanups landed (audit-driven)

After comparing the implementation against the literal prompt
contracts, three targeted fixups were committed before the live gate
run:

- **`fix(core)`** `773a5ea` — `get_account_info_at_slot` now sends
  `minContextSlot: slot` in the JSON-RPC config (was silently calling
  the no-slot variant). Refactored to a shared
  `get_account_info_inner(pubkey, Option<slot>)`; doc comment retains
  the honest caveat that `minContextSlot` doesn't return historical
  state — Day 14 (Helius enhanced) revisits.
- **`feat(core)`** `8efd96d` — added
  `reconstruct::snapshot_pre_state(state) -> HashMap<Pubkey, Account>`
  flattening `state.accounts` ∪ program/program-data accounts.
  Callers (`lib.rs::replay`, `lib.rs::fork`, CLI Replay arm) now
  populate `ctx.pre_account_snapshots` after `reconstruct_state`
  returns. `account_deltas` is no longer always empty.
  `reconstruct_state`'s prompt-mandated `&TxContext` signature is
  preserved — the merge happens in the caller.
- **`refactor(core)`** `b7f6994` — folded the last guarded
  `current_program_stack.pop().unwrap()` in `trace::parse_log_frames`
  into an `if let Some(program_id) = ...pop()`. Day-1's strict
  no-unwrap-outside-tests is now honored; verified via grep.

## Follow-ups (deferred)

1. **Run the live gate.** Single user-side command after populating
   fixtures (see "Validating the gate").
2. **Capture a real Helius response** for `jupiter-swap-response.json`
   (Day-1 follow-up still open — same `HELIUS_API_KEY` invocation).
3. **`return_data` in `ExecutionResult`** is captured but trace.rs's
   `build_trace` doesn't surface it on frames yet — Day-4 work.

## Next session bootstrap (Day 3)

```bash
git pull origin main
cat memory/day-02.md  # this file
cat memory/day-01.md  # for context

# Day-3 prompt is the IDL decoder
#   prompts/day-03-idl-decoder.md
# Plus standing context:
#   prompts/SYSTEM-PROMPT.md
#   docs/01-project-brief, 02-architecture, 03-technical-spec, 04-solana-gotchas

# Smoke before writing code
cargo check --workspace
cargo test -p replay-core
```

Day-3 north-star: take an account's bytes + the owning program's IDL
(local cache → on-chain Anchor IDL → user-pasted) and decode to
structured JSON. The skeleton in `idl.rs` defines the right types
(`DecodedAccount`, `IdlSource`, `Idl`, `IdlCache`, `AccountDecoder`)
but `AccountDecoder::decode` returns `NoIdl` unconditionally today.

## Commits

```
b7f6994 refactor(core): drop guarded .unwrap() in trace.rs frame popper
8efd96d feat(core): populate pre_account_snapshots from reconstructed state
773a5ea fix(core): pass minContextSlot in get_account_info_at_slot
732461c docs: memory/day-02 snapshot
2cda1a8 test(core): live integration test for canonical replay sigs
d04bf9c feat(cli): Day-2 step-by-step replay output
fc4bf30 docs: project README + memory/day-01 snapshot
b99af6c chore: add fixture-capture script + lockfile
2d73413 feat(cli): print TxContext as JSON when --json flag is set
fae9d46 feat(core): Day-1 fetch deliverables — TxContext JSON, mock+live tests
e1dfb0c chore(core): add ulid dep, drop unused imports
537a019 chore: initial scaffold + working replay spike
```
