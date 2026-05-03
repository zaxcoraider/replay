// Types mirror the Rust serde JSON output from replay-core.

export type TxResult =
  | { status: "success" }
  | { status: "failure"; error: string; error_code: number | null };

export interface CpiFrame {
  depth: number;
  program_id: string;
  program_name: string | null;
  instruction_index: number;
  instruction_name: string | null;
  accounts: FrameAccount[];
  data_hex: string;
  decoded_args: unknown | null;
  logs: string[];
  cu_consumed: number;
  cu_remaining_after: number;
  children: CpiFrame[];
  result: TxResult;
  return_data: string | null;
}

export interface FrameAccount {
  pubkey: string;
  label: string | null;
  is_signer: boolean;
  is_writable: boolean;
}

export interface AccountDelta {
  pubkey: string;
  label: string | null;
  lamports_before: number;
  lamports_after: number;
  data_len_before: number;
  data_len_after: number;
  owner_before: string;
  owner_after: string;
  fields_changed: FieldDelta[];
}

export interface FieldDelta {
  path: string;
  before: unknown;
  after: unknown;
}

export interface LogDivergence {
  first_divergent_line: number;
  mainnet_line: string | null;
  replay_line: string | null;
}

export interface Trace {
  signature: string;
  slot: number;
  block_time: number | null;
  mainnet_result: TxResult;
  replay_result: TxResult;
  frames: CpiFrame[];
  account_deltas: AccountDelta[];
  total_cu: number;
  log_divergence: LogDivergence | null;
}

export interface TraceDiff {
  baseline_cu: number;
  forked_cu: number;
  cu_delta: number;
  baseline_result: TxResult;
  forked_result: TxResult;
  result_changed: boolean;
  account_deltas: AccountDelta[];
  changed_accounts: string[];
  log_diff: LogLineDiff[];
}

export interface LogLineDiff {
  line: number;
  baseline: string | null;
  forked: string | null;
}

export type Mutation =
  | { type: "lamports"; new_value: number }
  | { type: "owner"; new_value: string }
  | { type: "raw_bytes"; offset: number; bytes: string; extend?: boolean }
  | { type: "field"; path: string; new_value: unknown };

export interface ReplayClientOptions {
  apiUrl: string;
}

export interface ForkResult {
  session_id: string;
  baseline_trace: Trace;
  expires_at: string;
}

export interface ExecuteResult {
  trace: Trace;
  mutation_count: number;
}

export interface MutateResult {
  applied: boolean;
  mutation_count: number;
}
