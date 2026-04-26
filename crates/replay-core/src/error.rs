//! Error types. Fail loudly, fail specifically — every variant carries the
//! context a user needs to understand and act on the failure.

use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("transaction not found on Helius")]
    TxNotFound,

    #[error("rpc error: {0}")]
    Rpc(String),

    #[error("http transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("state reconstruction failed at step `{step}`: {detail}")]
    StateReconstruction { step: Cow<'static, str>, detail: String },

    #[error("slot {slot} may be pruned; try a more recent transaction (Helius retains ~90 days)")]
    SlotPruned { slot: u64 },

    #[error("program {program_id} has no known bytecode at slot {slot}")]
    MissingProgramBytecode { program_id: String, slot: u64 },

    #[error("address lookup table {lut} resolution failed: {detail}")]
    LutResolution { lut: String, detail: String },

    #[error("execution error: {0}")]
    Execution(String),

    #[error("IDL parse error for program {program_id}: {detail}")]
    Idl { program_id: String, detail: String },

    #[error("account decoder error: {0}")]
    Decoder(String),

    #[error("mutation path `{path}` not valid for account type `{type_name}`")]
    InvalidMutationPath { path: String, type_name: String },

    #[error("session expired or not found: {0}")]
    SessionNotFound(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl ReplayError {
    /// Short, stable error code suitable for a JSON payload.
    /// Kept out of Display because the error message is already rich.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidSignature(_) => "INVALID_SIGNATURE",
            Self::TxNotFound => "TX_NOT_FOUND",
            Self::Rpc(_) | Self::Http(_) => "RPC_ERROR",
            Self::Serde(_) => "SERDE_ERROR",
            Self::StateReconstruction { .. } => "STATE_RECONSTRUCTION_FAILED",
            Self::SlotPruned { .. } => "SLOT_PRUNED",
            Self::MissingProgramBytecode { .. } => "MISSING_PROGRAM_BYTECODE",
            Self::LutResolution { .. } => "LUT_RESOLUTION_FAILED",
            Self::Execution(_) => "EXECUTION_ERROR",
            Self::Idl { .. } => "IDL_ERROR",
            Self::Decoder(_) => "DECODER_ERROR",
            Self::InvalidMutationPath { .. } => "INVALID_MUTATION_PATH",
            Self::SessionNotFound(_) => "SESSION_NOT_FOUND",
            Self::Io(_) => "IO_ERROR",
        }
    }
}
