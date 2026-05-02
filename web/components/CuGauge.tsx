"use client";
import { useMemo } from "react";
import type { CpiFrame } from "@/lib/types";
import { useReplayStore } from "@/store/replay-store";

function findFrame(frames: CpiFrame[], id: string): CpiFrame | null {
  for (const f of frames) {
    if (`${f.depth}-${f.instruction_index}-${f.program_id}` === id) return f;
    const c = findFrame(f.children, id);
    if (c) return c;
  }
  return null;
}

interface Props {
  frames: CpiFrame[];
  totalCu: number;
}

export function CuGauge({ frames, totalCu }: Props) {
  const { selectedFrameId, currentTrace } = useReplayStore();

  // Sum CU up to and including the selected frame (top-level frames in order).
  const { consumed, selectedCu } = useMemo(() => {
    if (!selectedFrameId || frames.length === 0) {
      return { consumed: 0, selectedCu: 0 };
    }
    let acc = 0;
    let sel = 0;
    for (const f of frames) {
      acc += f.cu_consumed;
      if (`${f.depth}-${f.instruction_index}-${f.program_id}` === selectedFrameId) {
        sel = f.cu_consumed;
        break;
      }
    }
    // Also check children — if a child is selected, use the parent's running total.
    if (sel === 0 && currentTrace) {
      const found = findFrame(frames, selectedFrameId);
      if (found) sel = found.cu_consumed;
    }
    return { consumed: acc, selectedCu: sel };
  }, [selectedFrameId, frames, currentTrace]);

  const pct = totalCu > 0 ? Math.min(100, (consumed / totalCu) * 100) : 0;
  const danger = pct > 80;
  const warn = pct > 60;

  return (
    <div className="flex items-center gap-2 shrink-0" title={`${consumed.toLocaleString()} / ${totalCu.toLocaleString()} CU`}>
      {/* Bar */}
      <div className="w-20 h-2 rounded-full bg-zinc-800 overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-300 ${
            danger ? "bg-red-500" : warn ? "bg-yellow-500" : "bg-blue-500"
          }`}
          style={{ width: `${pct}%` }}
        />
      </div>
      {/* Label */}
      <span className="text-xs text-zinc-400 tabular-nums whitespace-nowrap">
        {pct.toFixed(0)}%{selectedCu > 0 && (
          <span className="text-zinc-600"> ({selectedCu.toLocaleString()})</span>
        )}
      </span>
    </div>
  );
}
