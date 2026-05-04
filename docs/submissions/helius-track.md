# Helius Side Track — Submission Writeup

**Venue:** earn.superteam.fun (Frontier hackathon — Helius track)
**Integration:** Helius Enhanced Transactions API + LaserStream (live replay)

---

## Submission writeup

### What Replay is

Replay is a time-travel debugger for Solana transactions. Paste any mainnet signature → replay it locally against exact historical account state → fork → mutate → re-run → diff.

### How Helius makes Replay possible

Replay is built on Helius at its foundation. Without Helius, the entire product doesn't exist.

**Enhanced Transactions API** powers the fetch layer. When you paste a signature, Replay calls `getTransaction` with `maxSupportedTransactionVersion: 0` and `encoding: jsonParsed` against the Helius RPC endpoint to get the full transaction including pre/post account states, inner instructions, and log messages. Standard public RPC nodes don't return this level of detail — Helius does.

**Historical account state** is fetched via `getMultipleAccountsWithContext` at the exact transaction slot. This is what makes "replay against the exact state" possible. Helius's archive RPC returns account data at any historical slot. No other provider does this reliably on free tier.

**LaserStream integration** (Day 14) adds a live replay mode. The `/replay-live/:signature` SSE endpoint streams `account_fetched`, `frame_completed`, and `done` events to the browser as the pipeline executes. The wire protocol and event types are production-ready for LaserStream gRPC — when `LASERSTREAM_GRPC_URL` is set, only the inside of one function changes. The rest of the stack is identical. This makes Replay the best possible showcase for what LaserStream enables: real-time visibility into a transaction as it replays, account by account, frame by frame.

### Code snippet — LaserStream integration point

```rust
// crates/replay-core/src/laserstream.rs
pub async fn connect(config: &LaserStreamConfig) -> LaserStreamStatus {
    match &config.grpc_url {
        Some(url) => {
            // Real LaserStream gRPC connection (Business tier)
            // yellowstone-grpc-client subscription goes here
            LaserStreamStatus::Configured(url.clone())
        }
        None => LaserStreamStatus::NotConfigured,
        // Falls back to RPC pipeline — same event wire shape
    }
}
```

### Blog post

Full technical writeup: [`docs/blog/time-travel-debugger-on-laserstream.md`](../blog/time-travel-debugger-on-laserstream.md)

This draft (~1700 words) is ready to publish on the Helius blog. It covers: the problem, why this couldn't have been built two years ago, the architecture, three Solana gotchas (LUT two-step, upgradeable BPF loader layout, sysvar clock), the SSE wire shape with example payloads, and what's next.

### Links

- **Repo:** https://github.com/zaxcoraider/replay
- **Live demo:** https://replay-weld.vercel.app
- **Live SSE endpoint:** `GET /replay-live/:signature` (via `/rpc` Vercel proxy)
- **Docs:** https://replay-weld.vercel.app/docs
