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
import { LiveReplayPanel } from "@/components/LiveReplayPanel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { AddressChip } from "@/components/AddressChip";
import { Play, ArrowLeft, Radio, GitCompare } from "lucide-react";

interface PageProps {
  params: Promise<{ signature: string }>;
}

function ResultBadge({ trace }: { trace: Trace }) {
  const ok = trace.mainnet_result.status === "success";
  const replayDiverged = trace.replay_result.status !== trace.mainnet_result.status;
  return (
    <span
      className={[
        "inline-flex items-center gap-1 text-[11px] font-semibold px-2 py-0.5 rounded-full border",
        ok
          ? "bg-[#14F195]/10 text-[#14F195] border-[#14F195]/30"
          : "bg-red-500/10 text-red-400 border-red-500/30",
      ].join(" ")}
      title={replayDiverged ? "Mainnet result. Replay simulation diverged." : undefined}
    >
      <span className={["w-1.5 h-1.5 rounded-full", ok ? "bg-[#14F195]" : "bg-red-400"].join(" ")} />
      {ok ? "Success" : "Failed"}
    </span>
  );
}

function formatTime(ts: number | null | undefined): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function ReplayPage({ params }: PageProps) {
  const { signature } = use(params);
  const decoded = decodeURIComponent(signature);
  const router = useRouter();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [rerunning, setRerunning] = useState(false);
  const [showDiff, setShowDiff] = useState(false);
  const [view, setView] = useState<"trace" | "live">("trace");

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
    } catch (e) {
      console.error("Re-run failed:", e);
    } finally {
      setRerunning(false);
    }
  };

  /* ── Loading ──────────────────────────────────────────── */
  if (loading) {
    const short =
      decoded.length > 24
        ? decoded.slice(0, 10) + "…" + decoded.slice(-8)
        : decoded;
    return (
      <div className="relative flex flex-col items-center justify-center min-h-screen gap-5 bg-zinc-950">
        <div
          className="absolute inset-0 pointer-events-none"
          style={{
            background:
              "radial-gradient(ellipse 60% 60% at 50% 40%, rgba(153,69,255,0.07), transparent)",
          }}
        />
        {/* Spinner */}
        <div className="relative w-12 h-12">
          <div className="absolute inset-0 rounded-full border border-zinc-800" />
          <div
            className="absolute inset-0 rounded-full border-2 border-transparent animate-spin"
            style={{ borderTopColor: "#9945FF" }}
          />
          <div
            className="absolute inset-3 rounded-full opacity-40"
            style={{ background: "radial-gradient(circle, #9945FF, transparent)" }}
          />
        </div>
        <div className="text-center space-y-1.5">
          <p className="text-zinc-200 text-sm font-medium">Fetching transaction…</p>
          <p className="text-zinc-600 text-xs font-mono">{short}</p>
          <p className="text-zinc-700 text-xs">usually 2–5 seconds</p>
        </div>
      </div>
    );
  }

  /* ── Error ────────────────────────────────────────────── */
  if (error || !currentTrace) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen gap-5 bg-zinc-950 px-4">
        <div className="w-10 h-10 rounded-full bg-red-500/10 border border-red-500/20 flex items-center justify-center">
          <span className="text-red-400 font-mono text-base">✗</span>
        </div>
        <div className="text-center space-y-1">
          <p className="text-zinc-200 text-sm font-medium">Replay failed</p>
          <p className="text-red-400/70 text-xs max-w-sm leading-relaxed">
            {error ?? "No trace data."}
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => router.push("/")}>
          <ArrowLeft size={12} className="mr-1" />
          Back to home
        </Button>
      </div>
    );
  }

  const trace = currentTrace;

  /* ── Main UI ──────────────────────────────────────────── */
  return (
    <div className="flex flex-col h-screen overflow-hidden bg-zinc-950">
      {/* ── Top bar ─────────────────────────────────────── */}
      <header className="flex items-center gap-2 px-3 py-2 border-b border-zinc-800/80 bg-zinc-950/80 backdrop-blur-sm shrink-0">
        {/* Back */}
        <button
          onClick={() => router.push("/")}
          className="p-1 text-zinc-500 hover:text-zinc-300 rounded hover:bg-zinc-800 transition-colors"
          aria-label="Back"
        >
          <ArrowLeft size={14} />
        </button>

        {/* Sig + badges */}
        <AddressChip address={trace.signature} />
        <ResultBadge trace={trace} />

        {/* Slot + time */}
        <span className="text-[11px] text-zinc-600 tabular-nums hidden sm:block">
          Slot{" "}
          <span className="text-zinc-400">{trace.slot.toLocaleString()}</span>
        </span>
        <span className="text-[11px] text-zinc-700 hidden md:block">
          {formatTime(trace.block_time)}
        </span>

        {/* Trace / Live toggle */}
        <div className="ml-1 inline-flex items-center rounded-md border border-zinc-800 overflow-hidden text-[11px] font-medium">
          <button
            onClick={() => setView("trace")}
            className={[
              "flex items-center gap-1.5 px-2.5 py-1 transition-colors",
              view === "trace"
                ? "bg-zinc-800 text-zinc-100"
                : "text-zinc-600 hover:text-zinc-400",
            ].join(" ")}
          >
            <GitCompare size={11} />
            Trace
          </button>
          <button
            onClick={() => setView("live")}
            className={[
              "flex items-center gap-1.5 px-2.5 py-1 border-l border-zinc-800 transition-colors",
              view === "live"
                ? "bg-zinc-800 text-zinc-100"
                : "text-zinc-600 hover:text-zinc-400",
            ].join(" ")}
          >
            <Radio size={11} className={view === "live" ? "text-[#14F195] animate-pulse" : ""} />
            Live
          </button>
        </div>

        {/* Right side */}
        <div className="ml-auto flex items-center gap-2 min-w-0">
          {diff && !showDiff && (
            <DiffSummaryCard
              diff={diff}
              onView={() => setShowDiff(true)}
              onClose={() => setDiff(null)}
            />
          )}

          <CuGauge frames={trace.frames} totalCu={trace.total_cu} />
          <span className="text-[11px] text-zinc-500 tabular-nums hidden sm:block">
            {trace.total_cu.toLocaleString()} CU
          </span>

          {trace.log_divergence && (
            <Badge variant="destructive" className="text-[10px] shrink-0">
              Log ⚠
            </Badge>
          )}

          {showDiff && (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setShowDiff(false)}
              className="h-7 text-xs text-zinc-400 shrink-0"
            >
              <ArrowLeft size={11} className="mr-1" />
              Trace
            </Button>
          )}

          {/* Re-run */}
          <button
            onClick={handleRerun}
            disabled={rerunning || pendingMutations.length === 0}
            className="shrink-0 h-7 px-3 rounded-md text-[11px] font-semibold text-white flex items-center gap-1.5 transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed hover:brightness-110 active:scale-95"
            style={{
              background:
                pendingMutations.length > 0
                  ? "linear-gradient(135deg, #9945FF 0%, #7d2be8 100%)"
                  : "rgba(39,39,42,1)",
              boxShadow:
                pendingMutations.length > 0
                  ? "0 0 14px rgba(153,69,255,0.3)"
                  : "none",
            }}
          >
            {rerunning ? (
              <>
                <span className="w-3 h-3 border-[1.5px] border-white/30 border-t-white rounded-full animate-spin" />
                Running…
              </>
            ) : (
              <>
                <Play size={10} />
                Re-run
              </>
            )}
            {pendingMutations.length > 0 && (
              <span className="ml-0.5 bg-white/20 rounded-full text-[9px] font-bold px-1.5 py-0.5 leading-none">
                {pendingMutations.length}
              </span>
            )}
          </button>
        </div>
      </header>

      {/* Timeline */}
      {!showDiff && view === "trace" && (
        <Timeline frames={trace.frames} totalCu={trace.total_cu} />
      )}

      {/* Body */}
      {view === "live" ? (
        <div className="flex-1 overflow-hidden">
          <LiveReplayPanel signature={decoded} />
        </div>
      ) : showDiff && diff ? (
        <div className="flex-1 overflow-hidden">
          <DiffView diff={diff} />
        </div>
      ) : (
        <div className="flex flex-1 overflow-hidden">
          {/* Left: CPI tree */}
          <aside className="w-64 shrink-0 border-r border-zinc-800/60 overflow-y-auto bg-zinc-950">
            <div className="px-3 py-2 text-[10px] uppercase tracking-widest text-zinc-600 border-b border-zinc-800/60 flex items-center gap-1.5">
              <span className="w-1 h-3 rounded-sm bg-[#9945FF]/60" />
              Trace tree
            </div>
            <CpiTree frames={trace.frames} totalCu={trace.total_cu} />
          </aside>

          {/* Center: frame detail */}
          <main className="flex-1 overflow-hidden bg-zinc-950">
            <FrameDetail />
          </main>

          {/* Right: account inspector */}
          <aside className="w-72 shrink-0 border-l border-zinc-800/60 overflow-hidden flex flex-col bg-zinc-950">
            <div className="px-3 py-2 text-[10px] uppercase tracking-widest text-zinc-600 border-b border-zinc-800/60 flex items-center gap-1.5">
              <span className="w-1 h-3 rounded-sm bg-zinc-600/60" />
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
