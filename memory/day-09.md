# Day 9 — Diff View

**Date:** 2026-05-02
**Prompt:** [`prompts/day-09-diff-view.md`](../prompts/day-09-diff-view.md)
**Result:** Done in one session. TypeScript clean, build clean.

## Goal

After a re-run, show baseline vs forked trace side-by-side so judges immediately see the impact of a mutation.

## What landed

### `web/components/DiffView.tsx`

Full-height diff panel that replaces the 3-panel layout when "View diff" is clicked.

**Summary banner** (top strip):
- Result: `✓ → ✗` (or vice versa) with "CHANGED" badge if flipped
- CU: `842,000 → 997,000 (+18.4%)`
- Accounts changed: `7 changed`

**Side-by-side frame rows:**
- `grid grid-cols-2` with Baseline / Re-run column headers
- Per-row highlights:
  - `bg-red-950/25 border-l-2 border-l-red-700` — result changed for this frame
  - `bg-yellow-950/20 border-l-2 border-l-yellow-700` — CU shifted >15%
- Each cell: colored dot (green=success, red=failed) + instruction name + CU + delta

**Log diff section:**
- Computes set difference of all logs (baseline flat vs latest flat)
- Removed lines: red background, strikethrough, `- ` prefix
- Added lines: green background, `+ ` prefix
- Hidden if logs are identical

**Changed accounts section:**
- Lists each pubkey from `diff.changed_accounts`
- Per account: lamports delta in both runs, badges for `lamports` / `data` changed
- Uses `AddressChip` for truncated pubkey display

### `web/components/DiffSummaryCard.tsx`

Compact header card shown after re-run when not in diff mode:
- `✓ → ✗` result indicator with `!` badge if changed
- CU delta percentage
- Account count
- "View diff" button → enters diff mode
- `×` button → dismisses (calls `setDiff(null)`)

### `web/app/replay/[signature]/page.tsx` — updated

New state: `showDiff: boolean` (default false, reset on new sig)

Header additions:
- `DiffSummaryCard` rendered when `diff && !showDiff` 
- "← Trace" ghost button when `showDiff` is true

Body switch:
```tsx
{showDiff && diff ? (
  <DiffView diff={diff} />
) : (
  // existing 3-panel layout
)}
```

Timeline hidden in diff mode (no `<Timeline>` when `showDiff`).

`handleRerun`: calls `execute()` → `setTrace()` then `getDiff()` → `setDiff()`. Does NOT auto-open diff — user clicks "View diff" in summary card.

## UX flow (the demo moment)

1. Paste Whirlpool swap sig → 3-panel loads
2. Click pool account → inspector shows decoded fields
3. Edit `fee_rate` → Apply (1) → mutation queued
4. Re-run → summary card appears in header: `✓ → ✗  CU +18%  7 accts`
5. Click "View diff" → full diff panel with red frame on the right, green on left

## Tests / build

```bash
cd web && pnpm tsc --noEmit   # clean
cd web && pnpm build           # clean
```

## Commits

```
fed73cf feat(day-09): Diff view — baseline vs re-run side-by-side
```

## Next session bootstrap (Day 10)

Day 10 = Demo preload + live deployment. Read:
- `memory/day-09.md` (this file)
- `prompts/day-10-demo-preload.md`

Key deliverables:
1. 3 hardcoded demo transaction signatures on the landing page (Whirlpool swap, Jupiter route, Drift trade)
2. Deploy API to Railway/Fly.io
3. Deploy web to Vercel
4. Public URL judges can visit without a Helius key

The backend needs a `HELIUS_API_KEY` env var. For the public demo the key lives in the deployed API env — the web just points to the deployed API URL via `NEXT_PUBLIC_REPLAY_API_URL`.
