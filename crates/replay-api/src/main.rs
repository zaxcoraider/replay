//! Replay API server — axum-based HTTP/WebSocket service.
//!
//! Routes:
//!   POST /replay                    one-shot replay
//!   POST /fork                      create a session
//!   POST /session/:id/mutate        add a mutation
//!   POST /session/:id/execute       re-run with mutations
//!   GET  /session/:id/diff          diff baseline vs latest
//!   GET  /health                    liveness
//!   GET  /version                   build info

use replay_api::{build_app, state::AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8787".into());
    let bind = std::env::var("REPLAY_BIND_ADDR")
        .unwrap_or_else(|_| format!("0.0.0.0:{}", port));
    let addr: SocketAddr = bind.parse()?;

    let state = Arc::new(AppState::from_env()?);

    // Rate limit: 20 requests/min per IP, burst of 20.
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(3_000) // 1 per 3 s ≈ 20/min
            .burst_size(20)
            .use_headers()
            .finish()
            .unwrap(),
    );
    let governor_limiter = governor_conf.limiter().clone();

    // Background tasks: prune expired sessions + rate-limiter entries.
    {
        let state_clone = Arc::clone(&state);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                state_clone.prune_expired();
                governor_limiter.retain_recent();
            }
        });
    }

    let app = build_app(Arc::clone(&state))
        .layer(GovernorLayer {
            config: Arc::clone(&governor_conf),
        })
        .layer(CorsLayer::very_permissive());

    tracing::info!(?addr, "replay-api listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
