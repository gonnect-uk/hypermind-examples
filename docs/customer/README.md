# Customer Documentation

**Professional documentation for rust-kgdb users and evaluators.**

---

## Quick Navigation

### 🚀 [Getting Started](getting-started/)
- **[QUICKSTART.md](getting-started/QUICKSTART.md)** - 5-minute guide to first query
- **[INSTALLATION.md](getting-started/INSTALLATION.md)** - Platform-specific installation
- **[FIRST_QUERY.md](getting-started/FIRST_QUERY.md)** - SPARQL query examples

### 🏗️ [Architecture](architecture/)
- **[OVERVIEW.md](architecture/OVERVIEW.md)** - System design and components
- **[STORAGE_DESIGN.md](architecture/STORAGE_DESIGN.md)** - Pluggable backend architecture
- **[SPARQL_ENGINE.md](architecture/SPARQL_ENGINE.md)** - Query execution model
- **[HYPERGRAPH_MODEL.md](architecture/HYPERGRAPH_MODEL.md)** - Beyond RDF triples

### ⚡ [Performance](performance/)
- **[BENCHMARKS.md](performance/BENCHMARKS.md)** - Real measurements vs competitors
- **[vs_COMPETITORS.md](performance/vs_COMPETITORS.md)** - Feature & performance comparison
- **[OPTIMIZATION_GUIDE.md](performance/OPTIMIZATION_GUIDE.md)** - Tuning for production

### ✅ [W3C Compliance](w3c-compliance/)
- **[SPARQL_1.1.md](w3c-compliance/SPARQL_1.1.md)** - Complete SPARQL 1.1 support
- **[RDF_1.2.md](w3c-compliance/RDF_1.2.md)** - RDF 1.2 features (RDF-star, Turtle 1.2)
- **[CERTIFICATION.md](w3c-compliance/CERTIFICATION.md)** - Conformance test results

---

## Key Facts

| Metric | Value | vs Competition |
|--------|-------|----------------|
| **Lookup Speed** | 2.78 µs | 35-180x faster than RDFox |
| **Memory Efficiency** | 24 bytes/triple | 25% better than RDFox |
| **Bulk Insert** | 146K triples/sec | 73% of RDFox (optimizing) |
| **SPARQL Functions** | 64 builtin | More than Jena/RDFox |
| **Mobile Support** | iOS + Android | ONLY triple store |
| **W3C Compliance** | 100% | SPARQL 1.1 + RDF 1.2 |

---

## Production-Ready Features

✅ **Complete SPARQL 1.1** - Query + Update operations
✅ **Zero-Copy Architecture** - No GC, predictable performance
✅ **Pluggable Storage** - InMemory, RocksDB, LMDB
✅ **Native Hypergraphs** - N-ary relationships beyond triples
✅ **Mobile-First** - <100ms cold start, <20MB for 100K triples
✅ **Memory Safe** - Rust guarantees, no segfaults
✅ **521 Tests Passing** - 100% success rate

---

## Use Cases

**Mobile Applications:**
- Knowledge graphs on iOS/Android without JVM overhead
- Offline semantic reasoning with <100ms startup
- <20MB memory footprint for 100K triples

**Enterprise Systems:**
- Sub-millisecond query latency for real-time applications
- Pluggable persistence (RocksDB/LMDB) for ACID guarantees
- Production-grade software craftsmanship

**Research & Academia:**
- Complete W3C compliance for reproducible research
- Hypergraph native model for advanced reasoning
- Comparable performance to RDFox, better than Jena

---

## Support

- **Issues**: [GitHub Issues](https://github.com/gonnect/rust-kgdb/issues)
- **Documentation**: This directory
- **Contributing**: See [developer/contributing/](../developer/contributing/)
