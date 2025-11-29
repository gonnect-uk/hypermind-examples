//! Error types for the SDK

use thiserror::Error;

/// Result type alias for SDK operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the SDK
#[derive(Error, Debug)]
pub enum Error {
    /// Query parsing or execution error
    #[error("Query error: {0}")]
    Query(String),

    /// Storage backend error
    #[error("Storage error: {0}")]
    Storage(String),

    /// RDF parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Invalid operation error
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Internal error (should not happen in normal use)
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<storage::StorageError> for Error {
    fn from(err: storage::StorageError) -> Self {
        Error::Storage(err.to_string())
    }
}
