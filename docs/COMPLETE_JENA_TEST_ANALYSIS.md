# Complete Apache Jena Test Suite Analysis & Mapping to rust-kgdb

**Generated**: 2025-11-25
**Repository**: https://github.com/apache/jena.git (HEAD)
**Total Test Files**: 1,918 Java test classes

---

## Executive Summary

Apache Jena's test suite spans **1,918 test files** across **20 modules**, with the most critical tests concentrated in:
1. **jena-arq** (516 files) - SPARQL 1.1 Query/Update engine
2. **jena-core** (411 files) - RDF model, graphs, serialization
3. **jena-geosparql** (147 files) - GeoSPARQL extensions
4. **jena-tdb1/tdb2** (215 files) - Persistent triple stores

Additionally, Jena maintains **4,940+ W3C conformance test files** in `jena-arq/testing/` covering:
- SPARQL 1.0/1.1/1.2 queries (1,229 tests)
- DAWG evaluation tests (440 tests)
- RDF parsing tests (2,264+ tests)
- SPARQL UPDATE tests (13 tests)
- SPARQL-CDTs (707 tests)

---

## Complete Module Inventory

| Module | Test Files | Primary Focus | rust-kgdb Target Crate |
|--------|------------|---------------|------------------------|
| **jena-arq** | 516 | SPARQL Query/Update, RIOT parsers | `sparql`, `rdf-io` |
| **jena-core** | 411 | RDF model, graphs, reasoning | `rdf-model`, `storage`, `reasoning` |
| **jena-geosparql** | 147 | GeoSPARQL extensions | *(future extension)* |
| **jena-tdb1** | 133 | TDB1 persistent storage | `storage` (RocksDB backend) |
| **jena-fuseki2** | 113 | HTTP server | *(not applicable)* |
| **jena-tdb2** | 82 | TDB2 storage | `storage` (LMDB backend) |
| **jena-db** | 76 | Database layer | `storage` |
| **jena-integration-tests** | 72 | End-to-end tests | All crates |
| **jena-benchmarks** | 62 | Performance tests | Criterion benchmarks |
| **jena-base** | 56 | Foundation types | `rdf-model` |
| **jena-ontapi** | 51 | OWL ontologies | `reasoning` (OWL 2 RL) |
| **jena-text** | 49 | Full-text search | *(future extension)* |
| **jena-extras** | 43 | Additional utilities | *(case-by-case)* |
| **jena-shex** | 29 | ShEx validation | `shex` |
| **jena-shacl** | 22 | SHACL validation | `shacl` |
| **jena-iri3986** | 20 | IRI/URI handling | `rdf-model` |
| **jena-rdfconnection** | 14 | RDF connection API | `mobile-ffi` |
| **jena-rdfpatch** | 11 | RDF patches | *(future extension)* |
| **jena-cmds** | 6 | Command-line tools | *(not applicable)* |
| **jena-langtag** | 5 | Language tags | `rdf-model` |
| **TOTAL** | **1,918** | | |

---

## jena-core Test Breakdown (411 files)

### By Package (Top 10)
| Package | Count | rust-kgdb Crate | Description |
|---------|-------|-----------------|-------------|
| `rdf` | 60 | `rdf-model` | Core RDF model: Node, Triple, Statement |
| `rdfxml` | 51 | `rdf-io` | RDF/XML parser/serializer |
| `reasoner` | 49 | `reasoning` | RDFS, OWL reasoners |
| `graph` | 46 | `storage` | Graph implementations, quad stores |
| `mem2` | 34 | `storage` | In-memory graph store v2 |
| `ttl_test` | 31 | `rdf-io` | Turtle parser tests |
| `assembler` | 26 | *(skip)* | Jena configuration framework |
| `util` | 25 | `rdf-model` | Utilities: iterators, collections |
| `ontology` | 22 | `reasoning` | Ontology API tests |
| `mem` | 12 | `storage` | In-memory graph store v1 |
| **Others** | 55 | Various | enhanced, irix, vocabulary, shared, datatypes |

### Critical Test Files (jena-core)
```
jena-core/src/test/java/org/apache/jena/
├── rdf/
│   ├── model/
│   │   ├── TestModelFactory.java
│   │   ├── TestNodeIterator.java
│   │   ├── TestResourceFactory.java
│   │   └── TestStatementImpls.java
│   └── parser/
│       ├── TestNTriples.java
│       └── TestTurtle.java
├── graph/
│   ├── TestGraph.java
│   ├── TestGraphMaker.java
│   ├── TestTriple.java
│   └── compose/
│       ├── TestUnion.java
│       └── TestIntersection.java
├── reasoner/
│   ├── rulesys/
│   │   ├── TestBasicRDFS.java
│   │   ├── TestOWLRules.java
│   │   ├── TestGenericRules.java
│   │   └── TestForwardRuleInfGraphImpl.java
│   └── test/
│       ├── TestRDFSReasoners.java
│       └── TestTransitiveReasoner.java
└── mem2/
    ├── GraphMem2Test.java
    ├── GraphMem2FastTest.java
    └── store/
        ├── fast/FastTripleStoreTest.java
        └── roaring/RoaringTripleStoreTest.java
```

---

## jena-arq Test Breakdown (516 files)

### By Package (Top 10)
| Package | Count | rust-kgdb Crate | Description |
|---------|-------|-----------------|-------------|
| `sparql` | 266 | `sparql` | SPARQL query/update engine |
| `riot` | 134 | `rdf-io` | RDF I/O, parsers, serializers |
| `arq` | 37 | `sparql` | ARQ engine core |
| `atlas` | 22 | *(skip)* | Utility library |
| `system` | 19 | All | System-level integration tests |
| `rdfs` | 14 | `reasoning` | RDFS entailment |
| `rdf12` | 14 | `rdf-model` | RDF 1.2 features |
| `query` | 5 | `sparql` | Query API |
| `http` | 3 | *(skip)* | HTTP protocol |
| `util` | 2 | Various | Utilities |

### SPARQL Subcategories (266 tests)
| Subcategory | Count | Focus |
|-------------|-------|-------|
| `core` | 51 | Core SPARQL algebra |
| `engine` | 37 | Query execution engine |
| `expr` | 29 | Expression evaluation |
| `algebra` | 29 | Algebra transformations |
| `util` | 21 | Utilities |
| `function` | 18 | Builtin functions |
| `modify` | 12 | SPARQL UPDATE |
| `graph` | 12 | Graph operations |
| `syntax` | 9 | Parser tests |
| `path` | 5 | Property paths |
| **Others** | 43 | resultset, transaction, exec, solver, etc. |

### RIOT Subcategories (134 tests)
| Subcategory | Count | Focus |
|-------------|-------|-------|
| `lang` | 42 | Language parsers (Turtle, N-Triples, RDF/XML) |
| `system` | 22 | System integration |
| `writer` | 12 | RDF serializers |
| `out` | 6 | Output formatting |
| `thrift` | 5 | Thrift serialization |
| `protobuf` | 5 | Protobuf serialization |
| **Others** | 42 | stream, rowset, tokens, web, adapters |

### Critical Test Files (jena-arq)
```
jena-arq/src/test/java/org/apache/jena/
├── sparql/
│   ├── core/
│   │   ├── TestDatasetDescription.java
│   │   ├── TestQueryExecution.java
│   │   └── TestVarExprList.java
│   ├── engine/
│   │   ├── TestQueryEngineMultiThreaded.java
│   │   ├── binding/TestBindingComparator.java
│   │   ├── join/TestJoin.java
│   │   └── iterator/TestQueryIteratorBase.java
│   ├── expr/
│   │   ├── TestExpressions.java
│   │   ├── TestExprTransform.java
│   │   └── TestNodeValue.java
│   ├── algebra/
│   │   ├── TestAlgebra.java
│   │   ├── TestQuadPattern.java
│   │   └── optimize/TestOptimizer.java
│   ├── function/
│   │   ├── TestFunctionExpansion.java
│   │   ├── TestCastXSD.java
│   │   └── library/TestFnFunctionsString.java
│   ├── modify/
│   │   ├── TestUpdateAPI.java
│   │   ├── TestUpdateOperations.java
│   │   └── TestUpdateTransform.java
│   └── path/
│       ├── TestPath.java
│       └── TestPropertyPath.java
├── riot/
│   ├── lang/
│   │   ├── TestLangTurtle.java
│   │   ├── TestLangNTriples.java
│   │   ├── TestLangNQuads.java
│   │   ├── TestLangRDFXML.java
│   │   └── TestLangTriG.java
│   ├── writer/
│   │   ├── TestRDFWriter.java
│   │   ├── TestNTriplesWriter.java
│   │   └── TestTurtleWriter.java
│   └── system/
│       ├── TestStreamRDFLib.java
│       └── TestIOUtils.java
└── arq/
    ├── exec/TestExecutorAPI.java
    └── test/TestScriptRunner.java
```

---

## W3C Conformance Test Suites

Located in `jena-arq/testing/` directory with **4,940+ test files**.

### 1. DAWG-Final (SPARQL 1.0 Test Suite)
**Location**: `jena-arq/testing/DAWG-Final/`
**Test Files**: 440 `.rq` query files, 668 total test files
**Manifest**: `manifest-evaluation.ttl`, `manifest-syntax.ttl`

**Categories** (23 test suites):
| Category | Description | rust-kgdb Priority |
|----------|-------------|--------------------|
| `basic` | Basic triple patterns | **P0 - Critical** |
| `triple-match` | Triple matching semantics | **P0 - Critical** |
| `algebra` | SPARQL algebra operations | **P0 - Critical** |
| `optional` | OPTIONAL patterns | **P0 - Critical** |
| `optional-filter` | OPTIONAL with FILTER | **P1 - High** |
| `graph` | GRAPH operations | **P1 - High** |
| `dataset` | Dataset operations | **P1 - High** |
| `expr-builtin` | Builtin functions | **P1 - High** |
| `expr-ops` | Operators (+, -, *, /) | **P1 - High** |
| `expr-equals` | Equality semantics | **P1 - High** |
| `bound` | BOUND() tests | **P1 - High** |
| `regex` | REGEX() function | **P1 - High** |
| `cast` | Type casting | **P2 - Medium** |
| `type-promotion` | Numeric type promotion | **P2 - Medium** |
| `boolean-effective-value` | Boolean EBV | **P2 - Medium** |
| `bnode-coreference` | Blank node handling | **P2 - Medium** |
| `construct` | CONSTRUCT queries | **P1 - High** |
| `ask` | ASK queries | **P1 - High** |
| `distinct` | DISTINCT modifier | **P1 - High** |
| `reduced` | REDUCED modifier | **P2 - Medium** |
| `sort` | ORDER BY | **P1 - High** |
| `solution-seq` | Solution sequences | **P2 - Medium** |
| `open-world` | Open-world semantics | **P2 - Medium** |

**Example Test Structure**:
```
DAWG-Final/basic/
├── manifest.ttl          # Test manifest
├── base-prefix-1.rq      # Query file
├── base-prefix-1.ttl     # Data file
└── base-prefix-1.srx     # Expected result (SPARQL XML)
```

### 2. RDF Tests Community Group (SPARQL 1.1/1.2)
**Location**: `jena-arq/testing/rdf-tests-cg/sparql/`
**Test Files**: 1,229 `.rq`/`.ru` query files, 2,264+ total files
**Source**: https://github.com/w3c/rdf-tests

**Structure**:
- `sparql10/` - SPARQL 1.0 tests (legacy)
- `sparql11/` - SPARQL 1.1 official tests (property paths, aggregates, subqueries)
- `sparql12/` - SPARQL 1.2 draft tests (CDTs, IF/COALESCE)

**SPARQL 1.1 Categories**:
| Category | Test Files | Description |
|----------|-----------|-------------|
| `aggregates/` | ~50 | COUNT, SUM, AVG, MIN, MAX, GROUP_CONCAT |
| `bind/` | ~20 | BIND clause |
| `bindings/` | ~15 | VALUES clause |
| `clear/` | ~10 | CLEAR operation |
| `construct/` | ~30 | CONSTRUCT WHERE |
| `csv-tsv-res/` | ~25 | CSV/TSV result formats |
| `delete/` | ~20 | DELETE operations |
| `delete-data/` | ~15 | DELETE DATA |
| `delete-insert/` | ~25 | DELETE/INSERT |
| `delete-where/` | ~15 | DELETE WHERE |
| `drop/` | ~10 | DROP operations |
| `exists/` | ~15 | EXISTS and NOT EXISTS |
| `functions/` | ~40 | Builtin functions |
| `grouping/` | ~30 | GROUP BY |
| `insert/` | ~20 | INSERT operations |
| `insert-data/` | ~15 | INSERT DATA |
| `json-res/` | ~10 | JSON result format |
| `negation/` | ~20 | NOT EXISTS, MINUS |
| `project-expression/` | ~15 | SELECT expressions |
| `property-path/` | ~50 | Property paths (*, +, ?) |
| `service/` | ~15 | SERVICE federation |
| `subquery/` | ~30 | Subqueries |
| `syntax-query/` | ~100 | Query syntax tests |
| `syntax-update/` | ~50 | UPDATE syntax tests |

### 3. ARQ-Specific Tests
**Location**: `jena-arq/testing/ARQ/`
**Test Files**: 1,201 files
**Description**: ARQ extension tests (Jena-specific features)

**Categories**:
- Basic SPARQL features
- Extension functions
- Property functions
- Algebra optimizations
- Dataset features

### 4. RIOT RDF Parser Tests
**Location**: `jena-arq/testing/RIOT/`
**Test Files**: 54 files
**Description**: RDF I/O tests (Turtle, N-Triples, RDF/XML, TriG, N-Quads)

### 5. SPARQL-CDTs (Community Draft)
**Location**: `jena-arq/testing/SPARQL-CDTs/`
**Test Files**: 707 files
**Source**: https://github.com/awslabs/SPARQL-CDTs
**Description**: Collection and datatype tests (SPARQL 1.2 preview)

### 6. UPDATE Tests
**Location**: `jena-arq/testing/Update/`
**Test Files**: 13 files
**Description**: SPARQL UPDATE operations (INSERT, DELETE, LOAD, CLEAR)

---

## Test-to-Crate Mapping

### Priority 0: Critical Path (Foundation)
**Target**: Achieve basic SPARQL 1.0 + RDF I/O functionality

| Jena Module | Test Category | rust-kgdb Crate | Test Count | Priority |
|-------------|--------------|-----------------|------------|----------|
| jena-core | `rdf/model` | `rdf-model` | 60 | **P0** |
| jena-core | `graph` | `storage` | 46 | **P0** |
| jena-arq | `sparql/core` | `sparql` | 51 | **P0** |
| jena-arq | `riot/lang` | `rdf-io` | 42 | **P0** |
| W3C DAWG | `basic`, `triple-match` | `sparql` | ~100 | **P0** |

**Deliverable**: Execute basic SPARQL queries on in-memory triples, parse Turtle/N-Triples.

### Priority 1: Core SPARQL 1.1 Features
**Target**: Complete SPARQL 1.1 query engine

| Jena Module | Test Category | rust-kgdb Crate | Test Count | Priority |
|-------------|--------------|-----------------|------------|----------|
| jena-arq | `sparql/engine` | `sparql` | 37 | **P1** |
| jena-arq | `sparql/expr` | `sparql` | 29 | **P1** |
| jena-arq | `sparql/algebra` | `sparql` | 29 | **P1** |
| jena-arq | `sparql/function` | `sparql` | 18 | **P1** |
| W3C DAWG | `optional`, `graph`, `construct`, `ask` | `sparql` | ~150 | **P1** |
| W3C CG | `sparql11/aggregates` | `sparql` | ~50 | **P1** |
| W3C CG | `sparql11/property-path` | `sparql` | ~50 | **P1** |

**Deliverable**: Full SPARQL 1.1 query support with aggregates, property paths, subqueries.

### Priority 2: SPARQL UPDATE + Persistence
**Target**: SPARQL UPDATE operations + persistent storage

| Jena Module | Test Category | rust-kgdb Crate | Test Count | Priority |
|-------------|--------------|-----------------|------------|----------|
| jena-arq | `sparql/modify` | `sparql` | 12 | **P2** |
| jena-core | `mem2` | `storage` | 34 | **P2** |
| jena-tdb1 | All | `storage` (RocksDB) | 133 | **P2** |
| jena-tdb2 | All | `storage` (LMDB) | 82 | **P2** |
| W3C CG | `sparql11/delete-*`, `insert-*` | `sparql` | ~100 | **P2** |

**Deliverable**: INSERT/DELETE operations, persistent triple stores (RocksDB, LMDB).

### Priority 3: Reasoning + Validation
**Target**: RDFS/OWL reasoning, SHACL/ShEx validation

| Jena Module | Test Category | rust-kgdb Crate | Test Count | Priority |
|-------------|--------------|-----------------|------------|----------|
| jena-core | `reasoner/rulesys` | `reasoning` | 34 | **P3** |
| jena-ontapi | All | `reasoning` | 51 | **P3** |
| jena-shacl | All | `shacl` | 22 | **P3** |
| jena-shex | All | `shex` (future) | 29 | **P3** |
| jena-arq | `rdfs` | `reasoning` | 14 | **P3** |

**Deliverable**: RDFS entailment, OWL 2 RL, SHACL validation, ShEx validation.

### Priority 4: Advanced Formats + Extensions
**Target**: Additional RDF formats, serializations

| Jena Module | Test Category | rust-kgdb Crate | Test Count | Priority |
|-------------|--------------|-----------------|------------|----------|
| jena-core | `rdfxml` | `rdf-io` | 51 | **P4** |
| jena-arq | `riot/writer` | `rdf-io` | 12 | **P4** |
| jena-arq | `riot/thrift`, `protobuf` | `rdf-io` | 10 | **P4** |
| jena-geosparql | All | *(future extension)* | 147 | **P4** |

**Deliverable**: RDF/XML parser, Thrift/Protobuf serialization, GeoSPARQL.

---

## Recommended Porting Strategy

### Phase 1: Foundation (Weeks 1-4)
**Goal**: Port critical P0 tests to rust-kgdb

1. **Week 1**: `rdf-model` crate
   - Port 60 jena-core `rdf/model` tests
   - Focus: Node, Triple, Quad, Dictionary
   - Expected: 90%+ pass rate

2. **Week 2**: `storage` crate
   - Port 46 jena-core `graph` tests
   - Focus: InMemoryBackend, SPOC indexes
   - Expected: 85%+ pass rate

3. **Week 3**: `sparql` crate (basic)
   - Port 51 jena-arq `sparql/core` tests
   - Port 100 W3C DAWG `basic` + `triple-match` tests
   - Focus: BGP matching, simple queries
   - Expected: 70%+ pass rate

4. **Week 4**: `rdf-io` crate
   - Port 42 jena-arq `riot/lang` tests
   - Focus: Turtle, N-Triples parsers
   - Expected: 80%+ pass rate

**Milestone**: Execute `SELECT ?s ?p ?o WHERE { ?s ?p ?o }` on in-memory Turtle data.

### Phase 2: Core SPARQL (Weeks 5-8)
**Goal**: Achieve SPARQL 1.1 query feature parity

1. **Weeks 5-6**: SPARQL engine
   - Port 95 jena-arq `sparql/engine`, `expr`, `algebra` tests
   - Port 150 W3C DAWG tests (optional, graph, construct, ask)
   - Focus: JOIN, OPTIONAL, FILTER, UNION

2. **Weeks 7-8**: Builtin functions + property paths
   - Port 18 jena-arq `sparql/function` tests
   - Port 50 W3C CG `property-path` tests
   - Port 50 W3C CG `aggregates` tests
   - Focus: 64 SPARQL builtins, property paths, GROUP BY

**Milestone**: Pass W3C SPARQL 1.1 conformance suite (70%+ pass rate).

### Phase 3: UPDATE + Persistence (Weeks 9-12)
**Goal**: Writable graph store with persistence

1. **Weeks 9-10**: SPARQL UPDATE
   - Port 12 jena-arq `sparql/modify` tests
   - Port 100 W3C CG `delete-*`, `insert-*` tests
   - Focus: INSERT DATA, DELETE WHERE, DELETE/INSERT

2. **Weeks 11-12**: Persistent storage
   - Port 34 jena-core `mem2` tests
   - Port 50 jena-tdb1 + jena-tdb2 tests (representative sample)
   - Focus: RocksDB, LMDB backends, transaction semantics

**Milestone**: Persistent triple store with SPARQL UPDATE support.

### Phase 4: Reasoning + Validation (Weeks 13-16)
**Goal**: Production-ready inference and constraint validation

1. **Weeks 13-14**: RDFS reasoning
   - Port 34 jena-core `reasoner/rulesys` tests
   - Port 14 jena-arq `rdfs` tests
   - Focus: RDFS entailment rules

2. **Weeks 15-16**: SHACL + OWL
   - Port 22 jena-shacl tests
   - Port 20 jena-ontapi tests (RDFS/OWL 2 RL subset)
   - Focus: SHACL constraint validation, OWL 2 RL rules

**Milestone**: RDFS + OWL 2 RL reasoner, SHACL validator.

---

## W3C Conformance Test Execution

### Running W3C Tests in rust-kgdb

Jena's W3C tests use **test manifests** (Turtle files) with standardized structure:

**Example Manifest** (`DAWG-Final/basic/manifest.ttl`):
```turtle
@prefix mf: <http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#> .
@prefix qt: <http://www.w3.org/2001/sw/DataAccess/tests/test-query#> .

<#test-name> rdf:type mf:QueryEvaluationTest ;
    mf:name "Test name" ;
    mf:action [
        qt:query <query.rq> ;
        qt:data <data.ttl>
    ] ;
    mf:result <result.srx> .
```

**Rust Implementation Strategy**:
```rust
// crates/sparql/tests/w3c_conformance/mod.rs
use rdf_model::{Node, Triple, Quad};
use sparql::{QueryEngine, Executor};
use rdf_io::{parse_turtle, parse_sparql_xml};

#[test]
fn test_w3c_dawg_basic() {
    let manifest = parse_manifest("test-data/DAWG-Final/basic/manifest.ttl");

    for test_case in manifest.tests {
        // 1. Load data
        let data = parse_turtle(test_case.data_file);
        let store = QuadStore::new();
        store.insert_triples(data);

        // 2. Execute query
        let query = parse_query(test_case.query_file);
        let executor = Executor::new(&store);
        let result = executor.execute(query);

        // 3. Compare results
        let expected = parse_sparql_xml(test_case.result_file);
        assert_eq!(result, expected, "Test failed: {}", test_case.name);
    }
}
```

### Manifest Parser Requirements
- Parse Turtle manifests (`mf:Manifest`, `mf:entries`)
- Extract test cases (`mf:QueryEvaluationTest`, `mf:UpdateEvaluationTest`)
- Handle actions (`qt:query`, `qt:data`, `qt:graphData`)
- Parse expected results (`.srx`, `.ttl`, `.csv`, `.tsv`)

### Test Harness Features
1. **Parallel Execution**: Run independent tests in parallel (rayon)
2. **Test Filtering**: Run specific categories (`--filter basic`)
3. **Failure Tracking**: Generate detailed failure reports
4. **Progress Reporting**: Real-time pass/fail counters
5. **Diff Output**: Show expected vs actual results

**Implementation**:
```rust
// crates/sparql/tests/w3c_conformance/harness.rs
pub struct TestHarness {
    manifests: Vec<Manifest>,
    results: TestResults,
}

impl TestHarness {
    pub fn run_all(&mut self) {
        self.manifests.par_iter().for_each(|manifest| {
            for test in &manifest.tests {
                let result = self.run_test(test);
                self.results.record(test.name.clone(), result);
            }
        });
    }

    pub fn report(&self) {
        println!("W3C Conformance: {}/{} passed ({:.1}%)",
            self.results.passed,
            self.results.total,
            self.results.pass_rate());
    }
}
```

---

## Test Coverage Tracking

### Current rust-kgdb Coverage (Baseline)
| Crate | Jena Tests Ported | Total Jena Tests | Coverage |
|-------|-------------------|------------------|----------|
| `rdf-model` | 0 | 60 | 0% |
| `storage` | 0 | 80 | 0% |
| `sparql` | ~15 (manual) | 266 | 6% |
| `rdf-io` | 0 | 42 | 0% |
| `reasoning` | 0 | 83 | 0% |
| `shacl` | 0 | 22 | 0% |
| **TOTAL** | ~15 | **553** | **2.7%** |

### Target Coverage (End of Phase 4)
| Crate | Jena Tests Ported | Total Jena Tests | Coverage |
|-------|-------------------|------------------|----------|
| `rdf-model` | 54 | 60 | 90% |
| `storage` | 70 | 80 | 87% |
| `sparql` | 230 | 266 | 86% |
| `rdf-io` | 38 | 42 | 90% |
| `reasoning` | 68 | 83 | 82% |
| `shacl` | 20 | 22 | 91% |
| **TOTAL** | **480** | **553** | **87%** |

Plus W3C conformance:
- DAWG-Final: 70%+ pass rate (308/440 tests)
- SPARQL 1.1 CG: 60%+ pass rate (737/1,229 tests)
- Total W3C: 1,045+ passing tests

---

## Critical Test Files to Port First

### Top 20 High-Value Tests (Immediate Impact)

#### RDF Model (rdf-model crate)
1. `jena-core/rdf/model/TestModelFactory.java` - Model creation
2. `jena-core/rdf/model/TestNodeIterator.java` - Iteration
3. `jena-core/rdf/model/TestResourceFactory.java` - Resource creation
4. `jena-core/graph/TestTriple.java` - Triple semantics

#### Storage (storage crate)
5. `jena-core/graph/TestGraph.java` - Graph operations
6. `jena-core/mem2/GraphMem2Test.java` - In-memory store
7. `jena-core/mem2/store/fast/FastTripleStoreTest.java` - Fast indexing

#### SPARQL Core (sparql crate)
8. `jena-arq/sparql/core/TestQueryExecution.java` - Query execution
9. `jena-arq/sparql/engine/TestQueryEngineMultiThreaded.java` - Concurrency
10. `jena-arq/sparql/algebra/TestAlgebra.java` - Algebra transformations
11. `jena-arq/sparql/expr/TestExpressions.java` - Expression evaluation
12. `jena-arq/sparql/function/TestFnFunctionsString.java` - String functions

#### SPARQL UPDATE (sparql crate)
13. `jena-arq/sparql/modify/TestUpdateAPI.java` - UPDATE API
14. `jena-arq/sparql/modify/TestUpdateOperations.java` - INSERT/DELETE

#### RDF I/O (rdf-io crate)
15. `jena-arq/riot/lang/TestLangTurtle.java` - Turtle parser
16. `jena-arq/riot/lang/TestLangNTriples.java` - N-Triples parser
17. `jena-arq/riot/writer/TestRDFWriter.java` - RDF serialization

#### Reasoning (reasoning crate)
18. `jena-core/reasoner/rulesys/TestBasicRDFS.java` - RDFS rules
19. `jena-core/reasoner/test/TestRDFSReasoners.java` - RDFS reasoner

#### SHACL (shacl crate)
20. `jena-shacl/tests/std/TS_StdSHACL.java` - W3C SHACL conformance

---

## Integration with rust-kgdb CI/CD

### GitHub Actions Workflow

```yaml
# .github/workflows/w3c-conformance.yml
name: W3C Conformance Tests

on: [push, pull_request]

jobs:
  conformance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Clone W3C Test Suite
        run: |
          git clone --depth 1 https://github.com/w3c/rdf-tests test-data/rdf-tests

      - name: Run DAWG Tests
        run: |
          cargo test --package sparql --test w3c_conformance -- dawg --nocapture

      - name: Run SPARQL 1.1 Tests
        run: |
          cargo test --package sparql --test w3c_conformance -- sparql11 --nocapture

      - name: Generate Report
        run: |
          cargo test --package sparql --test w3c_conformance -- --report-json > report.json

      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: conformance-report
          path: report.json
```

### Cargo Test Configuration

```toml
# crates/sparql/Cargo.toml
[[test]]
name = "w3c_conformance"
path = "tests/w3c_conformance/mod.rs"
required-features = ["conformance-tests"]

[features]
conformance-tests = []
```

---

## Appendix: Complete File Listings

### jena-core Test Files (411 total)
Generated with: `grep "^./jena-core" /tmp/jena-all-tests.txt`

**Categories**:
- `mem2/`: 34 files (modern in-memory graph)
- `rdf/model`: 60 files (RDF API)
- `rdfxml/`: 51 files (RDF/XML parser)
- `reasoner/rulesys`: 34 files (rule-based reasoning)
- `graph/`: 46 files (graph implementations)
- `ttl_test/`: 31 files (Turtle tests)
- `assembler/`: 26 files (configuration)
- `ontology/`: 22 files (ontology API)
- Others: 107 files

### jena-arq Test Files (516 total)
Generated with: `grep "^./jena-arq" /tmp/jena-all-tests.txt`

**Categories**:
- `sparql/`: 266 files (SPARQL engine)
  - `core/`: 51 files
  - `engine/`: 37 files
  - `expr/`: 29 files
  - `algebra/`: 29 files
  - `function/`: 18 files
  - `modify/`: 12 files
  - Others: 90 files
- `riot/`: 134 files (RDF I/O)
  - `lang/`: 42 files
  - `system/`: 22 files
  - `writer/`: 12 files
  - Others: 58 files
- `arq/`: 37 files (ARQ core)
- `atlas/`: 22 files (utilities)
- `system/`: 19 files (integration)
- Others: 38 files

### W3C Test Data Locations
- **DAWG-Final**: `jena-arq/testing/DAWG-Final/` (440 queries, 668 files)
- **SPARQL 1.1**: `jena-arq/testing/rdf-tests-cg/sparql/sparql11/` (1,229 queries)
- **SPARQL 1.2**: `jena-arq/testing/rdf-tests-cg/sparql/sparql12/` (draft)
- **RDF Parsing**: `jena-arq/testing/rdf-tests-cg/rdf/` (2,264+ files)
- **SPARQL-CDTs**: `jena-arq/testing/SPARQL-CDTs/` (707 files)
- **ARQ Tests**: `jena-arq/testing/ARQ/` (1,201 files)

---

## Next Actions

1. **Create Test Harness** (Week 1)
   - Implement manifest parser (`crates/sparql/tests/w3c_conformance/manifest.rs`)
   - Build test runner (`harness.rs`)
   - Set up CI/CD integration

2. **Port Priority 0 Tests** (Weeks 1-4)
   - Start with `rdf-model` tests (60 files)
   - Move to `storage` tests (46 files)
   - Port basic SPARQL tests (151 files)
   - Port RDF I/O tests (42 files)

3. **Measure Baseline** (Week 1)
   - Run current rust-kgdb against W3C DAWG tests
   - Document pass/fail rates per category
   - Create tracking dashboard

4. **Weekly Progress Reports**
   - Track tests ported vs total
   - Report pass rates by category
   - Identify common failure patterns

5. **Feature Parity Milestones**
   - **Milestone 1** (Week 4): Basic SPARQL queries working
   - **Milestone 2** (Week 8): SPARQL 1.1 query features complete
   - **Milestone 3** (Week 12): SPARQL UPDATE + persistence
   - **Milestone 4** (Week 16): Reasoning + validation

---

## Summary

Apache Jena provides **1,918 production-grade test files** plus **4,940+ W3C conformance tests**, covering every aspect of RDF/SPARQL implementation. By systematically porting these tests to rust-kgdb, we will achieve:

1. **Feature Parity**: Match Apache Jena's SPARQL 1.1 support
2. **W3C Compliance**: Pass official conformance suites
3. **Production Readiness**: Confidence from exhaustive testing
4. **Mobile Optimization**: Zero-copy semantics for iOS/Android

**Estimated Timeline**: 16 weeks to 87% test coverage + 70% W3C conformance.

**Key Success Metrics**:
- 480+ Jena tests ported
- 1,045+ W3C tests passing
- 70%+ DAWG conformance
- 60%+ SPARQL 1.1 conformance
- 100% mobile FFI test coverage

This analysis provides the complete roadmap to transform rust-kgdb into a production-ready, W3C-compliant, mobile-first RDF database.
