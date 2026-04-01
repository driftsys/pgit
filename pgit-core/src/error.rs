use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PgitError {
    #[error("invalid PURL: {0}")]
    InvalidPurl(String),

    #[error("manifest parse error: {0}")]
    InvalidManifest(String),

    #[error("version not found: {0}")]
    VersionNotFound(String),

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("unsupported registry mode: {0}")]
    UnsupportedMode(String),
}
