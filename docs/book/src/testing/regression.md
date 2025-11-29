# Regression Testing

Strategies for preventing and detecting regressions in rust-kgdb.

## What is Regression Testing?

A regression occurs when previously working functionality breaks due to new changes. Regression testing prevents this by:
- Maintaining a test suite of known-good behaviors
- Running tests on every commit
- Detecting performance degradation
- Preventing API breakage

## Three Types of Regressions

### 1. Functional Regressions

**Problem**: Feature that worked stops working
**Example**: Query that returned 10 results now returns 5

**Prevention**:
```rust
#[test]
fn test_query_returns_correct_count() {
    let store = create_test_store();
    store.load_test_data();

    let results = store.execute_query("SELECT ?x WHERE { ?x ?p ?o }");

    // This assertion prevents regression
    assert_eq!(results.len(), EXPECTED_COUNT);
}
```

### 2. Performance Regressions

**Problem**: Feature still works but runs slower
**Example**: Query takes 100 µs instead of 10 µs

**Prevention**:
```rust
#[bench]
fn bench_lookup_performance(b: &mut Bencher) {
    let store = create_store();

    // Fails if lookup > 5 µs (regression)
    b.iter(|| {
        let start = Instant::now();
        store.lookup_triple(...);
        assert!(start.elapsed() < Duration::from_micros(5));
    });
}
```

### 3. API Regressions

**Problem**: Public API changes break client code
**Example**: Function parameter type changes

**Prevention**:
```rust
// Compile-time check: Function signature can't change
pub fn execute_query(
    &self,
    query: &str,  // If you remove &str, compilation fails
    dict: &Dictionary
) -> Result<Vec<Binding>>
```

## Regression Test Suite

### Basic Suite (Quick - 30 seconds)

```bash
# Run all unit tests
cargo test --lib --workspace
```

**Covers**:
- API contracts
- Critical paths
- Known edge cases

### Extended Suite (Thorough - 5 minutes)

```bash
# Run all tests including integration tests
cargo test --workspace
```

**Covers**:
- Inter-component integration
- End-to-end workflows
- Storage backend operations

### Comprehensive Suite (Complete - 30+ minutes)

```bash
# Run all tests + W3C conformance + benchmarks
cargo test --workspace
cargo test --test w3c_conformance -- --ignored
cargo bench --workspace
```

**Covers**:
- Everything above
- SPARQL 1.1 compliance
- Performance characteristics

## Regression Categories

### Critical Regressions

Must never happen. Verified with every commit.

```rust
#[test]
#[should_panic]  // MUST NOT compile without panic!
fn test_invalid_uri_rejected() {
    let dict = Dictionary::new();
    // Invalid URIs should be rejected
    dict.intern("not a valid uri");  // Must panic
}

#[test]
fn test_triple_store_doesnt_lose_data() {
    let store = InMemoryBackend::new();
    let triple = test_triple();

    store.put(&triple).unwrap();

    // Data must be present after insert
    let result = store.get_all().unwrap();
    assert!(result.contains(&triple));
}
```

### High-Priority Regressions

Very likely to break client code. Tested on every PR.

```rust
#[test]
fn test_sparql_select_returns_bindings() {
    let executor = create_executor();
    let results = executor.execute_query(
        "SELECT ?x WHERE { ?x ?p ?o }",
        &dict
    );

    // Must return results (not empty or error)
    assert!(!results.is_empty());
}

#[test]
fn test_dictionary_interning_works() {
    let dict = Dictionary::new();
    let uri1 = dict.intern("http://example.org/resource");
    let uri2 = dict.intern("http://example.org/resource");

    // Interning must actually intern (same object)
    assert_eq!(uri1 as *const str, uri2 as *const str);
}
```

### Medium-Priority Regressions

Could impact some use cases. Tested weekly.

```rust
#[test]
fn test_large_dataset_loading() {
    let store = InMemoryBackend::new();
    let triples = generate_triples(100_000);

    // Must handle large datasets
    for triple in triples {
        store.put(&triple).unwrap();
    }

    let all = store.get_all().unwrap();
    assert_eq!(all.len(), 100_000);
}

#[test]
fn test_concurrent_access() {
    let store = Arc::new(InMemoryBackend::new());

    // Multiple threads accessing store must work safely
    let mut handles = vec![];
    for i in 0..4 {
        let store_clone = Arc::clone(&store);
        handles.push(std::thread::spawn(move || {
            store_clone.put(&test_triple(i)).unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(store.get_all().unwrap().len(), 4);
}
```

## Regression Detection Strategies

### Strategy 1: Property-Based Testing

Test invariants that must always hold:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_triple_equality(s in ".*", p in ".*", o in ".*") {
        let dict = Dictionary::new();
        let triple1 = Triple::new(
            dict.intern(&s),
            dict.intern(&p),
            dict.intern(&o),
        );
        let triple2 = Triple::new(
            dict.intern(&s),
            dict.intern(&p),
            dict.intern(&o),
        );

        // Property: equal inputs produce equal triples
        prop_assert_eq!(triple1, triple2);
    }

    #[test]
    fn prop_query_idempotent(query in valid_sparql_query()) {
        let store = create_store();
        let results1 = store.execute_query(&query).unwrap();
        let results2 = store.execute_query(&query).unwrap();

        // Property: same query gives same results
        prop_assert_eq!(results1, results2);
    }
}
```

### Strategy 2: Snapshot Testing

Verify output hasn't changed:

```rust
use insta::assert_snapshot;

#[test]
fn test_query_output() {
    let store = create_store();
    let results = store.execute_query("SELECT ?x WHERE { ?x ?p ?o }");

    // Snapshot is stored in snapshot file
    // If changed, test fails and shows diff
    assert_snapshot!(results);
}
```

### Strategy 3: Differential Testing

Compare behavior against reference implementation:

```rust
#[test]
fn test_matches_rdf4j() {
    let query = "SELECT ?x WHERE { ?x ?p ?o }";

    let rust_results = rust_store.execute_query(query);
    let java_results = rdf4j_store.execute_query(query);

    // Results must match
    assert_eq!(
        normalize_results(&rust_results),
        normalize_results(&java_results)
    );
}
```

## Regression Prevention Checklist

- [ ] All existing unit tests pass
- [ ] All integration tests pass
- [ ] W3C conformance tests pass
- [ ] Benchmarks show no regression >2%
- [ ] No panics in error cases
- [ ] API signatures unchanged
- [ ] No unsafe code introduced
- [ ] Memory usage stable
- [ ] Concurrent access still works
- [ ] Large datasets still load
- [ ] Documentation updated

## Continuous Integration

### Pre-Merge Checks

Every PR must pass:

```yaml
name: Regression Detection
on: [pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --lib --workspace

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --test '*' --workspace

  w3c-conformance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: git clone https://github.com/w3c/rdf-tests test-data/rdf-tests
      - run: cargo test --test w3c_conformance -- --ignored

  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo bench --workspace --no-run
      - run: cargo bench --workspace -- --baseline main
```

### Post-Merge Monitoring

After PR is merged:

1. **Benchmark results** tracked over time
2. **Performance graphs** updated
3. **Regression alerts** if >5% slowdown
4. **Issue created** if critical regression

## Handling Regressions

### If Regression Found

1. **Identify commit**: Use `git bisect`
2. **Analyze change**: What was modified?
3. **Fix root cause**: Don't just revert
4. **Add regression test**: Prevent recurrence
5. **Document lesson**: Update guidelines

### Example Regression Investigation

```bash
# Find problematic commit
git bisect start
git bisect bad HEAD           # Current is broken
git bisect good v0.1.0        # Last known good
# ... git bisect continues until found

# Examine problematic commit
git show <hash>

# Add test to prevent recurrence
# (add test before reverting)

# Revert if necessary
git revert <hash>
```

## Regression Test Maintenance

### Remove Obsolete Tests

```rust
#[test]
#[ignore = "Fixed in commit abc123"]
fn test_old_workaround() {
    // ... test for old bug ...
}
```

### Document Regressions

```rust
/// BUG: Issue #123: Queries returned wrong results with empty graphs
/// FIXED: Commit abc123
/// REGRESSION TEST: test_empty_graph_query_fixed
#[test]
fn test_empty_graph_query_fixed() {
    // ...
}
```

## Key Metrics

Track these to detect regressions early:

| Metric | Tool | Target | Alert |
|--------|------|--------|-------|
| Unit test pass rate | CI | 100% | <100% |
| Test execution time | Criterion | <5 min | >10 min |
| Code coverage | Tarpaulin | >90% | <85% |
| Benchmark performance | Criterion | vs baseline | >5% slower |
| Build time | Cargo | <6 min | >10 min |

## Next Steps

- [Testing Strategy](./strategy.md) - Overview
- [Running Tests](./running.md) - Test execution
- [Benchmarks Guide](./benchmarks.md) - Performance testing
- [W3C Conformance](./w3c-conformance.md) - Standards compliance
