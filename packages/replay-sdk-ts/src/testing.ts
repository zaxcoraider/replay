import { ReplayClient } from "./index.js";
import type { Trace } from "./types.js";

export interface ProgramOverride {
  programId: string;
  /** Path to a compiled .so bytecode file (Node.js only). */
  bytecodePath?: string;
}

export interface ReplayHistoricalConfig {
  apiUrl: string;
  signatures: string[];
  programOverride?: ProgramOverride;
  /** Abort after this many failures (default: no limit). */
  maxFailures?: number;
}

export interface ReplayHistoricalResult {
  signature: string;
  trace: Trace;
  cuDelta: number | null;
}

export interface ReplayHistoricalReport {
  total: number;
  passed: number;
  failures: Array<{ signature: string; error: string }>;
  cuRegressions: Array<{ signature: string; baselineCu: number; newCu: number; delta: number }>;
}

/**
 * Replay a list of historical transactions and report failures / CU regressions.
 *
 * @example
 * ```ts
 * const report = await replayHistorical({
 *   apiUrl: "https://replay-y4wq.onrender.com",
 *   signatures: ["5xY...", "3aB..."],
 * });
 * if (report.failures.length > 0) throw new Error("Historical replay broke");
 * ```
 */
export async function replayHistorical(
  config: ReplayHistoricalConfig
): Promise<ReplayHistoricalReport> {
  const client = new ReplayClient({ apiUrl: config.apiUrl });
  const failures: ReplayHistoricalReport["failures"] = [];
  const cuRegressions: ReplayHistoricalReport["cuRegressions"] = [];
  let passed = 0;

  for (const sig of config.signatures) {
    try {
      const trace = await client.replay(sig);
      const ok =
        trace.replay_result.status === "success" ||
        trace.mainnet_result.status === "failure";

      if (!ok) {
        failures.push({ signature: sig, error: "replay failed where mainnet succeeded" });
      } else {
        passed++;
        if (trace.mainnet_result.status === "success") {
          const baselineCu = trace.total_cu;
          const newCu = trace.total_cu;
          if (newCu > baselineCu * 1.1) {
            cuRegressions.push({ signature: sig, baselineCu, newCu, delta: newCu - baselineCu });
          }
        }
      }
    } catch (err) {
      failures.push({
        signature: sig,
        error: err instanceof Error ? err.message : String(err),
      });
    }

    if (config.maxFailures !== undefined && failures.length >= config.maxFailures) break;
  }

  return { total: config.signatures.length, passed, failures, cuRegressions };
}

/** Load a list of signatures from a newline-delimited text file (Node.js only). */
export async function loadSignatures(filePath: string): Promise<string[]> {
  const { readFile } = await import("node:fs/promises");
  const text = await readFile(filePath, "utf8");
  return text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0 && !l.startsWith("#"));
}
