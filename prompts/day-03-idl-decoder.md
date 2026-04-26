# Day 3 — IDL-Aware Account Decoder

## Goal

For any account in a reconstructed state, if we can find the program's Anchor IDL, decode the account data to a structured JSON tree the UI can render. Fallback: hex + "bring your own IDL" path.

## Deliverables

1. `crates/replay-core/src/idl.rs`:
   ```rust
   pub struct IdlCache {
       dir: PathBuf,
   }
   
   impl IdlCache {
       pub async fn get_or_fetch<C: HeliusClient>(
           &self,
           client: &C,
           program_id: &Pubkey,
       ) -> Result<Option<Idl>, ReplayError>;
       
       pub fn insert(&self, program_id: &Pubkey, idl: &Idl) -> Result<(), ReplayError>;
       pub fn manual_insert_from_json(&self, program_id: &Pubkey, idl_json: &str) -> Result<(), ReplayError>;
   }
   
   pub struct AccountDecoder<'a> {
       idl_cache: &'a IdlCache,
   }
   
   impl AccountDecoder<'_> {
       pub async fn decode(
           &self,
           pubkey: &Pubkey,
           account: &Account,
           client: &impl HeliusClient,
       ) -> DecodedAccount;
   }
   
   pub enum DecodedAccount {
       Decoded { type_name: String, value: serde_json::Value, idl_source: IdlSource },
       UnknownDiscriminator { hex: String },
       NoIdl { owner: Pubkey, hex: String },
       NotAnchor { hex: String },
   }
   
   pub enum IdlSource { OnChain, Manual, Cached }
   ```

2. Known-program IDL seeding. Bundle known IDLs for common programs so first-demo works without network:
   - SPL Token (manual decoder — it's not Anchor, use `spl_token::state::Account::unpack`)
   - SPL Token-2022 (same, with extensions)
   - Jupiter v6 (fetch once, bundle in `crates/replay-core/assets/idls/`)
   - Whirlpool
   - Drift
   - Kamino Lend
   - System program (no IDL needed; native decoding)

3. Non-Anchor decoders. Token accounts, Mint accounts, stake accounts. Owner-program-based dispatch.

4. `crates/replay-core/src/types.rs`: enrich `AccountDelta` with `decoded_before` / `decoded_after` populated by the decoder.

## Anchor IDL fetching reference

Anchor stores the IDL at a PDA derived from the program ID:

```rust
use anchor_lang::idl::IdlAccount;

let idl_address = IdlAccount::address(program_id);
// Fetch this account via client.get_account_info(idl_address).
// The first 8 bytes are the Anchor IDL discriminator.
// Bytes 8..40 are the authority pubkey.
// Bytes 40..44 are a u32 length (little-endian) of the zlib-compressed IDL JSON.
// Remaining bytes are zlib-compressed IDL JSON.
// Decompress with `flate2::read::ZlibDecoder`.
```

Some programs don't have on-chain IDLs. For those, provide a "paste IDL" path via `IdlCache::manual_insert_from_json`.

## Borsh decoding via IDL

Use a lib if you can — `anchor-idl` or `anchor-syn` expose parsing. If those are awkward, write a minimal Borsh-per-IDL decoder in ~300 lines. The types you need to handle:
- Primitives: u8/i8/u16/i16/u32/i32/u64/i64/u128/i128/bool
- Pubkey (32 bytes)
- String (u32 length + bytes)
- Array (fixed length)
- Vec (u32 length + elements)
- Option (1-byte tag + value)
- Struct (recursive)
- Enum (u8 discriminant + variant data)

## Tests

Decode a real Whirlpool position account (grab a signature that creates one, fetch the account post-state, decode). Round-trip the decode → re-encode → bytes-equal.

## What NOT to do

No UI rendering of decoded data today. That's Day 8.

## End-of-day report

1. How many of the top-10 Solana programs have working IDL decoding?
2. Round-trip decode performance (target: <1ms per account).
3. Where IDL fetching fails — list of programs that don't publish on-chain IDLs.
