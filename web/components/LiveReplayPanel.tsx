"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import type { CpiFrame, LiveReplayEvent, LiveSource, Trace } from "@/lib/types";
import { Badge } from "@/components/ui/badge";
import { AddressChip } from "@/components/AddressChip";

// EventSource connects to the backend SSE endpoint. The backend streams
// progress events (slot_observed → account_fetched* → frame_completed* → done)
// so the user can watch the trace assemble in real time instead of waiting on
// a single blocking POST /replay response.
//
// We never call new EventSource() during SSR — the component is "use client"
// and the effect only runs in the browser.

interface LiveReplayPanelProps {
  signature: string;
  /** Resolved by web/lib/api.ts BASE — usually "/rpc". */
  basePath?: string;
}

interface AccountRow {
  pubkey: string;
  size: number;
  is_program: boolean;
  fetchedAtMs: number;
}

interface PanelState {
  source: LiveSource | null;
  slot: number | null;
  blockTime: number | null;
  accounts: AccountRow[];
  totalAccounts: number | null;
  executionStarted: boolean;
  frames: CpiFrame[];
  finalTrace: Trace | null;
  error: { code: string; message: string } | null;
  startedAtMs: number;
  doneAtMs: number | null;
}

const initialState: PanelState = {
  source: null,
  slot: null,
  blockTime: null,
  accounts: [],
  totalAccounts: null,
  executionStarted: false,
  frames: [],
  finalTrace: null,
  error: null,
  startedAtMs: 0,
  doneAtMs: null,
};

export function LiveReplayPanel({ signature, basePath = "/rpc" }: LiveReplayPanelProps) {
  const [state, setState] = useState<PanelState>(() => ({
    ...initialState,
    startedAtMs: typeof window !== "undefined" ? performance.now() : 0,
  }));
  const esRef = useRef<EventSource | null>(null);

  useEffect(() => {
    setState({ ...initialState, startedAtMs: performance.now() });
    const url = `${basePath}/replay-live/${encodeURIComponent(signature)}`;
    const es = new EventSource(url);
    esRef.current = es;

    es.onmessage = (msg) => {
      let ev: LiveReplayEvent;
      try {
        ev = JSON.parse(msg.data) as LiveReplayEvent;
      } catch {
        return;
      }
      setState((prev) => reduce(prev, ev));
      // Close the stream once we've reached a terminal event so the
      // browser doesn't keep the connection open and the backend's
      // live-session slot is freed promptly.
      if (ev.type === "done" || ev.type === "error") {
        es.close();
      }
    };

    es.onerror = () => {
      // EventSource auto-reconnects on transport errors; we only surface
      // an error if the stream never delivered a `mode` event (i.e. the
      // initial handshake failed — rate-limited, bad sig, etc.).
      setState((prev) =>
        prev.source === null
          ? {
              ...prev,
              error: {
                code: "STREAM_FAILED",
                message:
                  "Could not open the live stream. The endpoint may be rate-limited or the signature invalid.",
              },
              doneAtMs: performance.now(),
            }
          : prev
      );
      es.close();
    };

    return () => {
      es.close();
      esRef.current = null;
    };
  }, [signature, basePath]);

  const elapsedMs = useMemo(() => {
    const end = state.doneAtMs ?? performance.now();
    return Math.round(end - state.startedAtMs);
  }, [state.doneAtMs, state.startedAtMs]);

  const isDone = state.finalTrace !== null;
  const isError = state.error !== null;

  return (
    <div className="flex flex-col h-full overflow-hidden bg-zinc-950">
      <header className="flex items-center gap-3 px-4 py-2 border-b border-zinc-800 shrink-0">
        <span className="text-xs uppercase tracking-widest text-zinc-500">Live replay</span>
        <SourceBadge source={state.source} />
        {state.slot !== null && (
          <span className="text-xs text-zinc-500">slot {state.slot.toLocaleString()}</span>
        )}
        <span className="ml-auto text-xs text-zinc-500 tabular-nums">{elapsedMs} ms</span>
        {isDone && <Badge variant="outline" className="text-xs">done</Badge>}
        {isError && <Badge variant="destructive" className="text-xs">{state.error?.code}</Badge>}
      </header>

      {state.source === "rpc" && (
        <div className="px-4 py-2 text-[11px] text-zinc-500 border-b border-zinc-900 leading-relaxed">
          LaserStream not configured on this server — events sourced from
          standard Helius RPC. Set <code className="text-zinc-300">LASERSTREAM_GRPC_URL</code> +{" "}
          <code className="text-zinc-300">LASERSTREAM_X_TOKEN</code> on the API host to enable
          true gRPC streaming.
        </div>
      )}

      {isError && (
        <div className="m-4 p-3 border border-red-900/60 bg-red-950/40 rounded text-xs text-red-300">
          <div className="font-mono">{state.error?.code}</div>
          <div className="mt-1">{state.error?.message}</div>
        </div>
      )}

      <div className="flex-1 overflow-hidden grid grid-cols-2 divide-x divide-zinc-900">
        {/* Left: streaming account list */}
        <section className="overflow-y-auto">
          <SectionHeader>
            Accounts fetched{" "}
            <span className="text-zinc-500 normal-case tracking-normal font-mono ml-1">
              {state.accounts.length}
              {state.totalAccounts !== null && ` / ${state.totalAccounts}`}
            </span>
          </SectionHeader>
          <ul className="px-2 py-1 space-y-0.5">
            {state.accounts.map((a, i) => (
              <li
                key={`${a.pubkey}-${i}`}
                className="flex items-center gap-2 px-2 py-1 rounded hover:bg-zinc-900/40 text-xs"
              >
                <span className="text-zinc-600 font-mono w-6 text-right">{i + 1}</span>
                <AddressChip address={a.pubkey} />
                {a.is_program && (
                  <Badge variant="secondary" className="text-[10px] h-4 px-1.5">
                    program
                  </Badge>
                )}
                <span className="ml-auto text-zinc-500 tabular-nums">{a.size}B</span>
              </li>
            ))}
            {state.accounts.length === 0 && !isError && (
              <li className="px-2 py-2 text-xs text-zinc-600">Waiting for first account…</li>
            )}
          </ul>
        </section>

        {/* Right: streaming frame list */}
        <section className="overflow-y-auto">
          <SectionHeader>
            Frames executed{" "}
            <span className="text-zinc-500 normal-case tracking-normal font-mono ml-1">
              {state.frames.length}
            </span>
            {state.executionStarted && !isDone && (
              <span className="ml-2 inline-block w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
            )}
          </SectionHeader>
          <ul className="px-2 py-1 space-y-0.5">
            {state.frames.map((f, i) => (
              <li
                key={`frame-${i}`}
                className="flex items-center gap-2 px-2 py-1 rounded hover:bg-zinc-900/40 text-xs"
              >
                <span className="text-zinc-600 font-mono w-6 text-right">{i + 1}</span>
                <span className="text-zinc-300 truncate">
                  {f.program_name ?? `${f.program_id.slice(0, 4)}…${f.program_id.slice(-4)}`}
                </span>
                {f.instruction_name && (
                  <span className="text-zinc-500 truncate">{f.instruction_name}</span>
                )}
                <span className="ml-auto text-zinc-500 tabular-nums">
                  {f.cu_consumed.toLocaleString()} CU
                </span>
              </li>
            ))}
            {state.frames.length === 0 && state.executionStarted && (
              <li className="px-2 py-2 text-xs text-zinc-600">Executing…</li>
            )}
            {!state.executionStarted && (
              <li className="px-2 py-2 text-xs text-zinc-600">
                Waiting for accounts to finish…
              </li>
            )}
          </ul>
        </section>
      </div>

      {isDone && state.finalTrace && (
        <footer className="border-t border-zinc-900 px-4 py-2 flex items-center gap-3 text-xs text-zinc-400">
          <span>Total CU</span>
          <span className="font-mono text-zinc-200">
            {state.finalTrace.total_cu.toLocaleString()}
          </span>
          <span className="ml-4">Result</span>
          <span className="font-mono text-zinc-200">
            {state.finalTrace.replay_result.status}
          </span>
        </footer>
      )}
    </div>
  );
}

function SectionHeader({ children }: { children: React.ReactNode }) {
  return (
    <div className="px-3 py-2 text-[10px] uppercase tracking-widest text-zinc-600 border-b border-zinc-900 sticky top-0 bg-zinc-950 z-10">
      {children}
    </div>
  );
}

function SourceBadge({ source }: { source: LiveSource | null }) {
  if (source === null) {
    return (
      <Badge variant="secondary" className="text-[10px]">
        connecting…
      </Badge>
    );
  }
  if (source === "laserstream") {
    return (
      <Badge variant="outline" className="text-[10px] border-emerald-700 text-emerald-300">
        LaserStream
      </Badge>
    );
  }
  return (
    <Badge variant="secondary" className="text-[10px]">
      RPC fallback
    </Badge>
  );
}

function reduce(prev: PanelState, ev: LiveReplayEvent): PanelState {
  switch (ev.type) {
    case "mode":
      return { ...prev, source: ev.source };
    case "slot_observed":
      return { ...prev, slot: ev.slot, blockTime: ev.block_time };
    case "account_fetched":
      return {
        ...prev,
        accounts: [
          ...prev.accounts,
          {
            pubkey: ev.pubkey,
            size: ev.size,
            is_program: ev.is_program,
            fetchedAtMs: performance.now(),
          },
        ],
      };
    case "all_accounts_fetched":
      return { ...prev, totalAccounts: ev.count };
    case "execution_started":
      return { ...prev, executionStarted: true };
    case "frame_completed":
      return { ...prev, frames: [...prev.frames, ev.frame] };
    case "done":
      return { ...prev, finalTrace: ev.trace, doneAtMs: performance.now() };
    case "error":
      return {
        ...prev,
        error: { code: ev.code, message: ev.message },
        doneAtMs: performance.now(),
      };
  }
}
