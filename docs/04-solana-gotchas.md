# 04 — Solana Gotchas (Read Before Coding)

You have a multi-chain brain. That's a strength for product thinking and a liability for Solana mechanics. This doc catalogs the traps you *will* hit if you're not warned.

## The big seven

### 1. Programs are upgradeable. Bytecode at slot N ≠ bytecode today.

**Trap:** You write `svm.add_program_from_file(program_id, current_bytes)` and assume it works. For replays older than a few weeks, the on-chain bytecode has almost certainly changed at least once.

**Fix:** For the upgradeable BPF loader (the common case), a "program" is actually two accounts:
- The **program account** at `program_id`, owned by `BPFLoaderUpgradeab1e11111111111111111111111`. Its data is 36 bytes pointing to...
- The **program-data account**, whose address is derivable from the program account's data. This is where the bytecode lives.

To replay at historical slot `S`:
```rust
// 1. Fetch program account at slot S-1
let program_account = rpc.get_account_info_at(&program_id, S - 1).await?;

// 2. Parse bytes 4..36 to get program_data_address
let program_data_address = Pubkey::try_from(&program_account.data[4..36])?;

// 3. Fetch program-data account at slot S-1
let program_data_account = rpc.get_account_info_at(&program_data_address, S - 1).await?;

// 4. The actual bytecode starts after the 45-byte UpgradeableLoaderState header
let bytecode = &program_data_account.data[45..];

// 5. Seed both into the SVM
svm.set_account(program_id, program_account)?;
svm.set_account(program_data_address, program_data_account)?;
```

Do **not** use `add_program_from_file` for replays. Use `set_account` on both accounts and let the SVM's loader wire them up.

### 2. Address Lookup Tables (LUTs) hide accounts from the raw tx.

**Trap:** You iterate over `tx.message.account_keys` and fetch those accounts. You miss half the accounts because modern txs use LUTs.

**Fix:** When calling `getTransaction`, set `maxSupportedTransactionVersion: 0`. The response includes `meta.loaded_addresses: { writable: [...], readonly: [...] }`. These are the LUT-resolved addresses. **Concatenate**: the final account list is `static_account_keys ++ loaded_writable ++ loaded_readonly`. Order matters — it determines what `AccountMeta` indices point to.

### 3. Compute budget instructions are consumed by the CB program; replaying without them hits the 200k-CU default.

**Trap:** The mainnet tx set CU limit to 1.4M via `ComputeBudgetProgram::set_compute_unit_limit(1_400_000)`. You rebuild the tx but drop the compute budget instructions. Your replay hits the 200k default and fails with `exceeded CUs meter at BPF instruction`.

**Fix:** Preserve compute budget instructions verbatim when rebuilding the tx. The litesvm runner respects them.

### 4. You don't have the fee payer's private key.

**Trap:** Transactions must be signed. You don't have the signer's key, so you can't submit.

**Fix:** Replace the fee payer. litesvm doesn't enforce signature validity the way a real validator does — or more precisely, you can swap the fee payer to a keypair you control:
```rust
let dummy_payer = Keypair::new();
svm.airdrop(&dummy_payer.pubkey(), 10_000_000_000)?;

// Rebuild the message with dummy_payer as the fee payer
// Keep all other signers as "signed" via set_account tricks
// (litesvm has a `.with_sigverify(false)` option — use it for replay)
let svm = LiteSVM::new().with_sigverify(false);
```

**Important:** `with_sigverify(false)` is *exactly* the kind of option you want for a replay tool. Don't try to forge signatures; disable verification.

### 5. Sysvars must be set, or programs panic.

**Trap:** Programs call `Clock::get()` and get a default value. `unix_timestamp = 0`. DEX programs fail their oracle staleness checks and revert.

**Fix:** Set the `Clock` sysvar to match the target slot:
```rust
use solana_sdk::clock::Clock;

let clock = Clock {
    slot: target_slot,
    epoch_start_timestamp: block_time,
    epoch: target_slot / 432_000, // rough
    leader_schedule_epoch: target_slot / 432_000,
    unix_timestamp: block_time, // from getTransaction.blockTime
};
svm.set_sysvar::<Clock>(&clock);
```

Also consider setting `Rent`, `SlotHashes`, and `RecentBlockhashes` — most programs tolerate defaults here but some protocols (e.g., anything using VRF or on-chain randomness) check `SlotHashes` directly.

### 6. Account data is opaque bytes. There are no storage slots.

**Trap:** You assume you can introspect state like `contract.balanceOf[user]`. You can't — state is a raw `Vec<u8>` whose meaning is defined by the owning program's serialization logic.

**Fix:** Your decoder MUST be IDL-driven. For Anchor programs, the first 8 bytes are a discriminator identifying the account type; the rest is Borsh-encoded. For SPL Token: use `spl_token::state::Account::unpack`. For non-Anchor native programs: bring your own decoder or fall back to hex view.

Design the UI around the assumption that sometimes you can't decode — "Paste IDL" must be a first-class affordance, not an afterthought.

### 7. Rent-exempt minimums. Lamports below threshold = account gets garbage-collected.

**Trap:** User mutates an account's lamports down to 100. At the end of a tx, the runtime collects it. Replay behavior looks weird.

**Fix:** In the mutator UI, warn when lamports would drop below `rent.minimum_balance(data.len())`. Don't block it — a dev might *want* to test this edge case — but flag it.

## Secondary gotchas

### 8. `jsonParsed` encoding is lossy.

`getTransaction` with `encoding: "jsonParsed"` is human-readable but drops fields. Use `encoding: "base64"` and parse the tx yourself with `solana_sdk::transaction::VersionedTransaction::deserialize` for faithful replay. Use `jsonParsed` only for UI display.

### 9. Recent blockhash must be set, but content is irrelevant.

litesvm gives you `svm.latest_blockhash()` for free. Use that when rebuilding the versioned message. Don't try to use the mainnet tx's original blockhash — it will have expired.

### 10. Token-2022 ≠ SPL Token.

Two different programs: `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` (classic) and `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` (2022). Different serialization (Token-2022 has extensions). Your decoder must check the owner program and route accordingly.

### 11. CPIs can depth-limit at 4.

Solana enforces a max CPI depth of 4. If your replay is working but hitting "cross-program invocation depth exceeded," that's not a bug — that's reality. Log it clearly in the trace.

### 12. The `slot` in `getAccountInfo` semantics.

`minContextSlot` in the RPC config means "I will accept this response only if the node has at least observed this slot." It does NOT give you the state *as of that slot* — it gives you the state *as of now, confirmed that the node knows about slot N*.

For true historical state, you need:
- **Helius "Enhanced Transactions" API** provides `tx.accountData[].accountData` — the account state at that tx's slot, for every account in the tx.
- **Helius LaserStream historical replay** — a proper time-machine endpoint.
- If neither works, fall back to this heuristic: fetch current state, then "rewind" by reversing subsequent state-changing txs. Complex and rarely necessary if Helius's enhanced endpoints work.

**Use `getTransaction` with the right options** — the returned `meta.preBalances`, `meta.preTokenBalances`, and (Helius-specific enhanced) `accountData` at pre-state is often enough for most replays. Start there.

### 13. Compute units are per-tx, not per-instruction, in the default meter.

The total CU budget is shared across all instructions in the tx. Your trace UI should visualize the CU "fuel gauge" depleting across instructions, not reset per-instruction.

### 14. Some programs write to their own account data. Program upgrades during execution are impossible but program-owned PDAs can change.

Not a bug, just a mental model correction coming from EVM: a program cannot modify its own bytecode during execution (that's what upgrades are for, and upgrades are separate txs). But a program can modify accounts it owns. Don't be confused when you see program-owned PDAs change during a tx — that's normal.

## The EVM-dev reflexes that will bite you

| EVM reflex | What you should do on Solana |
|---|---|
| "State is in the contract" | State is in *accounts owned by* the program. Accounts are passed explicitly. |
| "`msg.sender` is the caller" | Check the `signers` array. For CPIs, check the invoking program ID plus any PDA signer seeds. |
| "Immutable contracts by default" | Everything is upgradeable by default. Upgrade authority is its own threat model. |
| "Gas is linear in opcodes" | CU consumption is per-syscall, per-BPF-instruction, with weird nonlinearities around Borsh deserialization and sysvars. |
| "Reentrancy is the main attack" | Reentrancy exists but is CPI-depth-limited. Account-confusion (wrong account passed into an instruction) is a bigger class of bugs. |
| "Events are logged, gas-metered" | Logs are strings. `sol_log`/`msg!` are the primary emission. Anchor events use a specific log format. |
| "Storage slots pack" | Account data is a flat `Vec<u8>`. Layout is whatever Borsh emits. Padding is manual. |

Keep this file open in a tab while you code.
