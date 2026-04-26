//! Shared data types.
//!
//! These are the contract between `replay-core`, `replay-api`, and the UI.
//! Keep serde-derived. Versioned changes go through deprecation.

use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, pubkey::Pubkey, transaction::VersionedTransaction};
use std::collections::HashMap;

/// Top-level replay outcome. Renders in the UI as the main trace view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub mainnet_result: TxResult,
    pub replay_result: TxResult,
    pub frames: Vec<CpiFrame>,
    pub account_deltas: Vec<AccountDelta>,
    pub total_cu: u64,
    pub log_divergence: Option<LogDivergence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TxResult {
    Success,
    Failure {
        error: String,
        error_code: Option<i64>,
    },
}

/// One node in the CPI tree. Top-level frames have `depth: 0`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpiFrame {
    pub depth: u32,
    pub program_id: String,
    pub program_name: Option<String>,
    pub instruction_index: usize,
    pub instruction_name: Option<String>,
    pub accounts: Vec<FrameAccount>,
    pub data_hex: String,
    pub decoded_args: Option<serde_json::Value>,
    pub logs: Vec<String>,
    pub cu_consumed: u64,
    pub cu_remaining_after: u64,
    pub children: Vec<CpiFrame>,
    pub result: TxResult,
    pub return_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameAccount {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDelta {
    pub pubkey: String,
    pub owner_before: String,
    pub owner_after: String,
    pub lamports_before: u64,
    pub lamports_after: u64,
    pub data_before_hex: String,
    pub data_after_hex: String,
    pub decoded_before: Option<serde_json::Value>,
    pub decoded_after: Option<serde_json::Value>,
    pub idl_type_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDivergence {
    pub first_divergent_line: usize,
    pub mainnet_line: String,
    pub replay_line: String,
    pub suspected_cause: Option<String>,
}

/// Output of [`crate::fetch::fetch_full_tx_context`]. Everything you need
/// to reconstruct state and re-execute.
#[derive(Debug, Clone, Serialize)]
pub struct TxContext {
    pub signature: solana_sdk::signature::Signature,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub original_tx: VersionedTransaction,
    pub resolved_account_keys: Vec<Pubkey>,
    pub mainnet_logs: Vec<String>,
    pub mainnet_result: TxResult,
    pub compute_budget_instructions: Vec<solana_sdk::instruction::CompiledInstruction>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    #[serde(serialize_with = "serialize_account_snapshots")]
    pub pre_account_snapshots: HashMap<Pubkey, Account>,
}

fn serialize_account_snapshots<S: serde::Serializer>(
    map: &HashMap<Pubkey, Account>,
    s: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    let mut m = s.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        m.serialize_entry(&k.to_string(), &SerializableAccount::from(v))?;
    }
    m.end()
}

#[derive(Serialize)]
struct SerializableAccount<'a> {
    lamports: u64,
    #[serde(with = "base64_bytes")]
    data: &'a [u8],
    owner: String,
    executable: bool,
    rent_epoch: u64,
}

impl<'a> From<&'a Account> for SerializableAccount<'a> {
    fn from(a: &'a Account) -> Self {
        Self {
            lamports: a.lamports,
            data: &a.data,
            owner: a.owner.to_string(),
            executable: a.executable,
            rent_epoch: a.rent_epoch,
        }
    }
}

mod base64_bytes {
    use base64::Engine;
    use serde::Serializer;

    pub fn serialize<S: Serializer>(bytes: &&[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
    }
}

/// Thin DTO around the raw Helius `getTransaction` response. Lives here
/// so `rpc.rs` and `fetch.rs` can share the shape.
#[derive(Debug, Clone, Deserialize)]
pub struct FetchedTx {
    pub slot: u64,
    pub block_time: Option<i64>,
    pub transaction_base64: String,
    pub meta: FetchedTxMeta,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchedTxMeta {
    pub err: Option<serde_json::Value>,
    pub log_messages: Vec<String>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub loaded_addresses: Option<LoadedAddresses>,
    pub compute_units_consumed: Option<u64>,
    pub inner_instructions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoadedAddresses {
    pub writable: Vec<String>,
    pub readonly: Vec<String>,
}

/// Output of [`crate::reconstruct::reconstruct_state`]. Ready to be
/// fed into [`crate::svm::SvmRunner::seed`].
#[derive(Debug, Clone)]
pub struct ReconstructedState {
    pub accounts: HashMap<Pubkey, Account>,
    pub programs: HashMap<Pubkey, ProgramInfo>,
}

#[derive(Debug, Clone)]
pub struct ProgramInfo {
    pub program_account: Account,
    pub program_data_address: Option<Pubkey>,
    pub program_data_account: Option<Account>,
    pub loader: ProgramLoader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramLoader {
    Native,
    BpfLoader,
    BpfLoaderDeprecated,
    BpfLoaderUpgradeable,
    LoaderV4,
}

/// A mutation applied to an account within a `ForkedSession`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountMutation {
    Field {
        path: String,
        new_value: serde_json::Value,
    },
    RawBytes {
        offset: usize,
        #[serde(with = "hex_bytes")]
        bytes: Vec<u8>,
        #[serde(default)]
        extend: bool,
    },
    Lamports {
        new_value: u64,
    },
    Owner {
        new_value: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDiff {
    pub baseline: Trace,
    pub latest: Trace,
    pub result_changed: bool,
    pub total_cu_delta: i64,
    pub changed_accounts: Vec<String>,
}

// --- helpers ---

mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        hex::decode(s.trim_start_matches("0x")).map_err(serde::de::Error::custom)
    }
}
