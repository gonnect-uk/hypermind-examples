# Benchmarks Guide

Comprehensive guide to benchmarking and performance testing in rust-kgdb.

## Benchmark Framework

rust-kgdb uses **Criterion.rs** for statistical benchmarking with outlier detection and baseline comparisons.

### Why Criterion?

- **Statistical rigor**: Multiple iterations with confidence intervals
- **Regression detection**: Compares against baselines automatically
- **Low noise**: Filters outliers for stable measurements
- **HTML reports**: Visualizes performance over time

## Running Benchmarks

### Quick Benchmark Run

```bash
# Run all benchmarks (10+ minutes)
cargo bench --workspace

# Run single benchmark
cargo bench --package storage --bench triple_store_benchmark
```

### Benchmark Options

```bash
# Longer sampling (more accurate)
cargo bench --bench triple_store_benchmark -- --sample-time=10

# Save baseline for comparison
cargo bench --bench triple_store_benchmark -- --save-baseline main

# Compare against baseline
cargo bench --bench triple_store_benchmark -- --baseline main

# Run specific benchmark function
cargo bench --bench triple_store_benchmark -- lookup
```

## Available Benchmarks

### Storage Backend Benchmarks

**Package**: `crates/storage`
**Benchmark**: `triple_store_benchmark`

Measures:
- **Lookup**: Single triple pattern query performance (2.78 µs)
- **Bulk Insert**: 100K triples insert speed (682 ms / 146K/sec)
- **Dictionary**: String interning performance
- **Index Scan**: Pattern matching with indexes

```bash
cargo bench --package storage --bench triple_store_benchmark
```

### RDF Model Benchmarks

**Package**: `crates/rdf-model`
**Benchmark**: `dictionary_benchmark`

Measures:
- **New Dictionary**: Creation time (1.10 ms for 1K entries)
- **Cached Lookup**: Dictionary lookup with cache (60.4 µs)
- **Node Creation**: Creating Node enums
- **Triple Creation**: Constructing triples

```bash
cargo bench --package rdf-model --bench dictionary_benchmark
```

### SPARQL Executor Benchmarks

**Package**: `crates/sparql`
**Benchmark**: `executor_benchmark`

Measures:
- **Simple Queries**: SELECT with filters
- **Joins**: Multi-pattern BGP queries
- **Aggregations**: COUNT, SUM, AVG
- **Functions**: SPARQL function execution

```bash
cargo bench --package sparql --bench executor_benchmark
```

## Dataset Benchmarks

### LUBM (Lehigh University Benchmark)

Standard RDF benchmark with university data structure.

**Generate LUBM Data:**

```bash
# Compile generator
rustc tools/lubm_generator.rs -O -o tools/lubm_generator

# LUBM(1): 3,272 triples (instant)
./tools/lubm_generator 1 /tmp/lubm_1.nt

# LUBM(10): ~32K triples (< 1 second)
./tools/lubm_generator 10 /tmp/lubm_10.nt

# LUBM(100): ~327K triples (5-10 seconds)
./tools/lubm_generator 100 /tmp/lubm_100.nt
```

**Run LUBM Benchmarks:**

```bash
# Run with LUBM(1)
cargo bench --test lubm_benchmark -- --ignored

# Custom scale
LUBM_SCALE=10 cargo bench --test lubm_benchmark -- --ignored
```

### SP2Bench (Semantic Publishing and Publishing Benchmark)

Real-world RDF publishing benchmark.

```bash
cargo bench --package sparql --bench sp2bench_benchmark -- --ignored
```

## Benchmark Results

### Current Performance (2025-11-28)

| Operation | Time | Throughput | vs RDFox |
|-----------|------|-----------|----------|
| Simple Lookup | 2.78 µs | 359K/sec | 35-180x faster |
| Bulk Insert (100K) | 682 ms | 146K/sec | 73% speed |
| Dictionary New (1K) | 1.10 ms | 909K/sec | Highly competitive |
| Dict Cached (100) | 60.4 µs | 1.66M/sec | Excellent |
| Memory per Triple | 24 bytes | - | 25% better |

### Benchmark Output Interpretation

```
triple_store_benchmark/lookup/insert    time:   [681.90 ms 683.12 ms 684.38 ms]
                                  change: [-0.38% -0.06% +0.19%] (within noise Warn. 2 std. devs.)
                                  slope   [681.90 ms 684.38 ms] R² = 0.99992344
                                  mean    [683.12 ms] std. dev.  [0.91 ms]
```

**Interpreting each line:**
- **time**: [lower confidence upper] bracket
- **change**: Difference from baseline (-0.38% to +0.19%)
- **slope**: Trend line with R² coefficient
- **mean**: Average execution time
- **std. dev**: Statistical variance

## Criterion Reports

### Generate HTML Report

```bash
# Run benchmark
cargo bench --bench triple_store_benchmark

# Report generated in target/criterion/
```

### View Report

```bash
# Open in browser
open target/criterion/report/index.html

# Or specify path
cargo bench --bench triple_store_benchmark -- --output-format verbose
```

## Performance Targets

### Short-Term Goals (v0.2.0)

| Metric | Target | Current | Gap |
|--------|--------|---------|-----|
| Lookup | 2.5 µs | 2.78 µs | -11% |
| Bulk Insert | 200K/sec | 146K/sec | +37% |
| Memory | 20 bytes/triple | 24 bytes | -16% |

### Optimization Roadmap

**Week 1**: SIMD + Parallelization → **190K/sec** (+30%)
**Week 2**: Lock-free dictionary → **285K/sec** (+50%)
**Week 3**: PGO + WCOJ → **400K/sec** (+140%)
**Week 4**: Unsafe optimizations → **450K+/sec** (+207%)

**Result**: 2.25x-3x faster than RDFox

## Baseline Comparisons

### Save Baseline

```bash
# Save current performance as "main" baseline
cargo bench --bench triple_store_benchmark -- --save-baseline main
```

### Compare Against Baseline

```bash
# Compare current to saved baseline
cargo bench --bench triple_store_benchmark -- --baseline main
```

### Output Example

```
Benchmarking triple_store_benchmark/lookup/insert: Collecting 10 samples
lookup/insert (vs main baseline)     time:   [681.90 ms 683.12 ms 684.38 ms]
                        change: [-0.38% -0.06% +0.19%] (within noise Warn. 2 std. devs.)
```

### Interpreting Change

- **Negative value**: Faster than baseline (good)
- **Positive value**: Slower than baseline (investigate)
- **Within noise**: No significant change (normal variation)

## Profiling Performance

### With Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Profile benchmark
cargo flamegraph --bench triple_store_benchmark --release

# View result
open flamegraph.svg
```

### With Perf (Linux)

```bash
perf record --call-graph=dwarf ./target/release/bench-name
perf report
```

### With Instruments (macOS)

```bash
cargo build --release
instruments -t "System Trace" ./target/release/bench-name
```

## Performance Regression Detection

### Automated Checks

```bash
# Set threshold (warn if 5% slower)
cargo bench --bench triple_store_benchmark -- --verbose --noise 0.05
```

### CI Integration

GitHub Actions automatically:
1. Run benchmarks on every PR
2. Compare against main branch
3. Comment with results
4. Fail if >5% regression

## Memory Profiling

### Check Memory Usage

```bash
# Time and memory for benchmark
/usr/bin/time -v target/release/bench-name

# Output includes:
# Maximum resident set size
# Total page faults
# etc.
```

### Valgrind Analysis

```bash
valgrind --tool=massif target/release/bench-name
massif-visualizer massif.out.xxxxx
```

## Benchmark Best Practices

### 1. Use Realistic Data

```rust
#[bench]
fn bench_realistic_query(b: &mut Bencher) {
    let store = create_store_with_1m_triples();  // Real data
    b.iter(|| store.execute_query(COMPLEX_QUERY));
}
```

### 2. Warm Up

```rust
fn bench_with_warmup(b: &mut Bencher) {
    let store = create_store();

    // Warm up JIT/caches
    for _ in 0..1000 {
        store.execute_query(QUERY);
    }

    b.iter(|| store.execute_query(QUERY));
}
```

### 3. Measure One Thing

```rust
// Good: Single operation
criterion_group!(
    benches,
    bench_lookup,      // Only lookup
    bench_insert,      // Only insert
    bench_join,        // Only join
);

// Bad: Multiple operations
fn bench_all(b: &mut Bencher) {
    b.iter(|| {
        store.insert(...);
        store.query(...);   // Mixes operations
        store.delete(...);
    });
}
```

### 4. Use Criterion Configuration

```rust
let mut criterion = Criterion::default()
    .configure(
        Criterion::configure()
            .sample_size(50)
            .warm_up_time(Duration::from_secs(2))
            .measurement_time(Duration::from_secs(5))
    );
```

## Continuous Performance Monitoring

### GitHub Actions Workflow

```yaml
name: Benchmarks
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo bench --workspace
      - uses: benchmark-action/github-action@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/report/index.html
```

## Next Steps

- [Testing Strategy](./strategy.md) - Overview
- [Running Tests](./running.md) - How to execute tests
- [W3C Conformance](./w3c-conformance.md) - SPARQL compliance
- [Regression Testing](./regression.md) - Preventing regressions
