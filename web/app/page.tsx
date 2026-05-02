"use client";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

const DEMOS = [
  {
    label: "Jupiter swap",
    sig: "replace_with_real_sig",
    description: "Route through multiple pools — mutate fee_rate, watch it fail",
  },
  {
    label: "SPL Token transfer",
    sig: "replace_with_real_sig_2",
    description: "Simple token transfer — inspect pre/post balances",
  },
];

export default function HomePage() {
  const [sig, setSig] = useState("");
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  const go = (s: string) => {
    const trimmed = (s || sig).trim();
    if (!trimmed) return;
    setLoading(true);
    router.push(`/replay/${encodeURIComponent(trimmed)}`);
  };

  return (
    <main className="flex flex-col items-center justify-center min-h-screen px-4 gap-10">
      {/* Hero */}
      <div className="text-center space-y-3">
        <div className="text-xs uppercase tracking-[0.3em] text-zinc-500">Solana</div>
        <h1 className="text-4xl font-bold tracking-tight">Replay</h1>
        <p className="text-zinc-400 text-sm max-w-md leading-relaxed">
          Time-travel debugger for Solana transactions. Paste a signature, replay
          it against exact historical state, fork, mutate, re-run, diff.
        </p>
      </div>

      {/* Input */}
      <div className="w-full max-w-xl flex gap-2">
        <Input
          value={sig}
          onChange={(e) => setSig(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && go(sig)}
          placeholder="Enter transaction signature…"
          className="font-mono bg-zinc-900 border-zinc-700 text-sm focus:border-zinc-500"
          disabled={loading}
          autoFocus
        />
        <Button onClick={() => go(sig)} disabled={loading || !sig.trim()} className="shrink-0">
          {loading ? "Loading…" : "Replay →"}
        </Button>
      </div>

      {/* Demo buttons */}
      <div className="w-full max-w-xl space-y-2">
        <p className="text-[10px] uppercase tracking-widest text-zinc-600">Try a demo</p>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
          {DEMOS.map((d) => (
            <button
              key={d.label}
              onClick={() => { setSig(d.sig); go(d.sig); }}
              disabled={loading || d.sig.startsWith("replace_")}
              className="text-left rounded-lg border border-zinc-800 px-4 py-3 hover:border-zinc-600 hover:bg-zinc-900/50 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <div className="font-semibold text-sm">{d.label}</div>
              <div className="text-xs text-zinc-500 mt-0.5">{d.description}</div>
            </button>
          ))}
        </div>
        <p className="text-[10px] text-zinc-600">Demo signatures filled once Day-10 runs.</p>
      </div>
    </main>
  );
}
