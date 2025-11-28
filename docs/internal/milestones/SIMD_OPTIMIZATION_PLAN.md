# SIMD Optimization Plan - Rust KGDB

**Date**: 2025-11-26
**Goal**: Implement SIMD optimizations to beat RDFox bulk insert performance
**Target**: 146K → 190K+ triples/sec (30% improvement, Week 1 milestone)
**Status**: Planning Phase

---

## Executive Summary

### Current Performance (Benchmark Results 2025-11-18)
- ✅ **Lookup**: 2.78 µs (359K/sec) - **35-180x faster than RDFox**
- ⚠️ **Bulk Insert**: 146K triples/sec - **73% of RDFox (200-300K)**
- ✅ **Memory**: 24 bytes/triple - **25% better than RDFox (32 bytes)**
- ✅ **Dictionary**: 909K new/sec, 1.65M cached/sec

### SIMD Optimization Targets
**Week 1 Goal**: 146K → 190K triples/sec (+30% = Beat RDFox)

**Hot Paths Identified**:
1. **Node Encoding** (indexes.rs) - 40% of insert time
2. **Prefix Matching** (quad_store.rs) - 25% of query time
3. **Dictionary Lookup** (dictionary.rs) - 20% of insert time
4. **Pattern Matching** (executor.rs) - 15% of query time

---

## 1. SIMD Technology Selection

### Option A: std::simd (Portable SIMD) - RECOMMENDED
**Pros**:
- ✅ Official Rust standard library (std::simd)
- ✅ Portable across x86_64 (AVX2/AVX-512) and ARM (NEON)
- ✅ Type-safe, zero-cost abstraction
- ✅ Automatic fallback to scalar on unsupported platforms

**Cons**:
- ❌ Requires Rust nightly (not stable yet)
- ❌ API still evolving

**Decision**: Use std::simd for production-quality implementation.

### Option B: packed_simd - ALTERNATIVE
**Pros**:
- ✅ Stable Rust compatible
- ✅ Mature API

**Cons**:
- ❌ External dependency
- ❌ Less actively maintained

**Fallback**: If std::simd issues arise, switch to packed_simd.

---

## 2. Hot Path Analysis

### 2.1 Node Encoding (indexes.rs) - 40% Insert Time

**Current Implementation** (Scalar):
```rust
pub(crate) fn encode_node(buf: &mut SmallVec<[u8; 256]>, node: &Node) {
    let type_byte = match node {
        Node::Iri(_) => 0u8,
        Node::Literal(_) => 1u8,
        Node::BlankNode(_) => 2u8,
        // ... more matches
    };
    buf.push(type_byte);

    // Encode string data with varint length
    let bytes = node.value().as_bytes();
    encode_varint(buf, bytes.len() as u64);
    buf.extend_from_slice(bytes);
}
```

**SIMD Optimization Strategy**:
1. **Batch 4-16 nodes at once** using SIMD lanes
2. **Vectorize type byte assignment** (4x u8 → u8x4 SIMD)
3. **Parallel varint encoding** for length prefixes
4. **Memcpy acceleration** for string data (use SIMD aligned copies)

**Expected Speedup**: 2-3x (encode 4 nodes simultaneously)

### 2.2 Prefix Matching (quad_store.rs) - 25% Query Time

**Current Implementation** (Scalar):
```rust
fn build_scan_prefix(&self, pattern: &QuadPattern, index_type: IndexType) -> Vec<u8> {
    let mut prefix: SmallVec<[u8; 256]> = SmallVec::new();

    // Encode nodes until first wildcard
    if let NodePattern::Concrete(node) = &pattern.subject {
        encode_node(&mut prefix, node);
    } else {
        return prefix.to_vec();
    }
    // ... continue for predicate, object, graph
}
```

**SIMD Optimization Strategy**:
1. **Vectorized prefix comparison**: Compare 16 bytes at once (u8x16)
2. **Parallel wildcard detection**: Check all 4 quad positions simultaneously
3. **SIMD memcmp** for prefix matching during scan

**Expected Speedup**: 1.5-2x (16-byte parallel prefix checks)

### 2.3 Dictionary Lookup (dictionary.rs) - 20% Insert Time

**Current Implementation** (Scalar):
```rust
pub fn intern(&self, s: &str) -> &'static str {
    // Fast path: read lock + hash lookup
    {
        let guard = self.strings.read();
        if let Some(existing) = guard.get(s) {
            return unsafe { &*(Arc::as_ptr(existing) as *const str) };
        }
    }
    // Slow path: write lock + insert
    // ...
}
```

**SIMD Optimization Strategy**:
1. **SIMD hash function**: Use vectorized FxHash for strings
2. **Parallel string comparison**: Compare 16 bytes at once during hash collision resolution
3. **Cache-aligned reads**: Use SIMD to load 64-byte cache lines

**Expected Speedup**: 1.3-1.5x (faster hash + comparison)

### 2.4 Pattern Matching (executor.rs) - 15% Query Time

**Current Implementation** (Scalar):
```rust
impl<'a> Iterator for QuadIterator<'a, B> {
    fn next(&mut self) -> Option<Quad<'a>> {
        while self.position < self.results.len() {
            let key = &self.results[self.position];
            if let Ok(quad) = self.index_type.decode_key(key, self.dictionary) {
                if self.pattern.matches(&quad) {
                    return Some(quad);
                }
            }
        }
        None
    }
}
```

**SIMD Optimization Strategy**:
1. **Batch decode 4-8 quads at once** using SIMD
2. **Vectorized pattern matching**: Check subject/predicate/object/graph in parallel
3. **SIMD filtering**: Use mask operations to skip non-matches

**Expected Speedup**: 2-3x (batch decode + parallel match)

---

## 3. Implementation Strategy

### 3.1 Feature Flag Architecture

**Cargo.toml**:
```toml
[features]
default = []
simd = []          # Enable SIMD optimizations
simd-avx512 = []   # Enable AVX-512 (x86_64 only)
simd-neon = []     # Enable NEON (ARM only)
```

**Code Structure**:
```rust
#[cfg(feature = "simd")]
mod simd_ops {
    use std::simd::{u8x16, u8x4, Simd};

    pub fn encode_nodes_batch(nodes: &[Node]) -> Vec<u8> {
        // SIMD implementation
    }
}

#[cfg(not(feature = "simd"))]
mod simd_ops {
    pub fn encode_nodes_batch(nodes: &[Node]) -> Vec<u8> {
        // Fallback to scalar
    }
}
```

### 3.2 Phased Implementation

**Phase 1: Node Encoding SIMD** (Days 1-2)
- File: `crates/storage/src/simd_encode.rs`
- Functions:
  - `encode_nodes_batch_simd()` - Encode 4 nodes with u8x4
  - `encode_varint_batch_simd()` - Parallel varint encoding
  - `memcpy_aligned_simd()` - SIMD memory copy
- Tests: Unit tests with property-based testing (proptest)
- Benchmark: Compare vs scalar encode_node()

**Phase 2: Prefix Matching SIMD** (Days 3-4)
- File: `crates/storage/src/simd_prefix.rs`
- Functions:
  - `prefix_compare_simd()` - u8x16 prefix comparison
  - `wildcard_detect_simd()` - Parallel wildcard check
- Tests: Unit tests + pattern matching property tests
- Benchmark: Compare vs scalar build_scan_prefix()

**Phase 3: Dictionary SIMD** (Days 5-6)
- File: `crates/rdf-model/src/simd_hash.rs`
- Functions:
  - `hash_string_simd()` - Vectorized FxHash
  - `string_compare_simd()` - u8x16 string comparison
- Tests: Unit tests + concurrent stress tests
- Benchmark: Compare vs scalar intern()

**Phase 4: Pattern Matching SIMD** (Day 7)
- File: `crates/sparql/src/simd_match.rs`
- Functions:
  - `decode_quads_batch_simd()` - Batch decode 4-8 quads
  - `match_pattern_simd()` - Vectorized pattern matching
- Tests: Unit tests + SPARQL query tests
- Benchmark: Compare vs scalar QuadIterator

---

## 4. Testing Strategy

### 4.1 Unit Tests (Per SIMD Function)

**Test Coverage**:
- ✅ Correctness: SIMD results match scalar exactly
- ✅ Edge cases: Empty inputs, single elements, non-aligned sizes
- ✅ Cross-platform: x86_64 AVX2, ARM NEON, scalar fallback
- ✅ Performance: SIMD faster than scalar (assert speedup > 1.2x)

**Example Test**:
```rust
#[test]
fn test_encode_nodes_batch_simd_correctness() {
    let dict = Dictionary::new();
    let nodes = vec![
        Node::iri(dict.intern("http://ex.org/s1")),
        Node::iri(dict.intern("http://ex.org/s2")),
        Node::iri(dict.intern("http://ex.org/s3")),
        Node::iri(dict.intern("http://ex.org/s4")),
    ];

    // Encode with SIMD
    let simd_result = encode_nodes_batch_simd(&nodes);

    // Encode with scalar
    let mut scalar_result = Vec::new();
    for node in &nodes {
        encode_node(&mut scalar_result, node);
    }

    // Must match exactly
    assert_eq!(simd_result, scalar_result);
}
```

### 4.2 Property-Based Tests (Proptest/Quickcheck)

**Test Strategy**:
- Generate random nodes (IRIs, Literals, BlankNodes)
- Encode with SIMD and scalar
- Assert results are identical
- Run 1000+ random test cases

**Example**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_encode_simd_matches_scalar(
        uris in prop::collection::vec(any::<String>(), 0..100)
    ) {
        let dict = Dictionary::new();
        let nodes: Vec<_> = uris.iter()
            .map(|s| Node::iri(dict.intern(s)))
            .collect();

        let simd_result = encode_nodes_batch_simd(&nodes);
        let mut scalar_result = Vec::new();
        for node in &nodes {
            encode_node(&mut scalar_result, node);
        }

        prop_assert_eq!(simd_result, scalar_result);
    }
}
```

### 4.3 Benchmark Comparisons (Criterion)

**Benchmark Suite**:
```rust
// File: crates/storage/benches/simd_benchmark.rs

fn benchmark_node_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_encoding");

    // Setup test data
    let dict = Arc::new(Dictionary::new());
    let nodes: Vec<_> = (0..1000)
        .map(|i| Node::iri(dict.intern(&format!("http://ex.org/s{}", i))))
        .collect();

    // Scalar baseline
    group.bench_function("scalar", |b| {
        b.iter(|| {
            let mut buf = SmallVec::new();
            for node in &nodes {
                encode_node(&mut buf, node);
            }
            black_box(buf)
        });
    });

    // SIMD optimized
    group.bench_function("simd", |b| {
        b.iter(|| {
            let buf = encode_nodes_batch_simd(&nodes);
            black_box(buf)
        });
    });

    group.finish();
}
```

### 4.4 Cross-Platform Validation (CI)

**GitHub Actions Matrix**:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    arch: [x86_64, aarch64]
    features: [default, simd, simd-avx512, simd-neon]
```

**Test Commands**:
```bash
# Test all combinations
cargo test --features simd --target x86_64-unknown-linux-gnu
cargo test --features simd-avx512 --target x86_64-unknown-linux-gnu
cargo test --features simd-neon --target aarch64-unknown-linux-gnu
cargo test # Scalar fallback
```

---

## 5. Performance Metrics

### 5.1 Success Criteria

**Week 1 Milestone** (SIMD Node Encoding):
- ✅ Bulk insert: 146K → **190K triples/sec** (+30%)
- ✅ Node encoding: 2-3x speedup in microbenchmarks
- ✅ Zero regressions in existing tests
- ✅ Works on x86_64 AVX2 and ARM NEON

**Week 2 Milestone** (Full SIMD):
- ✅ Bulk insert: 190K → **270K triples/sec** (+85% total)
- ✅ Prefix matching: 1.5-2x speedup
- ✅ Dictionary lookup: 1.3-1.5x speedup
- ✅ Pattern matching: 2-3x speedup

### 5.2 Benchmark Targets

| Operation | Baseline | Week 1 Target | Week 2 Target | RDFox |
|-----------|----------|---------------|---------------|-------|
| **Bulk Insert** | 146K/sec | **190K/sec** | **270K/sec** | 200-300K/sec |
| **Node Encoding** | 1.0x | **2-3x** | **3-4x** | N/A |
| **Prefix Match** | 1.0x | **1.2x** | **1.5-2x** | N/A |
| **Dictionary** | 909K/sec | **1.2M/sec** | **1.4M/sec** | Unknown |
| **Pattern Match** | 1.0x | **1.2x** | **2-3x** | N/A |

### 5.3 Regression Tests

**Must NOT regress**:
- ✅ Lookup speed: Must stay at 2.78 µs or faster
- ✅ Memory usage: Must stay at 24 bytes/triple or better
- ✅ Test suite: 100% pass rate
- ✅ W3C conformance: All tests pass

---

## 6. Implementation Files

### New Files to Create

1. **`crates/storage/src/simd_encode.rs`** (Node encoding SIMD)
   - encode_nodes_batch_simd()
   - encode_varint_batch_simd()
   - memcpy_aligned_simd()

2. **`crates/storage/src/simd_prefix.rs`** (Prefix matching SIMD)
   - prefix_compare_simd()
   - wildcard_detect_simd()

3. **`crates/rdf-model/src/simd_hash.rs`** (Dictionary SIMD)
   - hash_string_simd()
   - string_compare_simd()

4. **`crates/sparql/src/simd_match.rs`** (Pattern matching SIMD)
   - decode_quads_batch_simd()
   - match_pattern_simd()

5. **`crates/storage/benches/simd_benchmark.rs`** (SIMD benchmarks)
   - Criterion benchmarks for all SIMD functions

6. **`crates/storage/tests/simd_tests.rs`** (SIMD integration tests)
   - Unit tests
   - Property-based tests
   - Cross-platform tests

### Modified Files

1. **`crates/storage/src/indexes.rs`**
   - Add `#[cfg(feature = "simd")]` conditional compilation
   - Call simd_encode functions when enabled

2. **`crates/storage/src/quad_store.rs`**
   - Add SIMD prefix matching when enabled
   - Fallback to scalar when disabled

3. **`crates/rdf-model/src/dictionary.rs`**
   - Add SIMD hash/compare when enabled
   - Keep scalar as default

4. **`crates/sparql/src/executor.rs`**
   - Add SIMD pattern matching when enabled
   - Batch decode optimization

5. **`Cargo.toml`** (workspace root)
   - Add SIMD feature flags
   - Add nightly toolchain config

---

## 7. Certification Checklist

### Pre-Merge Requirements

- [ ] **All existing tests pass** (cargo test --workspace)
- [ ] **New SIMD tests pass** (cargo test --features simd)
- [ ] **Benchmarks show >20% improvement** (cargo bench)
- [ ] **Cross-platform CI passes** (x86_64, ARM, Windows, Linux, macOS)
- [ ] **Documentation complete** (rustdoc + SIMD_TEST_REPORT.md)
- [ ] **Code review approved** (at least 1 reviewer)
- [ ] **Performance regression tests pass** (no slowdowns)
- [ ] **Feature flag works correctly** (builds with and without SIMD)

### Benchmark Certification

- [ ] **Bulk insert**: >190K triples/sec (Week 1) or >270K triples/sec (Week 2)
- [ ] **Node encoding**: >2x speedup in microbenchmark
- [ ] **Lookup speed**: No regression (must stay ≤2.78 µs)
- [ ] **Memory usage**: No regression (must stay ≤24 bytes/triple)
- [ ] **Dictionary performance**: No regression (must stay ≥909K/sec new)

### Documentation Certification

- [ ] **SIMD_TEST_REPORT.md**: Complete test results
- [ ] **Benchmark comparison**: SIMD vs scalar with charts
- [ ] **Cross-platform validation**: Test results on x86_64, ARM, scalar
- [ ] **Performance analysis**: Speedup breakdown by component
- [ ] **Usage instructions**: How to enable SIMD features

---

## 8. Risk Mitigation

### Risk 1: SIMD Complexity
**Mitigation**: Start with simplest SIMD operations (node type encoding), validate correctness before optimization.

### Risk 2: Cross-Platform Issues
**Mitigation**: Test on x86_64 AVX2 AND ARM NEON from day 1. Always maintain scalar fallback.

### Risk 3: Performance Regression
**Mitigation**: Run full benchmark suite before/after SIMD. Use Criterion's statistical analysis.

### Risk 4: API Instability (std::simd nightly)
**Mitigation**: Lock Rust nightly version. Have packed_simd as backup plan.

### Risk 5: Alignment Requirements
**Mitigation**: Use SmallVec with aligned buffers. Add alignment checks in tests.

---

## 9. Action Items

### Day 1-2: Node Encoding SIMD
- [ ] Create `crates/storage/src/simd_encode.rs`
- [ ] Implement `encode_nodes_batch_simd()` with u8x4
- [ ] Add unit tests + property tests
- [ ] Add Criterion benchmark
- [ ] Validate on x86_64 and ARM

### Day 3-4: Prefix Matching SIMD
- [ ] Create `crates/storage/src/simd_prefix.rs`
- [ ] Implement `prefix_compare_simd()` with u8x16
- [ ] Add tests + benchmarks
- [ ] Integrate into `quad_store.rs`

### Day 5-6: Dictionary SIMD
- [ ] Create `crates/rdf-model/src/simd_hash.rs`
- [ ] Implement `hash_string_simd()`
- [ ] Add tests + benchmarks
- [ ] Integrate into `dictionary.rs`

### Day 7: Integration + Testing
- [ ] Run full benchmark suite
- [ ] Generate SIMD_TEST_REPORT.md
- [ ] Validate all tests pass
- [ ] Create PR for review

### Day 8: Review + Merge
- [ ] Code review
- [ ] Address feedback
- [ ] Final benchmark run
- [ ] Merge to main
- [ ] Tag release: v0.2.0-simd

---

## 10. Expected Outcomes

### Week 1 Success Metrics
- ✅ **Bulk insert**: 146K → 190K triples/sec (+30%)
- ✅ **Beat RDFox lower bound**: 190K vs 200K (95% competitive)
- ✅ **Zero regressions**: All tests pass, lookup speed maintained
- ✅ **Production-ready**: Feature flag, cross-platform, documented

### Week 2 Success Metrics
- ✅ **Bulk insert**: 190K → 270K triples/sec (+85% total)
- ✅ **Beat RDFox decisively**: 270K vs 200-300K (competitive or better)
- ✅ **Comprehensive SIMD**: All hot paths optimized
- ✅ **Full certification**: Tests, benchmarks, docs complete

### Long-Term Impact
- 🎯 **Establish Rust KGDB as fastest RDF store**
- 🎯 **Demonstrate Rust's SIMD capabilities**
- 🎯 **Set foundation for further optimizations** (parallel joins, WCOJ)
- 🎯 **Prove mobile RDF is viable** (performance + safety)

---

## Appendix A: SIMD Instruction Reference

### x86_64 AVX2 (256-bit)
- **u8x32**: 32 bytes at once
- **_mm256_loadu_si256**: Unaligned load
- **_mm256_cmpeq_epi8**: Parallel byte comparison
- **_mm256_movemask_epi8**: Extract comparison mask

### ARM NEON (128-bit)
- **uint8x16_t**: 16 bytes at once
- **vld1q_u8**: Unaligned load
- **vceqq_u8**: Parallel byte comparison
- **vmovn_u16**: Extract comparison mask

### Rust std::simd (Portable)
- **u8x16, u8x32**: Platform-independent SIMD types
- **Simd::from_slice()**: Load from slice
- **simd.simd_eq()**: Parallel comparison
- **simd.to_bitmask()**: Extract mask

---

## Appendix B: References

1. **std::simd documentation**: https://doc.rust-lang.org/std/simd/
2. **packed_simd crate**: https://crates.io/crates/packed_simd
3. **Intel Intrinsics Guide**: https://www.intel.com/content/www/us/en/docs/intrinsics-guide/
4. **ARM NEON Guide**: https://developer.arm.com/architectures/instruction-sets/simd-isas/neon
5. **RDFox Performance Paper**: "RDFox: A Highly-Scalable RDF Store" (2015)
6. **WCOJ Paper**: "Worst-Case Optimal Join Algorithms" (Ngo et al. 2018)

---

**END OF SIMD OPTIMIZATION PLAN**
