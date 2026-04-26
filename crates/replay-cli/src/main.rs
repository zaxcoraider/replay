//! `replay` CLI.
//!
//! Subcommands:
//!   replay <signature>       One-shot replay; print trace
//!   replay fetch <signature> Just fetch, don't execute (debugging)
//!   replay serve             Run the API server locally
//!
//! Day 11 extends this with `fork` (REPL mode) and `diff` (cross-replay).

use anyhow::Context;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

#[derive(Parser)]
#[command(name = "replay")]
#[command(version)]
#[command(about = "Time-travel debugger for Solana transactions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Override the RPC URL (defaults to Helius mainnet via HELIUS_API_KEY).
    #[arg(long, global = true, env = "REPLAY_RPC_URL")]
    rpc: Option<String>,

    /// Emit JSON instead of pretty output.
    #[arg(long, global = true)]
    json: bool,

    /// Verbose tracing.
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Replay a transaction.
    Replay {
        signature: String,

        /// Print mainnet vs replay logs side-by-side when divergent.
        #[arg(long)]
        diff_logs: bool,
    },

    /// Fetch a tx + resolved state; print without executing. For debugging.
    Fetch { signature: String },

    /// Start the HTTP API server.
    Serve {
        #[arg(long, default_value = "0.0.0.0:8787")]
        bind: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .with_writer(std::io::stderr)
        .init();

    let client = build_client(cli.rpc.as_deref())?;

    match cli.command {
        Commands::Replay { signature, diff_logs } => {
            // JSON path: keep using the one-shot replay() for SDK parity.
            if cli.json {
                let trace = replay_core::replay(&signature, &client).await?;
                println!("{}", serde_json::to_string_pretty(&trace)?);
                return Ok(());
            }

            // Pretty path: orchestrate step-by-step so we can stream
            // ✓-prefixed progress lines as each phase completes.
            let sig = signature
                .parse()
                .with_context(|| format!("'{signature}' is not a valid base58 signature"))?;

            let ctx = replay_core::fetch::fetch_full_tx_context(&client, &sig).await?;
            println!(
                "{} Fetched tx from Helius (slot {})",
                "✓".green(),
                ctx.slot
            );

            let state = replay_core::reconstruct::reconstruct_state(&client, &ctx).await?;
            println!(
                "{} Reconstructed state: {} accounts, {} programs",
                "✓".green(),
                state.accounts.len(),
                state.programs.len()
            );

            let mut runner = replay_core::svm::SvmRunner::new();
            runner.seed(&state)?;
            runner.set_clock_for_slot(ctx.slot, ctx.block_time);
            println!("{} Loaded into LiteSVM", "✓".green());

            let execution = runner.execute(&ctx)?;
            match &execution.result {
                replay_core::TxResult::Success => {
                    println!("{} Replayed successfully", "✓".green());
                }
                replay_core::TxResult::Failure { error, .. } => {
                    println!("{} Replay failed — {}", "✓".yellow(), error.dimmed());
                }
            }

            // Compare logs
            let mainnet = &ctx.mainnet_logs;
            let replay_logs = &execution.logs;
            let matched = mainnet
                .iter()
                .zip(replay_logs.iter())
                .take_while(|(m, r)| m == r)
                .count();
            let total = mainnet.len().max(replay_logs.len());
            if matched == mainnet.len() && mainnet.len() == replay_logs.len() {
                println!(
                    "{} Logs match mainnet: {}/{} lines identical",
                    "✓".green(),
                    matched,
                    total
                );
            } else {
                println!(
                    "{} Logs diverge at line {} ({}/{} matched)",
                    "✗".red(),
                    matched + 1,
                    matched,
                    total
                );
            }

            println!("Total CU consumed: {}", format_underscores(execution.cu_consumed));

            if diff_logs && matched < total {
                print_log_diff(mainnet, replay_logs);
            }
        }

        Commands::Fetch { signature } => {
            let sig = signature
                .parse()
                .with_context(|| format!("'{signature}' is not a valid base58 signature"))?;
            let ctx = replay_core::fetch::fetch_full_tx_context(&client, &sig).await?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&ctx)?);
                return Ok(());
            }

            println!(
                "{}  slot={}  accounts={}  logs={}  cb_ix={}",
                ctx.signature.to_string().bright_white(),
                ctx.slot,
                ctx.resolved_account_keys.len(),
                ctx.mainnet_logs.len(),
                ctx.compute_budget_instructions.len(),
            );
            for (i, k) in ctx.resolved_account_keys.iter().enumerate() {
                println!("  [{:>3}] {}", i.dimmed(), k);
            }
        }

        Commands::Serve { bind } => {
            println!("Serving requires the replay-api binary; run:");
            println!("  REPLAY_BIND_ADDR={bind} cargo run -p replay-api");
        }
    }

    Ok(())
}

fn build_client(rpc: Option<&str>) -> anyhow::Result<replay_core::HeliusRpcClient> {
    let client = match rpc {
        Some(url) => replay_core::HeliusRpcClient::from_url(url)?,
        None => {
            let key = std::env::var("HELIUS_API_KEY")
                .context("set HELIUS_API_KEY or pass --rpc")?;
            replay_core::HeliusRpcClient::from_api_key(&key)?
        }
    };
    Ok(client)
}

fn print_log_diff(mainnet: &[String], replay: &[String]) {
    println!();
    println!("{}", "Log diff (mainnet vs replay)".bold());
    let max = mainnet.len().max(replay.len());
    for i in 0..max {
        let m = mainnet.get(i).map(String::as_str).unwrap_or("<missing>");
        let r = replay.get(i).map(String::as_str).unwrap_or("<missing>");
        if m == r {
            println!("  {:>3} {} {}", i + 1, "=".dimmed(), m.dimmed());
        } else {
            println!("  {:>3} {} {}", i + 1, "-".red(), m.red());
            println!("      {} {}", "+".green(), r.green());
        }
    }
}

/// Format a u64 with underscore thousands separators: 842119 -> "842_119".
fn format_underscores(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push('_');
        }
        out.push(*b as char);
    }
    out
}
