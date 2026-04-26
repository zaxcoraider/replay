# Day 9 — Diff View

## Goal

After a re-run, show baseline-vs-forked side-by-side. This is where judges go "ohhh."

## Deliverables

1. `web/components/DiffView.tsx`:
   - Top: summary banner — "Result: succeeded → failed" (or vice versa), "CU: 842k → 997k (+18%)", "Accounts changed: 7 → 12".
   - Two-column layout: baseline trace (left), forked trace (right). Same vertical alignment per frame.
   - Frames that differ are highlighted (color based on change type: failed, CU spike, state divergence).
   - Click a frame pair → detail modal with full log diff (use `react-diff-viewer-continued`).

2. Account delta diff:
   - Section below the frame diff.
   - For each account that changed differently between baseline and fork, show:
     - Baseline delta (pre → post)
     - Forked delta (pre → post)
     - Field-level diff if IDL-decodable.

3. `web/components/DiffSummaryCard.tsx`:
   - Compact card version of the summary banner.
   - Renders in the timeline header when in "diff mode."

4. Modes:
   - Default view after execute: "Latest run only" (just shows the new trace).
   - Toggle: "Compare to baseline" → switches to split view.
   - Persistent: once user opens diff, it stays open until they explicitly close it.

## Interaction polish

- Scroll sync between left and right trace columns.
- "Jump to first divergence" button — scrolls both columns to the first frame that differs.
- Keyboard: `N` = next divergence, `P` = previous.

## What makes this the moment

The judge clicks the Whirlpool config → changes `fee_rate` → clicks Re-run → sees the diff view appear with "Swap failed: insufficient output" in red on the right, side-by-side with green "Swap success" on the left. You should be able to demo this in under 20 seconds from page load.

## End-of-day

Polish pass. This view will be on every screenshot in your pitch deck.
