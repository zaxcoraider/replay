import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Docs — Replay",
  description: "API reference, SDK guides, and architecture for the Replay Solana time-travel debugger.",
};

function H2({ id, children }: { id: string; children: React.ReactNode }) {
  return (
    <h2 id={id} className="text-xl font-bold text-zinc-100 mt-14 mb-4 scroll-mt-20 border-b border-zinc-800 pb-2">
      {children}
    </h2>
  );
}

function H3({ id, children }: { id?: string; children: React.ReactNode }) {
  return (
    <h3 id={id} className="text-base font-semibold text-zinc-200 mt-8 mb-3 scroll-mt-20">
      {children}
    </h3>
  );
}

function Code({ children }: { children: React.ReactNode }) {
  return <code className="bg-zinc-800 text-zinc-200 text-[12px] px-1.5 py-0.5 rounded">{children}</code>;
}

function Pre({ children }: { children: React.ReactNode }) {
  return (
    <pre className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 overflow-x-auto text-[12px] text-zinc-300 leading-relaxed my-4">
      {children}
    </pre>
  );
}

function Note({ children }: { children: React.ReactNode }) {
  return (
    <div className="border-l-2 border-zinc-600 pl-4 text-zinc-400 text-sm my-4">
      {children}
    </div>
  );
}

const NAV = [
  { href: "#overview", label: "Overview" },
  { href: "#api-reference", label: "API Reference" },
  { href: "#typescript-sdk", label: "TypeScript SDK" },
  { href: "#rust-sdk", label: "Rust SDK" },
  { href: "#cli", label: "CLI" },
  { href: "#self-hosting", label: "Self-hosting" },
  { href: "#architecture", label: "Architecture" },
];

export default function DocsPage() {
  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-300">
      {/* Top nav */}
      <header className="sticky top-0 z-10 border-b border-zinc-800 bg-zinc-950/90 backdrop-blur">
        <div className="max-w-5xl mx-auto px-6 h-12 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link href="/" className="text-sm font-semibold text-zinc-100 hover:text-white">
              ← Replay
            </Link>
            <span className="text-zinc-700">/</span>
            <span className="text-sm text-zinc-400">Docs</span>
          </div>
          <a
            href="https://github.com/zaxcoraider/replay"
            target="_blank"
            rel="noreferrer"
            className="text-[11px] text-zinc-500 hover:text-zinc-300"
          >
            GitHub ↗
          </a>
        </div>
      </header>

      <div className="max-w-5xl mx-auto px-6 py-10 flex gap-10">
        {/* Sidebar */}
        <aside className="hidden lg:block w-44 shrink-0">
          <nav className="sticky top-20 space-y-1">
            {NAV.map((n) => (
              <a
                key={n.href}
                href={n.href}
                className="block text-[12px] text-zinc-500 hover:text-zinc-200 py-1 transition-colors"
              >
                {n.label}
              </a>
            ))}
          </nav>
        </aside>

        {/* Main content */}
        <main className="flex-1 min-w-0 max-w-2xl">
          {/* ── Overview ── */}
          <H2 id="overview">Overview</H2>
          <p className="text-zinc-400 leading-relaxed mb-4">
            <strong className="text-zinc-200">Replay</strong> is a time-travel debugger for Solana transactions.
            Paste any mainnet signature, and Replay fetches the exact account state that existed at that slot,
            reconstructs it in a local{" "}
            <a href="https://github.com/litesvm/litesvm" target="_blank" rel="noreferrer" className="text-zinc-300 underline underline-offset-2">
              LiteSVM
            </a>{" "}
            sandbox, and re-executes the transaction. From there you can fork the state, mutate any field,
            re-run, and see a side-by-side diff.
          </p>

          <div className="grid grid-cols-2 gap-3 my-6">
            {[
              ["Fetch", "Pull tx + all accounts at exact slot via Helius"],
              ["Reconstruct", "Hydrate a LiteSVM sandbox with historical state"],
              ["Fork", "Snapshot sandbox into a mutable session"],
              ["Mutate", "IDL-decoded fields or raw byte splices"],
              ["Re-run", "Execute the mutated transaction"],
              ["Diff", "Result changed? CU delta? Which accounts?"],
            ].map(([k, v]) => (
              <div key={k} className="bg-zinc-900 border border-zinc-800 rounded-lg p-3">
                <div className="text-[11px] font-semibold text-zinc-200 mb-1">{k}</div>
                <div className="text-[11px] text-zinc-500">{v}</div>
              </div>
            ))}
          </div>

          {/* ── API Reference ── */}
          <H2 id="api-reference">API Reference</H2>
          <p className="text-zinc-400 text-sm mb-4">
            Base URL: <Code>https://replay-y4wq.onrender.com</Code>. All endpoints accept and return JSON.
            Rate limit: 20 req/min per IP.
          </p>

          <H3>Health</H3>
          <Pre>{`GET /health
# → {"status":"ok","version":"0.1.0"}

GET /version
# → {"name":"replay-api","version":"0.1.0"}`}</Pre>

          <H3 id="api-replay">POST /replay</H3>
          <p className="text-zinc-400 text-sm mb-2">One-shot replay. Returns a full <Code>Trace</Code>.</p>
          <Pre>{`POST /replay
Content-Type: application/json

{"signature": "5xYourSigHere..."}

# Response: Trace
{
  "frames": [...],          // CPI call tree
  "account_deltas": [...],  // accounts that changed
  "total_cu": 145823,       // compute units consumed
  "log_divergence": null    // null if logs match mainnet
}`}</Pre>

          <H3>POST /fork</H3>
          <p className="text-zinc-400 text-sm mb-2">Fork a transaction into a mutable session.</p>
          <Pre>{`POST /fork
{"signature": "5xYourSigHere..."}

# → {"session_id": "abc123"}`}</Pre>

          <H3>POST /session/:id/mutate</H3>
          <p className="text-zinc-400 text-sm mb-2">Apply a mutation to the forked state. Three mutation types:</p>
          <Pre>{`# 1. IDL field (dot-path into decoded account struct)
{"pubkey": "...", "mutation": {"type":"field","path":"feeRate","new_value":9999}}

# 2. Raw bytes splice
{"pubkey": "...", "mutation": {"type":"raw_bytes","offset":8,"bytes":"0f270000","extend":false}}

# 3. Lamports
{"pubkey": "...", "mutation": {"type":"lamports","new_value":1000000000}}`}</Pre>

          <H3>POST /session/:id/execute</H3>
          <Pre>{`POST /session/abc123/execute
# → Trace (same shape as /replay)`}</Pre>

          <H3>GET /session/:id/diff</H3>
          <Pre>{`GET /session/abc123/diff

{
  "result_changed": true,
  "total_cu_delta": -12345,
  "changed_accounts": ["pubkeyA", "pubkeyB"],
  "baseline": { ...Trace },
  "latest": { ...Trace }
}`}</Pre>

          <H3>Error shape</H3>
          <Pre>{`{
  "error": {
    "code": "tx_not_found",
    "message": "transaction not found on-chain"
  }
}

HTTP 400 — invalid signature / malformed body
HTTP 404 — tx or session not found
HTTP 422 — state reconstruction failure / bad mutation path
HTTP 429 — rate limit (20 req/min per IP)
HTTP 502 — Helius RPC error`}</Pre>

          {/* ── TypeScript SDK ── */}
          <H2 id="typescript-sdk">TypeScript SDK</H2>
          <Pre>{`npm install @replay/sdk`}</Pre>

          <H3>One-shot replay</H3>
          <Pre>{`import { ReplayClient } from '@replay/sdk';

const client = new ReplayClient({ apiUrl: 'https://replay-y4wq.onrender.com' });

const trace = await client.replay('5xYourSigHere...');
console.log('CU:', trace.total_cu);
console.log('Frames:', trace.frames.length);`}</Pre>

          <H3>Fork → mutate → re-run → diff</H3>
          <Pre>{`const session = await client.fork('5xYourSigHere...');

// Mutate an IDL-decoded field
await session.mutate(poolPubkey, {
  type: 'field',
  path: 'feeRate',
  new_value: 9999,
});

const newTrace = await session.execute();
const diff = await session.diff();

console.log('Result changed:', diff.result_changed);
console.log('CU delta:', diff.total_cu_delta);`}</Pre>

          <H3>CI regression helper</H3>
          <Pre>{`import { replayHistorical, loadSignatures } from '@replay/sdk/testing';

const report = await replayHistorical({
  apiUrl: 'https://replay-y4wq.onrender.com',
  signatures: await loadSignatures('./fixtures/historical-swaps.txt'),
});

if (report.failures.length > 0) {
  throw new Error(\`Historical replay regressed: \${report.failures.length} failures\`);
}`}</Pre>

          <H3>Types</H3>
          <Pre>{`interface Trace {
  frames: CpiFrame[];
  account_deltas: AccountDelta[];
  total_cu: number;
  mainnet_result: TxResult;
  replay_result: TxResult;
  log_divergence: LogDivergence | null;
}

interface CpiFrame {
  program_id: string;
  program_name: string | null;
  depth: number;
  cu_used: number;
  inner: CpiFrame[];
}

interface TraceDiff {
  result_changed: boolean;
  total_cu_delta: number;
  changed_accounts: string[];
  baseline: Trace;
  latest: Trace;
}`}</Pre>

          {/* ── Rust SDK ── */}
          <H2 id="rust-sdk">Rust SDK</H2>
          <Pre>{`# Cargo.toml
[dependencies]
replay-sdk = "0.1"`}</Pre>

          <H3>One-shot replay</H3>
          <Pre>{`use replay_sdk::{ReplayClient, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Reads HELIUS_API_KEY or REPLAY_RPC_URL from env
    let client = ReplayClient::from_env()?;

    let trace = client.replay("5xYourSigHere...").await?;
    println!("CU: {}", trace.total_cu);
    Ok(())
}`}</Pre>

          <H3>Fork → mutate → re-run → diff</H3>
          <Pre>{`let mut session = client.fork("5xYourSigHere...").await?;

session.mutate_field(
    "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".parse().unwrap(),
    "feeRate",
    serde_json::json!(9999),
)?;

let _trace = session.execute().await?;
let diff = session.diff().unwrap();
println!("Result changed: {}", diff.result_changed);`}</Pre>

          <H3>Historical regression batch</H3>
          <Pre>{`let report = replay_sdk::replay_historical(&client, &[
    "5xYourSigHere...",
    "3aBanotherSig...",
]).await?;

if report.has_failures() {
    eprintln!("{} signatures regressed", report.failures().count());
    std::process::exit(1);
}`}</Pre>

          {/* ── CLI ── */}
          <H2 id="cli">CLI</H2>
          <Pre>{`cargo install replay-cli

# Set your Helius key
export HELIUS_API_KEY=your_key_here

# Replay a transaction (full CPI trace + CU table)
replay replay 5xYourSigHere...

# Show log diff vs mainnet
replay replay 5xYourSigHere... --diff-logs

# Inspect a specific account (IDL-decoded)
replay inspect 5xYourSigHere... --account <PUBKEY>

# Fetch raw transaction JSON
replay fetch 5xYourSigHere... --json`}</Pre>

          <H3>Example output</H3>
          <Pre>{`⠙ Fetching transaction…
⠹ Reconstructing state (42 accounts)…
⠸ Replaying…

✓ Replayed in 234ms  ·  CU: 145,823  ·  Success

┌─────────────────────────────────────────────┬────────────┬──────┐
│ Program                                     │ CU Used    │ Depth│
├─────────────────────────────────────────────┼────────────┼──────┤
│ JUP6Lkbzwr3WkFBkUNHkNVTvsB8GpfZwMnzRmRt7Yf │ 98,234     │ 0    │
│   whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uct │ 32,100     │ 1    │
│   TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ  │ 4,512      │ 2    │
└─────────────────────────────────────────────┴────────────┴──────┘`}</Pre>

          {/* ── Self-hosting ── */}
          <H2 id="self-hosting">Self-hosting</H2>
          <Note>
            You need a <a href="https://helius.xyz" target="_blank" rel="noreferrer" className="text-zinc-300 underline underline-offset-2">Helius</a> API key.
            Free tier is sufficient for development.
          </Note>

          <H3>Docker (recommended)</H3>
          <Pre>{`# Clone
git clone https://github.com/zaxcoraider/replay
cd replay

# Set env
echo "HELIUS_API_KEY=your_key" > .env

# Build + run
docker build -t replay-api .
docker run -p 8787:8787 --env-file .env replay-api

# Verify
curl http://localhost:8787/health`}</Pre>

          <H3>From source</H3>
          <Pre>{`cp .env.example .env
# edit .env → add HELIUS_API_KEY

# API
cargo run -p replay-api        # http://localhost:8787

# Web UI (separate terminal)
cd web && pnpm install && pnpm dev   # http://localhost:3000`}</Pre>

          <H3>Environment variables</H3>
          <div className="overflow-x-auto">
            <table className="text-[12px] w-full border-collapse my-4">
              <thead>
                <tr className="border-b border-zinc-800 text-zinc-400 text-left">
                  <th className="py-2 pr-4 font-medium">Variable</th>
                  <th className="py-2 pr-4 font-medium">Required</th>
                  <th className="py-2 font-medium">Description</th>
                </tr>
              </thead>
              <tbody className="text-zinc-500">
                {[
                  ["HELIUS_API_KEY", "✅", "Helius RPC key for mainnet"],
                  ["REPLAY_BIND_ADDR", "—", "Listen address (default: 0.0.0.0:8787)"],
                  ["REPLAY_SESSION_TTL_SECS", "—", "Session expiry in seconds (default: 3600)"],
                  ["REPLAY_MAX_SESSIONS", "—", "Max concurrent sessions (default: 100)"],
                  ["RUST_LOG", "—", "Log level (default: warn)"],
                ].map(([k, r, d]) => (
                  <tr key={k} className="border-b border-zinc-900">
                    <td className="py-2 pr-4 font-mono text-zinc-300">{k}</td>
                    <td className="py-2 pr-4">{r}</td>
                    <td className="py-2">{d}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* ── Architecture ── */}
          <H2 id="architecture">Architecture</H2>
          <Pre>{`┌─────────────────────────────────────────────────────┐
│                    Replay                           │
│                                                     │
│  ┌──────────┐    ┌──────────────┐    ┌──────────┐  │
│  │  Web UI  │───▶│  replay-api  │───▶│  Helius  │  │
│  │ (Next.js)│    │   (axum)     │    │   RPC    │  │
│  └──────────┘    └──────┬───────┘    └──────────┘  │
│                         │                           │
│                  ┌──────▼───────┐                  │
│                  │ replay-core  │                   │
│                  │              │                   │
│                  │ • fetch      │                   │
│                  │ • reconstruct│                   │
│                  │ • LiteSVM    │                   │
│                  │ • IDL decode │                   │
│                  │ • fork/diff  │                   │
│                  └──────────────┘                   │
└─────────────────────────────────────────────────────┘`}</Pre>

          <H3>Crate layout</H3>
          <div className="overflow-x-auto">
            <table className="text-[12px] w-full border-collapse my-4">
              <thead>
                <tr className="border-b border-zinc-800 text-zinc-400 text-left">
                  <th className="py-2 pr-4 font-medium">Crate</th>
                  <th className="py-2 font-medium">Role</th>
                </tr>
              </thead>
              <tbody className="text-zinc-500">
                {[
                  ["replay-core", "Engine: fetch, reconstruct, execute, IDL decode, fork sessions"],
                  ["replay-api", "Axum HTTP server + session store + rate limiting"],
                  ["replay-cli", "CLI binary with spinner, CPI table, inspect subcommand"],
                  ["replay-sdk", "Stable Rust SDK (ReplayClient, Session, replay_historical)"],
                ].map(([k, v]) => (
                  <tr key={k} className="border-b border-zinc-900">
                    <td className="py-2 pr-4 font-mono text-zinc-300">{k}</td>
                    <td className="py-2">{v}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <H3>Key design decisions</H3>
          <ul className="space-y-2 text-sm text-zinc-400 list-none">
            {[
              ["LiteSVM sandbox", "Full SVM execution without a validator — fast enough for interactive use (~200ms for most txs)"],
              ["Helius getTransaction v0", "Provides account data at the exact slot, so we get historical state without archival node access"],
              ["Fork sessions", "Snapshots are cheap (clone the LiteSVM state); sessions expire after 1h to cap memory"],
              ["IDL bundles", "Jupiter, Whirlpool, Drift, and Kamino IDLs are bundled at compile time — no runtime IDL fetch needed"],
              ["CorsLayer outermost", "tower-http CORS middleware must wrap GovernorLayer so OPTIONS preflight requests aren't rate-limited"],
            ].map(([k, v]) => (
              <li key={k} className="bg-zinc-900 border border-zinc-800 rounded-lg p-3">
                <span className="font-semibold text-zinc-200">{k} — </span>
                {v}
              </li>
            ))}
          </ul>

          <div className="mt-16 pt-8 border-t border-zinc-800 text-[11px] text-zinc-600 flex items-center justify-between">
            <span>MIT licensed · Built for Colosseum Frontier 2026</span>
            <a
              href="https://github.com/zaxcoraider/replay"
              target="_blank"
              rel="noreferrer"
              className="hover:text-zinc-400"
            >
              GitHub ↗
            </a>
          </div>
        </main>
      </div>
    </div>
  );
}
