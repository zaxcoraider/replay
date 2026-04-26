# Day 1 — Spike Part 1: Fetch and parse a mainnet transaction

## Goal for today

By end of day: given a mainnet transaction signature, we can fetch it from Helius and produce a structured list of every account it touches (including LUT-resolved accounts, program accounts, and program-data accounts), with the slot it executed at.

**No replay yet. Just faithful fetching.** This is 50% of the spike.

## Constraints

- Work in `crates/replay-core/src/fetch.rs` and `crates/replay-core/src/rpc.rs`.
- `HeliusClient` must be a trait + concrete impl, so tests can mock it.
- Use `tokio` + `reqwest` with JSON features.
- No `.unwrap()` outside of tests. Every error path uses `ReplayError` from `error.rs`.

## Deliverables

1. `crates/replay-core/src/rpc.rs`: `HeliusClient` trait + `HeliusRpcClient` impl with methods:
   - `get_transaction(sig: &Signature) -> Result<FetchedTx, ReplayError>`
   - `get_account_info_at_slot(pubkey: &Pubkey, slot: u64) -> Result<Option<Account>, ReplayError>`
   - `get_account_info(pubkey: &Pubkey) -> Result<Option<Account>, ReplayError>`

2. `crates/replay-core/src/fetch.rs`: `fetch_full_tx_context` function:
   ```rust
   pub async fn fetch_full_tx_context<C: HeliusClient>(
       client: &C,
       signature: &Signature,
   ) -> Result<TxContext, ReplayError>;
   ```
   Returns a `TxContext` containing:
   - Original transaction (`VersionedTransaction`)
   - Slot it executed at
   - Block time
   - Full resolved account list (static keys + loaded writable + loaded readonly from LUTs)
   - Mainnet logs (from `meta.log_messages`)
   - Mainnet result (success or error)
   - Compute-budget instructions extracted separately
   - Pre-balances and post-balances for reference

3. `crates/replay-core/src/types.rs`: `TxContext`, `FetchedTx`, relevant structs.

4. Unit test in `crates/replay-core/src/fetch.rs` using a `MockHeliusClient` that returns a canned response for a Jupiter swap signature stored in `tests/fixtures/jupiter-swap-response.json`. Download one real response and commit it.

## Specific Solana gotchas to handle

1. **LUT resolution.** When calling `getTransaction`, set `maxSupportedTransactionVersion: 0`. The response `meta.loaded_addresses: { writable, readonly }` contains the LUT-resolved addresses. Concatenate in order: `static_account_keys ++ loaded_writable ++ loaded_readonly`. Index into this combined list is what `AccountMeta.account_index` refers to.

2. **Encoding.** Use `encoding: "base64"` for faithful parse. `jsonParsed` is lossy. Use `solana_sdk::transaction::VersionedTransaction::deserialize` from the base64 bytes.

3. **Block time.** `getTransaction` response has `blockTime: i64 | null`. Store as `Option<i64>`. Required later for `Clock` sysvar setup.

4. **Compute budget instructions.** Iterate the tx's instructions. Any instruction with `program_id == ComputeBudget111...` is a compute-budget instruction. Extract these separately — they must be preserved verbatim in the replay.

5. **Err field.** `meta.err` is `null` on success, or an `InstructionError` / `TransactionError` object on failure. Parse into a `TxResult` enum.

## Acceptance tests (run before marking done)

```bash
cargo test -p replay-core fetch
```

Then a smoke test with a real network call (gated behind `REPLAY_LIVE_TESTS=1`):
```bash
REPLAY_LIVE_TESTS=1 HELIUS_API_KEY=... cargo test -p replay-core -- --ignored live_
```

The live test fetches one specific Jupiter swap signature (put it in `tests/fixtures/demo-signature.txt`), resolves LUTs, and asserts the account count is > 10 (Jupiter swaps always touch many accounts).

## What NOT to do today

- No `litesvm` integration yet. That's tomorrow.
- No IDL decoding. That's Day 3.
- No CLI wiring. Day 11.
- No web UI thinking. Day 6.
- No premature abstraction. One `HeliusClient` trait, one impl, that's it.

## Ask me before you start

If any of these are unclear, ask:
- Do I want `reqwest` or `ureq`? (Default: `reqwest` with `rustls-tls`.)
- Retry logic for Helius 429? (Default: exponential backoff, max 3 retries, all in `rpc.rs`.)
- Which specific Helius endpoint flavor — "Enhanced Transactions" or standard `getTransaction`? (Default: standard `getTransaction` for now. Enhanced in a later day.)

## At end of day

Report:
1. Files changed.
2. `cargo test -p replay-core` output.
3. A CLI demo I can run: `cargo run -p replay-cli -- fetch <signature>` should print a JSON dump of the `TxContext` for any mainnet signature.
4. Specific things you noticed about Helius's response format that aren't in the spec and might bite us later.
