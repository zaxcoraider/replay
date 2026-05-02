# Day 7 — Timeline Scrubber

**Date:** 2026-05-02
**Prompt:** [`prompts/day-07-timeline-scrubber.md`](../prompts/day-07-timeline-scrubber.md)
**Result:** Done in one session. TypeScript clean, build clean.

## Goal

Add a horizontal timeline bar above the 3-panel view showing every top-level
instruction as a CU-proportional segment. Clicking selects the frame. Add a
CU gauge in the top-right corner.

## What landed

### `web/components/Timeline.tsx`

Horizontal flex-row of segments, one per top-level `CpiFrame`:
- Width via `flex-grow` proportional to `cu_consumed / total_cu`
- `min-w-[8px]` ensures tiny-CU instructions (ComputeBudget) are still clickable
- Colors: blue when selected, red/80 for failed frames, muted grey for
  ComputeBudget, zinc-600 for normal
- Hover tooltip (Base UI) shows: program name, instruction name, CU count,
  error message if failed, CPI child summary (program + name + CU per child)
- Click → `setSelectedFrame(id)` in Zustand (same `id` format as CpiTree:
  `"${depth}-${instruction_index}-${program_id}"`)
- Keyboard: `ArrowLeft` / `ArrowRight` navigate between top-level frames
  (listener on `window`, active when body or container has focus)
- Instruction label rendered in segment (hidden on narrow viewports via `sm:block`)
- Filler `flex-1` div prevents last segment stretching to fill remaining space

### `web/components/CuGauge.tsx`

Compact fuel gauge in the header top-right:
- Horizontal bar (80px wide, `transition-all duration-300` smooth animation)
- Color: blue → yellow (>60%) → red (>80%)
- Label: percentage + selected frame's CU in parentheses
- Computes "consumed so far" by summing CU of top-level frames up to and
  including the selected one (left-to-right reading order)

### `web/app/replay/[signature]/page.tsx` — wired in

- `<Timeline frames={trace.frames} totalCu={trace.total_cu} />` inserted between
  the top bar and the 3-panel body
- `<CuGauge frames={trace.frames} totalCu={trace.total_cu} />` in the top bar,
  replacing the plain CU text (CU text kept next to it)

## Design decisions

- `flex-grow` approach (not SVG, not absolute positioning) — simpler, naturally
  responsive, no ResizeObserver needed
- Minimum 8px guaranteed via Tailwind `min-w-[8px]` so ComputeBudget instructions
  are always reachable with a click
- No zoom affordance for v1 (spec says ok to skip for ≤20 instructions)
- Base UI tooltip `side="bottom"` to avoid clipping by the top bar

## Tests / build

```bash
cd web && pnpm tsc --noEmit   # clean
cd web && pnpm build           # clean
```

## Commits

```
<hash> feat(web): Day-7 Timeline scrubber + CuGauge
```

## Next session bootstrap (Day 8)

Day 8 = Account Mutator UI. Read:
- `memory/day-07.md` (this file)
- `prompts/day-08-account-mutator.md`

Key deliverables:
1. `web/components/AccountInspector.tsx` — upgrade to editable form (field editor
   for IDL-decoded accounts, raw hex splice editor, lamports/owner always editable)
2. `web/components/FieldEditor.tsx` — recursive IDL field editor
3. Fork-session flow: `POST /fork` on page load, store `sessionId` in Zustand,
   enable "Apply mutation" + "Re-run" buttons
4. `POST /session/:id/mutate` on Apply, `POST /session/:id/execute` on Re-run

The `sessionId` field is already in `useReplayStore` (set by `setSession`).
Currently the page only calls `replay()` (one-shot). Day 8 switches it to
`fork()` instead, storing the session ID for mutations.
