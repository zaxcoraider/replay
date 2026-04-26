# Local memory

One file per working day. Each file is a self-contained snapshot of where
the project stands after that day's session, written so a fresh Claude
Code session can recover full state without reading every commit.

## Convention

Each `day-XX.md` file should answer:

1. **Goal** — what the day's prompt asked for.
2. **What landed** — files touched, key code added, with line refs where
   useful.
3. **Tests** — what runs green; what's gated; what's skipped and why.
4. **Decisions worth remembering** — non-obvious calls (dep versions,
   API quirks, things that surprised us).
5. **Follow-ups** — work explicitly deferred, in priority order.
6. **Next session bootstrap** — exact commands + prompt files to open.

## Files

- [`day-01.md`](day-01.md) — fetch path complete (HeliusClient, fetch_full_tx_context, mock+live tests, CLI --json).
