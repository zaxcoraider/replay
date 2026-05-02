"use client";
import { useEffect, useRef } from "react";
import type { CpiFrame } from "@/lib/types";
import { useReplayStore } from "@/store/replay-store";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";
import { clsx } from "clsx";

const KNOWN_NAMES: Record<string, string> = {
  JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4: "Jupiter v6",
  whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc: "Whirlpool",
  ComputeBudget111111111111111111111111111111: "ComputeBudget",
  "11111111111111111111111111111111": "System",
  TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA: "SPL Token",
  dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH: "Drift v2",
};

const CB_PROGRAM = "ComputeBudget111111111111111111111111111111";

function frameId(f: CpiFrame) {
  return `${f.depth}-${f.instruction_index}-${f.program_id}`;
}

function segmentColor(frame: CpiFrame, selected: boolean): string {
  if (selected) return "bg-blue-500";
  if (frame.result.status === "failure") return "bg-red-600/80 hover:bg-red-500";
  if (frame.program_id === CB_PROGRAM) return "bg-zinc-700 hover:bg-zinc-600";
  return "bg-zinc-600 hover:bg-zinc-400";
}

function progName(f: CpiFrame): string {
  return f.program_name ?? KNOWN_NAMES[f.program_id] ?? `${f.program_id.slice(0, 6)}…`;
}

function childSummary(children: CpiFrame[]): string {
  if (children.length === 0) return "";
  return children
    .map((c) => `  → ${progName(c)}${c.instruction_name ? ` (${c.instruction_name})` : ""} ${c.cu_consumed.toLocaleString()} CU`)
    .join("\n");
}

interface Props {
  frames: CpiFrame[];
  totalCu: number;
}

export function Timeline({ frames, totalCu }: Props) {
  const { selectedFrameId, setSelectedFrame } = useReplayStore();
  const containerRef = useRef<HTMLDivElement>(null);

  // Keyboard navigation
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!containerRef.current?.contains(document.activeElement) &&
          document.activeElement?.tagName !== "BODY") return;
      if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
        e.preventDefault();
        const idx = selectedFrameId
          ? frames.findIndex((f) => frameId(f) === selectedFrameId)
          : -1;
        const next = e.key === "ArrowRight"
          ? Math.min(frames.length - 1, idx + 1)
          : Math.max(0, idx - 1);
        if (frames[next]) setSelectedFrame(frameId(frames[next]));
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [frames, selectedFrameId, setSelectedFrame]);

  if (frames.length === 0) return null;

  // Each segment gets at least 8px; remaining width is distributed proportionally.
  const MIN_PX = 8;
  const totalMinPx = frames.length * MIN_PX;
  // We express widths as flex-grow values (proportional to CU, floored at MIN_PX via min-w).
  // Using a flex layout with min-w-[8px] on each segment achieves this cleanly.

  return (
    <div
      ref={containerRef}
      className="flex items-stretch gap-0.5 px-4 py-2 bg-zinc-900 border-b border-zinc-800 h-12 shrink-0"
      role="toolbar"
      aria-label="Instruction timeline"
    >
      {frames.map((frame, i) => {
        const id = frameId(frame);
        const selected = selectedFrameId === id;
        const cuShare = totalCu > 0 ? frame.cu_consumed / totalCu : 1 / frames.length;
        // flex-grow proportional to CU; min-w ensures tiny instructions are still clickable
        const grow = Math.max(cuShare * 100, 0.5);
        const name = frame.instruction_name ?? progName(frame);
        const tip = [
          `${progName(frame)}`,
          frame.instruction_name ? `Instruction: ${frame.instruction_name}` : "",
          `CU: ${frame.cu_consumed.toLocaleString()}`,
          frame.result.status === "failure"
            ? `Failed: ${(frame.result as { error: string }).error}`
            : "",
          frame.children.length > 0
            ? `\nCPI calls:\n${childSummary(frame.children)}`
            : "",
        ]
          .filter(Boolean)
          .join("\n");

        return (
          <Tooltip key={i}>
            <TooltipTrigger>
              <button
                className={clsx(
                  "h-full rounded-sm flex flex-col justify-end px-1 pb-1 transition-colors cursor-pointer focus:outline-none focus:ring-1 focus:ring-blue-400 min-w-[8px]",
                  segmentColor(frame, selected)
                )}
                style={{ flexGrow: grow }}
                onClick={() => setSelectedFrame(selected ? null : id)}
                aria-label={`${name} — ${frame.cu_consumed.toLocaleString()} CU`}
                aria-pressed={selected}
              >
                {/* Label only if segment is wide enough — hide on narrow segments */}
                <span className="text-[9px] text-white/70 truncate leading-none hidden sm:block">
                  {name}
                </span>
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom" className="whitespace-pre text-xs max-w-xs">
              {tip}
            </TooltipContent>
          </Tooltip>
        );
      })}

      {/* Filler so last segment doesn't stretch to fill */}
      <div className="flex-1" />
    </div>
  );
}
