//! Shared app state — session store + RPC client.

use anyhow::Context;
use dashmap::DashMap;
use replay_core::{ForkedSession, HeliusClient, HeliusRpcClient};
use std::sync::Arc;
use std::time::Duration;

pub struct AppState {
    pub sessions: DashMap<String, ForkedSession>,
    pub client: Arc<dyn HeliusClient>,
    pub session_ttl: Duration,
    pub max_sessions: usize,
}

impl AppState {
    pub fn from_env() -> anyhow::Result<Self> {
        let client = match std::env::var("REPLAY_RPC_URL") {
            Ok(url) => HeliusRpcClient::from_url(url)?,
            Err(_) => {
                let api_key = std::env::var("HELIUS_API_KEY")
                    .context("set HELIUS_API_KEY or REPLAY_RPC_URL")?;
                HeliusRpcClient::from_api_key(&api_key)?
            }
        };

        let ttl_secs: u64 = std::env::var("REPLAY_SESSION_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600);

        let max_sessions: usize = std::env::var("REPLAY_MAX_SESSIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        Ok(Self {
            sessions: DashMap::new(),
            client: Arc::new(client),
            session_ttl: Duration::from_secs(ttl_secs),
            max_sessions,
        })
    }

    #[doc(hidden)]
    pub fn with_client(client: impl HeliusClient + 'static) -> Self {
        Self {
            sessions: DashMap::new(),
            client: Arc::new(client),
            session_ttl: Duration::from_secs(3600),
            max_sessions: 100,
        }
    }

    /// Remove expired sessions. Called periodically by a background task,
    /// but also safe to call opportunistically on each request.
    pub fn prune_expired(&self) {
        let ttl = self.session_ttl;
        let now = std::time::Instant::now();
        self.sessions
            .retain(|_id, s| now.duration_since(s.created_at) < ttl);

        // Also enforce the max_sessions hard cap by evicting oldest.
        while self.sessions.len() > self.max_sessions {
            let oldest = self
                .sessions
                .iter()
                .min_by_key(|e| e.value().created_at)
                .map(|e| e.key().clone());
            if let Some(id) = oldest {
                self.sessions.remove(&id);
            } else {
                break;
            }
        }
    }
}
