# Replay

> Time-travel debugger for Solana transactions.

Paste any mainnet transaction signature. Replay it locally against the exact
account state that existed at its slot. Mutate any account. Re-run. See the
diff.

Built for the Colosseum Frontier 2026 hackathon (deadline: **May 11, 2026**). MIT-licensed from day 1.

## Status

**Day 7 of 18 done.** The full backend is live and the web UI scaffold is running.

- Engine (`replay-core`): fetch → reconstruct historical state → replay in litesvm → CPI trace tree with IDL-decoded args
- API (`replay-api`): axum HTTP server, fork/mutate/execute/diff session lifecycle, rate limiting, 7 integration tests
- Web (`web/`): Next.js 15 + Tailwind + shadcn. Landing page → `/replay/[sig]` → 3-panel view (CPI tree | frame detail | account inspector)

See [`memory/day-07.md`](memory/day-07.md) for the full session snapshot.

| Day | Topic | State |
|----:|-------|-------|
|  1  | Fetch tx + resolve LUTs + extract compute-budget | ✅ done |
|  2  | Replay in litesvm + reconstruct historical state + log diff | ✅ done |
|  3  | IDL-aware account decoder + bundled IDLs | ✅ done |
|  4  | CPI trace tree (nesting + per-frame CU + IDL-decoded args) | ✅ done |
|  5  | Fork sessions + HTTP API (axum, rate limiting, integration tests) | ✅ done |
|  6  | Web UI scaffold (landing + 3-panel replay view) | ✅ done |
|  7  | Timeline scrubber (CU bar, clickable segments) | ✅ done |
|  8  | Account mutator UI (fork → edit fields → re-run) | next |
|  9  | Diff view (baseline vs forked, side-by-side) | planned |
| 10  | Demo preload (real sigs + guided tour) | planned |
| 11  | CLI polish | planned |
| 12  | TypeScript SDK | planned |
| 13  | Rust SDK | planned |
| 14  | Helius integration | planned |
| 15–18 | Polish → submission | planned |

## Repo layout

```
replay/
├── Cargo.toml                # workspace root
├── README.md                 # this file
├── PLAN.md                   # 18-day hackathon plan
├── crates/
│   ├── replay-core/          # engine: fetch, reconstruct, execute, IDL decode
│   ├── replay-api/           # axum HTTP server + session store
│   ├── replay-cli/           # replay <signature> binary
│   └── replay-sdk/           # stable Rust SDK (Day 13)
├── web/                      # Next.js 15 web UI
├── docs/                     # project brief, architecture, API examples, etc.
├── prompts/                  # SYSTEM-PROMPT + day-01..18 session prompts
├── memory/                   # per-day session snapshots (start here each day)
├── scripts/                  # tooling (capture-fixture.sh, ...)
└── spikes/                   # exploratory single-file scripts
```

## Quick start

```bash
# 1. Set your Helius API key
cp .env.example .env
# edit .env → add HELIUS_API_KEY

# 2. Build Rust workspace
cargo build

# 3. CLI — fetch + replay
cargo run -p replay-cli -- fetch <SIGNATURE>
cargo run -p replay-cli -- replay <SIGNATURE>
cargo run -p replay-cli -- replay <SIGNATURE> --json

# 4. Start the API server
cargo run -p replay-api        # binds 0.0.0.0:8787

# 5. Start the web UI (separate terminal)
cd web && pnpm dev             # http://localhost:3000
```

## Development

```bash
# Rust
cargo check --workspace
cargo test -p replay-core --lib          # 27 unit tests, no network
cargo test -p replay-api                 # 7 integration tests, no network
cargo clippy -- -D warnings
cargo fmt

# Web
cd web && pnpm tsc --noEmit
cd web && pnpm build
cd web && pnpm dev
```

### Live (network) tests

```bash
# Requires HELIUS_API_KEY in .env
REPLAY_LIVE_TESTS=1 cargo test -p replay-core --test live_replay -- --ignored --nocapture
```

## Session bootstrap (next session = Day 7)

```bash
git pull origin main
cat memory/day-06.md    # full Day-6 snapshot
cat memory/day-05.md    # API context
cat prompts/day-07-timeline-scrubber.md
# Then start coding — no additional context needed.
```

## License

MIT. See [`LICENSE`](LICENSE).
