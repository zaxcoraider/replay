# Replay

> Time-travel debugger for Solana transactions.

Paste any mainnet transaction signature. Replay it locally against the exact
account state that existed at its slot. Mutate any account. Re-run. See the
diff.

Built for the Colosseum Frontier 2026 hackathon. MIT-licensed from day 1.

## Status

Day 2 of 18 — full replay pipeline runs end-to-end against mainnet.
`fetch → reconstruct → seed → execute → trace` works through the
Cargo workspace; CPI-invoked programs and Address Lookup Table
accounts are seeded correctly; the integrated path passed
infrastructure validation against three real signatures (no
`MissingAccount` / `AddressLookupTableNotFound` / fetch errors).
Faithful-replay caveats (state drift on volatile accounts, 107-CU
remaining-budget drift on token transfers) are documented in
[`memory/day-02.md`](memory/day-02.md) — not Day-2 blockers; revisited
in Day 14 (Helius enhanced APIs).

The standalone spike at [`spikes/spike-replay.rs`](spikes/spike-replay.rs)
remains as reference for the proven approach.

| Day | Topic | State |
|----:|-------|-------|
|  1  | Fetch a tx + resolve LUTs + extract compute-budget | done |
|  2  | Replay in litesvm + reconstruct historical state + log diff | done |
|  3  | IDL-aware account decoder | next |
| 4-18 | Trace tree → fork/mutate → UI → CLI → SDK → submission | planned |

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
