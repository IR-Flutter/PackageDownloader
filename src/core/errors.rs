use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PdownError>;

#[derive(Error, Debug)]
pub enum PdownError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML serialization error: {0}")]
    Toml(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("HTTP network error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Mirror not found: '{0}'")]
    MirrorNotFound(String),

    #[error("Provider '{provider}' is unsupported: {reason}")]
    ProviderUnsupported { provider: String, reason: String },

    #[error("Registry validation failed: {0}")]
    RegistryValidation(String),

    #[error("Digital signature verification failed: {0}")]
    SignatureVerification(String),

    #[error("Schema version mismatch: expected <= {expected}, got {got}")]
    SchemaMismatch { expected: u32, got: u32 },

    #[error("Registry rollback rejected: active revision is {current}, proposed is {incoming}")]
    RollbackRejected { current: u64, incoming: u64 },

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Transaction failed: {transaction_error}. Rollback ALSO failed: {rollback_errors:?}. Recovery advice: {recovery_advice}")]
    TransactionAndRollbackFailed {
        transaction_error: String,
        rollback_errors: Vec<String>,
        recovery_advice: String,
    },

    #[error("Backup failure for path {path}: {message}")]
    BackupFailed { path: PathBuf, message: String },

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Unknown error: {0}")]
    Other(String),
}

impl PdownError {
    pub fn exit_code(&self) -> i32 {
        match self {
            PdownError::MirrorNotFound(_) => 3,
            PdownError::ProviderUnsupported { .. } => 4,
            PdownError::Http(_) => 5,
            PdownError::SignatureVerification(_) => 6,
            PdownError::SchemaMismatch { .. } | PdownError::RollbackRejected { .. } => 6,
            PdownError::TransactionFailed(_)
            | PdownError::TransactionAndRollbackFailed { .. }
            | PdownError::RollbackFailed(_) => 7,
            PdownError::InvalidConfig(_) => 2,
            _ => 1,
        }
    }
}
