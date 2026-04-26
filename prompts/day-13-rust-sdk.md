# Day 13 — Rust SDK + Docs Site

## Goal

`replay-sdk` as a publishable crate on crates.io. Plus a docs site.

## Deliverables

1. `crates/replay-sdk/` — a thin wrapper around `replay-core` exposing only the stable public API. Purpose: hide internal types, present a clean surface, be the recommended entry point.

2. Public API:
   ```rust
   pub struct ReplayClient { /* ... */ }
   
   impl ReplayClient {
       pub fn new(rpc_url: impl Into<String>) -> Self;
       pub async fn replay(&self, signature: &str) -> Result<Trace, Error>;
       pub async fn fork(&self, signature: &str) -> Result<ForkedSession, Error>;
   }
   
   pub struct ForkedSession { /* ... */ }
   
   impl ForkedSession {
       pub fn mutate_field(&mut self, pubkey: Pubkey, path: &str, value: Value) -> Result<(), Error>;
       pub fn mutate_raw(&mut self, pubkey: Pubkey, offset: usize, bytes: &[u8]) -> Result<(), Error>;
       pub fn set_lamports(&mut self, pubkey: Pubkey, lamports: u64) -> Result<(), Error>;
       pub fn execute(&mut self) -> Result<Trace, Error>;
       pub fn diff(&self) -> TraceDiff;
   }
   ```

3. CI regression helper (match the TS one):
   ```rust
   pub async fn replay_historical(
       config: ReplayHistoricalConfig,
   ) -> Result<ReplayHistoricalReport, Error>;
   ```

4. Publish:
   - `cargo publish -p replay-sdk`
   - Version 0.1.0 — semver starts here.

5. Docs site: `docs-site/` — a simple mdBook or Docusaurus setup.
   - Guides: Quickstart, Concepts, Architecture.
   - API reference: generated from rustdoc + typedoc.
   - Examples: 5 real use cases.
   - Deploy to `docs.replay.dev` via Vercel.

## Concepts page (the crucial doc)

Explain, in 500 words, how Replay works and what guarantees it offers.
- What "faithful replay" means and its limits (stale program bytecode, pruned slots, clock nondeterminism).
- How forking works (snapshot, mutate, replay from snapshot).
- What mutation types are supported and when each is the right choice.
- When Replay is NOT the right tool (live debugging, real-time monitoring — point to other tools).

Honesty about limits builds trust with sophisticated users. Don't oversell.

## End-of-day

- Crates.io: `replay-sdk` version 0.1.0 live.
- Docs site live at its URL.
- README links to everything.
