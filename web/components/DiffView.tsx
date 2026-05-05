"use client";
import type { TraceDiff, CpiFrame, AccountDelta } from "@/lib/types";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { AddressChip } from "./AddressChip";

const LAMPORTS_PER_SOL = 1_000_000_000;

interface Props {
  diff: TraceDiff;
}

// ---------- Frame row ----------

function FrameRow({ base, latest }: { base: CpiFrame | null; latest: CpiFrame | null }) {
  const label = (f: CpiFrame | null) =>
    f ? (f.instruction_name ?? f.program_name ?? f.program_id.slice(0, 8) + "…") : null;

  const baseOk = !base || base.result.status === "success";
  const latestOk = !latest || latest.result.status === "success";
  const resultChanged = base && latest && baseOk !== latestOk;
  const cuDelta = base && latest ? latest.cu_consumed - base.cu_consumed : 0;
  const bigCuShift = base && Math.abs(cuDelta) / (base.cu_consumed || 1) > 0.15;

  const rowBg = resultChanged
    ? "bg-red-950/25 border-l-2 border-l-red-700"
    : bigCuShift
    ? "bg-yellow-950/20 border-l-2 border-l-yellow-700"
    : "";

  return (
    <div className={`grid grid-cols-2 gap-2 px-3 py-1.5 border-b border-zinc-800/60 ${rowBg}`}>
      <div className="flex items-center gap-1.5 min-w-0">
        {base ? (
          <>
            <span
              className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                baseOk ? "bg-green-500" : "bg-red-500"
              }`}
            />
            <span className="text-xs font-mono text-zinc-300 truncate">{label(base)}</span>
            <span className="text-[10px] text-zinc-600 tabular-nums shrink-0 ml-auto">
              {base.cu_consumed.toLocaleString()}
            </span>
          </>
        ) : (
          <span className="text-zinc-700 text-xs italic">removed</span>
        )}
      </div>

      <div className="flex items-center gap-1.5 min-w-0">
        {latest ? (
          <>
            <span
              className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                latestOk ? "bg-green-500" : "bg-red-500"
              }`}
            />
            <span className="text-xs font-mono text-zinc-300 truncate">{label(latest)}</span>
            <div className="flex items-center gap-1 ml-auto shrink-0">
              <span className="text-[10px] text-zinc-600 tabular-nums">
                {latest.cu_consumed.toLocaleString()}
              </span>
              {cuDelta !== 0 && (
                <span
                  className={`text-[10px] tabular-nums font-medium ${
                    cuDelta > 0 ? "text-red-400" : "text-green-400"
                  }`}
                >
                  {cuDelta > 0 ? "+" : ""}
                  {cuDelta.toLocaleString()}
                </span>
              )}
            </div>
          </>
        ) : (
          <span className="text-zinc-700 text-xs italic">removed</span>
        )}
      </div>
    </div>
  );
}

// ---------- Log diff ----------

function LogDiff({ baseLogs, latestLogs }: { baseLogs: string[]; latestLogs: string[] }) {
  const baseSet = new Set(baseLogs);
  const latestSet = new Set(latestLogs);
  const allLines = [
    ...baseLogs.filter((l) => !latestSet.has(l)).map((l) => ({ line: l, side: "base" as const })),
    ...latestLogs.filter((l) => !baseSet.has(l)).map((l) => ({ line: l, side: "latest" as const })),
  ];
  if (allLines.length === 0) return <p className="text-zinc-600 text-xs">Logs identical.</p>;
  return (
    <div className="space-y-0.5">
      {allLines.map((item, i) => (
        <div
          key={i}
          className={`text-[10px] font-mono px-1.5 py-0.5 rounded ${
            item.side === "base"
              ? "bg-red-950/40 text-red-300 line-through decoration-red-600/50"
              : "bg-green-950/40 text-green-300"
          }`}
        >
          {item.side === "base" ? "- " : "+ "}
          {item.line}
        </div>
      ))}
    </div>
  );
}

// ---------- Account delta row ----------

function AccountDiffRow({
  pubkey,
  baselineDelta,
  latestDelta,
}: {
  pubkey: string;
  baselineDelta: AccountDelta | undefined;
  latestDelta: AccountDelta | undefined;
}) {
  const baseLamDelta = baselineDelta
    ? baselineDelta.lamports_after - baselineDelta.lamports_before
    : 0;
  const latLamDelta = latestDelta
    ? latestDelta.lamports_after - latestDelta.lamports_before
    : 0;
  const lamChanged = baseLamDelta !== latLamDelta;
  const dataChanged = baselineDelta?.data_after_hex !== latestDelta?.data_after_hex;

  const fmt = (v: number) =>
    `${v >= 0 ? "+" : ""}${(v / LAMPORTS_PER_SOL).toFixed(6)} SOL`;

  return (
    <div className="px-3 py-2 border-b border-zinc-800/60">
      <div className="flex items-center gap-2 mb-1">
        <AddressChip address={pubkey} />
        <div className="flex gap-1">
          {lamChanged && (
            <Badge variant="outline" className="text-[9px] text-yellow-400 border-yellow-700 py-0">
              lamports
            </Badge>
          )}
          {dataChanged && (
            <Badge variant="outline" className="text-[9px] text-blue-400 border-blue-700 py-0">
              data
            </Badge>
          )}
        </div>
      </div>

      {lamChanged && (
        <div className="grid grid-cols-2 gap-2 text-[11px] font-mono mt-0.5">
          <span className="text-zinc-400">{fmt(baseLamDelta)}</span>
          <span className={latLamDelta !== baseLamDelta ? "text-yellow-300" : "text-zinc-400"}>
            {fmt(latLamDelta)}
          </span>
        </div>
      )}
    </div>
  );
}

// ---------- Main DiffView ----------

export function DiffView({ diff }: Props) {
  const { baseline, latest, result_changed, total_cu_delta, changed_accounts } = diff;

  const baseOk = baseline.replay_result.status === "success";
  const latestOk = latest.replay_result.status === "success";
  const cuPct =
    baseline.total_cu > 0 ? (total_cu_delta / baseline.total_cu) * 100 : 0;

  const maxLen = Math.max(baseline.frames.length, latest.frames.length);
  const frameRows = Array.from({ length: maxLen }, (_, i) => ({
    base: baseline.frames[i] ?? null,
    latest: latest.frames[i] ?? null,
  }));

  // Collect all unique logs from both traces (flat across all frames)
  const baseLogs = baseline.frames.flatMap((f) => f.logs);
  const latestLogs = latest.frames.flatMap((f) => f.logs);
  const hasLogDiff =
    JSON.stringify(baseLogs) !== JSON.stringify(latestLogs);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Summary banner */}
      <div className="px-4 py-3 border-b border-zinc-800/60 bg-zinc-900/60 shrink-0">
        <div className="flex items-center gap-6 flex-wrap">
          {/* Result */}
          <div className="flex flex-col gap-0.5">
            <span className="text-[9px] uppercase tracking-widest text-zinc-600">Result</span>
            <div className="flex items-center gap-1.5 font-mono text-sm font-semibold">
              <span className={baseOk ? "text-[#14F195]" : "text-red-400"}>{baseOk ? "✓" : "✗"}</span>
              <span className="text-zinc-700 text-xs">→</span>
              <span className={latestOk ? "text-[#14F195]" : "text-red-400"}>{latestOk ? "✓" : "✗"}</span>
              {result_changed && (
                <Badge variant="destructive" className="text-[9px] ml-1">CHANGED</Badge>
              )}
            </div>
          </div>

          <div className="w-px h-8 bg-zinc-800" />

          {/* CU */}
          <div className="flex flex-col gap-0.5">
            <span className="text-[9px] uppercase tracking-widest text-zinc-600">Compute units</span>
            <div className="flex items-center gap-1.5 text-sm font-mono">
              <span className="text-zinc-300 tabular-nums">{baseline.total_cu.toLocaleString()}</span>
              <span className="text-zinc-700 text-xs">→</span>
              <span className="text-zinc-300 tabular-nums">{latest.total_cu.toLocaleString()}</span>
              {total_cu_delta !== 0 && (
                <span
                  className={`text-xs font-bold tabular-nums px-1.5 py-0.5 rounded ${
                    total_cu_delta > 0
                      ? "text-red-400 bg-red-500/10"
                      : "text-[#14F195] bg-[#14F195]/10"
                  }`}
                >
                  {total_cu_delta > 0 ? "+" : ""}{cuPct.toFixed(1)}%
                </span>
              )}
            </div>
          </div>

          <div className="w-px h-8 bg-zinc-800" />

          {/* Accounts */}
          <div className="flex flex-col gap-0.5">
            <span className="text-[9px] uppercase tracking-widest text-zinc-600">Accounts</span>
            <span className="text-sm font-semibold text-zinc-300">
              {changed_accounts.length}{" "}
              <span className="text-zinc-600 font-normal text-xs">changed</span>
            </span>
          </div>
        </div>
      </div>

      {/* Column headers */}
      <div className="grid grid-cols-2 gap-2 px-3 py-1.5 border-b border-zinc-800/60 bg-zinc-900/20 shrink-0">
        <span className="text-[10px] uppercase tracking-widest text-zinc-600">Baseline</span>
        <span className="text-[10px] uppercase tracking-widest text-zinc-600">Re-run</span>
      </div>

      <ScrollArea className="flex-1">
        {/* Frame-level diff */}
        {frameRows.map((row, i) => (
          <FrameRow key={i} base={row.base} latest={row.latest} />
        ))}

        {/* Log diff */}
        {hasLogDiff && (
          <div className="mt-2 border-t border-zinc-800">
            <div className="px-3 py-1.5 text-[10px] uppercase tracking-widest text-zinc-600 border-b border-zinc-800 bg-zinc-900/50">
              Log diff
            </div>
            <div className="px-3 py-2">
              <LogDiff baseLogs={baseLogs} latestLogs={latestLogs} />
            </div>
          </div>
        )}

        {/* Account delta diff */}
        {changed_accounts.length > 0 && (
          <div className="mt-2 border-t border-zinc-800">
            <div className="px-3 py-1.5 text-[10px] uppercase tracking-widest text-zinc-600 border-b border-zinc-800 bg-zinc-900/50">
              Changed accounts
            </div>
            {changed_accounts.map((pk) => (
              <AccountDiffRow
                key={pk}
                pubkey={pk}
                baselineDelta={baseline.account_deltas.find((d) => d.pubkey === pk)}
                latestDelta={latest.account_deltas.find((d) => d.pubkey === pk)}
              />
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
