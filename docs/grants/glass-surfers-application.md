# Glass Surfers Dev Tooling Grant — Application

**Grant program:** Glass Surfers Dev Tooling Grants
**Application URL:** earn.superteam.fun/grants/glass-surfers-dev-tooling-grants
**Requested amount:** $5,000 USD
**Project:** Replay — Time-Travel Debugger for Solana

---

## Project summary

Replay lets any Solana developer paste a mainnet transaction signature and instantly replay it locally against the exact historical account state that existed at that slot. No devnet approximations. No manual account setup. The exact state — LUT-resolved, upgradeable-program-aware — reconstructed in a LiteSVM sandbox in seconds.

From there, developers can fork the sandbox, mutate any account field (IDL-decoded for Jupiter, Whirlpool, Drift, Kamino), re-run the transaction, and see a side-by-side diff: did the result change? How much did CU shift? Which accounts are different?

**Live now:**
- Web UI: https://replay-weld.vercel.app
- Docs: https://replay-weld.vercel.app/docs
- Rust SDK on crates.io: https://crates.io/crates/replay-sdk
- TypeScript SDK on npm: https://www.npmjs.com/package/@zaxcoraider/replay-sdk
- GitHub: https://github.com/zaxcoraider/replay

---

## The problem

Solana debugging today means reading logs, guessing at account state, and re-deploying to devnet. There is no way to answer the question: "if this account had a different value, would this transaction have succeeded?" without setting up a shadow environment by hand.

Existing tools (Explorer, SolanaFM, Shank) are read-only. They show you what happened — they don't let you ask "what if."

Replay is the only tool that closes this gap for Solana mainnet transactions.

---

## What the $5,000 grant funds

**Milestone: VSCode extension for in-editor transaction replay**

The grant funds a dedicated 6-week sprint to build a VSCode extension that embeds Replay directly into the editor. Developers will be able to:

1. Right-click any transaction signature in their code → "Replay in Replay"
2. See the CPI trace panel open inline in the editor
3. Click any account in the trace → inline diff between historical and current on-chain state
4. Run mutation scenarios from a `.replay.json` config file in the workspace

This is the most-requested feature from the Colosseum community and the logical next step after the CLI and web UI. It makes Replay a first-class part of the Anchor/native Solana development workflow rather than a separate browser tab.

**Deliverables (6 weeks):**
- `replay-vscode` extension published to VS Marketplace (free, open source)
- Webview panel: CPI trace tree, account inspector, diff view (reusing web UI components)
- Command palette: `Replay: Replay transaction`, `Replay: Inspect account`
- `.replay.json` workspace config for pinned signatures and mutation scenarios
- Integration tests via `@vscode/test-electron`

---

## Team

**zaxcoraider** — solo builder. Built the full Replay stack (Rust engine, axum API, Next.js UI, TypeScript SDK, Rust SDK, live SSE mode) in 14 days for the Colosseum Frontier 2026 hackathon. Background in systems programming and DeFi protocol development.

GitHub: https://github.com/zaxcoraider

---

## Why Replay is a public good

- MIT-licensed, no VC funding, no token
- The engine (`replay-core`) is fully open — anyone can embed it
- The TypeScript SDK ships a `replayHistorical` CI helper so protocols can test their programs against their own transaction history
- The Rust SDK is published to crates.io so any Rust program can call into the replay pipeline

Every line of Replay exists to make the Solana developer experience less painful for everyone, not to extract rent from it.
