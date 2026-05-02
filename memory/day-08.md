# Day 8 — Account Mutator UI

**Date:** 2026-05-02
**Prompt:** [`prompts/day-08-account-mutator.md`](../prompts/day-08-account-mutator.md)
**Result:** Done in one session. TypeScript clean, build clean, 26 Rust tests passing.

## Goal

Complete the mutation loop: fork session on page load → edit IDL-decoded account fields → apply mutations via API → Re-run button to execute and see updated trace.

## What landed

### `crates/replay-core/src/idl.rs`

New public function:
```rust
pub fn apply_field_mutation(idl: &Idl, data: &[u8], path: &str, new_value: &Value) -> Result<Vec<u8>, ReplayError>
```
- Finds account type by discriminator (existing `anchor_account_discriminator`)
- Decodes full account to JSON via existing `decode_type`
- Navigates dot-separated `path` via `set_json_path` and replaces the value
- Re-encodes the mutated JSON back to Borsh via `encode_type`
- Returns new byte vec with original 8-byte discriminator prepended

New private helpers:
- `set_json_path(val, path, new_val)` — dot-separated path navigation + update
- `encode_type(idl, ty, value, buf)` — full Borsh encoder mirroring `decode_type` (struct/enum/option/vec/array/defined)
- `encode_primitive(name, value, buf)` — all primitives: u8-u128, i8-i128, f32/f64, bool, string, publicKey, bytes (hex)
- `coerce_u64/i64/u128/i128` — accept both JSON number and JSON string (since u64/i64/u128/i128 are serialized as strings to avoid precision loss)

### `crates/replay-core/src/session.rs`

- `apply_mutation` now takes `idl_cache: &IdlCache` parameter
- `AccountMutation::Field` variant now fully implemented:
  - Looks up IDL by `account.owner` via `idl_cache.get_local()`
  - Calls `apply_field_mutation` to get the patched bytes
  - Returns `InvalidMutationPath` if no IDL is available (bundled or disk-cached)
- `resolve_mutations` creates `IdlCache::default()` and passes to `apply_mutation`

### `web/components/FieldEditor.tsx` — NEW

Recursive component for editing IDL-decoded JSON values:
- **bool**: checkbox
- **number**: number input (yellow border when modified)
- **string**: text input (yellow border when modified)
- **array**: collapsible with `[i]` index labels
- **enum** `{variant, payload}`: shows variant name + recurses into payload
- **object**: collapsible when >3 fields, renders each key as a row with label
- "edited" badge shown on any modified leaf
- Props: `value`, `path` (dot-sep), `onChange(path, newValue)`, `modifiedPaths: Set<string>`, `depth`

### `web/components/AccountInspector.tsx` — upgraded from read-only stub

New features:
- **IDL detection**: extracts `{typeName, value}` from `decoded_after.kind === "decoded"`
- **Lamports + owner fields** (shown only when `sessionId` exists): controlled inputs, rent-exempt warning when lamports below 890,880
- **Decoded tab**: renders each top-level field via `FieldEditor` when session active; read-only JSON otherwise
- **Apply / Discard strip** (shown only when `sessionId` exists):
  - "Apply (N)" button: calls `mutate()` API for each lamports/owner/field change in sequence, then `addPendingMutation` for each
  - "Discard" clears all local edits
  - Error message shown on API failure
- Fallback for non-IDL accounts: shows raw JSON blob

### `web/app/replay/[signature]/page.tsx` — switched to fork()

- `useEffect` now calls `fork(signature)` instead of `replay()`, stores result via `setSession(session_id, baseline_trace)`
- `handleRerun`: calls `execute(sessionId)` → `setTrace(result.trace)`, then `getDiff(sessionId)` → `setDiff(d)`
- **Re-run button** in top-right of header:
  - Disabled when `pendingMutations.length === 0`
  - Shows badge with mutation count when mutations are pending
  - Triggers `handleRerun` on click

## Design decisions

- `apply_field_mutation` decode → patch → re-encode approach: simple, correct, no partial-update footguns
- `coerce_*` helpers accept both JSON number and string — necessary because u64/i64/u128/i128 are stored as strings in the decoded JSON to avoid JS `number` precision loss
- FieldEditor tracks modifications via `modifiedPaths: Set<string>` passed from parent — keeps the component stateless (parent owns all pending-edit state)
- Lamports/owner mutation fields only appear when a session is active — clean UX, no dead inputs when viewing read-only

## Tests / build

```bash
cargo test -p replay-core --lib   # 26 passed
cd web && pnpm tsc --noEmit       # clean
cd web && pnpm build               # clean
```

## Commits

```
df86854 feat(day-08): Account Mutator UI — fork session, field editor, Re-run
```

## Next session bootstrap (Day 9)

Day 9 = Diff view. Read:
- `memory/day-08.md` (this file)
- `prompts/day-09-diff-view.md`

Key deliverables:
1. `web/components/DiffView.tsx` — side-by-side baseline vs forked trace comparison
2. Show changed accounts with before/after lamports + data diff
3. CU delta display (total and per-frame)
4. Result change indicator (success → failure or vice versa)

`setDiff(d)` is already called in `handleRerun` in page.tsx. The `diff: TraceDiff | null` is already in Zustand store. Day 9 just needs to render it.

`TraceDiff` type (from `web/lib/types.ts`):
```typescript
interface TraceDiff {
  baseline: Trace;
  latest: Trace;
  result_changed: boolean;
  total_cu_delta: number;
  changed_accounts: string[];  // pubkeys
}
```
