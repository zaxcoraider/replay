# Bundled IDLs

Each file here is a pre-fetched Anchor IDL JSON, keyed by program ID:

```
<base58_program_id>.json
```

`IdlCache::default()` (in `src/idl.rs`) loads this directory at startup
via `load_bundled_idls()`. Lookups try bundled first, then the on-disk
cache (`~/.replay/idl-cache/`), then on-chain fetch. Bundling skips
network round-trips for common programs and lets the demo replay run
without a Helius IDL fetch.

## What to bundle

Targets per `prompts/day-03-idl-decoder.md`:

| Program | Program ID |
|---|---|
| Jupiter v6 | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` |
| Whirlpool | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` |
| Drift v2 | `dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH` |
| Kamino Lending | `KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD` |

SPL Token / Token-2022 / System are decoded by hand (see `idl.rs`'s
`decode_spl_token` + System branch); they don't need IDLs.

## How to populate

The on-disk cache and the bundled set use the same JSON schema, so the
fastest way is to let `IdlCache::get_or_fetch` do the fetch + write
once, then move the resulting file from the disk cache into this
directory:

```bash
# One-shot helper — uses replay-cli to trigger an IDL fetch via
# any tx that touches the program. Replace SIG with a real swap.
HELIUS_API_KEY=... cargo run -p replay-cli -- replay <SIG>
# Now the IDL lives at ~/.replay/idl-cache/<program_id>.json. Copy it:
cp ~/.replay/idl-cache/JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4.json \
   crates/replay-core/assets/idls/
```

Alternatively, paste a known-good IDL JSON directly:

```bash
cat my-idl.json > crates/replay-core/assets/idls/<program_id>.json
```

## When NOT to bundle

If a program publishes upgraded IDLs frequently, bundling will pin a
stale copy. The 7-day disk cache TTL doesn't apply to bundled entries;
they're always preferred. Drop the file from this directory if you
suspect drift.
