# Day 10 — Demo Pre-Load

## Goal

End of MVP. Ship the demo story. Hard-code three "try me" experiences that reliably produce a wow moment. Nothing else today — polish, test, polish.

## Deliverables

1. Three canonical demo signatures, each with a pre-configured mutation story:

   **Demo A: "The Jupiter swap"**
   - A clean Jupiter v6 swap.
   - Suggested mutation: swap one pool's `fee_rate` to 9999. Re-run shows insufficient output, whole tx fails.
   - Narrative: "See how route composability propagates failures."

   **Demo B: "The oracle freeze"**
   - A tx that depends on a Pyth or Switchboard price.
   - Suggested mutation: set the oracle account's price field to 0 or set the publish_slot to a stale value.
   - Re-run: tx fails the staleness check.
   - Narrative: "See how oracle manipulation gets caught — or doesn't."

   **Demo C: "The historical exploit"** (if you can find a suitable candidate; otherwise skip)
   - Pick a real historical incident where a state value made the difference.
   - Mutate that value to what it "should" have been.
   - Show the tx would have behaved correctly.
   - Narrative: "Post-mortem debugging, without the VM headaches."

2. `web/lib/demo-signatures.ts`:
   ```ts
   export const DEMOS = [
     {
       id: 'jupiter-swap',
       title: 'Jupiter swap — mutate fee, watch it fail',
       signature: '...',
       suggested_mutation: {
         account: '...',
         field: 'fee_rate',
         new_value: 9999,
       },
       narrative: '...',
     },
     // ...
   ];
   ```

3. Update landing page: 3 big demo cards with titles, screenshots, "Try it" buttons that fill in the signature AND pre-apply the suggested mutation as a preview in the inspector.

4. "Guided tour" mode: a lightweight step-by-step overlay that walks a first-time user through: "1. This is the timeline. 2. Click here to inspect the Whirlpool config. 3. Change this value. 4. Re-run. 5. See the diff." Use `driver.js` or similar.

## Polish pass

- All loading states have honest copy.
- All error states have actionable next steps ("Helius rate limited — try again in 30s").
- No horizontal scroll at 1280px width.
- Timeline renders correctly for txs with 20+ instructions.
- Mobile layout gracefully degrades (doesn't need to be great — just not broken).

## End-of-day

- All 3 demos work flawlessly from a fresh browser session.
- Record a fresh 2-minute run-through with OBS. This is a rehearsal for the pitch video.
- MVP is now done. Everything after this is polish, distribution, and pitch.
