# Day 17 — End-to-End Fix + Prod Verified

**Date:** 2026-05-04
**Result:** Critical API bug found and fixed. Full stack working locally and on prod.

## Root cause found (Day 14 blocker resolved)

`replay-api` was returning `500 "Unable To Extract Key!"` on EVERY request including
`/health` — not a port conflict or browser extension. Root cause:

`tower_governor` (rate limiter) requires `ConnectInfo<SocketAddr>` to extract the
peer IP as a rate-limit key. We were calling `axum::serve(listener, app)` which
does NOT inject `ConnectInfo`. Result: every request failed key extraction before
reaching any route handler (including health, which never touches Helius).

**Fix:** `axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())`

One line. Commit: `5ebc156`

## Also fixed

- Default port changed from 8787 → 8080 (port 8787 is intercepted by Cloudflare
  Workers tooling on Windows)
- `web/.env.local` updated to `http://localhost:8080` (local dev only, gitignored)
- `web/next.config.ts` default updated to 8080

## Keep-alive added

`web/app/api/keepalive/route.ts` + `web/components/KeepAlive.tsx` — pings
`/health` every 10 min from the browser to prevent Render free-tier spin-down
during the hackathon demo window. Commit: `f22eb79`

## Full demo flow verified

Tested locally (port 8080) and on prod (replay-weld.vercel.app):
- ✅ GET /health → 200 ok
- ✅ POST /replay → full CPI trace (Jupiter v6 swap, real mainnet tx)
- ✅ Live tab → SSE stream, RPC fallback mode, "done" event
- ✅ Fork → auto on page load
- ✅ Mutate lamports → staged correctly, balance delta shown
- ✅ Re-run → executes against mutated state
- ✅ Diff view → result ✗→✗, CU 264,498→264,498, 1 account changed
- ✅ Prod Render deploy → green, same behavior as local

## Next: Day 17 remaining / Day 18

Day 17 prompt calls for:
- 2-min demo video (record with OBS following docs/06-demo-script.md)
- 3-min pitch video (camera on, good lighting)
- 10-slide pitch deck
- Colosseum submission needs both video links

Day 18 = final submission to all tracks before May 11 deadline.
