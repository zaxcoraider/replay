"use client";
import type { TraceDiff } from "@/lib/types";
import { Badge } from "@/components/ui/badge";
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
    <div className="flex items-center gap-2 px-2 py-1 bg-zinc-800/70 border border-zinc-700 rounded text-xs shrink-0">
      {/* Result indicator */}
      <div className="flex items-center gap-1">
        <span className={baseOk ? "text-green-400" : "text-red-400"}>{baseOk ? "✓" : "✗"}</span>
        <span className="text-zinc-600 text-[10px]">→</span>
        <span className={latestOk ? "text-green-400" : "text-red-400"}>{latestOk ? "✓" : "✗"}</span>
        {result_changed && (
          <Badge variant="destructive" className="text-[9px] ml-0.5 py-0">!</Badge>
        )}
      </div>

      {/* CU delta */}
      {total_cu_delta !== 0 && (
        <span
          className={`tabular-nums font-medium ${
            total_cu_delta > 0 ? "text-red-400" : "text-green-400"
          }`}
        >
          CU {total_cu_delta > 0 ? "+" : ""}
          {cuPct.toFixed(1)}%
        </span>
      )}

      {/* Account count */}
      {changed_accounts.length > 0 && (
        <span className="text-zinc-400">{changed_accounts.length} accts</span>
      )}

      <Button
        size="sm"
        variant="outline"
        onClick={onView}
        className="h-5 text-[10px] px-2 py-0 border-blue-700 text-blue-400 hover:text-blue-300"
      >
        View diff
      </Button>
      <button
        onClick={onClose}
        className="text-zinc-600 hover:text-zinc-400 text-xs leading-none"
        aria-label="Close diff"
      >
        ×
      </button>
    </div>
  );
}
