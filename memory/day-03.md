# Day 3 — IDL-Aware Account Decoder

**Date:** 2026-04-27
**Prompt:** [`prompts/day-03-idl-decoder.md`](../prompts/day-03-idl-decoder.md)
**Result:** Done in one session. All deliverables landed including
the four bundled IDLs (Jupiter v6, Whirlpool, Drift v2, Kamino
Lending). The decoder is wired into `lib.rs::replay` so every
`Trace.account_deltas` entry now carries `decoded_before` /
`decoded_after` / `idl_type_name`.

## Goal

For any account in a reconstructed state, find the program's IDL (or
hand-decode known native programs) and produce a structured JSON tree
the UI can render. Failure modes degrade gracefully — never an `Err`,
always a `DecodedAccount` variant the UI can dispatch on.

## What landed

### `crates/replay-core/src/idl.rs` (full rewrite, ~600 LOC)

Three layers, in priority order:

1. **Owner-program dispatch** in `AccountDecoder::decode`. Hand-written
   byte-walk decoders for known native programs that don't have an IDL:
   - SPL Token (`Tokenkeg…`) — Account (165 bytes), Mint (82 bytes),
     Multisig (355 bytes)
   - Token-2022 (`Tokenz…`) — base layout decoded; extension bytes
     after byte 165 noted via `has_extensions` flag
   - System program (`1111…`) — lamports + raw data hex

2. **`IdlCache`** with three-tier resolution:
   - **bundled** — `assets/idls/<program_id>.json`, loaded at
     `IdlCache::default()` startup via `load_bundled_idls()`. Path
     is baked at compile time via `env!("CARGO_MANIFEST_DIR")`, so
     it only works when running from the source tree. Acceptable
     for now; SDK consumers get an empty bundled set and fall through.
   - **disk cache** — `~/.replay/idl-cache/` (or
     `REPLAY_IDL_CACHE_DIR`), 7-day TTL per `docs/03-spec`.
   - **on-chain** — Anchor IDL account at the canonical PDA:
     `Pubkey::create_with_seed(find_program_address(&[], program_id).0,
     "anchor:idl", program_id)`. Account layout: `[disc:8][auth:32]
     [len:u32 LE][zlib(idl_json)]`. Decompressed with
     `flate2::read::ZlibDecoder` and parsed as JSON.
   - Plus `manual_insert_from_json` for the bring-your-own-IDL path.

3. **Anchor Borsh-via-IDL recursive decoder** (`decode_anchor` →
   `decode_type` → `decode_primitive`). Handles every Anchor IDL type
   shape we've seen in the wild:
   - Primitives: `u8/i8` … `u64/i64`, `u128/i128` (str-encoded so JSON
     doesn't lose precision), `f32/f64`, `bool`, `string`, `bytes`,
     `publicKey`/`pubkey`
   - Composites: `option`, `vec`, fixed `array` (length pinned), `defined`
     (lookup into `idl.types[]`), nested `struct`, `enum` (named-field
     + tuple variants, both with `{ variant, payload }` shape)
   - Discriminator match: SHA-256(`account:<Name>`)[..8] via
     `solana_program::hash::hash`

`DecodedAccount` enum has 5 variants:
`Decoded` (IDL-decoded, with `IdlSource: Bundled|Cached|OnChain|Manual`),
`Native` (hand-decoded SPL/System), `UnknownDiscriminator`, `NoIdl`,
`NotAnchor`. Decoder *never errors* — every failure path becomes a
variant the UI can render.

### `crates/replay-core/src/lib.rs`

Added `decode_account_deltas(trace, ctx, execution, decoder, client)`
which walks `trace.account_deltas` and decodes each `before` / `after`
account through `AccountDecoder::decode`. `lib.rs::replay` calls it
after `build_trace`, so the public `Trace` returned to SDK consumers
already has `decoded_before` / `decoded_after` populated. `idl_type_name`
gets set when the decoder yields a `Decoded` or `Native` variant.

### `crates/replay-core/assets/idls/`

Pre-fetched Anchor IDLs for the four programs the prompt named:

| Program | File size | Source |
|---|---:|---|
| Jupiter v6 | 60 KB | on-chain at PDA |
| Whirlpool | 194 KB | on-chain at PDA |
| Drift v2 | 427 KB | on-chain at PDA |
| Kamino Lending | 231 KB | on-chain at PDA |

Plus an `assets/idls/README.md` documenting the convention and the
re-bundling command.

### `crates/replay-core/examples/bundle_idls.rs`

One-shot maintenance binary that fetches the canonical IDLs via the
production `IdlCache::get_or_fetch` path and writes them into
`assets/idls/`. Used to populate the bundled set; re-run when
upgrading bundled IDLs after upstream program updates.

```bash
HELIUS_API_KEY=... cargo run -p replay-core --example bundle_idls
```

## Tests

```text
cargo test -p replay-core --lib
  18 passed; 0 failed; 1 ignored (live)
```

| Test | Status |
|------|--------|
| `idl::tests::anchor_idl_pda_is_deterministic` | green |
| `idl::tests::anchor_account_discriminator_matches_expected` | green |
| `idl::tests::spl_mint_decoder_round_trips_a_known_layout` | green |
| `idl::tests::spl_token_account_decoder_round_trips` | green |
| `idl::tests::decode_dispatches_to_spl_token` | green |
| `idl::tests::decode_returns_no_idl_for_unknown_owner` | green |
| `idl::tests::anchor_borsh_decode_struct_with_primitives` | green |
| `idl::tests::anchor_borsh_decode_option_vec_array_defined` | green |
| `idl::tests::idl_cache_manual_insert_and_read_disk` | green |
| All Day-1 + Day-2 tests | still green |

`cargo check --workspace` clean. The `bundle_idls` example is the
strongest end-to-end test: a successful bundle of 4 mainnet programs
proves PDA derivation + on-chain fetch + zlib decompression + JSON
parse all work against real Helius traffic.

## Decisions worth remembering

- **The decoder never returns `Err`.** Even when IDL fetch panics
  internally, the path catches it and returns `NoIdl`. Reasoning:
  the trace UI always wants something to render, and "this account's
  decoder choked" is a worse UX than "no IDL, here's the hex." Bug
  reports should still surface via `tracing::warn!`.

- **`u128`/`i128` serialize as strings.** JSON's number type is f64;
  emitting `(2^53)+1` as a number silently corrupts. The Anchor
  decoder str-encodes anything that doesn't fit f64. `u64`/`i64` are
  also string-encoded for consistency — pretty-printers can format
  later.

- **Bundled IDLs are preferred over disk-cached.** Disk cache has a
  7-day TTL; bundled IDLs are always preferred (no TTL). Trade-off:
  if Anchor pushes a backwards-incompatible IDL change for one of
  the bundled programs, the bundled stale copy wins until manually
  refreshed via `bundle_idls`. Documented in `assets/idls/README.md`.

- **`env!("CARGO_MANIFEST_DIR")` is brittle for shipped binaries.**
  The bundled-IDL path is baked at compile time — fine when running
  from the source tree, broken when shipping a binary to a different
  machine. SDK consumers should `manual_insert_from_json` if they
  need offline IDL access. Worth revisiting for the published
  `replay-sdk` (Day 13).

- **Discriminator is SHA-256, not Keccak.** Anchor's account
  discriminator is `sha256("account:<Name>")[..8]`. We use
  `solana_program::hash::hash` rather than pulling in `sha2`
  directly — solana-program is already a transitive dep.

- **Owner-dispatch happens BEFORE IDL fetch.** SPL Token accounts
  have owner = SPL Token program, which doesn't publish an Anchor
  IDL. Without the dispatch, we'd hit `NoIdl` and miss every token
  decoding. The dispatch list is hardcoded for now; could become
  registry-driven later.

## Followups (deferred)

1. **Bundle SPL Token / Token-2022 / System "pseudo-IDLs"** so the
   `idl_source` field is consistent with the Anchor path. Currently
   their decoded form lands as `Native` with no `IdlSource`. Cosmetic;
   the UI doesn't care today but Day-9 (diff view) might.
2. **`AccountDelta.idl_type_name` is set from the first non-`NoIdl`
   variant** between before/after. If `before=NoIdl` and `after=Decoded`,
   we get the type from `after`. If they differ (account changed
   owner during execution), we currently take the first one. That's
   a rare edge case but worth noting.
3. **Anchor v0.30+ "new IDL format"**: this codebase decodes the
   pre-0.30 format (string types like `"u64"` and `"publicKey"`).
   Newer IDLs may use `{"defined": {"name": "Foo"}}` instead of
   `{"defined": "Foo"}`. Surfaces as `unrecognised type shape` warns.
   Easy to add when we hit it.
4. **CPI tree decoding** — the Day-4 trace tree should also decode
   instruction args via IDL. Trace currently has `data_hex`; IDL
   decoder needs the program's `instructions` schema. Day-4 work.
5. **`return_data` decoding** — `ExecutionResult.return_data` is
   a raw byte vector. IDL has return-type info on instructions;
   could decode. Day-4 work.

## Next session bootstrap (Day 4)

```bash
git pull origin main
cat memory/day-03.md  # this file
cat memory/day-02.md  # for context

# Day-4 prompt is the trace tree (CPI nesting + per-frame CU + IDL-decoded args)
#   prompts/day-04-trace-tree.md
# Plus standing context:
#   prompts/SYSTEM-PROMPT.md
#   docs/01-project-brief, 02-architecture, 03-technical-spec, 04-solana-gotchas

# Smoke before writing code
cargo check --workspace
cargo test -p replay-core --lib
```

Day-4 north-star: turn the flat `Vec<CpiFrame>` (one per top-level
instruction) into a real CPI tree. Parse the log lines as a state
machine — `Program X invoke [N]` pushes, `Program X consumed K of M
compute units` annotates, `Program X success/failed` pops. Each frame
gets IDL-decoded `decoded_args` via `AccountDecoder` against the
program's `instructions` schema. Today's `trace::parse_log_frames` is
a flat-only stub; the data structures are right, the parser needs to
respect depth.

## Commits

```
c0e941c feat(core): bundle 4 canonical IDLs + bundle_idls maintenance example
6fd6b94 feat(core): Day-3 IDL-aware account decoder
7f74b60 docs: README — Day-2 done, Day-3 next; refresh smoke commands
413afde docs: update day-02 snapshot with live-gate run results
f93a277 test(core): wire live-gate runtime — dotenv + real canonical sigs
ddc92fc fix(core): seed CPI-invoked programs and ALT lookup tables
e3bda12 docs: update day-02 snapshot with post-audit cleanups
b7f6994 refactor(core): drop guarded .unwrap() in trace.rs frame popper
8efd96d feat(core): populate pre_account_snapshots from reconstructed state
773a5ea fix(core): pass minContextSlot in get_account_info_at_slot
732461c docs: memory/day-02 snapshot
2cda1a8 test(core): live integration test for canonical replay sigs
d04bf9c feat(cli): Day-2 step-by-step replay output
fc4bf30 docs: project README + memory/day-01 snapshot
b99af6c chore: add fixture-capture script + lockfile
2d73413 feat(cli): print TxContext as JSON when --json flag is set
fae9d46 feat(core): Day-1 fetch deliverables — TxContext JSON, mock+live tests
e1dfb0c chore(core): add ulid dep, drop unused imports
537a019 chore: initial scaffold + working replay spike
```
