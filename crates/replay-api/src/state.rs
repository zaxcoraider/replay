//! Shared app state — session store + RPC client.

use anyhow::Context;
use dashmap::DashMap;
use replay_core::{ForkedSession, HeliusClient, HeliusRpcClient};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct AppState {
    pub sessions: DashMap<String, ForkedSession>,
    pub client: Arc<dyn HeliusClient>,
    pub session_ttl: Duration,
    pub max_sessions: usize,
    pub live_sessions: LiveSessionLimiter,
}

/// Counts in-flight `/replay-live` SSE sessions. A `LiveSessionPermit` is
/// returned by [`LiveSessionLimiter::try_acquire`] and decrements the
/// counter when dropped, so callers can't leak slots on error paths.
pub struct LiveSessionLimiter {
    inflight: Arc<AtomicUsize>,
    cap: usize,
}

pub struct LiveSessionPermit {
    inflight: Arc<AtomicUsize>,
}

impl Drop for LiveSessionPermit {
    fn drop(&mut self) {
        self.inflight.fetch_sub(1, Ordering::Release);
    }
}

impl LiveSessionLimiter {
    pub fn new(cap: usize) -> Self {
        Self {
            inflight: Arc::new(AtomicUsize::new(0)),
            cap,
        }
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    /// Increment the in-flight counter if it is below the cap. CAS so
    /// concurrent acquires don't race past the limit.
    pub fn try_acquire(&self) -> Option<LiveSessionPermit> {
        let mut current = self.inflight.load(Ordering::Acquire);
        loop {
            if current >= self.cap {
                return None;
            }
            match self.inflight.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Some(LiveSessionPermit {
                        inflight: Arc::clone(&self.inflight),
                    });
                }
                Err(observed) => current = observed,
            }
        }
    }
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

        let max_live_sessions: usize = std::env::var("REPLAY_MAX_LIVE_SESSIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        Ok(Self {
            sessions: DashMap::new(),
            client: Arc::new(client),
            session_ttl: Duration::from_secs(ttl_secs),
            max_sessions,
            live_sessions: LiveSessionLimiter::new(max_live_sessions),
        })
    }

    #[doc(hidden)]
    pub fn with_client(client: impl HeliusClient + 'static) -> Self {
        Self {
            sessions: DashMap::new(),
            client: Arc::new(client),
            session_ttl: Duration::from_secs(3600),
            max_sessions: 100,
            live_sessions: LiveSessionLimiter::new(5),
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
