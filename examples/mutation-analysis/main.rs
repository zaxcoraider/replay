/// Fork a transaction, mutate an account field, re-run, and print the diff.
///
/// Usage:
///   HELIUS_API_KEY=<key> REPLAY_SIG=<sig> MUTATE_PUBKEY=<pubkey> FIELD=<path> VALUE=<json> cargo run
use replay_sdk::{ReplayClient, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let sig = std::env::var("REPLAY_SIG")
        .unwrap_or_else(|_| "5xYourSignatureHere".to_string());
    let pubkey = std::env::var("MUTATE_PUBKEY")
        .unwrap_or_else(|_| "YourPubkeyHere".to_string());
    let field = std::env::var("FIELD").unwrap_or_else(|_| "feeRate".to_string());
    let value: serde_json::Value = serde_json::from_str(
        &std::env::var("VALUE").unwrap_or_else(|_| "9999".to_string()),
    )
    .expect("VALUE must be valid JSON");

    let client = ReplayClient::from_env()?;

    println!("Forking {sig}");
    let mut session = client.fork(&sig).await?;

    println!("Mutating {pubkey} → {field} = {value}");
    session.mutate_field(pubkey.parse().unwrap(), &field, value)?;

    println!("Re-running...");
    let new_trace = session.execute().await?;

    let diff = session.diff().expect("diff available after execute");

    println!("\n--- diff ---");
    println!("result changed:  {}", diff.result_changed);
    println!("CU delta:        {:+}", diff.cu_delta);
    println!("accounts changed: {}", diff.account_diffs.len());

    for ad in &diff.account_diffs {
        println!("  {} — {} field(s) changed", ad.pubkey, ad.field_diffs.len());
        for fd in &ad.field_diffs {
            println!("    {}: {:?} → {:?}", fd.path, fd.before, fd.after);
        }
    }

    println!("\nnew result: {:?}", new_trace.result);
    println!("new CU:     {}", new_trace.total_cu);

    Ok(())
}
