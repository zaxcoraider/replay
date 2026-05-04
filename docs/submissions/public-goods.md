# Public Goods Track — Submission Writeup

**Venue:** Colosseum Frontier (flag within main submission or separate form — verify)
**Prize:** $10,000
**Required:** MIT license, CONTRIBUTING.md, clear README, published as package

---

## 200-word writeup

Replay is a time-travel debugger for Solana transactions. It is MIT-licensed, published to crates.io and npm, and designed from the ground up to be a public good.

The core engine (`replay-core`) is a Rust library. Any protocol team, auditor, or researcher can embed it directly — no hosted service required, no API key beyond a free Helius account. The CLI (`replay-cli`) installs with `cargo install`. Both SDKs (`replay-sdk` for Rust, `@zaxcoraider/replay-sdk` for TypeScript) are open source and include a `replayHistorical` CI helper so teams can regression-test their programs against their own mainnet transaction history.

The hosted web UI and API are convenience wrappers, not gatekeepers. The intelligence is in the open library.

Solana has world-class block explorers but no debugging primitives. Developers who hit a production bug on mainnet have no way to reproduce it, no way to ask "what if," and no path to a systematic fix. Replay gives every Solana developer — from a solo protocol founder to a security auditor writing a post-mortem — the ability to answer those questions.

That is the public good: a debugging primitive that the ecosystem was missing, given away free.

---

## Checklist of qualifying criteria

- [x] MIT license — [`LICENSE`](../../LICENSE)
- [x] `CONTRIBUTING.md` with setup, test, and PR instructions
- [x] `CODE_OF_CONDUCT.md` — Contributor Covenant 2.1
- [x] `replay-core` published to crates.io (public library)
- [x] `replay-sdk` published to crates.io
- [x] `@zaxcoraider/replay-sdk` published to npm
- [x] No token, no paywall, no VC funding
- [x] CI badge on README — GitHub Actions
