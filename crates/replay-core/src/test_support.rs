//! Test-only helpers shared across the crate. Not exposed publicly.

use crate::error::ReplayError;
use crate::rpc::HeliusClient;
use crate::types::FetchedTx;
use async_trait::async_trait;
use solana_sdk::{account::Account, pubkey::Pubkey, signature::Signature};
use std::collections::HashMap;

/// Wire up canned responses per-test; keeps unit tests network-free.
#[derive(Default)]
pub struct MockHeliusClient {
    pub txs: HashMap<String, FetchedTx>,
    pub accounts: HashMap<Pubkey, Account>,
}

#[async_trait]
impl HeliusClient for MockHeliusClient {
    async fn get_transaction(&self, sig: &Signature) -> Result<Option<FetchedTx>, ReplayError> {
        Ok(self.txs.get(&sig.to_string()).cloned())
    }

    async fn get_account_info_at_slot(
        &self,
        pubkey: &Pubkey,
        _slot: u64,
    ) -> Result<Option<Account>, ReplayError> {
        Ok(self.accounts.get(pubkey).cloned())
    }

    async fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<Account>, ReplayError> {
        Ok(self.accounts.get(pubkey).cloned())
    }
}
