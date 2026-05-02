import { create } from "zustand";
import type { Trace, TraceDiff } from "@/lib/types";

interface ReplayState {
  currentTrace: Trace | null;
  sessionId: string | null;
  selectedFrameId: string | null; // "depth-instrIndex-progId"
  selectedAccountPubkey: string | null;
  diff: TraceDiff | null;
  pendingMutations: Array<{ pubkey: string; label: string }>;

  setTrace: (t: Trace) => void;
  setSession: (id: string, baseline: Trace) => void;
  setSelectedFrame: (id: string | null) => void;
  setSelectedAccount: (pubkey: string | null) => void;
  setDiff: (d: TraceDiff | null) => void;
  addPendingMutation: (pubkey: string, label: string) => void;
  clearMutations: () => void;
  reset: () => void;
}

export const useReplayStore = create<ReplayState>((set) => ({
  currentTrace: null,
  sessionId: null,
  selectedFrameId: null,
  selectedAccountPubkey: null,
  diff: null,
  pendingMutations: [],

  setTrace: (t) => set({ currentTrace: t }),
  setSession: (id, baseline) => set({ sessionId: id, currentTrace: baseline, diff: null, pendingMutations: [] }),
  setSelectedFrame: (id) => set({ selectedFrameId: id }),
  setSelectedAccount: (pubkey) => set({ selectedAccountPubkey: pubkey }),
  setDiff: (d) => set({ diff: d }),
  addPendingMutation: (pubkey, label) =>
    set((s) => ({ pendingMutations: [...s.pendingMutations, { pubkey, label }] })),
  clearMutations: () => set({ pendingMutations: [] }),
  reset: () =>
    set({
      currentTrace: null,
      sessionId: null,
      selectedFrameId: null,
      selectedAccountPubkey: null,
      diff: null,
      pendingMutations: [],
    }),
}));
