# Replay

> Time-travel debugger for Solana transactions.

Paste any mainnet transaction signature. Replay it locally against the exact
account state that existed at its slot. Mutate any account field. Re-run. See the diff.

Built for the Colosseum Frontier 2026 hackathon (deadline: **May 11, 2026**). MIT-licensed.

## Live demo

| Service | URL |
|---------|-----|
| Web UI | **https://replay-weld.vercel.app** |
| API | https://replay-y4wq.onrender.com *(Render, proxied via Vercel)* |
| Docs | **https://replay-weld.vercel.app/docs** |

## Status

**Day 13 of 18 done.** SDKs published. Docs live.

| Day | Topic | State |
|----:|-------|-------|
|  1  | Fetch tx + resolve LUTs + compute-budget | ✅ |
|  2  | Replay in litesvm + reconstruct historical state | ✅ |
|  3  | IDL-aware account decoder + bundled IDLs | ✅ |
|  4  | CPI trace tree (nesting + per-frame CU + decoded args) | ✅ |
|  5  | Fork sessions + HTTP API (axum, rate limiting, 7 tests) | ✅ |
|  6  | Web UI scaffold (landing + 3-panel replay view) | ✅ |
|  7  | Timeline scrubber (CU bar, clickable segments) | ✅ |
|  8  | Account mutator UI (fork → edit fields → re-run) | ✅ |
|  9  | Diff view (baseline vs forked, side-by-side) | ✅ |
| 10  | Demo preload + Dockerfile + deployment | ✅ |
| 11  | CLI polish (spinner, CPI table, `inspect` subcommand) | ✅ |
| 12  | TypeScript SDK (`@zaxcoraider/replay-sdk`) | ✅ |
| 13  | Rust SDK (`replay-sdk`) + docs site + npm/crates.io publish | ✅ |
| 14  | Helius LaserStream integration | 🔜 next |
| 15–18 | Polish → submission | planned |

## What it does

1. **Fetch** — pull the transaction + all referenced accounts at the exact slot from Helius
2. **Reconstruct** — hydrate a LiteSVM sandbox with the historical account state
3. **Replay** — execute the transaction in the sandbox; compare logs and result to mainnet
4. **Fork** — snapshot the sandbox state into a mutable session
5. **Mutate** — change any account field (IDL-decoded for Jupiter, Whirlpool, Drift, Kamino)
6. **Re-run** — execute the mutated transaction
7. **Diff** — side-by-side: result changed? CU delta? which accounts changed?

## Repo layout

```
replay/
├── crates/
│   ├── replay-core/          # engine: fetch, reconstruct, execute, IDL decode
│   ├── replay-api/           # axum HTTP server + session store
│   ├── replay-cli/           # `replay` CLI binary
│   └── replay-sdk/           # stable Rust SDK
├── packages/
│   └── replay-sdk-ts/        # TypeScript SDK (@zaxcoraider/replay-sdk)
├── web/                      # Next.js 15 web UI
├── docs/                     # architecture, API reference, deploy guide
├── prompts/                  # day-01..18 session prompts
└── memory/                   # per-day session snapshots
```

## Quick start

```bash
# 1. Set your Helius API key
cp .env.example .env
# edit .env → add HELIUS_API_KEY=...

# 2. CLI
cargo run -p replay-cli -- replay <SIGNATURE>
cargo run -p replay-cli -- replay <SIGNATURE> --diff-logs
cargo run -p replay-cli -- inspect <SIGNATURE> --account <PUBKEY>
cargo run -p replay-cli -- fetch <SIGNATURE> --json

# 3. API server
cargo run -p replay-api        # binds 0.0.0.0:8787

# 4. Web UI
cd web && pnpm dev             # http://localhost:3000
```

## TypeScript SDK

```bash
npm install @zaxcoraider/replay-sdk
```

```ts
import { ReplayClient } from '@zaxcoraider/replay-sdk';

const client = new ReplayClient({ apiUrl: 'https://replay-y4wq.onrender.com' });

// One-shot
const trace = await client.replay('5xY...');
console.log('CU:', trace.total_cu);

// Fork → mutate → re-run → diff
const session = await client.fork('5xY...');
await session.mutate(poolPubkey, { type: 'field', path: 'feeRate', new_value: 9999 });
const newTrace = await session.execute();
const diff = await session.diff();
console.log('Result changed:', diff.result_changed);
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
    let client = ReplayClient::from_env()?; // reads HELIUS_API_KEY

    let trace = client.replay("5xY...").await?;
    println!("CU: {}", trace.total_cu);

    let mut session = client.fork("5xY...").await?;
    session.mutate_field(pool_pk, "feeRate", serde_json::json!(9999))?;
    let _ = session.execute().await?;
    let diff = session.diff().unwrap();
    println!("Result changed: {}", diff.result_changed);
    Ok(())
}
```

## Development

```bash
cargo check --workspace
cargo test -p replay-core --lib          # 26 unit tests, no network
cargo test -p replay-api                 # 7 integration tests, no network
cargo clippy -- -D warnings

cd web && pnpm tsc --noEmit
cd web && pnpm build
```

## Published packages

| Package | Registry |
|---------|----------|
| [`@zaxcoraider/replay-sdk`](https://www.npmjs.com/package/@zaxcoraider/replay-sdk) | npm |
| [`replay-sdk`](https://crates.io/crates/replay-sdk) | crates.io |
| [`replay-core`](https://crates.io/crates/replay-core) | crates.io |

## Deployment

| Service | Platform | URL |
|---------|----------|-----|
| API | Render (Docker) | https://replay-y4wq.onrender.com |
| Web | Vercel (Next.js) | https://replay-weld.vercel.app |
| Docs | Vercel (Next.js) | https://replay-weld.vercel.app/docs |

See [`docs/DEPLOY.md`](docs/DEPLOY.md) for full instructions.

## License

MIT. See [`LICENSE`](LICENSE).
