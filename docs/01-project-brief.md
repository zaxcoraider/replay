# 01 — Project Brief

*(This document is short on purpose. Paste it at the top of every Claude Code session so the model always has full product context.)*

## What Replay is

Replay is a time-travel debugger for Solana transactions. A developer pastes a mainnet transaction signature and gets:

1. A faithful local reproduction of the transaction, running in `litesvm` against the exact account state that existed at the transaction's slot.
2. An execution trace showing every instruction, every CPI boundary, and compute-unit consumption at each level.
3. The ability to **mutate any account's state** — change a fee, flip a flag, swap a PDA owner — and re-run to see how the outcome changes.
4. A visual diff between the baseline run and any forked run.

Think of it as Tenderly + rr (record-replay debugger) for Solana. This category does not yet exist on Solana at production quality. The closest prior art is Seer, which shows you what happened; Replay lets you ask *what if*.

## Who it's for

1. **Protocol developers** debugging failed mainnet transactions.
2. **Security researchers** reproducing exploits and proving fixes.
3. **CI systems** replaying historical transactions against a new program version to detect regressions before upgrade.

## Why it wins this hackathon

- **Colosseum judges pattern-match on "Tenderly for Solana"** — they are VCs and the analogy to a $40M+ category-leader on EVM is obvious and appealing.
- **Seer won the Infra track last cycle** with less ambitious scope. The obvious progression is time-travel + mutation, which no one is building.
- **Helius LaserStream** is a first-class integration — this project is arguably the best possible showcase of their historical streaming product.
- **Public Goods track $10k** is a near-certain stack — this is obviously a public good; it's MIT-licensed from day 1.
- **The demo is visual and dramatic** — "here's a real exploit, watch me prove it by mutating one field" is a judging-room moment.

## Product surfaces (three, same engine)

1. **Web UI** (`web/`) — the demo surface. Next.js 14 + shadcn/ui + Monaco.
2. **CLI** (`replay-cli`) — `replay <signature>` prints a trace; `replay fork <signature>` drops you into an interactive REPL.
3. **SDK** (Rust + TypeScript) — for programmatic use in CI. `@replay/sdk` on npm, `replay-sdk` on crates.io.

## Tech stack (do not deviate without reason)

- **Engine:** Rust. Crates: `litesvm`, `solana-client`, `solana-transaction-status`, `solana-sdk`, `anchor-syn` (for IDL parsing), `borsh`.
- **API:** `axum` + `tokio` + `tower-http` for CORS. WebSockets for live trace streaming.
- **Frontend:** Next.js 14 (App Router) + TypeScript + shadcn/ui + Tailwind + Monaco Editor + `@solana/web3.js` (for signature validation only, not RPC).
- **Data layer:** Helius RPC (`getTransaction`, `getAccountInfo` with `minContextSlot`). LaserStream for live mode in Day 14.
- **Hosting:** Render or Fly.io for the API. Vercel for the web app.

## The north-star rule (repeated because it matters)

**Ship a working demo of one real mainnet transaction being replayed, forked, mutated, and re-run — with visible diff — before adding anything else.** Every feature outside that path is a "nice to have" that gets cut first when time runs short.
