# rust-kgdb SDK Architecture

## Overview

**rust-kgdb** provides professional SDKs for 4 target ecosystems, all sharing the same high-performance Rust core:

1. **Python SDK** - UniFFI bindings for data science and ML
2. **Kotlin/Java SDK** - UniFFI bindings for JVM ecosystem
3. **TypeScript/Node.js SDK** - NAPI-RS bindings for web/server
4. **Native Rust SDK** - Ergonomic high-level API for Rust projects

**Design Principle**: All SDKs share the same core FFI layer (`crates/mobile-ffi`) with language-specific ergonomic wrappers.

---

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│         Language-Specific SDKs (Ergonomic APIs)             │
│  Python | Kotlin/Java | TypeScript/Node.js | Rust           │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│              FFI Bindings Layer (Auto-Generated)            │
│      UniFFI (Python/Kotlin/Swift) | NAPI-RS (Node.js)      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                  mobile-ffi Core (Rust)                     │
│     GraphDB | Query Execution | Error Handling              │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Core Engine (Rust)                       │
│  sparql | storage | rdf-model | reasoning | hypergraph     │
└─────────────────────────────────────────────────────────────┘
```

---

## 1. Python SDK

### Target Use Cases
- Data science and ML pipelines
- Jupyter notebooks for semantic analysis
- Backend services with semantic reasoning
- Integration with pandas, numpy, scikit-learn

### Technology Stack
- **FFI Layer**: UniFFI (auto-generates Python bindings)
- **Package Manager**: PyPI (pip)
- **Testing**: pytest with fixtures
- **Type Hints**: Full Python 3.8+ type annotations

### API Design

```python
from rust_kgdb import GraphDB, QueryResult

# Create database
db = GraphDB.new_in_memory()

# Insert triples
db.insert_triple(
    subject="http://example.org/alice",
    predicate="http://example.org/knows",
    object="http://example.org/bob"
)

# SPARQL query
results = db.query("""
    SELECT ?person ?name WHERE {
        ?person <http://xmlns.com/foaf/0.1/name> ?name .
    }
""")

# Iterate results
for binding in results:
    person = binding.get("person")
    name = binding.get("name")
    print(f"{person}: {name}")

# Close (auto-cleanup via __del__)
db.close()
```

### Package Structure

```
python-sdk/
├── rust_kgdb/
│   ├── __init__.py           # Public API
│   ├── _rust_kgdb.pyi        # Type stubs
│   ├── core.py               # Ergonomic wrappers
│   └── exceptions.py         # Custom exceptions
├── tests/
│   ├── test_basic.py
│   ├── test_sparql.py
│   ├── test_transactions.py
│   └── fixtures.py
├── examples/
│   ├── quickstart.py
│   ├── pandas_integration.py
│   └── reasoning_example.py
├── setup.py                  # Build configuration
├── pyproject.toml            # Modern Python packaging
└── README.md
```

---

## 2. Kotlin/Java SDK

### Target Use Cases
- Enterprise JVM applications
- Android apps (via mobile-ffi)
- Spring Boot microservices
- Kafka/Spark integration

### Technology Stack
- **FFI Layer**: UniFFI (auto-generates Kotlin bindings)
- **Build System**: Gradle/Maven
- **Testing**: JUnit 5 + Kotest
- **Package Manager**: Maven Central

### API Design (Kotlin)

```kotlin
import com.zenya.graphdb.GraphDB
import com.zenya.graphdb.Node

// Create database
val db = GraphDB.newInMemory()

// Insert triples
db.insertTriple(
    subject = Node.iri("http://example.org/alice"),
    predicate = Node.iri("http://xmlns.com/foaf/0.1/name"),
    obj = Node.literal("Alice")
)

// SPARQL query
val results = db.query("""
    SELECT ?person ?name WHERE {
        ?person <http://xmlns.com/foaf/0.1/name> ?name .
    }
""")

// Process results
results.forEach { binding ->
    val person = binding["person"]
    val name = binding["name"]
    println("$person: $name")
}

// Close
db.close()
```

### API Design (Java)

```java
import com.zenya.graphdb.GraphDB;
import com.zenya.graphdb.Node;
import com.zenya.graphdb.QueryResult;

// Create database
GraphDB db = GraphDB.newInMemory();

// Insert triples
db.insertTriple(
    Node.iri("http://example.org/alice"),
    Node.iri("http://xmlns.com/foaf/0.1/name"),
    Node.literal("Alice")
);

// SPARQL query
QueryResult results = db.query("""
    SELECT ?person ?name WHERE {
        ?person <http://xmlns.com/foaf/0.1/name> ?name .
    }
""");

// Process results
results.forEach(binding -> {
    Node person = binding.get("person");
    Node name = binding.get("name");
    System.out.println(person + ": " + name);
});

// Close
db.close();
```

### Package Structure

```
kotlin-sdk/
├── src/
│   ├── main/
│   │   ├── kotlin/com/zenya/graphdb/
│   │   │   ├── GraphDB.kt       # Main API
│   │   │   ├── Node.kt          # RDF nodes
│   │   │   ├── QueryResult.kt   # Results
│   │   │   └── Exceptions.kt    # Error types
│   │   └── resources/
│   │       └── META-INF/
│   └── test/
│       ├── kotlin/
│       │   ├── BasicTests.kt
│       │   ├── SparqlTests.kt
│       │   └── TransactionTests.kt
│       └── resources/
│           └── test-data/
├── build.gradle.kts
├── pom.xml                  # Maven alternative
└── README.md
```

---

## 3. TypeScript/Node.js SDK

### Target Use Cases
- Web applications (Express, Nest.js)
- Serverless functions (AWS Lambda, Vercel)
- Electron desktop apps
- Real-time semantic APIs

### Technology Stack
- **FFI Layer**: NAPI-RS (fast native Node.js bindings)
- **Package Manager**: npm/yarn/pnpm
- **Testing**: Jest with TypeScript
- **Build**: TypeScript compiler + Rollup

### API Design

```typescript
import { GraphDB, Node, QueryResult } from '@zenya/graphdb';

// Create database
const db = GraphDB.newInMemory();

// Insert triples
db.insertTriple(
  Node.iri('http://example.org/alice'),
  Node.iri('http://xmlns.com/foaf/0.1/name'),
  Node.literal('Alice')
);

// SPARQL query with type safety
const results: QueryResult = await db.query(`
  SELECT ?person ?name WHERE {
    ?person <http://xmlns.com/foaf/0.1/name> ?name .
  }
`);

// Process results
for (const binding of results) {
  const person = binding.get('person');
  const name = binding.get('name');
  console.log(`${person}: ${name}`);
}

// Auto-cleanup via garbage collection
```

### Package Structure

```
typescript-sdk/
├── src/
│   ├── index.ts              # Main exports
│   ├── graphdb.ts            # GraphDB wrapper
│   ├── node.ts               # RDF node types
│   ├── query.ts              # Query builders
│   └── errors.ts             # Error classes
├── native/
│   ├── Cargo.toml            # NAPI-RS build
│   └── src/
│       └── lib.rs            # Rust bindings
├── tests/
│   ├── basic.test.ts
│   ├── sparql.test.ts
│   └── performance.test.ts
├── examples/
│   ├── quickstart.ts
│   ├── express-server.ts
│   └── reasoning.ts
├── package.json
├── tsconfig.json
└── README.md
```

---

## 4. Native Rust SDK

### Target Use Cases
- High-performance Rust applications
- Embedded systems
- CLI tools
- Library integration in Rust projects

### Technology Stack
- **Build**: Cargo (standard Rust)
- **Testing**: Rust's built-in test framework
- **Docs**: cargo doc with examples
- **Package**: crates.io

### API Design

```rust
use rust_kgdb::{GraphDB, Node, SparqlQuery};

// Create database
let mut db = GraphDB::in_memory();

// Insert triples (builder pattern)
db.insert()
    .triple(
        Node::iri("http://example.org/alice"),
        Node::iri("http://xmlns.com/foaf/0.1/name"),
        Node::literal("Alice"),
    )
    .triple(
        Node::iri("http://example.org/alice"),
        Node::iri("http://xmlns.com/foaf/0.1/knows"),
        Node::iri("http://example.org/bob"),
    )
    .execute()?;

// SPARQL query with type-safe builder
let results = db.query()
    .select(&["?person", "?name"])
    .where_pattern("?person <http://xmlns.com/foaf/0.1/name> ?name")
    .execute()?;

// Process results with iterators
for binding in results {
    let person = binding.get("person")?;
    let name = binding.get("name")?;
    println!("{}: {}", person, name);
}
```

### Crate Structure

```
crates/sdk/
├── src/
│   ├── lib.rs                # Public API
│   ├── graphdb.rs            # High-level database
│   ├── query_builder.rs      # Fluent query API
│   ├── node.rs               # RDF node wrappers
│   ├── transaction.rs        # Transaction API
│   └── error.rs              # Error types
├── tests/
│   ├── basic_operations.rs
│   ├── sparql_queries.rs
│   ├── transactions.rs
│   └── performance.rs
├── examples/
│   ├── quickstart.rs
│   ├── reasoning.rs
│   └── advanced.rs
├── benches/
│   └── sdk_benchmarks.rs
└── Cargo.toml
```

---

## Shared FFI Core (mobile-ffi)

### Responsibilities
1. **Expose core functionality** via C ABI
2. **Memory management** (no leaks across FFI boundary)
3. **Error handling** (convert Rust errors to FFI errors)
4. **Thread safety** (Arc/Mutex for shared state)

### Current FFI Functions (mobile-ffi)

```rust
// Database lifecycle
pub fn graphdb_new_in_memory() -> Result<Arc<GraphDB>>;
pub fn graphdb_close(db: Arc<GraphDB>);

// Triple operations
pub fn insert_triple(db: &GraphDB, s: String, p: String, o: String) -> Result<()>;
pub fn query(db: &GraphDB, sparql: String) -> Result<Vec<Binding>>;

// Transactions
pub fn begin_transaction(db: &GraphDB) -> Result<Transaction>;
pub fn commit(tx: Transaction) -> Result<()>;
pub fn rollback(tx: Transaction);
```

---

## Testing Strategy

### 1. Unit Tests (Per SDK)
- **Coverage**: Every public API method
- **Scope**: Input validation, error handling, basic operations
- **Tools**: Language-specific test frameworks

### 2. Integration Tests
- **Coverage**: Multi-step workflows, transactions, reasoning
- **Scope**: Real-world usage patterns
- **Data**: Shared test datasets across all SDKs

### 3. Regression Tests
- **Coverage**: All SPARQL 1.1 features (119 features)
- **Scope**: Ensure SDKs expose full engine capabilities
- **Baseline**: Core engine tests (1058 tests)

### 4. Performance Tests
- **Coverage**: Benchmark critical operations
- **Scope**: Overhead of FFI boundary
- **Metrics**: Throughput, latency, memory usage

### 5. SME Tests (Semantic Correctness)
- **Coverage**: W3C SPARQL 1.1 compliance
- **Scope**: Property paths, aggregates, subqueries
- **Validation**: Results match reference implementations

---

## Build & Distribution

### Python
```bash
# Build wheel
python setup.py bdist_wheel

# Publish to PyPI
twine upload dist/*
```

### Kotlin/Java
```bash
# Build JAR
./gradlew build

# Publish to Maven Central
./gradlew publishToMavenCentral
```

### TypeScript
```bash
# Build native bindings + TS
npm run build

# Publish to npm
npm publish
```

### Rust
```bash
# Publish to crates.io
cargo publish -p rust-kgdb-sdk
```

---

## Version Compatibility Matrix

| SDK Version | Core Version | UniFFI | NAPI-RS | Rust |
|-------------|--------------|--------|---------|------|
| Python 0.1.x | 0.1.2+ | 0.30.0 | - | 1.83+ |
| Kotlin 0.1.x | 0.1.2+ | 0.30.0 | - | 1.83+ |
| TypeScript 0.1.x | 0.1.2+ | - | 2.x | 1.83+ |
| Rust SDK 0.1.x | 0.1.2+ | - | - | 1.83+ |

---

## Error Handling

### Unified Error Model

All SDKs expose the same error hierarchy:

```
GraphDBError
├── QueryError
│   ├── ParseError
│   ├── ExecutionError
│   └── TimeoutError
├── StorageError
│   ├── IOError
│   └── CorruptionError
├── FFIError
│   ├── NullPointerError
│   └── TypeMismatchError
└── ValidationError
    ├── InvalidIRIError
    └── InvalidLiteralError
```

---

## Documentation Standards

### Each SDK Must Include:

1. **README.md**
   - Quick start (5-minute setup)
   - Installation instructions
   - Basic examples
   - Links to full docs

2. **API Reference**
   - Auto-generated from code
   - Examples for every public method
   - Type signatures

3. **User Guide**
   - Tutorials
   - Best practices
   - Performance tuning
   - Migration guides

4. **Examples Directory**
   - Quickstart
   - Real-world use cases
   - Integration patterns

---

## Release Process

### For Each SDK Release:

1. ✅ Update version in manifest (Cargo.toml, package.json, etc.)
2. ✅ Run full test suite (unit + integration + regression)
3. ✅ Update CHANGELOG.md with changes
4. ✅ Build release artifacts
5. ✅ Run performance benchmarks
6. ✅ Update documentation
7. ✅ Create git tag (e.g., `python-sdk-v0.1.0`)
8. ✅ Publish to package registry
9. ✅ Announce release

---

## Next Steps

1. **Implement Python SDK** with comprehensive tests
2. **Implement Kotlin/Java SDK** with JUnit tests
3. **Implement TypeScript SDK** with Jest tests
4. **Implement Rust SDK** with ergonomic API
5. **Create unified test suite** across all SDKs
6. **Write SME documentation** for each SDK
7. **Run full regression** (1058+ tests across all SDKs)

---

**Generated**: 2025-11-28
**Version**: rust-kgdb v0.1.2
**Target**: Professional multi-language SDK ecosystem
