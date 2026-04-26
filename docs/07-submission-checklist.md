# 07 — Submission Checklist

Work down this list on Day 18. Check every box before you sleep.

## Repository

- [ ] `README.md` at root with: project name, 1-paragraph pitch, animated GIF of the demo, install instructions, links to live demo + video + docs
- [ ] `LICENSE` — MIT
- [ ] `CONTRIBUTING.md`
- [ ] `CODE_OF_CONDUCT.md` (Contributor Covenant, copy-paste)
- [ ] `.github/workflows/ci.yml` — runs `cargo test` + `cargo clippy -- -D warnings` + frontend `pnpm test` + `pnpm build`
- [ ] `.github/ISSUE_TEMPLATE/bug_report.md`
- [ ] Git history cleaned up (no leaked keys, no giant binary commits). If history is messy, `git filter-repo` or squash to a few meaningful commits.
- [ ] All `.env` files gitignored. Double-check no API keys in history.
- [ ] Repo description and topics set on GitHub (`solana`, `debugger`, `developer-tools`, `rust`).

## Live deployment

- [ ] API deployed — Render/Fly.io free tier, public URL, responds to `GET /health`.
- [ ] Web app deployed — Vercel, custom domain if you bought one (`replay.dev` or similar; fine if it's `replay-xyz.vercel.app`).
- [ ] CORS configured so web can hit API.
- [ ] Rate limiting enabled (even a simple per-IP token bucket). Assume 100 people will click your demo link in the first hour after submission.
- [ ] Error page for when Helius is rate-limited (give the user a clear "try again in a minute" message, not a 500).
- [ ] Three canonical demo signatures pre-loaded and tested from a fresh browser.

## Packages

- [ ] `replay-sdk` published to crates.io (version 0.1.0)
- [ ] `@replay/sdk` published to npm (version 0.1.0)
- [ ] `replay-cli` installable via `cargo install replay-cli` OR a `curl | sh` one-liner
- [ ] Version numbers consistent across all packages

## Videos

- [ ] 2-minute demo video — uploaded to YouTube as unlisted, linked from README and every submission
- [ ] 3-minute pitch video — same
- [ ] Both videos have captions (YouTube auto-generated is fine)
- [ ] Thumbnails are not default-grey; use a clean screenshot of the app

## Submissions

### Colosseum Frontier (main submission)
- [ ] arena.colosseum.org submission form completed
- [ ] Project name: **Replay**
- [ ] One-liner: "Time-travel debugger for Solana transactions"
- [ ] Repo link: ✓
- [ ] Demo video link: ✓
- [ ] Pitch video link: ✓
- [ ] Live demo link: ✓
- [ ] Submitted **Grand Champion** track (implicit — no track selection, judged on impact)

### Public Goods track (Colosseum)
- [ ] Separate form (or flag within Colosseum submission — verify)
- [ ] Write-up explaining public-goods framing (200 words)
- [ ] MIT license verified on repo

### Helius side track (Superteam Earn)
- [ ] earn.superteam.fun listing — submit
- [ ] Write-up specifically calls out LaserStream integration with code snippets
- [ ] Link to your Helius blog post draft

### Glass Surfers Dev Tooling Grant
- [ ] Grant form at earn.superteam.fun/grants/glass-surfers-dev-tooling-grants
- [ ] Scope your ask at $5k for specific future work (e.g., VSCode extension)

### Regional Superteam track (yours)
- [ ] Verify which track applies to your region
- [ ] Separate submission there

### Any other tracks that went live during the hackathon
- [ ] Final sweep of earn.superteam.fun filtered by `hackathon=frontier`

## Social + amplification

- [ ] Launch tweet — video + one-liner + link. Tag @heliuslabs, @SolanaFndn, @SuperteamDAO.
- [ ] Post in every Solana Discord you're in (OrcaDAO dev channel, Superteam, Anchor)
- [ ] HackerNews Show HN post (Saturday morning US time performs best)
- [ ] r/solana post
- [ ] Drop into Colosseum's Telegram/Discord — these judges hang out there

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
