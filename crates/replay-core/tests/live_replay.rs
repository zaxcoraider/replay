//! Live integration test: replay each canonical signature against mainnet
//! and assert replay logs match mainnet logs.
//!
//! Gated on REPLAY_LIVE_TESTS=1 + HELIUS_API_KEY. Skips silently otherwise
//! so `cargo test` stays hermetic.
//!
//! Reads canonical sigs from tests/fixtures/canonical-sigs.txt — one
//! `<label>=<sig>` per line; PLACEHOLDER_* entries are skipped.

use replay_core::{replay, HeliusRpcClient, TxResult};

#[tokio::test]
#[ignore]
async fn live_replay_canonical_sigs() {
    if std::env::var("REPLAY_LIVE_TESTS").is_err() {
        eprintln!("REPLAY_LIVE_TESTS not set; skipping");
        return;
    }
    let api_key =
        std::env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY must be set for live tests");

    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/canonical-sigs.txt");
    let raw = std::fs::read_to_string(&fixture_path)
        .expect("tests/fixtures/canonical-sigs.txt missing");

    let entries: Vec<(String, String)> = raw
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (label, sig) = line.split_once('=')?;
            let sig = sig.trim();
            if sig.starts_with("PLACEHOLDER") {
                eprintln!("  skipping {label}: placeholder");
                return None;
            }
            Some((label.trim().to_string(), sig.to_string()))
        })
        .collect();

    if entries.is_empty() {
        eprintln!(
            "no real signatures in canonical-sigs.txt yet; \
             populate it before enabling this test"
        );
        return;
    }

    let client = HeliusRpcClient::from_api_key(&api_key).expect("rpc client");

    let mut failures: Vec<String> = Vec::new();
    for (label, sig) in &entries {
        eprintln!("→ replay {label} ({sig})");
        match replay(sig, &client).await {
            Ok(trace) => {
                let outcome_match = match (&trace.mainnet_result, &trace.replay_result) {
                    (TxResult::Success, TxResult::Success) => true,
                    (TxResult::Failure { .. }, TxResult::Failure { .. }) => true,
                    _ => false,
                };
                if !outcome_match {
                    failures.push(format!(
                        "{label}: outcome mismatch — mainnet={:?} replay={:?}",
                        trace.mainnet_result, trace.replay_result
                    ));
                    continue;
                }
                if let Some(div) = &trace.log_divergence {
                    failures.push(format!(
                        "{label}: log divergence at line {} — mainnet={:?} replay={:?}",
                        div.first_divergent_line, div.mainnet_line, div.replay_line
                    ));
                } else {
                    eprintln!("  ✓ {label}: logs match, total_cu={}", trace.total_cu);
                }
            }
            Err(e) => {
                failures.push(format!("{label}: replay errored — {e}"));
            }
        }
    }

    if !failures.is_empty() {
        panic!("\n  - {}\n", failures.join("\n  - "));
    }
}
