"use client";
import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import Image from "next/image";
import { Badge } from "@/components/ui/badge";
import { DEMOS, isPlaceholder } from "@/lib/demo-signatures";
import { GitFork, Zap, Search, BarChart3, ArrowRight, Play } from "lucide-react";

const FEATURES = [
  { icon: GitFork, label: "Fork sessions" },
  { icon: Search, label: "IDL-decoded fields" },
  { icon: BarChart3, label: "CPI trace tree" },
  { icon: Zap, label: "Side-by-side diff" },
];

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
    <main className="relative flex flex-col items-center min-h-screen px-4 py-16 gap-12 max-w-3xl mx-auto">
      {/* Animated background */}
      <div className="fixed inset-0 -z-10 bg-zinc-950 bg-grid" />
      <div className="fixed inset-0 -z-10 bg-hero-glow pointer-events-none" />
      {/* Bottom fade */}
      <div className="fixed bottom-0 inset-x-0 -z-10 h-48 bg-gradient-to-t from-zinc-950 to-transparent pointer-events-none" />

      {/* Hero */}
      <div className="text-center space-y-5 w-full pt-4 animate-fade-up">
        {/* Logo */}
        <div className="flex justify-center">
          <Image
            src="/logo.png"
            alt="Replay logo"
            width={80}
            height={80}
            className="rounded-2xl"
            priority
          />
        </div>

        {/* Eyebrow */}
        <div className="inline-flex items-center gap-2 text-[11px] uppercase tracking-[0.3em] text-zinc-500 bg-zinc-900/80 border border-zinc-800 rounded-full px-4 py-1.5">
          <span className="w-1.5 h-1.5 rounded-full bg-[#9945FF] animate-pulse" />
          Solana developer tooling
        </div>

        {/* Title */}
        <h1 className="text-6xl sm:text-7xl font-bold tracking-tight">
          <span
            className="bg-clip-text text-transparent"
            style={{
              backgroundImage: "linear-gradient(135deg, #ffffff 0%, #e4e4e7 50%, #a1a1aa 100%)",
            }}
          >
            Replay
          </span>
        </h1>

        <p className="text-zinc-400 text-base max-w-lg mx-auto leading-relaxed">
          Time-travel debugger for Solana. Replay any transaction against exact
          historical state, mutate account fields, re-run, and see the diff.
        </p>

        {/* Feature pills */}
        <div className="flex flex-wrap justify-center gap-2 pt-1">
          {FEATURES.map(({ icon: Icon, label }) => (
            <span
              key={label}
              className="inline-flex items-center gap-1.5 text-[11px] px-3 py-1 rounded-full border border-zinc-800 text-zinc-400 bg-zinc-900/60 hover:border-[#9945FF]/50 hover:text-zinc-200 hover:bg-zinc-900 transition-all duration-200 cursor-default"
            >
              <Icon size={11} className="text-[#9945FF]" />
              {label}
            </span>
          ))}
        </div>
      </div>

      {/* Signature input */}
      <div className="w-full space-y-2.5 animate-fade-up" style={{ animationDelay: "0.08s" }}>
        <div className="flex gap-2">
          <input
            value={sig}
            onChange={(e) => setSig(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && go(sig)}
            placeholder="Paste any Solana transaction signature…"
            disabled={loading}
            autoFocus
            className="flex-1 h-12 px-4 rounded-md font-mono text-sm bg-zinc-900/80 border border-zinc-700 text-zinc-100 placeholder:text-zinc-600 outline-none transition-all duration-200 focus:border-[#9945FF]/50 focus:shadow-[0_0_0_3px_rgba(153,69,255,0.12)] disabled:opacity-50"
          />
          <button
            onClick={() => go(sig)}
            disabled={loading || !sig.trim() || isPlaceholder(sig.trim())}
            className="shrink-0 h-12 px-6 rounded-md text-sm font-semibold text-white flex items-center gap-2 transition-all duration-200 disabled:opacity-40 disabled:cursor-not-allowed hover:brightness-110 active:scale-95"
            style={{
              background: "linear-gradient(135deg, #9945FF 0%, #7d2be8 100%)",
              boxShadow: "0 0 24px rgba(153,69,255,0.3), 0 1px 0 rgba(255,255,255,0.08) inset",
            }}
          >
            {loading ? (
              <>
                <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Loading…
              </>
            ) : (
              <>
                <Play size={13} />
                Replay
              </>
            )}
          </button>
        </div>
        <p className="text-[11px] text-zinc-600 text-center">
          Requires a running Replay API with a Helius key.{" "}
          <a
            href="https://github.com/zaxcoraider/replay"
            target="_blank"
            rel="noreferrer"
            className="text-zinc-500 underline underline-offset-2 hover:text-zinc-300 transition-colors"
          >
            Self-host in 2 minutes →
          </a>
        </p>
      </div>

      {/* Demo cards */}
      <div className="w-full space-y-3 animate-fade-up" style={{ animationDelay: "0.14s" }}>
        <div className="flex items-center gap-3">
          <p className="text-[10px] uppercase tracking-widest text-zinc-600">Try a demo</p>
          <div className="flex-1 h-px bg-zinc-800" />
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          {DEMOS.map((d) => {
            const ready = !isPlaceholder(d.signature);
            return (
              <button
                key={d.id}
                onClick={() => ready && go(d.signature)}
                disabled={loading || !ready}
                className={[
                  "group text-left rounded-xl border px-4 py-4 space-y-2.5 transition-all duration-200",
                  ready
                    ? "border-zinc-800 bg-zinc-900/20 hover:border-[#9945FF]/40 hover:bg-zinc-900/60 hover:shadow-[0_0_24px_rgba(153,69,255,0.06)] cursor-pointer"
                    : "border-zinc-800/50 opacity-40 cursor-not-allowed",
                ].join(" ")}
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
                  <div className="font-semibold text-sm text-zinc-200 group-hover:text-white transition-colors">
                    {d.title}
                  </div>
                  <div className="text-[11px] text-zinc-500 mt-0.5">{d.subtitle}</div>
                </div>

                {/* Narrative */}
                <p className="text-[11px] text-zinc-600 leading-relaxed line-clamp-3">
                  {d.narrative}
                </p>

                {/* Mutation hint + arrow */}
                <div className="pt-1.5 border-t border-zinc-800/80 flex items-center justify-between gap-2">
                  <p className="text-[10px] text-zinc-600 leading-snug">
                    <span className="text-zinc-500">Suggested: </span>
                    {d.suggested_mutation.description}
                  </p>
                  {ready && (
                    <ArrowRight
                      size={12}
                      className="text-zinc-700 group-hover:text-[#9945FF] transition-colors shrink-0"
                    />
                  )}
                </div>

                {!ready && (
                  <p className="text-[10px] text-yellow-600/70">
                    Sig needed — see demo-signatures.ts
                  </p>
                )}
              </button>
            );
          })}
        </div>
      </div>

      {/* Footer */}
      <footer className="text-[11px] text-zinc-700 text-center pb-4 animate-fade-up" style={{ animationDelay: "0.2s" }}>
        <p className="flex items-center justify-center gap-2 flex-wrap">
          Built for{" "}
          <a
            href="https://frontier.colosseum.org"
            target="_blank"
            rel="noreferrer"
            className="text-zinc-600 hover:text-zinc-400 underline underline-offset-2 transition-colors"
          >
            Colosseum Frontier 2026
          </a>
          · MIT licensed ·{" "}
          <a
            href="https://github.com/zaxcoraider/replay"
            target="_blank"
            rel="noreferrer"
            className="text-zinc-600 hover:text-zinc-400 transition-colors"
          >
            GitHub
          </a>{" "}
          ·{" "}
          <Link
            href="/docs"
            className="text-zinc-600 hover:text-zinc-400 underline underline-offset-2 transition-colors"
          >
            Docs
          </Link>
        </p>
      </footer>
    </main>
  );
}
