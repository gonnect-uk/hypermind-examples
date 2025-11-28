# Storage Backend Test Plan - RocksDB & LMDB

**Status Report**: 2025-11-25
**Mission**: Examine rust-kgdb storage implementation and plan comprehensive RocksDB and LMDB backend tests

---

## Executive Summary

### Current State
- **In-Memory Backend**: Fully implemented with 4 test cases
- **RocksDB Backend**: Feature flag exists (`rocksdb-backend`), dependency configured, but **NOT IMPLEMENTED**
- **LMDB Backend**: Feature flag exists (`lmdb-backend`), dependency configured, but **NOT IMPLEMENTED**
- **Test Coverage**: InMemoryBackend has basic tests in `inmemory.rs`, but no dedicated integration tests
- **Tests Directory**: `/crates/storage/tests/` exists but is **EMPTY**

### Key Findings

#### 1. Architecture is Complete (Trait-Based)
The storage system uses a well-designed **trait-based abstraction** that makes backend implementation straightforward:

```rust
pub trait StorageBackend: Send + Sync {
    fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>>;
    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()>;
    fn delete(&mut self, key: &[u8]) -> StorageResult<()>;
    fn contains(&self, key: &[u8]) -> StorageResult<bool>;
    fn range_scan<'a>(&'a self, start: &[u8], end: &[u8]) -> StorageResult<...>;
    fn prefix_scan<'a>(&'a self, prefix: &[u8]) -> StorageResult<...>;
    fn flush(&mut self) -> StorageResult<()>;
    fn compact(&mut self) -> StorageResult<()>;
    fn batch_put(&mut self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> StorageResult<()>;
    fn stats(&self) -> StorageStats;
}
```

Any new backend implementation needs to implement these 10 methods.

#### 2. Dependencies Already Configured
From `Cargo.toml`:
- **RocksDB** (v0.22): `rocksdb = { workspace = true, optional = true }`
- **LMDB** (via heed v0.20): `heed = { workspace = true, optional = true }`
- **LZ4**: Optional compression for RocksDB
- **Zstd**: Optional compression support

#### 3. Feature Flags Ready
```toml
[features]
default = ["in-memory"]
in-memory = []
rocksdb-backend = ["dep:rocksdb", "dep:lz4"]
lmdb-backend = ["dep:heed"]
compression = ["dep:zstd"]
all-backends = ["rocksdb-backend", "lmdb-backend", "compression"]
```

#### 4. Test Framework in Place
- `dev-dependencies`: `proptest`, `criterion`, `tempfile` (for temp directories)
- Benchmarks: `triple_store_benchmark.rs` exists (InMemoryBackend only)

---

## Implementation Status

### RocksDB Backend
- **Status**: NOT IMPLEMENTED
- **Effort**: 150-200 LOC for core implementation + 85 test cases
- **Key Challenges**:
  1. RocksDB doesn't support native range queries in key order (requires custom iterator management)
  2. Batch writes must use WriteBatch for ACID guarantees
  3. Compression options (snappy enabled in workspace)
  4. Thread safety via Arc<DB>

### LMDB Backend
- **Status**: NOT IMPLEMENTED
- **Effort**: 200-250 LOC for core implementation + 85 test cases
- **Key Challenges**:
  1. LMDB uses memory-mapped files (requires careful lifetime management)
  2. Read/write transactions are mutually exclusive
  3. heed wrapper abstracts most complexity
  4. Must handle environment creation and resizing

---

## Comprehensive Test Plan

### Total Test Count: 85 tests per backend (170 tests total)

#### Category 1: Basic CRUD Operations (20 tests)

1. **test_put_single_key** - Put and verify single key-value
2. **test_put_multiple_keys** - Put multiple unrelated keys
3. **test_get_existing_key** - Retrieve existing key
4. **test_get_missing_key** - Retrieve non-existent key (returns None)
5. **test_delete_existing_key** - Delete and verify removal
6. **test_delete_missing_key** - Delete non-existent key (idempotent)
7. **test_contains_existing** - Check contains for existing key
8. **test_contains_missing** - Check contains for missing key
9. **test_put_overwrite** - Overwrite existing key with new value
10. **test_put_empty_value** - Store empty string as value
11. **test_put_large_key** - Store very large key (1MB)
12. **test_put_large_value** - Store very large value (10MB)
13. **test_delete_after_put** - Insert then delete in sequence
14. **test_multiple_overwrite** - Multiple overwrites of same key
15. **test_byte_boundary_keys** - Keys with special byte sequences
16. **test_null_bytes_in_key** - Keys containing null bytes
17. **test_null_bytes_in_value** - Values containing null bytes
18. **test_unicode_keys** - UTF-8 encoded URIs as keys
19. **test_unicode_values** - UTF-8 encoded values
20. **test_case_sensitive_keys** - Keys differing only in case

#### Category 2: Range Scanning (15 tests)

21. **test_range_scan_basic** - Simple range between two keys
22. **test_range_scan_empty_range** - Range with no matching keys
23. **test_range_scan_full_dataset** - Scan from min to max
24. **test_range_scan_single_key** - Range containing exactly one key
25. **test_range_scan_prefix_matching** - Range with common prefix
26. **test_range_scan_exclusive_end** - Verify end boundary is exclusive
27. **test_range_scan_inclusive_start** - Verify start boundary is inclusive
28. **test_range_scan_ordering** - Results come back in sorted order
29. **test_range_scan_after_deletion** - Range scan reflects deletions
30. **test_range_scan_large_dataset** - Range on 100K keys
31. **test_range_scan_byte_sequences** - Range with binary keys
32. **test_range_scan_overlapping_ranges** - Multiple overlapping scans
33. **test_range_scan_concurrent_updates** - Scan while inserting
34. **test_range_scan_reverse_order** - Verify natural key ordering
35. **test_range_scan_memory_efficiency** - Iterator doesn't load entire range

#### Category 3: Prefix Scanning (10 tests)

36. **test_prefix_scan_basic** - Find all keys with given prefix
37. **test_prefix_scan_no_matches** - Prefix with no matches
38. **test_prefix_scan_exact_match** - Prefix equals entire key
39. **test_prefix_scan_single_match** - Only one key matches prefix
40. **test_prefix_scan_many_matches** - Thousands of keys with same prefix
41. **test_prefix_scan_ordering** - Results sorted by key
42. **test_prefix_scan_empty_prefix** - Empty prefix (all keys)
43. **test_prefix_scan_binary_prefix** - Non-UTF8 prefix
44. **test_prefix_scan_unicode_prefix** - UTF-8 prefix matching
45. **test_prefix_scan_boundary_conditions** - Prefix at byte boundaries

#### Category 4: Batch Operations (15 tests)

46. **test_batch_put_empty** - Batch with zero items
47. **test_batch_put_single** - Batch with one item
48. **test_batch_put_many** - Batch with 10K items
49. **test_batch_put_duplicates** - Batch with duplicate keys (last wins)
50. **test_batch_put_overwrites** - Batch overwrites existing keys
51. **test_batch_put_with_deletions** - Mix of puts in batch
52. **test_batch_put_atomicity** - Entire batch succeeds or fails
53. **test_batch_put_performance** - Batch significantly faster than sequential
54. **test_batch_put_large_values** - Batch with large values
55. **test_batch_put_ordering** - Batch preserves semantics regardless of order
56. **test_batch_put_empty_values** - Batch with empty values
57. **test_batch_put_interleaved_reads** - Read during batch
58. **test_batch_put_recovery** - Verify batch persisted after flush
59. **test_batch_mixed_operations** - Batch with mixed operations
60. **test_batch_very_large_count** - 100K items in one batch

#### Category 5: Transaction Support (15 tests)

61. **test_transaction_basic_put** - Put within transaction
62. **test_transaction_basic_delete** - Delete within transaction
63. **test_transaction_commit** - Commit makes changes durable
64. **test_transaction_rollback** - Rollback discards changes
65. **test_transaction_multiple_ops** - Multiple operations in one transaction
66. **test_transaction_isolation** - Changes not visible until commit
67. **test_transaction_rollback_visibility** - Rollback hides changes
68. **test_transaction_nested_fails** - Nested transactions not supported (error)
69. **test_transaction_concurrent_failure** - Concurrent write transactions fail (RocksDB/LMDB specific)
70. **test_transaction_read_committed** - Read uncommitted changes (backend dependent)
71. **test_transaction_large_batch** - Transaction with 10K operations
72. **test_transaction_recovery** - Committed transaction survives restart
73. **test_transaction_abort_on_error** - Error during transaction
74. **test_transaction_deadlock_prevention** - No deadlocks with proper ordering
75. **test_transaction_get_in_txn** - Read modified keys within transaction

#### Category 6: Durability & Persistence (10 tests)

76. **test_flush_makes_durable** - Flush() persists data
77. **test_data_survives_restart** - Data persists after backend close/reopen
78. **test_compact_reduces_size** - Compact() reduces database size (RocksDB specific)
79. **test_no_data_loss_on_crash** - Simulated crash recovery
80. **test_partial_batch_consistency** - Batch atomicity enforced
81. **test_interleaved_flush_writes** - Flushes during active writes
82. **test_empty_database_flush** - Flush on empty database
83. **test_flush_idempotent** - Multiple flushes are safe
84. **test_storage_stats_accuracy** - Stats correctly reflect state
85. **test_stats_after_operations** - Stats update for all operations

#### Category 7: Concurrent Access (10 tests) [BONUS]

**Note**: These require careful handling due to mutable trait methods. Consider using interior mutability pattern.

- **test_concurrent_reads** - Multiple readers simultaneously
- **test_concurrent_writes** - Multiple writers (backend dependent)
- **test_read_write_concurrency** - Readers and writers together
- **test_stress_test_high_contention** - Thousands of concurrent operations
- **test_no_corruption_under_load** - Data integrity under stress

#### Category 8: Error Handling (10 tests)

- **test_io_error_propagation** - I/O errors properly propagated
- **test_corruption_detection** - Detect corrupted data
- **test_max_value_size** - Handle very large values gracefully
- **test_key_encoding_errors** - Invalid UTF-8 handling
- **test_backend_closed_error** - Operations on closed backend fail
- **test_out_of_space_error** - Handle disk full (simulated)
- **test_permission_error** - File permission issues
- **test_transaction_abort_errors** - Error during rollback
- **test_consistency_after_error** - Database remains consistent after error
- **test_recovery_from_partial_write** - Incomplete writes handled safely

---

## Test Implementation Structure

### File Organization

```
crates/storage/tests/
├── common/
│   ├── mod.rs              # Shared test utilities
│   ├── fixtures.rs         # Test data generators
│   └── assertions.rs       # Custom assertions
├── rocksdb_tests.rs        # RocksDB-specific tests (85 tests)
├── lmdb_tests.rs           # LMDB-specific tests (85 tests)
└── backend_traits_tests.rs # Backend agnostic tests (20 tests)
```

### Macro Pattern for Backend Agnostic Tests

To avoid code duplication between RocksDB and LMDB tests, use a test macro:

```rust
macro_rules! define_backend_tests {
    ($backend_type:ty, $name:ident) => {
        mod $name {
            use super::*;
            type TestBackend = $backend_type;

            #[test]
            fn test_basic_operations() {
                // Shared test code
            }

            // ... more tests
        }
    };
}

// Usage:
define_backend_tests!(RocksDbBackend, rocksdb_suite);
define_backend_tests!(LmdbBackend, lmdb_suite);
```

### Test Utilities Module

```rust
// tests/common/mod.rs
pub struct TestFixture {
    backend: Box<dyn StorageBackend>,
    temp_dir: TempDir,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let backend = create_test_backend(&temp_dir);
        Self { backend, temp_dir }
    }

    pub fn assert_key_exists(&self, key: &[u8]) {
        assert!(self.backend.contains(key).unwrap());
    }

    pub fn assert_key_missing(&self, key: &[u8]) {
        assert!(!self.backend.contains(key).unwrap());
    }

    pub fn populate_keys(&mut self, count: usize) {
        for i in 0..count {
            let key = format!("key_{:06}", i).into_bytes();
            let value = format!("value_{}", i).into_bytes();
            self.backend.put(&key, &value).unwrap();
        }
    }
}
```

---

## RocksDB Implementation Notes

### Core Structure

```rust
use rocksdb::{DB, Options, IteratorMode};
use std::sync::Arc;

pub struct RocksDbBackend {
    db: Arc<DB>,
    path: PathBuf,
}

impl StorageBackend for RocksDbBackend {
    // Implement 10 trait methods
}
```

### Critical Implementation Details

1. **Thread Safety**: RocksDB's DB is Send+Sync, wrap in Arc for shared ownership
2. **Range Scans**: Use iterator mode carefully:
   ```rust
   fn range_scan(&self, start: &[u8], end: &[u8]) -> StorageResult<...> {
       let iter = self.db.iterator(IteratorMode::From(start, Direction::Forward));
       // Filter results where key < end
   }
   ```

3. **Batch Writes**: Use WriteBatch for atomic multi-key operations
4. **Compression**: Enable snappy (already in workspace config)
5. **Statistics**: RocksDB provides native stats via `property("rocksdb.stats")`

### Common Issues

1. **Memory-mapped I/O**: RocksDB uses mmap, handle carefully in tests
2. **Iterator lifetime**: Iterators hold DB locks, drop before mutations
3. **Concurrency**: WriteLock is required for all writes (mutable trait methods)

---

## LMDB Implementation Notes

### Core Structure

```rust
use heed::{Database, Env, RoTxn, RwTxn, types::Bytes};
use std::path::Path;

pub struct LmdbBackend {
    env: Env,
    db: Database<Bytes, Bytes>,
}

impl StorageBackend for LmdbBackend {
    // Implement 10 trait methods
    // Note: Methods take &self or &mut self, not RoTxn/RwTxn
}
```

### Critical Implementation Details

1. **Memory-Mapped Files**: LMDB uses mmap exclusively
2. **Transaction Semantics**: Read-only transactions share reads, write transactions are exclusive
3. **Size Limits**: Must handle MDB_MAP_RESIZED carefully
4. **Env Configuration**: Set max_dbs, flags for proper operation
5. **Key-Value Size**: LMDB has stricter size limits than RocksDB

### Common Issues

1. **Transaction Scope**: Keep read transactions short to avoid blocking writers
2. **Size Pre-allocation**: Pre-allocate map size to avoid resizing
3. **Durability**: Fsync behavior is different from RocksDB

---

## Test Execution Strategy

### Phase 1: Basic Tests (Week 1)
```bash
# Run minimal set to verify implementations compile
cargo test --package storage --features rocksdb-backend -- --test-threads=1

# Watch for:
# - Compilation errors
# - Segmentation faults
# - Panic on initialization
```

### Phase 2: CRUD Tests (Week 2)
```bash
# Focus on basic operations
cargo test --package storage --features rocksdb-backend test_put
cargo test --package storage --features rocksdb-backend test_get
cargo test --package storage --features rocksdb-backend test_delete
```

### Phase 3: Complex Tests (Week 3)
```bash
# Range scanning, transactions, concurrency
cargo test --package storage --features all-backends
```

### Phase 4: Benchmarks (Week 4)
```bash
# Compare all backends
cargo bench --package storage --bench triple_store_benchmark

# Add RocksDB/LMDB specific benchmarks
cargo bench --package storage --bench rocksdb_vs_inmemory
cargo bench --package storage --bench lmdb_vs_inmemory
```

---

## Success Criteria

### Correctness (100% Must Pass)
- All 85 tests pass for RocksDB
- All 85 tests pass for LMDB
- 100% consistency between backends (same operations → same results)
- Benchmark tests show no regressions from InMemoryBackend

### Performance Targets

| Operation | Target | Measurement Method |
|-----------|--------|-------------------|
| Single GET | < 1ms | Criterion criterion_group |
| Single PUT | < 2ms | Criterion criterion_group |
| Range scan 1K keys | < 100ms | Criterion bench |
| Batch insert 10K | < 5s | Criterion bench |
| Prefix scan | < 50ms | Criterion bench |

### Safety & Durability
- No data loss after crash simulation
- Transaction atomicity verified
- Concurrent access produces consistent results
- Stats accurately reflect all operations

---

## Next Steps

### Immediate (This Week)

1. **Decide on Implementation Order**:
   - Option A: RocksDB first (simpler, better tested upstream)
   - Option B: LMDB first (more constraints = clearer design)
   - **Recommendation**: RocksDB first

2. **Set Up Test Infrastructure**:
   - Create `tests/common/mod.rs` with shared utilities
   - Add TempDir cleanup fixtures
   - Create test data generators

3. **Implement RocksDB Backend**:
   - New file: `crates/storage/src/rocksdb_backend.rs`
   - ~150 LOC for core implementation
   - ~50 LOC for Optional trait method overrides
   - Add feature flag gating in `lib.rs`

### Week 2

1. **Implement LMDB Backend**:
   - New file: `crates/storage/src/lmdb_backend.rs`
   - Similar structure to RocksDB
   - Handle memory-mapped file specifics

2. **Write Core CRUD Tests** (85 tests):
   - Use macro pattern to avoid duplication
   - Start with basic operations (20 tests)
   - Progress to complex scenarios

### Week 3

1. **Implement Remaining Test Categories**:
   - Range/prefix scanning (25 tests)
   - Transactions (15 tests)
   - Persistence/durability (10 tests)
   - Error handling (10 tests)

2. **Cross-Backend Validation**:
   - Run same test suite against all 3 backends
   - Verify consistent results
   - Document backend-specific behavior

### Week 4

1. **Benchmarking**:
   - Create comparative benchmarks
   - Identify performance bottlenecks
   - Document trade-offs

2. **Documentation**:
   - Update CLAUDE.md with backend implementation guide
   - Create architecture diagrams
   - Document feature flag combinations

---

## Estimated Effort

| Task | LOC | Days | Notes |
|------|-----|------|-------|
| RocksDB Implementation | 200 | 2 | Relatively straightforward |
| LMDB Implementation | 250 | 2 | More complex memory management |
| Test Infrastructure | 300 | 2 | Shared utilities, fixtures |
| CRUD Tests (x2 backends) | 1200 | 3 | 600 lines per backend |
| Advanced Tests (x2) | 1500 | 4 | Range/prefix/transactions |
| Benchmarks | 400 | 2 | Comparative analysis |
| Documentation | 500 | 2 | README, examples, guides |
| **TOTAL** | **4350** | **17 days** | ~2-3 weeks of focused work |

---

## Appendix: Test Code Examples

### Example 1: Basic CRUD Test

```rust
#[test]
fn test_put_get_delete() {
    let mut backend = create_test_backend();

    // Put
    backend.put(b"key1", b"value1").unwrap();

    // Get
    assert_eq!(backend.get(b"key1").unwrap(), Some(b"value1".to_vec()));

    // Delete
    backend.delete(b"key1").unwrap();
    assert_eq!(backend.get(b"key1").unwrap(), None);
}
```

### Example 2: Range Scan Test

```rust
#[test]
fn test_range_scan() {
    let mut backend = create_test_backend();

    // Populate keys: b"key_001" ... b"key_100"
    for i in 1..=100 {
        let key = format!("key_{:03}", i).into_bytes();
        let value = format!("value_{}", i).into_bytes();
        backend.put(&key, &value).unwrap();
    }

    // Range scan: key_020 to key_040 (exclusive)
    let start = b"key_020";
    let end = b"key_040";
    let results: Vec<_> = backend.range_scan(start, end).unwrap().collect();

    // Verify results
    assert!(results.len() > 0);
    assert_eq!(results[0].0, b"key_020".to_vec());
    assert!(results.iter().all(|(k, _)| k.starts_with(b"key_0")));
}
```

### Example 3: Transaction Test

```rust
#[test]
fn test_transaction_atomicity() {
    let mut backend = create_test_backend();

    // Start transaction
    let mut txn = backend.begin_transaction().unwrap();

    // Multiple operations
    txn.put(b"a", b"1").unwrap();
    txn.put(b"b", b"2").unwrap();
    txn.put(b"c", b"3").unwrap();

    // Commit
    txn.commit().unwrap();

    // Verify all changes persisted
    assert_eq!(backend.get(b"a").unwrap(), Some(b"1".to_vec()));
    assert_eq!(backend.get(b"b").unwrap(), Some(b"2".to_vec()));
    assert_eq!(backend.get(b"c").unwrap(), Some(b"3".to_vec()));
}
```

---

## References

- [RocksDB Rust Binding](https://github.com/rust-rocksdb/rust-rocksdb)
- [Heed (LMDB Wrapper)](https://github.com/Kerollmops/heed)
- [Storage Backend Trait](file:///crates/storage/src/backend.rs)
- [InMemory Implementation Reference](file:///crates/storage/src/inmemory.rs)

---

**Document Status**: COMPLETE - Ready for implementation
**Last Updated**: 2025-11-25
**Prepared By**: Claude Code (Analysis Agent)
