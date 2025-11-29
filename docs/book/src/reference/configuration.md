# Configuration Guide

Guide to configuring rust-kgdb for different environments and use cases.

## Storage Backend Configuration

### InMemory Backend

Simplest configuration, best for development:

```rust
use rust_kgdb::storage::InMemoryBackend;
use std::sync::Arc;

let backend = InMemoryBackend::new();
let store = Arc::new(backend);

// Use store...
```

**Configuration Options**: None (all data in memory)

**Best For**:
- Development and testing
- Temporary datasets
- Datasets <100MB
- Single-machine deployments

### RocksDB Backend

Persistent disk storage with ACID guarantees:

```rust
use rust_kgdb::storage::RocksDBBackend;
use std::sync::Arc;

let backend = RocksDBBackend::new("./my_database")?;
let store = Arc::new(backend);
```

**Configuration**:

```rust
use rocksdb::Options;

let mut opts = Options::default();
opts.create_if_missing(true);
opts.set_compression(rocksdb::DBCompressionType::Lz4);

let backend = RocksDBBackend::with_options("./db", opts)?;
```

**Environment Variables**:
- `ROCKS_CACHE_SIZE`: Memory cache size in MB (default: 64)
- `ROCKS_BLOCK_SIZE`: Block size in KB (default: 4)
- `ROCKS_COMPRESSION`: Compression type (default: lz4)

**Best For**:
- Production deployments
- Large datasets (100MB - 10GB)
- Multiple concurrent writers
- Data persistence requirements

### LMDB Backend

Memory-mapped files, optimized for read-heavy loads:

```rust
use rust_kgdb::storage::LMDBBackend;
use std::sync::Arc;

let backend = LMDBBackend::new("./my_database")?;
let store = Arc::new(backend);
```

**Configuration**:

```rust
let backend = LMDBBackend::with_max_dbs("./db", 32)?;
```

**Environment Variables**:
- `LMDB_MAP_SIZE`: Maximum map size in MB (default: 1024)
- `LMDB_LOCK`: Enable file locking (default: true)

**Best For**:
- Read-optimized workloads
- Memory-mapped access patterns
- Consistent performance requirements
- Small-to-medium datasets

## Executor Configuration

### Query Timeout

Prevent runaway queries:

```rust
use std::time::Duration;
use rust_kgdb::sparql::ExecutorBuilder;

let executor = ExecutorBuilder::new(store)
    .with_timeout(Duration::from_secs(30))
    .build();

// Queries exceeding 30 seconds fail with timeout error
```

### Result Limits

Cap maximum results:

```rust
let executor = ExecutorBuilder::new(store)
    .with_max_results(100_000)
    .build();

// Queries returning >100K results fail
```

### Optimization Level

Control query optimization aggressiveness:

```rust
use rust_kgdb::sparql::OptimizationLevel;

let executor = ExecutorBuilder::new(store)
    .with_optimization(OptimizationLevel::Aggressive)
    .build();

// Options: Conservative, Normal, Aggressive
```

## Dictionary Configuration

### Concurrency Level

Control dictionary lock contention:

```rust
use rust_kgdb::rdf_model::DictionaryBuilder;

let dict = DictionaryBuilder::new()
    .with_concurrency_level(8)
    .build();
```

**Recommendations**:
- **Single-threaded**: Level 1
- **2-4 threads**: Level 4
- **8+ threads**: Level 16+

### Capacity Hints

Pre-allocate capacity:

```rust
let dict = DictionaryBuilder::new()
    .with_capacity(100_000)  // Expect ~100K interned strings
    .build();
```

## Logging Configuration

### Environment-Based Logging

```bash
# Debug logging for all modules
RUST_LOG=debug cargo run

# Debug for specific module
RUST_LOG=sparql=debug cargo run

# Multiple modules
RUST_LOG=sparql=debug,storage=info cargo run

# Trace level (very verbose)
RUST_LOG=trace cargo run
```

### Programmatic Logging

```rust
use log::LevelFilter;
use env_logger::Builder;

fn init_logging() {
    Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .filter_module("sparql", LevelFilter::Trace)
        .filter_module("storage", LevelFilter::Info)
        .init();
}

init_logging();
```

## Performance Configuration

### Memory Tuning

```rust
use rust_kgdb::storage::StorageConfig;

let config = StorageConfig {
    cache_size: 512 * 1024 * 1024,  // 512 MB cache
    block_size: 4096,                // 4 KB blocks
    compression: CompressionType::Lz4,
    ..Default::default()
};

let backend = RocksDBBackend::with_config("./db", config)?;
```

### Threading Configuration

```rust
use rayon::ThreadPoolBuilder;

// Configure thread pool for parallel operations
ThreadPoolBuilder::new()
    .num_threads(8)
    .build_global()
    .unwrap();
```

## Batch Processing

### Optimal Batch Size

```rust
// For RocksDB
const OPTIMAL_BATCH_SIZE: usize = 10_000;

// For InMemory
const OPTIMAL_BATCH_SIZE: usize = 100_000;

// Process data in batches
for chunk in data.chunks(OPTIMAL_BATCH_SIZE) {
    for item in chunk {
        store.put(&item)?;
    }
    store.commit()?;
}
```

## Connection Pooling

### Pool Configuration

```rust
use r2d2::{Pool, ConnectionManager};

let config = r2d2::Config::default()
    .pool_size(8)
    .min_idle(Some(2));

let pool = Pool::new(config, manager)?;
```

## Feature Flags

Control available features at compile time:

```toml
[dependencies]
rust-kgdb = { version = "0.1", features = [
    "all-backends",      # All storage backends
    "reasoning",         # RDFS/OWL reasoning
    "http-server",       # HTTP API server
    "web-assembly",      # WebAssembly support
] }
```

## Environment Variables

Configuration via environment:

```bash
# Logging
RUST_LOG=debug
RUST_LOG_STYLE=always

# Storage
ROCKS_CACHE_SIZE=512
ROCKS_COMPRESSION=lz4

# Performance
RAYON_NUM_THREADS=8
```

## Configuration Files

### TOML Configuration

**config.toml**:
```toml
[database]
backend = "rocksdb"
path = "./data/kg"
cache_size_mb = 512

[query]
timeout_secs = 30
max_results = 100000
optimization = "aggressive"

[logging]
level = "debug"
modules = ["sparql=trace", "storage=info"]
```

**Load in code**:
```rust
use config::{Config, ConfigError};

fn load_config() -> Result<Config, ConfigError> {
    Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
}
```

### Environment File

**.env**:
```bash
RUST_LOG=debug
DATABASE_BACKEND=rocksdb
DATABASE_PATH=./data
QUERY_TIMEOUT=30
```

**Load with dotenv**:
```rust
use dotenv::dotenv;
use std::env;

fn init_from_env() {
    dotenv().ok();
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string());
}
```

## Platform-Specific Configuration

### Linux

```bash
# Optimize file descriptors
ulimit -n 65536

# Enable CPU affinity
export RAYON_THREAD_AFFINITY=1
```

### macOS

```bash
# Increase file descriptors
ulimit -n 65536

# Disable metal acceleration (optional)
export METAL_DEVICE_WRAPPER_TYPE=cpu
```

### Windows

```batch
:: Increase file handle limit
fsutil resource setAutochk C: /optimizespace:1

:: Configure thread pool
set RAYON_NUM_THREADS=8
```

## Docker Configuration

### Dockerfile

```dockerfile
FROM rust:1.75

WORKDIR /app
COPY . .

RUN cargo build --release

ENV RUST_LOG=info
ENV ROCKS_CACHE_SIZE=512

EXPOSE 8080
CMD ["./target/release/my_app"]
```

### Docker Compose

```yaml
version: '3'
services:
  graphdb:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./data:/data
    environment:
      - RUST_LOG=debug
      - DATABASE_PATH=/data/kg
```

## Kubernetes Configuration

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: graphdb-config
data:
  RUST_LOG: "info"
  ROCKS_CACHE_SIZE: "1024"
  QUERY_TIMEOUT: "30"
```

### StatefulSet

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: graphdb
spec:
  serviceName: graphdb
  replicas: 3
  template:
    spec:
      containers:
      - name: graphdb
        image: graphdb:latest
        envFrom:
        - configMapRef:
            name: graphdb-config
        volumeMounts:
        - name: data
          mountPath: /data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 100Gi
```

## Configuration Validation

### Validate Configuration

```rust
fn validate_config(config: &Config) -> Result<()> {
    if config.cache_size_mb < 64 {
        return Err("Cache size must be >= 64 MB".into());
    }

    if config.timeout_secs > 300 {
        return Err("Timeout must be <= 300 seconds".into());
    }

    Ok(())
}
```

## Configuration Profiles

### Development Profile

```rust
let config = ConfigProfile::Development {
    cache_size: 64 * 1024 * 1024,
    compression: None,
    logging: LogLevel::Debug,
};
```

### Production Profile

```rust
let config = ConfigProfile::Production {
    cache_size: 1024 * 1024 * 1024,
    compression: Some(CompressionType::Lz4),
    logging: LogLevel::Info,
};
```

## See Also

- [Best Practices](../sdk/rust/best-practices.md) - Configuration recommendations
- [Performance Guide](../sdk/rust/performance.md) - Performance tuning
- [Error Handling](./errors.md) - Error configuration
