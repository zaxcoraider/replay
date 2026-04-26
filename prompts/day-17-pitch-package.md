# Day 17 — Pitch Package

## Goal

Record both videos, build the deck, write every submission's "project description" field.

## Deliverables

1. **2-minute demo video** following `docs/06-demo-script.md`.
   - Record with OBS at 1080p, 30fps.
   - Clean audio (even your phone earbuds are fine if you're in a quiet room).
   - No background music unless you're skilled at mixing. Voice only is safer.
   - Edit in DaVinci Resolve or ScreenFlow. Cut filler. Keep it tight.
   - Export as MP4, H.264, reasonable bitrate.
   - Upload to YouTube (unlisted), grab the link.

2. **3-minute pitch video** following `docs/06-demo-script.md`.
   - Camera on. Good lighting (sit facing a window).
   - Don't read from a script visibly. Bullet points you glance at, not a teleprompter.
   - Record 3 takes. Pick the best.

3. **Pitch deck (10 slides):**
   1. Title + tagline + your handle
   2. The problem (EVM has Tenderly, Solana has logs)
   3. The solution (one screenshot of the diff view)
   4. How it works (the architecture diagram)
   5. Demo (embedded video or "see link")
   6. Market (Tenderly $40M raise, Foundry adoption, Solana gap)
   7. Traction (stars, users, who's tried it during the hackathon)
   8. Roadmap (VSCode extension, persistent sessions, team offering)
   9. Business model (hosted for teams, core stays MIT)
   10. Ask + contact
   
   Use keynote/Figma. Don't over-design. Readable > beautiful.

4. **Submission writeups:**
   - Colosseum main: 250 words, inspirational-but-concrete tone.
   - Public Goods: 200 words, focused on ecosystem benefit.
   - Helius track: 200 words, technical, specific about LaserStream usage.
   - Regional: 200 words, include your local context.
   - Every one of these is a separate document; none copy-paste the others.

5. **Sanity pass on everything:**
   - Load the live demo in 3 different browsers. Works?
   - Run the CLI from a fresh machine (or Docker container). Works?
   - Click every link in every submission. Every one resolves?
   - Watch the demo video end-to-end. Any cringe moment? Re-record.

## What NOT to do

- No new features. At all. The feature freeze from yesterday is in effect.
- Don't fiddle with deployment unless something is actually broken.

## End-of-day

- Both videos uploaded and linked.
- Deck exported as PDF.
- All submission writeups in `submissions/` folder locally.
- Full night's sleep.
