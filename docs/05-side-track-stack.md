# 05 — Side-Track Stack

Your goal: **one project, stacked across the main prize + as many side tracks as it naturally fits.** The word "naturally" is load-bearing — forcing integrations that don't fit will hurt the core demo.

## Tier 1: The locks (build straight into the project)

### Main Frontier (Colosseum)
- **Prize:** Grand Champion $30k, top-20 runners-up $10k each.
- **What wins:** Judges are VCs evaluating investability. They want novel infra with a clear market analog. "Tenderly for Solana" is the pitch.
- **Submission venue:** arena.colosseum.org
- **Deadline:** May 11, 2026 (verify on site).
- **Deliverables:** Repo link, pitch video (≤3 min), demo video (≤2 min), deck (optional but recommended).

### Public Goods Award (Colosseum)
- **Prize:** $10k.
- **What qualifies:** MIT/Apache 2.0 license, meaningful public benefit, not primarily a business. Dev tools are the canonical fit.
- **Requirements:** License file from day 1, CONTRIBUTING.md, clear README, published as package (crates.io + npm).
- **Gotcha:** Don't contradict yourself in the pitch. The VC pitch says "we'll monetize the hosted version." The Public Goods framing says "open tool for the ecosystem." Both can be true — the hosted version is *optional* convenience, the CLI and SDK remain free and MIT.

### Helius LaserStream Track (expected)
- **Prize:** Varied historically $3k–15k.
- **What qualifies:** Meaningful integration of Helius products — LaserStream gRPC, Enhanced Transactions API, or DAS API.
- **Why this fits:** Your state reconstruction is *built on* Helius. The Day 14 work specifically adds LaserStream live replay to make this a flagship showcase.
- **Tactic:** Write a blog post for Helius. "How we built a time-travel debugger on LaserStream." They amplify; you win mindshare + likely the track.

### Glass Surfers Dev Tooling Grant (Solana Foundation)
- **Prize:** Up to $5k USDC.
- **Not competitive** — separate application, usually approved for legitimate dev tools.
- **Tactic:** Apply on Day 15 with the repo link, a short "why this matters" writeup, and your submission plan.

## Tier 2: Verify-then-target

These need you to check the Superteam Earn board during the hackathon week for exact requirements. They rotate.

### Anchor / Coral track (if running)
- Likely requires Anchor IDL integration depth. You already have this (IDL decoder + on-chain IDL fetch).

### Ackee / Trident (security) track (if running)
- Security-focused tooling fits your use case ("replay this exploit"). If they run a track, write up one historical exploit your tool reproduces and submit.

### Your Regional Superteam track
- Superteam chapters (NG, IN, VN, Poland, etc.) run their own tracks, often with smaller pools and less competition. Check `earn.superteam.fun` for your region.

## Tier 3: Don't force it

### DeFi / Consumer tracks
Skip. Your project is infra; shoehorning a DeFi angle weakens the narrative. Judges reward focus.

### AI / Agent tracks
Tempting to bolt on "AI that debugs txs for you." **Don't.** It dilutes the demo. If you have 2 extra hours on Day 17, consider adding a single Claude-API-powered button: "Explain this trace in plain English." Small, useful, doesn't distract. That's the only AI angle worth adding.

## Submission hygiene

For every track:
1. **Unique submission per track** — never copy-paste identical descriptions. Each track's reviewers want to see you addressed their specific prompt.
2. **Lead with the demo.** Every submission's first sentence should make someone want to click the video.
3. **Link to GitHub + live demo + video.** All three. Every time.
4. **Tag the sponsor** in social posts if they have them. Free amplification.

## Expected stacked range

Realistic:
- If the project ships and the demo is tight: **$15k–$25k** (main pool top-20 + Public Goods + Helius + Glass Surfers).
- If you also land Grand Champion or top-5: **$45k–$60k+**.
- Worst case — you ship the CLI and engine but the UI is rough: **$5k–$10k** (Glass Surfers + maybe Helius + a small regional track).

## Week-by-week side-track timeline

| Week | Side-track work |
|---|---|
| Days 1–10 | Focus 100% on core product. No side-track work yet. |
| Days 11–13 | CLI + SDKs ready → you are now ready to pitch anywhere. |
| Day 14 | Helius integration (LaserStream live) + draft blog post. |
| Day 15 | Public Goods polish pass. Apply for Glass Surfers grant. |
| Day 16 | Regional Superteam submission + verify any tracks that went live mid-hackathon. |
| Day 17 | Pitch package. |
| Day 18 | Final submissions to every venue. |
