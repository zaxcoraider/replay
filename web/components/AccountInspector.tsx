"use client";
import { useReplayStore } from "@/store/replay-store";
import { AddressChip } from "./AddressChip";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const LAMPORTS_PER_SOL = 1_000_000_000;

export function AccountInspector() {
  const { currentTrace, selectedAccountPubkey } = useReplayStore();

  if (!selectedAccountPubkey) {
    return (
      <div className="flex items-center justify-center h-full text-zinc-500 text-sm text-center p-4">
        Click an account to inspect
      </div>
    );
  }

  const delta = currentTrace?.account_deltas.find(
    (d) => d.pubkey === selectedAccountPubkey
  );

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="px-3 py-2 border-b border-zinc-800">
        <div className="flex items-center gap-2">
          <span className="text-xs text-zinc-500">Account</span>
          <AddressChip address={selectedAccountPubkey} />
        </div>
        {delta && (
          <div className="mt-1 flex flex-wrap gap-2 text-xs text-zinc-400">
            <span>Owner: <AddressChip address={delta.owner_after} /></span>
            <span>
              {(delta.lamports_after / LAMPORTS_PER_SOL).toFixed(4)} SOL
              {delta.lamports_after !== delta.lamports_before && (
                <span className={delta.lamports_after > delta.lamports_before ? " text-green-400" : " text-red-400"}>
                  {" "}({delta.lamports_after > delta.lamports_before ? "+" : ""}
                  {((delta.lamports_after - delta.lamports_before) / LAMPORTS_PER_SOL).toFixed(4)})
                </span>
              )}
            </span>
            {delta.idl_type_name && (
              <Badge variant="outline" className="text-[10px]">{delta.idl_type_name}</Badge>
            )}
          </div>
        )}
      </div>

      {delta ? (
        <Tabs defaultValue="decoded" className="flex-1 flex flex-col overflow-hidden">
          <TabsList className="mx-3 mt-2 w-fit">
            <TabsTrigger value="decoded">Decoded</TabsTrigger>
            <TabsTrigger value="raw">Raw hex</TabsTrigger>
          </TabsList>

          <TabsContent value="decoded" className="flex-1 overflow-hidden mx-0 mt-0">
            <ScrollArea className="h-full px-3 py-2">
              {delta.decoded_after ? (
                <pre className="text-xs font-mono whitespace-pre-wrap text-zinc-300">
                  {JSON.stringify(delta.decoded_after, null, 2)}
                </pre>
              ) : (
                <p className="text-zinc-500 text-xs">No IDL — account not decodable.</p>
              )}
            </ScrollArea>
          </TabsContent>

          <TabsContent value="raw" className="flex-1 overflow-hidden mx-0 mt-0">
            <ScrollArea className="h-full px-3 py-2">
              <pre className="text-xs font-mono break-all text-zinc-400 whitespace-pre-wrap">
                {delta.data_after_hex}
              </pre>
            </ScrollArea>
          </TabsContent>
        </Tabs>
      ) : (
        <div className="flex-1 flex items-center justify-center text-zinc-500 text-xs">
          Account not modified in this transaction
        </div>
      )}
    </div>
  );
}
