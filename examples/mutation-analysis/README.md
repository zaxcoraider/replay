# Example: Mutation Analysis

Fork a transaction, mutate an account field, re-run, and print the diff. Useful for understanding how a protocol responds to changed state — fee rate changes, liquidity shifts, oracle price moves.

## Run

```bash
cd examples/mutation-analysis
HELIUS_API_KEY=your_key REPLAY_SIG=<signature> PUBKEY=<account> cargo run
```

## What it shows

- How to fork a replayed transaction into a mutable session
- How to mutate a specific field on an account using IDL-decoded field paths
- How to compare the original and mutated execution results
- How to read `TraceDiff`: result changed, CU delta, account deltas
