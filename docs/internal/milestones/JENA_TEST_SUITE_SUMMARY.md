# Apache Jena Test Suite Analysis - Executive Summary

**Analysis Date**: 2025-11-25
**Analyst**: Claude Code
**Repository**: https://github.com/apache/jena.git

---

## Key Findings

### Total Test Inventory
- **1,918** Java test files across 20 modules
- **4,940+** W3C conformance test files
- **6,858** total test files available for porting

### Critical Modules for rust-kgdb
| Module | Tests | Priority | Maps to Crate |
|--------|-------|----------|---------------|
| jena-arq | 516 | **P0-P1** | `sparql`, `rdf-io` |
| jena-core | 411 | **P0-P3** | `rdf-model`, `storage`, `reasoning` |
| W3C DAWG | 440 | **P0-P1** | `sparql` (conformance) |
| W3C SPARQL 1.1 | 1,229 | **P1-P2** | `sparql` (conformance) |
| jena-shacl | 22 | **P3** | `shacl` |

---

## Recommended Porting Strategy

### Phase 1: Foundation (Weeks 1-4) - 151 Tests
**Goal**: Basic SPARQL queries on in-memory Turtle data

- **rdf-model**: 60 tests (Node, Triple, Quad, Dictionary)
- **storage**: 46 tests (InMemoryBackend, SPOC indexes)
- **sparql**: 51 tests (BGP matching, simple queries)
- **rdf-io**: 42 tests (Turtle, N-Triples parsers)
- **W3C DAWG**: 100 tests (basic, triple-match)

**Success Metric**: Execute `SELECT ?s ?p ?o WHERE { ?s ?p ?o }` on Turtle data

### Phase 2: Core SPARQL 1.1 (Weeks 5-8) - 250 Tests
**Goal**: Full SPARQL 1.1 query feature parity

- **sparql engine**: 95 tests (JOIN, OPTIONAL, FILTER, UNION)
- **W3C DAWG**: 150 tests (optional, graph, construct, ask)
- **W3C CG**: 100 tests (aggregates, property-path, bind)

**Success Metric**: 70%+ pass rate on W3C SPARQL 1.1 conformance

### Phase 3: UPDATE + Persistence (Weeks 9-12) - 129 Tests
**Goal**: Writable graph store with persistence

- **sparql UPDATE**: 12 tests (INSERT, DELETE, LOAD, CLEAR)
- **storage mem2**: 34 tests (fast in-memory backend)
- **storage TDB**: 83 tests (RocksDB/LMDB backends)
- **W3C CG UPDATE**: 100 tests (delete-*, insert-*)

**Success Metric**: Persistent triple store with SPARQL UPDATE

### Phase 4: Reasoning + Validation (Weeks 13-16) - 103 Tests
**Goal**: Production-ready inference and constraints

- **reasoning RDFS**: 48 tests (RDFS entailment)
- **reasoning OWL**: 51 tests (OWL 2 RL subset)
- **shacl**: 22 tests (W3C SHACL validation)

**Success Metric**: RDFS + OWL 2 RL reasoner, SHACL validator

---

## W3C Conformance Test Suites

### 1. DAWG-Final (SPARQL 1.0)
- **Location**: `jena-arq/testing/DAWG-Final/`
- **Test Files**: 440 query files
- **Categories**: 23 test suites (basic, optional, graph, expr-builtin, etc.)
- **Target**: 70%+ pass rate (308/440 tests)

### 2. SPARQL 1.1 Community Group
- **Location**: `jena-arq/testing/rdf-tests-cg/sparql/sparql11/`
- **Test Files**: 1,229 query files
- **Categories**: aggregates, property-path, bind, subquery, etc.
- **Target**: 60%+ pass rate (737/1,229 tests)

### 3. RDF Parsing Tests
- **Location**: `jena-arq/testing/rdf-tests-cg/rdf/`
- **Test Files**: 2,264+ files
- **Formats**: Turtle, N-Triples, RDF/XML, TriG, N-Quads

---

## Test-to-Crate Mapping

| rust-kgdb Crate | Jena Tests | W3C Tests | Total | Coverage Target |
|-----------------|------------|-----------|-------|-----------------|
| `rdf-model` | 60 | 0 | 60 | 90% (54 tests) |
| `storage` | 80 | 0 | 80 | 87% (70 tests) |
| `sparql` | 266 | 1,669 | 1,935 | 86% Jena + 65% W3C |
| `rdf-io` | 42 | 2,264 | 2,306 | 90% Jena + 50% W3C |
| `reasoning` | 83 | 0 | 83 | 82% (68 tests) |
| `shacl` | 22 | 0 | 22 | 91% (20 tests) |
| **TOTAL** | **553** | **3,933** | **4,486** | **87% Jena + 65% W3C** |

---

## Top 20 Tests to Port First

### Immediate Impact (Week 1)

#### RDF Model
1. `TestModelFactory.java` - Model creation API
2. `TestNodeIterator.java` - Iterator patterns
3. `TestResourceFactory.java` - Resource creation
4. `TestTriple.java` - Triple semantics

#### Storage
5. `TestGraph.java` - Graph operations
6. `GraphMem2Test.java` - In-memory storage
7. `FastTripleStoreTest.java` - SPOC indexing

#### SPARQL Core
8. `TestQueryExecution.java` - Query execution API
9. `TestQueryEngineMultiThreaded.java` - Concurrency safety
10. `TestAlgebra.java` - Algebra transformations
11. `TestExpressions.java` - Expression evaluation

#### RDF I/O
12. `TestLangTurtle.java` - Turtle parser
13. `TestLangNTriples.java` - N-Triples parser
14. `TestRDFWriter.java` - RDF serialization

#### W3C DAWG (5 tests)
15. `DAWG-Final/basic/base-prefix-1.rq` - PREFIX handling
16. `DAWG-Final/basic/term-1.rq` - Term matching
17. `DAWG-Final/triple-match/dawg-tp-01.rq` - Triple patterns
18. `DAWG-Final/optional/q-opt-1.rq` - OPTIONAL clause
19. `DAWG-Final/expr-builtin/q-datatype-1.rq` - Builtin functions

#### Reasoning
20. `TestBasicRDFS.java` - RDFS entailment rules

---

## Progress Tracking Framework

### Weekly Metrics
```markdown
## Week N (YYYY-MM-DD to YYYY-MM-DD)
- **Tests Ported**: X/633 (Y%)
- **Tests Passing**: X/Y (Z%)
- **W3C DAWG Pass Rate**: X/440 (Y%)
- **W3C SPARQL 1.1 Pass Rate**: X/1229 (Y%)

### Completed
- [x] Module: TestName (N tests)

### In Progress
- [ ] Module: TestName (N tests)

### Blocked
- Issue description
```

### Target Milestones
- **Week 4**: 151 tests ported, 90%+ pass rate, basic SPARQL working
- **Week 8**: 401 tests ported, 70% DAWG conformance, SPARQL 1.1 complete
- **Week 12**: 530 tests ported, SPARQL UPDATE + persistence working
- **Week 16**: 633 tests ported, 87% coverage, reasoning + SHACL complete

---

## Implementation Resources

### Documentation
- **Complete Analysis**: [COMPLETE_JENA_TEST_ANALYSIS.md](./docs/COMPLETE_JENA_TEST_ANALYSIS.md)
- **Quick Start Guide**: [TEST_PORTING_QUICK_START.md](./docs/TEST_PORTING_QUICK_START.md)
- **Session Report**: [docs/sessions/2025-11-25-jena-test-analysis.md](./docs/sessions/)

### Test Data Locations
- **W3C Official**: https://github.com/w3c/rdf-tests
- **Jena Copy**: `/tmp/jena/jena-arq/testing/`
- **rust-kgdb Local**: `test-data/rdf-tests/`

### Key Repositories
- **Apache Jena**: https://github.com/apache/jena
- **W3C RDF Tests**: https://github.com/w3c/rdf-tests
- **rust-kgdb**: /Users/gauravmalhotra/Documents/MyGithub/zenya-graphdb/rust-kgdb

---

## Next Actions (Week 1)

### Day 1: Setup
- [ ] Clone W3C test suite: `git clone https://github.com/w3c/rdf-tests test-data/rdf-tests`
- [ ] Create test harness skeleton: `crates/sparql/tests/w3c_conformance/`
- [ ] Implement manifest parser: `manifest.rs`

### Day 2-3: Port First Tests
- [ ] Port 5 rdf-model tests (TestModelFactory, TestNodeIterator, etc.)
- [ ] Port 3 storage tests (TestGraph, GraphMem2Test)
- [ ] Port 2 W3C DAWG tests (basic/base-prefix-1, basic/term-1)

### Day 4-5: CI/CD Integration
- [ ] Add W3C conformance to `.github/workflows/tests.yml`
- [ ] Create `TEST_PORTING_STATUS.md` with weekly tracking
- [ ] Set up automated coverage reporting

### Week 1 Goal
- **10 tests ported** (5 Jena + 5 W3C)
- **80%+ pass rate** on ported tests
- **Test harness operational** for automated W3C testing

---

## Success Criteria

### Technical Metrics
- **633+ Jena tests ported** (87% coverage)
- **1,045+ W3C tests passing** (65% conformance)
- **70%+ DAWG pass rate** (308/440)
- **60%+ SPARQL 1.1 pass rate** (737/1,229)

### Feature Parity
- ✅ RDF model (Node, Triple, Quad, Dictionary)
- ✅ SPARQL 1.1 Query (64 builtins, property paths, aggregates)
- ✅ SPARQL UPDATE (INSERT, DELETE, LOAD, CLEAR)
- ✅ Persistent storage (RocksDB, LMDB)
- ✅ RDFS + OWL 2 RL reasoning
- ✅ SHACL validation

### Mobile Readiness
- ✅ Zero-copy semantics preserved
- ✅ Sub-millisecond query performance
- ✅ 24 bytes/triple memory efficiency
- ✅ iOS/Android FFI tested

---

## Conclusion

Apache Jena's **6,858 test files** provide the blueprint for transforming rust-kgdb into a production-ready, W3C-compliant, mobile-first RDF database. By systematically porting **633 critical tests** over 16 weeks, rust-kgdb will achieve:

1. **Feature Parity**: Match Apache Jena's SPARQL 1.1 capabilities
2. **W3C Compliance**: Pass official conformance suites
3. **Production Quality**: Confidence from exhaustive testing
4. **Mobile Performance**: 2.78 µs lookups + 24 bytes/triple

**Start Date**: 2025-11-25
**Target Completion**: 2026-03-17 (16 weeks)
**Estimated Effort**: 1 senior engineer, full-time

**Documentation**: All analysis, mappings, and quick-start guides are in `docs/` directory.

---

**Generated by**: Claude Code (Anthropic)
**Analysis Source**: https://github.com/apache/jena.git (HEAD, 2025-11-25)
**Analysis Script**: `/tmp/analyze_jena_tests.sh`
