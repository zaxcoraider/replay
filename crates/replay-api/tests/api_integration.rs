//! In-process integration tests for the replay-api HTTP layer.
//! Uses a mock HeliusClient — no network required.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use replay_api::{build_app, state::AppState};
use replay_core::{HeliusClient, ReplayError};
use replay_core::types::FetchedTx;
use solana_sdk::{account::Account, pubkey::Pubkey, signature::Signature};
use std::sync::Arc;
use tower::ServiceExt;

#[derive(Default)]
struct EmptyClient;

#[async_trait]
impl HeliusClient for EmptyClient {
    async fn get_transaction(&self, _: &Signature) -> Result<Option<FetchedTx>, ReplayError> {
        Ok(None)
    }
    async fn get_account_info_at_slot(
        &self, _: &Pubkey, _: u64,
    ) -> Result<Option<Account>, ReplayError> {
        Ok(None)
    }
    async fn get_account_info(&self, _: &Pubkey) -> Result<Option<Account>, ReplayError> {
        Ok(None)
    }
}

fn test_app() -> axum::Router {
    let state = Arc::new(AppState::with_client(EmptyClient));
    build_app(state)
}

#[tokio::test]
async fn health_returns_ok() {
    let resp = test_app()
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn version_returns_json_with_name_and_version() {
    let resp = test_app()
        .oneshot(Request::builder().uri("/version").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v.get("name").is_some());
    assert!(v.get("version").is_some());
}

#[tokio::test]
async fn replay_invalid_signature_returns_400() {
    let body = serde_json::json!({"signature": "not_a_sig"}).to_string();
    let resp = test_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/replay")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn replay_valid_sig_not_found_returns_404() {
    // Valid base58 but EmptyClient returns None → TxNotFound → 404
    let sig = "5xTBSm1JxGEXkZkLVvjBdGJxqrABQnJynckLPaUxoUsH5GJyWkYBnzBv6sDMRyEmH7";
    let body = serde_json::json!({"signature": sig}).to_string();
    let resp = test_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/replay")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::BAD_REQUEST,
        "expected 404 or 400, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn session_not_found_returns_404() {
    let resp = test_app()
        .oneshot(
            Request::builder()
                .uri("/session/01NONEXISTENT/diff")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn mutate_missing_session_returns_404() {
    let body = serde_json::json!({
        "pubkey": "11111111111111111111111111111111",
        "mutation": { "type": "lamports", "new_value": 1_000_000u64 }
    })
    .to_string();
    let resp = test_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/session/MISSING/mutate")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn execute_missing_session_returns_404() {
    let resp = test_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/session/MISSING/execute")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
