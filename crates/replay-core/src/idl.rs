//! IDL-aware account decoder.
//!
//! This module is a skeleton for Day 3. The goal is to take an account's
//! bytes and produce structured JSON decoded against the owning program's
//! Anchor IDL. Falls back to hex + "paste your IDL" when the IDL can't
//! be found on-chain.

use crate::error::ReplayError;
use crate::rpc::HeliusClient;
use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecodedAccount {
    Decoded {
        type_name: String,
        value: serde_json::Value,
        idl_source: IdlSource,
    },
    UnknownDiscriminator {
        hex: String,
    },
    NoIdl {
        owner: String,
        hex: String,
    },
    NotAnchor {
        owner: String,
        hex: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdlSource {
    OnChain,
    Manual,
    Cached,
    Bundled,
}

/// Lightweight IDL representation — stores raw JSON internally and walks it
/// at decode time. Avoids a full Anchor toolchain dependency.
#[derive(Debug, Clone)]
pub struct Idl {
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct IdlCache {
    pub dir: PathBuf,
}

impl Default for IdlCache {
    fn default() -> Self {
        let dir = std::env::var("REPLAY_IDL_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home).join(".replay/idl-cache")
            });
        Self { dir }
    }
}

impl IdlCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub async fn get_or_fetch<C: HeliusClient>(
        &self,
        _client: &C,
        _program_id: &Pubkey,
    ) -> Result<Option<Idl>, ReplayError> {
        // Day 3: implement disk cache check + on-chain IDL fetch.
        // Reference: anchor_lang::idl::IdlAccount::address(program_id).
        // See docs/04-solana-gotchas.md and prompts/day-03-idl-decoder.md.
        Ok(None)
    }

    pub fn insert(&self, _program_id: &Pubkey, _idl: &Idl) -> Result<(), ReplayError> {
        // Day 3: write JSON to disk in self.dir.
        Ok(())
    }

    pub fn manual_insert_from_json(
        &self,
        _program_id: &Pubkey,
        _idl_json: &str,
    ) -> Result<(), ReplayError> {
        // Day 3: validate, then write to cache.
        Ok(())
    }
}

pub struct AccountDecoder<'a> {
    idl_cache: &'a IdlCache,
}

impl<'a> AccountDecoder<'a> {
    pub fn new(idl_cache: &'a IdlCache) -> Self {
        Self { idl_cache }
    }

    /// Decode an account against the best-available IDL. Skeleton — Day 3.
    pub async fn decode(
        &self,
        _pubkey: &Pubkey,
        account: &Account,
        _client: &impl HeliusClient,
    ) -> DecodedAccount {
        // Day 3 work starts here. For now return NoIdl so downstream code
        // has a sensible default.
        let _ = self.idl_cache; // used in real impl
        DecodedAccount::NoIdl {
            owner: account.owner.to_string(),
            hex: hex::encode(&account.data),
        }
    }
}
