# Day 10 — Demo Preload + Live Deployment

**Date:** 2026-05-02
**Prompt:** [`prompts/day-10-demo-preload.md`](../prompts/day-10-demo-preload.md)
**Result:** Done in one session. Build clean, deployment config ready.

## Goal

Ship the demo story and the deployment plumbing. Hard-code 3 "try me" experiences and make the app deployable with one command.

## What landed

### `web/lib/demo-signatures.ts`

Typed `DEMOS` array with 3 canonical scenarios:
- **Jupiter v6 swap** — mutate `feeRate` to 9999, route fails with insufficient output
- **Whirlpool CLMM swap** — zero out `liquidity`, swap can't fill, fails
- **Drift perp trade** — crash `lastOracleNormalisedPrice`, position liquidated

Each demo has: `id`, `title`, `subtitle`, `signature`, `narrative`, `tags`, `suggested_mutation` (account_label, field, new_value, description).

`isPlaceholder(sig)` helper — returns true when sig starts with `FILL_ME_`. Cards are disabled and show a warning until real sigs are inserted.

**To activate demos:** find 3 recent mainnet transactions on solscan.io for each program and paste into `signature` fields.

### `web/app/page.tsx` — polished landing page

- Gradient hero title with feature pills (Fork sessions, IDL-decoded fields, CPI trace tree, Side-by-side diff)
- Full-width sig input + "Replay →" button with spinner state
- 3 demo cards in a responsive grid (1-col mobile, 3-col sm+): tags, title, subtitle, narrative, mutation hint
- Footer with Colosseum Frontier + GitHub links
- Disabled cards show "Sig needed — see demo-signatures.ts"

### `Dockerfile` — multi-stage Rust build

Builder stage:
- `rust:1.79-slim` base
- Dependency-cache layer (stub main.rs files, `cargo fetch`)
- Full source copy + `cargo build --release -p replay-api`

Runtime stage:
- `debian:bookworm-slim`
- Copies binary to `/usr/local/bin/replay-api`
- Copies `crates/replay-core/assets` to match compile-time `CARGO_MANIFEST_DIR` path so bundled IDLs are found at runtime
- ENV: `REPLAY_BIND_ADDR=0.0.0.0:8080`, `REPLAY_IDL_CACHE_DIR=/app/.replay/idl-cache`

### `fly.toml`

- App: `replay-api`, region: `ord`
- `shared-cpu-1x`, 512 MB RAM
- Auto-stop when idle, auto-start on request
- `internal_port = 8080`, `force_https = true`
- Env vars: session TTL, max sessions, log level

### `web/vercel.json` + `web/.env.example`

- `vercel.json`: framework=nextjs, buildCommand=pnpm build
- `.env.example`: documents `NEXT_PUBLIC_REPLAY_API_URL` env var

### `docs/DEPLOY.md`

Step-by-step guide:
1. Fly.io deploy (`fly launch` → `fly secrets set HELIUS_API_KEY=...` → `fly deploy`)
2. Vercel deploy (CLI or GitHub integration, set `NEXT_PUBLIC_REPLAY_API_URL`)
3. Fill demo signatures (solscan links for each program)
4. Full env var reference table

## What still needs manual work

1. **Fill demo signatures** — find 3 real recent mainnet txs and paste into `demo-signatures.ts`
2. **Create Fly.io account + app** — run `fly launch --no-deploy --name replay-api`
3. **Create Vercel project** — connect GitHub repo, set env var
4. **Record 2-minute demo video** with OBS

## Tests / build

```bash
cd web && pnpm tsc --noEmit   # clean
cd web && pnpm build           # clean
```

## Commits

```
d069fcf feat(day-10): demo preload, landing page polish, deployment config
```

## Next: Day 11 — CLI polish

The `replay-cli` binary exists but needs polish:
- Better output formatting (table, colors)
- `replay replay <sig>` output improvement
- `--json` flag cleanup
- Help text

Key files: `crates/replay-cli/src/main.rs`
