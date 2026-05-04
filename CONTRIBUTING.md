# Contributing to Replay

Thank you for your interest in contributing. This doc covers everything you need to go from zero to a merged PR.

## Table of contents

- [Dev environment setup](#dev-environment-setup)
- [Running the full stack locally](#running-the-full-stack-locally)
- [Running tests](#running-tests)
- [Project architecture](#project-architecture)
- [PR conventions](#pr-conventions)
- [Commit message format](#commit-message-format)

---

## Dev environment setup

**Prerequisites**

| Tool | Version | Install |
|------|---------|---------|
| Rust | ≥ 1.79 | `rustup update stable` |
| Node.js | ≥ 20 | [nodejs.org](https://nodejs.org) |
| pnpm | ≥ 9 | `npm i -g pnpm` |
| Helius API key | free tier | [helius.dev](https://helius.dev) |

**Clone and configure**

```bash
git clone https://github.com/zaxcoraider/replay
cd replay
cp .env.example .env
# edit .env — set HELIUS_API_KEY
```

**.env keys**

| Key | Required | Description |
|-----|----------|-------------|
| `HELIUS_API_KEY` | Yes | Free tier is fine for development |
| `REPLAY_API_URL` | No | Default: `http://localhost:8787` |
| `LASERSTREAM_GRPC_URL` | No | Helius Business tier only — leave unset to use RPC fallback |
| `LASERSTREAM_X_TOKEN` | No | Auth token for LaserStream |
| `REPLAY_MAX_LIVE_SESSIONS` | No | Default 5 — max concurrent SSE sessions |

---

## Running the full stack locally

**1. API server**

```bash
cargo run -p replay-api
# Listens on http://0.0.0.0:8787
# Health check: GET /health → "ok"
```

**2. Web UI**

```bash
cd web
pnpm install
pnpm dev
# Opens http://localhost:3000
```

The web app proxies `/rpc/*` → `http://localhost:8787` via `next.config.ts` rewrites, so you don't need CORS headers in development.

**3. CLI**

```bash
cargo run -p replay-cli -- replay <SIGNATURE>
cargo run -p replay-cli -- replay <SIGNATURE> --diff-logs
cargo run -p replay-cli -- inspect <SIGNATURE> --account <PUBKEY>
cargo run -p replay-cli -- fetch <SIGNATURE> --json
```

**4. TypeScript SDK**

```bash
cd packages/replay-sdk-ts
pnpm install
pnpm build
pnpm typecheck
```

---

## Running tests

**Rust (no network required)**

```bash
# All workspace checks
cargo check --workspace
cargo clippy -- -D warnings

# Unit tests (26 tests, no network)
cargo test -p replay-core --lib

# API integration tests (7 tests, no network — uses mockito)
cargo test -p replay-api
```

**TypeScript**

```bash
cd web
pnpm tsc --noEmit
pnpm build
```

---

## Project architecture

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full design doc.

**Crate responsibilities**

| Crate | Responsibility |
|-------|---------------|
| `replay-core` | Engine: Helius RPC client, LUT resolution, LiteSVM sandbox, IDL decode, CPI trace, fork sessions |
| `replay-api` | Axum HTTP server, session store (DashMap), rate limiting (tower-governor), SSE live-replay |
| `replay-cli` | CLI binary: `replay`, `inspect`, `fetch` subcommands |
| `replay-sdk` | Stable Rust SDK: `ReplayClient`, `Session`, `replay_historical` |
| `replay-sdk-ts` | TypeScript SDK: `ReplayClient`, `Session`, CI testing helpers |

**Key gotchas**

- `HeliusClient` is a trait with a blanket impl for `Arc<dyn HeliusClient>`. Pass `&client`, not `&*client`, into `<C: HeliusClient>` generic functions.
- `CorsLayer` must be the outermost layer in the axum middleware stack (outside `GovernorLayer`).
- The SSE callback passed to `reconstruct_state_with_progress` is **sync** — use `mpsc::Sender::try_send`, never `await` inside it.
- Solana upgradeable-program accounts require two-step account loading (program account + program data account).
- LUT accounts must be fetched before resolving `MessageAddressTableLookup` entries.

---

## PR conventions

1. **One concern per PR.** Bug fix, feature, or refactor — not all three.
2. **Title format:** `type(scope): short description` — e.g. `fix(api): return 400 on invalid signature`
3. **Keep the test suite green.** All CI checks must pass.
4. **No new clippy warnings.** Run `cargo clippy -- -D warnings` locally before pushing.
5. **Update the relevant day diary** in `memory/` if your change affects the architecture.
6. **Don't touch `Cargo.lock` or `pnpm-lock.yaml`** unless your PR specifically updates dependencies.

---

## Commit message format

```
type(scope): subject

body (optional — explain WHY, not what)
```

Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `ci`

Examples:
```
feat(core): add reconstruct_state_with_progress callback
fix(api): place CorsLayer outside GovernorLayer
docs(sdk): add mutation examples to README
chore(deps): bump litesvm to 0.6
```
