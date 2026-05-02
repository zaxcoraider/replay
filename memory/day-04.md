# Day 4 — CPI Trace Tree

**Date:** 2026-05-02
**Prompt:** [`prompts/day-04-trace-tree.md`](../prompts/day-04-trace-tree.md)
**Result:** Done in one session (work was pre-started in working tree, committed this session).

## Goal

Turn the flat `Vec<CpiFrame>` stub into a real CPI tree. Parse Solana's
program logs as a state machine. Decode instruction args via bundled IDLs.

## What landed

### `crates/replay-core/src/trace.rs` (full rewrite)

Log-line state machine in `parse_log_frames`:
- `"Program X invoke [N]"` → push frame onto stack at depth N-1
- `"Program X consumed K of M compute units"` → annotate CU on matching frame
- `"Program X success"` / `"Program X failed: <reason>"` → pop, nest into parent or roots
- `"Program return: X <b64>"` → capture return_data
- `"Program failed to complete: ..."` → CU exhaustion, mark top-of-stack failed
- Everything else → append to current frame's `logs` list; parse `"Program log: Instruction: <name>"` eagerly

`populate_instruction_details` walks the completed tree (top-level frames only
get `data_hex` from the original tx message) and calls
`AccountDecoder::decode_instruction_local` to fill `instruction_name`,
`decoded_args`, and `FrameAccount.role` from bundled/disk IDLs.

Unclosed frames (truncated/malformed logs) drained to roots at end.

### `crates/replay-core/src/idl.rs` (additions)

- `IdlCache::get_local` — sync lookup: bundled set → disk cache (no network)
- `AccountDecoder::decode_instruction_local` — sync instruction decode for trace builder
- `anchor_instruction_discriminator(name)` — `sha256("global:<name>")[..8]`
- `decode_instruction_data(idl, data)` — discriminator match + Borsh arg decode + account role names

## Tests

```
cargo test -p replay-core --lib
  27 passed; 0 failed; 1 ignored
```

9 new trace tests: `parse_invoke_extracts_depth`, `parse_consumed_extracts_cu_and_remaining`,
`parse_outcome_success_and_failure`, `parse_return_data_captures_b64`,
`anchor_log_instruction_name_extracted`, `builds_nested_cpi_tree`,
`handles_compute_budget_frame`, `failed_frame_propagates_error`,
`captures_return_data`, `three_level_nesting`.

## Commits

```
3ba97ec feat(core): Day-4 CPI trace tree + instruction decode
```

## Next session bootstrap (Day 5 → already done)

Day 5 was also completed in the same session. See `memory/day-05.md`.
