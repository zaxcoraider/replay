/// CI regression runner: replay historical signatures and fail on any mismatch.
///
/// Usage:
///   HELIUS_API_KEY=<key> REPLAY_API_URL=https://replay-y4wq.onrender.com npx ts-node index.ts
import { replayHistorical, loadSignatures } from '@zaxcoraider/replay-sdk/testing';
import * as path from 'path';

async function main() {
  const apiUrl = process.env.REPLAY_API_URL ?? 'http://localhost:8787';
  const sigFile = path.join(__dirname, 'fixtures', 'signatures.txt');

  console.log(`Running historical replay regression against ${apiUrl}`);

  const report = await replayHistorical({
    apiUrl,
    signatures: await loadSignatures(sigFile),
    concurrency: 3,
  });

  const total = report.results.length;
  const passed = report.results.filter(r => r.ok).length;
  const failed = report.failures.length;

  console.log(`\nResults: ${passed}/${total} passed`);

  for (const f of report.failures) {
    console.error(`  FAIL ${f.signature}: ${f.error}`);
  }

  if (failed > 0) {
    console.error(`\n${failed} regression(s) detected — see above`);
    process.exit(1);
  }

  console.log('All signatures replayed successfully.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
