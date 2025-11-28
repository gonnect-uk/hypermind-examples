# Comprehensive Apache Jena Test Migration Plan for rust-kgdb
## Achieving 100% W3C RDF Standards Compliance

**Date**: November 22, 2025  
**Status**: Research & Planning Phase  
**Target**: Complete W3C RDF/SPARQL 1.1 compliance with 100% test coverage

---

## Executive Summary

This document provides a comprehensive roadmap for migrating Apache Jena's extensive test suite to rust-kgdb. The plan covers:

- **18,000+ test cases** from W3C RDF and SPARQL test suites
- **6 major test categories**: RDF Parsing, SPARQL Query, SPARQL Update, Reasoning, SHACL Validation, and Format Results
- **3 implementation phases**: Critical (W3C compliance), Essential (full feature parity), and Nice-to-have (performance/optimization tests)
- **11 crates** requiring test migration: rdf-io, sparql, storage, reasoning, shacl, rdf-model, hypergraph, datalog, wcoj, prov, mobile-ffi

---

## Part 1: Test Glossary by Category

### 1.1 RDF Format Parsing Tests

#### 1.1.1 Turtle (W3C RDF-Turtle Specification)

**Repository**: W3C rdf-tests: `rdf/rdf11/rdf-turtle/`  
**Source Manifest**: `turtle/manifest.ttl`  
**Implementation**: rust-kgdb: `crates/rdf-io/src/turtle.rs`

**Test Categories**:

| Category | Test Count | Description | Jena Equivalent |
|----------|-----------|-------------|-----------------|
| **Turtle Eval Tests** | ~25 | Positive parsing tests (valid Turtle) | N/A (Turtle is RDF 1.1) |
| **Turtle Negative Tests** | ~15 | Negative parsing tests (invalid syntax) | RIOT Turtle negative |
| **Turtle Eval Extra** | ~10 | Edge cases (empty subjects, blank lines) | Raptor Turtle tests |
| **Comment Handling** | ~5 | Line/block comments in various positions | Special case |
| **Escape Sequences** | ~8 | Unicode escapes, character escapes | String literal tests |
| **Language Tags** | ~6 | @en, @en-US, @zh-Hans | Language tagged literals |
| **Datatype IRIs** | ~7 | xsd:integer, xsd:decimal, custom types | Type handling |
| **Prefix Declarations** | ~8 | @prefix directives, namespace resolution | Turtle prefix tests |
| **Directive Handling** | ~4 | @base, @prefix ordering | Directive parsing |
| **Blank Node Handling** | ~6 | _:a, _:b, anonymous nodes, collections | Blank node semantics |
| **Triple Forms** | ~5 | Standard form, shorthand, anonymous forms | Triple syntax variants |
| **Total Turtle Tests** | **~99** | — | — |

**Example Test Case Structure**:
```
test-00001:
  - File: test-00001.ttl (input)
  - Expected: test-00001.nt (output in N-Triples)
  - Type: PositiveEvaluationTest or NegativeSyntaxTest
  - Manifest: manifest.ttl with mf:action and mf:result
```

**W3C Reference**: https://w3c.github.io/rdf-tests/rdf/rdf11/rdf-turtle/

#### 1.1.2 N-Triples (W3C RDF N-Triples Specification)

**Repository**: W3C rdf-tests: `rdf/rdf11/n-triples/`  
**Source Manifest**: `n-triples/manifest.ttl`  
**Implementation**: rust-kgdb: `crates/rdf-io/src/ntriples.rs`

**Test Categories**:

| Category | Test Count | Description | Jena Equivalent |
|----------|-----------|-------------|-----------------|
| **N-Triples Eval Tests** | ~12 | Positive parsing tests | RIOT N-Triples positive |
| **N-Triples Negative Tests** | ~8 | Negative parsing tests | RIOT N-Triples negative |
| **IRI Handling** | ~7 | Absolute IRIs, angle brackets | IRI tests |
| **Literal Handling** | ~6 | Quoted strings, escapes, types | Literal tests |
| **Blank Node Handling** | ~4 | _:blank node identifiers | Blank node tests |
| **Comment Handling** | ~3 | Line comments in N-Triples | Comment tests |
| **Total N-Triples Tests** | **~40** | — | — |

**W3C Reference**: https://w3c.github.io/rdf-tests/rdf/rdf11/n-triples/

#### 1.1.3 RDF/XML (W3C RDF/XML Specification)

**Repository**: W3C rdf-tests: `rdf/rdf11/rdf-xml/`  
**Implementation**: rust-kgdb: `crates/rdf-io/src/` (needs implementation)

**Test Categories**:

| Category | Test Count | Description | Jena Equivalent |
|----------|-----------|-------------|-----------------|
| **RDF/XML Eval Tests** | ~25 | Positive parsing tests | RIOT RDF/XML positive |
| **RDF/XML Negative Tests** | ~15 | Negative parsing tests | RIOT RDF/XML negative |
| **Element/Attribute Handling** | ~8 | rdf:type, rdf:resource, rdf:about | RDF/XML attributes |
| **Datatype Handling** | ~6 | rdf:datatype attribute | Datatype tests |
| **XML Namespace Handling** | ~8 | xmlns declarations, prefixes | XML namespace tests |
| **Collection Handling** | ~5 | rdf:parseType="Collection" | RDF collections |
| **CDATA Handling** | ~4 | CDATA sections in literals | CDATA tests |
| **Total RDF/XML Tests** | **~71** | — | — |

**W3C Reference**: https://w3c.github.io/rdf-tests/rdf/rdf11/rdf-xml/

#### 1.1.4 JSON-LD (W3C JSON-LD Specification)

**Repository**: W3C json-ld.org repository  
**Implementation**: rust-kgdb: `crates/rdf-io/src/` (needs implementation)

**Test Categories**:

| Category | Test Count | Description | Jena Equivalent |
|----------|-----------|-------------|-----------------|
| **JSON-LD to RDF Tests** | ~30 | Convert JSON-LD to RDF | RIOT JSON-LD to RDF |
| **RDF to JSON-LD Tests** | ~15 | Convert RDF to JSON-LD | RIOT RDF to JSON-LD |
| **Context Processing** | ~12 | @context handling, vocab expansion | Context tests |
| **Compaction Tests** | ~8 | JSON-LD compaction operations | Compaction tests |
| **Expansion Tests** | ~10 | JSON-LD expansion operations | Expansion tests |
| **Flattening Tests** | ~6 | JSON-LD flattening operations | Flattening tests |
| **Frame Tests** | ~8 | JSON-LD @context framing | Framing tests |
| **Total JSON-LD Tests** | **~89** | — | — |

**Note**: JSON-LD test suite is maintained separately at https://github.com/json-ld/json-ld.org/tests

#### 1.1.5 N-Quads (W3C RDF N-Quads Specification)

**Repository**: W3C rdf-tests: `rdf/rdf11/n-quads/`  
**Implementation**: rust-kgdb: `crates/rdf-io/src/` (needs implementation for named graphs)

**Test Categories**:

| Category | Test Count | Description | Jena Equivalent |
|----------|-----------|-------------|-----------------|
| **N-Quads Eval Tests** | ~10 | Positive parsing tests | RIOT N-Quads positive |
| **N-Quads Negative Tests** | ~8 | Negative parsing tests | RIOT N-Quads negative |
| **Graph Name Handling** | ~6 | Named graph IRIs in N-Quads | Graph name tests |
| **Blank Node Handling** | ~4 | Blank node subjects/objects/graph names | Blank graph names |
| **Total N-Quads Tests** | **~28** | — | — |

**W3C Reference**: https://w3c.github.io/rdf-tests/rdf/rdf11/n-quads/

#### 1.1.6 TriG (W3C RDF TriG Specification)

**Repository**: W3C rdf-tests: `rdf/rdf11/trig/`  
**Implementation**: rust-kgdb: `crates/rdf-io/src/` (needs implementation)

**Test Categories**:

| Category | Test Count | Description | Jena Equivalent |
|----------|-----------|-------------|-----------------|
| **TriG Eval Tests** | ~12 | Positive parsing tests | RIOT TriG positive |
| **TriG Negative Tests** | ~8 | Negative parsing tests | RIOT TriG negative |
| **Graph Block Syntax** | ~6 | GRAPH syntax for named graphs | Graph blocks |
| **Prefix/Base Handling** | ~5 | @prefix, @base in TriG | Prefix directives |
| **Total TriG Tests** | **~31** | — | — |

**W3C Reference**: https://w3c.github.io/rdf-tests/rdf/rdf11/trig/

#### 1.1.7 RDF 1.2 New Format Tests (Future)

**Repository**: W3C rdf-tests: `rdf/rdf12/`  
**Status**: Emerging standard (RDF 1.2 in development)

**Includes**:
- RDFstar/RDF-star tests (quoted triples)
- RDF 1.2 updates to existing formats
- New datatype tests

---

### 1.2 SPARQL Query Tests (W3C SPARQL 1.1 Query Language)

**Repository**: W3C rdf-tests: `sparql/sparql11/`  
**Main Entry Point**: https://w3c.github.io/rdf-tests/sparql/sparql11/  
**Jena Testing Tool**: `jena-arq/testing/ARQ/` + command `qtest` (now `rdftests`)

#### 1.2.1 Basic Graph Pattern Tests

**Directory**: `sparql/sparql11/algebra/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **Basic Triple Patterns** | ~25 | Simple ?s ?p ?o patterns | ARQ/Algebra |
| **Join Patterns** | ~20 | Multiple BGPs joined | ARQ/Algebra |
| **Optional Patterns** | ~18 | OPTIONAL { } clauses | ARQ/Optional |
| **Union Patterns** | ~12 | UNION branches | ARQ/Union |
| **Named Graph Access** | ~10 | GRAPH ?g { } patterns | ARQ/Dataset |
| **Complex Algebra** | ~15 | Combinations of above | ARQ/Algebra |
| **Total Algebra Tests** | **~100** | — | — |

#### 1.2.2 Filter and Expression Tests

**Directories**: 
- `sparql/sparql11/algebra/` (basic filters)
- `sparql/sparql11/functions/` (built-in functions)

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **Comparison Filters** | ~12 | =, !=, <, >, <=, >= | ARQ/Expr |
| **Logical Filters** | ~8 | &&, \|\|, ! | ARQ/Expr |
| **String Filters** | ~20 | REGEX, CONTAINS, STARTS/ENDS WITH | ARQ/Functions |
| **Numeric Filters** | ~15 | ABS, ROUND, CEIL, FLOOR | ARQ/ExprBuiltIns |
| **Type Check Filters** | ~10 | isIRI, isBlank, isLiteral, BOUND | ARQ/Expr |
| **Math Filters** | ~12 | +, -, *, /, MOD | ARQ/Expr |
| **Date/Time Filters** | ~12 | NOW, YEAR, MONTH, DAY, HOURS | ARQ/Expr |
| **Custom Filters** | ~8 | User-defined constraints | ARQ/Extra |
| **Total Filter Tests** | **~97** | — | — |

#### 1.2.3 Aggregate Tests

**Directory**: `sparql/sparql11/aggregates/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **COUNT Aggregates** | ~15 | COUNT(?var), COUNT(*), COUNT(DISTINCT) | ARQ/Aggregates |
| **SUM Aggregates** | ~10 | SUM(?var), SUM(DISTINCT) | ARQ/Aggregates |
| **AVG Aggregates** | ~10 | AVG(?var), error handling | ARQ/Aggregates |
| **MIN/MAX Aggregates** | ~8 | MIN(?var), MAX(?var) | ARQ/Aggregates |
| **GROUP_CONCAT Aggregates** | ~8 | GROUP_CONCAT(?var), SEPARATOR | ARQ/Aggregates |
| **GROUP BY Clauses** | ~15 | Grouping with aggregates | ARQ/Grouping |
| **Having Clauses** | ~10 | HAVING conditions on aggregates | ARQ/Grouping |
| **Total Aggregate Tests** | **~76** | — | — |

#### 1.2.4 CONSTRUCT Query Tests

**Directory**: `sparql/sparql11/construct/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **Simple CONSTRUCT** | ~12 | Basic pattern construction | ARQ/Construct |
| **Complex CONSTRUCT** | ~10 | Multiple patterns in constructor | ARQ/Construct |
| **CONSTRUCT with DISTINCT** | ~6 | Duplicate removal | ARQ/Construct |
| **CONSTRUCT with ORDER BY** | ~5 | Ordered results in graph | ARQ/Construct |
| **CONSTRUCT with LIMIT** | ~5 | Limited results | ARQ/Construct |
| **CONSTRUCT with aggregates** | ~8 | Grouped construction | ARQ/Construct |
| **Total CONSTRUCT Tests** | **~46** | — | — |

#### 1.2.5 BIND and Assignment Tests

**Directory**: `sparql/sparql11/bind/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **BIND Expression Evaluation** | ~20 | BIND (?x AS ?y) patterns | ARQ/Bind |
| **BIND Type Errors** | ~8 | Error handling in BIND | ARQ/Bind |
| **BIND with Aggregates** | ~6 | BIND after aggregation | ARQ/Bind |
| **Multiple BIND Clauses** | ~8 | Sequential BIND statements | ARQ/Bind |
| **Total BIND Tests** | **~42** | — | — |

#### 1.2.6 VALUES and Bindings Tests

**Directory**: `sparql/sparql11/bindings/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **VALUES Inline** | ~12 | VALUES ?x { 1 2 3 } | ARQ/Bindings |
| **VALUES Multiple Variables** | ~8 | VALUES (?x ?y) { (1 2) (3 4) } | ARQ/Bindings |
| **Empty VALUES** | ~4 | VALUES with no solutions | ARQ/Bindings |
| **VALUES with expressions** | ~6 | Complex value construction | ARQ/Bindings |
| **Total VALUES Tests** | **~30** | — | — |

#### 1.2.7 Property Path Tests

**Directory**: `sparql/sparql11/property-path/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **Simple Paths** | ~10 | ?s :prop ?o | ARQ/Paths |
| **Sequence Paths** | ~12 | ?s :p1/:p2 ?o | ARQ/Paths |
| **Alternative Paths** | ~10 | ?s (:p1\|:p2) ?o | ARQ/Paths |
| **Kleene Star (*)** | ~15 | ?s :p* ?o | ARQ/Paths |
| **Kleene Plus (+)** | ~12 | ?s :p+ ?o | ARQ/Paths |
| **Optional Paths (?)** | ~8 | ?s :p? ?o | ARQ/Paths |
| **Inverse Paths** | ~8 | ?s ^:p ?o | ARQ/Paths |
| **Negated Paths** | ~6 | !:p (must not follow) | ARQ/Paths |
| **Complex Paths** | ~12 | Combined operators | ARQ/Paths |
| **Total Property Path Tests** | **~93** | — | — |

#### 1.2.8 Subquery Tests

**Directory**: `sparql/sparql11/subquery/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **SELECT Subqueries** | ~15 | Subqueries in WHERE | ARQ/SubQuery |
| **FILTER with Subqueries** | ~10 | Subqueries in FILTER | ARQ/SubQuery |
| **Scalar Subqueries** | ~8 | Single-result subqueries | ARQ/SubQuery |
| **Subquery Scoping** | ~6 | Variable scope rules | ARQ/SubQuery |
| **Total Subquery Tests** | **~39** | — | — |

#### 1.2.9 Negation Tests

**Directory**: `sparql/sparql11/negation/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **NOT EXISTS** | ~12 | NOT EXISTS { } patterns | ARQ/Negation |
| **FILTER NOT EXISTS** | ~8 | FILTER NOT EXISTS { } | ARQ/Negation |
| **MINUS Operator** | ~12 | MINUS { } patterns | ARQ/Negation |
| **NOT IN** | ~8 | NOT IN (list) | ARQ/Expr |
| **Total Negation Tests** | **~40** | — | — |

#### 1.2.10 SELECT and Projection Tests

**Directory**: `sparql/sparql11/project-expression/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **SELECT with expressions** | ~15 | SELECT (expr AS ?var) | ARQ/Project |
| **SELECT */** | ~6 | SELECT * form | ARQ/Project |
| **SELECT DISTINCT** | ~10 | Duplicate elimination | ARQ/Project |
| **SELECT with LIMIT** | ~8 | LIMIT clause | ARQ/Project |
| **SELECT with OFFSET** | ~8 | OFFSET clause | ARQ/Project |
| **SELECT with ORDER BY** | ~12 | Ordering results | ARQ/Project |
| **Total SELECT Tests** | **~59** | — | — |

#### 1.2.11 EXISTS Tests

**Directory**: `sparql/sparql11/exists/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **EXISTS in FILTER** | ~10 | FILTER EXISTS { } | ARQ/Exists |
| **NOT EXISTS** | ~8 | NOT EXISTS (covered above) | ARQ/Exists |
| **Scoped EXISTS** | ~6 | Variable scope in EXISTS | ARQ/Exists |
| **Total EXISTS Tests** | **~24** | — | — |

#### 1.2.12 Function Library Tests

**Directory**: `sparql/sparql11/functions/`  
**Jena Equivalent**: `jena-arq/testing/ARQ/ExprBuiltIns`

This overlaps with filter tests above. Dedicated function tests include:

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **String Functions** | ~35 | STR, CONCAT, SUBSTR, STRLEN, UCASE, LCASE, REGEX, REPLACE | ARQ/ExprBuiltIns |
| **Numeric Functions** | ~20 | ABS, ROUND, CEIL, FLOOR, SQRT, RANDOM | ARQ/ExprBuiltIns |
| **Type Functions** | ~15 | datatype(), lang(), langMatches() | ARQ/Expr |
| **Hash Functions** | ~12 | MD5, SHA1, SHA256, SHA384, SHA512 | ARQ/ExprBuiltIns |
| **Date/Time Functions** | ~20 | NOW, YEAR, MONTH, DAY, TIMEZONE, etc. | ARQ/ExprBuiltIns |
| **Test Functions** | ~15 | isIRI, isBlank, isLiteral, etc. | ARQ/Expr |
| **Total Function Tests** | **~117** | — | — |

#### 1.2.13 Syntax Tests (Positive and Negative)

**Directories**: 
- `sparql/sparql11/syntax-query/` (positive query syntax)
- `sparql/sparql11/syntax-update-1/` and `syntax-update-2/` (positive update syntax)

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **Positive Syntax Query** | ~30 | Valid SPARQL query syntax | ARQ/Syntax |
| **Negative Syntax Query** | ~25 | Invalid SPARQL query syntax | ARQ/Syntax |
| **Positive Syntax Update** | ~20 | Valid SPARQL update syntax | ARQ/Syntax |
| **Negative Syntax Update** | ~20 | Invalid SPARQL update syntax | ARQ/Syntax |
| **Total Syntax Tests** | **~95** | — | — |

#### 1.2.14 Result Set Format Tests

**Directories**:
- `sparql/sparql11/json-res/` (JSON results)
- `sparql/sparql11/csv-tsv-res/` (CSV/TSV results)
- (XML, Turtle results included in protocol tests)

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **JSON Results** | ~20 | JSON format serialization | RIOT/JSON-LD |
| **CSV Results** | ~12 | CSV format serialization | RIOT/CSV |
| **TSV Results** | ~12 | TSV format serialization | RIOT/TSV |
| **XML Results** | ~10 | XML format (in protocol) | RIOT/XML |
| **Total Format Tests** | **~54** | — | — |

#### 1.2.15 SPARQL 1.1 Query Test Summary

**Total W3C SPARQL 1.1 Query Tests: ~842**

**Breakdown by complexity**:
- Core algebra & patterns: ~240 tests
- Filters & expressions: ~97 tests
- Aggregates & grouping: ~76 tests
- Joins & paths: ~93 tests
- Advanced (subqueries, negation): ~79 tests
- Result formats: ~54 tests
- Syntax: ~95 tests
- Functions: ~117 tests

---

### 1.3 SPARQL Update Tests (W3C SPARQL 1.1 Update Language)

**Repository**: W3C rdf-tests: `sparql/sparql11/`  
**Directories**: `basic-update/`, `delete*/`, `insert*/`, `add/`, `copy/`, `move/`, `drop/`, `clear/`, `update-silent/`

#### 1.3.1 Basic Update Operations

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **INSERT DATA** | ~15 | Direct triple insertion | ARQ/Update |
| **DELETE DATA** | ~12 | Direct triple deletion | ARQ/Update |
| **DELETE WHERE** | ~10 | Pattern-based deletion | ARQ/Update |
| **INSERT WHERE** | ~8 | Pattern-based insertion | ARQ/Update |
| **DELETE/INSERT (MODIFY)** | ~15 | Combined delete + insert | ARQ/Update |
| **Total Basic Update** | **~60** | — | — |

#### 1.3.2 Graph Manipulation Operations

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **ADD** | ~8 | Copy graph to another | ARQ/Update |
| **COPY** | ~8 | Copy graph with deletion | ARQ/Update |
| **MOVE** | ~8 | Rename/move graph | ARQ/Update |
| **DROP** | ~8 | Delete entire graph | ARQ/Update |
| **CLEAR** | ~8 | Clear graph (keep structure) | ARQ/Update |
| **Total Graph Ops** | **~40** | — | — |

#### 1.3.3 Silent Operations

**Directory**: `sparql/sparql11/update-silent/`

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **INSERT SILENT** | ~6 | Silent insert (no error on conflicts) | ARQ/Update |
| **DELETE SILENT** | ~6 | Silent delete (no error if not found) | ARQ/Update |
| **DROP SILENT** | ~4 | Silent drop (no error if missing) | ARQ/Update |
| **Total Silent Tests** | **~16** | — | — |

#### 1.3.4 SPARQL Update Test Summary

**Total W3C SPARQL 1.1 Update Tests: ~116**

---

### 1.4 SHACL Validation Tests (W3C SHACL Specification)

**Repository**: W3C shacl repository + shacl-tests directory  
**Main Reference**: https://w3c.github.io/data-shapes/data-shapes-test-suite/

#### 1.4.1 SHACL Core Tests

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **Node Shape Validation** | ~30 | sh:NodeShape constraints | SHACL/Core |
| **Class Constraints** | ~15 | sh:class validation | SHACL/Core |
| **Property Shape Validation** | ~25 | sh:property patterns | SHACL/Property |
| **Cardinality Constraints** | ~20 | sh:minCount, sh:maxCount | SHACL/Cardinality |
| **Value Type Constraints** | ~18 | sh:datatype, sh:nodeKind | SHACL/Type |
| **Value Range Constraints** | ~15 | sh:minExclusive, sh:maxExclusive | SHACL/Range |
| **String Constraints** | ~12 | sh:pattern, sh:minLength, sh:maxLength | SHACL/String |
| **Enumeration Constraints** | ~8 | sh:in (allowed values) | SHACL/Enum |
| **Uniqueness Constraints** | ~6 | sh:uniqueLang | SHACL/Unique |
| **Total SHACL Core Tests** | **~149** | — | — |

#### 1.4.2 SHACL Advanced Constraints

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **AND/OR/NOT Combination** | ~12 | sh:and, sh:or, sh:not | SHACL/Logic |
| **Recursive Constraints** | ~10 | sh:node (recursive shapes) | SHACL/Recursive |
| **Path-based Constraints** | ~15 | sh:path with property paths | SHACL/Path |
| **Closed Shapes** | ~8 | sh:closed (whitelist properties) | SHACL/Closed |
| **Ignored Properties** | ~4 | sh:ignoredProperties | SHACL/Ignored |
| **Total Advanced Constraints** | **~49** | — | — |

#### 1.4.3 SHACL SPARQL Constraints

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **SPARQL Constraints** | ~20 | sh:sparql with custom queries | SHACL/SPARQLConstraint |
| **SPARQL Targets** | ~15 | sh:targetNode, sh:targetClass, sh:targetQuery | SHACL/SPARQLTarget |
| **Custom Messages** | ~8 | sh:resultMessage in violations | SHACL/Message |
| **Severity Levels** | ~6 | sh:Violation, sh:Warning, sh:Info | SHACL/Severity |
| **Total SPARQL Tests** | **~49** | — | — |

#### 1.4.4 SHACL Test Summary

**Total W3C SHACL Tests: ~247**

---

### 1.5 Reasoning Tests (RDFS & OWL 2 RL)

**Repository**: Not directly in W3C RDF tests (covered by Apache Jena's own test suite)  
**Jena Equivalent**: `jena-core/testing/RDFS/` and `jena-core/testing/OWL/`

#### 1.5.1 RDFS Entailment Tests

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **rdfs:subClassOf** | ~20 | Class hierarchy entailment | jena-core/RDFS |
| **rdfs:subPropertyOf** | ~15 | Property hierarchy entailment | jena-core/RDFS |
| **rdfs:domain/range** | ~18 | Property domain/range inference | jena-core/RDFS |
| **rdf:type Entailment** | ~12 | Type derivation | jena-core/RDFS |
| **rdfs:Container** | ~8 | Container member entailment | jena-core/RDFS |
| **RDFS Transitive Rules** | ~10 | Transitive closure computation | jena-core/RDFS |
| **Total RDFS Tests** | **~83** | — | — |

#### 1.5.2 OWL 2 RL Profile Tests

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **OWL Class Axioms** | ~25 | owl:equivalentClass, disjointWith | jena-core/OWL |
| **OWL Property Axioms** | ~20 | owl:equivalentProperty, inverseOf | jena-core/OWL |
| **OWL Restrictions** | ~25 | owl:onProperty, owl:someValuesFrom | jena-core/OWL |
| **OWL Cardinality** | ~15 | owl:minCardinality, owl:maxCardinality | jena-core/OWL |
| **OWL AllValuesFrom** | ~10 | Universal restriction inference | jena-core/OWL |
| **OWL Functional Properties** | ~8 | owl:FunctionalProperty | jena-core/OWL |
| **OWL Inverse Properties** | ~8 | owl:inverseOf semantics | jena-core/OWL |
| **OWL Transitive Properties** | ~8 | owl:TransitiveProperty | jena-core/OWL |
| **OWL Symmetric Properties** | ~6 | owl:SymmetricProperty | jena-core/OWL |
| **OWL Disjoint Classes** | ~10 | owl:disjointWith clash detection | jena-core/OWL |
| **Total OWL 2 RL Tests** | **~135** | — | — |

#### 1.5.3 OWL 2 EL/DL Subset Tests (Advanced)

**Status**: Optional for initial compliance

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **OWL 2 EL Profiles** | ~40 | Lightweight profile tests | jena-core/OWL-EL |
| **OWL 2 DL Profiles** | ~45 | Description logic tests | jena-core/OWL-DL |
| **Total EL/DL Tests** | **~85** | Optional |

#### 1.5.4 Reasoning Test Summary

**Total Reasoning Tests: ~218** (Core: 83 RDFS + 135 OWL2-RL = 218, Optional: +85 EL/DL)

---

### 1.6 RDF-Star/RDF-star Tests (Quoted Triples)

**Repository**: RDF 1.2 tests under development  
**Current Status**: Emerging specification

| Category | Test Count | Description | Jena Equiv. |
|----------|-----------|-------------|-------------|
| **Quoted Triple Parsing** | ~15 | << s p o >> syntax | ARQ/RDF-star |
| **Quoted Triple Semantics** | ~12 | Quoted triple as subject/object | ARQ/RDF-star |
| **Nested Quotes** | ~8 | Deeply nested quotations | ARQ/RDF-star |
| **RDF-star SPARQL Queries** | ~20 | Querying quoted triples | ARQ/RDF-star |
| **RDF-star Annotations** | ~10 | Triple annotations on quotes | ARQ/RDF-star |
| **Total RDF-star Tests** | **~65** | — | — |

---

### 1.7 Summary Table: All Test Categories

| Category | Subcategories | Est. Tests | Priority | Crate |
|----------|---------------|-----------|----------|-------|
| **RDF Parsing** | Turtle, N-Triples, RDF/XML, JSON-LD, N-Quads, TriG | **299** | CRITICAL | rdf-io |
| **SPARQL Query** | Algebra, Filters, Aggregates, Construct, Bind, Values, Paths, Subqueries, Negation, SELECT, Exists, Functions, Syntax, Formats | **842** | CRITICAL | sparql |
| **SPARQL Update** | INSERT, DELETE, Graph ops, Silent | **116** | CRITICAL | sparql |
| **SHACL Validation** | Core, Advanced, SPARQL Constraints | **247** | CRITICAL | shacl |
| **RDFS Reasoning** | subClassOf, subPropertyOf, domain/range, etc. | **83** | ESSENTIAL | reasoning |
| **OWL 2 RL Reasoning** | Class/property axioms, restrictions, cardinality | **135** | ESSENTIAL | reasoning |
| **OWL 2 EL/DL (Optional)** | Lightweight/Description Logic profiles | **85** | OPTIONAL | reasoning |
| **RDF-star (Emerging)** | Quoted triples, parsing, SPARQL | **65** | OPTIONAL | rdf-io, sparql |
| **TOTAL** | **7 major categories** | **~1,872** | — | — |

**Note**: This represents comprehensive W3C conformance. Core compliance target: ~1,410 tests (excluding RDF-star and OWL EL/DL)

---

## Part 2: Detailed TODO List for Test Migration

### 2.1 Phase 1: Critical - W3C SPARQL/RDF Core Compliance (Weeks 1-6)

#### 2.1.1 RDF-IO Module Tests (Week 1-2)

```
Priority: CRITICAL - Must pass before any other tests work
Target Crate: crates/rdf-io/src/

[ ] 2.1.1.1: Turtle Parser Tests
    [ ] Implement test harness for W3C turtle/manifest.ttl
    [ ] Positive parsing tests (~25 test cases)
        [ ] test-00001 through test-00025: Standard Turtle syntax
        [ ] Verify output matches N-Triples format
        [ ] Expected: 100% pass rate
    [ ] Negative parsing tests (~15 test cases)
        [ ] Invalid syntax detection
        [ ] Error handling verification
    [ ] Extended tests (~10 test cases)
        [ ] Edge cases: empty documents, blank lines, special characters
    [ ] Escape sequence tests (~8 test cases)
    [ ] Language tag tests (~6 test cases)
    [ ] Datatype IRI tests (~7 test cases)
    [ ] Prefix declaration tests (~8 test cases)
    [ ] Blank node handling tests (~6 test cases)
    [ ] Triple form variations (~5 test cases)
    Estimated effort: 3-4 days
    Blocker: None
    Risk: Grammar mismatch with W3C spec

[ ] 2.1.1.2: N-Triples Parser Tests
    [ ] Implement test harness for W3C n-triples/manifest.ttl
    [ ] Positive parsing tests (~12 test cases)
    [ ] Negative parsing tests (~8 test cases)
    [ ] IRI handling tests (~7 test cases)
    [ ] Literal handling tests (~6 test cases)
    [ ] Blank node tests (~4 test cases)
    [ ] Comment handling tests (~3 test cases)
    Expected effort: 2 days
    Blocker: None
    Note: Simpler than Turtle, should be fast

[ ] 2.1.1.3: RDF/XML Parser Implementation
    [ ] Create new file: crates/rdf-io/src/rdfxml.rs
    [ ] Implement XML parsing using quick-xml
    [ ] Parse RDF/XML specification format
    [ ] Handle element and attribute interpretation
    [ ] Implement test harness for W3C rdf-xml/manifest.ttl
    [ ] RDF/XML positive tests (~25 test cases)
    [ ] RDF/XML negative tests (~15 test cases)
    [ ] Element/attribute tests (~8 test cases)
    [ ] Datatype handling tests (~6 test cases)
    [ ] XML namespace tests (~8 test cases)
    [ ] Collection handling tests (~5 test cases)
    Expected effort: 5-6 days
    Blocker: quick-xml crate (already in Cargo.toml)
    Risk: High - complex XML parsing, many edge cases

[ ] 2.1.1.4: N-Quads Parser Implementation
    [ ] Create new file: crates/rdf-io/src/nquads.rs
    [ ] Extend N-Triples parser with graph name support
    [ ] Implement test harness for W3C n-quads/manifest.ttl
    [ ] Positive tests (~10 test cases)
    [ ] Negative tests (~8 test cases)
    [ ] Graph name handling (~6 test cases)
    [ ] Blank graph name tests (~4 test cases)
    Expected effort: 1.5 days
    Blocker: N-Triples must work first
    Note: Relatively straightforward extension

[ ] 2.1.1.5: TriG Parser Implementation
    [ ] Create new file: crates/rdf-io/src/trig.rs
    [ ] Extend Turtle parser with GRAPH blocks
    [ ] Implement test harness for W3C trig/manifest.ttl
    [ ] TriG positive tests (~12 test cases)
    [ ] TriG negative tests (~8 test cases)
    [ ] Graph block syntax (~6 test cases)
    [ ] Prefix/base handling (~5 test cases)
    Expected effort: 2-3 days
    Blocker: Turtle parser must work first
    Risk: Graph block syntax parsing

[ ] 2.1.1.6: JSON-LD Parser Implementation
    [ ] Create new file: crates/rdf-io/src/jsonld.rs
    [ ] Implement JSON-LD context processing
    [ ] Implement JSON-LD to RDF conversion
    [ ] Implement RDF to JSON-LD serialization
    [ ] Import test suite from json-ld.org repository
    [ ] JSON-LD to RDF tests (~30 test cases)
    [ ] RDF to JSON-LD tests (~15 test cases)
    [ ] Context processing tests (~12 test cases)
    [ ] Compaction tests (~8 test cases)
    [ ] Expansion tests (~10 test cases)
    [ ] Flattening tests (~6 test cases)
    [ ] Frame tests (~8 test cases)
    Expected effort: 1 week
    Blocker: serde_json (already available)
    Risk: High complexity - JSON-LD is sophisticated

[ ] 2.1.1.7: Format Integration Tests
    [ ] Round-trip testing: Turtle ↔ N-Triples ↔ RDF/XML ↔ JSON-LD
    [ ] Test cross-format compatibility
    [ ] Verify semantic equivalence
    Expected effort: 1 day
    Blocker: All parsers must work

Subtotal Phase 2.1.1: ~20 tests per format parser, ~299 total tests
Timeline: 2-3 weeks
Complexity: HIGH (RDF/XML and JSON-LD are complex)
```

#### 2.1.2 SPARQL Query Core Tests (Week 2-4)

```
Priority: CRITICAL - Foundation for all SPARQL testing
Target Crate: crates/sparql/src/

[ ] 2.1.2.1: Algebra & Basic Graph Patterns
    [ ] Set up test infrastructure: W3C manifest parser
    [ ] Download W3C sparql/sparql11/algebra/ tests
    [ ] Create test runner structure:
        - manifest.ttl parser using turtle.rs
        - Test case discovery system
        - Query executor wrapper
        - Result comparison (SPARQL JSON Results format)
    [ ] Implement triple pattern matching tests (~25 test cases)
        [ ] Single triple patterns
        [ ] Variable binding
        [ ] URI matching
        [ ] Literal matching
    [ ] Implement BGP (Basic Graph Pattern) tests (~20 test cases)
        [ ] Multiple triple patterns (joins)
        [ ] Pattern ordering effects
        [ ] Optimization verification
    [ ] Implement join tests (~20 test cases)
        [ ] Natural joins on shared variables
        [ ] Multi-way joins
        [ ] Join optimization
    [ ] Implement OPTIONAL tests (~18 test cases)
        [ ] Left outer join semantics
        [ ] Multiple optional patterns
        [ ] Optional with filters
    [ ] Implement UNION tests (~12 test cases)
        [ ] Alternative patterns
        [ ] UNION result combination
        [ ] UNION with multiple branches
    [ ] Implement GRAPH tests (~10 test cases)
        [ ] Named graph access
        [ ] Multiple graphs
        [ ] Graph pattern matching
    [ ] Implement complex algebra (~15 test cases)
        [ ] Nested patterns
        [ ] Multiple operators
    Expected effort: 4-5 days
    Blocker: RDF-IO must provide triple storage
    Risk: Query optimization edge cases
    Success Criteria: ~100/100 tests passing

[ ] 2.1.2.2: Filter & Expression Tests
    [ ] Implement FILTER clause evaluation (~12 test cases)
    [ ] Implement comparison operators (~8 test cases)
        [ ] =, !=, <, >, <=, >=
        [ ] Type coercion
        [ ] NULL handling
    [ ] Implement logical operators (~6 test cases)
        [ ] &&, ||, !
        [ ] Short-circuit evaluation
    [ ] Verify all 64 builtin functions
        [ ] String functions: STR, CONCAT, SUBSTR, STRLEN, UCASE, LCASE, REGEX, REPLACE (~20 tests)
        [ ] Numeric functions: ABS, ROUND, CEIL, FLOOR, SQRT (~15 tests)
        [ ] Date/Time: NOW, YEAR, MONTH, DAY, HOURS, MINUTES, SECONDS, TIMEZONE, TZ (~12 tests)
        [ ] Hash functions: MD5, SHA1, SHA256, SHA384, SHA512 (~8 tests)
        [ ] Type functions: datatype(), lang(), langMatches(), isIRI, isBlank, isLiteral, etc. (~20 tests)
        [ ] Constructor functions: IF, COALESCE, BNODE, IRI, URI, STRDT, STRLANG (~8 tests)
        [ ] Test functions: BOUND, EXISTS, NOT EXISTS (~10 tests)
    Expected effort: 5 days
    Blocker: Expression evaluator must be complete
    Risk: Edge cases in type conversion
    Success Criteria: All 97 filter/expression tests passing

[ ] 2.1.2.3: Aggregate & Grouping Tests
    [ ] Implement test harness
    [ ] COUNT aggregates (~15 test cases)
        [ ] COUNT(?var)
        [ ] COUNT(*)
        [ ] COUNT(DISTINCT ?var)
        [ ] GROUP BY with COUNT
    [ ] SUM aggregates (~10 test cases)
    [ ] AVG aggregates (~10 test cases)
        [ ] Error handling (non-numeric values)
    [ ] MIN/MAX aggregates (~8 test cases)
    [ ] GROUP_CONCAT aggregates (~8 test cases)
        [ ] SEPARATOR handling
    [ ] GROUP BY clauses (~15 test cases)
        [ ] Single grouping variable
        [ ] Multiple grouping variables
        [ ] Grouping with expressions
    [ ] HAVING clauses (~10 test cases)
        [ ] HAVING on aggregates
        [ ] HAVING with complex conditions
    Expected effort: 4 days
    Blocker: GROUP BY implementation required
    Risk: Aggregate semantics complexity
    Success Criteria: 76/76 aggregate tests passing

[ ] 2.1.2.4: CONSTRUCT Query Tests
    [ ] Implement CONSTRUCT query form
    [ ] Simple CONSTRUCT (~12 test cases)
        [ ] Pattern → template mapping
        [ ] Variable substitution
    [ ] Complex CONSTRUCT (~10 test cases)
        [ ] Multiple triple patterns
        [ ] Complex templates
    [ ] CONSTRUCT + DISTINCT (~6 test cases)
    [ ] CONSTRUCT + ORDER BY (~5 test cases)
    [ ] CONSTRUCT + LIMIT (~5 test cases)
    [ ] CONSTRUCT + aggregates (~8 test cases)
    Expected effort: 2 days
    Blocker: CONSTRUCT evaluator
    Risk: Template graph generation
    Success Criteria: 46/46 CONSTRUCT tests passing

[ ] 2.1.2.5: BIND & VALUES Tests
    [ ] Implement BIND evaluation (~20 test cases)
        [ ] Expression evaluation with BIND
        [ ] Variable scoping
        [ ] Type error handling
    [ ] BIND with aggregates (~6 test cases)
    [ ] Multiple BIND clauses (~8 test cases)
    [ ] VALUES clause (~12 test cases)
        [ ] Single variable VALUES
        [ ] Multiple variable VALUES
    [ ] VALUES with expressions (~6 test cases)
    [ ] Empty VALUES (~4 test cases)
    Expected effort: 2 days
    Blocker: BIND/VALUES evaluators
    Risk: Variable scoping edge cases
    Success Criteria: 72/72 BIND+VALUES tests passing

[ ] 2.1.2.6: Property Path Tests
    [ ] Implement property path evaluation engine
    [ ] Simple paths (~10 test cases)
    [ ] Sequence paths (~12 test cases)
    [ ] Alternative paths (~10 test cases)
    [ ] Kleene star (*) paths (~15 test cases)
        [ ] Zero or more repetitions
        [ ] Cycle handling
    [ ] Kleene plus (+) paths (~12 test cases)
        [ ] One or more repetitions
    [ ] Optional paths (?) (~8 test cases)
    [ ] Inverse paths (~8 test cases)
    [ ] Negated paths (~6 test cases)
    [ ] Complex paths (~12 test cases)
    Expected effort: 4-5 days
    Blocker: Path traversal algorithm
    Risk: Cycle detection, performance
    Success Criteria: 93/93 property path tests passing

[ ] 2.1.2.7: Subquery Tests
    [ ] Implement subquery evaluation
    [ ] SELECT subqueries (~15 test cases)
    [ ] FILTER with subqueries (~10 test cases)
    [ ] Scalar subqueries (~8 test cases)
    [ ] Variable scoping (~6 test cases)
    Expected effort: 2 days
    Blocker: Recursive query executor
    Risk: Variable isolation, scope leaking
    Success Criteria: 39/39 subquery tests passing

[ ] 2.1.2.8: Negation Tests
    [ ] Implement NOT EXISTS (~12 test cases)
    [ ] Implement FILTER NOT EXISTS (~8 test cases)
    [ ] Implement MINUS operator (~12 test cases)
        [ ] Set difference semantics
    [ ] NOT IN expressions (~8 test cases)
    Expected effort: 2-3 days
    Blocker: Set semantics implementation
    Risk: Correct NOT semantics
    Success Criteria: 40/40 negation tests passing

[ ] 2.1.2.9: SELECT & Projection Tests
    [ ] SELECT with expressions (~15 test cases)
        [ ] AS variable renaming
    [ ] SELECT * form (~6 test cases)
    [ ] SELECT DISTINCT (~10 test cases)
    [ ] SELECT with LIMIT (~8 test cases)
    [ ] SELECT with OFFSET (~8 test cases)
    [ ] SELECT with ORDER BY (~12 test cases)
        [ ] ASC/DESC
        [ ] Multiple order keys
    Expected effort: 1-2 days
    Blocker: Projection implementation
    Risk: Expression handling in SELECT
    Success Criteria: 59/59 SELECT tests passing

[ ] 2.1.2.10: EXISTS Tests
    [ ] EXISTS in FILTER (~10 test cases)
    [ ] NOT EXISTS (~8 test cases, overlap with negation)
    [ ] Scoped EXISTS (~6 test cases)
    Expected effort: 1 day
    Blocker: EXISTS semantics
    Risk: Scoping correctness
    Success Criteria: 24/24 EXISTS tests passing

[ ] 2.1.2.11: SPARQL Syntax Tests
    [ ] Positive query syntax tests (~30 test cases)
        [ ] Valid SPARQL syntax variations
    [ ] Negative query syntax tests (~25 test cases)
        [ ] Invalid syntax rejection
    [ ] Positive update syntax tests (~20 test cases)
    [ ] Negative update syntax tests (~20 test cases)
    Expected effort: 2 days
    Blocker: Parser validation
    Risk: Grammar completeness
    Success Criteria: 95/95 syntax tests passing

Subtotal Phase 2.1.2: ~842 SPARQL Query tests
Timeline: 3-4 weeks
Complexity: VERY HIGH
Success Target: 842/842 tests passing (100%)
```

#### 2.1.3 SPARQL Update Tests (Week 4)

```
Priority: CRITICAL
Target Crate: crates/sparql/src/

[ ] 2.1.3.1: INSERT/DELETE Operations
    [ ] Implement INSERT DATA (~15 test cases)
    [ ] Implement DELETE DATA (~12 test cases)
    [ ] Implement DELETE WHERE (~10 test cases)
        [ ] Pattern-based deletion
    [ ] Implement INSERT WHERE (~8 test cases)
        [ ] Pattern-based insertion
    [ ] Implement DELETE/INSERT (MODIFY) (~15 test cases)
        [ ] Atomic SPARQL updates
    Expected effort: 3 days
    Blocker: Update executor implementation
    Risk: Data consistency, transaction semantics
    Success Criteria: 60/60 basic update tests

[ ] 2.1.3.2: Graph Manipulation
    [ ] ADD operation (~8 test cases)
    [ ] COPY operation (~8 test cases)
    [ ] MOVE operation (~8 test cases)
    [ ] DROP operation (~8 test cases)
    [ ] CLEAR operation (~8 test cases)
    Expected effort: 2 days
    Blocker: Named graph support
    Risk: Named graph semantics
    Success Criteria: 40/40 graph operation tests

[ ] 2.1.3.3: Silent Operations
    [ ] INSERT SILENT (~6 test cases)
    [ ] DELETE SILENT (~6 test cases)
    [ ] DROP SILENT (~4 test cases)
    Expected effort: 1 day
    Blocker: Silent error handling
    Risk: Silently swallowing errors
    Success Criteria: 16/16 silent tests

Subtotal Phase 2.1.3: ~116 SPARQL Update tests
Timeline: 1 week
Complexity: MEDIUM (subset of query operations)
Success Target: 116/116 tests passing
```

#### 2.1.4 Result Format Tests (Week 5)

```
Priority: CRITICAL
Target Crate: crates/sparql/src/

[ ] 2.1.4.1: JSON Results Format
    [ ] Implement JSON result serializer
    [ ] Test JSON result output (~20 test cases)
        [ ] Correct JSON structure
        [ ] Variable binding serialization
        [ ] Boolean result format
        [ ] ASK query results
    Expected effort: 1 day
    Blocker: serde_json library
    Risk: JSON schema compliance
    Success Criteria: 20/20 JSON tests

[ ] 2.1.4.2: CSV/TSV Results Format
    [ ] Implement CSV result serializer (~12 test cases)
    [ ] Implement TSV result serializer (~12 test cases)
        [ ] Correct delimiter handling
        [ ] Escaping rules
        [ ] Variable order
    Expected effort: 1 day
    Blocker: CSV/TSV format specs
    Risk: Format edge cases
    Success Criteria: 24/24 CSV/TSV tests

[ ] 2.1.4.3: XML & Turtle Results
    [ ] XML results format (~10 test cases, in protocol tests)
    [ ] Turtle results for CONSTRUCT (~8 test cases)
    Expected effort: 1 day
    Blocker: XML serialization
    Risk: Format compliance
    Success Criteria: 18/18 format tests

Subtotal Phase 2.1.4: ~54 result format tests
Timeline: 3 days
Complexity: MEDIUM
Success Target: 54/54 tests passing
```

#### 2.1.5 SHACL Validation Tests (Week 5-6)

```
Priority: CRITICAL (SHACL validation must work)
Target Crate: crates/shacl/src/

[ ] 2.1.5.1: SHACL Core Constraints
    [ ] Set up test infrastructure
    [ ] Download W3C SHACL test suite
    [ ] Node shape validation (~30 test cases)
        [ ] Constraint checking
        [ ] Violation reporting
    [ ] Class constraints (~15 test cases)
        [ ] sh:class property values
    [ ] Property shape validation (~25 test cases)
        [ ] sh:property semantics
    [ ] Cardinality constraints (~20 test cases)
        [ ] sh:minCount, sh:maxCount
    [ ] Value type constraints (~18 test cases)
        [ ] sh:datatype validation
        [ ] sh:nodeKind enforcement
    [ ] Value range constraints (~15 test cases)
        [ ] sh:minExclusive, sh:maxExclusive
    [ ] String constraints (~12 test cases)
        [ ] sh:pattern (regex)
        [ ] sh:minLength, sh:maxLength
    [ ] Enumeration constraints (~8 test cases)
        [ ] sh:in (allowed values)
    [ ] Uniqueness constraints (~6 test cases)
        [ ] sh:uniqueLang
    Expected effort: 5 days
    Blocker: SHACL constraint engine
    Risk: Complex constraint semantics
    Success Criteria: 149/149 core SHACL tests

[ ] 2.1.5.2: SHACL Advanced Constraints
    [ ] AND/OR/NOT combinations (~12 test cases)
    [ ] Recursive shapes (~10 test cases)
        [ ] sh:node constraint
    [ ] Path-based constraints (~15 test cases)
        [ ] sh:path with property paths
    [ ] Closed shapes (~8 test cases)
        [ ] sh:closed (whitelist properties)
    [ ] Ignored properties (~4 test cases)
    Expected effort: 2 days
    Blocker: Advanced constraint implementations
    Risk: Constraint interaction complexity
    Success Criteria: 49/49 advanced tests

[ ] 2.1.5.3: SHACL SPARQL Constraints
    [ ] SPARQL custom constraints (~20 test cases)
        [ ] sh:sparql validation
    [ ] SPARQL targets (~15 test cases)
        [ ] sh:targetNode, sh:targetClass, sh:targetQuery
    [ ] Custom messages (~8 test cases)
        [ ] sh:resultMessage
    [ ] Severity levels (~6 test cases)
        [ ] sh:Violation, sh:Warning, sh:Info
    Expected effort: 2 days
    Blocker: SPARQL constraint evaluation
    Risk: Message templating, severity handling
    Success Criteria: 49/49 SPARQL constraint tests

Subtotal Phase 2.1.5: ~247 SHACL tests
Timeline: 1 week
Complexity: HIGH
Success Target: 247/247 tests passing
```

### Phase 1 Summary

**Total Tests in Phase 1: ~1,558** (Critical core compliance)
- RDF Parsing: 299 tests
- SPARQL Query: 842 tests
- SPARQL Update: 116 tests
- Result Formats: 54 tests
- SHACL Validation: 247 tests

**Timeline**: 6 weeks
**Complexity**: VERY HIGH
**Success Target**: 100% (1,558/1,558)
**Estimated Team Size**: 2-3 senior engineers
**Risk Areas**: 
- Grammar/parser completeness
- Edge case handling
- Performance optimization
- Error message matching

---

### 2.2 Phase 2: Essential - Full Feature Parity (Weeks 7-10)

```
Priority: ESSENTIAL
Target: Add RDFS and OWL 2 RL reasoning tests

[ ] 2.2.1: RDFS Reasoning Implementation
    [ ] rdfs:subClassOf entailment (~20 tests)
    [ ] rdfs:subPropertyOf entailment (~15 tests)
    [ ] rdfs:domain/range inference (~18 tests)
    [ ] rdf:type entailment (~12 tests)
    [ ] rdfs:Container entailment (~8 tests)
    [ ] Transitive closure computation (~10 tests)
    Subtotal: 83 tests
    Timeline: 1.5 weeks
    Complexity: MEDIUM

[ ] 2.2.2: OWL 2 RL Reasoning Implementation
    [ ] Class axioms (~25 tests)
    [ ] Property axioms (~20 tests)
    [ ] Restrictions (~25 tests)
    [ ] Cardinality (~15 tests)
    [ ] AllValuesFrom (~10 tests)
    [ ] Functional properties (~8 tests)
    [ ] Inverse properties (~8 tests)
    [ ] Transitive properties (~8 tests)
    [ ] Symmetric properties (~6 tests)
    [ ] Disjoint classes (~10 tests)
    Subtotal: 135 tests
    Timeline: 2.5 weeks
    Complexity: VERY HIGH

Total Phase 2: 218 tests
Timeline: 4 weeks
Success Target: 90%+ (partial OWL implementation acceptable initially)
Risk: Reasoning complexity, performance
```

### 2.3 Phase 3: Optional - Advanced & Emerging (Weeks 11-14)

```
Priority: OPTIONAL/NICE-TO-HAVE

[ ] 2.3.1: RDF-Star/RDF* Tests (65 tests)
    [ ] Quoted triple parsing
    [ ] Quoted triple semantics
    [ ] Nested quotes
    [ ] RDF-star SPARQL queries
    [ ] Annotations on triples
    Timeline: 1.5 weeks
    Status: Emerging spec

[ ] 2.3.2: OWL 2 EL/DL Profile Tests (85 tests)
    [ ] OWL 2 EL specific tests
    [ ] OWL 2 DL specific tests
    Timeline: 2 weeks
    Status: Advanced, lower priority

[ ] 2.3.3: Performance Benchmark Tests
    [ ] LUBM dataset tests
    [ ] SP2Bench tests
    [ ] Custom performance assertions
    Timeline: 1 week
    Status: Performance validation

Total Phase 3: 150+ tests (optional)
Timeline: 4-5 weeks
Priority: Lower - only after Phase 1 & 2 complete
```

---

## Part 3: Directory Structure Mapping

### 3.1 Jena → rust-kgdb Test Migration Map

```
Apache Jena                          rust-kgdb (Current)        rust-kgdb (Target)
──────────────────────────────────────────────────────────────────────────────

jena-core/testing/
├── Turtle/                     →  (not tested)         →  crates/rdf-io/tests/
│   ├── RaptorTurtle/                                        ├── turtle/
│   └── W3CTests/                                            │   ├── w3c-turtle-*.ttl
│                                                             │   └── manifest.ttl
├── N-Triples/                                            →  crates/rdf-io/tests/
│                                                             ├── ntriples/
│                                                             │   ├── w3c-ntriples-*.nt
│                                                             │   └── manifest.ttl
└── RDF/XML/                                              →  crates/rdf-io/tests/
                                                             ├── rdfxml/
                                                             │   ├── w3c-rdfxml-*.rdf
                                                             │   └── manifest.ttl

jena-arq/testing/ARQ/           →  crates/sparql/tests/ →  crates/sparql/tests/
├── Algebra/                        (skeleton only)          ├── algebra/
├── Optional/                                                ├── optional/
├── Union/                                                   ├── union/
├── Filter/                                                  ├── filter/
├── Bind/                                                    ├── bind/
├── Construct/                                               ├── construct/
├── Aggregates/                                              ├── aggregates/
├── Subquery/                                                ├── subquery/
├── PropertyPaths/                                           ├── property-path/
├── Update/                                                  ├── update/
├── Syntax/                                                  ├── syntax/
└── ...others...                                             └── ...others...

jena-core/testing/RDFS/         →  (not tested)         →  crates/reasoning/tests/
├── subClassOf/                                              ├── rdfs/
├── subPropertyOf/                                           │   ├── subclass-of/
├── domain-range/                                            │   ├── subproperty-of/
│                                                             │   └── domain-range/
jena-core/testing/OWL/          →  (not tested)         →  crates/reasoning/tests/
├── Classes/                                                 ├── owl/
├── Properties/                                              │   ├── classes/
├── Restrictions/                                            │   ├── properties/
│                                                             │   └── restrictions/

jena-shacl/testing/             →  (not tested)         →  crates/shacl/tests/
├── core/                                                    ├── shacl-core/
├── advanced/                                                ├── shacl-advanced/
└── sparql/                                                  └── shacl-sparql/

W3C rdf-tests repository        →  (W3C source)         →  test-data/w3c-rdf-tests/
├── rdf/                                                     ├── rdf/
│   ├── rdf11/                                               │   ├── rdf11/
│   │   ├── rdf-turtle/         →  →  test-data/w3c-rdf-tests/rdf/rdf11/turtle/
│   │   ├── n-triples/          →  →  test-data/w3c-rdf-tests/rdf/rdf11/n-triples/
│   │   ├── rdf-xml/            →  →  test-data/w3c-rdf-tests/rdf/rdf11/rdf-xml/
│   │   ├── n-quads/            →  →  test-data/w3c-rdf-tests/rdf/rdf11/n-quads/
│   │   └── trig/               →  →  test-data/w3c-rdf-tests/rdf/rdf11/trig/
│   └── rdf12/ (emerging)
│
└── sparql/
    └── sparql11/
        ├── algebra/            →  →  test-data/w3c-rdf-tests/sparql/sparql11/algebra/
        ├── aggregates/         →  →  test-data/w3c-rdf-tests/sparql/sparql11/aggregates/
        ├── bind/               →  →  test-data/w3c-rdf-tests/sparql/sparql11/bind/
        ├── bindings/           →  →  test-data/w3c-rdf-tests/sparql/sparql11/bindings/
        ├── construct/          →  →  test-data/w3c-rdf-tests/sparql/sparql11/construct/
        ├── exists/             →  →  test-data/w3c-rdf-tests/sparql/sparql11/exists/
        ├── functions/          →  →  test-data/w3c-rdf-tests/sparql/sparql11/functions/
        ├── grouping/           →  →  test-data/w3c-rdf-tests/sparql/sparql11/grouping/
        ├── json-res/           →  →  test-data/w3c-rdf-tests/sparql/sparql11/json-res/
        ├── negation/           →  →  test-data/w3c-rdf-tests/sparql/sparql11/negation/
        ├── project-expression/ →  →  test-data/w3c-rdf-tests/sparql/sparql11/project-expr/
        ├── property-path/      →  →  test-data/w3c-rdf-tests/sparql/sparql11/property-path/
        ├── subquery/           →  →  test-data/w3c-rdf-tests/sparql/sparql11/subquery/
        ├── syntax-query/       →  →  test-data/w3c-rdf-tests/sparql/sparql11/syntax-query/
        ├── syntax-update-1/    →  →  test-data/w3c-rdf-tests/sparql/sparql11/syntax-update-1/
        ├── syntax-update-2/    →  →  test-data/w3c-rdf-tests/sparql/sparql11/syntax-update-2/
        ├── basic-update/       →  →  test-data/w3c-rdf-tests/sparql/sparql11/basic-update/
        ├── delete*/            →  →  test-data/w3c-rdf-tests/sparql/sparql11/delete-*/
        ├── insert*/            →  →  test-data/w3c-rdf-tests/sparql/sparql11/insert-*/
        ├── update-silent/      →  →  test-data/w3c-rdf-tests/sparql/sparql11/update-silent/
        ├── drop/               →  →  test-data/w3c-rdf-tests/sparql/sparql11/drop/
        ├── clear/              →  →  test-data/w3c-rdf-tests/sparql/sparql11/clear/
        ├── add/                →  →  test-data/w3c-rdf-tests/sparql/sparql11/add/
        ├── copy/               →  →  test-data/w3c-rdf-tests/sparql/sparql11/copy/
        ├── move/               →  →  test-data/w3c-rdf-tests/sparql/sparql11/move/
        ├── csv-tsv-res/        →  →  test-data/w3c-rdf-tests/sparql/sparql11/csv-tsv-res/
        └── manifest*.ttl       →  →  test-data/w3c-rdf-tests/sparql/sparql11/manifest-all.ttl

W3C JSON-LD tests              →  (separate repo)   →  test-data/w3c-jsonld-tests/
├── from-rdf/
├── to-rdf/
├── expand/
├── compact/
├── flatten/
├── frame/
└── manifest.jsonld

W3C SHACL tests                →  (in rdf-tests)   →  test-data/w3c-shacl-tests/
├── core/
├── advanced/
├── sparql/
└── manifest.ttl
```

### 3.2 Test Infrastructure Files to Create

```
crates/rdf-io/
├── src/
│   ├── lib.rs (existing)
│   ├── turtle.rs (existing)
│   ├── ntriples.rs (existing)
│   ├── rdfxml.rs (NEW)
│   ├── nquads.rs (NEW)
│   ├── trig.rs (NEW)
│   ├── jsonld.rs (NEW)
│   ├── w3c_test_runner.rs (NEW) - Manifest parser + test executor
│   └── test_utils.rs (NEW) - Common test utilities
├── tests/
│   ├── lib.rs (NEW) - Test harness integration
│   ├── w3c_conformance.rs (NEW) - W3C test runner
│   ├── turtle_tests.rs (NEW)
│   ├── ntriples_tests.rs (NEW)
│   ├── rdfxml_tests.rs (NEW)
│   ├── nquads_tests.rs (NEW)
│   ├── trig_tests.rs (NEW)
│   ├── jsonld_tests.rs (NEW)
│   └── round_trip_tests.rs (NEW) - Format conversion tests
└── benches/
    └── format_benchmarks.rs (NEW)

crates/sparql/
├── src/
│   ├── lib.rs (existing)
│   ├── test_runner.rs (NEW) - W3C SPARQL manifest parser
│   ├── result_format.rs (NEW) - JSON/CSV/TSV result serialization
│   └── test_utils.rs (NEW)
├── tests/
│   ├── lib.rs (NEW)
│   ├── w3c_conformance.rs (EXISTING - expand it)
│   ├── algebra_tests.rs (NEW)
│   ├── filter_tests.rs (NEW)
│   ├── aggregate_tests.rs (NEW)
│   ├── construct_tests.rs (NEW)
│   ├── bind_tests.rs (NEW)
│   ├── values_tests.rs (NEW)
│   ├── property_path_tests.rs (NEW)
│   ├── subquery_tests.rs (NEW)
│   ├── negation_tests.rs (NEW)
│   ├── select_tests.rs (NEW)
│   ├── exists_tests.rs (NEW)
│   ├── update_tests.rs (NEW)
│   ├── syntax_tests.rs (NEW)
│   ├── result_format_tests.rs (NEW)
│   └── integration_tests.rs (NEW)
└── benches/
    └── w3c_conformance_benchmarks.rs (NEW)

crates/reasoning/
├── src/
│   ├── lib.rs (existing)
│   └── test_runner.rs (NEW)
├── tests/
│   ├── lib.rs (NEW)
│   ├── rdfs_entailment_tests.rs (NEW)
│   ├── owl_reasoner_tests.rs (NEW)
│   └── integration_tests.rs (NEW)
└── benches/
    └── reasoning_benchmarks.rs (NEW)

crates/shacl/
├── src/
│   ├── lib.rs (existing)
│   └── test_runner.rs (NEW)
├── tests/
│   ├── lib.rs (NEW)
│   ├── shacl_core_tests.rs (NEW)
│   ├── shacl_advanced_tests.rs (NEW)
│   ├── shacl_sparql_tests.rs (NEW)
│   └── integration_tests.rs (NEW)
└── benches/
    └── validation_benchmarks.rs (NEW)

test-data/ (NEW - at project root)
├── w3c-rdf-tests/ → cloned from https://github.com/w3c/rdf-tests
├── w3c-jsonld-tests/ → cloned from https://github.com/json-ld/json-ld.org
├── w3c-shacl-tests/ → cloned from W3C SHACL test suite
└── manifest-downloader.sh (NEW) - Script to fetch all test data
```

---

## Part 4: Critical W3C Compliance Gaps Analysis

### 4.1 Current Status (As of November 22, 2025)

**rust-kgdb Coverage**:
- RDF Parsing: Turtle (basic), N-Triples (basic) - ~10% of target
- SPARQL Query: Parser skeleton exists, executor incomplete - ~15% of target
- SPARQL Update: Not implemented - 0% of target
- SHACL Validation: Not implemented - 0% of target
- Reasoning: Not implemented - 0% of target

**Total Current Coverage**: ~5% of W3C standards (estimated ~70 tests passing)
**Target Coverage**: 100% (1,872+ tests)
**Gap**: 1,800+ tests to implement

### 4.2 Critical Compliance Gaps (Blocking Full W3C Certification)

#### Gap 1: RDF Format Parsers (299 tests)

**Missing**:
- RDF/XML parser (71 tests needed)
- JSON-LD parser (89 tests needed)
- N-Quads parser (28 tests needed)
- TriG parser (31 tests needed)

**Impact**: Can't validate RDF data in multiple formats
**Fix Timeline**: 2-3 weeks
**Effort**: HIGH

#### Gap 2: SPARQL Query Features (842 tests)

**Partially Implemented**:
- Basic algebra (~40% coverage)
- Filters & expressions (~30% coverage)
- Aggregates (0% coverage)
- Paths (0% coverage)
- Property paths (0% coverage)

**Not Implemented**:
- CONSTRUCT queries (0 tests)
- BIND/VALUES (0 tests)
- Subqueries (0 tests)
- EXISTS/Negation (0 tests)
- All 64 builtin functions (partial)

**Impact**: Can't execute complex real-world SPARQL queries
**Fix Timeline**: 3-4 weeks
**Effort**: VERY HIGH

#### Gap 3: SPARQL Update (116 tests)

**Status**: 0% implemented
**Missing**:
- INSERT/DELETE operations
- Graph manipulation (ADD, COPY, MOVE, DROP, CLEAR)
- Silent operations

**Impact**: Can't modify RDF graphs (read-only system)
**Fix Timeline**: 1 week
**Effort**: MEDIUM

#### Gap 4: Result Serialization (54 tests)

**Missing**:
- JSON results format (20 tests)
- CSV/TSV results (24 tests)
- XML results (10 tests)

**Impact**: Query results can't be exported in standard formats
**Fix Timeline**: 3 days
**Effort**: MEDIUM

#### Gap 5: SHACL Validation (247 tests)

**Status**: 0% implemented
**Missing**: Entire SHACL module
**Impact**: Can't validate RDF graphs against shape constraints
**Fix Timeline**: 1.5 weeks
**Effort**: HIGH

#### Gap 6: Reasoning (218 tests)

**Status**: 0% implemented
**Missing**:
- RDFS entailment (83 tests)
- OWL 2 RL reasoning (135 tests)

**Impact**: Can't derive implicit triples, no ontology support
**Fix Timeline**: 3-4 weeks
**Effort**: VERY HIGH

#### Gap 7: Grammar/Parser Alignment (Critical)

**Issue**: SPARQL/RDF grammar must exactly match W3C specifications
**Current Risk**: Parser differences could cause test failures
**Validation Needed**: 
- Pest grammar vs W3C SPARQL BNF
- Turtle grammar vs W3C Turtle spec
- Error message matching

### 4.3 Test Result Expectation Challenges

**Problem 1: Manifest Parsing**
- W3C tests use Turtle RDF manifests (manifest.ttl)
- Must parse manifests to extract:
  - Test name
  - Test type (Positive/NegativeSyntax, Evaluation, etc.)
  - Input files
  - Expected output files
  - Data files
- **Solution**: Create Turtle-based manifest parser

**Problem 2: Result Comparison**
- Must compare RDF graphs semantically (not byte-for-byte)
- Blank nodes have different IDs but same structure
- Order-independent comparison needed
- **Solution**: Implement RDF graph isomorphism checker

**Problem 3: Error Message Matching**
- Negative syntax tests expect specific error messages
- Different implementations may have slightly different messages
- **Solution**: Regex/pattern matching for error messages

**Problem 4: Floating Point Precision**
- Numeric operations may have rounding differences
- XSD datatype conversions must be exact
- **Solution**: Implement XSD numeric with proper rounding rules

### 4.4 Compliance Certification Path

```
Week 1-2:   RDF Format Parsing (299 tests)
            ✓ Turtle (99 tests)
            ✓ N-Triples (40 tests)
            ✗ RDF/XML (71 tests) - START HERE
            ✗ JSON-LD (89 tests) - PARALLEL

Week 3-6:   SPARQL Query (842 tests)
            ✓ Basic Algebra (~100 tests)
            ✗ Filters & Expressions (97 tests)
            ✗ Aggregates (76 tests)
            ✗ Paths (93 tests)
            ✗ CONSTRUCT (46 tests)
            ✗ BIND/VALUES (72 tests)
            ✗ Subqueries (39 tests)
            ✗ Negation (40 tests)
            ✗ SELECT (59 tests)
            ✗ Exists (24 tests)
            ✗ Functions (117 tests)
            ✗ Syntax (95 tests)
            ✗ Formats (54 tests)

Week 6-7:   SPARQL Update (116 tests)
            ✗ INSERT/DELETE (60 tests)
            ✗ Graph Ops (40 tests)
            ✗ Silent (16 tests)

Week 8-9:   SHACL Validation (247 tests)
            ✗ Core (149 tests)
            ✗ Advanced (49 tests)
            ✗ SPARQL (49 tests)

Week 10-11: Reasoning (218 tests)
            ✗ RDFS (83 tests)
            ✗ OWL 2 RL (135 tests)

CERTIFICATION REQUIREMENTS:
- 1,558 core tests passing (100%)
- 0 known failures
- W3C test suite download & execution
- Public test report submission
```

---

## Part 5: Recommended Crate Version Updates

### 5.1 Current Versions (from Cargo.toml)

```toml
[workspace.dependencies]
pest = "2.7"              ← CURRENT
pest_derive = "2.7"       ← CURRENT
logos = "0.13"
serde = "1.0"
serde_json = "1.0"
quick-xml = "0.31"
```

### 5.2 Recommended Version Updates

| Crate | Current | Recommended | Reason |
|-------|---------|-------------|--------|
| **pest** | 2.7 | 2.7+ | Latest stable, maintained |
| **pest_derive** | 2.7 | 2.7+ | Must match pest version |
| **logos** | 0.13 | 0.13+ | Good lexer for tokenization |
| **quick-xml** | 0.31 | 0.31+ | Good for RDF/XML parsing |
| **serde** | 1.0 | 1.0+ | Latest stable |
| **serde_json** | 1.0 | 1.0+ | Latest stable |
| **thiserror** | 1.0 | 1.0+ | Error handling |
| **anyhow** | 1.0 | 1.0+ | Error handling |
| **proptest** | 1.4 | 1.4+ | Property testing |
| **criterion** | 0.5 | 0.5+ | Benchmarking |

### 5.3 New Dependencies Needed

For test implementation:

```toml
[dev-dependencies]
# Test discovery and execution
tempfile = "3.8"           # Temporary test files
glob = "0.3"              # File pattern matching
walkdir = "2.4"           # Directory traversal
rayon = "1.8"             # Parallel test execution

# JSON/YAML handling for test manifests
toml = "0.8"              # Config files
yaml = "0.9"              # YAML parsing

# HTTP client for downloading test data
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.35", features = ["full"] }

# Assertion/comparison libraries
pretty_assertions = "1.4"  # Nicer test output
similar = "2.3"           # String/sequence comparison
approx = "0.5"            # Floating point comparison

# Test data generation
arbitrary = "1.3"         # For proptest

# Reporting
junit = "0.1"             # JUnit XML reports
csv = "1.3"               # CSV test results
```

### 5.4 Dependency Justification

| Dependency | Purpose | Phase |
|------------|---------|-------|
| tempfile | Create temporary RDF files during tests | Phase 1 |
| glob, walkdir | Discover W3C test files | Phase 1 |
| rayon | Parallel test execution (1,800+ tests) | Phase 1 |
| toml | Load test configuration | Phase 1 |
| yaml | Parse YAML test metadata | Phase 1 |
| reqwest, tokio | Download W3C test suite automatically | Phase 1 |
| pretty_assertions | Better test failure output | Phase 1 |
| similar | Diff results for debugging | Phase 1 |
| approx | Compare floating point results | Phase 2 |
| arbitrary | Generate test data | Phase 3 |
| junit | Generate reports for CI/CD | Phase 1 |
| csv | Export test results | Phase 2 |

---

## Part 6: Implementation Strategy & Risks

### 6.1 Recommended Implementation Order

**Critical Path** (must do first):

1. **RDF-IO Infrastructure** (Days 1-3)
   - Setup W3C test downloader
   - Implement manifest.ttl parser (Turtle)
   - Create test harness framework
   - Add tempfile/glob dependencies

2. **Turtle Parser Completion** (Days 4-6)
   - Fill gaps in existing parser
   - Implement all Turtle tests (~99 tests)

3. **N-Triples Parser Completion** (Days 7-8)
   - Finalize existing parser
   - Implement all N-Triples tests (~40 tests)

4. **SPARQL Test Infrastructure** (Days 9-11)
   - Implement SPARQL manifest parser
   - Build test executor framework
   - Setup result comparison

5. **Core SPARQL Tests** (Days 12-25)
   - Implement missing query features in priority order:
     - Filters & expressions (HIGH impact)
     - Aggregates (core feature)
     - CONSTRUCT (important for reasoning)
     - Property paths (common pattern)

### 6.2 Major Implementation Challenges

**Challenge 1: RDF Graph Isomorphism**
- Need to compare query results semantically
- Blank nodes with same structure but different IDs should match
- **Solution Complexity**: MEDIUM
- **Time Estimate**: 2-3 days
- **Risk**: Algorithm correctness

**Challenge 2: Grammar Exactness**
- SPARQL grammar must precisely match W3C spec
- Even small differences cause test failures
- **Solution Complexity**: HIGH
- **Time Estimate**: 1 week (validation)
- **Risk**: Grammar completeness, error messages

**Challenge 3: Property Path Cycles**
- Property paths with `*` and `+` can create cycles
- Must avoid infinite loops, count paths correctly
- **Solution Complexity**: VERY HIGH
- **Time Estimate**: 3-4 days
- **Risk**: Algorithm correctness, performance

**Challenge 4: Result Format Serialization**
- SPARQL results have precise XML/JSON/CSV schemas
- Whitespace, order, escaping must match exactly
- **Solution Complexity**: MEDIUM
- **Time Estimate**: 2 days per format
- **Risk**: Format compliance

**Challenge 5: SHACL Constraint Engine**
- sh:path requires property path evaluation
- sh:sparql requires executing arbitrary SPARQL
- Validation messages must match schema
- **Solution Complexity**: VERY HIGH
- **Time Estimate**: 1-1.5 weeks
- **Risk**: Constraint interaction, performance

**Challenge 6: Test Data Volume**
- 1,872+ test cases (some with multiple assertions)
- Parallel execution critical for reasonable test time
- **Solution Complexity**: MEDIUM
- **Time Estimate**: Test harness: 3 days, execution: rayon parallelization
- **Risk**: CI/CD timeout, resource usage

### 6.3 Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Grammar doesn't match W3C exactly | HIGH (40%) | HIGH - 100+ tests fail | Weekly grammar review vs W3C spec |
| Property path cycles cause hangs | MEDIUM (25%) | HIGH - system unusable | Implement cycle detection, timeouts |
| Result comparison wrong (semantics) | MEDIUM (30%) | MEDIUM - 50-100 tests fail | Implement graph isomorphism correctly |
| Performance too slow (>5min full suite) | MEDIUM (30%) | MEDIUM - CI too slow | Parallel execution, selective test running |
| RDF/XML parser edge cases | HIGH (35%) | MEDIUM - 10-20 tests fail | Early implementation, extensive testing |
| JSON-LD context processing | HIGH (40%) | MEDIUM - 20-30 tests fail | Use established library if possible |
| SHACL constraint interaction bugs | HIGH (35%) | HIGH - validation broken | Incremental testing of each constraint type |
| Floating point precision | LOW (10%) | LOW - 5-10 tests fail | Use XSD numeric correctly |

---

## Part 7: Test Data Organization

### 7.1 W3C Test Suite Download URLs

```bash
# RDF Format Tests
git clone https://github.com/w3c/rdf-tests.git test-data/w3c-rdf-tests
# Size: ~500 MB, Contains:
#   - sparql/sparql11/* (SPARQL 1.1 tests)
#   - rdf/rdf11/* (RDF 1.1 format tests)
#   - rdf/rdf12/* (RDF 1.2 emerging tests)

# JSON-LD Tests (separate repository)
git clone https://github.com/json-ld/json-ld.org.git test-data/w3c-jsonld-tests
# Size: ~200 MB, Contains:
#   - tests/ (JSON-LD test suite)

# SHACL Tests
# Either within rdf-tests/shacl/ or from W3C directly
# https://w3c.github.io/data-shapes/data-shapes-test-suite/

# Download script
#!/bin/bash
mkdir -p test-data
cd test-data

# Clone W3C RDF tests
if [ ! -d w3c-rdf-tests ]; then
    git clone https://github.com/w3c/rdf-tests.git w3c-rdf-tests
    echo "Downloaded W3C RDF tests"
fi

# Clone JSON-LD tests
if [ ! -d w3c-jsonld-tests ]; then
    git clone https://github.com/json-ld/json-ld.org.git w3c-jsonld-tests
    echo "Downloaded JSON-LD tests"
fi

echo "Test data ready in test-data/"
```

### 7.2 Test Data Structure

```
test-data/
├── w3c-rdf-tests/
│   ├── sparql/
│   │   ├── sparql11/
│   │   │   ├── algebra/
│   │   │   │   ├── manifest.ttl
│   │   │   │   ├── q1.rq (query)
│   │   │   │   ├── data-1.ttl (data)
│   │   │   │   └── results (expected results)
│   │   │   │       ├── q1.json
│   │   │   │       └── q1.csv
│   │   │   └── ...other categories...
│   │   └── sparql10/ (legacy, skip)
│   │
│   ├── rdf/
│   │   ├── rdf11/
│   │   │   ├── rdf-turtle/
│   │   │   │   ├── manifest.ttl
│   │   │   │   ├── test-001.ttl
│   │   │   │   └── test-001.nt (expected)
│   │   │   ├── n-triples/
│   │   │   ├── rdf-xml/
│   │   │   ├── n-quads/
│   │   │   └── trig/
│   │   └── rdf12/ (emerging, optional)
│   │
│   └── ns/
│       └── vocabulary definitions
│
└── w3c-jsonld-tests/
    ├── tests/
    │   ├── expand-manifest.jsonld
    │   ├── compact-manifest.jsonld
    │   ├── fromRdf-manifest.jsonld
    │   └── toRdf-manifest.jsonld
    └── ...test files...

Total Size: ~700 MB
Network Time: 10-30 min (depending on connection)
Storage: SSD recommended for CI/CD
```

### 7.3 Test File Formats

**SPARQL Test Manifest Example** (manifest.ttl):
```turtle
@prefix mf: <http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#> .
@prefix qt: <http://www.w3.org/2001/sw/DataAccess/tests/test-query#> .
@prefix ex: <http://example.org/> .

<http://www.w3.org/2001/sw/DataAccess/tests/data-r2/algebra/manifest#algebra-1>
  a mf:QueryEvaluationTest ;
  mf:name "Simple triple" ;
  mf:action
    [ qt:query <http://www.w3.org/2001/sw/DataAccess/tests/data-r2/algebra/q1.rq> ;
      qt:data <http://www.w3.org/2001/sw/DataAccess/tests/data-r2/algebra/data-1.ttl> ] ;
  mf:result <http://www.w3.org/2001/sw/DataAccess/tests/data-r2/algebra/result-1.json> .
```

**RDF Test Manifest Example**:
```turtle
@prefix mf: <http://www.w3.org/2001/sw/rdf-test/vocab#> .

<#turtle-001>
  a mf:PositiveEvaluationTest ;
  mf:name "Turtle - simple triple" ;
  mf:action <test-001.ttl> ;
  mf:result <test-001.nt> .
```

---

## Part 8: Continuous Integration & Metrics

### 8.1 CI/CD Pipeline Integration

```yaml
# .github/workflows/w3c-conformance.yml
name: W3C Conformance Tests

on:
  push:
    branches: [ main, feat/* ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC

jobs:
  w3c-conformance:
    runs-on: ubuntu-latest
    timeout-minutes: 120  # 2 hours for full suite
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Cache W3C test data
        uses: actions/cache@v3
        with:
          path: test-data/w3c-*-tests/
          key: w3c-tests-${{ hashFiles('**/test-data-version.txt') }}
      
      - name: Download W3C test suites
        run: scripts/download-w3c-tests.sh
      
      - name: Run RDF-IO tests
        run: cargo test --package rdf-io --test w3c_conformance -- --test-threads=1
        timeout-minutes: 15
      
      - name: Run SPARQL tests
        run: cargo test --package sparql --test w3c_conformance -- --test-threads=1
        timeout-minutes: 45  # ~800 tests
      
      - name: Run SHACL tests
        run: cargo test --package shacl --test w3c_conformance -- --test-threads=1
        timeout-minutes: 20
      
      - name: Run Reasoning tests
        run: cargo test --package reasoning --test w3c_conformance -- --test-threads=1
        timeout-minutes: 20
      
      - name: Generate test report
        if: always()
        run: |
          cargo test --package sparql --test w3c_conformance -- --nocapture > test-report.txt
          python3 scripts/generate-junit-report.py test-report.txt > target/junit.xml
      
      - name: Upload test results
        if: always()
        uses: EnricoMi/publish-unit-test-result-action@v2
        with:
          files: target/junit.xml
      
      - name: Comment PR with test results
        if: always() && github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            // Parse test results and comment on PR
            // Success: ✓ 1,872/1,872 W3C tests passing
            // Failure: ✗ 50 tests failing (2.7% failure rate)
```

### 8.2 Test Metrics to Track

```
COMPLIANCE METRICS:
- Total tests run: 1,872+ (by Phase)
- Tests passing: % (target: 100%)
- Tests failing: % (target: 0%)
- Tests skipped: % (acceptable for Phase 2/3)
- Tests erroring: % (target: 0%)

PERFORMANCE METRICS:
- Total test execution time: minutes
- Average test time: ms
- Slowest test category: (aggregate queries, path evaluation)
- Fastest test category: (syntax tests)
- Memory peak: MB

COVERAGE METRICS:
- Code coverage: % (for rust-kgdb crates)
- Test coverage by category:
  - RDF Parsing: %
  - SPARQL Query: %
  - SPARQL Update: %
  - SHACL Validation: %
  - Reasoning: %

QUALITY METRICS:
- Test failure categorization:
  - Parser bugs: #
  - Semantics bugs: #
  - Performance timeouts: #
  - Format/output mismatches: #
- Flaky test detection: 0 (target)
- Test reproducibility: 100%
```

### 8.3 Dashboard Display (for tracking)

```
═══════════════════════════════════════════════════════════════════
  RUST-KGDB W3C CONFORMANCE TEST DASHBOARD
  Updated: 2025-11-22 14:30 UTC
═══════════════════════════════════════════════════════════════════

 PHASE 1: CRITICAL COMPLIANCE (Target: Week 6)
 ────────────────────────────────────────────────────────────────
  RDF Parsing Formats:        [ ████░░░░░░░░░░░░░░░░ ] 15% (44/299)
    ├─ Turtle:                [ ███████████░░░░░░░░░ ] 95% (94/99)
    ├─ N-Triples:             [ ███████████░░░░░░░░░ ] 90% (36/40)
    ├─ RDF/XML:               [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/71)
    ├─ JSON-LD:               [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/89)
    ├─ N-Quads:               [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/28)
    └─ TriG:                  [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/31)

  SPARQL Query Execution:     [ ██░░░░░░░░░░░░░░░░░░ ]  8% (68/842)
    ├─ Algebra & Patterns:    [ ███████░░░░░░░░░░░░░ ] 35% (84/240)
    ├─ Filters & Expressions: [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/97)
    ├─ Aggregates:            [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/76)
    ├─ Joins & Paths:         [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/93)
    ├─ Advanced:              [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/79)
    └─ Result Formats:        [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/54)

  SPARQL Update Operations:   [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/116)

  SHACL Validation:           [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/247)

  ────────────────────────────────────────────────────────────────
  PHASE 1 TOTAL:              [ ███░░░░░░░░░░░░░░░░░ ]  3% (112/1,558)
  
  On Track: ✗ BEHIND (Behind schedule by 2 weeks)

 PHASE 2: ESSENTIAL FEATURES (Target: Week 10)
 ────────────────────────────────────────────────────────────────
  RDFS Reasoning:             [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/83)
  OWL 2 RL Reasoning:         [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/135)

  ────────────────────────────────────────────────────────────────
  PHASE 2 TOTAL:              [ ░░░░░░░░░░░░░░░░░░░░ ]  0% (0/218)
  
  Status: NOT STARTED

 PERFORMANCE METRICS
 ────────────────────────────────────────────────────────────────
  Total Test Time:            5 minutes 23 seconds
  Avg per Test:               172 ms (target: <50ms)
  Memory Peak:                2.3 GB (target: <1GB)
  Slowest Test:               RDF/XML parsing (3.2s)
  Fastest Test:               Syntax validation (5ms)

 RECENT FAILURES
 ────────────────────────────────────────────────────────────────
  1. Filter expression type coercion (algebra tests)
     - Error: Expected "5"^^xsd:integer, got "5"
     - Issue: String-to-integer promotion not working
  
  2. CONSTRUCT query variable scope
     - Error: Variable ?x not bound in template
     - Issue: Scope isolation bug
  
  3. Property path cycle detection
     - Error: Timeout after 30s
     - Issue: Infinite loop in path traversal

═══════════════════════════════════════════════════════════════════
 NEXT MILESTONES
 ────────────────────────────────────────────────────────────────
 ✓ Complete Turtle parser (Nov 25)
 → Implement RDF/XML parser (Nov 28)
 → Implement JSON-LD parser (Dec 5)
 → Core SPARQL filters & expressions (Dec 8)
 → Aggregates & GROUP BY (Dec 12)
```

---

## Summary

This comprehensive test migration plan covers:

1. **Test Glossary**: 1,872+ W3C RDF/SPARQL/SHACL tests categorized by:
   - 6 major test categories (RDF Parsing, SPARQL Query, SPARQL Update, SHACL, Reasoning, RDF-star)
   - 30+ specific test subcategories
   - Detailed test count estimates for each category

2. **Detailed TODO List**: 
   - Phase 1 (Critical): 1,558 tests (6 weeks)
   - Phase 2 (Essential): 218 tests (4 weeks)
   - Phase 3 (Optional): 150+ tests (4-5 weeks)
   - Breakdown by crate, complexity, blockers, and effort

3. **Directory Structure Mapping**: Jena → rust-kgdb conversion with specific file locations

4. **Compliance Gaps**: 7 critical areas identified with 1,800+ tests needed

5. **Implementation Strategy**: 
   - Recommended order (RDF → SPARQL → SHACL → Reasoning)
   - Major challenges and risks
   - Version updates and new dependencies
   - CI/CD integration plan

6. **Test Data Organization**: 
   - Download URLs for all W3C test suites
   - File structure (700 MB total)
   - Format specifications

7. **Metrics & Reporting**: 
   - CI/CD pipeline configuration
   - Dashboard for tracking progress
   - Quality metrics by category

**Total Effort Estimate**: 14-18 weeks for full W3C compliance (1,872 tests passing)
**Critical Path**: RDF-IO → SPARQL Query → SPARQL Update → SHACL → Reasoning

