# Day 12 — TypeScript SDK

## Goal

`npm install @replay/sdk` and programmatically replay any tx from Node or the browser.

## Deliverables

1. `packages/replay-sdk-ts/` — new pnpm workspace package (update root `pnpm-workspace.yaml`).

2. API:
   ```ts
   import { ReplayClient } from '@replay/sdk';
   
   const client = new ReplayClient({
     apiUrl: 'https://api.replay.dev',    // or self-hosted
     // OR for direct-to-Helius without the API server (v2?):
     // rpcUrl: 'https://mainnet.helius-rpc.com/?api-key=...',
   });
   
   // One-shot
   const trace = await client.replay('5xY...');
   
   // Sessioned
   const session = await client.fork('5xY...');
   await session.mutate(pubkey, { type: 'field', path: 'config.fee_bps', new_value: 9999 });
   const newTrace = await session.execute();
   const diff = await session.diff();
   session.close();
   ```

3. Regression test helper (the CI sell):
   ```ts
   import { replayHistorical } from '@replay/sdk/testing';
   
   test('program upgrade does not break historical swaps', async () => {
     const results = await replayHistorical({
       signatures: await loadSignatures('./fixtures/historical-swaps.txt'),
       programOverride: {
         programId: 'JUP6...',
         bytecodePath: './target/deploy/jupiter_v6.so',
       },
     });
     
     expect(results.failures).toEqual([]);
     expect(results.cuRegressions).toEqual([]);
   });
   ```

4. Full TypeScript types matching the Rust types in `replay-core`. Generate from an OpenAPI spec if feasible, or hand-write — this is only ~15 types.

5. Publish:
   - `pnpm build` bundles ESM + CJS + .d.ts using `tsup`.
   - `pnpm publish --access public` to npm as `@replay/sdk`.
   - README with 3 usage examples.

## Browser support

- The ESM bundle must work in Next.js / Vite / webpack without Node-only deps.
- Use `fetch` (native in modern Node + browsers), not `axios`.

## What NOT to do

- No CLI in this package (that's the Rust one).
- No on-device SVM (browser can't run it realistically for v1). SDK talks to API only.

## End-of-day

- Package published on npm.
- Example in README works when copy-pasted.
- Add a "Using the SDK" section to the web docs.
