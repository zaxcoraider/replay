//! Trace tree builder. Takes a flat log list + execution result and produces
//! a tree of CpiFrames, one per CPI invocation.
//!
//! Skeleton for Day 4. The hard part is a log-line state machine:
//!   "Program X invoke [N]" — push frame at depth N
//!   "Program X consumed K of M compute units" — record CU on the current frame
//!   "Program X success" / "Program X failed: ..." — pop frame

use crate::idl::AccountDecoder;
use crate::svm::ExecutionResult;
use crate::types::{AccountDelta, CpiFrame, FrameAccount, Trace, TxContext, TxResult};

pub async fn build_trace(
    ctx: &TxContext,
    execution: &ExecutionResult,
    _decoder: &AccountDecoder<'_>,
) -> Trace {
    let frames = parse_log_frames(&execution.logs, ctx);
    let account_deltas = build_deltas(ctx, execution);

    let log_divergence = find_first_divergence(&ctx.mainnet_logs, &execution.logs);

    Trace {
        signature: ctx.signature.to_string(),
        slot: ctx.slot,
        block_time: ctx.block_time,
        mainnet_result: ctx.mainnet_result.clone(),
        replay_result: execution.result.clone(),
        frames,
        account_deltas,
        total_cu: execution.cu_consumed,
        log_divergence,
    }
}

/// Parse log lines into a CPI tree. Day 4: full implementation.
/// For now: one flat frame per top-level instruction, no CPI nesting.
fn parse_log_frames(logs: &[String], ctx: &TxContext) -> Vec<CpiFrame> {
    let mut frames = Vec::new();
    let mut current_logs_by_depth: Vec<Vec<String>> = vec![Vec::new(); 8];
    let mut depth: u32 = 0;
    let mut current_program_stack: Vec<String> = Vec::new();

    for line in logs {
        if let Some(program_id) = parse_invoke_line(line) {
            depth += 1;
            current_program_stack.push(program_id);
            if depth as usize >= current_logs_by_depth.len() {
                current_logs_by_depth.push(Vec::new());
            }
            current_logs_by_depth[depth as usize].clear();
        } else if parse_success_or_failure(line).is_some() {
            // Pop frame
            if depth == 1 {
                if let Some(program_id) = current_program_stack.pop() {
                    frames.push(CpiFrame {
                        depth: 0,
                        program_id,
                        program_name: None,
                        instruction_index: frames.len(),
                        instruction_name: None,
                        accounts: Vec::new(),
                        data_hex: String::new(),
                        decoded_args: None,
                        logs: std::mem::take(&mut current_logs_by_depth[depth as usize]),
                        cu_consumed: parse_cu_consumed(line).unwrap_or(0),
                        cu_remaining_after: 0,
                        children: Vec::new(),
                        result: if line.contains("success") {
                            TxResult::Success
                        } else {
                            TxResult::Failure {
                                error: line.clone(),
                                error_code: None,
                            }
                        },
                        return_data: None,
                    });
                }
            } else {
                current_program_stack.pop();
            }
            if depth > 0 {
                depth -= 1;
            }
        } else {
            if (depth as usize) < current_logs_by_depth.len() {
                current_logs_by_depth[depth as usize].push(line.clone());
            }
        }
    }

    // Fill in accounts for top-level frames from the original tx.
    let instructions = ctx.original_tx.message.instructions();
    for (i, frame) in frames.iter_mut().enumerate() {
        if let Some(ci) = instructions.get(i) {
            frame.accounts = ci
                .accounts
                .iter()
                .filter_map(|idx| {
                    let pk = ctx.resolved_account_keys.get(*idx as usize)?;
                    Some(FrameAccount {
                        pubkey: pk.to_string(),
                        is_signer: ctx.original_tx.message.is_signer(*idx as usize),
                        is_writable: ctx
                            .original_tx
                            .message
                            .is_maybe_writable(*idx as usize, None),
                        role: None,
                    })
                })
                .collect();
            frame.data_hex = hex::encode(&ci.data);
        }
    }

    frames
}

fn parse_invoke_line(line: &str) -> Option<String> {
    // "Program <id> invoke [<depth>]"
    let rest = line.strip_prefix("Program ")?;
    let space = rest.find(' ')?;
    let program_id = &rest[..space];
    if rest[space..].contains("invoke [") {
        Some(program_id.to_string())
    } else {
        None
    }
}

fn parse_success_or_failure(line: &str) -> Option<bool> {
    let rest = line.strip_prefix("Program ")?;
    if rest.ends_with("success") {
        Some(true)
    } else if rest.contains("failed") {
        Some(false)
    } else {
        None
    }
}

fn parse_cu_consumed(line: &str) -> Option<u64> {
    // "Program <id> consumed <N> of <M> compute units" OR just success.
    let rest = line.strip_prefix("Program ")?;
    let consumed_idx = rest.find(" consumed ")?;
    let after = &rest[consumed_idx + " consumed ".len()..];
    let of_idx = after.find(" of ")?;
    after[..of_idx].parse::<u64>().ok()
}

fn build_deltas(ctx: &TxContext, execution: &ExecutionResult) -> Vec<AccountDelta> {
    let mut deltas = Vec::new();
    for pk in &ctx.resolved_account_keys {
        let before = ctx.pre_account_snapshots.get(pk);
        let after = execution.accounts_after.get(pk);

        let (Some(b), Some(a)) = (before, after) else { continue };

        if b.lamports != a.lamports || b.owner != a.owner || b.data != a.data {
            deltas.push(AccountDelta {
                pubkey: pk.to_string(),
                owner_before: b.owner.to_string(),
                owner_after: a.owner.to_string(),
                lamports_before: b.lamports,
                lamports_after: a.lamports,
                data_before_hex: hex::encode(&b.data[..b.data.len().min(256)]),
                data_after_hex: hex::encode(&a.data[..a.data.len().min(256)]),
                decoded_before: None,
                decoded_after: None,
                idl_type_name: None,
            });
        }
    }
    deltas
}

fn find_first_divergence(
    mainnet: &[String],
    replay: &[String],
) -> Option<crate::types::LogDivergence> {
    for (i, (m, r)) in mainnet.iter().zip(replay.iter()).enumerate() {
        if m != r {
            return Some(crate::types::LogDivergence {
                first_divergent_line: i,
                mainnet_line: m.clone(),
                replay_line: r.clone(),
                suspected_cause: suspect_cause(m, r),
            });
        }
    }
    if mainnet.len() != replay.len() {
        return Some(crate::types::LogDivergence {
            first_divergent_line: mainnet.len().min(replay.len()),
            mainnet_line: mainnet
                .get(mainnet.len().saturating_sub(1))
                .cloned()
                .unwrap_or_default(),
            replay_line: replay
                .get(replay.len().saturating_sub(1))
                .cloned()
                .unwrap_or_default(),
            suspected_cause: Some("log count mismatch".into()),
        });
    }
    None
}

fn suspect_cause(m: &str, r: &str) -> Option<String> {
    if m.contains("consumed") && r.contains("consumed") {
        Some("CU accounting drift — check sysvar setup".into())
    } else if m.contains("invoke") && !r.contains("invoke") {
        Some("missing program load — check program bytecode fetch".into())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_invoke_lines() {
        assert_eq!(
            parse_invoke_line("Program ABC invoke [1]"),
            Some("ABC".to_string())
        );
        assert_eq!(parse_invoke_line("Program log: hi"), None);
    }

    #[test]
    fn parses_cu_consumed() {
        let line = "Program ABC consumed 4123 of 200000 compute units";
        assert_eq!(parse_cu_consumed(line), Some(4123));
    }
}
