# Storage Backend Implementation - Quick Start Guide

**For Developers**: Copy-paste-ready code snippets and patterns

---

## File Creation Checklist

Create these files in order:

```bash
# 1. New backend implementations
touch crates/storage/src/rocksdb_backend.rs   # RocksDB implementation (~200 LOC)
touch crates/storage/src/lmdb_backend.rs      # LMDB implementation (~250 LOC)

# 2. Test infrastructure
mkdir -p crates/storage/tests/common
touch crates/storage/tests/common/mod.rs      # Shared utilities
touch crates/storage/tests/common/fixtures.rs # Test data generators
touch crates/storage/tests/rocksdb_tests.rs   # RocksDB tests (85 tests)
touch crates/storage/tests/lmdb_tests.rs      # LMDB tests (85 tests)
```

---

## Step 1: Update lib.rs (Feature Gate Code)

**File**: `crates/storage/src/lib.rs`

Add this after existing modules:

```rust
// Feature-gated backend implementations
#[cfg(feature = "rocksdb-backend")]
mod rocksdb_backend;
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb_backend::RocksDbBackend;

#[cfg(feature = "lmdb-backend")]
mod lmdb_backend;
#[cfg(feature = "lmdb-backend")]
pub use lmdb_backend::LmdbBackend;
```

This ensures:
- Code only compiles when feature is enabled
- No unused dependency warnings
- Clean build with `cargo build --features rocksdb-backend`

---

## Step 2: RocksDB Backend Template

**File**: `crates/storage/src/rocksdb_backend.rs`

```rust
//! RocksDB persistent storage backend
//!
//! Uses LSM-tree structure for fast writes and range scans.
//! Production-grade ACID compliance with compression support.

use crate::{StorageBackend, StorageError, StorageResult, StorageStats};
use rocksdb::{DB, Options, IteratorMode, Direction};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// RocksDB storage backend
///
/// Features:
/// - Persistent ACID storage
/// - LSM-tree based (fast writes)
/// - Compression (snappy)
/// - Batch operations
/// - Statistics tracking
pub struct RocksDbBackend {
    /// RocksDB instance (thread-safe)
    db: Arc<DB>,

    /// Database path
    path: PathBuf,

    /// Operation statistics
    stats: Arc<parking_lot::RwLock<StorageStats>>,
}

impl RocksDbBackend {
    /// Create a new RocksDB backend at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Create directory if needed
        std::fs::create_dir_all(&path)?;

        // Configure RocksDB options
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression(rocksdb::DBCompressionType::Snappy);

        // Open database
        let db = DB::open(&opts, &path)
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(Self {
            db: Arc::new(db),
            path,
            stats: Arc::new(parking_lot::RwLock::new(StorageStats::default())),
        })
    }

    /// Open an existing RocksDB backend
    pub fn open<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();

        let db = DB::open_for_read_only(&Options::default(), &path, false)
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(Self {
            db: Arc::new(db),
            path,
            stats: Arc::new(parking_lot::RwLock::new(StorageStats::default())),
        })
    }
}

impl StorageBackend for RocksDbBackend {
    fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        let result = self.db
            .get(key)
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        self.stats.write().reads += 1;
        Ok(result)
    }

    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        self.db
            .put(key, value)
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        self.stats.write().writes += 1;
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> StorageResult<()> {
        self.db
            .delete(key)
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        self.stats.write().deletes += 1;
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
        let iter = self.db.iterator(IteratorMode::From(start, Direction::Forward));

        let results: Vec<_> = iter
            .take_while(|(k, _)| k.as_slice() < end)
            .map(|(k, v)| (k.to_vec(), v.to_vec()))
            .collect();

        Ok(Box::new(results.into_iter()))
    }

    fn prefix_scan<'a>(
        &'a self,
        prefix: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        let iter = self.db.iterator(IteratorMode::From(prefix, Direction::Forward));

        let results: Vec<_> = iter
            .take_while(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.to_vec(), v.to_vec()))
            .collect();

        Ok(Box::new(results.into_iter()))
    }

    fn flush(&mut self) -> StorageResult<()> {
        self.db
            .flush()
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(())
    }

    fn compact(&mut self) -> StorageResult<()> {
        self.db.compact_range(None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }

    fn batch_put(&mut self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> StorageResult<()> {
        use rocksdb::WriteBatch;

        let mut batch = WriteBatch::default();

        for (key, value) in pairs.iter() {
            batch.put(key, value);
        }

        self.db
            .write(batch)
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        self.stats.write().writes += pairs.len() as u64;
        Ok(())
    }

    fn stats(&self) -> StorageStats {
        self.stats.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (RocksDbBackend, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let backend = RocksDbBackend::new(temp_dir.path()).unwrap();
        (backend, temp_dir)
    }

    #[test]
    fn test_rocksdb_basic_operations() {
        let (mut backend, _temp) = create_test_db();

        backend.put(b"key1", b"value1").unwrap();
        assert_eq!(backend.get(b"key1").unwrap(), Some(b"value1".to_vec()));

        backend.delete(b"key1").unwrap();
        assert_eq!(backend.get(b"key1").unwrap(), None);
    }
}
```

---

## Step 3: LMDB Backend Template

**File**: `crates/storage/src/lmdb_backend.rs`

```rust
//! LMDB persistent storage backend via heed
//!
//! Memory-mapped B+-tree optimized for read performance.
//! Excellent for read-heavy workloads with transactional support.

use crate::{StorageBackend, StorageError, StorageResult, StorageStats};
use heed::{Database, Env, RwTxn, RoTxn, types::Bytes};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// LMDB storage backend via heed
///
/// Features:
/// - Memory-mapped B+-tree
/// - ACID transactions
/// - Zero-copy reads
/// - Read optimized
pub struct LmdbBackend {
    /// LMDB environment (memory-mapped)
    env: Env,

    /// Key-value database
    db: Database<Bytes, Bytes>,

    /// Database path
    path: PathBuf,

    /// Statistics
    stats: Arc<parking_lot::RwLock<StorageStats>>,
}

impl LmdbBackend {
    /// Create a new LMDB backend at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Create directory
        std::fs::create_dir_all(&path)?;

        // Open environment with reasonable defaults
        let env = unsafe {
            Env::builder()
                .max_dbs(1)
                .map_size(1024 * 1024 * 1024)  // 1GB initial map
                .open(&path)
                .map_err(|e| StorageError::Backend(format!("LMDB open failed: {}", e)))?
        };

        // Create database
        let db = env
            .create_database::<_, Bytes, Bytes>(None)
            .map_err(|e| StorageError::Backend(format!("Database creation failed: {}", e)))?;

        Ok(Self {
            env,
            db,
            path,
            stats: Arc::new(parking_lot::RwLock::new(StorageStats::default())),
        })
    }
}

impl StorageBackend for LmdbBackend {
    fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        let rtxn = self.env
            .read_txn()
            .map_err(|e| StorageError::Backend(format!("Transaction failed: {}", e)))?;

        let result = self.db
            .get(&rtxn, key)
            .map_err(|e| StorageError::Backend(format!("Get failed: {}", e)))?
            .map(|v: &[u8]| v.to_vec());

        self.stats.write().reads += 1;
        Ok(result)
    }

    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        let mut wtxn = self.env
            .write_txn()
            .map_err(|e| StorageError::Backend(format!("Write transaction failed: {}", e)))?;

        self.db
            .put(&mut wtxn, key, value)
            .map_err(|e| StorageError::Backend(format!("Put failed: {}", e)))?;

        wtxn.commit()
            .map_err(|e| StorageError::Backend(format!("Commit failed: {}", e)))?;

        self.stats.write().writes += 1;
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> StorageResult<()> {
        let mut wtxn = self.env
            .write_txn()
            .map_err(|e| StorageError::Backend(format!("Write transaction failed: {}", e)))?;

        self.db
            .delete(&mut wtxn, key)
            .map_err(|e| StorageError::Backend(format!("Delete failed: {}", e)))?;

        wtxn.commit()
            .map_err(|e| StorageError::Backend(format!("Commit failed: {}", e)))?;

        self.stats.write().deletes += 1;
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
        let rtxn = self.env
            .read_txn()
            .map_err(|e| StorageError::Backend(format!("Transaction failed: {}", e)))?;

        let iter = self.db
            .range(&rtxn, &(start..end))
            .map_err(|e| StorageError::Backend(format!("Range scan failed: {}", e)))?;

        let results: Vec<_> = iter
            .map(|res| {
                res.map(|(k, v)| (k.to_vec(), v.to_vec()))
                    .map_err(|e| StorageError::Backend(format!("Iteration failed: {}", e)))
            })
            .collect::<StorageResult<Vec<_>>>()?;

        Ok(Box::new(results.into_iter()))
    }

    fn prefix_scan<'a>(
        &'a self,
        prefix: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        let rtxn = self.env
            .read_txn()
            .map_err(|e| StorageError::Backend(format!("Transaction failed: {}", e)))?;

        let iter = self.db
            .prefix_iter(&rtxn, prefix)
            .map_err(|e| StorageError::Backend(format!("Prefix scan failed: {}", e)))?;

        let results: Vec<_> = iter
            .map(|res| {
                res.map(|(k, v)| (k.to_vec(), v.to_vec()))
                    .map_err(|e| StorageError::Backend(format!("Iteration failed: {}", e)))
            })
            .collect::<StorageResult<Vec<_>>>()?;

        Ok(Box::new(results.into_iter()))
    }

    fn flush(&mut self) -> StorageResult<()> {
        // LMDB auto-flushes, but we can force a sync
        self.env
            .force_sync()
            .map_err(|e| StorageError::Backend(format!("Sync failed: {}", e)))?;

        Ok(())
    }

    fn compact(&mut self) -> StorageResult<()> {
        // LMDB doesn't have explicit compaction
        Ok(())
    }

    fn batch_put(&mut self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> StorageResult<()> {
        let mut wtxn = self.env
            .write_txn()
            .map_err(|e| StorageError::Backend(format!("Write transaction failed: {}", e)))?;

        for (key, value) in pairs.iter() {
            self.db
                .put(&mut wtxn, key, value)
                .map_err(|e| StorageError::Backend(format!("Batch put failed: {}", e)))?;
        }

        wtxn.commit()
            .map_err(|e| StorageError::Backend(format!("Commit failed: {}", e)))?;

        self.stats.write().writes += pairs.len() as u64;
        Ok(())
    }

    fn stats(&self) -> StorageStats {
        self.stats.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (LmdbBackend, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let backend = LmdbBackend::new(temp_dir.path()).unwrap();
        (backend, temp_dir)
    }

    #[test]
    fn test_lmdb_basic_operations() {
        let (mut backend, _temp) = create_test_db();

        backend.put(b"key1", b"value1").unwrap();
        assert_eq!(backend.get(b"key1").unwrap(), Some(b"value1".to_vec()));

        backend.delete(b"key1").unwrap();
        assert_eq!(backend.get(b"key1").unwrap(), None);
    }
}
```

---

## Step 4: Test Utilities

**File**: `crates/storage/tests/common/mod.rs`

```rust
//! Shared testing utilities for all storage backends

use storage::{StorageBackend, StorageResult};
use tempfile::TempDir;

/// Test fixture for storage backend testing
pub struct StorageTestFixture {
    pub temp_dir: TempDir,
}

impl StorageTestFixture {
    /// Create a new test fixture with temporary directory
    pub fn new() -> StorageResult<Self> {
        let temp_dir = TempDir::new()
            .map_err(|e| storage::StorageError::Io(e))?;

        Ok(Self { temp_dir })
    }

    /// Get the temporary directory path
    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Generate test keys with pattern
    pub fn generate_keys(count: usize) -> Vec<Vec<u8>> {
        (0..count)
            .map(|i| format!("key_{:06}", i).into_bytes())
            .collect()
    }

    /// Generate test values with pattern
    pub fn generate_values(count: usize) -> Vec<Vec<u8>> {
        (0..count)
            .map(|i| format!("value_{:06}", i).into_bytes())
            .collect()
    }
}

impl Default for StorageTestFixture {
    fn default() -> Self {
        Self::new().expect("Failed to create test fixture")
    }
}

/// Assertion helpers
pub mod assertions {
    use super::*;

    pub fn assert_backend_contains<B: StorageBackend>(
        backend: &B,
        key: &[u8],
        value: &[u8],
    ) -> StorageResult<()> {
        let result = backend.get(key)?;
        assert_eq!(result, Some(value.to_vec()), "Key {:?} not found or value mismatch",
            String::from_utf8_lossy(key));
        Ok(())
    }

    pub fn assert_backend_missing<B: StorageBackend>(
        backend: &B,
        key: &[u8],
    ) -> StorageResult<()> {
        let result = backend.get(key)?;
        assert_eq!(result, None, "Key {:?} should not exist",
            String::from_utf8_lossy(key));
        Ok(())
    }
}
```

---

## Step 5: Sample Test File

**File**: `crates/storage/tests/rocksdb_tests.rs` (LMDB similar)

```rust
//! RocksDB backend tests
//!
//! Run with: cargo test --test rocksdb_tests --features rocksdb-backend

#[cfg(feature = "rocksdb-backend")]
mod rocksdb_backend_tests {
    use storage::{RocksDbBackend, StorageBackend, StorageResult};
    use tempfile::TempDir;

    fn create_test_backend() -> StorageResult<(RocksDbBackend, TempDir)> {
        let temp_dir = TempDir::new().map_err(|e| storage::StorageError::Io(e))?;
        let backend = RocksDbBackend::new(temp_dir.path())?;
        Ok((backend, temp_dir))
    }

    #[test]
    fn test_basic_put_get() -> StorageResult<()> {
        let (mut backend, _temp) = create_test_backend()?;

        backend.put(b"key1", b"value1")?;
        assert_eq!(backend.get(b"key1")?, Some(b"value1".to_vec()));

        Ok(())
    }

    #[test]
    fn test_delete() -> StorageResult<()> {
        let (mut backend, _temp) = create_test_backend()?;

        backend.put(b"key1", b"value1")?;
        backend.delete(b"key1")?;
        assert_eq!(backend.get(b"key1")?, None);

        Ok(())
    }

    #[test]
    fn test_range_scan() -> StorageResult<()> {
        let (mut backend, _temp) = create_test_backend()?;

        for i in 0..10 {
            let key = format!("key_{:02}", i).into_bytes();
            backend.put(&key, &format!("value_{}", i).into_bytes())?;
        }

        let results: Vec<_> = backend.range_scan(b"key_03", b"key_07")?.collect();
        assert!(results.len() >= 3);

        Ok(())
    }

    #[test]
    fn test_prefix_scan() -> StorageResult<()> {
        let (mut backend, _temp) = create_test_backend()?;

        backend.put(b"prefix:1", b"value1")?;
        backend.put(b"prefix:2", b"value2")?;
        backend.put(b"other:1", b"value3")?;

        let results: Vec<_> = backend.prefix_scan(b"prefix:")?.collect();
        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[test]
    fn test_batch_put() -> StorageResult<()> {
        let (mut backend, _temp) = create_test_backend()?;

        let pairs = vec![
            (b"key1".to_vec(), b"value1".to_vec()),
            (b"key2".to_vec(), b"value2".to_vec()),
            (b"key3".to_vec(), b"value3".to_vec()),
        ];

        backend.batch_put(pairs)?;

        assert_eq!(backend.get(b"key1")?, Some(b"value1".to_vec()));
        assert_eq!(backend.get(b"key2")?, Some(b"value2".to_vec()));
        assert_eq!(backend.get(b"key3")?, Some(b"value3".to_vec()));

        Ok(())
    }
}
```

---

## Build & Test Commands

### Build with Feature Flags

```bash
# Build with RocksDB support
cargo build -p storage --features rocksdb-backend

# Build with LMDB support
cargo build -p storage --features lmdb-backend

# Build with all backends
cargo build -p storage --features all-backends
```

### Run Tests

```bash
# Test RocksDB backend
cargo test -p storage --features rocksdb-backend --test rocksdb_tests

# Test LMDB backend
cargo test -p storage --features lmdb-backend --test lmdb_tests

# Test all backends
cargo test -p storage --features all-backends

# Test with output
cargo test -p storage --features all-backends -- --nocapture

# Single test
cargo test -p storage --features rocksdb-backend test_basic_put_get
```

### Run Benchmarks

```bash
# Compare all backends
cargo bench -p storage --bench triple_store_benchmark

# Create new benchmarks in benches/rocksdb_benchmark.rs
cargo bench -p storage --bench rocksdb_benchmark
```

---

## Checklist for Implementation

### RocksDB Backend
- [ ] Create `crates/storage/src/rocksdb_backend.rs`
- [ ] Implement `StorageBackend` trait (10 methods)
- [ ] Add 85 comprehensive tests
- [ ] Build passes: `cargo build --features rocksdb-backend`
- [ ] All tests pass: `cargo test --features rocksdb-backend`
- [ ] Add to `lib.rs` feature gate

### LMDB Backend
- [ ] Create `crates/storage/src/lmdb_backend.rs`
- [ ] Implement `StorageBackend` trait (10 methods)
- [ ] Add 85 comprehensive tests
- [ ] Build passes: `cargo build --features lmdb-backend`
- [ ] All tests pass: `cargo test --features lmdb-backend`
- [ ] Add to `lib.rs` feature gate

### Test Infrastructure
- [ ] Create `crates/storage/tests/common/mod.rs`
- [ ] Create test fixtures and utilities
- [ ] Create assertion helpers
- [ ] Macro patterns for backend-agnostic tests
- [ ] Temporary directory cleanup

### Documentation
- [ ] Update CLAUDE.md with usage examples
- [ ] Add feature flag documentation
- [ ] Document performance characteristics
- [ ] Add troubleshooting guide

---

## Performance Testing Template

```bash
# Test sequence for new backend
./scripts/test_storage_backends.sh

# Or manually:
time cargo test -p storage --features rocksdb-backend -- --test-threads=1

# Measure memory usage
/usr/bin/time -v cargo test -p storage --features rocksdb-backend

# Benchmark specific operation
cargo bench -p storage --bench triple_store_benchmark -- rocksdb
```

---

**Total Implementation Time**: ~17 days
**Code to Write**: ~4,350 LOC
**Test Coverage**: 170 tests (85 per backend)
**Success Rate Target**: 100%

Ready to start? Begin with Step 1 and follow sequentially.
