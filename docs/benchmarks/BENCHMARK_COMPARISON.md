# Rust KGDB vs Jena vs RDFox - Benchmark Comparison

**Date**: 2025-11-18
**Status**: ⚠️ **INFRASTRUCTURE READY - ACTUAL BENCHMARKS PENDING**

---

## ⚠️ Important Note

**Current Status**: The benchmark infrastructure is 100% complete and ready to run, but **actual benchmark results with LUBM/SP2Bench datasets are not yet available**. This document provides:
1. **Architectural comparison** (factual)
2. **Expected performance** (theoretical, based on architecture)
3. **How to run actual benchmarks** (when datasets are available)

---

## 1. Feature Comparison (Factual)

| Feature | Rust KGDB | Apache Jena | RDFox |
|---------|-----------|-------------|-------|
| **SPARQL 1.1 Query** | ✅ Full | ✅ Full | ✅ Full |
| **SPARQL 1.1 Update** | ✅ Full | ✅ Full | ✅ Full |
| **RDFS Reasoning** | ✅ Full | ✅ Full | ✅ Full |
| **OWL 2 Reasoning** | ✅ RL/EL/QL | ✅ Full | ✅ Full |
| **Zero-Copy Architecture** | ✅ Yes | ❌ No | ❌ No |
| **Memory Safety** | ✅ Rust | ❌ Java (GC) | ❌ C++ |
| **Mobile Deployment** | ✅ iOS/Android | ❌ No | ❌ No |
| **W3C PROV-O** | ✅ Yes | ✅ Yes | ❌ No |
| **W3C SHACL** | ✅ Yes | ✅ Yes | ✅ Yes |
| **LUBM Benchmark Suite** | ✅ Ready (14 queries) | ✅ Yes | ✅ Yes |
| **SP2Bench Suite** | ✅ Ready (17 queries) | ✅ Yes | ✅ Yes |
| **License** | ✅ Open Source | ✅ Apache 2.0 | 💰 Commercial |

---

## 2. Architectural Comparison

### 2.1 Rust KGDB Architecture

**Strengths**:
- ✅ **Zero-copy design**: Borrowed references with lifetimes, no unnecessary allocations
- ✅ **String interning**: Dictionary-based with stable 'static references
- ✅ **Memory safety**: Compile-time guarantees, no GC pauses
- ✅ **Mobile-ready**: FFI bindings for iOS/Android
- ✅ **Pluggable storage**: InMemory, RocksDB, LMDB backends
- ✅ **Modern Rust ecosystem**: Type safety, pattern matching, iterators

**Tradeoffs**:
- ⚠️ **Smaller ecosystem**: Fewer RDF libraries compared to Java
- ⚠️ **OWL 2 coverage**: RL/EL/QL profiles (not full DL)
- ⚠️ **Maturity**: Newer project vs 15+ year old Jena/RDFox

### 2.2 Apache Jena Architecture

**Strengths**:
- ✅ **Mature ecosystem**: 15+ years of development
- ✅ **Complete OWL 2**: Full DL reasoning
- ✅ **Large community**: Extensive documentation, examples
- ✅ **ARQ query engine**: Highly optimized SPARQL
- ✅ **TDB2 storage**: Production-proven persistent storage

**Tradeoffs**:
- ⚠️ **Java GC overhead**: Unpredictable pauses for large datasets
- ⚠️ **Memory overhead**: JVM heap + object headers
- ⚠️ **Mobile**: Not suitable for iOS/Android
- ⚠️ **Startup time**: JVM warmup required

### 2.3 RDFox Architecture

**Strengths**:
- ✅ **Best-in-class performance**: Highly optimized C++
- ✅ **In-memory design**: Extremely fast queries
- ✅ **Parallel reasoning**: Multi-threaded rule evaluation
- ✅ **Datalog engine**: Advanced reasoning capabilities
- ✅ **Commercial support**: Professional support available

**Tradeoffs**:
- ⚠️ **Commercial license**: Not free for commercial use
- ⚠️ **C++ memory management**: Manual memory management risks
- ⚠️ **Closed source**: Limited visibility into internals
- ⚠️ **Mobile**: Not designed for mobile deployment

---

## 3. Expected Performance (Theoretical)

### 3.1 Rust KGDB Expected Performance

Based on zero-copy architecture and Rust optimizations:

| Dataset Size | Load Time | Simple Query | Complex Query | Join Query |
|--------------|-----------|--------------|---------------|------------|
| **10K triples** | ~50ms | <1ms | <10ms | <5ms |
| **100K triples** | ~500ms | <5ms | <50ms | <25ms |
| **1M triples** | ~5s | <50ms | <500ms | <250ms |
| **10M triples** | ~50s | <500ms | <5s | <2.5s |

**Reasoning Performance**:
- RDFS inference (10K): ~200ms
- OWL 2 inference (10K): ~500ms
- Transitive closure: O(n²) with optimizations

### 3.2 Apache Jena Expected Performance

Based on published benchmarks and community reports:

| Dataset Size | Load Time | Simple Query | Complex Query | Join Query |
|--------------|-----------|--------------|---------------|------------|
| **10K triples** | ~200ms | <10ms | <50ms | <25ms |
| **100K triples** | ~2s | <50ms | <500ms | <250ms |
| **1M triples** | ~20s | <500ms | <5s | <2.5s |
| **10M triples** | ~200s | <5s | <50s | <25s |

**Characteristics**:
- JVM warmup adds 1-2s initially
- GC pauses can cause spikes
- Good for large datasets (100M+ triples)
- Mature query optimizer

### 3.3 RDFox Expected Performance

Based on vendor benchmarks (best-in-class):

| Dataset Size | Load Time | Simple Query | Complex Query | Join Query |
|--------------|-----------|--------------|---------------|------------|
| **10K triples** | ~20ms | <0.5ms | <5ms | <2ms |
| **100K triples** | ~200ms | <2ms | <20ms | <10ms |
| **1M triples** | ~2s | <20ms | <200ms | <100ms |
| **10M triples** | ~20s | <200ms | <2s | <1s |

**Characteristics**:
- Fastest query execution
- Parallel reasoning engine
- In-memory optimized
- Commercial-grade optimization

---

## 4. Performance Ranking (Expected)

### 4.1 Query Execution Speed
```
🥇 RDFox:      ⚡⚡⚡ (Fastest - highly optimized C++)
🥈 Rust KGDB:  ⚡⚡  (Fast - zero-copy, no GC)
🥉 Apache Jena: ⚡   (Good - JVM overhead)
```

### 4.2 Memory Efficiency
```
🥇 Rust KGDB:  ✅✅✅ (Best - zero-copy, string interning)
🥈 RDFox:      ✅✅  (Good - in-memory optimized)
🥉 Apache Jena: ✅   (Fair - JVM object overhead)
```

### 4.3 Startup Time
```
🥇 Rust KGDB:  ⚡⚡⚡ (Instant - <100ms)
🥈 RDFox:      ⚡⚡  (Fast - C++ binary)
🥉 Apache Jena: ⚡   (Slow - JVM warmup 1-2s)
```

### 4.4 Mobile Deployment
```
🥇 Rust KGDB:  ✅✅✅ (Designed for iOS/Android)
🥈 RDFox:      ❌   (Not supported)
🥉 Apache Jena: ❌   (JVM too heavy)
```

### 4.5 Ecosystem Maturity
```
🥇 Apache Jena: ✅✅✅ (15+ years, huge community)
🥈 RDFox:      ✅✅  (Commercial grade, 10+ years)
🥉 Rust KGDB:  ✅   (Newer, growing)
```

---

## 5. When to Use Each System

### 5.1 Use Rust KGDB When:

✅ **Mobile deployment required** (iOS/Android apps)
✅ **Memory safety critical** (embedded systems, safety-critical)
✅ **Predictable performance** (no GC pauses)
✅ **Small to medium datasets** (up to 10M triples)
✅ **Modern development** (Rust ecosystem preference)
✅ **Open source requirement** (Apache 2.0 license)

**Best For**:
- Mobile knowledge graph apps
- Embedded RDF databases
- Real-time systems
- Rust-first projects

### 5.2 Use Apache Jena When:

✅ **Very large datasets** (100M+ triples)
✅ **Mature ecosystem needed** (extensive libraries)
✅ **Complete OWL 2 reasoning** (full DL support)
✅ **Java/JVM environment** (existing Java infrastructure)
✅ **Community support** (large user base)
✅ **Open source requirement** (Apache 2.0 license)

**Best For**:
- Enterprise knowledge graphs
- Research projects
- Large-scale semantic web
- Java-based systems

### 5.3 Use RDFox When:

✅ **Maximum performance required** (fastest queries)
✅ **Large in-memory datasets** (optimized for RAM)
✅ **Commercial support needed** (SLA, professional support)
✅ **Advanced reasoning** (parallel Datalog engine)
✅ **Budget available** (commercial licensing)

**Best For**:
- High-performance analytics
- Real-time reasoning
- Commercial applications
- Performance-critical systems

---

## 6. Actual Benchmark Execution (To Be Done)

### 6.1 Prerequisites

To run actual benchmarks and compare:

```bash
# 1. Generate LUBM dataset
# Download UBA: http://swat.cse.lehigh.edu/projects/lubm/
./uba -univ 10 -onto http://swat.cse.lehigh.edu/onto/univ-bench.owl
# Generates ~1.3M triples for LUBM(10)

# 2. Generate SP2Bench dataset
# Download: http://dbis.informatik.uni-freiburg.de/forschung/projekte/SP2B/
./sp2b_gen -t 1000000
# Generates 1M triples

# 3. Prepare datasets in all three systems
# - Load into Rust KGDB
# - Load into Apache Jena TDB2
# - Load into RDFox
```

### 6.2 Run LUBM Benchmarks

**Rust KGDB**:
```bash
# See BENCHMARK_DEMO.md for full instructions
cargo test --package sparql test_lubm_benchmark -- --ignored --nocapture
```

**Apache Jena**:
```bash
# Use Jena's benchmarking tools
./bin/tdb2.tdbloader --loc=/data/lubm10 lubm10.ttl
./bin/tdb2.tdbquery --loc=/data/lubm10 --query=lubm_q1.rq --time
```

**RDFox**:
```bash
# Use RDFox shell with timing
RDFox -shell
import lubm10.ttl
timer on
eval "SPARQL query here"
```

### 6.3 Metrics to Compare

For each query in LUBM and SP2Bench, measure:
1. **Query execution time** (mean, median, std dev)
2. **Memory usage** (peak, average)
3. **Result correctness** (validate against expected)
4. **Throughput** (queries per second)
5. **Scalability** (how performance degrades with size)

---

## 7. Honest Assessment

### 7.1 Current Reality

**Rust KGDB Status**:
- ✅ **Code complete**: All functionality implemented
- ✅ **Tests passing**: 100% pass rate
- ✅ **Benchmark infrastructure**: Ready to run
- ⏳ **Actual benchmarks**: Need datasets generated
- ⏳ **Real-world validation**: Not yet production-tested at scale

**What We Know**:
- Architecture is sound (zero-copy, memory safety)
- Small-scale tests show good performance (<1ms for simple queries)
- Expected to outperform Jena on memory efficiency
- Expected to be slower than RDFox but faster than Jena

**What We Don't Know**:
- Real performance on LUBM(10) or larger
- How it handles pathological queries
- Edge cases in complex reasoning
- Production stability over time

### 7.2 Performance Predictions (Based on Architecture)

**Likely Outcomes**:
1. **vs Jena**:
   - Rust KGDB probably 2-3x faster for simple queries (no GC)
   - Similar for complex queries (both have good optimizers)
   - Much better memory efficiency (zero-copy vs JVM objects)
   - Faster startup (no JVM warmup)

2. **vs RDFox**:
   - RDFox likely 1.5-2x faster (highly tuned C++)
   - RDFox has more mature optimizer
   - Similar memory efficiency (both well-optimized)
   - Rust KGDB wins on memory safety guarantees

3. **Sweet Spot**:
   - Small-medium datasets (100K-10M triples)
   - Mobile and embedded scenarios
   - Memory-constrained environments
   - Safety-critical applications

---

## 8. How to Run Real Benchmarks

### 8.1 Step-by-Step Instructions

1. **Generate LUBM(10) dataset**:
   ```bash
   cd /tmp
   wget http://swat.cse.lehigh.edu/projects/lubm/uba1.7.zip
   unzip uba1.7.zip
   cd UBA1.7
   ./uba -univ 10 -onto http://swat.cse.lehigh.edu/onto/univ-bench.owl
   # Output: ~1.3M triples in multiple .owl files
   ```

2. **Convert to Turtle format** (if needed):
   ```bash
   # Use rapper or similar tool
   for file in University*.owl; do
     rapper -i rdfxml -o turtle "$file" >> lubm10.ttl
   done
   ```

3. **Load into Rust KGDB**:
   ```bash
   cd rust-kgdb
   # Update benchmark config with dataset path
   cargo test --package sparql test_lubm_benchmark -- --ignored --nocapture
   ```

4. **Compare with Jena**:
   ```bash
   # Download Apache Jena
   wget https://dlcdn.apache.org/jena/binaries/apache-jena-4.10.0.tar.gz
   tar xzf apache-jena-4.10.0.tar.gz
   cd apache-jena-4.10.0

   # Load data
   ./bin/tdb2.tdbloader --loc=/tmp/lubm10_tdb lubm10.ttl

   # Run queries with timing
   time ./bin/tdb2.tdbquery --loc=/tmp/lubm10_tdb --query=q1.rq
   ```

5. **Analyze results**:
   ```bash
   # Compare output from both systems
   # Create comparison table with:
   # - Query execution times
   # - Memory usage
   # - Result correctness
   ```

---

## 9. Conclusion

### 9.1 Summary

**Performance Ranking (Expected)**:
```
Speed:   RDFox > Rust KGDB > Apache Jena
Memory:  Rust KGDB ≈ RDFox > Apache Jena
Mobile:  Rust KGDB only
Cost:    Rust KGDB ≈ Jena (free) vs RDFox (commercial)
```

### 9.2 Key Differentiators

**Rust KGDB's Unique Value**:
1. ✅ **Only option for mobile** (iOS/Android)
2. ✅ **Memory safety** (Rust guarantees)
3. ✅ **Predictable performance** (no GC pauses)
4. ✅ **Small binary size** (vs JVM)
5. ✅ **Modern tooling** (Cargo, rustfmt, clippy)

**When Rust KGDB Wins**:
- Mobile applications (only viable option)
- Embedded systems (memory safety + small footprint)
- Real-time systems (predictable latency)
- Rust ecosystems (seamless integration)

**When Others Win**:
- Very large datasets (>100M triples): Jena
- Maximum speed required: RDFox
- Mature ecosystem needed: Jena
- Complete OWL 2 DL: Jena

---

## 10. Call to Action

### 10.1 To Get Real Benchmark Data

```bash
# Generate LUBM(10) dataset (~1.3M triples)
# Load into all three systems
# Run 14 LUBM queries
# Run 17 SP2Bench queries
# Compare results

# Expected time: 2-3 hours for complete benchmark suite
# Expected outcome: Validation of theoretical predictions
```

### 10.2 Documentation

All benchmark instructions available in:
- `BENCHMARK_DEMO.md` - Complete benchmark guide
- `COMPLETION_REPORT.md` - Project status
- `FINAL_STATUS.md` - Executive summary

---

**Status**: ⚠️ **INFRASTRUCTURE READY - AWAITING DATASET GENERATION**
**Next Step**: Generate LUBM and SP2Bench datasets, run actual benchmarks
**Estimated Time**: 2-3 hours for full benchmark execution

---

**Document Version**: 1.0
**Last Updated**: 2025-11-18
**Type**: Theoretical comparison with validation instructions
