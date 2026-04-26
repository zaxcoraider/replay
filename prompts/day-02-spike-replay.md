# Day 2 — Spike Part 2: Reconstruct state and replay

## This is the go/no-go gate for the project.

By end of today: paste a mainnet signature, reconstruct pre-slot state of every account, load all programs at their historical bytecode, replay in litesvm, and get logs that match mainnet exactly.

**If this doesn't work by end of day, stop the project and pivot to the CU Profiler fallback.**

## Goal

```bash
cargo run -p replay-cli -- replay <SIGNATURE>
```

Output:
```
✓ Fetched tx from Helius (slot 271234567)
✓ Reconstructed state: 23 accounts, 5 programs
✓ Loaded into LiteSVM
✓ Replayed successfully
✓ Logs match mainnet: 47/47 lines identical
Total CU consumed: 842_119
```

With a `--diff-logs` flag, print mainnet and replay logs side-by-side when divergent.

## Deliverables

1. `crates/replay-core/src/reconstruct.rs`: `StateReconstructor`:
   ```rust
   pub async fn reconstruct_state<C: HeliusClient>(
       client: &C,
       ctx: &TxContext,
   ) -> Result<ReconstructedState, ReplayError>;
   
   pub struct ReconstructedState {
       pub accounts: HashMap<Pubkey, Account>,
       pub programs: HashMap<Pubkey, ProgramInfo>,
   }
   
   pub struct ProgramInfo {
       pub program_account: Account,
       pub program_data_address: Option<Pubkey>,  // None for non-upgradeable
       pub program_data_account: Option<Account>,
       pub loader: ProgramLoader,  // enum: Native, BpfLoader, BpfLoaderUpgradeable, LoaderV4
   }
   ```

2. `crates/replay-core/src/svm.rs`: `SvmRunner`:
   ```rust
   pub struct SvmRunner {
       svm: LiteSVM,
   }
   
   impl SvmRunner {
       pub fn new() -> Self;
       pub fn seed(&mut self, state: &ReconstructedState) -> Result<(), ReplayError>;
       pub fn set_clock_for_slot(&mut self, slot: u64, block_time: Option<i64>);
       pub fn execute(&mut self, ctx: &TxContext) -> Result<ExecutionResult, ReplayError>;
   }
   
   pub struct ExecutionResult {
       pub logs: Vec<String>,
       pub result: TxResult,
       pub cu_consumed: u64,
       pub accounts_after: HashMap<Pubkey, Account>,
   }
   ```

3. `crates/replay-core/src/lib.rs`: top-level `replay` function that wires fetch → reconstruct → seed → execute and returns a minimal `Trace` (full trace tree comes Day 4).

4. `crates/replay-cli/src/main.rs`: `replay <signature> [--diff-logs]` subcommand.

## Critical implementation details

### Fetching program bytecode at historical slot

For every `program_id` that appears in the tx's instructions:
1. Call `get_account_info_at_slot(&program_id, ctx.slot - 1)`. This gives you the program account.
2. Inspect `account.owner` to determine the loader:
   - `NativeLoader1111111111111111111111111111111` → native (e.g., System, ComputeBudget). Skip — litesvm has these built-in.
   - `BPFLoader1111111111111111111111111111111111` or `BPFLoader2111111111111111111111111111111111` → legacy loader. Bytecode is in the program account's data directly.
   - `BPFLoaderUpgradeab1e11111111111111111111111` → upgradeable. Parse `program_account.data[4..36]` as the program-data address. Fetch the program-data account at `ctx.slot - 1`. Bytecode starts at byte 45 of program-data's data.
   - `LoaderV411111111111111111111111111111111111` → loader-v4. Bytecode is in the program account after a header. Check the current loader-v4 spec when you hit this; it's newer.

### Seeding litesvm

```rust
let mut svm = LiteSVM::new()
    .with_sigverify(false)  // critical: we don't have real signers
    .with_blockhash_check(false); // we can't reproduce a valid blockhash

for (pubkey, account) in &state.accounts {
    svm.set_account(*pubkey, account.clone())
        .map_err(|e| ReplayError::Execution(format!("set_account {pubkey}: {e}")))?;
}

for (program_id, info) in &state.programs {
    // Set program account
    svm.set_account(*program_id, info.program_account.clone())?;
    // Set program-data account if upgradeable
    if let (Some(pda), Some(data_acc)) = (info.program_data_address, &info.program_data_account) {
        svm.set_account(pda, data_acc.clone())?;
    }
}
```

### Setting the Clock sysvar

```rust
use solana_sdk::clock::Clock;

let clock = Clock {
    slot: ctx.slot,
    epoch_start_timestamp: ctx.block_time.unwrap_or(0),
    epoch: ctx.slot / 432_000,
    leader_schedule_epoch: ctx.slot / 432_000,
    unix_timestamp: ctx.block_time.unwrap_or(0),
};
svm.set_sysvar::<Clock>(&clock);
```

### Rebuilding the transaction with a dummy fee payer

You don't have the real signer. Steps:
1. Generate a dummy keypair.
2. Airdrop it 10 SOL in the SVM.
3. Rebuild the `Message` with `dummy.pubkey()` as fee payer, **but preserve the original instructions verbatim including the original signers in AccountMeta** (they remain in the account list but won't actually sign because `with_sigverify(false)`).
4. Sign the rebuilt transaction with only the dummy keypair.

There's a subtlety: the original fee payer account (index 0 in the original message) is a real account the tx touched. Make sure it's still in the account list — just no longer at index 0.

**Easier alternative:** use litesvm's lower-level message-processing APIs if they exist. Check `litesvm::LiteSVM::simulate_transaction` vs `send_transaction`. Simulate is closer to what we want.

### Comparing logs

Mainnet logs live in `meta.log_messages`. Replay logs come from the SVM result. Compare line-by-line. The first divergent line is almost always informative:

- If mainnet says `Program X invoke [1]` and replay says nothing → program not loaded.
- If both say invoke but CU numbers differ wildly → sysvar divergence (usually Clock).
- If mainnet says `...success` and replay says `custom program error: 0x...` → account state divergence or missing LUT-resolved account.

## Test checklist

Before calling this done, replay these three signatures successfully (put them in `tests/fixtures/canonical-sigs.txt`):

1. A simple SOL transfer (SystemProgram only). ← must work first.
2. An SPL Token transfer. ← second milestone.
3. A Jupiter v6 swap. ← this is the real gate.

Each must produce replay logs that match mainnet logs exactly (ignoring any clock-dependent output — note any deviations in comments).

## What NOT to do today

- No IDL decoding. Raw bytes and raw logs only.
- No HTTP API. CLI only.
- No WebSocket streaming.
- No UI.
- No fork/mutate logic.
- No account decoder UI.

## The pivot

If by 10pm your Jupiter swap replay doesn't produce matching logs and you can't identify why:

1. Commit your work.
2. `git checkout -b cu-profiler-fallback`.
3. Open `docs/FALLBACK-cu-profiler.md` (write it — describe the alternative project scope).
4. Keep `replay-core::rpc` (still useful) and throw the rest away.
5. The fallback project: a cargo subcommand that instruments `sol_log_compute_units()` at CPI boundaries in an Anchor program and produces a flamegraph. Scope it for Days 3–18.

Do not romance this project into day 4. Pivot is a feature.

## At end of day

Report:
1. Which of the 3 canonical signatures replay with matching logs.
2. Any divergences and their root cause.
3. Total round-trip time for a Jupiter swap replay (target: <10 seconds cold cache).
4. Any litesvm API quirks you found that will affect later days.
