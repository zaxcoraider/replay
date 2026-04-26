# Replay — Build Package

**Project:** A time-travel debugger for Solana transactions.
**Hackathon:** Colosseum Frontier (deadline: **May 11, 2026**) + Superteam Earn side tracks.
**Builder:** Solo.
**AI leverage:** Claude Code.

This package is your complete operating manual. Read `docs/00-START-HERE.md` first, then work through the day-by-day prompts in `prompts/`.

## Package layout

```
replay-plan/
├── README.md                       ← you are here
├── docs/
│   ├── 00-START-HERE.md            Kick-off checklist, env setup, the spike
│   ├── 01-project-brief.md         The one-page pitch; paste into every Claude Code session
│   ├── 02-architecture.md          System design, data flow, module boundaries
│   ├── 03-technical-spec.md        Concrete types, API contracts, error handling
│   ├── 04-solana-gotchas.md        The multi-chain-brain traps list. Read before coding.
│   ├── 05-side-track-stack.md      Which prizes to target and how to qualify for each
│   ├── 06-demo-script.md           The judging demo — written out turn by turn
│   └── 07-submission-checklist.md  What to hand in and where
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
└── scaffolding/                    Drop-in starter code, ready to `cargo build`
    ├── Cargo.toml                  Workspace root
    ├── rust-toolchain.toml
    ├── .env.example
    ├── .gitignore
    ├── crates/
    │   ├── replay-core/            The engine — fetch, reconstruct, execute
    │   ├── replay-api/             axum HTTP + WebSocket server
    │   └── replay-cli/             `replay <signature>` binary
    └── web/                        Next.js 14 app (scaffolded separately)
```

## How to use this package

1. Read `docs/00-START-HERE.md` completely before you touch code.
2. Copy `scaffolding/` contents into a fresh git repo.
3. For each working day, open the corresponding `prompts/day-XX-*.md` file and feed it to Claude Code as the first message of a focused session. These prompts are written to be self-contained.
4. Refer to `docs/04-solana-gotchas.md` whenever something mysterious breaks. It will save hours.

## The one north-star rule

> **Ship a working demo of one real mainnet transaction being replayed, forked, mutated, and re-run — with visible diff — before you add any feature that isn't on that path.**

Everything else is scope you cut if time gets tight.
