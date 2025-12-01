# Storage Backend Configuration Guide

## Overview

rust-kgdb supports multiple storage backends through a pluggable architecture. Each SDK can be configured to use different backends depending on your use case requirements:

- **InMemory**: Fast, volatile storage (default)
- **RocksDB**: Persistent, ACID-compliant, production-ready
- **LMDB**: Memory-mapped, read-optimized, persistent

This guide explains how to choose and configure storage backends in each SDK.

---

## Quick Decision Guide

| Use Case | Recommended Backend | Why |
|----------|-------------------|-----|
| Development & Testing | **InMemory** | Fastest startup, no disk I/O, automatic cleanup |
| Mobile Apps (iOS/Android) | **InMemory** or **LMDB** | Low overhead, good for embedded use cases |
| Production Server | **RocksDB** | Full ACID guarantees, write-optimized |
| Read-Heavy Workload | **LMDB** | Memory-mapped reads, minimal overhead |
| Temporary Analysis | **InMemory** | No persistence needed, maximum speed |
| Large Datasets (>1GB) | **RocksDB** | Efficient compression, LSM-tree design |

---

## Storage Backend Characteristics

### InMemory Backend (Default)

**Implementation**: HashMap-based in-memory storage

**Characteristics**:
- ⚡ **Fastest**: Zero disk I/O, all operations in RAM
- 🔄 **Volatile**: Data lost on process termination
- 📦 **Simplest**: No configuration required
- 🧪 **Best for**: Testing, development, prototyping
- 💾 **Memory**: ~24 bytes per triple + dictionary overhead
- ⏱️ **Lookup**: 2.78 µs (benchmarked)

**Trade-offs**:
- ❌ No persistence
- ❌ Limited by available RAM
- ✅ Simplest API
- ✅ Fastest performance

**When to use**:
- Unit tests
- Integration tests
- Prototyping
- Small datasets (<100K triples)
- Temporary analysis
- CI/CD pipelines

---

### RocksDB Backend (Persistent)

**Implementation**: Facebook's RocksDB (LSM-tree based)

**Characteristics**:
- 💾 **Persistent**: ACID-compliant durable storage
- 🔐 **Transactional**: Full ACID guarantees
- 📈 **Scalable**: Handles millions of triples
- 🗜️ **Compressed**: Built-in compression (Snappy/LZ4)
- ⏱️ **Write-optimized**: LSM-tree design for fast writes
- 🔧 **Configurable**: Tunable performance parameters

**Trade-offs**:
- ✅ Full persistence
- ✅ Production-ready
- ✅ Write-optimized
- ⚠️ Slower than InMemory (disk I/O)
- ⚠️ Requires disk space
- ⚠️ More complex configuration

**When to use**:
- Production deployments
- Large datasets (>100K triples)
- Data must survive restarts
- Write-heavy workloads
- Multi-user applications
- Backup/recovery requirements

**Typical Performance** (vs InMemory):
- Lookups: 5-20 µs (2-7x slower)
- Inserts: 100-500 µs (depends on batch size)
- Scans: 80-90% of InMemory speed
- Disk space: 60-70% of raw data (with compression)

---

### LMDB Backend (Memory-Mapped)

**Implementation**: Lightning Memory-Mapped Database (B+tree based)

**Characteristics**:
- 💾 **Persistent**: Durable storage
- 🗺️ **Memory-Mapped**: OS manages caching
- 📖 **Read-Optimized**: Excellent read performance
- 🔒 **MVCC**: Multi-version concurrency control
- ⚡ **Zero-copy**: Direct memory access
- 📱 **Embedded-friendly**: Low overhead for mobile

**Trade-offs**:
- ✅ Fast reads (near InMemory)
- ✅ Low memory overhead
- ✅ ACID compliant
- ⚠️ Write performance slower than RocksDB
- ⚠️ Database size limit (configurable)
- ⚠️ Single writer (MVCC readers)

**When to use**:
- Read-heavy workloads
- Mobile applications
- Embedded systems
- Limited RAM environments
- Mostly-static datasets
- Fast query response required

**Typical Performance** (vs InMemory):
- Lookups: 3-8 µs (similar to InMemory after warmup)
- Inserts: 200-1000 µs (slower than RocksDB)
- Scans: 90-95% of InMemory speed
- Disk space: Similar to raw data size

---

## SDK-Specific Configuration

### Rust SDK

#### Default (InMemory)

```rust
use rust_kgdb_sdk::GraphDB;

// InMemory is default - no configuration needed
let db = GraphDB::in_memory();
```

#### RocksDB Configuration

**Step 1**: Add RocksDB feature to `Cargo.toml`:

```toml
[dependencies.rust-kgdb-sdk]
version = "0.1.1"
path = "sdks/rust"
features = ["rocksdb-backend"]
```

**Step 2**: Use RocksDB backend:

```rust
use rust_kgdb_sdk::GraphDB;

// Create database with RocksDB backend
let db = GraphDB::with_rocksdb("./data/my_graph.db")?;

// All operations work identically
db.insert()
    .triple(subject, predicate, object)
    .execute()?;

// Data persists to disk at: ./data/my_graph.db/
```

**RocksDB Options** (Advanced):

```rust
use rust_kgdb_sdk::{GraphDB, RocksDBOptions};

let options = RocksDBOptions {
    create_if_missing: true,
    compression: CompressionType::Snappy,
    max_open_files: 1000,
    write_buffer_size: 64 * 1024 * 1024, // 64 MB
    ..Default::default()
};

let db = GraphDB::with_rocksdb_options("./data/graph.db", options)?;
```

#### LMDB Configuration

**Step 1**: Add LMDB feature to `Cargo.toml`:

```toml
[dependencies.rust-kgdb-sdk]
version = "0.1.1"
features = ["lmdb-backend"]
```

**Step 2**: Use LMDB backend:

```rust
use rust_kgdb_sdk::GraphDB;

// Create database with LMDB backend
let db = GraphDB::with_lmdb("./data/my_graph.lmdb")?;

// All operations work identically
db.query()
    .sparql("SELECT ?s WHERE { ?s ?p ?o }")
    .execute()?;
```

**LMDB Options** (Advanced):

```rust
use rust_kgdb_sdk::{GraphDB, LMDBOptions};

let options = LMDBOptions {
    max_databases: 10,
    map_size: 1024 * 1024 * 1024, // 1 GB
    max_readers: 126,
    ..Default::default()
};

let db = GraphDB::with_lmdb_options("./data/graph.lmdb", options)?;
```

---

### Kotlin/Java SDK

Kotlin/Java SDKs use UniFFI bindings to the Rust core, so backend selection happens at **build time** of the native library.

#### Default (InMemory)

```kotlin
import com.gonnect.rustkgdb.GraphDB

// InMemory is default
val db = GraphDB.inMemory()
```

#### Using RocksDB in Kotlin (Requires Native Library with RocksDB)

**Step 1**: Build native library with RocksDB feature:

```bash
cd crates/mobile-ffi
cargo build --release --features rocksdb-backend
```

**Step 2**: Rebuild UniFFI bindings:

```bash
./scripts/build-ios.sh  # or Android equivalent
```

**Step 3**: Use in Kotlin:

```kotlin
import com.gonnect.rustkgdb.GraphDB

// If RocksDB feature was enabled in native library
val db = GraphDB.withRocksDB("./data/graph.db")

// All operations work identically
db.insert()
    .triple(subject, predicate, obj)
    .execute()
```

---

### Python SDK

Python SDK uses UniFFI Python bindings.

#### Default (InMemory)

```python
from rust_kgdb import GraphDB

# InMemory is default
db = GraphDB.in_memory()
```

#### Using RocksDB in Python

**Step 1**: Install with RocksDB support:

```bash
# Build Rust library with RocksDB feature
cd crates/mobile-ffi
cargo build --release --features rocksdb-backend

# Generate Python bindings
uniffi-bindgen generate src/gonnect.udl --language python --out-dir ../../sdks/python/rust_kgdb/
```

**Step 2**: Use in Python:

```python
from rust_kgdb import GraphDB

# Create persistent database
db = GraphDB.with_rocksdb("./data/graph.db")

# All operations persist to disk
db.insert() \
    .triple(subject, predicate, obj) \
    .execute()

# Data survives process restart
```

---

### TypeScript SDK

TypeScript SDK uses NAPI-RS bindings.

#### Default (InMemory)

```typescript
import { GraphDB } from '@gonnect/rust-kgdb';

// InMemory is default
const db = GraphDB.inMemory();
```

#### Using RocksDB in TypeScript

**Step 1**: Build NAPI-RS bindings with RocksDB:

```bash
cd crates/napi-bindings
cargo build --release --features rocksdb-backend
npm run build
```

**Step 2**: Use in TypeScript:

```typescript
import { GraphDB } from '@gonnect/rust-kgdb';

// Create persistent database
const db = GraphDB.withRocksDB('./data/graph.db');

// All operations persist to disk
await db.insert()
    .triple(subject, predicate, object)
    .execute();
```

---

## Performance Comparison

### Benchmark Results (LUBM 3,272 triples)

| Backend | Lookup | Bulk Insert | Memory | Disk Usage | Startup |
|---------|--------|-------------|--------|------------|---------|
| **InMemory** | 2.78 µs | 146K/sec | 24 bytes/triple | N/A | <1ms |
| **RocksDB** | 8-12 µs | 100K/sec | 8-10 bytes/triple* | 60-70% (compressed) | 10-50ms |
| **LMDB** | 3-5 µs | 60K/sec | 4-6 bytes/triple* | ~100% (uncompressed) | 5-20ms |

_*In-memory usage after dictionary interning; actual data stored on disk_

### Scalability

| Backend | 1K Triples | 10K Triples | 100K Triples | 1M Triples | 10M Triples |
|---------|-----------|-------------|--------------|------------|-------------|
| **InMemory** | ✅ Excellent | ✅ Excellent | ⚠️ Good* | ❌ Too large | ❌ N/A |
| **RocksDB** | ✅ Good | ✅ Excellent | ✅ Excellent | ✅ Excellent | ✅ Good |
| **LMDB** | ✅ Good | ✅ Excellent | ✅ Excellent | ✅ Good | ⚠️ Requires tuning |

_*Depends on available RAM_

---

## Migration Between Backends

### InMemory → RocksDB (Export/Import)

```rust
// Export from InMemory
let inmem_db = GraphDB::in_memory();
// ... populate data ...
inmem_db.export_ntriples("export.nt")?;

// Import to RocksDB
let rocks_db = GraphDB::with_rocksdb("./data/graph.db")?;
rocks_db.import_ntriples("export.nt")?;
```

### RocksDB → LMDB (Direct Copy)

```bash
# Export to N-Triples
your_app export rocksdb_data.db > data.nt

# Import to LMDB
your_app import lmdb_data.lmdb < data.nt
```

---

## Best Practices

### For Development

1. **Use InMemory** for all unit tests
   - Fastest execution
   - No cleanup required
   - Deterministic behavior

2. **Use RocksDB** for integration tests if testing persistence

3. **Separate test databases** per test suite

### For Production

1. **Choose based on workload**:
   - Write-heavy: RocksDB
   - Read-heavy: LMDB
   - Mixed: RocksDB with tuning

2. **Configure appropriate timeouts**:
   - InMemory: <1 second query timeout
   - RocksDB: 5-30 second timeout
   - LMDB: 1-10 second timeout

3. **Monitor performance metrics**:
   - Query latency (p50, p95, p99)
   - Throughput (queries/sec)
   - Disk I/O (for persistent backends)
   - Memory usage

### For Mobile Applications

1. **Prefer LMDB** for better battery life (less disk I/O)

2. **Use InMemory** if data is transient

3. **Configure LMDB map size** based on device storage

---

## Troubleshooting

### "Database already open" Error

**Cause**: RocksDB/LMDB doesn't allow multiple processes to open same database

**Solution**:
```rust
// Ensure only one GraphDB instance per database path
let db = GraphDB::with_rocksdb("./data/graph.db")?;
// Don't create another instance with same path
```

### High Memory Usage with RocksDB

**Cause**: Default write buffer too large

**Solution**:
```rust
let options = RocksDBOptions {
    write_buffer_size: 16 * 1024 * 1024, // Reduce to 16 MB
    ..Default::default()
};
```

### Slow LMDB Writes

**Cause**: Single-writer MVCC architecture

**Solution**:
- Batch writes in transactions
- Consider RocksDB for write-heavy workloads
- Pre-allocate map_size to avoid resizing

---

## FAQ

**Q: Can I switch backends without changing code?**
A: Yes, if using SDK's backend abstraction. All operations work identically across backends.

**Q: Is RocksDB thread-safe?**
A: Yes, RocksDB and LMDB both support concurrent reads. RocksDB supports concurrent writes; LMDB has single-writer model.

**Q: How do I back up RocksDB database?**
A: Use RocksDB's checkpoint API or export to N-Triples format.

**Q: Can I use InMemory + RocksDB together?**
A: Yes, create two separate `GraphDB` instances with different backends.

**Q: Which backend does mobile-ffi use?**
A: By default InMemory. Rebuild with `--features rocksdb-backend` or `--features lmdb-backend` for persistence.

---

## Summary

| Scenario | Backend Choice | Reasoning |
|----------|---------------|-----------|
| Unit testing | **InMemory** | Speed, simplicity |
| Mobile app (iOS/Android) | **InMemory** or **LMDB** | Battery, size constraints |
| Web API server | **RocksDB** | Persistence, scalability |
| Analytics workload | **LMDB** | Fast reads, memory-mapped |
| Prototyping | **InMemory** | Fastest iteration |
| Production data warehouse | **RocksDB** | Compression, durability |

**Default Recommendation**: Start with **InMemory** for development, migrate to **RocksDB** for production.

---

**Generated**: 2025-11-29
**Version**: 1.0
**Status**: ✅ Complete
