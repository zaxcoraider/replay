//! Fetch on-chain IDLs for the canonical programs and write them to
//! `crates/replay-core/assets/idls/<program_id>.json`. Intended as a
//! one-shot maintenance command — run by hand when the bundled set
//! needs refreshing.
//!
//! Usage:
//!     HELIUS_API_KEY=... cargo run -p replay-core --example bundle_idls
//! or, with .env at the workspace root:
//!     cargo run -p replay-core --example bundle_idls

use replay_core::{idl::IdlCache, HeliusRpcClient};
use solana_sdk::pubkey::Pubkey;
use std::path::PathBuf;
use std::str::FromStr;

const PROGRAMS: &[(&str, &str)] = &[
    ("jupiter_v6", "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"),
    ("whirlpool", "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
    ("drift_v2", "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH"),
    ("kamino_lending", "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env from the workspace root if present.
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let _ = dotenvy::from_path(workspace_root.join(".env"));

    let api_key = std::env::var("HELIUS_API_KEY")
        .map_err(|_| "HELIUS_API_KEY must be set (in shell or .env)")?;
    let client = HeliusRpcClient::from_api_key(&api_key)?;

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/idls");
    std::fs::create_dir_all(&assets_dir)?;
    // Empty bundled set so get_or_fetch always falls through to on-chain;
    // disk-write goes to assets_dir.
    let cache = IdlCache::new(assets_dir.clone());

    println!("Bundling IDLs into {}", assets_dir.display());
    let mut ok = 0usize;
    let mut none = 0usize;
    let mut err = 0usize;
    for (label, pid_str) in PROGRAMS {
        let pid = Pubkey::from_str(pid_str)?;
        match cache.get_or_fetch(&client, &pid).await {
            Ok(Some(idl)) => {
                println!(
                    "  ✓ {label:<16} {pid_str}  (source={:?}, {} top-level keys)",
                    idl.source,
                    idl.raw.as_object().map(|o| o.len()).unwrap_or(0)
                );
                ok += 1;
            }
            Ok(None) => {
                println!("  - {label:<16} {pid_str}  (no on-chain IDL)");
                none += 1;
            }
            Err(e) => {
                println!("  ✗ {label:<16} {pid_str}  — {e}");
                err += 1;
            }
        }
    }
    println!(
        "\nResult: {ok} bundled, {none} skipped (no IDL), {err} errored. \
         Files written to {}",
        assets_dir.display()
    );
    Ok(())
}
