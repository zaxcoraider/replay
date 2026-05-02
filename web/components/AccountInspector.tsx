"use client";
import { useState, useCallback } from "react";
import { useReplayStore } from "@/store/replay-store";
import { AddressChip } from "./AddressChip";
import { FieldEditor } from "./FieldEditor";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { mutate } from "@/lib/api";

const LAMPORTS_PER_SOL = 1_000_000_000;
const RENT_EXEMPT_FLOOR = 890_880;

function getDecodedValue(
  decoded: unknown
): { typeName: string; value: Record<string, unknown> } | null {
  if (!decoded || typeof decoded !== "object") return null;
  const d = decoded as Record<string, unknown>;
  if (d.kind !== "decoded" || !d.value || typeof d.value !== "object") return null;
  return {
    typeName: d.type_name as string,
    value: d.value as Record<string, unknown>,
  };
}

export function AccountInspector() {
  const { currentTrace, selectedAccountPubkey, sessionId, addPendingMutation } =
    useReplayStore();

  const [lamports, setLamports] = useState("");
  const [owner, setOwner] = useState("");
  const [fieldEdits, setFieldEdits] = useState<Record<string, unknown>>({});
  const [applying, setApplying] = useState(false);
  const [applyError, setApplyError] = useState<string | null>(null);

  const delta = currentTrace?.account_deltas.find(
    (d) => d.pubkey === selectedAccountPubkey
  );
  const decodedInfo = getDecodedValue(delta?.decoded_after);
  const modifiedPaths = new Set(Object.keys(fieldEdits));

  const mutationCount =
    (lamports.trim() ? 1 : 0) +
    (owner.trim() ? 1 : 0) +
    Object.keys(fieldEdits).length;
  const hasEdits = mutationCount > 0;

  const handleFieldChange = useCallback((path: string, newValue: unknown) => {
    setFieldEdits((prev) => ({ ...prev, [path]: newValue }));
  }, []);

  const discardEdits = () => {
    setLamports("");
    setOwner("");
    setFieldEdits({});
    setApplyError(null);
  };

  const applyEdits = async () => {
    if (!sessionId || !selectedAccountPubkey) return;
    setApplying(true);
    setApplyError(null);
    try {
      if (lamports.trim()) {
        const v = parseInt(lamports, 10);
        if (!isNaN(v)) {
          await mutate(sessionId, selectedAccountPubkey, { type: "lamports", new_value: v });
          addPendingMutation(selectedAccountPubkey, `lamports → ${v.toLocaleString()}`);
        }
      }
      if (owner.trim()) {
        await mutate(sessionId, selectedAccountPubkey, {
          type: "owner",
          new_value: owner.trim(),
        });
        addPendingMutation(
          selectedAccountPubkey,
          `owner → ${owner.trim().slice(0, 8)}…`
        );
      }
      for (const [path, newValue] of Object.entries(fieldEdits)) {
        await mutate(sessionId, selectedAccountPubkey, {
          type: "field",
          path,
          new_value: newValue,
        });
        addPendingMutation(
          selectedAccountPubkey,
          `${path} → ${String(newValue).slice(0, 16)}`
        );
      }
      discardEdits();
    } catch (e) {
      setApplyError((e as Error).message);
    } finally {
      setApplying(false);
    }
  };

  if (!selectedAccountPubkey) {
    return (
      <div className="flex items-center justify-center h-full text-zinc-500 text-sm text-center p-4">
        Click an account to inspect
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Account header */}
      <div className="px-3 py-2 border-b border-zinc-800 shrink-0">
        <div className="flex items-center gap-2">
          <span className="text-xs text-zinc-500">Account</span>
          <AddressChip address={selectedAccountPubkey} />
        </div>
        {delta && (
          <div className="mt-1 flex flex-wrap gap-2 text-xs text-zinc-400">
            <span>
              Owner: <AddressChip address={delta.owner_after} />
            </span>
            <span>
              {(delta.lamports_after / LAMPORTS_PER_SOL).toFixed(4)} SOL
              {delta.lamports_after !== delta.lamports_before && (
                <span
                  className={
                    delta.lamports_after > delta.lamports_before
                      ? " text-green-400"
                      : " text-red-400"
                  }
                >
                  {" "}
                  ({delta.lamports_after > delta.lamports_before ? "+" : ""}
                  {(
                    (delta.lamports_after - delta.lamports_before) /
                    LAMPORTS_PER_SOL
                  ).toFixed(4)}
                  )
                </span>
              )}
            </span>
            {decodedInfo && (
              <Badge variant="outline" className="text-[10px]">
                {decodedInfo.typeName}
              </Badge>
            )}
          </div>
        )}
      </div>

      {/* Lamports + owner mutation fields (only when session is active) */}
      {sessionId && delta && (
        <div className="px-3 pt-2 pb-2 border-b border-zinc-800 shrink-0 space-y-1.5">
          <div className="flex items-center gap-2">
            <span className="text-[11px] text-zinc-500 w-14 shrink-0">Lamports</span>
            <Input
              type="number"
              placeholder={delta.lamports_after.toString()}
              value={lamports}
              onChange={(e) => setLamports(e.target.value)}
              className={`h-6 text-xs font-mono py-0 px-1.5 bg-zinc-900 flex-1 ${
                lamports.trim() ? "border-yellow-600" : "border-zinc-700"
              }`}
            />
          </div>
          {lamports.trim() && parseInt(lamports, 10) < RENT_EXEMPT_FLOOR && (
            <p className="text-[10px] text-yellow-500">
              ⚠ Below rent-exempt minimum — account may be garbage collected.
            </p>
          )}
          <div className="flex items-center gap-2">
            <span className="text-[11px] text-zinc-500 w-14 shrink-0">Owner</span>
            <Input
              placeholder={delta.owner_after.slice(0, 20) + "…"}
              value={owner}
              onChange={(e) => setOwner(e.target.value)}
              className={`h-6 text-xs font-mono py-0 px-1.5 bg-zinc-900 flex-1 ${
                owner.trim() ? "border-yellow-600" : "border-zinc-700"
              }`}
            />
          </div>
        </div>
      )}

      {delta ? (
        <Tabs defaultValue="decoded" className="flex-1 flex flex-col overflow-hidden min-h-0">
          <TabsList className="mx-3 mt-2 w-fit shrink-0">
            <TabsTrigger value="decoded">Decoded</TabsTrigger>
            <TabsTrigger value="raw">Raw hex</TabsTrigger>
          </TabsList>

          <TabsContent
            value="decoded"
            className="flex-1 flex flex-col overflow-hidden mx-0 mt-0 min-h-0"
          >
            <ScrollArea className="flex-1 px-3 py-2">
              {decodedInfo ? (
                <div className="space-y-0.5">
                  {Object.entries(decodedInfo.value).map(([k, v]) => (
                    <div key={k} className="flex items-start gap-1.5 py-0.5">
                      <span className="text-zinc-500 text-[11px] font-mono shrink-0 pt-0.5 w-[84px] truncate">
                        {k}
                      </span>
                      {sessionId ? (
                        <FieldEditor
                          value={v}
                          path={k}
                          onChange={handleFieldChange}
                          modifiedPaths={modifiedPaths}
                        />
                      ) : (
                        <span className="text-zinc-300 text-xs font-mono break-all">
                          {typeof v === "object" ? JSON.stringify(v) : String(v)}
                        </span>
                      )}
                    </div>
                  ))}
                </div>
              ) : delta.decoded_after ? (
                <pre className="text-xs font-mono whitespace-pre-wrap text-zinc-300">
                  {JSON.stringify(delta.decoded_after, null, 2)}
                </pre>
              ) : (
                <p className="text-zinc-500 text-xs">No IDL — account not decodable.</p>
              )}
            </ScrollArea>

            {/* Apply / Discard strip */}
            {sessionId && (
              <div className="px-3 py-2 border-t border-zinc-800 shrink-0">
                {applyError && (
                  <p className="text-red-400 text-[10px] mb-1.5 break-words">{applyError}</p>
                )}
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={applyEdits}
                    disabled={!hasEdits || applying}
                    className="text-xs h-7 flex-1"
                  >
                    {applying
                      ? "Applying…"
                      : hasEdits
                      ? `Apply (${mutationCount})`
                      : "Apply"}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={discardEdits}
                    disabled={!hasEdits}
                    className="text-xs h-7"
                  >
                    Discard
                  </Button>
                </div>
              </div>
            )}
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
