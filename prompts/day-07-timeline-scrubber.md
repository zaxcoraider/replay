# Day 7 — Timeline Scrubber

## Goal

The screenshot moment. A horizontal timeline at the top of the replay page showing every instruction as a segment, width proportional to CU consumed, clickable to jump to that frame in the detail panel.

## Deliverables

1. `web/components/Timeline.tsx`:
   - Horizontal SVG or flex-div track.
   - One segment per top-level instruction (flatten depth-1 CPI into top-level instructions).
   - Segment width proportional to `cu_consumed / total_cu`.
   - Segment color: neutral for success, red for failure, muted grey for compute-budget instructions.
   - Hover: tooltip with full program name + instruction name + CU.
   - Click: selects the frame in the Zustand store.
   - Keyboard: arrow left/right to navigate between frames. Space to toggle expansion.

2. `web/components/CuGauge.tsx`:
   - A "fuel gauge" component that fills up as you scrub through the timeline.
   - Shows `current_consumed / total_budget` visually.
   - Animates smoothly as the selected frame changes.

3. Integrate into `web/app/replay/[signature]/page.tsx` — Timeline above the sidebar + detail split, CuGauge in the top-right corner.

## Design details

- Segments need minimum width (~8px) even for tiny-CU instructions, or they're unclickable.
- For transactions with many instructions, consider a "zoom" affordance — pinch or scroll-wheel. v1 can skip this if ≤20 instructions.
- Sub-instruction CPIs show up on hover as a mini-tree inside the segment. Don't overcrowd the horizontal view.

## Acceptance

A Jupiter swap tx should show 3–5 top-level instructions. The Route instruction should be by far the widest segment. Hovering any segment tells you what it did. Clicking updates the detail panel.

## End-of-day

Another screen capture. Compare to Day 6's. You should feel the product getting real.
