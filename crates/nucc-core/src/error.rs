//! Error types shared by the container crates.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, NuccError>;

#[derive(Debug, Error)]
pub enum NuccError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid NUCC header")]
    InvalidHeader,

    #[error("invalid NUCC index: {0}")]
    InvalidIndex(String),

    #[error("invalid chunk: {0}")]
    InvalidChunk(String),

    #[error("unsupported NUCC feature: {0}")]
    Unsupported(String),

    #[error("offset overflow")]
    Overflow,

    #[error("buffer is truncated")]
    Truncated,
}
