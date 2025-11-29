# Testing Strategy

rust-kgdb employs a comprehensive testing strategy across unit, integration, and performance tests.

## Three-Level Testing Approach

### Level 1: Unit Tests
**Location**: Within each crate (`crates/*/tests/`)
**Purpose**: Test individual components in isolation
**Tool**: Built-in Rust test framework
**Speed**: <1 second per test

```rust
#[test]
fn test_dictionary_interning() {
    let dict = Dictionary::new();
    let uri1 = dict.intern("http://example.org/resource");
    let uri2 = dict.intern("http://example.org/resource");

    // Same ID (interned)
    assert_eq!(uri1 as *const str, uri2 as *const str);
}

#[test]
fn test_triple_creation() {
    let subject = Node::IRI("http://example.org/s");
    let predicate = Node::IRI("http://example.org/p");
    let object = Node::IRI("http://example.org/o");

    let triple = Triple::new(subject, predicate, object);

    assert_eq!(triple.subject, subject);
    assert_eq!(triple.predicate, predicate);
    assert_eq!(triple.object, object);
}
```

**Run Unit Tests:**
```bash
cargo test --workspace
cargo test -p storage
cargo test --lib
```

### Level 2: W3C Conformance Tests
**Location**: `crates/sparql/tests/w3c_conformance/`
**Purpose**: Validate SPARQL 1.1 compliance
**Standard**: Official W3C test suite
**Coverage**: 100+ official test cases

```bash
# Clone W3C test data (one-time setup)
git clone https://github.com/w3c/rdf-tests test-data/rdf-tests

# Run conformance tests
cargo test --test w3c_conformance -- --ignored --nocapture
```

**Test Structure:**
```
w3c_conformance/
├── manifest.rs          # Test manifest parser
├── runners.rs           # SPARQL query/update runners
└── test_suite/          # W3C tests (generated)
    ├── algebra/
    ├── basic/
    ├── functions/
    └── syntax/
```

### Level 3: Performance Benchmarks
**Location**: `crates/*/benches/`
**Purpose**: Track performance characteristics
**Tool**: Criterion.rs for statistical analysis
**Frequency**: On every commit

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench --package storage --bench triple_store_benchmark

# Run with baseline comparison
cargo bench --bench triple_store_benchmark -- --baseline main
```

## Test Categories

### RDF Model Tests

**File**: `crates/rdf-model/tests/`

- Node creation and comparison
- Dictionary interning efficiency
- Triple and Quad construction
- URI validation
- Literal type checking

### Storage Backend Tests

**File**: `crates/storage/tests/`

- InMemory backend operations
- RocksDB persistence
- LMDB memory-mapping
- SPOC index correctness
- Transaction semantics
- Concurrent access

### SPARQL Parser Tests

**File**: `crates/sparql/tests/`

- Query parsing (SELECT, CONSTRUCT, ASK)
- Update parsing (INSERT, DELETE)
- Function invocation
- Filter expressions
- Property paths
- Aggregate functions

### SPARQL Executor Tests

**File**: `crates/sparql/tests/`

- Query execution with various patterns
- Join operations (BGPs)
- Filter application
- Grouping and aggregation
- Result binding
- Optional patterns

### RDF IO Tests

**File**: `crates/rdf-io/tests/`

- Turtle parsing (TTL)
- N-Triples parsing (NT)
- RDF/XML parsing
- N-Quads parsing (NQ)
- Prefix handling
- Blank node generation

### Reasoning Tests

**File**: `crates/reasoning/tests/`

- RDFS rule application
- OWL 2 RL inferences
- Subclass transitivity
- Property inheritance
- Domain/range constraints

## Benchmark Data

### LUBM (Lehigh University Benchmark)

Test data generation:

```bash
# Compile LUBM generator
rustc tools/lubm_generator.rs -O -o tools/lubm_generator

# Generate test datasets
./tools/lubm_generator 1 /tmp/lubm_1.nt       # 3,272 triples
./tools/lubm_generator 10 /tmp/lubm_10.nt     # ~32K triples
./tools/lubm_generator 100 /tmp/lubm_100.nt   # ~327K triples
```

### SP2Bench

Semantic Publishing and Publishing Benchmark:

```bash
cargo bench --package storage --bench sp2bench_benchmark
```

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Lookup | <3 µs | 2.78 µs | ✅ Pass |
| BGP (3 triples) | <100 µs | <100 µs | ✅ Pass |
| Bulk Insert | 150K/sec | 146K/sec | ✅ Near |
| Memory per Triple | <25 bytes | 24 bytes | ✅ Pass |

## Test Organization

```
crates/
├── rdf-model/tests/
│   ├── node_tests.rs
│   ├── dictionary_tests.rs
│   └── triple_tests.rs
├── storage/tests/
│   ├── inmemory_tests.rs
│   ├── rocksdb_tests.rs
│   ├── lmdb_tests.rs
│   └── index_tests.rs
├── sparql/tests/
│   ├── parser_tests.rs
│   ├── executor_tests.rs
│   ├── function_tests.rs
│   └── w3c_conformance/
└── rdf-io/tests/
    ├── turtle_tests.rs
    ├── ntriples_tests.rs
    └── rdf_xml_tests.rs
```

## Continuous Integration

Tests run on:
- **Every commit**: Unit tests, linting
- **Every PR**: Full test suite + W3C conformance
- **Weekly**: Extended benchmarks with comparisons
- **Release**: Full regression suite + all benchmarks

## Code Coverage

Target: >90% line coverage

```bash
# Install tarpaulin (code coverage tool)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --exclude-files tests/
```

## Testing Best Practices

### 1. Test Organization
- One test module per file
- Group related tests in `mod` blocks
- Use descriptive test names

### 2. Test Data
- Use minimal fixtures (1-10 triples)
- Avoid large datasets in unit tests
- Use LUBM/SP2Bench for performance tests

### 3. Assertions
- Test one behavior per test
- Use specific assertions (`assert_eq!`, not just `assert!`)
- Provide meaningful failure messages

### 4. Performance Tests
- Run in release mode (`--release`)
- Compare against baselines
- Document regressions

## Debugging Tests

### Run with Output
```bash
cargo test -- --nocapture
```

### Run Single Test
```bash
cargo test test_name
```

### Run with Backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

### Debug in IDE
- VS Code: Use CodeLLDB extension
- IntelliJ: Built-in Rust debugger
- Breakpoints and watch expressions supported

## Next Steps

- [Running Tests](./running.md) - Detailed instructions
- [Benchmarks Guide](./benchmarks.md) - Performance testing
- [W3C Conformance](./w3c-conformance.md) - SPARQL compliance
- [Regression Testing](./regression.md) - Preventing regressions
