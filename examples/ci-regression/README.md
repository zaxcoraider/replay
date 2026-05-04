# Example: CI Regression

Replay a list of historical transaction signatures and fail if any produce a different result than mainnet. Drop this into your CI pipeline to catch Solana program regressions before they hit prod.

## Run

```bash
cd examples/ci-regression
HELIUS_API_KEY=your_key cargo run
```

Or via the TypeScript SDK:

```bash
cd examples/ci-regression
npm install
HELIUS_API_KEY=your_key npx ts-node index.ts
```

## What it shows

- How to load a list of signatures from a text file
- How to use `replayHistorical` from `@zaxcoraider/replay-sdk/testing`
- How to structure a CI assertion that fails the build on regression

## Fixtures

`fixtures/signatures.txt` contains one signature per line. Replace these with real signatures from your protocol's history.
