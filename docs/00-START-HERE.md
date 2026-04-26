# 00 — START HERE

Before you write a single line of code, finish this document. It takes 30–60 minutes and saves days.

## The 18-day calendar

Submissions close **Sunday, May 11, 2026, 23:59 UTC** (verify on arena.colosseum.org). Count backward: you start **no later than Thursday, April 24, 2026** and you **freeze features on Thursday, May 8** to give yourself 3 days for pitch polish and video. If you read this after April 24, subtract lost days from the polish buffer, not from the spike.

## Pre-flight checklist

Tick every box before Day 1.

- [ ] **Rust toolchain** — `rustup default stable` (min 1.79). `rustc --version` should print cleanly.
- [ ] **Solana CLI** — `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`. Verify with `solana --version` (needs ≥ 1.18.8 for `cargo build-sbf`).
- [ ] **Anchor** — `cargo install --git https://github.com/solana-foundation/anchor avm && avm install latest && avm use latest`. This is optional for the engine but needed to test IDL decoding.
- [ ] **Node 20+** — for the Next.js UI. `nvm install 20 && nvm use 20`.
- [ ] **pnpm** — `npm i -g pnpm`. (Faster + better workspace ergonomics than npm.)
- [ ] **Helius API key** — sign up at helius.xyz, copy the key into `.env` as `HELIUS_API_KEY=...`. Free tier is enough for the hackathon.
- [ ] **GitHub repo** — create `replay` (public, MIT license from day 1 — Public Goods track requires it).
- [ ] **Claude Code** — installed and authenticated. Verify with `claude --version`.
- [ ] **Colosseum account** — register at arena.colosseum.org. Note any pre-submission requirements.
- [ ] **Superteam Earn account** — register at earn.superteam.fun. Bookmark the side-tracks page.

## The spike — your day-2 go/no-go gate

At the end of Day 2, you must demonstrate ONE thing working end-to-end:

> Paste a real mainnet Jupiter swap transaction signature into the CLI. The CLI fetches the transaction, reconstructs the state of all accounts at the slot before the tx, loads all referenced programs into litesvm, replays the tx, and prints logs that **match the mainnet logs character-for-character** (modulo clock-dependent output).

If this doesn't work by end of Day 2:
- **Do not proceed.** The whole project rests on this primitive.
- **Pivot to the CU Profiler idea** (see fallback note below). You lose one day but keep the hackathon.

### Fallback pivot plan if spike fails

If day 2 ends without a faithful replay:
1. Keep the repo and all infra work.
2. Reframe as **"solana-profile"** — a cargo subcommand that runs Anchor tests, samples `sol_log_compute_units()` at instruction/CPI boundaries, and produces a flamegraph of CU consumption.
3. The Helius integration, Public Goods submission, pitch structure, and side-track stack all transfer. You lose only the time-travel narrative.
4. See `docs/FALLBACK-cu-profiler.md` (write it only if you need to pivot).

## Your working rhythm

Treat each day as one **Claude Code session**. At the start of a session:
1. `git pull` and make sure tests pass.
2. Open `prompts/day-XX-*.md` and read it yourself.
3. Paste `prompts/SYSTEM-PROMPT.md` + the day's prompt into Claude Code.
4. Drive — don't disappear. Review every diff. Claude Code will confidently produce wrong code for Solana-specific things (account layouts, PDA derivations, compute budget). You are the domain expert; it is the typist.
5. At session end: `git commit` with a meaningful message, update `docs/LOG.md` (create it day 1) with what worked and what didn't.

## The single hardest thing you will hit

**Program bytecode at a historical slot.** Solana programs are upgradeable. The bytecode at program ID `JUP6...` today is not the bytecode that executed at slot N 6 months ago. You *must* fetch the program data account's state as of `slot-1` via `getAccountInfo` with `minContextSlot` set appropriately, or your replay will silently diverge.

This is covered in detail in `04-solana-gotchas.md` and in the Day 2 prompt. Do not skip that section.

## Done? Go to `prompts/day-01-spike-fetch-tx.md`.
