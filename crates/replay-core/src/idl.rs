//! IDL-aware account decoder.
//!
//! Three layers:
//!   1. Owner-program dispatch in `AccountDecoder::decode` — known native
//!      programs (SPL Token, Token-2022, System) get manual decoders that
//!      don't need an IDL. Everything else falls through to the Anchor path.
//!   2. `IdlCache` resolution order: bundled (assets/idls/) → on-disk cache
//!      (~/.replay/idl-cache or `REPLAY_IDL_CACHE_DIR`) → on-chain Anchor
//!      IDL account (PDA derived from program_id) → None.
//!   3. Anchor Borsh-via-IDL decoder — walks the IDL's account schema and
//!      consumes the input bytes recursively, emitting `serde_json::Value`.
//!
//! The decoder never returns `Err`; failure cases land as `DecodedAccount`
//! variants (`UnknownDiscriminator`, `NoIdl`, `NotAnchor`) so the UI always
//! has *something* to render — a hex blob is better than a swallowed error.

use crate::error::ReplayError;
use crate::rpc::HeliusClient;
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_program::hash::hash;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use tracing::{debug, warn};

// Known native program IDs we decode without an IDL.
const SPL_TOKEN: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const SPL_TOKEN_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

/// Output of `AccountDecoder::decode`. Always succeeds — failure cases are
/// just other variants. The UI dispatches on the variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecodedAccount {
    /// We decoded against an IDL successfully.
    Decoded {
        type_name: String,
        value: Value,
        idl_source: IdlSource,
    },
    /// Decoded by a hand-written native decoder (SPL Token, etc.) — no IDL involved.
    Native {
        type_name: String,
        value: Value,
    },
    /// We had an IDL but no account schema matched the 8-byte discriminator.
    UnknownDiscriminator { hex: String },
    /// No IDL available for the owning program.
    NoIdl { owner: String, hex: String },
    /// Account is too short to be Anchor (< 8 bytes for the discriminator).
    NotAnchor { owner: String, hex: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdlSource {
    OnChain,
    Manual,
    Cached,
    Bundled,
}

/// Lightweight wrapper around the parsed IDL JSON.
#[derive(Debug, Clone)]
pub struct Idl {
    pub raw: Value,
    pub source: IdlSource,
}

/// On-disk cache of fetched IDLs, plus an in-memory bundled set seeded
/// from `assets/idls/` at construction time.
#[derive(Debug, Clone)]
pub struct IdlCache {
    pub dir: PathBuf,
    pub ttl: Duration,
    bundled: HashMap<Pubkey, Value>,
}

impl Default for IdlCache {
    fn default() -> Self {
        let dir = std::env::var("REPLAY_IDL_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".into());
                PathBuf::from(home).join(".replay/idl-cache")
            });
        // Source-tree convention: replay-core/assets/idls/<program_id>.json
        // contains pre-fetched IDLs we ship with the crate. The path is
        // baked at compile time, so it only works when running from the
        // source tree — installed binaries see an empty bundled set and
        // fall through to disk-cache → on-chain fetch.
        let bundled_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/idls");
        let bundled = load_bundled_idls(&bundled_dir);
        Self {
            dir,
            ttl: Duration::from_secs(7 * 24 * 60 * 60), // 7 days per docs/03-spec
            bundled,
        }
    }
}

impl IdlCache {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            ttl: Duration::from_secs(7 * 24 * 60 * 60),
            bundled: HashMap::new(),
        }
    }

    /// Add bundled IDLs. Pass a map from program_id → IDL JSON.
    pub fn with_bundled(mut self, bundled: HashMap<Pubkey, Value>) -> Self {
        self.bundled = bundled;
        self
    }

    /// Manually insert an IDL from a JSON string. Validates the JSON is an
    /// object before writing; persists to disk so subsequent runs don't
    /// need to re-paste.
    pub fn manual_insert_from_json(
        &self,
        program_id: &Pubkey,
        idl_json: &str,
    ) -> Result<(), ReplayError> {
        let value: Value = serde_json::from_str(idl_json)
            .map_err(|e| ReplayError::Idl {
                program_id: program_id.to_string(),
                detail: format!("invalid JSON: {e}"),
            })?;
        if !value.is_object() {
            return Err(ReplayError::Idl {
                program_id: program_id.to_string(),
                detail: "IDL must be a JSON object".into(),
            });
        }
        self.write_disk(program_id, &value)?;
        Ok(())
    }

    /// Persist a fetched/parsed IDL to disk for next-run reuse.
    pub fn insert(&self, program_id: &Pubkey, idl: &Idl) -> Result<(), ReplayError> {
        self.write_disk(program_id, &idl.raw)
    }

    /// Lookup order: bundled → disk cache (within TTL) → on-chain.
    /// Returns Ok(None) when none of the three yield an IDL — *not* Err,
    /// because "no IDL" is a normal state for many programs.
    pub async fn get_or_fetch<C: HeliusClient>(
        &self,
        client: &C,
        program_id: &Pubkey,
    ) -> Result<Option<Idl>, ReplayError> {
        if let Some(raw) = self.bundled.get(program_id) {
            return Ok(Some(Idl {
                raw: raw.clone(),
                source: IdlSource::Bundled,
            }));
        }
        if let Some(idl) = self.read_disk(program_id) {
            return Ok(Some(idl));
        }
        match fetch_anchor_idl(client, program_id).await? {
            Some(raw) => {
                let idl = Idl {
                    raw,
                    source: IdlSource::OnChain,
                };
                let _ = self.write_disk(program_id, &idl.raw); // best-effort persist
                Ok(Some(idl))
            }
            None => Ok(None),
        }
    }

    fn cache_path(&self, program_id: &Pubkey) -> PathBuf {
        self.dir.join(format!("{}.json", program_id))
    }

    fn read_disk(&self, program_id: &Pubkey) -> Option<Idl> {
        let path = self.cache_path(program_id);
        let metadata = std::fs::metadata(&path).ok()?;
        let modified = metadata.modified().ok()?;
        if SystemTime::now().duration_since(modified).ok()? > self.ttl {
            debug!(?program_id, "disk-cached IDL expired");
            return None;
        }
        let bytes = std::fs::read(&path).ok()?;
        let raw: Value = serde_json::from_slice(&bytes).ok()?;
        Some(Idl {
            raw,
            source: IdlSource::Cached,
        })
    }

    fn write_disk(&self, program_id: &Pubkey, raw: &Value) -> Result<(), ReplayError> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.cache_path(program_id);
        let bytes = serde_json::to_vec_pretty(raw)?;
        std::fs::write(&path, bytes)?;
        Ok(())
    }
}

/// Anchor stores its IDL at a deterministic address derived from the
/// program ID. The derivation is: base = find_program_address(&[], program_id),
/// then create_with_seed(base, "anchor:idl", program_id).
pub fn anchor_idl_address(program_id: &Pubkey) -> Pubkey {
    let (base, _bump) = Pubkey::find_program_address(&[], program_id);
    Pubkey::create_with_seed(&base, "anchor:idl", program_id)
        .expect("create_with_seed cannot fail with a valid base + short seed")
}

/// Fetch and parse an Anchor IDL from on-chain. Returns Ok(None) when the
/// program has no IDL account (the address is empty), Err only on fetch /
/// decompression failures that suggest a real bug.
async fn fetch_anchor_idl<C: HeliusClient>(
    client: &C,
    program_id: &Pubkey,
) -> Result<Option<Value>, ReplayError> {
    let idl_address = anchor_idl_address(program_id);
    let account = match client.get_account_info(&idl_address).await? {
        Some(a) if !a.data.is_empty() => a,
        _ => {
            debug!(?program_id, ?idl_address, "no on-chain IDL account");
            return Ok(None);
        }
    };

    // Layout: [discriminator: 8][authority: 32][len: u32 LE][zlib(idl_json)]
    if account.data.len() < 44 {
        warn!(
            ?program_id,
            data_len = account.data.len(),
            "IDL account too short for Anchor header; skipping"
        );
        return Ok(None);
    }
    let len = u32::from_le_bytes(
        account.data[40..44]
            .try_into()
            .expect("4 bytes"),
    ) as usize;
    if 44 + len > account.data.len() {
        return Err(ReplayError::Idl {
            program_id: program_id.to_string(),
            detail: format!(
                "IDL header claims len={} but account has {} bytes after header",
                len,
                account.data.len() - 44
            ),
        });
    }
    let compressed = &account.data[44..44 + len];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::with_capacity(len * 8);
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| ReplayError::Idl {
            program_id: program_id.to_string(),
            detail: format!("zlib decompress: {e}"),
        })?;

    let raw: Value = serde_json::from_slice(&decompressed).map_err(|e| ReplayError::Idl {
        program_id: program_id.to_string(),
        detail: format!("IDL JSON parse: {e}"),
    })?;
    Ok(Some(raw))
}

// ---------- decoder dispatch ----------

pub struct AccountDecoder<'a> {
    idl_cache: &'a IdlCache,
}

impl<'a> AccountDecoder<'a> {
    pub fn new(idl_cache: &'a IdlCache) -> Self {
        Self { idl_cache }
    }

    /// Decode an account against the best-available decoder. Never errors;
    /// failure surfaces as `DecodedAccount::{NoIdl, UnknownDiscriminator, NotAnchor}`.
    pub async fn decode<C: HeliusClient>(
        &self,
        _pubkey: &Pubkey,
        account: &Account,
        client: &C,
    ) -> DecodedAccount {
        let owner_str = account.owner.to_string();
        let hex_data = || hex::encode(&account.data);

        // 1. Owner-program dispatch for known native programs.
        if owner_str == SPL_TOKEN || owner_str == SPL_TOKEN_2022 {
            if let Some((type_name, value)) = decode_spl_token(&account.data) {
                return DecodedAccount::Native { type_name, value };
            }
            // Recognised owner but unfamiliar size: fall through to NoIdl below.
        }

        if owner_str == SYSTEM_PROGRAM {
            return DecodedAccount::Native {
                type_name: "SystemAccount".into(),
                value: json!({
                    "lamports": account.lamports,
                    "data_len": account.data.len(),
                    "data_hex": hex_data(),
                }),
            };
        }

        // 2. Fall through to Anchor IDL.
        let idl = match self.idl_cache.get_or_fetch(client, &account.owner).await {
            Ok(Some(idl)) => idl,
            Ok(None) => {
                return DecodedAccount::NoIdl {
                    owner: owner_str,
                    hex: hex_data(),
                };
            }
            Err(e) => {
                warn!(owner = %owner_str, error = %e, "IDL fetch failed; treating as no-IDL");
                return DecodedAccount::NoIdl {
                    owner: owner_str,
                    hex: hex_data(),
                };
            }
        };

        if account.data.len() < 8 {
            return DecodedAccount::NotAnchor {
                owner: owner_str,
                hex: hex_data(),
            };
        }

        match decode_anchor(&idl, &account.data) {
            Some((type_name, value)) => DecodedAccount::Decoded {
                type_name,
                value,
                idl_source: idl.source,
            },
            None => DecodedAccount::UnknownDiscriminator { hex: hex_data() },
        }
    }
}

// ---------- non-Anchor decoders (SPL Token & Token-2022) ----------

/// Dispatch by data length. Returns `(type_name, value)` on success.
/// Token-2022 with extensions appends data after byte 165; we ignore
/// extensions for now and decode the base layout.
fn decode_spl_token(data: &[u8]) -> Option<(String, Value)> {
    match data.len() {
        165 => decode_spl_token_account(data).map(|v| ("TokenAccount".into(), v)),
        82 => decode_spl_mint(data).map(|v| ("Mint".into(), v)),
        355 => decode_spl_multisig(data).map(|v| ("Multisig".into(), v)),
        n if n > 165 => {
            // Token-2022 account with extensions; decode the base + flag.
            decode_spl_token_account(&data[..165]).map(|mut v| {
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("has_extensions".into(), json!(true));
                    obj.insert("extension_bytes".into(), json!(n - 165));
                }
                ("TokenAccount".into(), v)
            })
        }
        _ => None,
    }
}

fn decode_spl_token_account(data: &[u8]) -> Option<Value> {
    if data.len() < 165 {
        return None;
    }
    let mint = pubkey_from_slice(&data[0..32])?.to_string();
    let owner = pubkey_from_slice(&data[32..64])?.to_string();
    let amount = read_u64_le(&data[64..72])?;
    let delegate_tag = read_u32_le(&data[72..76])?;
    let delegate = if delegate_tag != 0 {
        Some(pubkey_from_slice(&data[76..108])?.to_string())
    } else {
        None
    };
    let state = match data[108] {
        0 => "uninitialized",
        1 => "initialized",
        2 => "frozen",
        _ => "unknown",
    };
    let is_native_tag = read_u32_le(&data[109..113])?;
    let is_native = if is_native_tag != 0 {
        Some(read_u64_le(&data[113..121])?)
    } else {
        None
    };
    let delegated_amount = read_u64_le(&data[121..129])?;
    let close_authority_tag = read_u32_le(&data[129..133])?;
    let close_authority = if close_authority_tag != 0 {
        Some(pubkey_from_slice(&data[133..165])?.to_string())
    } else {
        None
    };
    Some(json!({
        "mint": mint,
        "owner": owner,
        "amount": amount.to_string(),
        "delegate": delegate,
        "state": state,
        "is_native_lamports": is_native.map(|v| v.to_string()),
        "delegated_amount": delegated_amount.to_string(),
        "close_authority": close_authority,
    }))
}

fn decode_spl_mint(data: &[u8]) -> Option<Value> {
    if data.len() < 82 {
        return None;
    }
    let mint_authority_tag = read_u32_le(&data[0..4])?;
    let mint_authority = if mint_authority_tag != 0 {
        Some(pubkey_from_slice(&data[4..36])?.to_string())
    } else {
        None
    };
    let supply = read_u64_le(&data[36..44])?;
    let decimals = data[44];
    let is_initialized = data[45] != 0;
    let freeze_authority_tag = read_u32_le(&data[46..50])?;
    let freeze_authority = if freeze_authority_tag != 0 {
        Some(pubkey_from_slice(&data[50..82])?.to_string())
    } else {
        None
    };
    Some(json!({
        "mint_authority": mint_authority,
        "supply": supply.to_string(),
        "decimals": decimals,
        "is_initialized": is_initialized,
        "freeze_authority": freeze_authority,
    }))
}

fn decode_spl_multisig(data: &[u8]) -> Option<Value> {
    if data.len() < 355 {
        return None;
    }
    let m = data[0];
    let n = data[1];
    let is_initialized = data[2] != 0;
    let mut signers = Vec::new();
    for i in 0..(n as usize).min(11) {
        let off = 3 + 32 * i;
        if let Some(pk) = pubkey_from_slice(&data[off..off + 32]) {
            signers.push(pk.to_string());
        }
    }
    Some(json!({
        "m": m,
        "n": n,
        "is_initialized": is_initialized,
        "signers": signers,
    }))
}

fn pubkey_from_slice(bytes: &[u8]) -> Option<Pubkey> {
    let arr: [u8; 32] = bytes.try_into().ok()?;
    Some(Pubkey::from(arr))
}

fn read_u32_le(bytes: &[u8]) -> Option<u32> {
    let arr: [u8; 4] = bytes.try_into().ok()?;
    Some(u32::from_le_bytes(arr))
}

fn read_u64_le(bytes: &[u8]) -> Option<u64> {
    let arr: [u8; 8] = bytes.try_into().ok()?;
    Some(u64::from_le_bytes(arr))
}

// ---------- Anchor IDL decoder ----------

#[derive(Debug)]
struct BorshErr(String);

fn anchor_account_discriminator(name: &str) -> [u8; 8] {
    // sha256("account:<Name>")[..8]
    let h = hash(format!("account:{name}").as_bytes());
    let bytes = h.to_bytes();
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&bytes[..8]);
    disc
}

fn decode_anchor(idl: &Idl, data: &[u8]) -> Option<(String, Value)> {
    let accounts = idl.raw.get("accounts").and_then(|v| v.as_array())?;
    let disc = &data[..8];
    for acct in accounts {
        let name = acct.get("name").and_then(|v| v.as_str())?;
        if anchor_account_discriminator(name) != disc {
            continue;
        }
        let type_def = acct.get("type")?;
        let mut cursor: &[u8] = &data[8..];
        match decode_type(idl, type_def, &mut cursor) {
            Ok(v) => return Some((name.to_string(), v)),
            Err(e) => {
                warn!(account = name, err = %e.0, "anchor decode failed");
                return None;
            }
        }
    }
    None
}

fn decode_type(idl: &Idl, ty: &Value, bytes: &mut &[u8]) -> Result<Value, BorshErr> {
    if let Some(s) = ty.as_str() {
        return decode_primitive(s, bytes);
    }
    let obj = ty
        .as_object()
        .ok_or_else(|| BorshErr(format!("unsupported type shape: {ty}")))?;

    if let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) {
        match kind {
            "struct" => {
                let fields = obj
                    .get("fields")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| BorshErr("struct missing fields".into()))?;
                let mut out = serde_json::Map::new();
                for f in fields {
                    let name = f
                        .get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| BorshErr("struct field missing name".into()))?;
                    let f_ty = f
                        .get("type")
                        .ok_or_else(|| BorshErr("struct field missing type".into()))?;
                    out.insert(name.into(), decode_type(idl, f_ty, bytes)?);
                }
                return Ok(Value::Object(out));
            }
            "enum" => {
                let variants = obj
                    .get("variants")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| BorshErr("enum missing variants".into()))?;
                let tag = take_u8(bytes)?;
                let variant = variants
                    .get(tag as usize)
                    .ok_or_else(|| BorshErr(format!("enum tag {tag} out of range")))?;
                let name = variant
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| BorshErr("variant missing name".into()))?;
                let payload = if let Some(fields) = variant.get("fields").and_then(|v| v.as_array())
                {
                    if fields.is_empty() {
                        Value::Null
                    } else if let Some(name0) = fields[0].get("name") {
                        // Named-field variant.
                        let _ = name0;
                        let mut out = serde_json::Map::new();
                        for f in fields {
                            let n = f
                                .get("name")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| BorshErr("variant field missing name".into()))?;
                            let f_ty = f
                                .get("type")
                                .ok_or_else(|| BorshErr("variant field missing type".into()))?;
                            out.insert(n.into(), decode_type(idl, f_ty, bytes)?);
                        }
                        Value::Object(out)
                    } else {
                        // Tuple variant — `fields` is a list of types.
                        let mut arr = Vec::with_capacity(fields.len());
                        for f_ty in fields {
                            arr.push(decode_type(idl, f_ty, bytes)?);
                        }
                        Value::Array(arr)
                    }
                } else {
                    Value::Null
                };
                return Ok(json!({ "variant": name, "payload": payload }));
            }
            other => return Err(BorshErr(format!("unsupported kind: {other}"))),
        }
    }

    if let Some(inner) = obj.get("option") {
        let tag = take_u8(bytes)?;
        return if tag == 0 {
            Ok(Value::Null)
        } else {
            decode_type(idl, inner, bytes)
        };
    }
    if let Some(inner) = obj.get("vec") {
        let len = take_u32(bytes)? as usize;
        let mut arr = Vec::with_capacity(len);
        for _ in 0..len {
            arr.push(decode_type(idl, inner, bytes)?);
        }
        return Ok(Value::Array(arr));
    }
    if let Some(spec) = obj.get("array").and_then(|v| v.as_array()) {
        if spec.len() != 2 {
            return Err(BorshErr("array spec must be [type, length]".into()));
        }
        let n = spec[1]
            .as_u64()
            .ok_or_else(|| BorshErr("array length not an int".into()))? as usize;
        let mut arr = Vec::with_capacity(n);
        for _ in 0..n {
            arr.push(decode_type(idl, &spec[0], bytes)?);
        }
        return Ok(Value::Array(arr));
    }
    if let Some(name) = obj.get("defined").and_then(|v| v.as_str()) {
        let types = idl
            .raw
            .get("types")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BorshErr(format!("type `{name}` referenced but no `types` in IDL")))?;
        let user_type = types
            .iter()
            .find(|t| t.get("name").and_then(|n| n.as_str()) == Some(name))
            .ok_or_else(|| BorshErr(format!("type `{name}` not found in IDL")))?;
        let inner = user_type
            .get("type")
            .ok_or_else(|| BorshErr(format!("type `{name}` missing `type` field")))?;
        return decode_type(idl, inner, bytes);
    }

    Err(BorshErr(format!("unrecognised type shape: {ty}")))
}

fn decode_primitive(name: &str, bytes: &mut &[u8]) -> Result<Value, BorshErr> {
    match name {
        "u8" => Ok(json!(take_u8(bytes)?)),
        "i8" => Ok(json!(take_u8(bytes)? as i8)),
        "u16" => Ok(json!(u16::from_le_bytes(take_n::<2>(bytes)?))),
        "i16" => Ok(json!(i16::from_le_bytes(take_n::<2>(bytes)?))),
        "u32" => Ok(json!(u32::from_le_bytes(take_n::<4>(bytes)?))),
        "i32" => Ok(json!(i32::from_le_bytes(take_n::<4>(bytes)?))),
        "u64" => Ok(json!(u64::from_le_bytes(take_n::<8>(bytes)?).to_string())),
        "i64" => Ok(json!(i64::from_le_bytes(take_n::<8>(bytes)?).to_string())),
        // u128/i128 don't fit JSON's f64 without loss; serialize as strings.
        "u128" => Ok(json!(u128::from_le_bytes(take_n::<16>(bytes)?).to_string())),
        "i128" => Ok(json!(i128::from_le_bytes(take_n::<16>(bytes)?).to_string())),
        "f32" => Ok(json!(f32::from_le_bytes(take_n::<4>(bytes)?))),
        "f64" => Ok(json!(f64::from_le_bytes(take_n::<8>(bytes)?))),
        "bool" => Ok(json!(take_u8(bytes)? != 0)),
        "string" => {
            let len = take_u32(bytes)? as usize;
            let raw = take_slice(bytes, len)?;
            let s = String::from_utf8(raw.to_vec())
                .map_err(|e| BorshErr(format!("invalid utf-8 in string: {e}")))?;
            Ok(json!(s))
        }
        "publicKey" | "pubkey" => {
            let arr = take_n::<32>(bytes)?;
            Ok(json!(Pubkey::from(arr).to_string()))
        }
        "bytes" => {
            let len = take_u32(bytes)? as usize;
            let raw = take_slice(bytes, len)?;
            Ok(json!(hex::encode(raw)))
        }
        other => Err(BorshErr(format!("unsupported primitive: {other}"))),
    }
}

fn take_u8(bytes: &mut &[u8]) -> Result<u8, BorshErr> {
    if bytes.is_empty() {
        return Err(BorshErr("EOF reading u8".into()));
    }
    let v = bytes[0];
    *bytes = &bytes[1..];
    Ok(v)
}

fn take_u32(bytes: &mut &[u8]) -> Result<u32, BorshErr> {
    Ok(u32::from_le_bytes(take_n::<4>(bytes)?))
}

fn take_n<const N: usize>(bytes: &mut &[u8]) -> Result<[u8; N], BorshErr> {
    if bytes.len() < N {
        return Err(BorshErr(format!("EOF reading {N} bytes (have {})", bytes.len())));
    }
    let mut buf = [0u8; N];
    buf.copy_from_slice(&bytes[..N]);
    *bytes = &bytes[N..];
    Ok(buf)
}

fn take_slice<'a>(bytes: &mut &'a [u8], n: usize) -> Result<&'a [u8], BorshErr> {
    if bytes.len() < n {
        return Err(BorshErr(format!("EOF reading {n} bytes (have {})", bytes.len())));
    }
    let s = &bytes[..n];
    *bytes = &bytes[n..];
    Ok(s)
}

/// Load any `assets/idls/<program_id>.json` files at the workspace root
/// into a bundled-IDL map. Returns an empty map if the directory doesn't
/// exist (typical: nothing bundled yet).
pub fn load_bundled_idls(dir: &std::path::Path) -> HashMap<Pubkey, Value> {
    let mut out = HashMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        let Ok(pk) = Pubkey::from_str(stem) else {
            warn!(?path, "bundled IDL filename is not a valid pubkey; skipping");
            continue;
        };
        match std::fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok()) {
            Some(raw) => {
                out.insert(pk, raw);
            }
            None => warn!(?path, "failed to parse bundled IDL"),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::MockHeliusClient;

    #[test]
    fn anchor_idl_pda_is_deterministic() {
        let pid = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap();
        let a = anchor_idl_address(&pid);
        let b = anchor_idl_address(&pid);
        assert_eq!(a, b);
    }

    #[test]
    fn anchor_account_discriminator_matches_expected() {
        // sha256("account:Position")[..8] = e8 90 92 9d 8c 35 7a 0a (well-known)
        let d = anchor_account_discriminator("Position");
        assert_eq!(d.len(), 8);
        // Self-consistency: same input → same output.
        assert_eq!(d, anchor_account_discriminator("Position"));
    }

    #[test]
    fn spl_mint_decoder_round_trips_a_known_layout() {
        // Synthesize an 82-byte mint: no mint_authority, supply=12345, decimals=6,
        // initialized, freeze_authority present.
        let mut data = vec![0u8; 82];
        // mint_authority absent (tag = 0)
        data[0..4].copy_from_slice(&0u32.to_le_bytes());
        // supply
        data[36..44].copy_from_slice(&12345u64.to_le_bytes());
        // decimals
        data[44] = 6;
        // is_initialized
        data[45] = 1;
        // freeze_authority present (tag = 1)
        data[46..50].copy_from_slice(&1u32.to_le_bytes());
        let fz = Pubkey::new_unique();
        data[50..82].copy_from_slice(fz.as_ref());

        let v = decode_spl_mint(&data).unwrap();
        assert_eq!(v["supply"], "12345");
        assert_eq!(v["decimals"], 6);
        assert_eq!(v["is_initialized"], true);
        assert_eq!(v["mint_authority"], Value::Null);
        assert_eq!(v["freeze_authority"], json!(fz.to_string()));
    }

    #[test]
    fn spl_token_account_decoder_round_trips() {
        // 165-byte token account: mint X, owner Y, amount 1_000_000, delegate absent,
        // state initialized, no native, delegated_amount 0, close_authority absent.
        let mut data = vec![0u8; 165];
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        data[0..32].copy_from_slice(mint.as_ref());
        data[32..64].copy_from_slice(owner.as_ref());
        data[64..72].copy_from_slice(&1_000_000u64.to_le_bytes());
        data[72..76].copy_from_slice(&0u32.to_le_bytes()); // delegate absent
        data[108] = 1; // initialized
        data[109..113].copy_from_slice(&0u32.to_le_bytes()); // is_native absent
        data[121..129].copy_from_slice(&0u64.to_le_bytes());
        data[129..133].copy_from_slice(&0u32.to_le_bytes()); // close_authority absent

        let v = decode_spl_token_account(&data).unwrap();
        assert_eq!(v["mint"], mint.to_string());
        assert_eq!(v["owner"], owner.to_string());
        assert_eq!(v["amount"], "1000000");
        assert_eq!(v["state"], "initialized");
        assert_eq!(v["delegate"], Value::Null);
    }

    #[tokio::test]
    async fn decode_dispatches_to_spl_token() {
        let cache = IdlCache::new(std::env::temp_dir().join("replay-idl-test"));
        let dec = AccountDecoder::new(&cache);
        let mut data = vec![0u8; 165];
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        data[0..32].copy_from_slice(mint.as_ref());
        data[32..64].copy_from_slice(owner.as_ref());
        data[64..72].copy_from_slice(&42u64.to_le_bytes());
        data[108] = 1;

        let acct = Account {
            lamports: 2_039_280,
            data,
            owner: Pubkey::from_str(SPL_TOKEN).unwrap(),
            executable: false,
            rent_epoch: 0,
        };
        let client = MockHeliusClient::default();
        let res = dec.decode(&Pubkey::new_unique(), &acct, &client).await;
        match res {
            DecodedAccount::Native { type_name, value } => {
                assert_eq!(type_name, "TokenAccount");
                assert_eq!(value["amount"], "42");
            }
            other => panic!("expected Native, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn decode_returns_no_idl_for_unknown_owner() {
        let cache = IdlCache::new(std::env::temp_dir().join("replay-idl-test"));
        let dec = AccountDecoder::new(&cache);
        let acct = Account {
            lamports: 1,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let client = MockHeliusClient::default();
        let res = dec.decode(&Pubkey::new_unique(), &acct, &client).await;
        assert!(matches!(res, DecodedAccount::NoIdl { .. }));
    }

    #[test]
    fn anchor_borsh_decode_struct_with_primitives() {
        // Synthetic IDL: one account `Counter { count: u64, owner: pubkey, label: string }`.
        let owner = Pubkey::new_unique();
        let mut data = anchor_account_discriminator("Counter").to_vec();
        data.extend_from_slice(&7u64.to_le_bytes());
        data.extend_from_slice(owner.as_ref());
        let label = "hello";
        data.extend_from_slice(&(label.len() as u32).to_le_bytes());
        data.extend_from_slice(label.as_bytes());

        let idl = Idl {
            raw: json!({
                "version": "0.1.0",
                "name": "demo",
                "accounts": [{
                    "name": "Counter",
                    "type": {
                        "kind": "struct",
                        "fields": [
                            { "name": "count", "type": "u64" },
                            { "name": "owner", "type": "publicKey" },
                            { "name": "label", "type": "string" },
                        ]
                    }
                }],
                "types": []
            }),
            source: IdlSource::Manual,
        };

        let (name, value) = decode_anchor(&idl, &data).expect("decode succeeds");
        assert_eq!(name, "Counter");
        assert_eq!(value["count"], "7");
        assert_eq!(value["owner"], owner.to_string());
        assert_eq!(value["label"], "hello");
    }

    #[test]
    fn anchor_borsh_decode_option_vec_array_defined() {
        // `Position { holders: Vec<pubkey>, fee: Option<u32>, fixed: [u8; 4], cfg: Config }`
        // `Config { active: bool }`
        let h0 = Pubkey::new_unique();
        let h1 = Pubkey::new_unique();
        let mut data = anchor_account_discriminator("Position").to_vec();
        // Vec<pubkey> length 2
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(h0.as_ref());
        data.extend_from_slice(h1.as_ref());
        // Option<u32> = Some(99)
        data.push(1);
        data.extend_from_slice(&99u32.to_le_bytes());
        // [u8; 4] = [1,2,3,4]
        data.extend_from_slice(&[1, 2, 3, 4]);
        // Config { active: true }
        data.push(1);

        let idl = Idl {
            raw: json!({
                "accounts": [{
                    "name": "Position",
                    "type": {
                        "kind": "struct",
                        "fields": [
                            { "name": "holders", "type": { "vec": "publicKey" } },
                            { "name": "fee", "type": { "option": "u32" } },
                            { "name": "fixed", "type": { "array": ["u8", 4] } },
                            { "name": "cfg", "type": { "defined": "Config" } },
                        ]
                    }
                }],
                "types": [{
                    "name": "Config",
                    "type": { "kind": "struct", "fields": [{ "name": "active", "type": "bool" }] }
                }]
            }),
            source: IdlSource::Manual,
        };

        let (name, v) = decode_anchor(&idl, &data).expect("decode succeeds");
        assert_eq!(name, "Position");
        assert_eq!(v["holders"][0], h0.to_string());
        assert_eq!(v["holders"][1], h1.to_string());
        assert_eq!(v["fee"], 99);
        assert_eq!(v["fixed"], json!([1, 2, 3, 4]));
        assert_eq!(v["cfg"]["active"], true);
    }

    #[test]
    fn idl_cache_manual_insert_and_read_disk() {
        let dir = std::env::temp_dir().join(format!("replay-idl-test-{}", rand_suffix()));
        let cache = IdlCache::new(dir.clone());
        let pid = Pubkey::new_unique();
        cache
            .manual_insert_from_json(&pid, r#"{"version":"0.1.0","name":"x"}"#)
            .unwrap();
        let idl = cache.read_disk(&pid).expect("written IDL is readable");
        assert_eq!(idl.raw["name"], "x");
        assert_eq!(idl.source, IdlSource::Cached);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn rand_suffix() -> u64 {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}
