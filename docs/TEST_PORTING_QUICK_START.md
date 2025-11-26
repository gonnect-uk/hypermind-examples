# Test Porting Quick Start Guide

**See complete analysis**: [COMPLETE_JENA_TEST_ANALYSIS.md](./COMPLETE_JENA_TEST_ANALYSIS.md)

## TL;DR

Apache Jena has **1,918 test files** + **4,940+ W3C conformance tests**. We will port **480+ critical tests** to rust-kgdb in 4 phases (16 weeks) to achieve 87% coverage and 70% W3C conformance.

---

## Week 1 Action Items

### 1. Clone W3C Test Suite
```bash
cd /Users/gauravmalhotra/Documents/MyGithub/zenya-graphdb/rust-kgdb
mkdir -p test-data
cd test-data
git clone --depth 1 https://github.com/w3c/rdf-tests
```

### 2. Create Test Harness
```bash
# Create conformance test structure
mkdir -p crates/sparql/tests/w3c_conformance

# Files to create:
# - crates/sparql/tests/w3c_conformance/mod.rs (main test runner)
# - crates/sparql/tests/w3c_conformance/manifest.rs (Turtle manifest parser)
# - crates/sparql/tests/w3c_conformance/harness.rs (test execution)
```

### 3. Port First 10 Tests
**Priority 0 tests from jena-core**:
```
1. TestModelFactory.java → rdf-model/tests/model_factory.rs
2. TestNodeIterator.java → rdf-model/tests/node_iterator.rs
3. TestTriple.java → rdf-model/tests/triple.rs
4. TestGraph.java → storage/tests/graph.rs
5. GraphMem2Test.java → storage/tests/mem2.rs
```

**Priority 0 tests from W3C DAWG**:
```
6. DAWG-Final/basic/base-prefix-1.rq
7. DAWG-Final/basic/term-1.rq
8. DAWG-Final/triple-match/dawg-tp-01.rq
9. DAWG-Final/triple-match/dawg-tp-02.rq
10. DAWG-Final/optional/q-opt-1.rq
```

---

## Test Porting Template

### Java to Rust Conversion

**Jena Java Test**:
```java
// jena-core/src/test/java/org/apache/jena/rdf/model/TestModelFactory.java
@Test
public void testCreateDefaultModel() {
    Model model = ModelFactory.createDefaultModel();
    assertNotNull(model);
    assertTrue(model.isEmpty());
}
```

**rust-kgdb Rust Test**:
```rust
// crates/rdf-model/tests/model_factory.rs
use rdf_model::{Model, Graph};
use storage::InMemoryBackend;

#[test]
fn test_create_default_model() {
    let backend = InMemoryBackend::new();
    let model = Model::new(backend);

    assert!(model.is_empty());
}
```

### W3C Conformance Test

**Test Structure**:
```
DAWG-Final/basic/base-prefix-1/
├── manifest.ttl      # Test manifest
├── query.rq          # SPARQL query
├── data.ttl          # RDF data
└── result.srx        # Expected result (SPARQL XML)
```

**Rust Implementation**:
```rust
#[test]
fn test_dawg_basic_base_prefix_1() {
    // 1. Parse data
    let data = parse_turtle("test-data/rdf-tests/DAWG-Final/basic/base-prefix-1.ttl");
    let store = QuadStore::new();
    store.insert_triples(data);

    // 2. Execute query
    let query = parse_query("test-data/rdf-tests/DAWG-Final/basic/base-prefix-1.rq");
    let executor = Executor::new(&store);
    let result = executor.execute(query);

    // 3. Compare results
    let expected = parse_sparql_xml("test-data/rdf-tests/DAWG-Final/basic/base-prefix-1.srx");
    assert_eq!(result, expected);
}
```

---

## Test Categories by Priority

### P0: Critical Path (151 tests)
- **rdf-model**: 60 tests (model, node, triple, quad)
- **storage**: 46 tests (graph, indexes, iteration)
- **sparql**: 51 tests (core engine, BGP matching)
- **rdf-io**: 42 tests (Turtle, N-Triples parsers)
- **W3C DAWG**: 100 tests (basic, triple-match, algebra)

### P1: Core SPARQL 1.1 (250 tests)
- **sparql**: 95 tests (engine, expr, algebra, functions)
- **W3C DAWG**: 150 tests (optional, graph, construct, ask, builtin)
- **W3C CG**: 100 tests (aggregates, property-path, bind)

### P2: UPDATE + Persistence (129 tests)
- **sparql**: 12 tests (modify operations)
- **storage**: 34 tests (mem2 backend)
- **storage**: 83 tests (TDB1/TDB2 sample)
- **W3C CG**: 100 tests (delete, insert, update)

### P3: Reasoning + Validation (103 tests)
- **reasoning**: 48 tests (RDFS, rulesys)
- **reasoning**: 51 tests (OWL ontapi)
- **shacl**: 22 tests (W3C SHACL)

---

## Progress Tracking

Create `TEST_PORTING_STATUS.md` with weekly updates:

```markdown
# Test Porting Status

## Week 1 (2025-11-25 to 2025-12-01)
- Tests Ported: 10/480 (2%)
- Tests Passing: 8/10 (80%)
- W3C Pass Rate: 0/440 DAWG (0%)

### Completed
- [x] rdf-model: TestModelFactory (5 tests)
- [x] storage: TestGraph (3 tests)
- [x] W3C DAWG: basic/base-prefix-1 (1 test)
- [x] W3C DAWG: basic/term-1 (1 test)

### In Progress
- [ ] sparql: TestQueryExecution (12 tests)
- [ ] rdf-io: TestLangTurtle (8 tests)

### Blocked
- None
```

---

## CI/CD Integration

Add to `.github/workflows/tests.yml`:

```yaml
- name: Clone W3C Test Suite
  run: |
    git clone --depth 1 https://github.com/w3c/rdf-tests test-data/rdf-tests

- name: Run W3C Conformance Tests
  run: |
    cargo test --package sparql --test w3c_conformance -- --nocapture

- name: Generate Coverage Report
  run: |
    cargo tarpaulin --out Html --output-dir coverage/
```

---

## Key Files to Reference

### Jena Test Files (Temporary)
```bash
# Clone Jena for reference (temporary, delete after porting)
cd /tmp
git clone --depth 1 https://github.com/apache/jena.git
cd jena

# Find specific test
find . -name "TestModelFactory.java"

# View test
cat jena-core/src/test/java/org/apache/jena/rdf/model/TestModelFactory.java
```

### W3C Test Data (Permanent)
```bash
# Permanent location in rust-kgdb
cd /Users/gauravmalhotra/Documents/MyGithub/zenya-graphdb/rust-kgdb/test-data/rdf-tests

# SPARQL 1.0 (DAWG)
ls rdf-tests/sparql10/
# OR (Jena's copy)
ls /tmp/jena/jena-arq/testing/DAWG-Final/

# SPARQL 1.1
ls rdf-tests/sparql11/
```

---

## Success Metrics

### Phase 1 (Week 4)
- 151 P0 tests ported
- 90%+ pass rate on P0 tests
- Can execute basic SPARQL queries on Turtle data

### Phase 2 (Week 8)
- 401 tests ported (P0 + P1)
- 70%+ W3C DAWG pass rate (308/440)
- Full SPARQL 1.1 query support

### Phase 3 (Week 12)
- 530 tests ported (P0 + P1 + P2)
- SPARQL UPDATE working
- Persistent storage (RocksDB/LMDB)

### Phase 4 (Week 16)
- 633 tests ported (ALL priorities)
- 70% W3C conformance (1,045+ tests)
- RDFS + OWL 2 RL + SHACL validation

---

## Quick Reference: Test Locations

| Jena Module | Test Package | rust-kgdb Crate | Test Path |
|-------------|--------------|-----------------|-----------|
| jena-core | `rdf.model` | `rdf-model` | `crates/rdf-model/tests/` |
| jena-core | `graph` | `storage` | `crates/storage/tests/` |
| jena-core | `reasoner` | `reasoning` | `crates/reasoning/tests/` |
| jena-arq | `sparql` | `sparql` | `crates/sparql/tests/` |
| jena-arq | `riot` | `rdf-io` | `crates/rdf-io/tests/` |
| jena-shacl | All | `shacl` | `crates/shacl/tests/` |
| W3C DAWG | All | `sparql` | `crates/sparql/tests/w3c_conformance/` |

---

## Next Steps

1. **Today**: Clone W3C test suite, create test harness skeleton
2. **This Week**: Port first 10 P0 tests (5 Jena + 5 W3C)
3. **Next Week**: Port 40 more P0 tests, set up CI/CD
4. **Month 1**: Complete Phase 1 (151 P0 tests)

**Full details**: See [COMPLETE_JENA_TEST_ANALYSIS.md](./COMPLETE_JENA_TEST_ANALYSIS.md)
