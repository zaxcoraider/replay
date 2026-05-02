//! Trace tree builder. Takes a flat log list + execution result and produces
//! a tree of CpiFrames, one per CPI invocation, nested according to call depth.
//!
//! Log parsing is a line-based state machine:
//!   "Program X invoke [N]"                 → push frame at depth N
//!   "Program X consumed K of M ..."        → annotate CU on the matching frame
//!   "Program X success"                     → pop + attach to parent (or roots)
//!   "Program X failed: <reason>"            → same, with Failure result
//!   "Program return: X <b64>"               → capture return_data on frame
//!   "Program failed to complete: ..."       → fail top-of-stack (CU exhaustion)
//!   everything else                         → add to current frame's log list

use crate::idl::AccountDecoder;
use crate::svm::ExecutionResult;
use crate::types::{AccountDelta, CpiFrame, FrameAccount, Trace, TxContext, TxResult};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub async fn build_trace(
    ctx: &TxContext,
    execution: &ExecutionResult,
    decoder: &AccountDecoder<'_>,
) -> Trace {
    let mut frames = parse_log_frames(&execution.logs, ctx);
    populate_instruction_details(&mut frames, decoder);
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

// ---------- CPI tree state machine ----------

fn parse_log_frames(logs: &[String], ctx: &TxContext) -> Vec<CpiFrame> {
    let mut stack: Vec<CpiFrame> = Vec::new();
    let mut roots: Vec<CpiFrame> = Vec::new();
    let mut top_level_index: usize = 0;

    for line in logs {
        if let Some((program_id, invoke_depth)) = parse_invoke(line) {
            stack.push(CpiFrame {
                depth: invoke_depth - 1, // 1-based log depth → 0-based frame depth
                program_id,
                program_name: None,
                instruction_index: top_level_index,
                instruction_name: None,
                accounts: Vec::new(),
                data_hex: String::new(),
                decoded_args: None,
                logs: Vec::new(),
                cu_consumed: 0,
                cu_remaining_after: 0,
                children: Vec::new(),
                result: TxResult::Success,
                return_data: None,
            });
        } else if let Some((program_id, cu_used, cu_remaining)) = parse_consumed(line) {
            // Find the innermost matching frame — in well-formed logs it's always the top.
            for frame in stack.iter_mut().rev() {
                if frame.program_id == program_id {
                    frame.cu_consumed = cu_used;
                    frame.cu_remaining_after = cu_remaining;
                    break;
                }
            }
        } else if let Some((program_id, success, err_msg)) = parse_outcome(line) {
            // Pop the topmost matching frame. `rposition` handles any out-of-order edge
            // case, though in practice the logs are always properly nested.
            if let Some(pos) = stack.iter().rposition(|f| f.program_id == program_id) {
                let mut frame = stack.remove(pos);
                frame.result = if success {
                    TxResult::Success
                } else {
                    TxResult::Failure {
                        error: err_msg.unwrap_or_else(|| "unknown error".into()),
                        error_code: None,
                    }
                };
                if frame.depth == 0 {
                    top_level_index += 1;
                }
                if stack.is_empty() {
                    roots.push(frame);
                } else {
                    stack.last_mut().unwrap().children.push(frame);
                }
            }
        } else if let Some((prog_id, b64)) = parse_return_data(line) {
            for frame in stack.iter_mut().rev() {
                if frame.program_id == prog_id {
                    frame.return_data = Some(b64);
                    break;
                }
            }
        } else if line.starts_with("Program failed to complete") {
            // Compute-budget exhaustion — the validator logs this without a program ID.
            // Mark the top-of-stack frame as failed.
            if let Some(frame) = stack.last_mut() {
                frame.result = TxResult::Failure {
                    error: line.clone(),
                    error_code: None,
                };
            }
        } else {
            // Program log / Program data / anything else → attach to current frame.
            if let Some(frame) = stack.last_mut() {
                // Anchor emits "Program log: Instruction: <name>" — extract eagerly.
                if frame.instruction_name.is_none() {
                    if let Some(name) = parse_anchor_log_instruction(line) {
                        frame.instruction_name = Some(name);
                    }
                }
                frame.logs.push(line.clone());
            }
        }
    }

    // Drain unclosed frames (truncated / malformed logs). Each frame becomes
    // a child of the frame below it on the stack.
    while let Some(frame) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(frame);
        } else {
            roots.push(frame);
        }
    }

    // Fill top-level frame accounts + raw data from the original transaction message.
    let instructions = ctx.original_tx.message.instructions();
    for frame in &mut roots {
        let idx = frame.instruction_index;
        if let Some(ci) = instructions.get(idx) {
            frame.data_hex = hex::encode(&ci.data);
            frame.accounts = ci
                .accounts
                .iter()
                .filter_map(|&acc_idx| {
                    let pk = ctx.resolved_account_keys.get(acc_idx as usize)?;
                    Some(FrameAccount {
                        pubkey: pk.to_string(),
                        is_signer: ctx.original_tx.message.is_signer(acc_idx as usize),
                        is_writable: ctx
                            .original_tx
                            .message
                            .is_maybe_writable(acc_idx as usize, None),
                        role: None,
                    })
                })
                .collect();
        }
    }

    roots
}

/// Walk the frame tree and use bundled/disk-cached IDLs to populate:
///   - `instruction_name` (IDL discriminator match; overrides log-derived name)
///   - `decoded_args` (Borsh-decoded instruction arguments)
///   - `FrameAccount.role` (positional account name from IDL)
///
/// Only frames with `data_hex` set (top-level, populated above) are attempted.
/// CPI children inherit log-derived names from "Program log: Instruction:…" parsing.
fn populate_instruction_details(frames: &mut [CpiFrame], decoder: &AccountDecoder<'_>) {
    for frame in frames.iter_mut() {
        if !frame.data_hex.is_empty() {
            if let (Ok(data), Ok(pid)) = (
                hex::decode(&frame.data_hex),
                Pubkey::from_str(&frame.program_id),
            ) {
                if let Some((name, args, roles)) = decoder.decode_instruction_local(&pid, &data) {
                    frame.instruction_name = Some(name);
                    if !args.is_null() {
                        frame.decoded_args = Some(args);
                    }
                    for (i, acc) in frame.accounts.iter_mut().enumerate() {
                        if acc.role.is_none() {
                            if let Some(role) = roles.get(i) {
                                acc.role = Some(role.clone());
                            }
                        }
                    }
                }
            }
        }
        populate_instruction_details(&mut frame.children, decoder);
    }
}

// ---------- log line parsers ----------

/// "Program <id> invoke [<depth>]" → (program_id, depth)
fn parse_invoke(line: &str) -> Option<(String, u32)> {
    let rest = line.strip_prefix("Program ")?;
    let space = rest.find(' ')?;
    let pid = &rest[..space];
    let after = &rest[space + 1..];
    let inv = after.find("invoke [")?;
    let start = inv + "invoke [".len();
    let end = start + after[start..].find(']')?;
    let depth = after[start..end].parse::<u32>().ok()?;
    Some((pid.to_string(), depth))
}

/// "Program <id> consumed <N> of <M> compute units"
/// Returns (program_id, cu_used, cu_remaining_after)
fn parse_consumed(line: &str) -> Option<(String, u64, u64)> {
    let rest = line.strip_prefix("Program ")?;
    let consumed_off = rest.find(" consumed ")?;
    let pid = &rest[..consumed_off];
    let tail = &rest[consumed_off + " consumed ".len()..];
    let of_off = tail.find(" of ")?;
    let cu_used = tail[..of_off].parse::<u64>().ok()?;
    let after_of = tail[of_off + " of ".len()..].trim_end_matches(" compute units");
    let cu_budget = after_of.parse::<u64>().ok()?;
    Some((pid.to_string(), cu_used, cu_budget.saturating_sub(cu_used)))
}

/// "Program <id> success" | "Program <id> failed: <reason>"
/// Returns (program_id, is_success, error_message)
fn parse_outcome(line: &str) -> Option<(String, bool, Option<String>)> {
    let rest = line.strip_prefix("Program ")?;
    let space = rest.find(' ')?;
    let pid = &rest[..space];
    let after = &rest[space + 1..];
    if after == "success" {
        return Some((pid.to_string(), true, None));
    }
    if let Some(err) = after.strip_prefix("failed: ") {
        return Some((pid.to_string(), false, Some(err.to_string())));
    }
    None
}

/// "Program return: <id> <base64>" → (program_id, base64_payload)
fn parse_return_data(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("Program return: ")?;
    let space = rest.find(' ')?;
    Some((rest[..space].to_string(), rest[space + 1..].to_string()))
}

/// "Program log: Instruction: <name>" → name
fn parse_anchor_log_instruction(line: &str) -> Option<String> {
    let rest = line.strip_prefix("Program log: Instruction: ")?;
    Some(rest.trim().to_string())
}

// ---------- account delta builder ----------

fn build_deltas(ctx: &TxContext, execution: &ExecutionResult) -> Vec<AccountDelta> {
    let mut deltas = Vec::new();
    for pk in &ctx.resolved_account_keys {
        let before = ctx.pre_account_snapshots.get(pk);
        let after = execution.accounts_after.get(pk);

        let (Some(b), Some(a)) = (before, after) else {
            continue;
        };

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

// ---------- log divergence ----------

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

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_invoke_extracts_depth() {
        assert_eq!(
            parse_invoke("Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 invoke [1]"),
            Some((
                "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".into(),
                1
            ))
        );
        assert_eq!(
            parse_invoke("Program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc invoke [2]"),
            Some((
                "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".into(),
                2
            ))
        );
        assert_eq!(parse_invoke("Program log: Instruction: Swap"), None);
    }

    #[test]
    fn parse_consumed_extracts_cu_and_remaining() {
        let (pid, used, remaining) = parse_consumed(
            "Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 consumed 87234 of 1400000 compute units",
        )
        .unwrap();
        assert_eq!(pid, "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
        assert_eq!(used, 87_234);
        assert_eq!(remaining, 1_400_000 - 87_234);
    }

    #[test]
    fn parse_outcome_success_and_failure() {
        let (pid, ok, err) = parse_outcome("Program ABC123 success").unwrap();
        assert_eq!(pid, "ABC123");
        assert!(ok);
        assert!(err.is_none());

        let (pid2, ok2, err2) =
            parse_outcome("Program ABC123 failed: custom program error: 0x1").unwrap();
        assert_eq!(pid2, "ABC123");
        assert!(!ok2);
        assert_eq!(err2.unwrap(), "custom program error: 0x1");

        assert_eq!(parse_outcome("Program log: hi"), None);
    }

    #[test]
    fn parse_return_data_captures_b64() {
        let (pid, b64) =
            parse_return_data("Program ABC123 return: ABC123 aGVsbG8=").unwrap_or_default();
        // "Program return: <id> <b64>" format
        let (pid2, b64_2) =
            parse_return_data("Program return: ABC123 aGVsbG8=").unwrap();
        assert_eq!(pid2, "ABC123");
        assert_eq!(b64_2, "aGVsbG8=");
        let _ = (pid, b64);
    }

    #[test]
    fn anchor_log_instruction_name_extracted() {
        assert_eq!(
            parse_anchor_log_instruction("Program log: Instruction: Route"),
            Some("Route".into())
        );
        assert_eq!(parse_anchor_log_instruction("Program log: some other log"), None);
    }

    /// Core tree-building test: a two-level CPI (Jupiter → Whirlpool).
    #[test]
    fn builds_nested_cpi_tree() {
        let jup = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
        let whirl = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
        let logs: Vec<String> = vec![
            format!("Program {jup} invoke [1]"),
            "Program log: Instruction: Route".into(),
            format!("Program {whirl} invoke [2]"),
            "Program log: Instruction: Swap".into(),
            format!("Program {whirl} consumed 45123 of 200000 compute units"),
            format!("Program {whirl} success"),
            format!("Program {jup} consumed 87234 of 1400000 compute units"),
            format!("Program {jup} success"),
        ];

        let ctx = minimal_ctx();
        let frames = parse_log_frames(&logs, &ctx);

        assert_eq!(frames.len(), 1, "one top-level frame");
        let jup_frame = &frames[0];
        assert_eq!(jup_frame.program_id, jup);
        assert_eq!(jup_frame.depth, 0);
        assert_eq!(jup_frame.cu_consumed, 87_234);
        assert_eq!(jup_frame.cu_remaining_after, 1_400_000 - 87_234);
        assert_eq!(jup_frame.instruction_name.as_deref(), Some("Route"));
        assert!(matches!(jup_frame.result, TxResult::Success));

        assert_eq!(jup_frame.children.len(), 1, "one CPI child");
        let whirl_frame = &jup_frame.children[0];
        assert_eq!(whirl_frame.program_id, whirl);
        assert_eq!(whirl_frame.depth, 1);
        assert_eq!(whirl_frame.cu_consumed, 45_123);
        assert_eq!(whirl_frame.instruction_name.as_deref(), Some("Swap"));
    }

    /// ComputeBudget frames are legitimate depth-1 frames with no CU consumption.
    #[test]
    fn handles_compute_budget_frame() {
        let cb = "ComputeBudget111111111111111111111111111111";
        let prog = "SomeProgram1111111111111111111111111111111111";
        let logs: Vec<String> = vec![
            format!("Program {cb} invoke [1]"),
            format!("Program {cb} success"),
            format!("Program {prog} invoke [1]"),
            format!("Program {prog} consumed 10000 of 200000 compute units"),
            format!("Program {prog} success"),
        ];

        let ctx = minimal_ctx();
        let frames = parse_log_frames(&logs, &ctx);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].program_id, cb);
        assert_eq!(frames[0].cu_consumed, 0);
        assert_eq!(frames[0].instruction_index, 0);
        assert_eq!(frames[1].instruction_index, 1);
    }

    #[test]
    fn failed_frame_propagates_error() {
        let prog = "SomeProgram1111111111111111111111111111111111";
        let logs: Vec<String> = vec![
            format!("Program {prog} invoke [1]"),
            format!("Program {prog} consumed 5000 of 200000 compute units"),
            format!("Program {prog} failed: custom program error: 0x1"),
        ];

        let ctx = minimal_ctx();
        let frames = parse_log_frames(&logs, &ctx);
        assert_eq!(frames.len(), 1);
        assert!(matches!(frames[0].result, TxResult::Failure { .. }));
        if let TxResult::Failure { error, .. } = &frames[0].result {
            assert!(error.contains("0x1"));
        }
    }

    #[test]
    fn captures_return_data() {
        let prog = "SomeProgram1111111111111111111111111111111111";
        let logs: Vec<String> = vec![
            format!("Program {prog} invoke [1]"),
            format!("Program return: {prog} aGVsbG8="),
            format!("Program {prog} consumed 1000 of 200000 compute units"),
            format!("Program {prog} success"),
        ];

        let ctx = minimal_ctx();
        let frames = parse_log_frames(&logs, &ctx);
        assert_eq!(frames[0].return_data.as_deref(), Some("aGVsbG8="));
    }

    #[test]
    fn three_level_nesting() {
        let a = "ProgramA111111111111111111111111111111111111";
        let b = "ProgramB111111111111111111111111111111111111";
        let c = "ProgramC111111111111111111111111111111111111";
        let logs: Vec<String> = vec![
            format!("Program {a} invoke [1]"),
            format!("Program {b} invoke [2]"),
            format!("Program {c} invoke [3]"),
            format!("Program {c} consumed 100 of 100000 compute units"),
            format!("Program {c} success"),
            format!("Program {b} consumed 500 of 100000 compute units"),
            format!("Program {b} success"),
            format!("Program {a} consumed 1000 of 100000 compute units"),
            format!("Program {a} success"),
        ];

        let ctx = minimal_ctx();
        let frames = parse_log_frames(&logs, &ctx);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].children.len(), 1);
        assert_eq!(frames[0].children[0].children.len(), 1);
        assert_eq!(frames[0].children[0].children[0].program_id, c);
        assert_eq!(frames[0].children[0].children[0].depth, 2);
    }

    // Minimal TxContext suitable for unit tests (no accounts, no instructions).
    fn minimal_ctx() -> TxContext {
        use solana_sdk::message::{Message, VersionedMessage};
        use solana_sdk::transaction::VersionedTransaction;
        TxContext {
            signature: solana_sdk::signature::Signature::default(),
            slot: 0,
            block_time: None,
            original_tx: VersionedTransaction {
                signatures: vec![solana_sdk::signature::Signature::default()],
                message: VersionedMessage::Legacy(Message::new(&[], None)),
            },
            resolved_account_keys: vec![],
            mainnet_logs: vec![],
            mainnet_result: TxResult::Success,
            compute_budget_instructions: vec![],
            pre_balances: vec![],
            post_balances: vec![],
            pre_account_snapshots: std::collections::HashMap::new(),
        }
    }
}
