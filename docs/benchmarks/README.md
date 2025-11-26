# Benchmark Documentation

Performance benchmarks, comparisons, and optimization plans for rust-kgdb.

## Current Benchmarks (2025-11-18)

### Key Performance Metrics

| Metric | Result | vs RDFox |
|--------|--------|----------|
| **Lookup Speed** | 2.78 µs | ✅ **35-180x faster** |
| **Bulk Insert** | 146,627 triples/sec | ⚠️ 73% (gap closing) |
| **Dictionary New** | 909,091 URIs/sec | ✅ Competitive |
| **Dictionary Cached** | 1.66M lookups/sec | ✅ Excellent |
| **Memory/Triple** | 24 bytes | ✅ **25% better** |

## Files

### [BENCHMARK_RESULTS_REPORT.md](BENCHMARK_RESULTS_REPORT.md) ⭐
**Latest comprehensive benchmark report** (2025-11-18)
- Full Criterion benchmark results
- Comparison with RDFox and Apache Jena
- Test environment details
- Performance breakdown by component

### [HONEST_BENCHMARK_PLAN.md](HONEST_BENCHMARK_PLAN.md)
**4-week optimization roadmap**
- Week-by-week tasks and expected improvements
- Target: Beat RDFox on all metrics
- From 146K → 450K+ triples/sec

### [COMPLETE_FEATURE_COMPARISON.md](COMPLETE_FEATURE_COMPARISON.md)
**Feature-by-feature comparison matrix**
- Rust KGDB vs Apache Jena vs RDFox
- Corrected builtin function count: **64 functions**
- Storage backends, SPARQL support, mobile capabilities

### [BENCHMARK_COMPARISON.md](BENCHMARK_COMPARISON.md)
**Architectural comparison and competitive analysis**
- Zero-copy vs traditional architectures
- Memory efficiency analysis
- Where Rust KGDB wins today

### [BENCHMARK_DEMO.md](BENCHMARK_DEMO.md)
**Benchmark implementation guide**
- LUBM and SP2Bench query listings
- How to run benchmarks
- Expected results format

## Running Benchmarks

```bash
# Generate LUBM test data
rustc tools/lubm_generator.rs -O -o tools/lubm_generator
./tools/lubm_generator 1 /tmp/lubm_1.nt

# Run Criterion benchmarks
cargo bench --package storage --bench triple_store_benchmark

# Results will be in target/criterion/
```

## Benchmark History

- **2025-11-18**: First real performance measurements with LUBM data
  - 2.78 µs lookup speed (359,712 ops/sec)
  - 146,627 triples/sec bulk insert
  - Proven 35-180x faster than RDFox on lookups

---

**Status**: ✅ Benchmarks complete and validated
**Next**: Week 1 optimizations (SIMD, rayon, batching)
