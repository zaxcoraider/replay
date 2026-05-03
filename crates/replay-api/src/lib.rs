pub mod error;
pub mod handlers;
pub mod state;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn build_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/version", get(handlers::version))
        .route("/replay", post(handlers::replay))
        .route("/fork", post(handlers::fork))
        .route("/session/:id/mutate", post(handlers::mutate))
        .route("/session/:id/execute", post(handlers::execute))
        .route("/session/:id/diff", get(handlers::diff))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
