//! LMDB persistent storage backend
//!
//! Memory-mapped B+tree storage with MVCC and zero-copy reads.
//! Optimized for read-heavy workloads with consistent read performance.

use crate::{StorageBackend, StorageError, StorageResult, StorageStats};
use heed::types::*;
use heed::{Database, Env, EnvOpenOptions};
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// LMDB storage backend
///
/// Persistent storage using LMDB (Lightning Memory-Mapped Database).
/// Features:
/// - Memory-mapped I/O for zero-copy reads
/// - MVCC (Multi-Version Concurrency Control)
/// - B+tree ordered storage
/// - Excellent read performance
/// - ACID transactions
#[derive(Clone)]
pub struct LmdbBackend {
    /// LMDB environment handle (thread-safe)
    env: Arc<Env>,

    /// Main database (key-value pairs)
    db: Database<Bytes, Bytes>,

    /// Database path on disk
    path: PathBuf,

    /// Statistics
    stats: Arc<RwLock<StorageStats>>,
}

impl LmdbBackend {
    /// Create a new LMDB backend at the specified path
    ///
    /// # Arguments
    /// * `path` - Directory where LMDB will store data
    ///
    /// # Errors
    /// Returns `StorageError::Backend` if database cannot be opened
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        Self::with_map_size(path, 10 * 1024 * 1024 * 1024) // 10 GB default
    }

    /// Create LMDB backend with custom map size
    ///
    /// # Arguments
    /// * `path` - Directory where LMDB will store data
    /// * `map_size` - Maximum size of the memory map in bytes
    ///
    /// # Errors
    /// Returns `StorageError::Backend` if database cannot be opened
    pub fn with_map_size<P: AsRef<Path>>(path: P, map_size: usize) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&path).map_err(|e| {
            StorageError::Backend(format!("Failed to create LMDB directory: {}", e))
        })?;

        // Open LMDB environment
        // SAFETY: LMDB requires unsafe to open environment. We ensure:
        // 1. Path is valid and accessible
        // 2. Only one environment per process for this path
        // 3. All operations through safe API after this point
        #[allow(unsafe_code)]
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(map_size)
                .max_dbs(1)
                .open(&path)
                .map_err(|e| StorageError::Backend(format!("Failed to open LMDB: {}", e)))?
        };

        // Open/create main database
        let mut wtxn = env.write_txn().map_err(|e| {
            StorageError::Backend(format!("Failed to create write transaction: {}", e))
        })?;

        let db: Database<Bytes, Bytes> = env
            .create_database(&mut wtxn, Some("main"))
            .map_err(|e| StorageError::Backend(format!("Failed to create database: {}", e)))?;

        wtxn.commit().map_err(|e| {
            StorageError::Backend(format!("Failed to commit database creation: {}", e))
        })?;

        Ok(Self {
            env: Arc::new(env),
            db,
            path,
            stats: Arc::new(RwLock::new(StorageStats::default())),
        })
    }

    /// Get database path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get approximate size on disk in bytes
    pub fn disk_size(&self) -> StorageResult<u64> {
        // Get the data.mdb file size
        let data_path = self.path.join("data.mdb");
        match std::fs::metadata(&data_path) {
            Ok(metadata) => Ok(metadata.len()),
            Err(_) => Ok(0), // File doesn't exist yet
        }
    }

    /// Get number of keys (exact count via iteration)
    pub fn key_count(&self) -> StorageResult<u64> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StorageError::Backend(format!("Failed to open read transaction: {}", e)))?;

        let count = self
            .db
            .len(&rtxn)
            .map_err(|e| StorageError::Backend(format!("Failed to get key count: {}", e)))?;

        Ok(count)
    }

    /// Sync data to disk (force flush)
    pub fn sync(&self) -> StorageResult<()> {
        self.env
            .force_sync()
            .map_err(|e| StorageError::Backend(format!("LMDB sync error: {}", e)))?;
        Ok(())
    }

    /// Get LMDB statistics
    pub fn lmdb_stats(&self) -> StorageResult<String> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StorageError::Backend(format!("Failed to open read transaction: {}", e)))?;

        let stat = self
            .db
            .stat(&rtxn)
            .map_err(|e| StorageError::Backend(format!("Failed to get LMDB stats: {}", e)))?;

        Ok(format!(
            "LMDB Stats - Page size: {}, Depth: {}, Branch pages: {}, Leaf pages: {}, Overflow pages: {}, Entries: {}",
            stat.page_size,
            stat.depth,
            stat.branch_pages,
            stat.leaf_pages,
            stat.overflow_pages,
            stat.entries
        ))
    }
}

impl StorageBackend for LmdbBackend {
    fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StorageError::Backend(format!("Failed to open read transaction: {}", e)))?;

        let result = self
            .db
            .get(&rtxn, key)
            .map_err(|e| StorageError::Backend(format!("LMDB get error: {}", e)))?
            .map(|v| v.to_vec());

        // Update stats
        self.stats.write().reads += 1;

        Ok(result)
    }

    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StorageError::Backend(format!("Failed to open write transaction: {}", e)))?;

        self.db
            .put(&mut wtxn, key, value)
            .map_err(|e| StorageError::Backend(format!("LMDB put error: {}", e)))?;

        wtxn.commit()
            .map_err(|e| StorageError::Backend(format!("Failed to commit write transaction: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write();
        stats.writes += 1;
        stats.key_count = self.key_count().unwrap_or(stats.key_count);

        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> StorageResult<()> {
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StorageError::Backend(format!("Failed to open write transaction: {}", e)))?;

        let existed = self
            .db
            .delete(&mut wtxn, key)
            .map_err(|e| StorageError::Backend(format!("LMDB delete error: {}", e)))?;

        wtxn.commit()
            .map_err(|e| StorageError::Backend(format!("Failed to commit delete transaction: {}", e)))?;

        // Update stats only if key existed
        if existed {
            let mut stats = self.stats.write();
            stats.deletes += 1;
            stats.key_count = self.key_count().unwrap_or(stats.key_count);
        }

        Ok(())
    }

    fn contains(&self, key: &[u8]) -> StorageResult<bool> {
        Ok(self.get(key)?.is_some())
    }

    fn range_scan<'a>(
        &'a self,
        start: &[u8],
        end: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        // Create read-only transaction
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StorageError::Backend(format!("Failed to open read transaction: {}", e)))?;

        // LMDB iterators require owned transactions, so we collect results
        let end_vec = end.to_vec();
        let mut results = Vec::new();

        // Iterate over all entries and filter by range
        let iter = self
            .db
            .iter(&rtxn)
            .map_err(|e| StorageError::Backend(format!("LMDB range error: {}", e)))?;

        // Collect entries within [start, end)
        for result in iter {
            let (key, value) = result
                .map_err(|e| StorageError::Backend(format!("LMDB iterator error: {}", e)))?;

            // Skip keys before start
            if key < start {
                continue;
            }

            // Stop at end
            if key >= end_vec.as_slice() {
                break;
            }

            results.push((key.to_vec(), value.to_vec()));
        }

        Ok(Box::new(results.into_iter()))
    }

    fn prefix_scan<'a>(
        &'a self,
        prefix: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        // Create read-only transaction
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| StorageError::Backend(format!("Failed to open read transaction: {}", e)))?;

        let prefix_vec = prefix.to_vec();
        let mut results = Vec::new();

        // Iterate over all entries and filter by prefix
        let iter = self
            .db
            .iter(&rtxn)
            .map_err(|e| StorageError::Backend(format!("LMDB prefix error: {}", e)))?;

        // Collect all matching entries
        for result in iter {
            let (key, value) = result
                .map_err(|e| StorageError::Backend(format!("LMDB iterator error: {}", e)))?;

            if key.starts_with(&prefix_vec) {
                results.push((key.to_vec(), value.to_vec()));
            } else if !results.is_empty() {
                // If we've already found prefix matches and now found a non-match,
                // we can stop (assuming keys are sorted)
                break;
            }
        }

        Ok(Box::new(results.into_iter()))
    }

    fn batch_put(&mut self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> StorageResult<()> {
        if pairs.is_empty() {
            return Ok(());
        }

        // Single write transaction for all pairs
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| StorageError::Backend(format!("Failed to open write transaction: {}", e)))?;

        for (key, value) in &pairs {
            self.db
                .put(&mut wtxn, key, value)
                .map_err(|e| StorageError::Backend(format!("LMDB batch put error: {}", e)))?;
        }

        wtxn.commit()
            .map_err(|e| StorageError::Backend(format!("Failed to commit batch transaction: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write();
        stats.writes += pairs.len() as u64;
        stats.key_count = self.key_count().unwrap_or(stats.key_count);

        Ok(())
    }

    fn flush(&mut self) -> StorageResult<()> {
        // Force sync to disk
        self.sync()
    }

    fn stats(&self) -> StorageStats {
        let mut stats = self.stats.read().clone();

        // Update with LMDB internal stats
        if let Ok(count) = self.key_count() {
            stats.key_count = count;
        }

        if let Ok(size) = self.disk_size() {
            stats.total_bytes = size;
        }

        stats
    }
}

// Implement Drop to clean up resources
impl Drop for LmdbBackend {
    fn drop(&mut self) {
        // LMDB handles cleanup automatically when Arc<Env> is dropped
        // No explicit cleanup needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_db() -> (LmdbBackend, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = LmdbBackend::new(temp_dir.path()).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_lmdb_basic_put_get() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key1", b"value1").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
    }

    #[test]
    fn test_lmdb_get_nonexistent() {
        let (db, _temp) = create_temp_db();
        assert_eq!(db.get(b"nonexistent").unwrap(), None);
    }

    #[test]
    fn test_lmdb_overwrite() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key", b"value1").unwrap();
        db.put(b"key", b"value2").unwrap();
        assert_eq!(db.get(b"key").unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_lmdb_delete() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key", b"value").unwrap();
        assert!(db.contains(b"key").unwrap());

        db.delete(b"key").unwrap();
        assert!(!db.contains(b"key").unwrap());
        assert_eq!(db.get(b"key").unwrap(), None);
    }

    #[test]
    fn test_lmdb_delete_nonexistent() {
        let (mut db, _temp) = create_temp_db();

        // Should not error
        db.delete(b"nonexistent").unwrap();
    }

    #[test]
    fn test_lmdb_contains() {
        let (mut db, _temp) = create_temp_db();

        assert!(!db.contains(b"key").unwrap());

        db.put(b"key", b"value").unwrap();
        assert!(db.contains(b"key").unwrap());

        db.delete(b"key").unwrap();
        assert!(!db.contains(b"key").unwrap());
    }

    #[test]
    fn test_lmdb_multiple_keys() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();
        db.put(b"key3", b"value3").unwrap();

        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get(b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(db.get(b"key3").unwrap(), Some(b"value3".to_vec()));
    }

    #[test]
    fn test_lmdb_range_scan() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"a", b"1").unwrap();
        db.put(b"b", b"2").unwrap();
        db.put(b"c", b"3").unwrap();
        db.put(b"d", b"4").unwrap();
        db.put(b"e", b"5").unwrap();

        let results: Vec<_> = db.range_scan(b"b", b"e").unwrap().collect();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, b"b");
        assert_eq!(results[1].0, b"c");
        assert_eq!(results[2].0, b"d");
    }

    #[test]
    fn test_lmdb_range_scan_empty() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"a", b"1").unwrap();
        db.put(b"b", b"2").unwrap();

        let results: Vec<_> = db.range_scan(b"c", b"d").unwrap().collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_lmdb_range_scan_single() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"a", b"1").unwrap();
        db.put(b"b", b"2").unwrap();
        db.put(b"c", b"3").unwrap();

        let results: Vec<_> = db.range_scan(b"b", b"c").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, b"b");
    }

    #[test]
    fn test_lmdb_prefix_scan() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"prefix:1", b"value1").unwrap();
        db.put(b"prefix:2", b"value2").unwrap();
        db.put(b"prefix:3", b"value3").unwrap();
        db.put(b"other:1", b"value4").unwrap();

        let results: Vec<_> = db.prefix_scan(b"prefix:").unwrap().collect();

        assert_eq!(results.len(), 3);
        assert!(results[0].0.starts_with(b"prefix:"));
        assert!(results[1].0.starts_with(b"prefix:"));
        assert!(results[2].0.starts_with(b"prefix:"));
    }

    #[test]
    fn test_lmdb_prefix_scan_no_match() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();

        let results: Vec<_> = db.prefix_scan(b"prefix:").unwrap().collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_lmdb_batch_put() {
        let (mut db, _temp) = create_temp_db();

        let pairs = vec![
            (b"key1".to_vec(), b"value1".to_vec()),
            (b"key2".to_vec(), b"value2".to_vec()),
            (b"key3".to_vec(), b"value3".to_vec()),
        ];

        db.batch_put(pairs).unwrap();

        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get(b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(db.get(b"key3").unwrap(), Some(b"value3".to_vec()));
    }

    #[test]
    fn test_lmdb_batch_put_empty() {
        let (mut db, _temp) = create_temp_db();

        db.batch_put(vec![]).unwrap();
        // Should not error
    }

    #[test]
    fn test_lmdb_batch_put_large() {
        let (mut db, _temp) = create_temp_db();

        let pairs: Vec<_> = (0..1000)
            .map(|i| (format!("key{}", i).into_bytes(), format!("value{}", i).into_bytes()))
            .collect();

        db.batch_put(pairs).unwrap();

        // Verify some entries
        assert_eq!(db.get(b"key0").unwrap(), Some(b"value0".to_vec()));
        assert_eq!(db.get(b"key500").unwrap(), Some(b"value500".to_vec()));
        assert_eq!(db.get(b"key999").unwrap(), Some(b"value999".to_vec()));
    }

    #[test]
    fn test_lmdb_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Create database, write data, drop it
        {
            let mut db = LmdbBackend::new(&path).unwrap();
            db.put(b"persistent_key", b"persistent_value").unwrap();
            db.flush().unwrap();
        }

        // Reopen database and verify data persisted
        {
            let db = LmdbBackend::new(&path).unwrap();
            assert_eq!(
                db.get(b"persistent_key").unwrap(),
                Some(b"persistent_value".to_vec())
            );
        }
    }

    #[test]
    fn test_lmdb_persistence_multiple_keys() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Write multiple keys
        {
            let mut db = LmdbBackend::new(&path).unwrap();
            for i in 0..100 {
                db.put(format!("key{}", i).as_bytes(), format!("value{}", i).as_bytes())
                    .unwrap();
            }
            db.flush().unwrap();
        }

        // Reopen and verify
        {
            let db = LmdbBackend::new(&path).unwrap();
            for i in 0..100 {
                let key = format!("key{}", i);
                let expected_value = format!("value{}", i);
                assert_eq!(
                    db.get(key.as_bytes()).unwrap(),
                    Some(expected_value.into_bytes())
                );
            }
        }
    }

    #[test]
    fn test_lmdb_flush() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key", b"value").unwrap();
        db.flush().unwrap();

        assert_eq!(db.get(b"key").unwrap(), Some(b"value".to_vec()));
    }

    #[test]
    fn test_lmdb_stats() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();
        db.delete(b"key1").unwrap();
        let _ = db.get(b"key2").unwrap();

        let stats = db.stats();
        assert_eq!(stats.writes, 2);
        assert_eq!(stats.deletes, 1);
        assert_eq!(stats.reads, 1);
    }

    #[test]
    fn test_lmdb_key_count() {
        let (mut db, _temp) = create_temp_db();

        assert_eq!(db.key_count().unwrap(), 0);

        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();
        assert_eq!(db.key_count().unwrap(), 2);

        db.delete(b"key1").unwrap();
        assert_eq!(db.key_count().unwrap(), 1);
    }

    #[test]
    fn test_lmdb_disk_size() {
        let (mut db, _temp) = create_temp_db();

        // Write some data
        for i in 0..100 {
            db.put(format!("key{}", i).as_bytes(), b"value").unwrap();
        }
        db.flush().unwrap();

        let size = db.disk_size().unwrap();
        assert!(size > 0, "Disk size should be greater than 0");
    }

    #[test]
    fn test_lmdb_path() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        let db = LmdbBackend::new(path).unwrap();

        assert_eq!(db.path(), path);
    }

    #[test]
    fn test_lmdb_custom_map_size() {
        let temp_dir = TempDir::new().unwrap();
        let map_size = 1024 * 1024 * 100; // 100 MB

        let db = LmdbBackend::with_map_size(temp_dir.path(), map_size).unwrap();
        assert!(db.path().exists());
    }

    #[test]
    fn test_lmdb_binary_keys() {
        let (mut db, _temp) = create_temp_db();

        let binary_key = vec![0u8, 1, 2, 3, 255, 254, 253];
        let binary_value = vec![10u8, 20, 30, 40, 50];

        db.put(&binary_key, &binary_value).unwrap();
        assert_eq!(db.get(&binary_key).unwrap(), Some(binary_value));
    }

    #[test]
    fn test_lmdb_empty_key() {
        let (mut db, _temp) = create_temp_db();

        // LMDB does not support empty keys (database constraint)
        let result = db.put(b"", b"empty_key_value");
        assert!(result.is_err(), "LMDB should reject empty keys");
    }

    #[test]
    fn test_lmdb_empty_value() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key", b"").unwrap();
        assert_eq!(db.get(b"key").unwrap(), Some(b"".to_vec()));
    }

    #[test]
    fn test_lmdb_large_value() {
        let (mut db, _temp) = create_temp_db();

        let large_value = vec![42u8; 10_000]; // 10 KB
        db.put(b"large_key", &large_value).unwrap();
        assert_eq!(db.get(b"large_key").unwrap(), Some(large_value));
    }

    #[test]
    fn test_lmdb_ordered_iteration() {
        let (mut db, _temp) = create_temp_db();

        // Insert in random order
        db.put(b"c", b"3").unwrap();
        db.put(b"a", b"1").unwrap();
        db.put(b"b", b"2").unwrap();

        // Range scan should return in sorted order
        let results: Vec<_> = db.range_scan(b"a", b"d").unwrap().collect();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, b"a");
        assert_eq!(results[1].0, b"b");
        assert_eq!(results[2].0, b"c");
    }

    #[test]
    fn test_lmdb_lmdb_stats() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key", b"value").unwrap();
        db.flush().unwrap();

        let stats_str = db.lmdb_stats().unwrap();
        assert!(stats_str.contains("LMDB Stats"));
        assert!(stats_str.contains("Page size"));
    }

    #[test]
    fn test_lmdb_clone() {
        let (mut db1, _temp) = create_temp_db();

        db1.put(b"key", b"value").unwrap();

        let db2 = db1.clone();
        assert_eq!(db2.get(b"key").unwrap(), Some(b"value".to_vec()));
    }

    #[test]
    fn test_lmdb_sync() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"key", b"value").unwrap();
        db.sync().unwrap();

        assert_eq!(db.get(b"key").unwrap(), Some(b"value".to_vec()));
    }
}
