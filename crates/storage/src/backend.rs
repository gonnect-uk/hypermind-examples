//! Storage backend trait abstraction
//!
//! Provides a unified interface that works across different storage implementations.

use std::fmt;

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Errors that can occur in storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Key not found in storage
    #[error("Key not found: {0}")]
    NotFound(String),

    /// I/O error (file system, network, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Database corruption
    #[error("Database corruption: {0}")]
    Corruption(String),

    /// Backend-specific error
    #[error("Backend error: {0}")]
    Backend(String),
}

/// Abstract storage backend trait
///
/// Provides low-level key-value operations that storage implementations must support.
/// Built on top of this, we provide higher-level quad store operations.
pub trait StorageBackend: Send + Sync {
    /// Get a value by key
    fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>>;

    /// Put a key-value pair
    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()>;

    /// Delete a key
    fn delete(&mut self, key: &[u8]) -> StorageResult<()>;

    /// Check if a key exists
    fn contains(&self, key: &[u8]) -> StorageResult<bool> {
        Ok(self.get(key)?.is_some())
    }

    /// Scan a range of keys
    ///
    /// Returns an iterator over (key, value) pairs where `start <= key < end`.
    fn range_scan<'a>(
        &'a self,
        start: &[u8],
        end: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>>;

    /// Scan with a key prefix
    ///
    /// Returns all (key, value) pairs where key starts with `prefix`.
    fn prefix_scan<'a>(
        &'a self,
        prefix: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>>;

    /// Flush any pending writes to durable storage
    fn flush(&mut self) -> StorageResult<()> {
        // Default: no-op for in-memory backends
        Ok(())
    }

    /// Compact the storage (for LSM-tree backends)
    fn compact(&mut self) -> StorageResult<()> {
        // Default: no-op
        Ok(())
    }

    /// Batch insert multiple key-value pairs (optimized for bulk operations)
    ///
    /// Default implementation calls `put()` sequentially, but backends can override
    /// for better performance (e.g., single lock acquisition, write batching).
    fn batch_put(&mut self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> StorageResult<()> {
        for (key, value) in pairs {
            self.put(&key, &value)?;
        }
        Ok(())
    }

    /// Get storage statistics
    fn stats(&self) -> StorageStats {
        StorageStats::default()
    }
}

/// Storage statistics
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    /// Total number of keys
    pub key_count: u64,

    /// Total size in bytes
    pub total_bytes: u64,

    /// Average key size in bytes
    pub avg_key_size: u64,

    /// Average value size in bytes
    pub avg_value_size: u64,

    /// Number of read operations
    pub reads: u64,

    /// Number of write operations
    pub writes: u64,

    /// Number of delete operations
    pub deletes: u64,
}

impl fmt::Display for StorageStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StorageStats {{ keys: {}, size: {} MB, reads: {}, writes: {}, deletes: {} }}",
            self.key_count,
            self.total_bytes / 1024 / 1024,
            self.reads,
            self.writes,
            self.deletes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_stats_display() {
        let stats = StorageStats {
            key_count: 1000,
            total_bytes: 1024 * 1024 * 10, // 10 MB
            avg_key_size: 100,
            avg_value_size: 500,
            reads: 5000,
            writes: 1000,
            deletes: 100,
        };

        let display = format!("{}", stats);
        assert!(display.contains("keys: 1000"));
        assert!(display.contains("size: 10 MB"));
    }
}
