"use client";
import { useState } from "react";
import type { CpiFrame } from "@/lib/types";
import { useReplayStore } from "@/store/replay-store";
import { AddressChip } from "./AddressChip";
import { ChevronRight, ChevronDown } from "lucide-react";
import { clsx } from "clsx";

const KNOWN_NAMES: Record<string, string> = {
  JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4: "Jupiter v6",
  whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc: "Whirlpool",
  ComputeBudget111111111111111111111111111111: "ComputeBudget",
  "11111111111111111111111111111111": "System",
  TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA: "SPL Token",
};

function frameId(f: CpiFrame) {
  return `${f.depth}-${f.instruction_index}-${f.program_id}`;
}

function isSuccess(f: CpiFrame) {
  return f.result.status === "success";
}

function CuBar({ cu, total }: { cu: number; total: number }) {
  const pct = total > 0 ? Math.max(2, (cu / total) * 100) : 0;
  return (
    <div className="mt-1 h-0.5 rounded bg-zinc-700 w-full">
      <div className="h-full rounded bg-blue-500 opacity-60" style={{ width: `${pct}%` }} />
    </div>
  );
}

interface NodeProps {
  frame: CpiFrame;
  totalCu: number;
}

function FrameNode({ frame, totalCu }: NodeProps) {
  const [open, setOpen] = useState(frame.depth === 0);
  const { selectedFrameId, setSelectedFrame } = useReplayStore();
  const id = frameId(frame);
  const selected = selectedFrameId === id;
  const name =
    frame.instruction_name ??
    KNOWN_NAMES[frame.program_id] ??
    `${frame.program_id.slice(0, 6)}…`;

  return (
    <div className="select-none">
      <div
        className={clsx(
          "flex items-start gap-1 rounded px-2 py-1 cursor-pointer text-sm group",
          selected ? "bg-zinc-700" : "hover:bg-zinc-800",
          !isSuccess(frame) && "text-red-400"
        )}
        style={{ paddingLeft: `${frame.depth * 14 + 8}px` }}
        onClick={() => setSelectedFrame(selected ? null : id)}
      >
        {frame.children.length > 0 ? (
          <button
            className="mt-0.5 shrink-0 text-zinc-400"
            onClick={(e) => { e.stopPropagation(); setOpen((o) => !o); }}
          >
            {open ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
          </button>
        ) : (
          <span className="w-3.5 shrink-0" />
        )}

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 flex-wrap">
            <span className="font-semibold truncate">{name}</span>
            <AddressChip address={frame.program_id} />
            {!isSuccess(frame) && (
              <span className="text-xs text-red-400 font-medium">failed</span>
            )}
          </div>
          <div className="text-xs text-zinc-500">
            {frame.cu_consumed.toLocaleString()} CU
          </div>
          <CuBar cu={frame.cu_consumed} total={totalCu} />
        </div>
      </div>

      {open && frame.children.map((child, i) => (
        <FrameNode key={i} frame={child} totalCu={totalCu} />
      ))}
    </div>
  );
}

export function CpiTree({ frames, totalCu }: { frames: CpiFrame[]; totalCu: number }) {
  if (frames.length === 0) {
    return <p className="text-zinc-500 text-sm p-4">No frames.</p>;
  }
  return (
    <div className="space-y-0.5">
      {frames.map((f, i) => (
        <FrameNode key={i} frame={f} totalCu={totalCu} />
      ))}
    </div>
  );
}
