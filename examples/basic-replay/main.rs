/// Minimal example: replay a single mainnet transaction and print the CPI trace.
///
/// Usage:
///   HELIUS_API_KEY=<key> REPLAY_SIG=<signature> cargo run
use replay_sdk::{ReplayClient, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let sig = std::env::var("REPLAY_SIG")
        .unwrap_or_else(|_| "5xYourSignatureHere".to_string());

    let client = ReplayClient::from_env()?;

    println!("Replaying {sig}");
    let trace = client.replay(&sig).await?;

    println!("  slot:       {}", trace.slot);
    println!("  result:     {:?}", trace.result);
    println!("  total CU:   {}", trace.total_cu);
    println!("  mainnet CU: {}", trace.mainnet_cu.unwrap_or(0));

    println!("\nCPI trace:");
    for frame in &trace.frames {
        let indent = "  ".repeat(frame.depth + 1);
        let name = frame.program_name.as_deref().unwrap_or(&frame.program_id);
        println!("{indent}[{}] {name}  ({} CU)", frame.depth, frame.cu_used);
    }

    Ok(())
}
