# Day 16 — Regional Submission + Side-Track Sweep

**Date:** 2026-05-04
**Result:** All submission writeups drafted. Checklist updated. Social post templates ready. Awaiting manual submissions and Render redeploy.

## What landed

### docs/submissions/ (all new)
- `colosseum-main.md` — 300-word writeup for arena.colosseum.org Grand Champion track
- `public-goods.md` — 200-word writeup for Colosseum Public Goods track ($10k)
- `helius-track.md` — Helius side track writeup with LaserStream code snippet + blog link
- `ackee-security.md` — 500-word Mango Markets exploit reproduction writeup
- `social-posts.md` — copy-paste templates: launch tweet, thread, Helius DM, Anchor/Orca/Superteam Discord, r/solana, HN

### docs/07-submission-checklist.md — updated
- Checked off all Day 15 completions (README, CONTRIBUTING, workflows, packages)
- Every submission track now has: status, writeup file pointer, and manual action items

### Verified clean
- `.env` is in `.gitignore` — confirmed
- No API keys in git history — confirmed
- Day 15 commit pushed to GitHub — confirmed (4503d53)

## Manual actions still needed (user must do)

### GitHub
- [ ] Set repo description: "Time-travel debugger for Solana transactions"
- [ ] Set topics: solana, debugger, developer-tools, rust, typescript
- [ ] Pin repo on profile

### Submissions (all writeups ready in docs/submissions/)
- [ ] Glass Surfers grant — earn.superteam.fun/grants/glass-surfers-dev-tooling-grants
- [ ] Helius side track — earn.superteam.fun
- [ ] Ackee/security track — verify on earn.superteam.fun then submit
- [ ] Regional Superteam track — verify which track, then submit
- [ ] Colosseum main — arena.colosseum.org (needs demo video — Day 17)

### Helius blog
- [ ] Send `docs/blog/time-travel-debugger-on-laserstream.md` to Helius dev-rel
- [ ] Use DM template in `docs/submissions/social-posts.md`

### Social
- [ ] Post launch tweet + thread (template in social-posts.md)
- [ ] Post in Anchor, OrcaDAO, Superteam Discords
- [ ] r/solana post

### Render redeploy (pending since Day 14)
- [ ] Trigger redeploy on Render dashboard — bakes in `/replay-live/:sig` route
- [ ] ~30 min Rust build on free tier
- [ ] After deploy: test `https://replay-weld.vercel.app/replay/<SIG>` → Live tab

## Next: Day 17 — Pitch package
- Read `prompts/day-17-pitch-package.md`
- Record 2-min demo video + 3-min pitch video
- Colosseum submission needs both video links
