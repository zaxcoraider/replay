# Day 4 — Trace Tree Builder (CPI Frames + CU Breakdown)

## Goal

Parse execution logs into a tree of `CpiFrame`s with CU consumption at each level. This is the structure the UI renders.

## Deliverables

1. `crates/replay-core/src/trace.rs`:
   ```rust
   pub fn build_trace(
       ctx: &TxContext,
       execution: &ExecutionResult,
       decoder: &AccountDecoder<'_>,
   ) -> Trace;
   ```

2. Log parser that turns this:
   ```
   Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 invoke [1]
   Program log: Instruction: Route
   Program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc invoke [2]
   Program log: Instruction: Swap
   Program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc consumed 45123 of 200000 compute units
   Program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc success
   Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 consumed 87234 of 1400000 compute units
   Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 success
   ```
   
   Into a tree:
   ```
   Jupiter (Route)
     └─ Whirlpool (Swap) — 45,123 CU
   Total: 87,234 CU
   ```

3. Instruction name inference: for each top-level instruction, use the IDL's instruction discriminator (first 8 bytes of instruction data) to look up the instruction name. Store as `CpiFrame.instruction_name`.

4. Account role inference: for each account in an instruction, use the IDL's `accounts` field to look up the role name. Store as `FrameAccount.role`.

5. Account deltas: diff `ctx.pre_state` vs `execution.accounts_after` for every account. Populate `AccountDelta` with decoded before/after.

## Log parser gotchas

- Parentheses in program logs can confuse naive regex. Use a proper line-based state machine.
- "Program failed to complete" and "Program returned error" are distinct. Handle both.
- Anchor emits `Program log: AnchorError caused by account: foo. Error Code: ...` — parse these into structured errors.
- Compute budget instructions emit `Program ComputeBudget111... invoke [1]` / `success` — these are legitimate frames but CU-free. Mark them specially.

## Return data

Some instructions return data via `sol_set_return_data`. Logs show `Program return: <program_id> <base64>`. Capture these in frames for completeness.

## Tests

Golden tests: for each of your three canonical signatures, capture the current output as a fixture JSON and assert equality. This catches regressions fast.

## End-of-day report

- Tree correctness: does a Jupiter swap's tree match Solscan's visualization?
- Performance: trace build time (target: <50ms for a 100-log tx).
