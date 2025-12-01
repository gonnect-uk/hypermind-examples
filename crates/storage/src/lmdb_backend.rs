//! LMDB persistent storage backend (Planned for v0.2.0)
//!
//! This module is a placeholder for the LMDB backend implementation.
//! LMDB (Lightning Memory-Mapped Database) will provide:
//! - B+tree storage for read-heavy workloads
//! - Memory-mapped I/O for fast reads
//! - MVCC (Multi-Version Concurrency Control)
//! - Zero-copy reads
//!
//! For now, use RocksDB for persistent storage.

use crate::{StorageBackend, StorageError, StorageResult, StorageStats};
use std::path::Path;

/// LMDB storage backend (not yet implemented)
///
/// This is a placeholder for the LMDB backend. For persistent storage,
/// use `RocksDbBackend` instead.
///
/// # Panics
/// All methods will panic with "LMDB backend not yet implemented".
#[derive(Clone, Debug)]
pub struct LmdbBackend {
    _path: std::path::PathBuf,
}

impl LmdbBackend {
    /// Create a new LMDB backend (not yet implemented)
    ///
    /// # Panics
    /// Always panics - LMDB is planned for v0.2.0
    pub fn new<P: AsRef<Path>>(_path: P) -> StorageResult<Self> {
        Err(StorageError::Backend(
            "LMDB backend not yet implemented. Use RocksDbBackend for persistent storage. \
             LMDB is planned for v0.2.0.".to_string()
        ))
    }
}

impl StorageBackend for LmdbBackend {
    fn get(&self, _key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        unimplemented!("LMDB backend planned for v0.2.0")
    }

    fn put(&mut self, _key: &[u8], _value: &[u8]) -> StorageResult<()> {
        unimplemented!("LMDB backend planned for v0.2.0")
    }

    fn delete(&mut self, _key: &[u8]) -> StorageResult<()> {
        unimplemented!("LMDB backend planned for v0.2.0")
    }

    fn range_scan<'a>(
        &'a self,
        _start: &[u8],
        _end: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        unimplemented!("LMDB backend planned for v0.2.0")
    }

    fn prefix_scan<'a>(
        &'a self,
        _prefix: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        unimplemented!("LMDB backend planned for v0.2.0")
    }

    fn stats(&self) -> StorageStats {
        StorageStats::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lmdb_not_implemented() {
        let result = LmdbBackend::new("/tmp/test");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not yet implemented"));
    }
}
