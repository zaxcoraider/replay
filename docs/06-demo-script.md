# 06 — Demo Script

This is how you demo to a judge. Every second matters. Practice it out loud three times before recording.

## The 2-minute demo video (the one you submit)

### 0:00–0:10 — Hook
> "Every Solana dev has hit this: a transaction fails on mainnet with a cryptic error and you have no way to reproduce it. Today I'm showing you Replay — a time-travel debugger for Solana."

Visual: a failed tx on Solscan, then cut to the Replay UI.

### 0:10–0:30 — Paste and replay
Paste a real mainnet transaction signature. Hit enter. The UI fills in:
- Timeline with 7 instruction segments.
- Account inspector showing 12 accounts with decoded Anchor state.
- Logs matching mainnet exactly.

> "I just replayed a Jupiter swap with full fidelity. Every account's pre-state is reconstructed from the slot it executed at. The bytecode is the exact bytecode that was live that day."

### 0:30–1:00 — Step through
Click through instruction frames in the timeline.

> "I can inspect state at every CPI boundary. This is where Whirlpool got called. This is where the oracle was read. You can see compute units depleting across instructions."

Visual: the CU fuel gauge filling up as you move through the timeline.

### 1:00–1:30 — The mutation (the wow moment)

Click on an account — the Whirlpool config. Edit a field — `fee_rate` from 30 to 9999.

> "Now let's ask a what-if. What if this fee rate had been different?"

Click "Fork + Re-run."

The timeline re-renders. Red bars. The third instruction now fails.

> "The transaction that succeeded on mainnet fails in the fork. I can pinpoint exactly which instruction broke and why — in seconds, without deploying anything."

### 1:30–1:50 — The CI pitch
Cut to terminal.

```bash
$ replay diff <sig1> <sig2> --against=./my-new-program.so
[✓] Replayed 47 historical transactions
[✗] 3 regressions found:
    - Tx 4abc...: compute units +18% (was 842k, now 997k)
    - Tx 9def...: instruction #3 now fails with 0x1771
    - Tx 2fee...: account 7xyz... post-state differs
```

> "Replay ships as a CLI and SDK — you can replay historical transactions against a new program version in CI, before you ever deploy."

### 1:50–2:00 — Close
> "Replay is open source, MIT-licensed, running live at replay.dev. This is how debugging should feel on Solana. Thank you."

End card: repo link, your Twitter, the tagline.

---

## The 3-minute pitch video (separate submission)

Camera on you. Terminal visible side-by-side if you want. This is a founder pitch, not a product demo.

### 0:00–0:20 — Problem
> "I've built on Ethereum for three years. When a transaction fails there, I have Tenderly. I can fork mainnet, replay the tx, step through every opcode, and find the bug in minutes. On Solana, I have... log files. And prayer."

### 0:20–0:50 — Market
> "Tenderly raised $40M. Foundry is foundational to every EVM dev's workflow. These tools exist because debugging complex on-chain state is a hard problem that only gets harder as protocols compose. Solana has composition — every DeFi interaction touches four programs and twenty accounts — but doesn't have the debugging tools to match. That's the gap."

### 0:50–1:30 — Solution
Cut to a 30-second speedrun of the demo above. Paste → replay → mutate → diff.

> "Replay is a time-travel debugger for Solana. Fork any mainnet transaction, mutate any account, re-run. Plus a CLI and SDK for CI regression testing against historical mainnet traffic."

### 1:30–2:15 — How we got here
> "I'm a multi-chain developer. I've felt this pain across Ethereum, Arbitrum, and Solana. The reason Replay hasn't existed yet is that it needed three ingredients: Helius LaserStream's historical streaming, litesvm's fast in-process VM, and Anchor's IDL standardization. All three matured in the last year. This is the right moment."

### 2:15–2:45 — What's next
> "Three surfaces: hosted web app for exploration, CLI and SDK for programmatic use in CI, and a forthcoming VSCode extension for in-editor debugging. Distribution is through every protocol that touches Solana — they all have the same pain. Monetization is hosted infra for teams; the core stays MIT forever."

### 2:45–3:00 — Close
> "Replay is live today at replay.dev. Repo is linked below. I'd love to talk about how to make debugging on Solana as good as it is everywhere else."

---

## The live judging moment (if you get interview)

If you're pulled into a live judging conversation (Colosseum does this for finalists), the template:

1. **Have the demo ready on screen before the call starts.** Do not screen-share-fumble.
2. **Lead with the mutation.** Don't do the full demo. Go straight to the "change this field, re-run" moment. That's the only thing they'll remember.
3. **Know three numbers cold:**
   - Replay round-trip time on a cold cache (target: <5 seconds).
   - Number of protocols you've tested against (target: ≥8).
   - Historical tx coverage (target: "any tx in the last ~90 days" — Helius's realistic window).
4. **If asked about monetization**, say: "Core stays MIT. Hosted team offering with persistent sessions, CI integrations, private repos, and priority compute is the business."
5. **If asked about moat**, say: "We're not a UI — we're an SVM fork engine. Every other tool builds on top of us."
6. **If you don't know, say you don't know.** Judges respect it. Making up an answer gets caught.

---

## The one thing to practice

**Paste → replay → mutate → diff.** The whole flow. Do it 20 times. You want zero hesitation. Any "let me just click around to find the..." in the video kills you.

Record the video with OBS. Use a real signature you've tested a hundred times. Have a backup signature ready in case the primary one has an issue at recording time.
