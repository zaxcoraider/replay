# Day 18 — Final Submission Day

## Goal

Ship every submission. Sleep.

## Timeline (UTC — adjust to your timezone)

Treat the deadline as **3 hours earlier than the real deadline.** Submission portals slow down under load. People in your timezone will be spamming the same form. Don't be the person submitting at 23:58.

### Morning (6 hours before cutoff)

1. Fresh-machine sanity test. Ideally on a different device than your dev machine:
   - Load `replay.dev` (or your Vercel URL). Run all 3 demo signatures. All succeed?
   - `cargo install replay-cli` in a fresh terminal. Run a replay. Works?
   - Hit `/health` on the API. 200?
   - Open the demo video. Watch it fully. Cringe moments?

2. Run through `docs/07-submission-checklist.md` top to bottom. Check every box.

### Midday (4 hours before cutoff)

3. **Submit to Colosseum first.** This is the main prize. Do not wait. Fill out every field. Include:
   - Repo link
   - Live demo link
   - 2-min demo video link
   - 3-min pitch video link
   - Deck (PDF)
   - Team info (you)
   - One-liner
   - 250-word project description
   - Public Goods flag/field if separate

4. **Submit to Superteam Earn side tracks.** One per track, each with unique writeup.

5. **Submit to Glass Surfers grant** (if not done Day 15).

6. Screenshot every submission confirmation. Save to `submissions/` locally.

### Afternoon (2 hours before cutoff)

7. **Launch tweet goes live.** Don't queue earlier — you want it fresh when people check the Colosseum leaderboard.

8. **Discord + community posts.** Every server. Keep it short, link-heavy.

9. **Update README** with "Submitted to Colosseum Frontier 2026 ✓" at the top.

### Final 30 minutes

10. Reload live demo one last time in a private browser window. Works?

11. Re-check every submission URL. All still reachable?

12. Send final follow-up to Helius dev-rel ("Submitted today, here's the link, would love to chat about the blog post").

13. Stop.

## What NOT to do today

- No code changes after 4 hours before cutoff. None. Zero. Even "tiny fixes" break things under deadline stress.
- No last-minute scope additions.
- No social media refreshing. Submit and walk away.

## Post-submission

Write `docs/POSTMORTEM.md` within 48 hours of submission while memory is fresh:
- What worked
- What broke
- What you'd do differently
- Time spent per day
- Side-track outcomes as they come in

This is gold for the next hackathon.

## Results timeline

- Colosseum typically announces top-20 within 3–4 weeks of submission.
- Side tracks announce on their own cadence — usually 1–2 weeks.
- Glass Surfers grant responses in 2–4 weeks.

Don't refresh the leaderboard. Go build the next thing. If Replay resonates, the right next move after the hackathon is to incorporate and go raise — you'll have the data (stars, users, tx volume) to make that pitch real.
