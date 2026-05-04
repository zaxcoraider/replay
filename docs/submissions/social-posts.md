# Social Posts — Day 16 Launch

Copy-paste these. Post in order: tweet first, then Discord, then Reddit.

---

## Launch tweet

```
Built a time-travel debugger for Solana transactions.

Paste any mainnet tx → replay locally against exact historical state → fork → mutate any account → re-run → diff.

No devnet approximations. The exact accounts. The exact slot.

→ https://replay-weld.vercel.app

Open source · MIT · Rust + TypeScript SDKs

@heliuslabs @SolanaFndn @SuperteamDAO
```

---

## Follow-up tweet (thread — post 2 minutes later)

```
How it works:

1. Fetch tx + all accounts at exact slot (Helius archive RPC)
2. Hydrate a LiteSVM sandbox with that state
3. Re-execute locally — compare result to mainnet
4. Fork the sandbox, mutate any IDL-decoded field
5. Re-run, see the diff

CPI trace, CU delta, account-level diff. All local. No network after step 1.

github.com/zaxcoraider/replay
```

---

## Helius Discord DM (to dev-rel contact)

```
Hey — I built Replay, a time-travel debugger for Solana transactions, as part of Colosseum Frontier. The whole state reconstruction layer is built on Helius's archive RPC and Enhanced Transactions API.

For Day 14 I added a LaserStream integration — real SSE endpoint, real event protocol, env-var gated so it falls back to RPC when LASERSTREAM_GRPC_URL isn't set.

I wrote a ~1700 word technical blog post about it: "How we built a time-travel debugger on LaserStream." It covers the architecture, three Solana gotchas, and the SSE wire shape. Happy to send it over if the Helius blog would be interested.

Repo: https://github.com/zaxcoraider/replay
Demo: https://replay-weld.vercel.app
Blog draft: [attach docs/blog/time-travel-debugger-on-laserstream.md]
```

---

## Anchor Discord — #dev-tools channel

```
Shipped Replay — a time-travel debugger for Solana.

Paste any mainnet tx signature → reconstructs the exact historical account state → replays in LiteSVM → you get a full CPI trace with decoded args (Anchor IDL-aware for Jupiter, Whirlpool, Drift, Kamino).

Fork the sandbox, mutate any account field, re-run, see the diff.

Demo: https://replay-weld.vercel.app
Repo: https://github.com/zaxcoraider/replay

TypeScript SDK: npm install @zaxcoraider/replay-sdk
Rust SDK: replay-sdk on crates.io
```

---

## OrcaDAO Discord — #dev channel

```
Hey — built a Solana tx debugger that's Whirlpool-aware.

Replay: paste a mainnet Whirlpool swap signature → full CPI trace with decoded swap args → fork → mutate pool state (feeRate, liquidity, sqrtPrice) → re-run → diff.

Useful for understanding how a specific pool config affected a swap outcome.

Demo: https://replay-weld.vercel.app
Repo: https://github.com/zaxcoraider/replay
```

---

## r/solana post

**Title:** I built a time-travel debugger for Solana — replay any mainnet tx locally, mutate state, diff the result

```
For the past few weeks I've been building Replay as part of the Colosseum Frontier hackathon.

**What it does:** Paste any mainnet transaction signature. It fetches the transaction + all referenced accounts from Helius at the exact historical slot, hydrates a LiteSVM sandbox, and re-executes locally. You get a CPI trace, and you can fork the sandbox, mutate any account field (IDL-decoded for known programs), re-run, and see what changed.

**Why I built it:** I kept running into production bugs that were impossible to reproduce on devnet because the account state was different. I wanted Tenderly-but-for-Solana.

**Links:**
- Demo: https://replay-weld.vercel.app  
- Repo (MIT): https://github.com/zaxcoraider/replay
- npm SDK: `npm install @zaxcoraider/replay-sdk`
- Rust SDK: `replay-sdk` on crates.io

Happy to answer questions about how the state reconstruction works (LUT resolution, upgradeable programs, sysvar slots — all the fun stuff).
```

---

## Superteam Discord — #hackathon channel

```
Submitting Replay for Frontier — time-travel debugger for Solana.

Demo: https://replay-weld.vercel.app
Repo: https://github.com/zaxcoraider/replay

Main track + Public Goods + Helius + Glass Surfers grant application. Let me know if you have feedback before May 11.
```
