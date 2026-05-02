"use client";
import type { CpiFrame } from "@/lib/types";
import { useReplayStore } from "@/store/replay-store";
import { AddressChip } from "./AddressChip";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const KNOWN_NAMES: Record<string, string> = {
  JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4: "Jupiter v6",
  whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc: "Whirlpool",
  ComputeBudget111111111111111111111111111111: "ComputeBudget",
  "11111111111111111111111111111111": "System",
  TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA: "SPL Token",
};

function findFrame(frames: CpiFrame[], id: string): CpiFrame | null {
  for (const f of frames) {
    const fid = `${f.depth}-${f.instruction_index}-${f.program_id}`;
    if (fid === id) return f;
    const child = findFrame(f.children, id);
    if (child) return child;
  }
  return null;
}

export function FrameDetail() {
  const { currentTrace, selectedFrameId, setSelectedAccount } = useReplayStore();

  if (!currentTrace || !selectedFrameId) {
    return (
      <div className="flex items-center justify-center h-full text-zinc-500 text-sm">
        Select a frame from the tree
      </div>
    );
  }

  const frame = findFrame(currentTrace.frames, selectedFrameId);
  if (!frame) return null;

  const progName = frame.program_name ?? KNOWN_NAMES[frame.program_id];
  const success = frame.result.status === "success";

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="px-4 py-3 border-b border-zinc-800">
        <div className="flex items-center gap-2 flex-wrap">
          <h2 className="font-semibold text-sm">
            {frame.instruction_name ?? progName ?? "Unknown instruction"}
          </h2>
          <Badge variant={success ? "outline" : "destructive"} className="text-xs">
            {success ? "success" : "failed"}
          </Badge>
          <span className="text-xs text-zinc-500">
            {frame.cu_consumed.toLocaleString()} CU
          </span>
        </div>
        <div className="flex items-center gap-2 mt-1">
          <span className="text-xs text-zinc-500">Program:</span>
          <AddressChip address={frame.program_id} />
          {progName && <span className="text-xs text-zinc-400">{progName}</span>}
        </div>
        {!success && frame.result.status === "failure" && (
          <p className="mt-1 text-xs text-red-400">{frame.result.error}</p>
        )}
      </div>

      <Tabs defaultValue="logs" className="flex-1 flex flex-col overflow-hidden">
        <TabsList className="mx-4 mt-2 w-fit">
          <TabsTrigger value="logs">Logs</TabsTrigger>
          <TabsTrigger value="accounts">Accounts ({frame.accounts.length})</TabsTrigger>
          {frame.decoded_args && <TabsTrigger value="args">Args</TabsTrigger>}
        </TabsList>

        <TabsContent value="logs" className="flex-1 overflow-hidden mx-0 mt-0">
          <ScrollArea className="h-full px-4 py-2">
            {frame.logs.length === 0 ? (
              <p className="text-zinc-500 text-xs">No logs.</p>
            ) : (
              <pre className="text-xs font-mono whitespace-pre-wrap text-zinc-300 space-y-0.5">
                {frame.logs.map((l, i) => <div key={i}>{l}</div>)}
              </pre>
            )}
          </ScrollArea>
        </TabsContent>

        <TabsContent value="accounts" className="flex-1 overflow-hidden mx-0 mt-0">
          <ScrollArea className="h-full px-4 py-2">
            <div className="space-y-2">
              {frame.accounts.map((acc, i) => (
                <div
                  key={i}
                  className="flex items-center gap-2 text-xs cursor-pointer hover:text-white text-zinc-300"
                  onClick={() => setSelectedAccount(acc.pubkey)}
                >
                  <span className="text-zinc-500 w-5 text-right shrink-0">{i}</span>
                  <AddressChip address={acc.pubkey} />
                  {acc.role && <span className="text-zinc-400 font-mono">{acc.role}</span>}
                  <div className="flex gap-1 ml-auto">
                    {acc.is_writable && <Badge variant="outline" className="text-[10px] px-1 py-0">W</Badge>}
                    {acc.is_signer && <Badge variant="outline" className="text-[10px] px-1 py-0">S</Badge>}
                  </div>
                </div>
              ))}
            </div>
          </ScrollArea>
        </TabsContent>

        {frame.decoded_args && (
          <TabsContent value="args" className="flex-1 overflow-hidden mx-0 mt-0">
            <ScrollArea className="h-full px-4 py-2">
              <pre className="text-xs font-mono whitespace-pre-wrap text-zinc-300">
                {JSON.stringify(frame.decoded_args, null, 2)}
              </pre>
            </ScrollArea>
          </TabsContent>
        )}
      </Tabs>
    </div>
  );
}
