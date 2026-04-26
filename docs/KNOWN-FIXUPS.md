# Known Fixups — Day 1 First Hour

The scaffolding is structured correctly but contains a handful of API surface guesses I made without verifying against live `cargo doc` output. Fix these before running the spike.

Hand this entire file to Claude Code at the start of Day 1 with the instruction: **"Open these files, run `cargo check`, and fix the field/method names against the real APIs. Do not change any logic."**

## 1. `litesvm` API surface — `crates/replay-core/src/svm.rs`

I assumed these field names on the `LiteSVM::send_transaction` result. Verify against `cargo doc --open -p litesvm` or the GitHub README:

- `meta.logs` — may actually be `meta.logs` or `meta.log_messages`. Check.
- `meta.compute_units_consumed` — likely correct, but verify.
- `meta.return_data.data` — `return_data` may be `Option<TransactionReturnData>`; the inner field name may differ.
- `failed.meta` and `failed.err` — the failure type might be a single `FailedTransactionMetadata` struct or a tuple. Check the exact shape.

If field names differ, just rename in `svm.rs::SvmRunner::execute`. Logic is correct.

## 2. `LiteSVM` constructor options

I used:
```rust
LiteSVM::new()
    .with_sigverify(false)
    .with_blockhash_check(false);
```

Verify these method names exist. They might be `with_sigverify` / `with_blockhash_check` (most likely) or differ in case/signature. If a method is missing, it's safe to drop it — defaults are usually permissive enough for a replay tool.

## 3. `solana-sdk` 2.x message accessor names — `crates/replay-core/src/trace.rs`

In `parse_log_frames`:
```rust
ctx.original_tx.message.is_signer(idx)
ctx.original_tx.message.is_maybe_writable(idx, None)
```

These are the v2.x SDK names; on older versions the names differ. If on a different SDK version, swap to the equivalent. The signature may also be `is_maybe_writable(idx)` without the second arg.

## 4. `solana_sdk::pubkey!` macro — `crates/replay-core/src/reconstruct.rs`

```rust
const LOADER_V4_ID: Pubkey = solana_sdk::pubkey!("LoaderV4...");
```

If the macro path is wrong, swap to:
```rust
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
let loader_v4_id = Pubkey::from_str("LoaderV411111111111111111111111111111111111").unwrap();
```
Or use a `lazy_static!` / `OnceCell` if `from_str` isn't const.

## 5. `bincode` version compatibility — `crates/replay-core/src/fetch.rs`

I used `bincode::deserialize`. If using bincode 2.x, the API is `bincode::serde::decode_from_slice` instead. Check `Cargo.toml`'s `bincode = "1.3"` is what actually resolved; if Solana pulls 2.x transitively there could be a conflict.

## 6. `compute_budget::id()` import — `crates/replay-core/src/fetch.rs`

The import path `solana_sdk::compute_budget` may instead be `solana_sdk::compute_budget::id` directly. If `compute_budget::id()` doesn't resolve, try:
```rust
use solana_compute_budget_interface::ID as compute_budget_id;
// or
const COMPUTE_BUDGET: Pubkey = solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");
```

## 7. `axum` 0.7 path extractor — `crates/replay-api/src/handlers.rs`

The `Path<String>` syntax is correct for axum 0.7+. If you're on 0.6 or earlier, the import path differs (`axum::extract::Path`).

## 8. `tower_governor` may need a layer wrap — `crates/replay-api/src/main.rs`

I haven't actually wired up rate limiting in `main.rs` — the dependency is there but not used. Add it on Day 5 with the real session work, not now.

## How to clear all of these

```bash
cd scaffolding
cargo check 2>&1 | tee /tmp/check.log
```

Then feed `/tmp/check.log` to Claude Code with: **"Fix every error. Don't change logic. Match real APIs."** Should take 15-30 minutes.

After `cargo check` passes:
```bash
cargo test
```

Then move to `prompts/day-01-spike-fetch-tx.md`.

## What is NOT a fixup

The architecture, the data flow, the type definitions, the error taxonomy, and the module boundaries are all correct. Don't let Claude Code "improve" any of these — they're load-bearing for the rest of the plan.
