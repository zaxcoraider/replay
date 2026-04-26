//! Unified error response for the API. Maps replay-core errors to HTTP
//! status codes and a consistent JSON shape:
//!   { "error": { "code": "...", "message": "...", "context": {...} } }

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use replay_core::ReplayError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: ApiErrorInner,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorInner {
    pub code: &'static str,
    pub message: String,
}

pub struct ApiError(pub ReplayError);

impl From<ReplayError> for ApiError {
    fn from(e: ReplayError) -> Self {
        Self(e)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self(ReplayError::Execution(e.to_string()))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let code = self.0.code();
        let message = self.0.to_string();

        let status = match &self.0 {
            ReplayError::InvalidSignature(_) => StatusCode::BAD_REQUEST,
            ReplayError::TxNotFound => StatusCode::NOT_FOUND,
            ReplayError::SessionNotFound(_) => StatusCode::NOT_FOUND,
            ReplayError::StateReconstruction { .. }
            | ReplayError::SlotPruned { .. }
            | ReplayError::MissingProgramBytecode { .. }
            | ReplayError::LutResolution { .. }
            | ReplayError::InvalidMutationPath { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            ReplayError::Rpc(_) | ReplayError::Http(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ApiErrorBody {
            error: ApiErrorInner { code, message },
        };
        (status, Json(body)).into_response()
    }
}
