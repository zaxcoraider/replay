# Day 15 — Public Goods Polish + Grant Application

**Date:** 2026-05-04
**Result:** Done. Repo now looks like a serious open-source project. Grant writeup drafted.

## What landed

### README.md — full rewrite
- ASCII banner at top
- CI / npm / crates.io / license badges
- ASCII architecture diagram (clients → api → core → Helius/LiteSVM)
- Tightened pitch — removed hackathon framing from the lead
- Contributing section linking to CONTRIBUTING.md
- Status table updated to Day 15

### CONTRIBUTING.md (new)
- Prerequisites table (Rust ≥ 1.79, Node ≥ 20, pnpm ≥ 9, Helius key)
- All env vars documented
- Full stack run instructions (API, web, CLI, TS SDK)
- Test commands (cargo test, clippy, tsc)
- Key gotchas section (HeliusClient trait, CorsLayer order, sync SSE callback)
- PR conventions and commit format

### CODE_OF_CONDUCT.md (new)
- Contributor Covenant 2.1, adapted with maintainer email

### .github/ (all new)
- `workflows/ci.yml` — runs on push/PR to main: cargo check, clippy, test (core + api), pnpm tsc, pnpm build
- `workflows/release.yml` — tag-triggered (v*.*.*), builds CLI binary for linux/mac/windows, creates GitHub release with artifacts
- `ISSUE_TEMPLATE/bug_report.md`
- `ISSUE_TEMPLATE/feature_request.md`
- `PULL_REQUEST_TEMPLATE.md`

### examples/ (all new)
- `examples/basic-replay/` — replay a single tx, print CPI trace (Rust)
- `examples/ci-regression/` — replay a list of sigs, fail on mismatch (TypeScript)
- `examples/mutation-analysis/` — fork → mutate → re-run → diff (Rust)
- Each example has its own README

### packages/replay-sdk-ts/package.json
- Added `homepage`, `bugs.url`
- Expanded keywords to include `blockchain`, `developer-tools`

### docs/grants/glass-surfers-application.md (new)
- $5k ask for VSCode extension milestone
- Covers: problem, solution, deliverables (6 weeks), team, public-goods framing
- Ready to paste into earn.superteam.fun/grants/glass-surfers-dev-tooling-grants

## Pending from Day 14 (still unresolved)
- `/replay-live/:sig` not smoke-tested locally (port :8787 proxy issue on Windows)
- Render redeploy pending — new route is in code but not on prod binary yet

## Next: Day 16 — Regional submission
- Read `prompts/day-16-regional-submission.md`
- Submit grant application at earn.superteam.fun
- Send LaserStream blog draft to Helius dev-rel
- Trigger Render redeploy (bakes in `/replay-live` route)
- Set repo topics on GitHub: solana, debugger, developer-tools, rust, typescript
- Pin repo on GitHub profile
