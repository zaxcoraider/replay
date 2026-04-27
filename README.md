# Replay

> Time-travel debugger for Solana transactions.

Paste any mainnet transaction signature. Replay it locally against the exact
account state that existed at its slot. Mutate any account. Re-run. See the
diff.

Built for the Colosseum Frontier 2026 hackathon. MIT-licensed from day 1.

## Status

Day 3 of 18 — IDL-aware account decoder lives. `Trace.account_deltas`
now carries `decoded_before` / `decoded_after` / `idl_type_name`.
Three resolution layers: hand-written native decoders (SPL Token,
Token-2022, System) → bundled IDLs in `assets/idls/` (Jupiter v6,
Whirlpool, Drift v2, Kamino Lending pre-fetched, ~900 KB total) →
on-disk cache (`~/.replay/idl-cache/`, 7-day TTL) → on-chain Anchor
IDL fetched at the canonical PDA, zlib-decompressed. Failures degrade
gracefully through the `DecodedAccount` enum — never `Err`. See
[`memory/day-03.md`](memory/day-03.md) for the full snapshot.

The standalone spike at [`spikes/spike-replay.rs`](spikes/spike-replay.rs)
remains as reference for the proven replay approach.

| Day | Topic | State |
|----:|-------|-------|
|  1  | Fetch a tx + resolve LUTs + extract compute-budget | done |
|  2  | Replay in litesvm + reconstruct historical state + log diff | done |
|  3  | IDL-aware account decoder + bundled IDLs | done |
|  4  | Trace tree (CPI nesting + per-frame CU + IDL-decoded args) | next |
| 5-18 | Fork/mutate → UI → CLI → SDK → submission | planned |

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

# 3. Fetch a tx (parse + resolve LUTs + extract compute-budget; no replay)
cargo run -p replay-cli -- fetch <SIGNATURE>          # pretty summary
cargo run -p replay-cli -- fetch <SIGNATURE> --json   # full TxContext JSON

# 4. Replay a tx end-to-end against historical state
cargo run -p replay-cli -- replay <SIGNATURE>             # ✓ progress + log match
cargo run -p replay-cli -- replay <SIGNATURE> --diff-logs # side-by-side log diff
cargo run -p replay-cli -- replay <SIGNATURE> --json      # full Trace JSON
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
# Run the full replay gate against canonical sigs in tests/fixtures/canonical-sigs.txt.
# HELIUS_API_KEY auto-loads from .env (gitignored).
REPLAY_LIVE_TESTS=1 cargo test -p replay-core --test live_replay -- --ignored --nocapture

# Single-sig fetch test (asserts >10 resolved accounts for a Jupiter swap)
REPLAY_LIVE_TESTS=1 cargo test -p replay-core -- --ignored live_fetch

# Capture a real Helius response fixture
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
