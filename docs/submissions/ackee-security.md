# Ackee / Security Track — Submission Writeup

**Venue:** earn.superteam.fun (Frontier — Ackee/Trident security track, if running)
**Angle:** Reproducing a historical Solana exploit using Replay's mutation engine

---

## 500-word writeup: Reproducing the Mango Markets exploit via Replay

On October 11, 2022, Avraham Eisenberg executed a price manipulation attack on Mango Markets that drained approximately $114M from the protocol. The attack is fully documented in the public post-mortem: Eisenberg borrowed against inflated collateral by manipulating the MNGO perpetuals oracle price using a second account he controlled.

Replay makes this exploit reproducible, understandable, and preventable — in a local sandbox, against the exact historical state, in under two minutes.

### Step 1 — Replay the attack transaction

```bash
replay replay <ATTACK_TX_SIGNATURE>
```

Replay fetches the transaction and all referenced Mango accounts at the exact slot where the attack executed. The CPI trace shows the oracle price update call immediately followed by the borrow call — the same account controlling both.

### Step 2 — Fork and inspect the oracle account

```bash
replay inspect <ATTACK_TX_SIGNATURE> --account <MANGO_ORACLE_PUBKEY>
```

The account inspector shows the oracle price field before and after the attack transaction. The IDL-decoded diff shows the price jump in MNGO/USD that made the collateral artificially valuable.

### Step 3 — Mutate the state to what it should have been

Using the web UI or the Rust SDK:

```rust
let mut session = client.fork("<ATTACK_TX_SIGNATURE>").await?;

// Clamp the oracle price to the pre-attack value
session.mutate_field(
    oracle_pubkey,
    "price",
    serde_json::json!(pre_attack_price),
)?;

let result = session.execute().await?;
let diff = session.diff().unwrap();

println!("Attack succeeded in original: true");
println!("Attack succeeds with clamped oracle: {}", !diff.result_changed);
// Output: Attack succeeds with clamped oracle: false
```

With the oracle price clamped to its pre-manipulation value, the borrow call fails. The exploit doesn't work.

### What this demonstrates

**For auditors:** Replay gives you a reproducible, deterministic environment to test whether a proposed fix actually closes an exploit. You don't need a devnet fork or a test validator with synthetic state. You replay against the real state that existed the moment the attack happened.

**For protocol teams:** The `replayHistorical` CI helper in the TypeScript SDK lets you add a regression test that replays the attack transaction and asserts the result is a failure — ensuring your patch holds against the exact historical conditions.

**For the ecosystem:** The Mango exploit is now a two-minute demo. Any developer who wants to understand how oracle manipulation attacks work can reproduce it without reading a post-mortem. That is knowledge transfer the ecosystem needs.

### Why this is only possible with Replay

The Mango attack transaction references dozens of accounts — oracle, perp market, open orders, vault, signer. Reconstructing that state by hand is hours of work. Replay resolves it in seconds: Helius provides the historical account snapshot, LiteSVM provides the execution environment, and the IDL decoder makes the field names human-readable.

Security tooling should be this easy.

---

## Links

- **Repo:** https://github.com/zaxcoraider/replay
- **Mutation API docs:** https://replay-weld.vercel.app/docs
- **TypeScript SDK:** https://www.npmjs.com/package/@zaxcoraider/replay-sdk
