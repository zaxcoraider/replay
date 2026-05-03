# Building a Time-Travel Debugger for Solana on Helius LaserStream

> Draft — for Helius dev-rel review. Author: zaxcoraider. Project:
> [github.com/zaxcoraider/replay](https://github.com/zaxcoraider/replay).
> Live demo: [replay-weld.vercel.app](https://replay-weld.vercel.app).

## TL;DR

We built [Replay](https://replay-weld.vercel.app), a time-travel debugger
for Solana mainnet transactions: paste any signature, watch it re-execute
locally against the exact historical state, fork into a sandbox, mutate any
account, re-run, and diff. The whole engine fits in a single Rust crate,
runs in litesvm, and produces a step-by-step CPI trace with decoded args
and account deltas — the kind of thing every Solana developer has wanted
since the first time `Program log: Custom program error: 0x1771` cost them
an afternoon.

The hard part is not the SVM. The hard part is faithfully reconstructing
the world a transaction saw — all the accounts it touched, the exact
bytecode the runtime had loaded for every program, the right sysvar
values, the LUT contents — at the moment it landed. For historical replay
we lean on Helius RPC. For *live* replay — replaying a transaction the
moment the validator confirms it, while it's still warm — we lean on
**Helius LaserStream**. This post is about what made each piece tractable,
where the gnarly bits live, and what we'd build next with deeper
LaserStream integration.

## The problem nobody else is solving

Solana has Anchor's logging, the Solana Explorer's transaction view, and
an excellent Helius enhanced API. None of these let you do the one thing
you actually want when something goes wrong on mainnet: **run the failing
transaction again, but with one tiny change, and see what would happen.**

- "What if this swap had 0.5% more slippage?"
- "What if this user's vault had three more SOL?"
- "What if I bump the mint's supply by one?"
- "What if the oracle returned a different price?"

These are forks of a real, observed transaction with a single account
mutation. There is no good way to do them today. The closest you can get is
spinning up a local validator with `solana-test-validator --clone <pubkey>
--clone <pubkey> --clone <pubkey>...` for every account the tx touched, hand-
patching them, and praying the bytecode matches. Replay does this in
~3 seconds, in the browser, against any mainnet signature.

## Why this couldn't have been built two years ago

Three things had to land before this was practical:

1. **litesvm** — Anza's lightweight in-process SVM. Spinning up a real
   validator per replay would be insane; litesvm gives you the runtime in
   a struct you can `seed()` and `execute()` against in milliseconds.

2. **Helius's historical state APIs** — `getAccountInfo` with
   `minContextSlot`, the upgradeable BPF loader's two-account layout
   exposed cleanly, and (critically) retention. Most public RPCs prune
   account state aggressively. Helius keeps enough that we can replay any
   tx from the last ~90 days reliably.

3. **LaserStream** — the gRPC streaming product Helius shipped on top of
   Yellowstone. This is what turned Replay from a batch tool into a live
   debugger. More on this in the next section.

Without any one of these, the project is either a toy or a six-month
infra build.

## Why LaserStream specifically

The first version of Replay was pure RPC. You'd paste a signature and we'd
serialize the pipeline:

```
getTransaction → resolve LUTs → fetch ~20 accounts at slot-1 →
fetch program-data accounts → seed litesvm → execute → decode trace
```

Most of the wall-clock time is the account fetches. Best case ~1.5s, worst
case (Jupiter aggregator routing through 6 pools, ~30 accounts) ~4s. That's
fine for "I want to inspect a tx I saw an hour ago." It is not fine for
"this transaction just landed and I want to see why it failed before the
user closes the tab."

LaserStream changes the shape of the problem. Instead of polling for the
transaction we want, the validator pushes:

- The slot, the moment it's confirmed.
- The transaction itself, with full meta, in the same push.
- Account writes, as they happen, for any pubkey we care to subscribe to.

For a live replay this means we don't have to wait for `getTransaction`
to settle and then fan out a dozen `getAccountInfo` calls. We can subscribe
*before* the tx lands, prefetch the accounts we know it'll touch the
moment the slot opens, and start re-executing the moment the meta arrives.

In practice the win is bigger than the latency math suggests, because the
*perception* changes. Even when the replay still takes 2 seconds end-to-
end, the UI now renders something on every event:

```
slot_observed → account_fetched (×20) → all_accounts_fetched →
execution_started → frame_completed (×N) → done
```

Each event is a few bytes; the browser draws one row per account as it
arrives, then watches the CPI tree assemble frame-by-frame. There is no
spinner state. Demoing Replay against `replay-weld.vercel.app` with the
Live tab open is, frankly, the most fun part of the project.

## Architecture, end to end

```
                ┌──────────────────────────────────────────────┐
                │  Helius (mainnet)                            │
                │   ┌──────────────┐    ┌──────────────────┐   │
                │   │ JSON-RPC     │    │ LaserStream gRPC │   │
                │   └──────┬───────┘    └────────┬─────────┘   │
                └──────────┼─────────────────────┼─────────────┘
                           │                     │
                ┌──────────▼─────────────────────▼─────────────┐
                │  replay-core (Rust)                          │
                │   fetch → reconstruct → litesvm → trace      │
                └──────────────────┬───────────────────────────┘
                                   │
                ┌──────────────────▼───────────────────────────┐
                │  replay-api (axum)                           │
                │    POST /replay   POST /fork                 │
                │    POST /session/:id/mutate                  │
                │    POST /session/:id/execute                 │
                │    GET  /session/:id/diff                    │
                │    GET  /replay-live/:signature   (SSE)      │
                └──────────────────┬───────────────────────────┘
                                   │
                ┌──────────────────▼───────────────────────────┐
                │  web (Next.js, Vercel)                       │
                │    Trace tree • Account inspector • Diff     │
                │    Live tab (EventSource → progressive UI)   │
                └──────────────────────────────────────────────┘
```

`replay-core` is the engine, ~3000 lines of Rust. The only thing it knows
about the network is a `HeliusClient` trait — three async methods, easy to
mock. Everything else (LUT resolution, the upgradeable BPF loader's
two-account layout, sysvar handling, IDL decode, CPI tree assembly) is
synchronous logic over deserialized data.

`replay-api` is a thin axum service. Six JSON endpoints plus the new
`/replay-live/:signature` SSE endpoint. The SSE handler caps concurrent
live sessions at 5 and auto-closes idle sessions after 5 minutes — every
live session holds a fetch loop and several MB of per-replay state.

`web` is Next.js on Vercel, with a `/rpc` rewrite that proxies to the
replay-api host. The Live tab is a single 250-line React component that
opens an `EventSource` and pushes incoming events into a reducer.

## Three Solana gotchas this project taught us

These are the parts that took the longest to get right. If you're building
anything that re-executes transactions from raw RPC data, you'll hit them.

### 1. LUT resolution is two-step

`getTransaction` returns `meta.loadedAddresses.{writable,readonly}` for V0
messages, but **the lookup table accounts themselves are not in the result.**
litesvm's V0-message path resolves lookups against its own account store at
execute time, so for every `address_table_lookups[].account_key` in the
versioned message you have to fetch the LUT account itself at slot-1 and
seed it into the runtime. Miss this and the runtime says "lookup table
not found" and you spend an hour staring at a `loaded_writable` list that
looks fine.

```rust
if let VersionedMessage::V0(v0) = &ctx.original_tx.message {
    for atl in &v0.address_table_lookups {
        let account = client
            .get_account_info_at_slot(&atl.account_key, fetch_slot)
            .await?
            .ok_or_else(|| ReplayError::LutResolution { ... })?;
        accounts.insert(atl.account_key, account);
    }
}
```

### 2. The upgradeable BPF loader is two accounts

When a program is deployed via `bpf_loader_upgradeable`, the program ID
holds a tiny stub (4-byte discriminator + 32-byte pubkey) pointing at a
*program-data* account that holds the actual ELF. You have to fetch both,
and you have to fetch them **at the slot of the transaction**, not now —
otherwise you're replaying against a newer version of the program than the
runtime saw. We classify each fetched account as native, non-upgradeable,
or upgradeable, and for the upgradeable ones do a second fetch:

```rust
let pda = Pubkey::try_from(&program_account.data[4..36])?;
let pda_account = client
    .get_account_info_at_slot(&pda, fetch_slot)
    .await?
    .ok_or_else(|| ReplayError::MissingProgramBytecode { ... })?;
```

This is also why the `account_fetched` count in our Live UI is sometimes
~30 for a tx that "only touches 20 accounts" — the program-data accounts
are real fetches.

### 3. Sysvars cannot be pulled forward

Clock, EpochSchedule, Rent, SlotHashes — litesvm builds these from its
own state, not from the seeded accounts. You have to call
`set_clock_for_slot(ctx.slot, ctx.block_time)` *before* execute, or the
replay's clock will read whatever litesvm initialized to and any tx that
checks `Clock::get()?` will diverge silently. You won't get a runtime
error; you'll get a different log output and a head-scratcher. The fix
is one line; finding it the first time was three hours.

## What the SSE stream actually looks like

The wire format is one tagged-union JSON object per SSE event:

```json
{"type":"mode","source":"laserstream"}
{"type":"slot_observed","slot":271234567,"block_time":1715000000}
{"type":"account_fetched","pubkey":"So11...112","size":82,"is_program":false}
{"type":"account_fetched","pubkey":"JUP6...n4","size":36,"is_program":true}
...
{"type":"all_accounts_fetched","count":24}
{"type":"execution_started"}
{"type":"frame_completed","frame":{ ... CpiFrame ... }}
{"type":"frame_completed","frame":{ ... CpiFrame ... }}
{"type":"done","trace":{ ... full Trace ... }}
```

When LaserStream is not configured on the server, the same stream is
served from a progressive walk over standard Helius RPC. The events are
identical; only the `mode.source` field differs (`"rpc"` vs
`"laserstream"`). This was a deliberate design choice — we wanted Replay
to be useful to anyone with a free Helius key and *more* useful to anyone
with LaserStream, rather than gating the whole live experience behind a
paid tier.

## Results

After 14 days of solo development:

- **~3000 lines of Rust** for the engine, **~2000 lines of TypeScript**
  for the web UI.
- **3 mainnet demo signatures** preloaded (Jupiter v6 swap, Whirlpool LP,
  Drift v2 perp).
- **Replay latency** end-to-end: ~2.4s p50, ~4s p99, dominated by the
  ~20 historical `getAccountInfo` calls per tx.
- **SDKs published**: [`@zaxcoraider/replay-sdk`](https://www.npmjs.com/package/@zaxcoraider/replay-sdk)
  on npm, [`replay-core`](https://crates.io/crates/replay-core) and
  [`replay-sdk`](https://crates.io/crates/replay-sdk) on crates.io.
- **Docs**: [replay-weld.vercel.app/docs](https://replay-weld.vercel.app/docs).

## What's next

The current LaserStream integration is shaped for a single transaction at
a time. The interesting next step is the bulk case: subscribe to a program
filter, replay every transaction targeting it as it lands, and surface
divergences (a swap that paid more or less than the AMM should have priced
it, an oracle update that came in stale, a vault that lost lamports between
ix N and ix N+1). That's a real-time invariant checker for any Solana
program, with no need to fork a custom validator. The engine is already
there; what's left is the LaserStream subscription topology and a UI for
"failures per minute" and "anomalies per slot."

If you're at Helius and reading this — we'd love feedback on the SSE event
shape, on what other gRPC subscription patterns would unlock further
latency wins, and on whether a hosted, multi-tenant version of Replay is
something the LaserStream team would want to co-promote. Replay is
MIT-licensed and the engine is on crates.io; the hosted version is one
Render service plus one Vercel project away from being your debugger too.

— *zaxcoraider, Colosseum Frontier 2026*
