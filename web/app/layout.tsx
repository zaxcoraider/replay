import type { Metadata } from "next";
import { Geist_Mono } from "next/font/google";
import { TooltipProvider } from "@/components/ui/tooltip";
import { KeepAlive } from "@/components/KeepAlive";
import "./globals.css";

const mono = Geist_Mono({ variable: "--font-mono", subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Replay — Solana time-travel debugger",
  description: "Replay any Solana transaction against exact historical state. Fork, mutate, re-run, diff.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${mono.variable} h-full dark antialiased`}>
      <body className="min-h-full bg-zinc-950 text-zinc-100 font-mono flex flex-col">
        <KeepAlive />
        <TooltipProvider delay={300}>
          {children}
        </TooltipProvider>
      </body>
    </html>
  );
}
