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
            let trace = replay_core::replay(&signature, &client).await?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&trace)?);
                return Ok(());
            }

            print_trace_pretty(&trace, diff_logs);
        }

        Commands::Fetch { signature } => {
            let sig = signature.parse()?;
            let ctx = replay_core::fetch::fetch_full_tx_context(&client, &sig).await?;
            println!(
                "{}  slot={}  accounts={}  logs={}",
                ctx.signature.to_string().bright_white(),
                ctx.slot,
                ctx.resolved_account_keys.len(),
                ctx.mainnet_logs.len()
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

fn print_trace_pretty(trace: &replay_core::Trace, diff_logs: bool) {
    println!(
        "{}  {}  slot={}",
        "Replay".bright_green().bold(),
        trace.signature.bright_white(),
        trace.slot
    );

    println!("  mainnet: {:?}", trace.mainnet_result);
    println!("  replay:  {:?}", trace.replay_result);
    println!("  total CU: {}", trace.total_cu);
    println!();

    for (i, frame) in trace.frames.iter().enumerate() {
        let status = match &frame.result {
            replay_core::TxResult::Success => "✓".green().to_string(),
            replay_core::TxResult::Failure { .. } => "✗".red().to_string(),
        };
        println!(
            "  [{}] {} {}  {} CU",
            i.dimmed(),
            status,
            frame.program_id.bright_cyan(),
            frame.cu_consumed
        );
    }

    if diff_logs {
        if let Some(div) = &trace.log_divergence {
            println!();
            println!("{}", "Log divergence detected".red().bold());
            println!("  first diff at line {}", div.first_divergent_line);
            println!("    mainnet:  {}", div.mainnet_line.yellow());
            println!("    replay:   {}", div.replay_line.yellow());
            if let Some(cause) = &div.suspected_cause {
                println!("    suspect:  {}", cause.dimmed());
            }
        } else {
            println!();
            println!("{}", "Logs match mainnet ✓".green());
        }
    }
}
