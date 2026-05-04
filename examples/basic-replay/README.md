# Example: Basic Replay

Fetch a mainnet transaction, replay it in a local LiteSVM sandbox, and print the CPI trace.

## Run

```bash
# From the repo root
export HELIUS_API_KEY=your_key_here
cargo run -p replay-cli -- replay 5xYour...Signature
```

Or using the Rust SDK directly:

```bash
cd examples/basic-replay
HELIUS_API_KEY=your_key cargo run
```

## What it shows

- How to construct a `ReplayClient` from environment variables
- How to call `client.replay(signature)` and inspect the returned `Trace`
- How to print the CPI frame tree (program name, CU used, decoded args)

## Expected output

```
Replaying 5xYour...Signature
  slot:        259481234
  result:      Ok
  total CU:    48291
  mainnet CU:  48291
  CU delta:    0

CPI trace:
  [0] Jupiter Aggregator V6  (34291 CU)
    [1] Token Program: transfer  (912 CU)
    [2] Token Program: transfer  (912 CU)
    [3] Whirlpool: swap  (12176 CU)
```
