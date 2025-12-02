//! Error types for the cluster crate.
//!
//! Provides a unified error type for all cluster operations including:
//! - Partitioning errors
//! - Network/gRPC errors
//! - Configuration errors
//! - Node management errors

use std::fmt;
use thiserror::Error;

/// Result type alias using ClusterError
pub type ClusterResult<T> = Result<T, ClusterError>;

/// Unified error type for cluster operations
#[derive(Error, Debug)]
pub enum ClusterError {
    /// Error during partition assignment or lookup
    #[error("Partition error: {0}")]
    Partition(String),

    /// Node not found in cluster
    #[error("Node not found: {0}")]
    NodeNotFound(u64),

    /// Partition not found
    #[error("Partition not found: {0}")]
    PartitionNotFound(u16),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network/communication error
    #[error("Network error: {0}")]
    Network(String),

    /// Timeout waiting for operation
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Node is not in a valid state for the operation
    #[error("Invalid node state: expected {expected}, got {actual}")]
    InvalidState {
        /// Expected state
        expected: String,
        /// Actual state
        actual: String,
    },

    /// Rebalancing operation failed
    #[error("Rebalancing error: {0}")]
    Rebalancing(String),

    /// Storage backend error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Internal error (should not happen)
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ClusterError {
    /// Create a partition error
    pub fn partition(msg: impl Into<String>) -> Self {
        ClusterError::Partition(msg.into())
    }

    /// Create a node not found error
    pub fn node_not_found(id: u64) -> Self {
        ClusterError::NodeNotFound(id)
    }

    /// Create a partition not found error
    pub fn partition_not_found(id: u16) -> Self {
        ClusterError::PartitionNotFound(id)
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        ClusterError::Config(msg.into())
    }

    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        ClusterError::Network(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        ClusterError::Timeout(msg.into())
    }

    /// Create an invalid state error
    pub fn invalid_state(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        ClusterError::InvalidState {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a rebalancing error
    pub fn rebalancing(msg: impl Into<String>) -> Self {
        ClusterError::Rebalancing(msg.into())
    }

    /// Create a storage error
    pub fn storage(msg: impl Into<String>) -> Self {
        ClusterError::Storage(msg.into())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        ClusterError::Serialization(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        ClusterError::Internal(msg.into())
    }

    /// Check if this is a retriable error
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            ClusterError::Network(_) | ClusterError::Timeout(_)
        )
    }

    /// Check if this is a fatal error
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            ClusterError::Config(_) | ClusterError::Internal(_)
        )
    }
}

/// Error code for wire protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ErrorCode {
    /// Success (no error)
    Ok = 0,
    /// Unknown error
    Unknown = 1,
    /// Partition error
    Partition = 100,
    /// Node not found
    NodeNotFound = 101,
    /// Partition not found
    PartitionNotFound = 102,
    /// Configuration error
    Config = 200,
    /// Network error
    Network = 300,
    /// Timeout
    Timeout = 301,
    /// Invalid state
    InvalidState = 400,
    /// Rebalancing error
    Rebalancing = 401,
    /// Storage error
    Storage = 500,
    /// Serialization error
    Serialization = 501,
    /// Internal error
    Internal = 999,
}

impl From<&ClusterError> for ErrorCode {
    fn from(err: &ClusterError) -> Self {
        match err {
            ClusterError::Partition(_) => ErrorCode::Partition,
            ClusterError::NodeNotFound(_) => ErrorCode::NodeNotFound,
            ClusterError::PartitionNotFound(_) => ErrorCode::PartitionNotFound,
            ClusterError::Config(_) => ErrorCode::Config,
            ClusterError::Network(_) => ErrorCode::Network,
            ClusterError::Timeout(_) => ErrorCode::Timeout,
            ClusterError::InvalidState { .. } => ErrorCode::InvalidState,
            ClusterError::Rebalancing(_) => ErrorCode::Rebalancing,
            ClusterError::Storage(_) => ErrorCode::Storage,
            ClusterError::Serialization(_) => ErrorCode::Serialization,
            ClusterError::Internal(_) => ErrorCode::Internal,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ClusterError::partition("invalid partition");
        assert!(matches!(err, ClusterError::Partition(_)));
        assert_eq!(err.to_string(), "Partition error: invalid partition");
    }

    #[test]
    fn test_error_code_conversion() {
        let err = ClusterError::network("connection refused");
        assert_eq!(ErrorCode::from(&err), ErrorCode::Network);
        assert_eq!(ErrorCode::Network as u16, 300);
    }

    #[test]
    fn test_retriable_errors() {
        assert!(ClusterError::network("timeout").is_retriable());
        assert!(ClusterError::timeout("deadline exceeded").is_retriable());
        assert!(!ClusterError::config("invalid").is_retriable());
    }

    #[test]
    fn test_fatal_errors() {
        assert!(ClusterError::config("missing").is_fatal());
        assert!(ClusterError::internal("bug").is_fatal());
        assert!(!ClusterError::network("timeout").is_fatal());
    }
}
