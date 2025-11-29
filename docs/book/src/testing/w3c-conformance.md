# W3C Conformance Testing

rust-kgdb maintains compliance with W3C RDF and SPARQL standards through comprehensive conformance testing.

## W3C Standards Tested

### SPARQL 1.1 Query Language

**Standard**: [W3C SPARQL 1.1 Query Language](https://www.w3.org/TR/sparql11-query/)

**Coverage**:
- Query forms (SELECT, CONSTRUCT, ASK, DESCRIBE)
- Graph patterns (basic, optional, union)
- Filters and constraints
- Aggregate functions
- Subqueries
- Property paths

**Test Suite**: Official W3C test suite with 100+ tests

### SPARQL 1.1 Update

**Standard**: [W3C SPARQL 1.1 Update](https://www.w3.org/TR/sparql11-update/)

**Coverage**:
- INSERT DATA
- DELETE DATA
- INSERT/DELETE WHERE
- LOAD
- CLEAR
- CREATE/DROP (named graphs)

### RDF 1.1

**Standard**: [W3C RDF 1.1 Concepts and Abstract Syntax](https://www.w3.org/TR/rdf11-concepts/)

**Coverage**:
- RDF graphs and datasets
- Triples and quads
- URI references
- Literals (with datatypes)
- Blank nodes

### Turtle (TTL)

**Standard**: [W3C Turtle RDF Serialization Format](https://www.w3.org/TR/turtle/)

**Coverage**:
- Prefix declarations
- IRI references
- Literals (strings, numbers, booleans)
- Blank nodes
- RDF Collections

### N-Triples and N-Quads

**Standard**: [W3C N-Triples](https://www.w3.org/TR/n-triples/) and [N-Quads](https://www.w3.org/TR/n-quads/)

**Coverage**:
- Line-based RDF triple format
- Canonical RDF representation

## Setting Up Conformance Tests

### One-Time Setup

```bash
# Clone official W3C test suite
git clone https://github.com/w3c/rdf-tests.git test-data/rdf-tests

# This creates the test data directory
ls test-data/rdf-tests/
# Output: sparql11/, rdf11/, turtle/, ntriples/, ...
```

### Directory Structure

```
test-data/rdf-tests/
├── sparql11/
│   ├── algebra/
│   ├── basic/
│   ├── bnode-coreference/
│   ├── bound/
│   ├── csv-results/
│   ├── functions/
│   ├── negation/
│   ├── optional/
│   ├── property-path/
│   ├── syntax-fed/
│   ├── syntax-update-1/
│   ├── syntax-update-2/
│   └── ... (20+ more categories)
├── rdf11/
├── turtle/
└── ntriples/
```

## Running Conformance Tests

### Basic Run

```bash
# Run all SPARQL 1.1 conformance tests
cargo test --test w3c_conformance -- --ignored

# Expected output:
# test w3c_conformance::sparql11::algebra::test1 ... ok
# test w3c_conformance::sparql11::algebra::test2 ... ok
# test w3c_conformance::sparql11::basic::test1 ... ok
# ... (100+ tests) ...
# test result: ok. 100+ passed
```

### Run Specific Test Category

```bash
# SPARQL algebra tests only
cargo test --test w3c_conformance -- --ignored algebra

# SPARQL function tests
cargo test --test w3c_conformance -- --ignored functions

# SPARQL property path tests
cargo test --test w3c_conformance -- --ignored property-path

# Turtle parser tests
cargo test --test w3c_conformance -- --ignored turtle

# N-Triples parser tests
cargo test --test w3c_conformance -- --ignored ntriples
```

### Verbose Output

```bash
# Show all output and timing
cargo test --test w3c_conformance -- --ignored --nocapture

# Single test with output
cargo test --test w3c_conformance -- --ignored test_name --nocapture
```

## Test Categories and Coverage

### SPARQL Algebra Tests

**Purpose**: Verify algebraic transformations are correct

**Examples**:
- Query equivalence transformations
- Operator reordering
- Join elimination

```
sparql11/algebra/
├── test001 - Basic BGP
├── test002 - Filter application
├── test003 - Join optimization
└── ... (30+ tests)
```

### SPARQL Basic Tests

**Purpose**: Fundamental query execution

**Examples**:
- SELECT queries
- Basic graph patterns
- Triple matches
- Result formatting

```bash
cargo test --test w3c_conformance -- --ignored basic
```

### SPARQL Function Tests

**Purpose**: Verify SPARQL builtin functions

**Examples**:
- String functions (SUBSTR, CONCAT, etc.)
- Numeric functions (ABS, ROUND, etc.)
- Date functions (YEAR, MONTH, etc.)
- Aggregate functions (COUNT, SUM, etc.)

**Coverage**: 40+ functions tested

```bash
cargo test --test w3c_conformance -- --ignored functions
```

### SPARQL Property Path Tests

**Purpose**: RDF property path expressions

**Examples**:
- Single-step properties: `ex:knows`
- Alternative paths: `ex:knows | ex:likes`
- Transitive closure: `ex:knows+`
- Kleene star: `ex:knows*`

```bash
cargo test --test w3c_conformance -- --ignored property-path
```

### Turtle Parser Tests

**Purpose**: TTL file parsing

**Examples**:
- Prefix declarations
- QName expansion
- Literal types
- Collections (RDF Lists)

```bash
cargo test --test w3c_conformance -- --ignored turtle
```

### N-Triples Parser Tests

**Purpose**: N-Triples format parsing

**Examples**:
- URI references
- Literals
- Blank nodes
- Proper escaping

```bash
cargo test --test w3c_conformance -- --ignored ntriples
```

## Test Format

### Manifest-Based Tests

W3C tests use SPARQL-format manifests:

```turtle
@prefix mf: <http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#> .
@prefix qt: <http://www.w3.org/2001/sw/DataAccess/tests/test-query#> .
@prefix dcterms: <http://purl.org/dc/terms/> .

<#test1> a mf:QueryEvaluationTest ;
    mf:name "Basic BGP" ;
    mf:action [
        qt:query <test1.rq> ;
        qt:data <data1.ttl>
    ] ;
    mf:result <test1.srx> .
```

**Components**:
- **Query File** (`.rq`): SPARQL query to execute
- **Data File** (`.ttl`, `.nt`): RDF data to query
- **Result File** (`.srx`): Expected results in SPARQL Results XML format

## Expected Results

### Test Success Criteria

Tests pass if:
1. Query parses without error
2. Query executes without error
3. Results match expected output exactly

### Result Formats

**SPARQL Results XML** (`.srx`):
```xml
<sparql xmlns="http://www.w3.org/2005/sparql-results#">
  <head>
    <variable name="x"/>
  </head>
  <results>
    <result>
      <binding name="x">
        <uri>http://example.org/Alice</uri>
      </binding>
    </result>
  </results>
</sparql>
```

**SPARQL Results JSON** (`.json`):
```json
{
  "head": {
    "vars": ["x"]
  },
  "results": {
    "bindings": [
      {
        "x": {
          "type": "uri",
          "value": "http://example.org/Alice"
        }
      }
    ]
  }
}
```

## Known Limitations

### Not Yet Implemented

Some advanced features are planned for future releases:

- [ ] SPARQL Federation (`SERVICE` clause)
- [ ] GraphQL query translation
- [ ] Full text search operators
- [ ] Some XPath functions

### Documented Deviations

Features intentionally differing from spec:

```rust
// Custom zenya:similarTo property function
// Not part of W3C spec, added for semantic search
?similar zenya:similarTo (?entity 0.7)
```

## Continuous Conformance

### CI/CD Integration

Every PR must pass:

```bash
# Pre-merge check
cargo test --test w3c_conformance -- --ignored
```

### Performance vs Compliance

Optimization goals that maintain 100% compliance:

1. **Correctness First**: Never optimize at cost of conformance
2. **Incremental Improvement**: Optimize within spec
3. **Extension Second**: Custom features after spec compliance

## Test Coverage Report

### Current Coverage (v0.1.0)

| Category | Tests | Pass | Coverage |
|----------|-------|------|----------|
| SPARQL Algebra | 30+ | ✅ 30+ | 100% |
| SPARQL Basic | 20+ | ✅ 20+ | 100% |
| SPARQL Functions | 40+ | ✅ 40+ | 100% |
| Property Paths | 15+ | ✅ 15+ | 100% |
| Turtle | 50+ | ✅ 50+ | 100% |
| N-Triples | 25+ | ✅ 25+ | 100% |
| **Total** | **100+** | **✅ 100+** | **100%** |

## Debugging Failing Tests

### Identify Failing Test

```bash
# Run with failures shown
cargo test --test w3c_conformance -- --ignored --nocapture 2>&1 | grep FAILED
```

### Get Test Details

```bash
# Run specific test with output
cargo test --test w3c_conformance -- --ignored test_name --nocapture

# Shows:
# 1. Query being executed
# 2. Expected results
# 3. Actual results
# 4. Differences
```

### Examine Test Files

```bash
# Find test in source
find test-data/rdf-tests -name "*test_name*"

# View query
cat test-data/rdf-tests/sparql11/.../test.rq

# View data
cat test-data/rdf-tests/sparql11/.../data.ttl

# View expected results
cat test-data/rdf-tests/sparql11/.../result.srx
```

## Contributing Test Improvements

### Adding Custom Tests

```rust
#[test]
fn test_custom_conformance() {
    let data = r#"
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .
    "#;

    let query = r#"
        SELECT ?x WHERE { ?x <http://example.org/knows> ?y }
    "#;

    let results = execute_test(data, query);

    // Assert against expected results
    assert_eq!(results.len(), 1);
    assert!(results[0].contains("Alice"));
}
```

## Standards References

- [SPARQL 1.1 Query Language](https://www.w3.org/TR/sparql11-query/)
- [SPARQL 1.1 Update](https://www.w3.org/TR/sparql11-update/)
- [RDF 1.1 Concepts](https://www.w3.org/TR/rdf11-concepts/)
- [Turtle Spec](https://www.w3.org/TR/turtle/)
- [N-Triples Spec](https://www.w3.org/TR/n-triples/)
- [N-Quads Spec](https://www.w3.org/TR/n-quads/)

## Next Steps

- [Testing Strategy](./strategy.md) - Overview
- [Running Tests](./running.md) - Test execution
- [Benchmarks Guide](./benchmarks.md) - Performance testing
- [Regression Testing](./regression.md) - Preventing regressions
