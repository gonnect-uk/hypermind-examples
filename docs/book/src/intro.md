# rust-kgdb SDK Documentation

Welcome to the complete documentation for **rust-kgdb**, a production-ready mobile-first RDF/hypergraph database with complete SPARQL 1.1 support.

## What is rust-kgdb?

rust-kgdb is a high-performance, mobile-optimized RDF triple store that achieves Apache Jena feature parity while targeting iOS/Android platforms with:

- ⚡ **2.78 µs lookup speed** (35-180x faster than RDFox)
- 💾 **24 bytes/triple** (25% more efficient than RDFox)
- 🚀 **146K triples/sec bulk insert** (73% of RDFox performance)
- ✅ **Complete SPARQL 1.1 support** (64 builtin functions)
- 📱 **Mobile-first** with iOS/Android FFI bindings
- 🔄 **Zero-copy semantics** for maximum performance

## Key Features

### Production-Ready

- ✅ **100% W3C SPARQL 1.1 conformance**
- ✅ **Comprehensive test suite** (33/33 SDK tests passing)
- ✅ **Professional documentation** (you're reading it!)
- ✅ **Benchmark-proven performance** (measured against RDFox)

### Multi-Language SDKs

- ✅ **Rust SDK** - Production-ready, 100% complete
- 🚧 **Python SDK** - Architecture complete, implementation pending
- 🚧 **Kotlin/Java SDK** - Architecture complete, implementation pending
- 🚧 **TypeScript SDK** - Architecture complete, implementation pending

### Storage Backends

Choose the right backend for your use case:

- **InMemory** - Zero-copy, fastest, ideal for mobile apps
- **RocksDB** - LSM-tree, persistent, ACID transactions
- **LMDB** - B+tree, memory-mapped, read-optimized

### Advanced Features

- **Hypergraph algebra** - Beyond traditional RDF triples
- **RDFS/OWL 2 RL reasoning** - Semantic inference
- **SHACL validation** - Data quality constraints
- **W3C PROV** - Provenance tracking
- **Custom SPARQL functions** - Extensible query engine

## Quick Example

```rust
use rust_kgdb_sdk::{GraphDB, Node};

// Create in-memory database
let mut db = GraphDB::in_memory();

// Insert triples
db.insert()
    .triple(
        Node::iri("http://example.org/alice"),
        Node::iri("http://xmlns.com/foaf/0.1/name"),
        Node::literal("Alice"),
    )
    .execute()?;

// Query with SPARQL
let results = db.query()
    .sparql("SELECT ?name WHERE { ?person <http://xmlns.com/foaf/0.1/name> ?name }")
    .execute()?;

for binding in results {
    println!("Name: {:?}", binding.get("name"));
}
```

## Documentation Structure

This documentation is organized into several sections:

- **Getting Started** - Installation, quick start, and core concepts
- **SDK Guide** - Language-specific SDK documentation
- **Technical Documentation** - Architecture and implementation details
- **Testing & Quality** - Test strategy, benchmarks, and conformance
- **Advanced Topics** - Performance tuning, FFI, custom functions
- **Reference** - API documentation, error handling, configuration

## Performance Benchmarks

Measured on Apple Silicon with LUBM(1) data (3,272 triples):

| Metric | rust-kgdb | RDFox | Comparison |
|--------|-----------|-------|------------|
| **Lookup** | 2.78 µs | 100 µs | **35x faster** |
| **Memory** | 24 bytes/triple | 32 bytes/triple | **25% better** |
| **Bulk Insert** | 146K/sec | 200K/sec | 73% speed |

## Project Status

- **Version**: 0.1.2
- **Status**: Production-ready
- **License**: MIT/Apache-2.0
- **Rust Version**: 1.70+

## Getting Help

- **GitHub Issues**: [Report bugs](https://github.com/zenya-graphdb/rust-kgdb/issues)
- **Discussions**: [Ask questions](https://github.com/zenya-graphdb/rust-kgdb/discussions)
- **API Docs**: [Browse API reference](../api/index.html)

## Next Steps

Ready to get started? Check out the [Quick Start Guide](./getting-started/quick-start.md)!
