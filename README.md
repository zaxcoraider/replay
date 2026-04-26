# Replay

> Time-travel debugger for Solana transactions.

Paste any mainnet transaction signature. Replay it locally against the exact
account state that existed at its slot. Mutate any account. Re-run. See the
diff.

Built for the Colosseum Frontier 2026 hackathon. MIT-licensed from day 1.

## Status

Day 1 of 18 — fetch path complete. See [`memory/day-01.md`](memory/day-01.md)
for the full status snapshot. The standalone spike at
[`spikes/spike-replay.rs`](spikes/spike-replay.rs) already demonstrates the
full pipeline end-to-end against mainnet (fetch → state reconstruct → litesvm
replay → log diff).

| Day | Topic | State |
|----:|-------|-------|
|  1  | Fetch a tx + resolve LUTs + extract compute-budget | done |
|  2  | Spike replay in litesvm against historical state | next |
| 3-18 | IDL decoder → trace tree → fork/mutate → UI → CLI → SDK → submission | planned |

## Repo layout

```
replay/
├── Cargo.toml                # workspace root
├── README.md                 # this file
├── PLAN.md                   # 18-day hackathon plan
├── crates/
│   ├── replay-core/          # the engine: fetch, reconstruct, execute
│   ├── replay-api/           # axum HTTP/WebSocket server
│   ├── replay-cli/           # replay <signature> binary
│   └── replay-sdk/           # stable Rust SDK
├── docs/                     # 00-START-HERE, project brief, architecture,
│                             # technical spec, Solana gotchas, demo script,
│                             # submission checklist, KNOWN-FIXUPS
├── prompts/                  # SYSTEM-PROMPT + day-01..18 session prompts
├── memory/                   # local per-day status snapshots
├── scripts/                  # tooling (capture-fixture.sh, ...)
└── spikes/                   # exploratory single-file scripts
```

## Quick start

```bash
# 1. Set your Helius API key
cp .env.example .env
# edit .env and add HELIUS_API_KEY

# 2. Build
cargo build

# 3. Fetch a tx (Day-1: parses the tx + resolves LUTs, no replay yet)
cargo run -p replay-cli -- fetch <SIGNATURE>          # pretty summary
cargo run -p replay-cli -- fetch <SIGNATURE> --json   # full TxContext JSON

# 4. (Day-2+) Replay a tx
cargo run -p replay-cli -- replay <SIGNATURE>
```

## Development

```bash
cargo check --workspace      # ~20s incremental, several minutes cold
cargo test -p replay-core    # mock-backed unit tests, no network
cargo clippy -- -D warnings
cargo fmt
```

### Live (network) tests

```bash
# After populating tests/fixtures/demo-signature.txt with a real Jupiter swap sig:
REPLAY_LIVE_TESTS=1 HELIUS_API_KEY=... cargo test -p replay-core -- --ignored live_

# Capture a real Helius fixture:
HELIUS_API_KEY=... ./scripts/capture-fixture.sh <jupiter-swap-sig>
```

## Working with Claude Code

Each working day is one focused session. At the start of a session:

1. Open `memory/day-XX.md` for the most recent day to recover state.
2. Paste `prompts/SYSTEM-PROMPT.md` + the day's prompt (`prompts/day-XX-*.md`)
   as the first message.
3. Drive the diff — Claude Code will confidently produce wrong code for
   Solana-specific things (account layouts, PDA derivations, compute budget).
   You are the domain expert; it's the typist.

## License

MIT. See [`LICENSE`](LICENSE).
