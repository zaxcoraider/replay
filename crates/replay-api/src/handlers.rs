//! HTTP handlers. Thin — all real work lives in replay-core.

use axum::{
    extract::{Path, State},
    Json,
};
use replay_core::{AccountMutation, ReplayError, Trace, TraceDiff};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;

use crate::error::ApiError;
use crate::state::AppState;

pub async fn health() -> &'static str {
    "ok"
}

#[derive(Serialize)]
pub struct VersionInfo {
    name: &'static str,
    version: &'static str,
}

pub async fn version() -> Json<VersionInfo> {
    Json(VersionInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Deserialize)]
pub struct ReplayReq {
    pub signature: String,
}

#[derive(Serialize)]
pub struct ReplayResp {
    pub trace: Trace,
}

pub async fn replay(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReplayReq>,
) -> Result<Json<ReplayResp>, ApiError> {
    let trace = replay_core::replay(&req.signature, &state.client).await?;
    Ok(Json(ReplayResp { trace }))
}

#[derive(Deserialize)]
pub struct ForkReq {
    pub signature: String,
}

#[derive(Serialize)]
pub struct ForkResp {
    pub session_id: String,
    pub baseline_trace: Trace,
    pub expires_at: String,
}

pub async fn fork(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForkReq>,
) -> Result<Json<ForkResp>, ApiError> {
    state.prune_expired();

    let session = replay_core::fork(&req.signature, &state.client).await?;
    let id = session.id.clone();
    let baseline_trace = session.baseline_trace.clone();
    let expires_at = chrono::Utc::now()
        + chrono::Duration::from_std(state.session_ttl).unwrap_or(chrono::Duration::hours(1));

    state.sessions.insert(id.clone(), session);

    Ok(Json(ForkResp {
        session_id: id,
        baseline_trace,
        expires_at: expires_at.to_rfc3339(),
    }))
}

#[derive(Deserialize)]
pub struct MutateReq {
    pub pubkey: String,
    pub mutation: AccountMutation,
}

#[derive(Serialize)]
pub struct MutateResp {
    pub applied: bool,
    pub mutation_count: usize,
}

pub async fn mutate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<MutateReq>,
) -> Result<Json<MutateResp>, ApiError> {
    let mut entry = state
        .sessions
        .get_mut(&id)
        .ok_or_else(|| ApiError(ReplayError::SessionNotFound(id.clone())))?;

    let pk = Pubkey::from_str(&req.pubkey)
        .map_err(|e| ApiError(ReplayError::InvalidMutationPath {
            path: req.pubkey.clone(),
            type_name: format!("bad pubkey: {e:?}"),
        }))?;

    entry.mutate(pk, req.mutation)?;

    Ok(Json(MutateResp {
        applied: true,
        mutation_count: entry.mutations.len(),
    }))
}

#[derive(Serialize)]
pub struct ExecuteResp {
    pub trace: Trace,
    pub mutation_count: usize,
}

pub async fn execute(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ExecuteResp>, ApiError> {
    let mut entry = state
        .sessions
        .get_mut(&id)
        .ok_or_else(|| ApiError(ReplayError::SessionNotFound(id.clone())))?;

    let trace = entry.execute(&state.client).await?;
    Ok(Json(ExecuteResp {
        trace,
        mutation_count: entry.mutations.len(),
    }))
}

pub async fn diff(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TraceDiff>, ApiError> {
    let entry = state
        .sessions
        .get(&id)
        .ok_or_else(|| ApiError(ReplayError::SessionNotFound(id.clone())))?;

    let diff = entry
        .diff()
        .ok_or_else(|| ApiError(ReplayError::Execution(
            "session has no execution yet; call POST /session/:id/execute first".into(),
        )))?;

    Ok(Json(diff))
}
