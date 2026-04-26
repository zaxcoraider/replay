# Replay

> Time-travel debugger for Solana transactions.

Paste any mainnet transaction signature. Replay it locally against the exact
account state that existed at its slot. Mutate any account. Re-run. See the
diff.

## Status

Hackathon WIP. Submitted to Colosseum Frontier 2026.

## Quick start

```bash
# 1. Set your Helius API key
cp .env.example .env
# edit .env and add HELIUS_API_KEY

# 2. Build everything
cargo build --release

# 3. Replay a transaction (CLI)
cargo run -p replay-cli --release -- replay <SIGNATURE>

# 4. Or run the API server + web UI
cargo run -p replay-api --release
# then in another shell:
cd web && pnpm install && pnpm dev
```

## Architecture

```
replay-core    The engine. Fetch, reconstruct state, replay in litesvm.
replay-api     HTTP/WebSocket server. Sessions, mutations, diffs.
replay-cli     `replay <signature>` and friends.
replay-sdk     Stable Rust SDK for embedding Replay in other tooling.
web/           Next.js 14 UI.
```

See `docs/02-architecture.md` for the full design.

## Development

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt
```

## License

MIT. See `LICENSE`.
