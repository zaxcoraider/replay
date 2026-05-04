```
██████╗ ███████╗██████╗ ██╗      █████╗ ██╗   ██╗
██╔══██╗██╔════╝██╔══██╗██║     ██╔══██╗╚██╗ ██╔╝
██████╔╝█████╗  ██████╔╝██║     ███████║ ╚████╔╝
██╔══██╗██╔══╝  ██╔═══╝ ██║     ██╔══██║  ╚██╔╝
██║  ██║███████╗██║     ███████╗██║  ██║   ██║
╚═╝  ╚═╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝
```

> **Time-travel debugger for Solana transactions.**

Paste any mainnet signature. Replay it locally against the exact historical
account state. Fork the sandbox, mutate any account field, re-run, and diff.

[![CI](https://github.com/zaxcoraider/replay/actions/workflows/ci.yml/badge.svg)](https://github.com/zaxcoraider/replay/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/replay-sdk.svg)](https://crates.io/crates/replay-sdk)
[![npm](https://img.shields.io/npm/v/@zaxcoraider/replay-sdk)](https://www.npmjs.com/package/@zaxcoraider/replay-sdk)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Live demo

| Service | URL |
|---------|-----|
| Web UI | **https://replay-weld.vercel.app** |
| API | https://replay-y4wq.onrender.com *(proxied via Vercel `/rpc`)* |
| Docs | **https://replay-weld.vercel.app/docs** |

## How it works

```
 ┌─────────────┐   ┌──────────────┐   ┌──────────────────────┐
 │  replay-cli │   │ Next.js UI   │   │ TypeScript SDK       │
 │  (Rust bin) │   │ (Vercel)     │   │ @zaxcoraider/replay  │
 └──────┬──────┘   └──────┬───────┘   └──────────┬───────────┘
        │                 │                       │
        └─────────────────┴───────────────────────┘
                          │ HTTP / SSE
                 ┌────────▼─────────┐
                 │   replay-api     │
                 │ (axum · Render)  │
                 └────────┬─────────┘
                          │
                 ┌────────▼─────────┐
                 │   replay-core    │  ← engine
                 └────┬──────┬──────┘
                      │      │
          ┌───────────┘      └────────────┐
   ┌──────▼──────┐              ┌─────────▼──────┐
   │ Helius RPC  │              │ LiteSVM sandbox │
   │ (historical │              │ + IDL decoder   │
   │  state)     │              │ (Anchor, Drift, │
   └─────────────┘              │  Jupiter, more) │
                                └────────────────┘
```

**Pipeline per replay request:**
1. **Fetch** — pull tx + all accounts at exact slot from Helius
2. **Reconstruct** — hydrate LiteSVM with historical account state (LUT-resolved, upgradeable-program-aware)
3. **Replay** — execute in sandbox; compare result to mainnet
4. **Fork** — snapshot into mutable session
5. **Mutate** — change any account field (IDL-decoded for known programs)
6. **Re-run** — execute against the mutated state
7. **Diff** — result changed? CU delta? which accounts flipped?

## Quick start

```bash
# Prerequisites: Rust ≥ 1.79, a free Helius API key (https://helius.dev)

git clone https://github.com/zaxcoraider/replay
cd replay
cp .env.example .env          # add HELIUS_API_KEY=...
```

**CLI**
```bash
cargo run -p replay-cli -- replay <SIGNATURE>
cargo run -p replay-cli -- replay <SIGNATURE> --diff-logs
cargo run -p replay-cli -- inspect <SIGNATURE> --account <PUBKEY>
```

**API server + Web UI**
```bash
cargo run -p replay-api        # binds :8787
cd web && pnpm dev             # binds :3000 → http://localhost:3000
```

**Install the CLI globally**
```bash
cargo install replay-cli
```

## TypeScript SDK

```bash
npm install @zaxcoraider/replay-sdk
```

```ts
import { ReplayClient } from '@zaxcoraider/replay-sdk';

const client = new ReplayClient({ apiUrl: 'https://replay-y4wq.onrender.com' });

// One-shot replay
const trace = await client.replay('5xY...');
console.log('CU used:', trace.total_cu);

// Fork → mutate → re-run → diff
const session = await client.fork('5xY...');
await session.mutate(poolPubkey, { type: 'field', path: 'feeRate', new_value: 9999 });
await session.execute();
const diff = await session.diff();
console.log('Result changed:', diff.result_changed, '| CU delta:', diff.cu_delta);
```

```ts
// CI regression helper
import { replayHistorical, loadSignatures } from '@zaxcoraider/replay-sdk/testing';

const report = await replayHistorical({
  apiUrl: 'https://replay-y4wq.onrender.com',
  signatures: await loadSignatures('./fixtures/historical-swaps.txt'),
});
if (report.failures.length > 0) throw new Error('historical replay regressed');
```

## Rust SDK

```toml
[dependencies]
replay-sdk = "0.1"
```

```rust
use replay_sdk::{ReplayClient, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = ReplayClient::from_env()?; // reads HELIUS_API_KEY + REPLAY_API_URL

    let trace = client.replay("5xY...").await?;
    println!("CU: {}", trace.total_cu);

    let mut session = client.fork("5xY...").await?;
    session.mutate_field(pool_pk, "feeRate", serde_json::json!(9999))?;
    session.execute().await?;
    let diff = session.diff().unwrap();
    println!("Result changed: {}", diff.result_changed);
    Ok(())
}
```

See [`examples/`](examples/) for three runnable end-to-end examples.

## Repo layout

```
replay/
├── crates/
│   ├── replay-core/      # engine: fetch, reconstruct, execute, IDL decode
│   ├── replay-api/       # axum HTTP server + session store
│   ├── replay-cli/       # `replay` binary
│   └── replay-sdk/       # stable Rust SDK
├── packages/
│   └── replay-sdk-ts/    # TypeScript SDK (@zaxcoraider/replay-sdk)
├── web/                  # Next.js 15 web UI + live-replay panel
├── docs/                 # architecture, API reference, deploy guide
└── examples/             # runnable end-to-end examples
```

## Development

```bash
cargo check --workspace
cargo test -p replay-core --lib    # 26 unit tests (no network)
cargo test -p replay-api           # 7 integration tests (no network)
cargo clippy -- -D warnings

cd web && pnpm tsc --noEmit
cd web && pnpm build
```

## Published packages

| Package | Registry | Docs |
|---------|----------|------|
| [`@zaxcoraider/replay-sdk`](https://www.npmjs.com/package/@zaxcoraider/replay-sdk) | npm | [SDK guide](https://replay-weld.vercel.app/docs/ts-sdk) |
| [`replay-sdk`](https://crates.io/crates/replay-sdk) | crates.io | [SDK guide](https://replay-weld.vercel.app/docs/rust-sdk) |
| [`replay-core`](https://crates.io/crates/replay-core) | crates.io | [docs.rs](https://docs.rs/replay-core) |

## Deployment

| Service | Platform | URL |
|---------|----------|-----|
| API | Render (Docker) | https://replay-y4wq.onrender.com |
| Web | Vercel (Next.js) | https://replay-weld.vercel.app |
| Docs | Vercel (Next.js) | https://replay-weld.vercel.app/docs |

See [`docs/DEPLOY.md`](docs/DEPLOY.md) for full instructions.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a PR.

## License

MIT. See [LICENSE](LICENSE).
