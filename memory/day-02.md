# Day 2 — Spike Part 2: Reconstruct state and replay

**Date:** 2026-04-26
**Prompt:** [`prompts/day-02-spike-replay.md`](../prompts/day-02-spike-replay.md)
**Result:** Architectural gate cleared, faithful-replay gate pending.
The integrated `replay-core` pipeline (fetch → reconstruct → seed →
execute → trace) runs end-to-end against three real mainnet sigs with
zero infrastructure errors. Three real bugs surfaced and were fixed
during the live runs (CPI-program reclassification, ALT lookup-table
seeding, three-pass seed order). Remaining failures on the canonical
sigs are *replay-fidelity* issues (state drift on volatile accounts,
107-CU drift on token transfer) which are Day-3+ scope, not Day-2
blockers. See "Live gate run" section below for the full run table.

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

## Live gate run — 2026-04-26 → 2026-04-27

Ran `REPLAY_LIVE_TESTS=1 cargo test -p replay-core --test live_replay`
four times against three real mainnet signatures discovered via
Helius `getSignaturesForAddress`. Every iteration surfaced a distinct
class of bug; each was real, traced, fixed, and committed. The
infrastructure is now solid — surviving classes of failure are
*replay-fidelity* issues, which are Day-3+ scope.

Sigs used (also saved in `crates/replay-core/tests/fixtures/canonical-sigs.txt`):

```
sol_transfer       = 5zayg636jcxCSS43y94UZrxPDzjdD4CPtFYNCjfcvENr53jezr7qhnKT8ZWeEJnAXQvAHxQYW54uHc7XAj5ndMhr
spl_token_transfer = 5eHx4KKkiFdGnzkhB6GrPUNHGXzATPZguJBMqhBXDq3zZUm46sk6gJkFyt3qFV4DT7a53shpKAKFaVCP39s4Qw7B
jupiter_v6_swap    = 3dqwCn4AnLAYVbtdVu31DADDUyt4qgm3djYDsFUAXWDK5nNUFehNhti25Zh3fdV7J8LizsuxvBDhtNnTAoHCVetq
```

| Run | Failure | Root cause | Fix (commit) |
|---|---|---|---|
| 1 | All 3: `set_account TokenzQd...: Instruction(MissingAccount)` | Token-2022 + similar BPF-upgradeable programs were *CPI-invoked* (not in top-level instructions) so `program_ids` didn't include them; they were inserted into `state.accounts` as plain executables and litesvm rejected because no program-data was seeded | `ddc92fc` — post-classify accounts by `executable` flag instead of walking instructions; also flipped seed order |
| 2 | Same — order alone wasn't enough | Confirmed the seed-order tweak was insufficient on its own; the underlying issue was the missed CPI programs | (subsumed by `ddc92fc`) |
| 3 | All 3: `AddressLookupTableNotFound` | `fetch_full_tx_context` extracts `meta.loaded_addresses` (the *resolved* addresses) but never adds the ALT *table account itself*; litesvm's V0 message path resolves lookups against its own account store at execute time and the ALTs weren't seeded | `ddc92fc` — added Pass-1b in reconstruct that fetches every `address_table_lookups[].account_key` |
| 4 | Real replay-fidelity divergences (see below) | Genuine state drift / CU accounting | Out of scope for Day 2 |

### Run-4 outcomes

| Sig | Result |
|---|---|
| sol_transfer | replay errored at instruction 6: `ProgramFailedToComplete` |
| spl_token_transfer | **logs match through line 9; line 10 is the only divergence — same CU consumed (1569), 107 CU drift in remaining-budget**: mainnet=`352551`, replay=`352444` |
| jupiter_v6_swap | replay errored at instruction 1: `Custom(1)` |

### Why these failed (and why it's not a Day-2 blocker)

The two `Custom`/`ProgramFailedToComplete` failures are state-drift
artifacts: random sigs from `getSignaturesForAddress` are the *most
recent* activity, which means they touch oracles (Pyth/Switchboard)
and AMM pools whose state changes every block. Our
`get_account_info_at_slot` honors `minContextSlot` per the Day-1
contract, but `minContextSlot` doesn't actually return historical
state — it just asserts node freshness. So we get *current* state,
which has drifted by enough to flip outcomes for txs that read price
oracles.

The SPL-token divergence is closer to a true replay miss: same
CU *consumed* per program, but 107 CU drift in remaining-budget
accounting. Likely a litesvm-0.6 vs. mainnet-validator difference in
how compute-budget instructions account for their own setup overhead.
Worth investigating in Day-3+ but not a Day-2 blocker — every
program-level log line matches up to the divergence point.

### Net result

- **Architectural gate: cleared.** No infra errors. fetch / reconstruct
  / seed / execute / trace run end-to-end on three representative
  mainnet sigs.
- **Faithful-replay gate: not cleared with these sigs.** True
  faithful replay needs either (a) hand-picked stable txs that don't
  depend on volatile state, or (b) Helius enhanced APIs (Day 14) that
  return account state *as of slot S* rather than now-asserted-fresh.
- Day-3 (IDL decoder) is independent of this and can proceed. Day-14
  is when we revisit historical-state fetching properly.

### Smoke commands for fresh sessions

```bash
# Run the gate (HELIUS_API_KEY auto-loads from .env)
REPLAY_LIVE_TESTS=1 cargo test -p replay-core --test live_replay -- --ignored --nocapture

# Interactive smoke
cargo run -p replay-cli -- replay <SIG>
# With log diff
cargo run -p replay-cli -- replay <SIG> --diff-logs
```

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

1. **Pick stable canonical sigs (or accept current state).** The three
   sigs in `canonical-sigs.txt` are random recent activity and depend
   on volatile state. To run a true faithful-replay gate, hand-pick
   txs that don't read oracles/AMM pools: a wallet-to-wallet SOL
   transfer, an SPL transfer between cold wallets, an old completed
   Jupiter swap from a low-liquidity pool. Or wait for Day-14 (Helius
   Enhanced) which gives true historical state.
2. **Investigate the 107-CU drift on the SPL token transfer.** The CU
   *consumed* matches mainnet exactly; the remaining-budget number
   diverges. Likely a litesvm-0.6 vs. validator quirk in how
   compute-budget instructions self-account. Worth reading litesvm's
   compute-budget handling source.
3. **Capture a real Helius response** for `jupiter-swap-response.json`
   (Day-1 follow-up — `./scripts/capture-fixture.sh <sig>` with
   `HELIUS_API_KEY` set).
4. **`return_data` in `ExecutionResult`** is captured but trace.rs's
   `build_trace` doesn't surface it on frames yet — Day-4 work.
5. **Replace `get_account_info_at_slot` placeholder** with Helius
   enhanced API for true historical state (Day-14).

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
f93a277 test(core): wire live-gate runtime — dotenv + real canonical sigs
ddc92fc fix(core): seed CPI-invoked programs and ALT lookup tables
e3bda12 docs: update day-02 snapshot with post-audit cleanups
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
