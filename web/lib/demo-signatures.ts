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
    // Replace with a recent Jupiter v6 swap (program JUP6Lkb2abBUvo9pe2O5xFnTQHJJCfq1VYJqkZrSAH8)
    signature: "FILL_ME_JUPITER_V6_SWAP_SIG",
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
    // Replace with a recent Orca Whirlpool swap (program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc)
    signature: "FILL_ME_WHIRLPOOL_SWAP_SIG",
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
    // Replace with a recent Drift v2 perp trade (program dRiftyHA3MooBbHFgJ5JecU3BHpHtYjDv8a9quSrm3e)
    signature: "FILL_ME_DRIFT_PERP_TRADE_SIG",
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
