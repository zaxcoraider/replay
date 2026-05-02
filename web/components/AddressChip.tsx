"use client";
import { useState } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

interface Props {
  address: string;
  className?: string;
}

export function AddressChip({ address, className }: Props) {
  const [copied, setCopied] = useState(false);
  const short = `${address.slice(0, 6)}…${address.slice(-6)}`;

  const copy = () => {
    navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <Tooltip>
      <TooltipTrigger>
        <button
          onClick={copy}
          className={`font-mono text-xs px-1.5 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 transition-colors ${className ?? ""}`}
        >
          {copied ? "copied!" : short}
        </button>
      </TooltipTrigger>
      <TooltipContent side="top" className="font-mono text-xs">
        {address}
      </TooltipContent>
    </Tooltip>
  );
}
