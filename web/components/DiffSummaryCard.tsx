"use client";
import type { TraceDiff } from "@/lib/types";
import { Button } from "@/components/ui/button";

interface Props {
  diff: TraceDiff;
  onView: () => void;
  onClose: () => void;
}

export function DiffSummaryCard({ diff, onView, onClose }: Props) {
  const { result_changed, total_cu_delta, changed_accounts, baseline, latest } = diff;
  const baseOk = baseline.replay_result.status === "success";
  const latestOk = latest.replay_result.status === "success";
  const cuPct =
    baseline.total_cu > 0 ? (total_cu_delta / baseline.total_cu) * 100 : 0;

  return (
    <div className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-zinc-700/60 bg-zinc-900/80 text-xs shrink-0 backdrop-blur-sm">
      {/* Result flow */}
      <div className="flex items-center gap-1 font-mono">
        <span className={baseOk ? "text-[#14F195]" : "text-red-400"}>
          {baseOk ? "✓" : "✗"}
        </span>
        <span className="text-zinc-700 text-[10px]">→</span>
        <span className={latestOk ? "text-[#14F195]" : "text-red-400"}>
          {latestOk ? "✓" : "✗"}
        </span>
        {result_changed && (
          <span className="ml-0.5 text-[9px] font-bold text-red-400 bg-red-500/15 border border-red-500/25 rounded px-1">
            CHANGED
          </span>
        )}
      </div>

      {/* CU delta */}
      {total_cu_delta !== 0 && (
        <span
          className={`tabular-nums font-semibold text-[11px] ${
            total_cu_delta > 0 ? "text-red-400" : "text-[#14F195]"
          }`}
        >
          {total_cu_delta > 0 ? "+" : ""}
          {cuPct.toFixed(1)}% CU
        </span>
      )}

      {/* Accounts changed */}
      {changed_accounts.length > 0 && (
        <span className="text-zinc-500">
          {changed_accounts.length} acct{changed_accounts.length !== 1 ? "s" : ""}
        </span>
      )}

      <Button
        size="sm"
        variant="outline"
        onClick={onView}
        className="h-5 text-[10px] px-2 py-0 border-[#9945FF]/40 text-[#9945FF] hover:text-white hover:bg-[#9945FF]/20 hover:border-[#9945FF]"
      >
        View diff
      </Button>
      <button
        onClick={onClose}
        className="text-zinc-600 hover:text-zinc-300 text-sm leading-none transition-colors"
        aria-label="Close diff"
      >
        ×
      </button>
    </div>
  );
}
