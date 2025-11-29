# API Reference

Complete reference documentation for rust-kgdb APIs.

## Overview

This page links to comprehensive API documentation for all rust-kgdb components.

## Core APIs

### RDF Model API
- **Module**: `rust_kgdb::rdf_model`
- **Types**: `Node`, `Triple`, `Quad`, `Dictionary`
- **Purpose**: RDF data types and string interning
- **See Also**: [SDK API Guide](../sdk/rust/api.md)

**Key Types**:
```rust
pub enum Node<'a> {
    IRI(&'a str),
    Literal(&'a str, &'a str),
    BlankNode(u64),
    QuotedTriple(Box<Triple<'a>>),
    Variable(&'a str),
}

pub struct Triple<'a> {
    pub subject: Node<'a>,
    pub predicate: Node<'a>,
    pub object: Node<'a>,
}

pub struct Dictionary {
    // String interning implementation
}
```

### Storage API
- **Module**: `rust_kgdb::storage`
- **Trait**: `StorageBackend`
- **Implementations**: `InMemoryBackend`, `RocksDBBackend`, `LMDBBackend`
- **Purpose**: Persistent triple storage
- **See Also**: [SDK API Guide](../sdk/rust/api.md)

**Backend Trait**:
```rust
pub trait StorageBackend {
    fn put(&self, quad: &Quad) -> Result<()>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn scan(&self, prefix: &[u8]) -> Result<ScanIterator>;
    fn delete(&self, key: &[u8]) -> Result<()>;
    fn commit(&self) -> Result<()>;
}
```

### SPARQL API
- **Module**: `rust_kgdb::sparql`
- **Type**: `Executor`
- **Purpose**: Query and update execution
- **See Also**: [SDK API Guide](../sdk/rust/api.md)

**Executor Methods**:
```rust
pub struct Executor {
    // Query execution engine
}

impl Executor {
    pub fn execute_query(&self, query: &str, dict: &Dictionary) -> Result<Vec<Binding>>;
    pub fn execute_construct(&self, query: &str, dict: &Dictionary) -> Result<Vec<Triple>>;
    pub fn execute_ask(&self, query: &str, dict: &Dictionary) -> Result<bool>;
    pub fn execute_update(&self, update: &str, dict: &Dictionary) -> Result<()>;
}
```

### Reasoning API
- **Module**: `rust_kgdb::reasoning`
- **Types**: `RDFSReasoner`, `OWL2RLReasoner`
- **Purpose**: Semantic inference
- **See Also**: [Core Concepts](../getting-started/core-concepts.md#reasoning)

### RDF IO API
- **Module**: `rust_kgdb::rdf_io`
- **Types**: `TurtleParser`, `NTriplesParser`
- **Purpose**: RDF file parsing
- **See Also**: [First Steps](../getting-started/first-steps.md#step-7-loading-rdf-files)

## API Documentation Structure

### Stable APIs (v0.1.0+)

Core types and functions guaranteed not to change:

- `Node` enum
- `Triple` struct
- `Dictionary` type
- `StorageBackend` trait
- `Executor` type

### Experimental APIs

May change without notice before v1.0:

- Custom function registration
- Query optimization hints
- Advanced indexing strategies

## Error Handling

All APIs return `Result<T>` where errors are:

```rust
pub enum Error {
    StorageError(String),
    ParseError(String),
    QueryError(String),
    ReasoningError(String),
    IOError(io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Feature Flags

Control which APIs are available:

```toml
# All storage backends
features = ["all-backends"]

# Individual backends
features = ["rocksdb-backend", "lmdb-backend"]

# Reasoning
features = ["reasoning"]

# Web/HTTP
features = ["http-server"]
```

## Thread Safety

- `Dictionary`: Thread-safe (Arc-wrapped)
- `StorageBackend`: Thread-safe implementations
- `Executor`: Thread-safe for concurrent queries
- `Node`: Safe (lifetime-bound, no mutation)

## Performance Characteristics

- **Lookup**: O(log n) with indexes
- **Insert**: O(log n) amortized
- **Scan**: O(k) where k is result size
- **Memory**: 24 bytes per triple + index overhead

## See Also

- [SDK API Guide](../sdk/rust/api.md) - Complete Rust API reference
- [Code Examples](../sdk/rust/examples.md) - Practical usage patterns
- [Best Practices](../sdk/rust/best-practices.md) - Design recommendations
