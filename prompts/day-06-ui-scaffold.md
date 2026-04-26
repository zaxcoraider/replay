# Day 6 — Web UI Scaffold

## Goal

Next.js app running locally with: signature input → calls `/replay` → renders the trace tree. Basic but functional. No mutation UI yet.

## Deliverables

1. `web/` directory scaffolded with:
   ```bash
   pnpm create next-app@latest web --typescript --tailwind --app --eslint --no-src-dir
   cd web && pnpm dlx shadcn@latest init
   pnpm add @monaco-editor/react zustand clsx bs58 react-json-view-lite
   pnpm dlx shadcn@latest add button input card tabs scroll-area separator tooltip badge
   ```

2. `web/lib/api.ts` — typed client wrapping the replay-api. Functions:
   - `replay(signature: string): Promise<Trace>`
   - `fork(signature: string): Promise<{ session_id, baseline_trace }>`
   - `mutate(sessionId, pubkey, mutation): Promise<...>`
   - `execute(sessionId): Promise<Trace>`
   - `diff(sessionId): Promise<TraceDiff>`

3. `web/app/page.tsx` — landing page:
   - Big centered signature input with "Replay" button.
   - Three "try me" demo buttons that fill in preset signatures.
   - On submit, navigate to `/replay/[signature]`.

4. `web/app/replay/[signature]/page.tsx` — results page:
   - Header: signature, slot, block time, overall result (✓ Success / ✗ Failure).
   - Left sidebar: trace tree (recursive component rendering `CpiFrame` children).
   - Main panel: selected frame details — instruction name, accounts list, logs, CU.
   - Right sidebar: Account inspector (stub for now; real mutation UI on Day 8).

5. Styling: dark, terminal-adjacent aesthetic. Monospace for addresses/hashes. Use shadcn's dark theme.

6. State: Zustand store `useReplayStore` holding `currentTrace`, `selectedFrameIndex`, `selectedAccountPubkey`.

## Design principles

- **Addresses are long.** Render as `JUP6Lk...NyVTaV4` with a copy-to-clipboard button. Tooltip shows full.
- **The CU bar is visual.** Each frame in the tree renders with a colored bar proportional to its CU share. At a glance, you see which instruction ate the budget.
- **Failed frames are red.** Successful frames are neutral/muted. Not all green — green is visual noise.
- **Loading states are instant and honest.** Show "Fetching tx from Helius... (usually 2s)" not a spinner with no context.

## What NOT to do

- No mutation UI (Day 8).
- No timeline scrubber (Day 7).
- No diff view (Day 9).
- No auth, no accounts, no persistence.
- No mobile optimization (this is a desktop dev tool).

## Environment

`web/.env.local`:
```
NEXT_PUBLIC_REPLAY_API_URL=http://localhost:8787
```

## End-of-day

Record a 20-second screen capture of pasting a signature and seeing the trace tree render. Commit the video (or a gif) to `docs/progress/day-06.mp4`. This habit pays off when writing the pitch — you'll have the whole arc.
