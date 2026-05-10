# Replay — Solana Time-Travel Debugger

> Paste any mainnet transaction signature. Replay it locally against the **exact historical account state**. Fork the sandbox, mutate any field, re-run, and see the diff.

[![CI](https://github.com/zaxcoraider/replay/actions/workflows/ci.yml/badge.svg)](https://github.com/zaxcoraider/replay/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/@zaxcoraider/replay-sdk)](https://www.npmjs.com/package/@zaxcoraider/replay-sdk)
[![Crates.io](https://img.shields.io/crates/v/replay-core.svg)](https://crates.io/crates/replay-core)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Live Demo

| | URL |
|---|---|
| **Web UI** | **https://replay-weld.vercel.app** |
| **Docs** | https://replay-weld.vercel.app/docs |
| **API** | https://replay-y4wq.onrender.com/health |

Try a real Jupiter V6 swap or Whirlpool trade — paste any Solana mainnet signature and Replay fetches, simulates, and diffs it in seconds.

---

## What is Replay?

Solana has no time-travel debugger. When a transaction behaves unexpectedly — wrong CU, failed simulation, unexpected account mutation — the only option is staring at logs.

**Replay changes that.** It reconstructs the exact on-chain state at the transaction's slot, re-executes in a local [LiteSVM](https://github.com/litesvm/litesvm) sandbox, and gives you a full interactive view of the CPI call tree, account deltas, and compute units. Then you can **fork** the state, mutate any account field (IDL-decoded for Jupiter, Whirlpool, Drift, and Kamino), and re-run to see exactly what changes.

### Core use cases

- **Debug** — why did my transaction fail? Which CPI call hit the CU limit?
- **Hypothesize** — what happens if I raise this pool's feeRate? What if this account has more lamports?
- **Regression test** — replay historical signatures in CI to catch simulation drift between program upgrades

---

## How It Works

```
  Browser / CLI / SDK
        │
        │  HTTP / SSE
        ▼
  ┌─────────────────┐
  │   replay-api    │  axum · Render · rate-limited
  └────────┬────────┘
           │
           ▼
  ┌─────────────────┐
  │   replay-core   │  engine (pure Rust)
  │                 │
  │  1. Fetch       │──► Helius RPC  (tx + all accounts at exact slot)
  │  2. Reconstruct │──► LiteSVM     (hydrate sandbox with historical state)
  │  3. Replay      │──► LiteSVM     (execute, compare to mainnet result)
  │  4. Fork        │──► SessionStore (snapshot into mutable session)
  │  5. Mutate      │──► IDL decoder  (Anchor · Jupiter · Whirlpool · Drift)
  │  6. Re-run      │──► LiteSVM     (execute against mutated state)
  │  7. Diff        │──► result changed? CU delta? which accounts?
  └─────────────────┘
```

**Pipeline per request:**

| Step | What happens |
|------|-------------|
| **Fetch** | Pull the transaction + every account it touched at the exact slot via Helius `getTransaction` |
| **Reconstruct** | Hydrate a LiteSVM sandbox with those accounts — LUT-resolved, upgradeable-program-aware |
| **Replay** | Execute in the sandbox; compare result and logs to mainnet |
| **Fork** | Snapshot the sandbox into an isolated mutable session |
| **Mutate** | Change any account field via IDL-decoded path, raw byte splice, or lamport override |
| **Re-run** | Execute the transaction against the mutated sandbox |
| **Diff** | Report: did the result change? CU delta? which account data changed? |

---

## Architecture

```
replay/
├── crates/
│   ├── replay-core/      # Engine: fetch, reconstruct, execute, IDL decode, fork, diff
│   ├── replay-api/       # Axum HTTP server · session store · rate limiting (tower-governor)
│   ├── replay-cli/       # `replay` binary · indicatif spinner · CPI table · inspect subcommand
│   └── replay-sdk/       # Stable Rust SDK (ReplayClient, Session, replay_historical)
├── packages/
│   └── replay-sdk-ts/    # TypeScript SDK (@zaxcoraider/replay-sdk on npm)
├── web/                  # Next.js 15 web UI · timeline scrubber · live-replay SSE panel
├── docs/                 # API reference, SDK guides, deploy instructions
└── examples/             # Three runnable end-to-end examples
```

### Key design decisions

**LiteSVM sandbox** — Full SVM execution without a validator. Fast enough for interactive use (~200 ms for most transactions).

**Helius `getTransaction` v0** — Returns account data at the exact slot, giving us historical state without needing an archival node.

**Fork sessions** — Snapshots are cheap (clone the in-memory LiteSVM state). Sessions expire after 1 h to cap memory on the free-tier Render instance.

**Bundled IDLs** — Jupiter, Whirlpool, Drift, and Kamino IDLs are compiled in at build time. No runtime IDL fetch; no Anchor CLI dependency.

**LaserStream / SSE** — The live-replay panel streams `frame_completed` events over SSE in real time. Falls back to standard RPC if `LASERSTREAM_GRPC_URL` is not set.

**CorsLayer outermost** — tower-http CORS middleware must wrap `GovernorLayer` so `OPTIONS` preflight requests are not rate-limited.

---

## Quick Start

**Requirements:** Rust ≥ 1.79 · a free [Helius API key](https://helius.dev)

```bash
git clone https://github.com/zaxcoraider/replay
cd replay
cp .env.example .env
# open .env and set HELIUS_API_KEY=your_key_here
```

### Run the API server + Web UI

```bash
# Terminal 1 — API (binds :8080)
cargo run -p replay-api

# Terminal 2 — Web UI (binds :3000)
cd web && npm install && npm run dev
```

Open http://localhost:3000, paste any Solana mainnet signature, and hit **Replay**.

### CLI

```bash
cargo install replay-cli

export HELIUS_API_KEY=your_key_here

# Full CPI trace + CU table
replay replay <SIGNATURE>

# Show log diff vs mainnet
replay replay <SIGNATURE> --diff-logs

# Inspect a specific account (IDL-decoded if known program)
replay inspect <SIGNATURE> --account <PUBKEY>
```

### Docker

```bash
docker build -t replay-api .
docker run -p 8080:8080 --env-file .env replay-api

curl http://localhost:8080/health
# → {"status":"ok","version":"0.1.0"}
```

---

## TypeScript SDK

Published on npm as [`@zaxcoraider/replay-sdk`](https://www.npmjs.com/package/@zaxcoraider/replay-sdk).

```bash
npm install @zaxcoraider/replay-sdk
```

```ts
import { ReplayClient } from '@zaxcoraider/replay-sdk';

const client = new ReplayClient({ apiUrl: 'https://replay-y4wq.onrender.com' });

// One-shot replay
const trace = await client.replay('5xYourSigHere...');
console.log('CU used:', trace.total_cu);
console.log('Mainnet result:', trace.mainnet_result.status);

// Fork → mutate → re-run → diff
const session = await client.fork('5xYourSigHere...');

await session.mutate(poolPubkey, {
  type: 'field',
  path: 'feeRate',
  new_value: 9999,
});

await session.execute();
const diff = await session.diff();

console.log('Result changed:', diff.result_changed);
console.log('CU delta:', diff.total_cu_delta);
console.log('Accounts changed:', diff.changed_accounts);
```

```ts
// CI regression helper — replay historical signatures, fail on divergence
import { replayHistorical, loadSignatures } from '@zaxcoraider/replay-sdk/testing';

const report = await replayHistorical({
  apiUrl: 'https://replay-y4wq.onrender.com',
  signatures: await loadSignatures('./fixtures/historical-swaps.txt'),
});

if (report.failures.length > 0) {
  throw new Error(`Historical replay regressed: ${report.failures.length} failures`);
}
```

Full SDK docs: [replay-weld.vercel.app/docs#typescript-sdk](https://replay-weld.vercel.app/docs#typescript-sdk)

---

## Rust SDK

Published on crates.io as [`replay-sdk`](https://crates.io/crates/replay-sdk).

```toml
[dependencies]
replay-sdk = "0.1"
```

```rust
use replay_sdk::{ReplayClient, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Reads HELIUS_API_KEY from env
    let client = ReplayClient::from_env()?;

    // One-shot replay
    let trace = client.replay("5xYourSigHere...").await?;
    println!("CU: {}", trace.total_cu);

    // Fork → mutate → re-run → diff
    let mut session = client.fork("5xYourSigHere...").await?;

    session.mutate_field(
        pool_pubkey,
        "feeRate",
        serde_json::json!(9999),
    )?;

    session.execute().await?;
    let diff = session.diff().unwrap();
    println!("Result changed: {}", diff.result_changed);
    Ok(())
}
```

```rust
// Historical regression batch
let report = replay_sdk::replay_historical(&client, &[
    "5xYourSigHere...",
    "3aBanotherSig...",
]).await?;

if report.has_failures() {
    eprintln!("{} signatures regressed", report.failures().count());
    std::process::exit(1);
}
```

Full SDK docs: [replay-weld.vercel.app/docs#rust-sdk](https://replay-weld.vercel.app/docs#rust-sdk)

---

## API Reference

Base URL: `https://replay-y4wq.onrender.com` · Rate limit: 20 req/min per IP · All endpoints return JSON.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/replay` | One-shot replay → full `Trace` |
| `POST` | `/fork` | Fork a tx into a mutable session → `session_id` |
| `POST` | `/session/:id/mutate` | Apply a mutation to the forked state |
| `POST` | `/session/:id/execute` | Re-execute the mutated session → `Trace` |
| `GET` | `/session/:id/diff` | Diff baseline vs latest → `TraceDiff` |
| `GET` | `/replay-live/:sig` | SSE stream of live replay events |

Full API reference: [replay-weld.vercel.app/docs#api-reference](https://replay-weld.vercel.app/docs#api-reference)

---

## Published Packages

| Package | Registry | Version |
|---------|----------|---------|
| [`@zaxcoraider/replay-sdk`](https://www.npmjs.com/package/@zaxcoraider/replay-sdk) | npm | 0.1.0 |
| [`replay-sdk`](https://crates.io/crates/replay-sdk) | crates.io | 0.1.0 |
| [`replay-core`](https://crates.io/crates/replay-core) | crates.io | 0.1.0 |

Rust crate API docs: [docs.rs/replay-core](https://docs.rs/replay-core)

---

## Deployment

| Service | Platform | URL |
|---------|----------|-----|
| API | Render (Docker) | https://replay-y4wq.onrender.com |
| Web | Vercel (Next.js) | https://replay-weld.vercel.app |

See [`docs/DEPLOY.md`](docs/DEPLOY.md) for step-by-step instructions.

---

## Development

```bash
# Run all checks (no network required)
cargo check --workspace
cargo test -p replay-core --lib     # 26 unit tests
cargo test -p replay-api            # 7 integration tests
cargo clippy -- -D warnings

# Web type-check + build
cd web && npm run build
```

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a PR.

## License

MIT — see [LICENSE](LICENSE).
