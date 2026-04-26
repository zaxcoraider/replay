use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use litesvm::LiteSVM;
use reqwest::Client;
use serde_json::{json, Value};
use solana_account::Account;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_clock::Clock;
use solana_message::{v0::MessageAddressTableLookup, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_transaction::versioned::VersionedTransaction;
use std::{collections::HashSet, env, str::FromStr};

// BPFLoaderUpgradeab1e program ID
const UPGRADEABLE_LOADER: &str = "BPFLoaderUpgradeab1e11111111111111111111111";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        bail!(
            "Usage: {} <TRANSACTION_SIGNATURE>\n\
             Env:   HELIUS_API_KEY=<key>",
            args[0]
        );
    }
    let sig_str = &args[1];
    Signature::from_str(sig_str)
        .with_context(|| format!("'{}' is not a valid base58 signature", sig_str))?;

    let api_key = env::var("HELIUS_API_KEY")
        .context("HELIUS_API_KEY not set — export HELIUS_API_KEY=<your-helius-key>")?;

    let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);
    let http = Client::new();

    // ── 1. Fetch the transaction ──────────────────────────────────────────────
    eprintln!("[1/5] Fetching transaction {}...", sig_str);
    let tx_resp = rpc(
        &http,
        &rpc_url,
        "getTransaction",
        json!([
            sig_str,
            {
                "encoding": "base64",
                "maxSupportedTransactionVersion": 0,
                "commitment": "confirmed"
            }
        ]),
    )
    .await
    .context("getTransaction RPC call failed")?;

    if tx_resp.is_null() {
        bail!(
            "Transaction '{}' not found. \
             It may not be confirmed yet, or may be too old for the RPC endpoint.",
            sig_str
        );
    }

    let slot = tx_resp["slot"]
        .as_u64()
        .context("Transaction response is missing 'slot'")?;

    let block_time = tx_resp["blockTime"].as_i64().unwrap_or(0);

    let mainnet_logs: Vec<String> = tx_resp["meta"]["logMessages"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let tx_b64 = tx_resp["transaction"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .context("Transaction response is missing base64 transaction data")?;

    let tx_bytes = B64
        .decode(tx_b64)
        .context("Failed to base64-decode transaction bytes")?;

    let tx: VersionedTransaction = bincode::deserialize(&tx_bytes)
        .context("Failed to deserialize transaction with bincode")?;

    // ── 2. Collect every account key referenced in the transaction ────────────
    eprintln!("[2/5] Resolving account keys...");
    let mut needed: HashSet<Pubkey> = HashSet::new();

    let static_keys: Vec<Pubkey> = match &tx.message {
        VersionedMessage::Legacy(m) => m.account_keys.clone(),
        VersionedMessage::V0(m) => m.account_keys.clone(),
    };
    for k in &static_keys {
        needed.insert(*k);
    }

    // Resolve address lookup tables (V0 transactions only)
    let atl_lookups: Vec<MessageAddressTableLookup> = match &tx.message {
        VersionedMessage::V0(m) => m.address_table_lookups.clone(),
        _ => vec![],
    };

    let mut resolved_alts: Vec<(Pubkey, Vec<Pubkey>)> = Vec::new();

    for atl in &atl_lookups {
        eprintln!("  Fetching ALT {}...", atl.account_key);
        needed.insert(atl.account_key); // seed the ALT account itself

        let acc_resp = rpc(
            &http,
            &rpc_url,
            "getAccountInfo",
            json!([
                atl.account_key.to_string(),
                { "encoding": "base64", "commitment": "confirmed" }
            ]),
        )
        .await
        .with_context(|| format!("getAccountInfo for ALT {} failed", atl.account_key))?;

        let raw = decode_account_data(&acc_resp["value"])
            .with_context(|| format!("Failed to decode data for ALT {}", atl.account_key))?;

        let table = AddressLookupTable::deserialize(&raw)
            .with_context(|| format!("Failed to deserialize ALT {}", atl.account_key))?;

        let addresses: Vec<Pubkey> = table.addresses.iter().copied().collect();

        for &idx in atl
            .writable_indexes
            .iter()
            .chain(atl.readonly_indexes.iter())
        {
            match addresses.get(idx as usize) {
                Some(&addr) => {
                    needed.insert(addr);
                }
                None => bail!(
                    "ALT {} has {} entries but index {} was requested",
                    atl.account_key,
                    addresses.len(),
                    idx
                ),
            }
        }
        resolved_alts.push((atl.account_key, addresses));
    }

    eprintln!("  {} unique accounts to fetch.", needed.len());

    // ── 3. Fetch all accounts at slot-1 (pre-transaction state) ──────────────
    let pre_slot = slot.saturating_sub(1);
    eprintln!(
        "[3/5] Fetching {} accounts at slot {} (pre-tx state)...",
        needed.len(),
        pre_slot
    );

    let needed_vec: Vec<Pubkey> = needed.into_iter().collect();
    let mut account_map: Vec<(Pubkey, Account)> = Vec::new();

    for chunk in needed_vec.chunks(100) {
        let keys: Vec<String> = chunk.iter().map(|k| k.to_string()).collect();
        let resp = rpc(
            &http,
            &rpc_url,
            "getMultipleAccounts",
            json!([
                keys,
                {
                    "encoding": "base64",
                    "minContextSlot": pre_slot,
                    "commitment": "confirmed"
                }
            ]),
        )
        .await
        .context("getMultipleAccounts failed")?;

        let values = resp["value"]
            .as_array()
            .context("getMultipleAccounts: response missing 'value' array")?;

        for (key, val) in chunk.iter().zip(values.iter()) {
            if val.is_null() {
                eprintln!(
                    "  (account {} is null/not-found at slot {}, skipping)",
                    key, pre_slot
                );
                continue;
            }
            match parse_account(val) {
                Ok(acc) => account_map.push((*key, acc)),
                Err(e) => eprintln!("  Warning: failed to parse account {}: {:#}", key, e),
            }
        }
    }

    // ── 4. Fetch program-data accounts for upgradeable loader programs ─────────
    eprintln!("[4/5] Checking for upgradeable program-data accounts...");
    let upgradeable = Pubkey::from_str(UPGRADEABLE_LOADER).unwrap();
    let mut pd_keys: HashSet<Pubkey> = HashSet::new();

    for (_, acc) in &account_map {
        if acc.owner == upgradeable && acc.executable {
            if let Some(pd) = extract_program_data_address(&acc.data) {
                pd_keys.insert(pd);
            }
        }
    }

    if pd_keys.is_empty() {
        eprintln!("  No upgradeable programs found.");
    } else {
        eprintln!("  Fetching {} program-data accounts...", pd_keys.len());
        let pd_vec: Vec<Pubkey> = pd_keys.into_iter().collect();

        for chunk in pd_vec.chunks(100) {
            let keys: Vec<String> = chunk.iter().map(|k| k.to_string()).collect();
            let resp = rpc(
                &http,
                &rpc_url,
                "getMultipleAccounts",
                json!([
                    keys,
                    {
                        "encoding": "base64",
                        "minContextSlot": pre_slot,
                        "commitment": "confirmed"
                    }
                ]),
            )
            .await
            .context("getMultipleAccounts for program-data accounts failed")?;

            let values = resp["value"]
                .as_array()
                .context("program-data getMultipleAccounts: missing 'value' array")?;

            for (key, val) in chunk.iter().zip(values.iter()) {
                if val.is_null() {
                    eprintln!(
                        "  Warning: program-data account {} not found — replay may fail",
                        key
                    );
                    continue;
                }
                match parse_account(val) {
                    Ok(acc) => account_map.push((*key, acc)),
                    Err(e) => {
                        eprintln!("  Warning: failed to parse program-data {}: {:#}", key, e)
                    }
                }
            }
        }
    }

    // ── 5. Seed LiteSVM and replay ────────────────────────────────────────────
    eprintln!("[5/5] Seeding LiteSVM and replaying...");
    let mut svm = LiteSVM::new();

    // Set Clock sysvar to match the transaction's slot/epoch
    let epoch = slot / 432_000;
    svm.set_sysvar(&Clock {
        slot,
        epoch_start_timestamp: block_time,
        epoch,
        leader_schedule_epoch: epoch + 1,
        unix_timestamp: block_time,
    });

    let seeded = account_map.len();
    for (key, acc) in account_map {
        svm.set_account(key, acc)
            .with_context(|| format!("Failed to seed account {} into LiteSVM", key))?;
    }
    eprintln!("  Seeded {} accounts.", seeded);

    // Compute budget instructions are already embedded in the transaction — litesvm honours them.
    let result = svm.send_transaction(tx);

    // ── Print results ─────────────────────────────────────────────────────────
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  MAINNET LOGS  ({} lines)", mainnet_logs.len());
    println!("╚══════════════════════════════════════════════════════════════╝");
    for line in &mainnet_logs {
        println!("{}", line);
    }

    let (replay_logs, replay_ok, replay_err) = match &result {
        Ok(meta) => (meta.logs.clone(), true, None),
        Err(failed) => (
            failed.meta.logs.clone(),
            false,
            Some(format!("{}", failed.err)),
        ),
    };

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  REPLAY LOGS  ({} lines)", replay_logs.len());
    println!("╚══════════════════════════════════════════════════════════════╝");
    for line in &replay_logs {
        println!("{}", line);
    }

    if replay_ok {
        println!("\n[Replay] SUCCESS");
    } else {
        println!("\n[Replay] FAILED — {}", replay_err.unwrap());
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  LOG DIFF");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let max_lines = mainnet_logs.len().max(replay_logs.len());
    let mut diffs = 0usize;
    for i in 0..max_lines {
        let m = mainnet_logs.get(i).map(String::as_str).unwrap_or("<missing>");
        let r = replay_logs.get(i).map(String::as_str).unwrap_or("<missing>");
        if m != r {
            println!("Line {:>3}:", i + 1);
            println!("  - MAINNET : {}", m);
            println!("  + REPLAY  : {}", r);
            diffs += 1;
        }
    }

    if diffs == 0 {
        println!("Logs match exactly.");
    } else {
        println!("\n{} line(s) differ.", diffs);
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn rpc(http: &Client, url: &str, method: &str, params: Value) -> Result<Value> {
    let body = json!({
        "jsonrpc": "2.0",
        "id":      1,
        "method":  method,
        "params":  params,
    });
    let resp = http
        .post(url)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("HTTP POST failed for method '{}'", method))?;

    let json: Value = resp
        .json()
        .await
        .with_context(|| format!("Failed to parse JSON response for method '{}'", method))?;

    if let Some(err) = json.get("error") {
        bail!("{} RPC error: {}", method, err);
    }
    Ok(json["result"].clone())
}

fn decode_account_data(val: &Value) -> Result<Vec<u8>> {
    let b64str = val["data"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .context("account 'data' field is not a [base64, encoding] array")?;
    B64.decode(b64str).context("base64 decode of account data failed")
}

fn parse_account(val: &Value) -> Result<Account> {
    let data = decode_account_data(val)?;
    let lamports = val["lamports"].as_u64().context("account missing 'lamports'")?;
    let owner = Pubkey::from_str(val["owner"].as_str().context("account missing 'owner'")?)
        .context("account 'owner' is not a valid pubkey")?;
    let executable = val["executable"].as_bool().unwrap_or(false);
    let rent_epoch = val["rentEpoch"].as_u64().unwrap_or(u64::MAX);
    Ok(Account {
        lamports,
        data,
        owner,
        executable,
        rent_epoch,
    })
}

/// BPFLoaderUpgradeable `Program` variant discriminant = 2 (u32 LE).
/// Layout: [discriminant: 4 bytes][programdata_address: 32 bytes]
fn extract_program_data_address(data: &[u8]) -> Option<Pubkey> {
    if data.len() < 36 {
        return None;
    }
    let disc = u32::from_le_bytes(data[0..4].try_into().ok()?);
    if disc == 2 {
        let bytes: [u8; 32] = data[4..36].try_into().ok()?;
        Some(Pubkey::from(bytes))
    } else {
        None
    }
}
