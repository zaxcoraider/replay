export type {
  Trace, TraceDiff, CpiFrame, FrameAccount, AccountDelta, FieldDelta,
  LogDivergence, LogLineDiff, TxResult, Mutation,
  ReplayClientOptions, ForkResult, ExecuteResult, MutateResult,
} from "./types.js";

import type {
  Trace, TraceDiff, Mutation,
  ReplayClientOptions, ForkResult, ExecuteResult, MutateResult,
} from "./types.js";

export class ReplayClient {
  private readonly apiUrl: string;

  constructor(options: ReplayClientOptions) {
    this.apiUrl = options.apiUrl.replace(/\/$/, "");
  }

  /** One-shot replay — fetch, reconstruct, execute, return trace. */
  async replay(signature: string): Promise<Trace> {
    const r = await this.post<{ trace: Trace }>("/replay", { signature });
    return r.trace;
  }

  /**
   * Fork a transaction into a mutable session.
   * The returned `Session` lets you mutate accounts, re-run, and diff.
   */
  async fork(signature: string): Promise<Session> {
    const r = await this.post<ForkResult>("/fork", { signature });
    return new Session(this.apiUrl, r.session_id, r.baseline_trace);
  }

  private async post<T>(path: string, body?: unknown): Promise<T> {
    return apiPost(this.apiUrl, path, body);
  }
}

export class Session {
  readonly sessionId: string;
  readonly baselineTrace: Trace;
  private readonly apiUrl: string;

  constructor(apiUrl: string, sessionId: string, baselineTrace: Trace) {
    this.apiUrl = apiUrl;
    this.sessionId = sessionId;
    this.baselineTrace = baselineTrace;
  }

  /** Apply a mutation to an account in this session. */
  async mutate(pubkey: string, mutation: Mutation): Promise<MutateResult> {
    return apiPost(this.apiUrl, `/session/${this.sessionId}/mutate`, {
      pubkey,
      mutation,
    });
  }

  /** Re-run the transaction with all applied mutations. */
  async execute(): Promise<ExecuteResult> {
    return apiPost(this.apiUrl, `/session/${this.sessionId}/execute`);
  }

  /** Diff the baseline trace against the latest execution. */
  async diff(): Promise<TraceDiff> {
    const res = await fetch(`${this.apiUrl}/session/${this.sessionId}/diff`);
    return parseResponse<TraceDiff>(res);
  }
}

async function apiPost<T>(
  base: string,
  path: string,
  body?: unknown
): Promise<T> {
  const res = await fetch(`${base}${path}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  return parseResponse<T>(res);
}

async function parseResponse<T>(res: Response): Promise<T> {
  const text = await res.text();
  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    throw new Error(`Replay API error (${res.status}): ${text.slice(0, 200)}`);
  }
  if (!res.ok) {
    const msg =
      (json as { error?: { message?: string } })?.error?.message ??
      res.statusText;
    throw new Error(`Replay API error (${res.status}): ${msg}`);
  }
  return json as T;
}
