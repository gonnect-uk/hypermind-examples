//! RocksDB persistent storage backend
//!
//! LSM-tree based storage with ACID transactions, compression, and durability.
//! Optimized for write-heavy workloads with fast sequential reads.

use crate::{StorageBackend, StorageError, StorageResult, StorageStats};
use parking_lot::RwLock;
use rocksdb::{Direction, IteratorMode, Options, WriteBatch, DB};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// RocksDB storage backend
///
/// Persistent storage using Facebook's RocksDB (LSM-tree).
/// Features:
/// - ACID transactions
/// - Compression (LZ4/Snappy)
/// - Crash recovery
/// - Fast sequential writes
/// - Range iteration
#[derive(Clone)]
pub struct RocksDbBackend {
    /// RocksDB database handle (thread-safe)
    db: Arc<DB>,

    /// Database path on disk
    path: PathBuf,

    /// Statistics
    stats: Arc<RwLock<StorageStats>>,
}

impl RocksDbBackend {
    /// Create a new RocksDB backend at the specified path
    ///
    /// # Arguments
    /// * `path` - Directory where RocksDB will store data
    ///
    /// # Errors
    /// Returns `StorageError::Io` if database cannot be opened
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Performance tuning
        opts.set_max_open_files(1000);
        opts.set_keep_log_file_num(10);
        opts.set_max_background_jobs(4);

        // Compression (use Snappy which is built-in to RocksDB)
        use rocksdb::DBCompressionType;
        opts.set_compression_type(DBCompressionType::Snappy);

        let db = DB::open(&opts, &path)
            .map_err(|e| StorageError::Backend(format!("Failed to open RocksDB: {}", e)))?;

        Ok(Self {
            db: Arc::new(db),
            path,
            stats: Arc::new(RwLock::new(StorageStats::default())),
        })
    }

    /// Create with custom options
    pub fn with_options<P: AsRef<Path>>(path: P, opts: Options) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();

        let db = DB::open(&opts, &path)
            .map_err(|e| StorageError::Backend(format!("Failed to open RocksDB: {}", e)))?;

        Ok(Self {
            db: Arc::new(db),
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
        // RocksDB property to estimate database size
        let size_str = self.db
            .property_value("rocksdb.total-sst-files-size")
            .map_err(|e| StorageError::Backend(format!("Failed to get size: {}", e)))?
            .unwrap_or_else(|| "0".to_string());

        size_str.parse::<u64>()
            .map_err(|e| StorageError::Backend(format!("Failed to parse size: {}", e)))
    }

    /// Get number of keys (approximate)
    pub fn key_count(&self) -> StorageResult<u64> {
        let count_str = self.db
            .property_value("rocksdb.estimate-num-keys")
            .map_err(|e| StorageError::Backend(format!("Failed to get key count: {}", e)))?
            .unwrap_or_else(|| "0".to_string());

        count_str.parse::<u64>()
            .map_err(|e| StorageError::Backend(format!("Failed to parse count: {}", e)))
    }
}

impl StorageBackend for RocksDbBackend {
    fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        let result = self.db
            .get(key)
            .map_err(|e| StorageError::Backend(format!("RocksDB get error: {}", e)))?;

        // Update stats
        self.stats.write().reads += 1;

        Ok(result)
    }

    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        self.db
            .put(key, value)
            .map_err(|e| StorageError::Backend(format!("RocksDB put error: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write();
        stats.writes += 1;
        stats.key_count = self.key_count().unwrap_or(stats.key_count);

        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> StorageResult<()> {
        self.db
            .delete(key)
            .map_err(|e| StorageError::Backend(format!("RocksDB delete error: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write();
        stats.deletes += 1;
        stats.key_count = self.key_count().unwrap_or(stats.key_count);

        Ok(())
    }

    fn contains(&self, key: &[u8]) -> StorageResult<bool> {
        // RocksDB doesn't have a direct contains(), use get()
        Ok(self.get(key)?.is_some())
    }

    fn range_scan<'a>(
        &'a self,
        start: &[u8],
        end: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        // RocksDB iterator from start key
        let iter = self.db.iterator(IteratorMode::From(start, Direction::Forward));

        // Filter keys within [start, end) range
        let end_vec = end.to_vec();
        let filtered_iter = iter
            .map(|res| res.map(|(k, v)| (k.to_vec(), v.to_vec())))
            .take_while(move |res| {
                if let Ok((ref key, _)) = res {
                    key.as_slice() < end_vec.as_slice()
                } else {
                    true // Let errors pass through
                }
            })
            .filter_map(|res| {
                res.map_err(|e| {
                    eprintln!("RocksDB iterator error: {}", e);
                })
                .ok()
            });

        Ok(Box::new(filtered_iter))
    }

    fn prefix_scan<'a>(
        &'a self,
        prefix: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        // RocksDB iterator from prefix
        let iter = self.db.iterator(IteratorMode::From(prefix, Direction::Forward));

        // Filter keys with matching prefix
        let prefix_vec = prefix.to_vec();
        let filtered_iter = iter
            .map(|res| res.map(|(k, v)| (k.to_vec(), v.to_vec())))
            .take_while(move |res| {
                if let Ok((ref key, _)) = res {
                    key.starts_with(&prefix_vec)
                } else {
                    true // Let errors pass through
                }
            })
            .filter_map(|res| {
                res.map_err(|e| {
                    eprintln!("RocksDB iterator error: {}", e);
                })
                .ok()
            });

        Ok(Box::new(filtered_iter))
    }

    fn batch_put(&mut self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> StorageResult<()> {
        // Use RocksDB WriteBatch for atomic batch writes
        let mut batch = WriteBatch::default();

        for (key, value) in &pairs {
            batch.put(key, value);
        }

        self.db
            .write(batch)
            .map_err(|e| StorageError::Backend(format!("RocksDB batch write error: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write();
        stats.writes += pairs.len() as u64;
        stats.key_count = self.key_count().unwrap_or(stats.key_count);

        Ok(())
    }

    fn flush(&mut self) -> StorageResult<()> {
        self.db
            .flush()
            .map_err(|e| StorageError::Backend(format!("RocksDB flush error: {}", e)))?;
        Ok(())
    }

    fn compact(&mut self) -> StorageResult<()> {
        // Compact entire database (full range)
        self.db.compact_range(None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }

    fn stats(&self) -> StorageStats {
        let mut stats = self.stats.read().clone();

        // Update with RocksDB internal stats
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
impl Drop for RocksDbBackend {
    fn drop(&mut self) {
        // RocksDB handles cleanup automatically when Arc<DB> is dropped
        // No explicit cleanup needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_db() -> (RocksDbBackend, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = RocksDbBackend::new(temp_dir.path()).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_rocksdb_basic_operations() {
        let (mut db, _temp) = create_temp_db();

        // Put
        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();

        // Get
        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get(b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(db.get(b"key3").unwrap(), None);

        // Contains
        assert!(db.contains(b"key1").unwrap());
        assert!(!db.contains(b"key3").unwrap());

        // Delete
        db.delete(b"key1").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), None);

        // Stats
        let stats = db.stats();
        assert_eq!(stats.writes, 2);
        assert_eq!(stats.deletes, 1);
    }

    #[test]
    fn test_rocksdb_range_scan() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"a", b"1").unwrap();
        db.put(b"b", b"2").unwrap();
        db.put(b"c", b"3").unwrap();
        db.put(b"d", b"4").unwrap();

        let results: Vec<_> = db.range_scan(b"b", b"d").unwrap().collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, b"b");
        assert_eq!(results[1].0, b"c");
    }

    #[test]
    fn test_rocksdb_prefix_scan() {
        let (mut db, _temp) = create_temp_db();

        db.put(b"prefix:1", b"value1").unwrap();
        db.put(b"prefix:2", b"value2").unwrap();
        db.put(b"other:1", b"value3").unwrap();

        let results: Vec<_> = db.prefix_scan(b"prefix:").unwrap().collect();

        assert_eq!(results.len(), 2);
        assert!(results[0].0.starts_with(b"prefix:"));
        assert!(results[1].0.starts_with(b"prefix:"));
    }

    #[test]
    fn test_rocksdb_batch_put() {
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
    fn test_rocksdb_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Create database, write data, drop it
        {
            let mut db = RocksDbBackend::new(&path).unwrap();
            db.put(b"persistent_key", b"persistent_value").unwrap();
            db.flush().unwrap();
        }

        // Reopen database and verify data persisted
        {
            let db = RocksDbBackend::new(&path).unwrap();
            assert_eq!(
                db.get(b"persistent_key").unwrap(),
                Some(b"persistent_value".to_vec())
            );
        }
    }

    #[test]
    fn test_rocksdb_compact() {
        let (mut db, _temp) = create_temp_db();

        // Write and delete data to create tombstones
        for i in 0..100 {
            db.put(format!("key{}", i).as_bytes(), b"value").unwrap();
        }
        for i in 0..50 {
            db.delete(format!("key{}", i).as_bytes()).unwrap();
        }

        // Compact should reclaim space
        db.compact().unwrap();

        // Verify remaining keys still accessible
        assert_eq!(db.get(b"key50").unwrap(), Some(b"value".to_vec()));
        assert_eq!(db.get(b"key0").unwrap(), None);
    }
}
