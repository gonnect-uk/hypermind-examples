# Rust SDK Overview

The Rust SDK is the primary interface for rust-kgdb. It provides direct access to all RDF triple store operations, SPARQL queries, and semantic reasoning capabilities.

## What You Can Do

- **Load RDF Data**: Parse and store RDF from Turtle, N-Triples, RDF/XML
- **Query with SPARQL**: Execute SELECT, CONSTRUCT, ASK, and DESCRIBE queries
- **Update Data**: INSERT, DELETE, LOAD, and CLEAR operations
- **Semantic Reasoning**: RDFS and OWL 2 RL reasoning
- **Choose Storage**: InMemory, RocksDB, or LMDB backends
- **Custom Functions**: Register and execute custom SPARQL functions
- **Performance**: Achieves 2.78 µs lookup speed with 24 bytes/triple

## Quick Example

```rust
use rust_kgdb::storage::InMemoryBackend;
use rust_kgdb::rdf_model::Dictionary;
use rust_kgdb::sparql::Executor;
use std::sync::Arc;

let dict = Arc::new(Dictionary::new());
let store = Arc::new(InMemoryBackend::new());

// Add data
let subject = dict.intern("http://example.org/Alice");
let predicate = dict.intern("http://example.org/knows");
let object = dict.intern("http://example.org/Bob");
store.put(&Triple::new(subject, predicate, object))?;

// Query
let results = Executor::new(store).execute_query(
    "SELECT ?x WHERE { ?x <http://example.org/knows> ?y }",
    &dict
)?;
```

## Documentation Structure

- **[Detailed Overview](./overview.md)** - Architecture and design
- **[API Guide](./api.md)** - Complete API reference
- **[Code Examples](./examples.md)** - Real-world usage patterns
- **[Best Practices](./best-practices.md)** - Tips and recommendations
- **[Performance Guide](./performance.md)** - Optimization strategies

## Getting Started

1. [Quick Start](../../getting-started/quick-start.md) - 5-minute setup
2. [Installation](../../getting-started/installation.md) - Detailed setup
3. [First Steps](../../getting-started/first-steps.md) - Interactive tutorial
4. [Core Concepts](../../getting-started/core-concepts.md) - RDF and SPARQL fundamentals

## Supported Rust Versions

- **Minimum**: 1.70
- **Recommended**: 1.75+
- **LTS Supported**: Yes

## Feature Flags

```toml
# All backends
features = ["all-backends"]

# Individual backends
features = ["rocksdb-backend"]
features = ["lmdb-backend"]

# Reasoning
features = ["reasoning", "rdfs", "owl2-rl"]
```

## Common Use Cases

### Knowledge Graph for Web Application
Store facts about entities and query relationships using SPARQL.

### Semantic Search
Use RDF properties and custom predicates for intelligent search.

### Data Integration
Combine data from multiple sources using RDF and SPARQL.

### Compliance and Auditing
Track entity relationships and reasoning decisions with provenance.

## Performance Characteristics

| Operation | Speed | Notes |
|-----------|-------|-------|
| Simple Lookup | 2.78 µs | Single triple pattern |
| BGP Query (3 triples) | <100 µs | Basic graph pattern |
| Bulk Insert (100K) | 682 ms | 146K triples/sec |
| Dictionary Lookup | 60.4 µs | Cached access |

See [Performance Guide](./performance.md) for optimization tips.

## Mobile Development

rust-kgdb supports iOS and Android development:

- **iOS**: XCFramework with Swift bindings
- **Android**: Native library with Kotlin FFI
- **Memory**: Optimized for mobile constraints
- **Performance**: Sub-millisecond queries on device

## Versioning

rust-kgdb follows semantic versioning (semver):
- **v0.x.y**: Pre-release (API may change)
- **v1.x.y**: Stable (API guarantees)

Current version: **0.1.0** (pre-release)

## Next Steps

- [Detailed Overview](./overview.md) - Learn the architecture
- [API Guide](./api.md) - Explore available functions
- [Code Examples](./examples.md) - See practical usage
