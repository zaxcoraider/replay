// Canonical demo transactions. Replace the FILL_ME_ signatures with real mainnet
// hashes (must be within Helius' ~90-day retention window).
//
// How to find good candidates:
//   - Jupiter v6 swap:  https://solscan.io → search program JUP6Lkb...
//   - Whirlpool swap:   https://solscan.io → search program whirLbMi...
//   - Drift perp trade: https://solscan.io → search program dRiftyHA...
//
// Paste the signature here, re-run `pnpm build`, and the demo cards go live.

export interface SuggestedMutation {
  account_label: string;
  field: string;
  new_value: unknown;
  description: string;
}

export interface Demo {
  id: string;
  title: string;
  subtitle: string;
  signature: string;
  narrative: string;
  tags: string[];
  suggested_mutation: SuggestedMutation;
}

export const DEMOS: Demo[] = [
  {
    id: "jupiter-swap",
    title: "Jupiter v6 Swap",
    subtitle: "Mutate pool fee → watch route fail",
    // Jupiter v6 swap (program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4)
    signature: "LowpTGnE4j4msHkGCVKwrKK6vxsziH2ciCBeYSm6dLv8GywVxfqHzYUznj1pzHvMLmprqz5kNuBfsHGKVhRkvs2",
    narrative:
      "A multi-hop Jupiter v6 swap through Whirlpool pools. Max out the pool fee_rate to 9999 and Re-run — the swap produces insufficient output and the entire transaction reverts.",
    tags: ["Jupiter", "Whirlpool", "DeFi"],
    suggested_mutation: {
      account_label: "Whirlpool pool",
      field: "feeRate",
      new_value: 9999,
      description: "Max out pool fee. Swap math fails with insufficient output.",
    },
  },
  {
    id: "whirlpool-swap",
    title: "Whirlpool CLMM Swap",
    subtitle: "Drain pool liquidity → slippage failure",
    // Orca Whirlpool swap (program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc)
    signature: "2JCFCrQTrRF656UT6W5cHr9JQP4XDxsrCNtYB3P6NhFswcVMQMxKWhw6afsFswgL3bgGbTAa4wxd6gF6prfDz4Zp",
    narrative:
      "A concentrated liquidity swap on Orca Whirlpool. Zero out the pool liquidity and Re-run — the swap can't fill the order and fails with insufficient liquidity.",
    tags: ["Orca", "Whirlpool", "CLMM"],
    suggested_mutation: {
      account_label: "Whirlpool pool",
      field: "liquidity",
      new_value: "0",
      description: "Zero pool liquidity. Swap fails — no liquidity to trade against.",
    },
  },
  {
    id: "drift-trade",
    title: "Drift Perp Trade",
    subtitle: "Crash oracle price → instant liquidation",
    // Drift v2 perp trade (program dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH)
    signature: "4EjzUEijQvZDrW5LbFmfjuEHDK7Q9P6fgpML6d1fd4uRjGNnKGCY82sGrVcqmNGWsGib1PBhu9tJRXRS9fpHzaag",
    narrative:
      "A Drift v2 perpetuals trade. Halve the oracle price for the base asset and Re-run — the position breaches its margin requirement and gets liquidated.",
    tags: ["Drift", "Perps", "Oracle"],
    suggested_mutation: {
      account_label: "Drift market",
      field: "lastOracleNormalisedPrice",
      new_value: "1000000",
      description: "Crash the oracle price 50%. Position hits margin threshold.",
    },
  },
];

export function isPlaceholder(sig: string): boolean {
  return sig.startsWith("FILL_ME_");
}
