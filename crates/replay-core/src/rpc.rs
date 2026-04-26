//! Helius RPC client. A trait + concrete impl so tests can mock.
//!
//! The real implementation uses reqwest directly instead of `solana-client`
//! because we need JSON-RPC extras (Helius Enhanced APIs) and finer control
//! over `minContextSlot` / encoding options than the SDK exposes cleanly.

use crate::error::ReplayError;
use crate::types::{FetchedTx, FetchedTxMeta};
use async_trait::async_trait;
use serde_json::{json, Value};
use solana_sdk::{account::Account, pubkey::Pubkey, signature::Signature};
use std::time::Duration;
use tracing::{debug, warn};

#[async_trait]
pub trait HeliusClient: Send + Sync {
    /// Fetch a full transaction by signature. Uses `encoding: "base64"` so
    /// we can faithfully deserialize the versioned message.
    async fn get_transaction(&self, sig: &Signature) -> Result<Option<FetchedTx>, ReplayError>;

    /// Fetch account info at a specific slot, with `commitment: "confirmed"`
    /// and `minContextSlot: slot`. Returns `None` if the account does not exist.
    async fn get_account_info_at_slot(
        &self,
        pubkey: &Pubkey,
        slot: u64,
    ) -> Result<Option<Account>, ReplayError>;

    /// Fetch current account info (no slot constraint). Used for IDL accounts
    /// and similar cases where current state is what we want.
    async fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<Account>, ReplayError>;
}

#[derive(Clone)]
pub struct HeliusRpcClient {
    http: reqwest::Client,
    rpc_url: String,
}

impl HeliusRpcClient {
    /// Construct from a Helius API key. Uses the mainnet endpoint.
    pub fn from_api_key(api_key: &str) -> Result<Self, ReplayError> {
        Self::from_url(format!(
            "https://mainnet.helius-rpc.com/?api-key={}",
            api_key
        ))
    }

    /// Construct from an explicit RPC URL (any JSON-RPC-compatible Solana
    /// endpoint will work for the basic methods; historical queries are
    /// Helius-dependent).
    pub fn from_url(rpc_url: impl Into<String>) -> Result<Self, ReplayError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            http,
            rpc_url: rpc_url.into(),
        })
    }

    async fn json_rpc(&self, method: &str, params: Value) -> Result<Value, ReplayError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        for attempt in 0..3 {
            let res = self.http.post(&self.rpc_url).json(&body).send().await?;
            let status = res.status();

            if status.as_u16() == 429 {
                let wait = Duration::from_millis(500 * (1 << attempt));
                warn!(?wait, "helius rate limited, backing off");
                tokio::time::sleep(wait).await;
                continue;
            }

            if !status.is_success() {
                let text = res.text().await.unwrap_or_default();
                return Err(ReplayError::Rpc(format!(
                    "http {}: {}",
                    status.as_u16(),
                    text
                )));
            }

            let value: Value = res.json().await?;
            if let Some(err) = value.get("error") {
                return Err(ReplayError::Rpc(err.to_string()));
            }
            return Ok(value
                .get("result")
                .cloned()
                .unwrap_or(Value::Null));
        }

        Err(ReplayError::Rpc("exhausted retries on 429".to_string()))
    }
}

#[async_trait]
impl HeliusClient for HeliusRpcClient {
    #[tracing::instrument(skip(self))]
    async fn get_transaction(&self, sig: &Signature) -> Result<Option<FetchedTx>, ReplayError> {
        let params = json!([
            sig.to_string(),
            {
                "encoding": "base64",
                "commitment": "confirmed",
                "maxSupportedTransactionVersion": 0
            }
        ]);

        let raw = self.json_rpc("getTransaction", params).await?;
        if raw.is_null() {
            return Ok(None);
        }

        let slot = raw
            .get("slot")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ReplayError::Rpc("getTransaction response missing slot".into()))?;

        let block_time = raw.get("blockTime").and_then(|v| v.as_i64());

        let transaction_base64 = raw
            .get("transaction")
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ReplayError::Rpc("getTransaction response missing transaction[0]".into()))?
            .to_string();

        let meta_value = raw
            .get("meta")
            .cloned()
            .ok_or_else(|| ReplayError::Rpc("getTransaction response missing meta".into()))?;
        let meta: FetchedTxMeta = serde_json::from_value(meta_value)?;

        Ok(Some(FetchedTx {
            slot,
            block_time,
            transaction_base64,
            meta,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn get_account_info_at_slot(
        &self,
        pubkey: &Pubkey,
        slot: u64,
    ) -> Result<Option<Account>, ReplayError> {
        // Honors the Day-1 contract: pass `minContextSlot: slot` to the RPC.
        // CAVEAT: `minContextSlot` does NOT actually return historical state
        // — it asserts the node has observed the given slot before answering.
        // For TRUE historical state we rely on Helius enhanced APIs (Day 14)
        // or on the fact that for recent txs (<~few hours) current state is
        // close enough. See docs/04-solana-gotchas.md §12.
        self.get_account_info_inner(pubkey, Some(slot)).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<Account>, ReplayError> {
        self.get_account_info_inner(pubkey, None).await
    }
}

impl HeliusRpcClient {
    async fn get_account_info_inner(
        &self,
        pubkey: &Pubkey,
        min_context_slot: Option<u64>,
    ) -> Result<Option<Account>, ReplayError> {
        let mut config = serde_json::Map::new();
        config.insert("encoding".into(), Value::String("base64".into()));
        config.insert("commitment".into(), Value::String("confirmed".into()));
        if let Some(slot) = min_context_slot {
            config.insert(
                "minContextSlot".into(),
                Value::Number(serde_json::Number::from(slot)),
            );
        }

        let params = json!([pubkey.to_string(), Value::Object(config)]);

        let raw = self.json_rpc("getAccountInfo", params).await?;

        let v = match raw.get("value") {
            Some(v) if !v.is_null() => v,
            _ => {
                debug!(?pubkey, "account not found");
                return Ok(None);
            }
        };

        let lamports = v
            .get("lamports")
            .and_then(|x| x.as_u64())
            .ok_or_else(|| ReplayError::Rpc("account missing lamports".into()))?;

        let owner_str = v
            .get("owner")
            .and_then(|x| x.as_str())
            .ok_or_else(|| ReplayError::Rpc("account missing owner".into()))?;
        let owner = owner_str
            .parse::<Pubkey>()
            .map_err(|e| ReplayError::Rpc(format!("invalid owner pubkey: {e}")))?;

        let data_b64 = v
            .get("data")
            .and_then(|x| x.get(0))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        let data = if data_b64.is_empty() {
            Vec::new()
        } else {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(data_b64)
                .map_err(|e| ReplayError::Rpc(format!("base64 decode failed: {e}")))?
        };

        let executable = v.get("executable").and_then(|x| x.as_bool()).unwrap_or(false);
        let rent_epoch = v.get("rentEpoch").and_then(|x| x.as_u64()).unwrap_or(0);

        Ok(Some(Account {
            lamports,
            data,
            owner,
            executable,
            rent_epoch,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::MockHeliusClient;
    use crate::types::FetchedTxMeta;

    #[tokio::test]
    async fn mock_client_returns_canned_tx() {
        let sig = Signature::new_unique();
        let mut mock = MockHeliusClient::default();
        mock.txs.insert(
            sig.to_string(),
            FetchedTx {
                slot: 12345,
                block_time: Some(1_700_000_000),
                transaction_base64: String::new(),
                meta: FetchedTxMeta {
                    err: None,
                    log_messages: vec!["Program log: hi".to_string()],
                    pre_balances: vec![1_000_000_000],
                    post_balances: vec![999_995_000],
                    loaded_addresses: None,
                    compute_units_consumed: Some(1234),
                    inner_instructions: None,
                },
            },
        );

        let result = mock.get_transaction(&sig).await.unwrap().unwrap();
        assert_eq!(result.slot, 12345);
        assert_eq!(result.meta.log_messages.len(), 1);
    }
}
