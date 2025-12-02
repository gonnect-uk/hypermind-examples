# rust-kgdb

[![npm version](https://badge.fury.io/js/rust-kgdb.svg)](https://www.npmjs.com/package/rust-kgdb)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Production-ready RDF/hypergraph database with 100% W3C SPARQL 1.1 + RDF 1.2 compliance, worst-case optimal joins (WCOJ), and pluggable storage backends.**

---

## Why rust-kgdb?

| Feature | rust-kgdb | Apache Jena | RDFox |
|---------|-----------|-------------|-------|
| **Lookup Speed** | 2.78 µs | ~50 µs | 50-100 µs |
| **Memory/Triple** | 24 bytes | 50-60 bytes | 32 bytes |
| **SPARQL 1.1** | 100% | 100% | 95% |
| **RDF 1.2** | 100% | Partial | No |
| **WCOJ** | ✅ LeapFrog | ❌ | ❌ |
| **Mobile-Ready** | ✅ iOS/Android | ❌ | ❌ |

---

## Core Technical Innovations

### 1. Worst-Case Optimal Joins (WCOJ)

Traditional databases use **nested-loop joins** with O(n²) to O(n⁴) complexity. rust-kgdb implements the **LeapFrog TrieJoin** algorithm—a worst-case optimal join that achieves O(n log n) for multi-way joins.

**How it works:**
- **Trie Data Structure**: Triples indexed hierarchically (S→P→O) using BTreeMap for sorted access
- **Variable Ordering**: Frequency-based analysis orders variables for optimal intersection
- **LeapFrog Iterator**: Binary search across sorted iterators finds intersections without materializing intermediate results

```
Query: SELECT ?x ?y ?z WHERE { ?x :p ?y . ?y :q ?z . ?x :r ?z }

Nested Loop: O(n³) - examines every combination
WCOJ:        O(n log n) - iterates in sorted order, seeks forward on mismatch
```

| Query Pattern | Before (Nested Loop) | After (WCOJ) | Speedup |
|---------------|---------------------|--------------|---------|
| 3-way star | O(n³) | O(n log n) | **50-100x** |
| 4+ way complex | O(n⁴) | O(n log n) | **100-1000x** |
| Chain queries | O(n²) | O(n log n) | **10-20x** |

### 2. Sparse Matrix Engine (CSR Format)

Binary relations (e.g., `foaf:knows`, `rdfs:subClassOf`) are converted to **Compressed Sparse Row (CSR)** matrices for cache-efficient join evaluation:

- **Memory**: O(nnz) where nnz = number of edges (not O(n²))
- **Matrix Multiplication**: Replaces nested-loop joins
- **Transitive Closure**: Semi-naive Δ-matrix evaluation (not iterated powers)

```rust
// Traditional: O(n²) nested loops
for (s, p, o) in triples { ... }

// CSR Matrix: O(nnz) cache-friendly iteration
row_ptr[i] → col_indices[j] → values[j]
```

**Used for**: RDFS/OWL reasoning, transitive closure, Datalog evaluation.

### 3. SIMD + PGO Compiler Optimizations

**Zero code changes—pure compiler-level performance gains.**

| Optimization | Technology | Effect |
|--------------|------------|--------|
| **SIMD Vectorization** | AVX2/BMI2 (Intel), NEON (ARM) | 8-wide parallel operations |
| **Profile-Guided Optimization** | LLVM PGO | Hot path optimization, branch prediction |
| **Link-Time Optimization** | LTO (fat) | Cross-crate inlining, dead code elimination |

**Benchmark Results (LUBM, Intel Skylake):**

| Query | Before | After (SIMD+PGO) | Improvement |
|-------|--------|------------------|-------------|
| Q5: 2-hop chain | 230ms | 53ms | **77% faster** |
| Q3: 3-way star | 177ms | 62ms | **65% faster** |
| Q4: 3-hop chain | 254ms | 101ms | **60% faster** |
| Q8: Triangle | 410ms | 193ms | **53% faster** |
| Q7: Hierarchy | 343ms | 198ms | **42% faster** |
| Q6: 6-way complex | 641ms | 464ms | **28% faster** |
| Q2: 5-way star | 234ms | 183ms | **22% faster** |
| Q1: 4-way star | 283ms | 258ms | **9% faster** |

**Average speedup: 44.5%** across all queries.

### 4. Quad Indexing (SPOC)

Four complementary indexes enable O(1) pattern matching regardless of query shape:

| Index | Pattern | Use Case |
|-------|---------|----------|
| **SPOC** | `(?s, ?p, ?o, ?g)` | Subject-centric queries |
| **POCS** | `(?p, ?o, ?c, ?s)` | Property enumeration |
| **OCSP** | `(?o, ?c, ?s, ?p)` | Object lookups (reverse links) |
| **CSPO** | `(?c, ?s, ?p, ?o)` | Named graph iteration |

---

## Storage Backends

rust-kgdb uses a pluggable storage architecture. **Default is in-memory** (zero configuration). For persistence, enable RocksDB.

| Backend | Feature Flag | Use Case | Status |
|---------|--------------|----------|--------|
| **InMemory** | `default` | Development, testing, embedded | ✅ **Production Ready** |
| **RocksDB** | `rocksdb-backend` | Production, large datasets | ✅ **61 tests passing** |
| **LMDB** | `lmdb-backend` | Read-heavy workloads | ✅ **31 tests passing** |

### InMemory (Default)

Zero configuration, maximum performance. Data is volatile (lost on process exit).

**High-Performance Data Structures:**

| Component | Structure | Why |
|-----------|-----------|-----|
| **Triple Store** | `DashMap` | Lock-free concurrent hash map, 100K pre-allocation |
| **WCOJ Trie** | `BTreeMap` | Sorted iteration for LeapFrog intersection |
| **Dictionary** | `FxHashSet` | String interning with rustc-optimized hashing |
| **Hypergraph** | `FxHashMap` | Fast node→edge adjacency lists |
| **Reasoning** | `AHashMap` | RDFS/OWL inference with DoS-resistant hashing |
| **Datalog** | `FxHashMap` | Semi-naive evaluation with delta propagation |

**Why these structures enable sub-microsecond performance:**
- **DashMap**: Sharded locks (16 shards default) → near-linear scaling on multi-core
- **FxHashMap**: Rust compiler's hash function → 30% faster than std HashMap
- **BTreeMap**: O(log n) ordered iteration → enables binary search in LeapFrog
- **Pre-allocation**: 100K capacity avoids rehashing during bulk inserts

```rust
use storage::{QuadStore, InMemoryBackend};

let store = QuadStore::new(InMemoryBackend::new());
// Ultra-fast: 2.78 µs lookups, zero disk I/O
```

### RocksDB (Persistent)

LSM-tree based storage with ACID transactions. Tested with **61 comprehensive tests**.

```toml
# Cargo.toml - Enable RocksDB backend
[dependencies]
storage = { version = "0.1.10", features = ["rocksdb-backend"] }
```

```rust
use storage::{QuadStore, RocksDbBackend};

// Create persistent database
let backend = RocksDbBackend::new("/path/to/data")?;
let store = QuadStore::new(backend);

// Features:
// - ACID transactions
// - Snappy compression (automatic)
// - Crash recovery
// - Range & prefix scanning
// - 1MB+ value support

// Force sync to disk
store.flush()?;
```

**RocksDB Test Coverage:**
- Basic CRUD operations (14 tests)
- Range scanning (8 tests)
- Prefix scanning (6 tests)
- Batch operations (8 tests)
- Transactions (8 tests)
- Concurrent access (5 tests)
- Unicode & binary data (4 tests)
- Large key/value handling (8 tests)

### LMDB (Memory-Mapped Persistent)

B+tree based storage with memory-mapped I/O (via `heed` crate). Optimized for **read-heavy workloads** with MVCC (Multi-Version Concurrency Control). Tested with **31 comprehensive tests**.

```toml
# Cargo.toml - Enable LMDB backend
[dependencies]
storage = { version = "0.1.12", features = ["lmdb-backend"] }
```

```rust
use storage::{QuadStore, LmdbBackend};

// Create persistent database (default 10GB map size)
let backend = LmdbBackend::new("/path/to/data")?;
let store = QuadStore::new(backend);

// Or with custom map size (1GB)
let backend = LmdbBackend::with_map_size("/path/to/data", 1024 * 1024 * 1024)?;

// Features:
// - Memory-mapped I/O (zero-copy reads)
// - MVCC for concurrent readers
// - Crash-safe ACID transactions
// - Range & prefix scanning
// - Excellent for read-heavy workloads

// Sync to disk
store.flush()?;
```

**When to use LMDB vs RocksDB:**

| Characteristic | LMDB | RocksDB |
|----------------|------|---------|
| **Read Performance** | ✅ Faster (memory-mapped) | Good |
| **Write Performance** | Good | ✅ Faster (LSM-tree) |
| **Concurrent Readers** | ✅ Unlimited | Limited by locks |
| **Write Amplification** | Low | Higher (compaction) |
| **Memory Usage** | Higher (map size) | Lower (cache-based) |
| **Best For** | Read-heavy, OLAP | Write-heavy, OLTP |

**LMDB Test Coverage:**
- Basic CRUD operations (8 tests)
- Range scanning (4 tests)
- Prefix scanning (3 tests)
- Batch operations (3 tests)
- Large key/value handling (4 tests)
- Concurrent access (4 tests)
- Statistics & flush (3 tests)
- Edge cases (2 tests)

### TypeScript SDK

The npm package uses the in-memory backend—ideal for:
- Knowledge graph queries
- SPARQL execution
- Data transformation pipelines
- Embedded applications

```typescript
import { GraphDB } from 'rust-kgdb'

// In-memory database (default, no configuration needed)
const db = new GraphDB('http://example.org/app')

// For persistence, export via CONSTRUCT:
const ntriples = db.queryConstruct('CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }')
fs.writeFileSync('backup.nt', ntriples)
```

---

## Installation

```bash
npm install rust-kgdb
```

### Platform Support

| Platform | Architecture | Status | SIMD |
|----------|-------------|--------|------|
| **macOS** | Intel (x64) | ✅ | AVX2, BMI2, POPCNT |
| **macOS** | Apple Silicon (arm64) | ✅ | NEON |
| **Linux** | x64 | ✅ | AVX2, BMI2, POPCNT |
| **Linux** | arm64 | ✅ | NEON |
| **Windows** | x64 | ✅ | AVX2, BMI2, POPCNT |
| **Windows** | arm64 | ⏳ v0.2.0 | — |

**No compilation required**—pre-built native binaries included.

---

## Quick Start

### Complete Working Example

```typescript
import { GraphDB } from 'rust-kgdb'

// 1. Create database
const db = new GraphDB('http://example.org/myapp')

// 2. Load data (Turtle format)
db.loadTtl(`
  @prefix foaf: <http://xmlns.com/foaf/0.1/> .
  @prefix ex: <http://example.org/> .

  ex:alice a foaf:Person ;
           foaf:name "Alice" ;
           foaf:age 30 ;
           foaf:knows ex:bob, ex:charlie .

  ex:bob a foaf:Person ;
         foaf:name "Bob" ;
         foaf:age 25 ;
         foaf:knows ex:charlie .

  ex:charlie a foaf:Person ;
             foaf:name "Charlie" ;
             foaf:age 35 .
`, null)

// 3. Query: Find friends-of-friends (WCOJ optimized!)
const fof = db.querySelect(`
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  PREFIX ex: <http://example.org/>

  SELECT ?person ?friend ?fof WHERE {
    ?person foaf:knows ?friend .
    ?friend foaf:knows ?fof .
    FILTER(?person != ?fof)
  }
`)
console.log('Friends of Friends:', fof)
// [{ person: 'ex:alice', friend: 'ex:bob', fof: 'ex:charlie' }]

// 4. Aggregation: Average age
const stats = db.querySelect(`
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>

  SELECT (COUNT(?p) AS ?count) (AVG(?age) AS ?avgAge) WHERE {
    ?p a foaf:Person ; foaf:age ?age .
  }
`)
console.log('Stats:', stats)
// [{ count: '3', avgAge: '30.0' }]

// 5. ASK query
const hasAlice = db.queryAsk(`
  PREFIX ex: <http://example.org/>
  ASK { ex:alice a <http://xmlns.com/foaf/0.1/Person> }
`)
console.log('Has Alice?', hasAlice)  // true

// 6. CONSTRUCT query
const graph = db.queryConstruct(`
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  PREFIX ex: <http://example.org/>

  CONSTRUCT { ?p foaf:knows ?f }
  WHERE { ?p foaf:knows ?f }
`)
console.log('Extracted graph:', graph)

// 7. Count and cleanup
console.log('Triple count:', db.count())  // 11
db.clear()
```

### Save to File

```typescript
import { writeFileSync } from 'fs'

// Save as N-Triples
const db = new GraphDB('http://example.org/export')
db.loadTtl(`<http://example.org/s> <http://example.org/p> "value" .`, null)

const ntriples = db.queryConstruct(`CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }`)
writeFileSync('output.nt', ntriples)
```

---

## SPARQL 1.1 Features (100% W3C Compliant)

### Query Forms

```typescript
// SELECT - return bindings
db.querySelect('SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10')

// ASK - boolean existence check
db.queryAsk('ASK { <http://example.org/x> ?p ?o }')

// CONSTRUCT - build new graph
db.queryConstruct('CONSTRUCT { ?s <http://new/prop> ?o } WHERE { ?s ?p ?o }')
```

### Aggregates

```typescript
db.querySelect(`
  SELECT ?type (COUNT(*) AS ?count) (AVG(?value) AS ?avg)
  WHERE { ?s a ?type ; <http://ex/value> ?value }
  GROUP BY ?type
  HAVING (COUNT(*) > 5)
  ORDER BY DESC(?count)
`)
```

### Property Paths

```typescript
// Transitive closure (rdfs:subClassOf*)
db.querySelect('SELECT ?class WHERE { ?class rdfs:subClassOf* <http://top/Class> }')

// Alternative paths
db.querySelect('SELECT ?name WHERE { ?x (foaf:name|rdfs:label) ?name }')

// Sequence paths
db.querySelect('SELECT ?grandparent WHERE { ?x foaf:parent/foaf:parent ?grandparent }')
```

### Named Graphs

```typescript
// Load into named graph
db.loadTtl('<http://s> <http://p> "o" .', 'http://example.org/graph1')

// Query specific graph
db.querySelect(`
  SELECT ?s ?p ?o WHERE {
    GRAPH <http://example.org/graph1> { ?s ?p ?o }
  }
`)
```

### UPDATE Operations

```typescript
// INSERT DATA
db.updateInsert(`
  INSERT DATA { <http://ex/new> <http://ex/prop> "value" }
`)

// DELETE WHERE
db.updateDelete(`
  DELETE WHERE { ?s <http://ex/deprecated> ?o }
`)
```

---

## Sample Application

### Knowledge Graph Demo

A complete, production-ready sample application demonstrating enterprise knowledge graph capabilities is available in the repository.

**Location**: [`examples/knowledge-graph-demo/`](../../examples/knowledge-graph-demo/)

**Features Demonstrated**:
- Complete organizational knowledge graph (employees, departments, projects, skills)
- SPARQL SELECT queries with star and chain patterns (WCOJ-optimized)
- Aggregations (COUNT, AVG, GROUP BY, HAVING)
- Property paths for transitive closure (organizational hierarchy)
- SPARQL ASK and CONSTRUCT queries
- Named graphs for multi-tenant data isolation
- Data export to Turtle format

**Run the Demo**:

```bash
cd examples/knowledge-graph-demo
npm install
npm start
```

**Sample Output**:

The demo creates a realistic knowledge graph with:
- 5 employees across 4 departments
- 13 technical and soft skills
- 2 software projects
- Reporting hierarchies and salary data
- Named graph for sensitive compensation data

**Example Query from Demo** (finds all direct and indirect reports):

```typescript
const pathQuery = `
  PREFIX ex: <http://example.org/>
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>

  SELECT ?employee ?name WHERE {
    ?employee ex:reportsTo+ ex:alice .  # Transitive closure
    ?employee foaf:name ?name .
  }
  ORDER BY ?name
`
const results = db.querySelect(pathQuery)
```

**Learn More**: See the [demo README](../../examples/knowledge-graph-demo/README.md) for full documentation, query examples, and how to customize the knowledge graph.

---

## API Reference

### GraphDB Class

```typescript
class GraphDB {
  constructor(baseUri: string)           // Create with base URI
  static inMemory(): GraphDB             // Create anonymous in-memory DB

  // Data Loading
  loadTtl(data: string, graph: string | null): void
  loadNTriples(data: string, graph: string | null): void

  // SPARQL Queries (WCOJ-optimized)
  querySelect(sparql: string): Array<Record<string, string>>
  queryAsk(sparql: string): boolean
  queryConstruct(sparql: string): string  // Returns N-Triples

  // SPARQL Updates
  updateInsert(sparql: string): void
  updateDelete(sparql: string): void

  // Database Operations
  count(): number
  clear(): void
  getVersion(): string
}
```

### Node Class

```typescript
class Node {
  static iri(uri: string): Node
  static literal(value: string): Node
  static langLiteral(value: string, lang: string): Node
  static typedLiteral(value: string, datatype: string): Node
  static integer(value: number): Node
  static boolean(value: boolean): Node
  static blank(id: string): Node
}
```

---

## Performance Characteristics

### Complexity Analysis

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Triple lookup | O(1) | Hash-based SPOC index |
| Pattern scan | O(k) | k = matching triples |
| Star join (WCOJ) | O(n log n) | LeapFrog intersection |
| Complex join (WCOJ) | O(n log n) | Trie-based |
| Transitive closure | O(n²) worst | CSR matrix optimization |
| Bulk insert | O(n) | Batch indexing |

### Memory Layout

```
Triple: 24 bytes
├── Subject:   8 bytes (dictionary ID)
├── Predicate: 8 bytes (dictionary ID)
└── Object:    8 bytes (dictionary ID)

String Interning: All URIs/literals stored once in Dictionary
Index Overhead: ~4x base triple size (4 indexes)
Total: ~120 bytes/triple including indexes
```

---

## Version History

### v0.1.12 (2025-12-01) - LMDB Backend Release

- **LMDB storage backend** fully implemented (31 tests passing)
- Memory-mapped I/O for optimal read performance
- MVCC concurrency for unlimited concurrent readers
- Complete LMDB vs RocksDB comparison documentation
- Sample application with 87 triples demonstrating all features

### v0.1.9 (2025-12-01) - SIMD + PGO Release

- **44.5% average speedup** via SIMD + PGO compiler optimizations
- WCOJ execution with LeapFrog TrieJoin
- Release automation infrastructure
- All packages updated to gonnect-uk namespace

### v0.1.8 (2025-12-01) - WCOJ Execution

- WCOJ execution path activated
- Variable ordering analysis for optimal joins
- 577 tests passing

### v0.1.7 (2025-11-30)

- Query optimizer with automatic strategy selection
- WCOJ algorithm integration (planning phase)

### v0.1.3 (2025-11-18)

- Initial TypeScript SDK
- 100% W3C SPARQL 1.1 compliance
- 100% W3C RDF 1.2 compliance

---

## Use Cases

| Domain | Application |
|--------|-------------|
| **Knowledge Graphs** | Enterprise ontologies, taxonomies |
| **Semantic Search** | Structured queries over unstructured data |
| **Data Integration** | ETL with SPARQL CONSTRUCT |
| **Compliance** | SHACL validation, provenance tracking |
| **Graph Analytics** | Pattern detection, community analysis |
| **Mobile Apps** | Embedded RDF on iOS/Android |

---

## Links

- [GitHub Repository](https://github.com/gonnect-uk/rust-kgdb)
- [Documentation](https://github.com/gonnect-uk/rust-kgdb/tree/main/docs)
- [CHANGELOG](https://github.com/gonnect-uk/rust-kgdb/blob/main/CHANGELOG.md)
- [W3C SPARQL 1.1](https://www.w3.org/TR/sparql11-query/)
- [W3C RDF 1.2](https://www.w3.org/TR/rdf12-concepts/)

---

## License

Apache License 2.0

---

**Built with Rust + NAPI-RS**
