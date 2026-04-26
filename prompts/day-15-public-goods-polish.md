# Day 15 — Public Goods Polish + Grant Application

## Goal

Make the repo look like a serious open-source project, not a hackathon dump. Apply for the Glass Surfers grant.

## Deliverables

### Repo polish

1. `README.md` rewrite. Structure:
   - Logo/banner (even a simple text one in monospace)
   - One-paragraph pitch
   - Animated GIF of the demo (record with `terminalizer` for CLI, Kap for UI)
   - Quick install (`cargo install`, `npm install`, web URL)
   - Usage examples (3 of them)
   - Architecture diagram (ascii or SVG)
   - Link to docs site
   - Contributing section
   - License

2. `CONTRIBUTING.md` with:
   - Dev environment setup
   - How to run tests
   - How to run the full stack locally
   - PR conventions
   - Architecture doc pointer

3. `CODE_OF_CONDUCT.md` — Contributor Covenant 2.1, adapted.

4. `.github/`:
   - `workflows/ci.yml` — test on PR
   - `workflows/release.yml` — tag-triggered, builds and publishes CLI binaries
   - `ISSUE_TEMPLATE/` — bug report + feature request
   - `PULL_REQUEST_TEMPLATE.md`
   - `FUNDING.yml` — if you have a GitHub Sponsors account

5. Repo metadata on GitHub:
   - Description: "Time-travel debugger for Solana transactions"
   - Topics: `solana` `debugger` `developer-tools` `rust` `typescript`
   - Pin the repo on your profile

6. Examples directory with 3 runnable examples:
   - `examples/basic-replay/`
   - `examples/ci-regression/`
   - `examples/mutation-analysis/`
   Each has its own README.

### Glass Surfers Dev Tooling Grant

1. Apply at `earn.superteam.fun/grants/glass-surfers-dev-tooling-grants`.
2. Writeup scope your $5k ask as "VSCode extension for in-editor tx replay" — a clear roadmap item beyond the MVP.
3. Include: repo link, demo video, roadmap, team info.

### crates.io + npm metadata

Every package should have:
- Description (≤160 chars, includes "Solana" keyword)
- Keywords: `solana`, `debugger`, `blockchain`, `developer-tools`
- Categories: `development-tools`, `api-bindings`
- Homepage, repository, documentation URLs
- License (MIT)

## What NOT to do

- No new features today. Repo polish only.
- Don't refactor. You will break things and lose hours.

## End-of-day

- Repo looks professional. A new visitor understands what Replay is in 10 seconds.
- Grant application submitted.
- `cargo package --list` and `pnpm pack` produce clean publishable bundles.
