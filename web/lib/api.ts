import type { Trace, TraceDiff, ApiError } from "./types";

const BASE = "/rpc";

async function parseResponse<T>(res: Response): Promise<T> {
  const text = await res.text();
  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    throw new Error(`API unreachable (${res.status}): ${text.slice(0, 120)}`);
  }
  if (!res.ok) throw new Error((json as ApiError).error?.message ?? res.statusText);
  return json as T;
}

async function post<T>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  return parseResponse<T>(res);
}

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`);
  return parseResponse<T>(res);
}

export async function replay(signature: string): Promise<Trace> {
  const r = await post<{ trace: Trace }>("/replay", { signature });
  return r.trace;
}

export async function fork(
  signature: string
): Promise<{ session_id: string; baseline_trace: Trace; expires_at: string }> {
  return post("/fork", { signature });
}

export interface MutationLamports { type: "lamports"; new_value: number }
export interface MutationOwner    { type: "owner";    new_value: string }
export interface MutationRawBytes { type: "raw_bytes"; offset: number; bytes: string; extend?: boolean }
export interface MutationField    { type: "field";    path: string; new_value: unknown }
export type Mutation = MutationLamports | MutationOwner | MutationRawBytes | MutationField;

export async function mutate(
  sessionId: string,
  pubkey: string,
  mutation: Mutation
): Promise<{ applied: boolean; mutation_count: number }> {
  return post(`/session/${sessionId}/mutate`, { pubkey, mutation });
}

export async function execute(
  sessionId: string
): Promise<{ trace: Trace; mutation_count: number }> {
  return post(`/session/${sessionId}/execute`);
}

export async function diff(sessionId: string): Promise<TraceDiff> {
  return get(`/session/${sessionId}/diff`);
}
