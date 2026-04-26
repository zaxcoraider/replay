# Day 14 — Helius LaserStream Integration + Blog Post

## Goal

Add a LaserStream-powered "live replay" mode: paste a signature for a transaction that happened in the last N seconds and watch the replay stream in real-time. Then write a blog post positioning Replay as a flagship LaserStream consumer.

## Deliverables

1. `crates/replay-core/src/laserstream.rs`:
   - gRPC client for Helius LaserStream. Use `yellowstone-grpc-client` or Helius's fork; check their docs for the current recommended approach.
   - Subscribe to transaction updates filtered by signature (or slot range for discovery).
   - Stream account updates for the accounts referenced by the target tx.

2. API endpoint: `GET /replay-live/:signature` — Server-Sent Events or WebSocket stream:
   ```json
   {"type": "slot_observed", "slot": 271234567}
   {"type": "account_fetched", "pubkey": "...", "size": 165}
   {"type": "all_accounts_fetched", "count": 23}
   {"type": "execution_started"}
   {"type": "frame_completed", "frame": {...}}
   {"type": "done", "trace": {...}}
   ```

3. UI: new "Live" tab on the replay page. When active, renders events as they arrive — builds the trace tree progressively. This is visually stunning even when the actual latency is unchanged; it feels instant.

4. **The blog post** — this is arguably as important as the code today.
   - Draft in `docs/blog/time-travel-debugger-on-laserstream.md`.
   - 1500–2000 words.
   - Structure: The problem → Why we couldn't build this before LaserStream → Technical deep-dive (LUT resolution, historical bytecode, sysvar handling) → Results → What's next.
   - Include code snippets and screenshots.
   - Pitch it to Helius on Day 16 for their blog — they publish community content regularly.

## Rate limiting and cost control

LaserStream has a cost model (gRPC bandwidth). Add:
- Cap on concurrent live sessions (default: 5).
- Auto-close idle live sessions after 5 minutes.
- Clear error message when limits hit.

## What NOT to do

- Don't rebuild your core engine on LaserStream. It's supplemental. Most replays still go through `getTransaction` + historical `getAccountInfo`.
- Don't try to support devnet/testnet LaserStream — mainnet only.

## End-of-day

- Blog post draft reviewed, ready to send to Helius's dev-rel contact on Day 16.
- "Live" tab in the UI works for at least one recent tx.
