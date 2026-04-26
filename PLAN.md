# Replay — Build Package

**Project:** A time-travel debugger for Solana transactions.
**Hackathon:** Colosseum Frontier (deadline: **May 11, 2026**) + Superteam Earn side tracks.
**Builder:** Solo.
**AI leverage:** Claude Code.

This package is your complete operating manual. Read `docs/00-START-HERE.md` first, then work through the day-by-day prompts in `prompts/`.

## Repo layout

The original `replay-plan.zip` package has been promoted into the repo root —
`scaffolding/` contents live at the top level, `docs/` and `prompts/` sit
alongside.

```
replay/
├── Cargo.toml                      Workspace root
├── README.md                       Project overview + status
├── PLAN.md                         ← you are here
├── crates/
│   ├── replay-core/                The engine — fetch, reconstruct, execute
│   ├── replay-api/                 axum HTTP + WebSocket server
│   ├── replay-cli/                 `replay <signature>` binary
│   └── replay-sdk/                 Stable Rust SDK
├── docs/
│   ├── 00-START-HERE.md            Kick-off checklist, env setup, the spike
│   ├── 01-project-brief.md         The one-page pitch
│   ├── 02-architecture.md          System design, data flow, module boundaries
│   ├── 03-technical-spec.md        Concrete types, API contracts, error handling
│   ├── 04-solana-gotchas.md        Multi-chain-brain traps list
│   ├── 05-side-track-stack.md      Which prizes to target and how to qualify
│   ├── 06-demo-script.md           Judging demo, turn by turn
│   ├── 07-submission-checklist.md  What to hand in and where
│   └── KNOWN-FIXUPS.md             Day-1 first-hour API-name fixups
├── prompts/
│   ├── SYSTEM-PROMPT.md            Paste at the top of every Claude Code session
│   ├── day-01-spike-fetch-tx.md
│   ├── day-02-spike-replay.md
│   ├── day-03-idl-decoder.md
│   ├── day-04-trace-tree.md
│   ├── day-05-fork-sessions.md
│   ├── day-06-ui-scaffold.md
│   ├── day-07-timeline-scrubber.md
│   ├── day-08-account-mutator.md
│   ├── day-09-diff-view.md
│   ├── day-10-demo-preload.md
│   ├── day-11-cli.md
│   ├── day-12-ts-sdk.md
│   ├── day-13-rust-sdk.md
│   ├── day-14-helius-integration.md
│   ├── day-15-public-goods-polish.md
│   ├── day-16-regional-submission.md
│   ├── day-17-pitch-package.md
│   └── day-18-final-submission.md
├── memory/                         Local per-day status snapshots
├── scripts/                        Tooling (capture-fixture.sh)
└── spikes/                         Exploratory single-file scripts
```

## How to use this package

1. Read `docs/00-START-HERE.md` completely before you touch code.
2. For each working day, open `memory/day-XX.md` for the previous day's
   snapshot, then feed `prompts/SYSTEM-PROMPT.md` + `prompts/day-XX-*.md`
   to Claude Code as the first message of a focused session.
3. Refer to `docs/04-solana-gotchas.md` whenever something mysterious breaks.
   It will save hours.

## The one north-star rule

> **Ship a working demo of one real mainnet transaction being replayed, forked, mutated, and re-run — with visible diff — before you add any feature that isn't on that path.**

Everything else is scope you cut if time gets tight.
