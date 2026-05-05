"use client";
import { useState } from "react";
import type { CpiFrame } from "@/lib/types";
import { useReplayStore } from "@/store/replay-store";
import { AddressChip } from "./AddressChip";
import { ChevronRight, ChevronDown } from "lucide-react";
import { clsx } from "clsx";

/* Program metadata: name + tailwind color classes */
interface ProgramMeta {
  name: string;
  dot: string;    // dot bg color
  text: string;   // label text color when not failed/selected
}

const PROGRAM_META: Record<string, ProgramMeta> = {
  JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4: {
    name: "Jupiter v6",
    dot: "bg-purple-400",
    text: "text-purple-300",
  },
  whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc: {
    name: "Whirlpool",
    dot: "bg-blue-400",
    text: "text-blue-300",
  },
  ComputeBudget111111111111111111111111111111: {
    name: "ComputeBudget",
    dot: "bg-zinc-600",
    text: "text-zinc-500",
  },
  "11111111111111111111111111111111": {
    name: "System",
    dot: "bg-zinc-500",
    text: "text-zinc-400",
  },
  TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA: {
    name: "SPL Token",
    dot: "bg-emerald-400",
    text: "text-emerald-300",
  },
  dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH: {
    name: "Drift v2",
    dot: "bg-cyan-400",
    text: "text-cyan-300",
  },
  ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bEg: {
    name: "Assoc. Token",
    dot: "bg-teal-400",
    text: "text-teal-300",
  },
};

function getMeta(frame: CpiFrame): ProgramMeta {
  return (
    PROGRAM_META[frame.program_id] ?? {
      name: `${frame.program_id.slice(0, 6)}…`,
      dot: "bg-zinc-500",
      text: "text-zinc-300",
    }
  );
}

function frameId(f: CpiFrame) {
  return `${f.depth}-${f.instruction_index}-${f.program_id}`;
}

function isSuccess(f: CpiFrame) {
  return f.result.status === "success";
}

function CuBar({ cu, total }: { cu: number; total: number }) {
  const pct = total > 0 ? Math.max(2, (cu / total) * 100) : 0;
  return (
    <div className="mt-1.5 h-0.5 rounded-full bg-zinc-800 w-full overflow-hidden">
      <div
        className="h-full rounded-full"
        style={{
          width: `${pct}%`,
          background: "linear-gradient(90deg, #9945FF80, #9945FF)",
        }}
      />
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
  const failed = !isSuccess(frame);
  const meta = getMeta(frame);
  const displayName = frame.instruction_name ?? meta.name;

  return (
    <div className="select-none">
      <div
        className={clsx(
          "flex items-start gap-1.5 rounded-md mx-1 px-2 py-1.5 cursor-pointer transition-colors duration-100 group",
          selected
            ? "bg-[#9945FF]/15 border border-[#9945FF]/25"
            : "border border-transparent hover:bg-zinc-800/60"
        )}
        style={{ paddingLeft: `${frame.depth * 14 + 8}px` }}
        onClick={() => setSelectedFrame(selected ? null : id)}
      >
        {/* Expand toggle */}
        {frame.children.length > 0 ? (
          <button
            className="mt-0.5 shrink-0 text-zinc-500 hover:text-zinc-300 transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              setOpen((o) => !o);
            }}
          >
            {open ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
          </button>
        ) : (
          <span className="w-3.5 shrink-0" />
        )}

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 flex-wrap">
            {/* Program color dot */}
            <span
              className={clsx(
                "w-1.5 h-1.5 rounded-full shrink-0 mt-0.5",
                failed ? "bg-red-400" : meta.dot
              )}
            />
            <span
              className={clsx(
                "font-medium text-sm truncate",
                failed ? "text-red-400" : selected ? "text-zinc-100" : meta.text
              )}
            >
              {displayName}
            </span>
            {failed && (
              <span className="text-[10px] text-red-500 font-semibold bg-red-500/10 px-1 py-0 rounded">
                failed
              </span>
            )}
          </div>

          <div className="flex items-center gap-2 mt-0.5">
            <AddressChip address={frame.program_id} />
            <span className="text-[10px] text-zinc-600 tabular-nums ml-auto">
              {frame.cu_consumed.toLocaleString()} CU
            </span>
          </div>
          <CuBar cu={frame.cu_consumed} total={totalCu} />
        </div>
      </div>

      {open &&
        frame.children.map((child, i) => (
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
    <div className="py-1 space-y-0.5">
      {frames.map((f, i) => (
        <FrameNode key={i} frame={f} totalCu={totalCu} />
      ))}
    </div>
  );
}
