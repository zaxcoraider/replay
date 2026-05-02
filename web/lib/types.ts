// Mirror of replay-core types — keep in sync with crates/replay-core/src/types.rs

export type TxResult =
  | { status: "success" }
  | { status: "failure"; error: string; error_code?: number | null };

export interface FrameAccount {
  pubkey: string;
  is_signer: boolean;
  is_writable: boolean;
  role?: string | null;
}

export interface CpiFrame {
  depth: number;
  program_id: string;
  program_name?: string | null;
  instruction_index: number;
  instruction_name?: string | null;
  accounts: FrameAccount[];
  data_hex: string;
  decoded_args?: Record<string, unknown> | null;
  logs: string[];
  cu_consumed: number;
  cu_remaining_after: number;
  children: CpiFrame[];
  result: TxResult;
  return_data?: string | null;
}

export interface AccountDelta {
  pubkey: string;
  owner_before: string;
  owner_after: string;
  lamports_before: number;
  lamports_after: number;
  data_before_hex: string;
  data_after_hex: string;
  decoded_before?: unknown;
  decoded_after?: unknown;
  idl_type_name?: string | null;
}

export interface LogDivergence {
  first_divergent_line: number;
  mainnet_line: string;
  replay_line: string;
  suspected_cause?: string | null;
}

export interface Trace {
  signature: string;
  slot: number;
  block_time?: number | null;
  mainnet_result: TxResult;
  replay_result: TxResult;
  frames: CpiFrame[];
  account_deltas: AccountDelta[];
  total_cu: number;
  log_divergence?: LogDivergence | null;
}

export interface TraceDiff {
  baseline: Trace;
  latest: Trace;
  result_changed: boolean;
  total_cu_delta: number;
  changed_accounts: string[];
}

export interface ApiError {
  error: { code: string; message: string };
}
