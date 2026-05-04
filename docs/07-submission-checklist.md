# 07 — Submission Checklist

Work down this list on Day 18. Check every box before you sleep.

## Repository

- [x] `README.md` — rewritten Day 15 (banner, arch diagram, badges, clean pitch)
- [x] `LICENSE` — MIT
- [x] `CONTRIBUTING.md` — done Day 15
- [x] `CODE_OF_CONDUCT.md` — done Day 15
- [x] `.github/workflows/ci.yml` — done Day 15 (cargo check/clippy/test + pnpm tsc/build)
- [x] `.github/ISSUE_TEMPLATE/bug_report.md` — done Day 15
- [ ] Git history: no leaked keys — verify with `git log -p | grep HELIUS`
- [ ] All `.env` files gitignored — **verify `.env` is in `.gitignore`**
- [ ] Repo description and topics set on GitHub (`solana`, `debugger`, `developer-tools`, `rust`, `typescript`) — **do manually on github.com**
- [ ] Repo pinned on GitHub profile — **do manually**

## Live deployment

- [ ] API deployed — Render/Fly.io free tier, public URL, responds to `GET /health`.
- [ ] Web app deployed — Vercel, custom domain if you bought one (`replay.dev` or similar; fine if it's `replay-xyz.vercel.app`).
- [ ] CORS configured so web can hit API.
- [ ] Rate limiting enabled (even a simple per-IP token bucket). Assume 100 people will click your demo link in the first hour after submission.
- [ ] Error page for when Helius is rate-limited (give the user a clear "try again in a minute" message, not a 500).
- [ ] Three canonical demo signatures pre-loaded and tested from a fresh browser.

## Packages

- [x] `replay-sdk` published to crates.io (0.1.0) — done Day 13
- [x] `replay-core` published to crates.io (0.1.0) — done Day 13
- [x] `@zaxcoraider/replay-sdk` published to npm (0.1.0) — done Day 12
- [ ] `replay-cli` installable via `cargo install replay-cli` — **verify this works from scratch**
- [x] Version numbers consistent across all packages

## Videos

- [ ] 2-minute demo video — uploaded to YouTube as unlisted, linked from README and every submission
- [ ] 3-minute pitch video — same
- [ ] Both videos have captions (YouTube auto-generated is fine)
- [ ] Thumbnails are not default-grey; use a clean screenshot of the app

## Submissions

### Colosseum Frontier (main submission)
- [ ] arena.colosseum.org submission form completed
- [ ] Writeup ready: `docs/submissions/colosseum-main.md`
- [ ] Demo video link — **still needed (Day 17)**
- [ ] Pitch video link — **still needed (Day 17)**
- [ ] Live demo link: https://replay-weld.vercel.app ✓

### Public Goods track (Colosseum)
- [ ] Separate form (or flag within Colosseum submission — verify)
- [x] Write-up ready: `docs/submissions/public-goods.md` (200 words)
- [x] MIT license on repo ✓

### Helius side track (Superteam Earn)
- [ ] earn.superteam.fun — submit
- [x] Writeup ready: `docs/submissions/helius-track.md`
- [x] Blog draft ready: `docs/blog/time-travel-debugger-on-laserstream.md`
- [ ] Send blog draft to Helius dev-rel (Discord DM — template in `docs/submissions/social-posts.md`)

### Glass Surfers Dev Tooling Grant
- [ ] Submit at earn.superteam.fun/grants/glass-surfers-dev-tooling-grants
- [x] Application ready: `docs/grants/glass-surfers-application.md`

### Ackee / Security track (if running)
- [x] Writeup ready: `docs/submissions/ackee-security.md` (500 words, Mango exploit)
- [ ] Verify track is live on earn.superteam.fun, then submit

### Regional Superteam track
- [ ] Verify which track applies to your region on earn.superteam.fun
- [ ] Submit with writeup from colosseum-main.md (adapt intro for region)

### Any other tracks
- [ ] Final sweep of earn.superteam.fun filtered by `hackathon=frontier` before Day 18

## Social + amplification

All drafts ready in `docs/submissions/social-posts.md`.

- [ ] Launch tweet (+ thread) — template ready
- [ ] Anchor Discord #dev-tools — template ready
- [ ] OrcaDAO Discord #dev — template ready
- [ ] Superteam Discord #hackathon — template ready
- [ ] Helius Discord DM to dev-rel — template ready
- [ ] r/solana post — template ready
- [ ] HackerNews Show HN (Saturday morning US time performs best)

## Paper trail

- [ ] Screenshot every submission confirmation into `submissions/` folder (not committed, kept locally)
- [ ] Note submission timestamps in `docs/LOG.md`

## The final 30 minutes before submission deadline

- [ ] Load the live demo in a private browser window. Does it work?
- [ ] Click through the 3 canonical demo signatures. Do they all complete?
- [ ] Watch your demo video start-to-finish. Does any moment make you cringe?
- [ ] Run `curl` against your API's `/health` endpoint. 200?
- [ ] `cargo install replay-cli && replay <signature>`. Works from a fresh machine?
- [ ] All submission forms say "submitted"?

Sleep. You're done.
