//! `replay` CLI — time-travel debugger for Solana transactions.
//!
//! Subcommands:
//!   replay <signature>           One-shot replay; print trace
//!   replay fetch <signature>     Fetch tx + resolved state (no execution)
//!   replay inspect <sig> -a <pk> Decode one account at the tx's pre-slot
//!   replay serve                 Print how to start the API server
//!
//! Global flags: --rpc, --json, --verbose

use anyhow::Context;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "replay")]
#[command(version)]
#[command(about = "Time-travel debugger for Solana transactions")]
#[command(long_about = "\
Replay any Solana mainnet transaction against its exact historical account \
state. Fork the session, mutate fields, re-run, and inspect the diff.\n\n\
Requires HELIUS_API_KEY in the environment (or a .env file), or pass --rpc \
with a full RPC URL.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Override the RPC URL (defaults to Helius mainnet via HELIUS_API_KEY).
    #[arg(long, global = true, env = "REPLAY_RPC_URL")]
    rpc: Option<String>,

    /// Emit JSON instead of pretty output.
    #[arg(long, global = true)]
    json: bool,

    /// Verbose tracing output.
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Replay a transaction and print its trace.
    Replay {
        /// Base58 transaction signature.
        signature: String,

        /// Print mainnet vs replay logs side-by-side when they diverge.
        #[arg(long)]
        diff_logs: bool,
    },

    /// Fetch a transaction and its resolved account state without executing.
    Fetch {
        /// Base58 transaction signature.
        signature: String,
    },

    /// Decode and print one account's state at the transaction's pre-slot.
    Inspect {
        /// Base58 transaction signature (determines the slot).
        signature: String,

        /// Public key of the account to inspect.
        #[arg(short, long)]
        account: String,
    },

    /// Print instructions for starting the HTTP API server.
    Serve {
        /// Address to bind (used only in the printed example).
        #[arg(long, default_value = "0.0.0.0:8787")]
        bind: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    let filter = if cli.verbose { "debug" } else { "warn" };
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
            if cli.json {
                let sp = spinner("Replaying…");
                let trace = replay_core::replay(&signature, &client).await?;
                sp.finish_and_clear();
                println!("{}", serde_json::to_string_pretty(&trace)?);
                return Ok(());
            }

            let sig = signature
                .parse()
                .with_context(|| format!("'{signature}' is not a valid base58 signature"))?;

            // ── Fetch ────────────────────────────────────────────────────
            let sp = spinner("Fetching transaction from Helius…");
            let mut ctx = replay_core::fetch::fetch_full_tx_context(&client, &sig).await?;
            sp.finish_with_message(format!(
                "{} Fetched      slot={}  accounts={}  logs={}",
                "✓".green(),
                ctx.slot.to_string().bright_white(),
                ctx.resolved_account_keys.len().to_string().bright_white(),
                ctx.mainnet_logs.len().to_string().bright_white(),
            ));

            // ── Reconstruct ──────────────────────────────────────────────
            let sp = spinner("Reconstructing historical state…");
            let state = replay_core::reconstruct::reconstruct_state(&client, &ctx).await?;
            ctx.pre_account_snapshots = replay_core::reconstruct::snapshot_pre_state(&state);
            sp.finish_with_message(format!(
                "{} Reconstructed accounts={}  programs={}",
                "✓".green(),
                state.accounts.len().to_string().bright_white(),
                state.programs.len().to_string().bright_white(),
            ));

            // ── Execute ──────────────────────────────────────────────────
            let sp = spinner("Executing in LiteSVM…");
            let mut runner = replay_core::svm::SvmRunner::new();
            runner.seed(&state)?;
            runner.set_clock_for_slot(ctx.slot, ctx.block_time);
            let execution = runner.execute(&ctx)?;
            let result_label = match &execution.result {
                replay_core::TxResult::Success => "success".green().to_string(),
                replay_core::TxResult::Failure { error, .. } => {
                    format!("failed — {}", error.as_str().dimmed())
                }
            };
            sp.finish_with_message(format!(
                "{} Replayed     {}  CU={}",
                "✓".green(),
                result_label,
                format_cu(execution.cu_consumed).bright_white(),
            ));

            // ── Log match ────────────────────────────────────────────────
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
                    "{} Logs match   {}/{} lines identical",
                    "✓".green(),
                    matched.to_string().bright_white(),
                    total.to_string().bright_white(),
                );
            } else {
                println!(
                    "{} Logs diverge at line {} ({}/{} matched)",
                    "✗".red(),
                    (matched + 1).to_string().bright_white(),
                    matched.to_string().bright_white(),
                    total.to_string().bright_white(),
                );
            }

            // ── CPI frame table ──────────────────────────────────────────
            // Use the full trace to get the CPI tree.
            let sp = spinner("Building CPI trace…");
            let trace = replay_core::replay(&signature, &client).await?;
            sp.finish_and_clear();
            if !trace.frames.is_empty() {
                println!();
                println!("{}", "Programs invoked:".bold());
                println!(
                    "  {:<6}  {:<48}  {:>12}",
                    "depth".dimmed(),
                    "program".dimmed(),
                    "CU".dimmed(),
                );
                print_frames(&trace.frames, 0);
            }

            // ── Log diff ─────────────────────────────────────────────────
            if diff_logs && matched < total {
                println!();
                print_log_diff(mainnet, replay_logs);
            }
        }

        Commands::Fetch { signature } => {
            let sig = signature
                .parse()
                .with_context(|| format!("'{signature}' is not a valid base58 signature"))?;

            let sp = spinner("Fetching transaction…");
            let ctx = replay_core::fetch::fetch_full_tx_context(&client, &sig).await?;
            sp.finish_and_clear();

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
                println!("  [{:>3}] {}", i.to_string().dimmed(), k);
            }
        }

        Commands::Inspect { signature, account } => {
            let sig = signature
                .parse()
                .with_context(|| format!("'{signature}' is not a valid base58 signature"))?;
            let pk: solana_sdk::pubkey::Pubkey = account
                .parse()
                .with_context(|| format!("'{account}' is not a valid pubkey"))?;

            let sp = spinner("Fetching + reconstructing state…");
            let ctx = replay_core::fetch::fetch_full_tx_context(&client, &sig).await?;
            let state = replay_core::reconstruct::reconstruct_state(&client, &ctx).await?;
            sp.finish_and_clear();

            match state.accounts.get(&pk) {
                None => {
                    eprintln!(
                        "{} Account {} not found in reconstructed state",
                        "✗".red(),
                        pk
                    );
                    std::process::exit(1);
                }
                Some(acc) => {
                    if cli.json {
                        let v = serde_json::json!({
                            "pubkey": pk.to_string(),
                            "lamports": acc.lamports,
                            "owner": acc.owner.to_string(),
                            "executable": acc.executable,
                            "data_len": acc.data.len(),
                        });
                        println!("{}", serde_json::to_string_pretty(&v)?);
                    } else {
                        println!("{}", pk.to_string().bright_white().bold());
                        println!("  lamports   {}", format_cu(acc.lamports));
                        println!("  owner      {}", acc.owner.to_string().dimmed());
                        println!("  executable {}", acc.executable);
                        println!("  data       {} bytes", acc.data.len());
                        if !acc.data.is_empty() {
                            let preview = acc
                                .data
                                .iter()
                                .take(32)
                                .map(|b| format!("{:02x}", b))
                                .collect::<Vec<_>>()
                                .join(" ");
                            println!("  hex[0..32] {}", preview.dimmed());
                        }
                    }
                }
            }
        }

        Commands::Serve { bind } => {
            println!("{}", "To start the API server:".bold());
            println!(
                "  REPLAY_BIND_ADDR={} cargo run -p replay-api --release",
                bind
            );
        }
    }

    Ok(())
}

fn build_client(rpc: Option<&str>) -> anyhow::Result<replay_core::HeliusRpcClient> {
    match rpc {
        Some(url) => Ok(replay_core::HeliusRpcClient::from_url(url)?),
        None => {
            let key = std::env::var("HELIUS_API_KEY")
                .context("set HELIUS_API_KEY in .env or pass --rpc <url>")?;
            Ok(replay_core::HeliusRpcClient::from_api_key(&key)?)
        }
    }
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

fn print_frames(frames: &[replay_core::CpiFrame], _indent: usize) {
    for frame in frames {
        let prog = frame
            .program_name
            .as_deref()
            .unwrap_or("unknown");
        let short_pid = abbrev(&frame.program_id);
        let ix_name = frame
            .instruction_name
            .as_deref()
            .map(|n| format!("  {}", n.dimmed()))
            .unwrap_or_default();
        println!(
            "  [{:<2}]  {:<20} {}{}  {}",
            frame.depth,
            prog.bold(),
            short_pid.dimmed(),
            ix_name,
            format_cu(frame.cu_consumed).bright_white(),
        );
        if !frame.children.is_empty() {
            print_frames(&frame.children, _indent + 1);
        }
    }
}

fn abbrev(s: &str) -> String {
    if s.len() > 10 {
        format!("{}…{}", &s[..4], &s[s.len() - 4..])
    } else {
        s.to_string()
    }
}

fn print_log_diff(mainnet: &[String], replay: &[String]) {
    println!("{}", "Log diff (mainnet vs replay):".bold());
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

fn format_cu(n: u64) -> String {
    let s = n.to_string();
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in b.iter().enumerate() {
        if i > 0 && (b.len() - i) % 3 == 0 {
            out.push('_');
        }
        out.push(*ch as char);
    }
    out
}
