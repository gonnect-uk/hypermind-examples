# Running Tests

Complete guide to executing tests in rust-kgdb.

## Quick Start

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run in release mode (faster)
cargo test --workspace --release
```

## Test Levels

### Unit Tests (Fast)

```bash
# All unit tests
cargo test --workspace --lib

# Single crate
cargo test -p rdf-model
cargo test -p storage
cargo test -p sparql

# Single test
cargo test test_dictionary_interning
```

### Integration Tests

```bash
# All integration tests
cargo test --test '*'

# Specific integration test
cargo test --test w3c_conformance -- --ignored
```

### Doc Tests

```bash
# Test code examples in documentation
cargo test --doc

# Doc tests for single crate
cargo test --doc -p sparql
```

## Running Specific Tests

### By Name Pattern

```bash
# All tests with "sparql" in name
cargo test sparql

# All parser tests
cargo test parser

# All lookup tests
cargo test lookup
```

### By Module

```bash
# All tests in storage module
cargo test storage::

# All tests in dictionary module
cargo test dictionary::
```

### Single Test Function

```bash
# Exact name match
cargo test --lib test_exact_name -- --exact

# Contains pattern
cargo test test_pattern
```

## W3C Conformance Tests

### One-Time Setup

```bash
# Clone W3C test suite
git clone https://github.com/w3c/rdf-tests test-data/rdf-tests

# This creates test-data/rdf-tests/ with official test cases
```

### Run Conformance Tests

```bash
# Run ignored (long-running) tests
cargo test --test w3c_conformance -- --ignored

# Run with output to see progress
cargo test --test w3c_conformance -- --ignored --nocapture

# Run specific category
cargo test --test w3c_conformance -- --ignored algebra
cargo test --test w3c_conformance -- --ignored functions
```

### Conformance Coverage

Test categories:
- Algebra tests (~30 tests)
- Basic tests (~20 tests)
- Function tests (~40 tests)
- Syntax tests (~30 tests)

Total: **100+ official test cases**

## Performance Benchmarks

### Run All Benchmarks

```bash
# Full benchmark suite (10+ minutes)
cargo bench --workspace

# Release mode only
cargo bench --workspace --release
```

### Run Specific Benchmark

```bash
# Triple store benchmark
cargo bench --package storage --bench triple_store_benchmark

# Dictionary benchmark
cargo bench --package rdf-model --bench dictionary_benchmark

# SPARQL executor benchmark
cargo bench --package sparql --bench executor_benchmark
```

### Benchmark Options

```bash
# Run benchmarks with 60-second sample time
cargo bench --bench triple_store_benchmark -- --sample-time=60

# Generate baseline for comparison
cargo bench --bench triple_store_benchmark -- --save-baseline main

# Compare against baseline
cargo bench --bench triple_store_benchmark -- --baseline main

# Run specific benchmark function
cargo bench --bench triple_store_benchmark -- lookup
```

### Understanding Benchmark Output

```
lookup/insert                       time:   [681.90 ms 683.12 ms 684.38 ms]
                            change: [-0.38% -0.06% +0.19%] (within noise Warn. 2 std. devs.)

lookup/simple                       time:   [2.7760 µs 2.7820 µs 2.7880 µs]
                            change: [-1.28% -0.66% -0.08%] (within noise Warn. 2 std. devs.)
```

- **time**: [lower upper upper] in microseconds
- **change**: Difference from baseline (if specified)

## LUBM Benchmarks

### Generate LUBM Data

```bash
# Compile generator
rustc tools/lubm_generator.rs -O -o tools/lubm_generator

# Generate datasets
./tools/lubm_generator 1 /tmp/lubm_1.nt       # 3,272 triples
./tools/lubm_generator 10 /tmp/lubm_10.nt     # ~32K triples
./tools/lubm_generator 100 /tmp/lubm_100.nt   # ~327K triples
```

### Run LUBM Benchmarks

```bash
# LUBM query performance
cargo bench --test lubm_benchmark -- --ignored

# LUBM with LUBM(10) dataset
cargo bench --test lubm_benchmark -- --ignored -- --lubm-scale 10

# LUBM with LUBM(100) dataset
cargo bench --test lubm_benchmark -- --ignored -- --lubm-scale 100
```

## Test Modes

### Debug Mode (Development)

```bash
# Fast compilation, slow execution
cargo test

# Recommended for quick iteration
# Best for fixing bugs
```

### Release Mode (Performance)

```bash
# Slow compilation, fast execution
cargo test --release

# Recommended for benchmarks
# Best for performance validation
```

## Output Control

### Suppress Output

```bash
# Fail silently on success
cargo test

# Show all println! statements
cargo test -- --nocapture

# Show test execution order
cargo test -- --test-threads=1
```

### Colored Output

```bash
# Force colored output
cargo test -- --color=always

# No colored output
cargo test -- --color=never
```

### Filter Test Output

```bash
# Show only test summary (no output)
cargo test

# Show all output
cargo test -- --nocapture

# Quiet mode
cargo test -q
```

## Concurrent vs Sequential

### Parallel (Default)

```bash
# Run tests in parallel (faster)
cargo test
```

### Sequential

```bash
# Run tests one at a time (slower but clearer output)
cargo test -- --test-threads=1
```

### Limited Concurrency

```bash
# Run with 4 threads
cargo test -- --test-threads=4
```

## Environment Variables

### Logging

```bash
# Show debug logs during tests
RUST_LOG=debug cargo test

# Show trace logs (verbose)
RUST_LOG=trace cargo test --lib

# Filter to specific module
RUST_LOG=sparql=debug cargo test
```

### Test Behavior

```bash
# Run ignored tests only
cargo test -- --ignored

# Run both regular and ignored tests
cargo test -- --include-ignored

# Show backtrace on panic
RUST_BACKTRACE=1 cargo test

# Full backtrace
RUST_BACKTRACE=full cargo test
```

## Code Coverage

### Install Coverage Tool

```bash
cargo install cargo-tarpaulin
```

### Generate Coverage Report

```bash
# HTML report
cargo tarpaulin --out Html --exclude-files tests/

# Open report
open tarpaulin-report.html
```

### Coverage Targets

Aim for >90% line coverage across crates.

## Continuous Integration

### Local Checks Before Push

```bash
# Run all checks locally
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

### Pre-commit Hook

```bash
# Create .git/hooks/pre-commit
#!/bin/bash
cargo test --lib || exit 1
cargo fmt --all -- --check || exit 1
cargo clippy --workspace || exit 1
```

## Troubleshooting

### Test Hangs

```bash
# Kill hanging tests
cargo test -- --test-threads=1  # Debug with one thread

# Check logs for deadlocks
RUST_LOG=debug cargo test -- --nocapture
```

### Memory Issues

```bash
# Run with limited parallelism
cargo test -- --test-threads=2

# Check memory usage
/usr/bin/time -v cargo test
```

### Flaky Tests

```bash
# Run test multiple times
for i in {1..10}; do
    cargo test test_name || break
done
```

### Slow Tests

```bash
# Identify slowest tests
cargo test -- --test-threads=1 --nocapture | tee test-output.txt
grep "test result:" test-output.txt | sort
```

## Next Steps

- [Testing Strategy](./strategy.md) - Overview and organization
- [Benchmarks Guide](./benchmarks.md) - Performance testing
- [W3C Conformance](./w3c-conformance.md) - SPARQL compliance
- [Regression Testing](./regression.md) - Preventing regressions
