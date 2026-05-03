"use client";
import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { DEMOS, isPlaceholder } from "@/lib/demo-signatures";

export default function HomePage() {
  const [sig, setSig] = useState("");
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  const go = (s: string) => {
    const trimmed = (s || sig).trim();
    if (!trimmed || isPlaceholder(trimmed)) return;
    setLoading(true);
    router.push(`/replay/${encodeURIComponent(trimmed)}`);
  };

  return (
    <main className="flex flex-col items-center min-h-screen px-4 py-16 gap-14 max-w-3xl mx-auto">
      {/* Hero */}
      <div className="text-center space-y-4 w-full">
        <div className="text-[10px] uppercase tracking-[0.4em] text-zinc-600">
          Solana developer tooling
        </div>
        <h1 className="text-5xl font-bold tracking-tight bg-gradient-to-b from-white to-zinc-400 bg-clip-text text-transparent">
          Replay
        </h1>
        <p className="text-zinc-400 text-base max-w-lg mx-auto leading-relaxed">
          Time-travel debugger for Solana. Replay any transaction against exact
          historical state, mutate account fields, re-run, and see the diff.
        </p>

        {/* Feature pills */}
        <div className="flex flex-wrap justify-center gap-2 pt-1">
          {["Fork sessions", "IDL-decoded fields", "CPI trace tree", "Side-by-side diff"].map(
            (f) => (
              <span
                key={f}
                className="text-[11px] px-2.5 py-0.5 rounded-full border border-zinc-700 text-zinc-400"
              >
                {f}
              </span>
            )
          )}
        </div>
      </div>

      {/* Signature input */}
      <div className="w-full space-y-2">
        <div className="flex gap-2">
          <Input
            value={sig}
            onChange={(e) => setSig(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && go(sig)}
            placeholder="Paste any Solana transaction signature…"
            className="font-mono bg-zinc-900 border-zinc-700 text-sm focus:border-zinc-500 h-11"
            disabled={loading}
            autoFocus
          />
          <Button
            onClick={() => go(sig)}
            disabled={loading || !sig.trim() || isPlaceholder(sig.trim())}
            className="shrink-0 h-11 px-6"
          >
            {loading ? (
              <span className="flex items-center gap-2">
                <span className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Loading…
              </span>
            ) : (
              "Replay →"
            )}
          </Button>
        </div>
        <p className="text-[11px] text-zinc-600 text-center">
          Requires a running Replay API with a Helius key.{" "}
          <a
            href="https://github.com/zaxcoraider/replay"
            target="_blank"
            rel="noreferrer"
            className="text-zinc-500 underline underline-offset-2 hover:text-zinc-300"
          >
            Self-host in 2 minutes →
          </a>
        </p>
      </div>

      {/* Demo cards */}
      <div className="w-full space-y-3">
        <p className="text-[10px] uppercase tracking-widest text-zinc-600">
          Try a demo scenario
        </p>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          {DEMOS.map((d) => {
            const ready = !isPlaceholder(d.signature);
            return (
              <button
                key={d.id}
                onClick={() => ready && go(d.signature)}
                disabled={loading || !ready}
                className={`text-left rounded-xl border px-4 py-4 transition-all space-y-2 ${
                  ready
                    ? "border-zinc-700 hover:border-zinc-500 hover:bg-zinc-900/60 cursor-pointer"
                    : "border-zinc-800 opacity-50 cursor-not-allowed"
                }`}
              >
                {/* Tags */}
                <div className="flex flex-wrap gap-1">
                  {d.tags.map((t) => (
                    <Badge
                      key={t}
                      variant="outline"
                      className="text-[9px] px-1.5 py-0 border-zinc-700 text-zinc-500"
                    >
                      {t}
                    </Badge>
                  ))}
                </div>

                {/* Title */}
                <div>
                  <div className="font-semibold text-sm text-zinc-200">{d.title}</div>
                  <div className="text-[11px] text-zinc-500 mt-0.5">{d.subtitle}</div>
                </div>

                {/* Narrative */}
                <p className="text-[11px] text-zinc-600 leading-relaxed line-clamp-3">
                  {d.narrative}
                </p>

                {/* Mutation hint */}
                <div className="pt-1 border-t border-zinc-800">
                  <p className="text-[10px] text-zinc-600">
                    <span className="text-zinc-500">Suggested: </span>
                    {d.suggested_mutation.description}
                  </p>
                </div>

                {!ready && (
                  <p className="text-[10px] text-yellow-600">
                    Sig needed — see demo-signatures.ts
                  </p>
                )}
              </button>
            );
          })}
        </div>
      </div>

      {/* Footer */}
      <div className="text-[11px] text-zinc-700 text-center space-y-1 pb-4">
        <p>
          Built for{" "}
          <a
            href="https://frontier.colosseum.org"
            target="_blank"
            rel="noreferrer"
            className="text-zinc-600 hover:text-zinc-400 underline underline-offset-2"
          >
            Colosseum Frontier 2026
          </a>{" "}
          · MIT licensed ·{" "}
          <a
            href="https://github.com/zaxcoraider/replay"
            target="_blank"
            rel="noreferrer"
            className="text-zinc-600 hover:text-zinc-400 underline underline-offset-2"
          >
            GitHub
          </a>
          {" "}·{" "}
          <Link
            href="/docs"
            className="text-zinc-600 hover:text-zinc-400 underline underline-offset-2"
          >
            Docs
          </Link>
        </p>
      </div>
    </main>
  );
}
