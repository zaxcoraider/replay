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

mod error;
mod handlers;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present.
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let bind = std::env::var("REPLAY_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8787".into());
    let addr: SocketAddr = bind.parse()?;

    let state = Arc::new(AppState::from_env()?);

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/version", get(handlers::version))
        .route("/replay", post(handlers::replay))
        .route("/fork", post(handlers::fork))
        .route("/session/:id/mutate", post(handlers::mutate))
        .route("/session/:id/execute", post(handlers::execute))
        .route("/session/:id/diff", get(handlers::diff))
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!(?addr, "replay-api listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
