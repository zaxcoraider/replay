# Day 6 — Web UI Scaffold

**Date:** 2026-05-02
**Prompt:** [`prompts/day-06-ui-scaffold.md`](../prompts/day-06-ui-scaffold.md)
**Result:** Done in one session. Build clean, TypeScript clean.

## Goal

Next.js app running locally: signature input → calls `/replay` → renders the
trace tree. Basic but functional. No mutation UI yet.

## Stack

- Next.js 15 (App Router, TypeScript, Tailwind)
- shadcn — **Base UI** variant (not Radix; note different API: `delay` not
  `delayDuration` on `TooltipProvider`, no `asChild` on `TooltipTrigger`)
- Zustand for client state
- lucide-react icons
- `@monaco-editor/react`, `bs58`, `react-json-view-lite` installed (used Day 8+)

## What landed

### `web/lib/types.ts`
TypeScript mirror of `replay-core` types: `Trace`, `CpiFrame`, `AccountDelta`,
`TxResult`, `TraceDiff`, `LogDivergence`, `FrameAccount`, `ApiError`.

### `web/lib/api.ts`
Typed client wrapping the replay-api:
- `replay(sig)` → `Trace`
- `fork(sig)` → `{session_id, baseline_trace, expires_at}`
- `mutate(sessionId, pubkey, mutation)` → `{applied, mutation_count}`
- `execute(sessionId)` → `{trace, mutation_count}`
- `diff(sessionId)` → `TraceDiff`

Base URL from `NEXT_PUBLIC_REPLAY_API_URL` (default: `http://localhost:8787`).
Set in `web/.env.local`.

### `web/store/replay-store.ts`
Zustand store: `currentTrace`, `sessionId`, `selectedFrameId`,
`selectedAccountPubkey`, `diff`, `pendingMutations`. Actions: `setTrace`,
`setSession`, `setSelectedFrame`, `setSelectedAccount`, `setDiff`,
`addPendingMutation`, `clearMutations`, `reset`.

### `web/app/layout.tsx`
Dark theme, monospace font (Geist Mono), `TooltipProvider` wrapping everything.

### `web/app/page.tsx`
Landing page: signature input + "Replay →" button, 2 demo cards (stubs until
Day 10 fills real sigs). Navigates to `/replay/[signature]` on submit.

### `web/app/replay/[signature]/page.tsx`
Results page: fetches trace via `replay()` on mount, 3-panel layout:
- **Left (256px):** CPI tree sidebar
- **Center (flex-1):** Frame detail panel
- **Right (288px):** Account inspector
Top bar: signature chip, success/failure badge, slot, block time, total CU,
log-divergence badge. Loading spinner with honest copy ("Fetching tx from
Helius… usually 2–5s").

### Components

**`AddressChip.tsx`** — truncated address (`JUP6Lk…NyVTaV4`) with copy-to-
clipboard on click. Tooltip shows full address. Uses Base UI tooltip.

**`CpiTree.tsx`** — recursive tree rendering `CpiFrame` with chevron expand/
collapse, depth indentation, CU bar (proportional fill), program name mapping,
red text for failed frames. Clicking a frame selects it in the Zustand store.

**`FrameDetail.tsx`** — shows selected frame: header (name, badge, CU, program
chip), tabs: Logs (raw log lines) / Accounts (index, pubkey, role, W/S badges)
/ Args (decoded_args JSON). Clicking an account selects it for the inspector.

**`AccountInspector.tsx`** — shows selected account delta: SOL balance + delta,
owner chip, IDL type name badge. Tabs: Decoded (JSON tree of `decoded_after`) /
Raw hex. Falls back gracefully when account not in deltas.

## Key gotcha: Base UI vs Radix

shadcn v2+ uses `@base-ui/react` not `@radix-ui`. API differences:
- `<TooltipProvider delay={N}>` not `delayDuration`
- `<TooltipTrigger>` wraps children directly — no `asChild` prop

## Environment

```
NEXT_PUBLIC_REPLAY_API_URL=http://localhost:8787   # web/.env.local
HELIUS_API_KEY=...                                  # .env (root, for the API)
```

## Run locally

```bash
# Terminal 1 — API
cargo run -p replay-api

# Terminal 2 — Web
cd web && pnpm dev    # http://localhost:3000
```

## Tests / build

```bash
cd web && pnpm tsc --noEmit   # clean
cd web && pnpm build           # clean, 3 routes: / + /_not-found + /replay/[sig]
```

## Commits

```
a94c077 feat(web): Day-6 Next.js UI scaffold — landing page + 3-panel replay view
```

## Next session bootstrap (Day 7)

Day 7 = Timeline scrubber. Read:
- `memory/day-06.md` (this file)
- `prompts/day-07-timeline-scrubber.md`

Key deliverables:
1. `web/components/Timeline.tsx` — horizontal CU-proportional bar per top-level
   instruction, clickable, hover tooltip
2. `web/components/CuGauge.tsx` — running total fuel gauge
3. Wire both into `/replay/[signature]/page.tsx` above the 3-panel split

No new Rust work needed for Day 7.

The `selectedFrameId` in Zustand is already wired — Timeline just needs to call
`setSelectedFrame` on click, same as `CpiTree` does.
