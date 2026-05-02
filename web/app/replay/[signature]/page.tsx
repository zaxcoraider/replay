"use client";
import { use, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { fork, execute, diff as getDiff } from "@/lib/api";
import type { Trace } from "@/lib/types";
import { useReplayStore } from "@/store/replay-store";
import { CpiTree } from "@/components/CpiTree";
import { FrameDetail } from "@/components/FrameDetail";
import { AccountInspector } from "@/components/AccountInspector";
import { Timeline } from "@/components/Timeline";
import { CuGauge } from "@/components/CuGauge";
import { DiffView } from "@/components/DiffView";
import { DiffSummaryCard } from "@/components/DiffSummaryCard";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { AddressChip } from "@/components/AddressChip";

interface PageProps {
  params: Promise<{ signature: string }>;
}

function ResultBadge({ trace }: { trace: Trace }) {
  const ok = trace.replay_result.status === "success";
  return (
    <Badge variant={ok ? "outline" : "destructive"} className="text-xs">
      {ok ? "✓ Success" : "✗ Failed"}
    </Badge>
  );
}

function formatTime(ts: number | null | undefined): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleString();
}

export default function ReplayPage({ params }: PageProps) {
  const { signature } = use(params);
  const decoded = decodeURIComponent(signature);
  const router = useRouter();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [rerunning, setRerunning] = useState(false);
  const [showDiff, setShowDiff] = useState(false);

  const { currentTrace, sessionId, pendingMutations, diff, setSession, setTrace, setDiff } =
    useReplayStore();

  useEffect(() => {
    setLoading(true);
    setError(null);
    setShowDiff(false);
    fork(decoded)
      .then((r) => {
        setSession(r.session_id, r.baseline_trace);
        setLoading(false);
      })
      .catch((e: Error) => {
        setError(e.message);
        setLoading(false);
      });
  }, [decoded, setSession]);

  const handleRerun = async () => {
    if (!sessionId) return;
    setRerunning(true);
    try {
      const result = await execute(sessionId);
      setTrace(result.trace);
      const d = await getDiff(sessionId);
      setDiff(d);
      // Don't auto-open diff — let the user click "View diff" in the summary card
    } catch (e) {
      console.error("Re-run failed:", e);
    } finally {
      setRerunning(false);
    }
  };

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen gap-3">
        <div className="w-5 h-5 border-2 border-zinc-600 border-t-zinc-300 rounded-full animate-spin" />
        <p className="text-zinc-500 text-sm">Fetching tx from Helius… (usually 2–5s)</p>
        <p className="text-zinc-600 text-xs font-mono truncate max-w-xs">{decoded}</p>
      </div>
    );
  }

  if (error || !currentTrace) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen gap-4">
        <p className="text-red-400 text-sm max-w-md text-center">{error ?? "No trace data."}</p>
        <Button variant="outline" onClick={() => router.push("/")}>← Back</Button>
      </div>
    );
  }

  const trace = currentTrace;

  return (
    <div className="flex flex-col h-screen overflow-hidden">
      {/* Top bar */}
      <header className="flex items-center gap-3 px-4 py-2 border-b border-zinc-800 shrink-0">
        <button
          onClick={() => router.push("/")}
          className="text-zinc-500 hover:text-zinc-300 text-sm"
        >
          ←
        </button>
        <AddressChip address={trace.signature} />
        <ResultBadge trace={trace} />
        <span className="text-xs text-zinc-500">Slot {trace.slot.toLocaleString()}</span>
        <span className="text-xs text-zinc-500">{formatTime(trace.block_time)}</span>

        <div className="ml-auto flex items-center gap-2">
          {/* Diff summary card — shown after a re-run when not in diff mode */}
          {diff && !showDiff && (
            <DiffSummaryCard
              diff={diff}
              onView={() => setShowDiff(true)}
              onClose={() => setDiff(null)}
            />
          )}

          <CuGauge frames={trace.frames} totalCu={trace.total_cu} />
          <span className="text-xs text-zinc-500">{trace.total_cu.toLocaleString()} CU</span>

          {trace.log_divergence && (
            <Badge variant="destructive" className="text-xs">
              Log divergence
            </Badge>
          )}

          {/* Toggle back from diff view */}
          {showDiff && (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setShowDiff(false)}
              className="h-7 text-xs text-zinc-400"
            >
              ← Trace
            </Button>
          )}

          {/* Re-run button */}
          <Button
            size="sm"
            onClick={handleRerun}
            disabled={rerunning || pendingMutations.length === 0}
            className="h-7 text-xs gap-1.5"
          >
            {rerunning ? "Running…" : "Re-run"}
            {pendingMutations.length > 0 && (
              <Badge variant="secondary" className="text-[10px] h-4 px-1.5 py-0">
                {pendingMutations.length}
              </Badge>
            )}
          </Button>
        </div>
      </header>

      {/* Timeline scrubber — hidden in diff mode */}
      {!showDiff && <Timeline frames={trace.frames} totalCu={trace.total_cu} />}

      {/* Body */}
      {showDiff && diff ? (
        <div className="flex-1 overflow-hidden">
          <DiffView diff={diff} />
        </div>
      ) : (
        <div className="flex flex-1 overflow-hidden">
          {/* Left: CPI tree */}
          <aside className="w-64 shrink-0 border-r border-zinc-800 overflow-y-auto">
            <div className="px-3 py-2 text-[10px] uppercase tracking-widest text-zinc-600 border-b border-zinc-800">
              Trace tree
            </div>
            <CpiTree frames={trace.frames} totalCu={trace.total_cu} />
          </aside>

          {/* Center: frame detail */}
          <main className="flex-1 overflow-hidden">
            <FrameDetail />
          </main>

          {/* Right: account inspector */}
          <aside className="w-72 shrink-0 border-l border-zinc-800 overflow-hidden flex flex-col">
            <div className="px-3 py-2 text-[10px] uppercase tracking-widest text-zinc-600 border-b border-zinc-800">
              Account inspector
            </div>
            <div className="flex-1 overflow-hidden">
              <AccountInspector />
            </div>
          </aside>
        </div>
      )}
    </div>
  );
}
