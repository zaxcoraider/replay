# Colosseum Frontier — Main Submission

**Venue:** arena.colosseum.org
**Track:** Grand Champion
**One-liner:** Time-travel debugger for Solana transactions.

---

## Project name
Replay

## Tagline
Paste any mainnet transaction. Replay it locally against exact historical state. Fork. Mutate. Re-run. Diff.

## Description (300 words)

Debugging a Solana transaction today means reading logs and guessing. There is no way to answer "what if this account had a different value?" without setting up a shadow environment by hand — which is impossible for mainnet-specific state.

Replay solves this. Paste any mainnet transaction signature. Replay fetches the transaction and all referenced accounts from Helius at the exact slot where the transaction landed, hydrates a LiteSVM sandbox with that state (resolving lookup tables, upgradeable program accounts, sysvar slots), and re-executes the transaction locally. You get a full CPI trace — every cross-program invocation, CU cost, decoded arguments — and a confirmation that the local result matches mainnet.

From there you can fork the sandbox into a mutable session. Change any account field — Replay uses the on-chain Anchor IDL to decode known programs (Jupiter, Whirlpool, Drift, Kamino) into named fields. Re-run the transaction against the mutated state. See a side-by-side diff: did the result change? How much did CU shift? Which accounts are different?

Everything is accessible three ways: a web UI at replay-weld.vercel.app, a CLI (`cargo install replay-cli`), and SDKs for TypeScript (`@zaxcoraider/replay-sdk` on npm) and Rust (`replay-sdk` on crates.io).

Replay is the Tenderly equivalent for Solana — but built natively on the Solana execution model, using LiteSVM instead of a simulated EVM fork, and Helius's historical state API instead of archive node snapshots.

It is MIT-licensed, fully open source, and built to be embedded: the core engine is a Rust library anyone can call from their own program, CI pipeline, or security audit tool.

## Links

- **Repo:** https://github.com/zaxcoraider/replay
- **Live demo:** https://replay-weld.vercel.app
- **Docs:** https://replay-weld.vercel.app/docs
- **npm:** https://www.npmjs.com/package/@zaxcoraider/replay-sdk
- **crates.io (SDK):** https://crates.io/crates/replay-sdk
- **crates.io (core):** https://crates.io/crates/replay-core

## Tech stack

- **Engine:** Rust, LiteSVM, Helius RPC + Enhanced Transactions API
- **API:** Axum (Rust), deployed on Render
- **Web:** Next.js 15, deployed on Vercel
- **SDKs:** Rust (crates.io), TypeScript (npm)
- **Live replay:** SSE endpoint, Helius LaserStream-ready (gRPC stub, env-var gated)
