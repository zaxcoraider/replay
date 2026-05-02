# Deployment Guide

## Architecture

```
Vercel (web)  →  Fly.io (replay-api)  →  Helius RPC
```

The web app is a static + SSR Next.js app deployed on Vercel.
The API is a Rust binary deployed on Fly.io.

---

## 1. Deploy the API to Fly.io

### Prerequisites
- [flyctl](https://fly.io/docs/hands-on/install-flyctl/) installed
- A Helius API key (free tier at helius.xyz is enough)

### Steps

```bash
# From repo root

# 1. Create the Fly app (first time only)
fly launch --no-deploy --name replay-api

# 2. Set your Helius key as a secret (never stored in fly.toml)
fly secrets set HELIUS_API_KEY=your_actual_key_here

# 3. Deploy
fly deploy

# 4. Verify
curl https://replay-api.fly.dev/health
# → {"status":"ok","version":"0.1.0"}
```

The API will be available at `https://replay-api.fly.dev`.

---

## 2. Deploy the web to Vercel

### Option A: Vercel CLI

```bash
cd web
npx vercel --prod
# When prompted for environment variables, set:
# NEXT_PUBLIC_REPLAY_API_URL = https://replay-api.fly.dev
```

### Option B: GitHub integration (recommended)

1. Go to [vercel.com](https://vercel.com) → New Project → Import `zaxcoraider/replay`
2. Set **Root Directory** to `web`
3. Add environment variable:
   - Key: `NEXT_PUBLIC_REPLAY_API_URL`
   - Value: `https://replay-api.fly.dev`
4. Deploy

---

## 3. Fill in demo signatures

Once the API is live, find 3 real mainnet transaction signatures:

| Demo | Where to find |
|------|--------------|
| Jupiter v6 swap | solscan.io → Programs → `JUP6Lkb...` → Transactions |
| Whirlpool swap | solscan.io → Programs → `whirLbMi...` → Transactions |
| Drift perp trade | solscan.io → Programs → `dRiftyHA...` → Transactions |

Paste them into `web/lib/demo-signatures.ts` replacing the `FILL_ME_*` values. Commit and push — Vercel auto-deploys.

---

## Environment variables reference

### replay-api (Fly.io secrets / env)

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `HELIUS_API_KEY` | ✅ | — | Helius API key for mainnet RPC |
| `REPLAY_BIND_ADDR` | — | `0.0.0.0:8080` | Listen address |
| `REPLAY_SESSION_TTL_SECS` | — | `3600` | Session expiry in seconds |
| `REPLAY_MAX_SESSIONS` | — | `100` | Max concurrent fork sessions |
| `RUST_LOG` | — | `info` | Log level |

### web (Vercel env / .env.local)

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `NEXT_PUBLIC_REPLAY_API_URL` | — | `http://localhost:8787` | API base URL |

---

## Local development

```bash
# Terminal 1 — API
cp .env.example .env
# edit .env → set HELIUS_API_KEY
cargo run -p replay-api        # http://localhost:8787

# Terminal 2 — Web
cd web
cp .env.example .env.local
# .env.local already defaults to http://localhost:8787
pnpm dev                       # http://localhost:3000
```
