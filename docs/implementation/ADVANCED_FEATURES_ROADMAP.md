# Advanced Features Implementation Roadmap
**Goal**: Implement WCOJ and SIMD to achieve unprecedented performance

## Phase 1: WCOJ Algorithm (Worst-Case Optimal Joins)
**Timeline**: 2-3 weeks
**Impact**: 10-100x faster on star/cyclic queries

### Week 1: Foundation
**Files to Create/Modify**:
- `crates/wcoj/src/trie.rs` - Trie data structure for sorted access
- `crates/wcoj/src/leapfrog.rs` - LeapFrog iterator
- `crates/wcoj/src/intersection.rs` - Multi-way intersection

**Implementation Steps**:

#### 1.1 Trie Data Structure
```rust
// crates/wcoj/src/trie.rs
pub struct Trie<'a> {
    // Maps each level to sorted values
    // Level 0: Subjects, Level 1: Predicates, Level 2: Objects
    levels: Vec<TrieLevel<'a>>,
}

pub struct TrieLevel<'a> {
    // Sorted unique values at this level
    values: Vec<Node<'a>>,
    // Maps value -> children at next level
    children: HashMap<Node<'a>, Box<TrieLevel<'a>>>,
}

impl<'a> Trie<'a> {
    // Build trie from quad index (SPOC/POCS/etc)
    pub fn from_index(store: &QuadStore<'a>, index_type: IndexType) -> Self;

    // Core trie operations
    pub fn seek(&mut self, value: Node<'a>) -> bool;
    pub fn next(&mut self) -> Option<Node<'a>>;
    pub fn at_end(&self) -> bool;
}
```

#### 1.2 LeapFrog Iterator
```rust
// crates/wcoj/src/leapfrog.rs
pub struct LeapfrogIterator<'a> {
    // Multiple tries to intersect
    tries: Vec<Trie<'a>>,
    // Current position in each trie
    positions: Vec<usize>,
}

impl<'a> LeapfrogIterator<'a> {
    // Core LeapFrog operations
    pub fn leapfrog_seek(&mut self, target: Node<'a>) -> bool;
    pub fn leapfrog_next(&mut self) -> Option<Node<'a>>;

    // Key algorithm: find next common value across all tries
    fn find_intersection(&mut self) -> Option<Node<'a>>;
}
```

#### 1.3 Multi-way Intersection
```rust
// crates/wcoj/src/intersection.rs
pub struct WCOJIntersection<'a> {
    variables: Vec<Variable>,
    leapfrog_iters: Vec<LeapfrogIterator<'a>>,
}

impl<'a> WCOJIntersection<'a> {
    // Intersect multiple triple patterns
    pub fn from_bgp(bgp: &[TriplePattern], store: &QuadStore<'a>) -> Self;

    // Yield all solutions
    pub fn execute(&mut self) -> Vec<Binding<'a>>;
}
```

### Week 2: Integration with SPARQL

**Files to Modify**:
- `crates/sparql/src/optimizer.rs` - Add WCOJ cost model
- `crates/sparql/src/executor.rs` - Use WCOJ for complex BGPs

**Integration Points**:

```rust
// crates/sparql/src/optimizer.rs
impl QueryOptimizer {
    fn should_use_wcoj(&self, bgp: &[TriplePattern]) -> bool {
        // Use WCOJ when:
        // 1. Star query (many patterns share subject/object)
        // 2. Cyclic query (variables form cycles)
        // 3. More than 3 triple patterns
        let shared_vars = self.count_shared_variables(bgp);
        let is_star = self.is_star_query(bgp);
        let is_cyclic = self.is_cyclic_query(bgp);

        shared_vars >= 2 || is_star || is_cyclic || bgp.len() > 3
    }
}

// crates/sparql/src/executor.rs
impl<'a> Executor<'a> {
    fn execute_bgp_wcoj(&self, bgp: &[TriplePattern]) -> Vec<Binding<'a>> {
        let wcoj = WCOJIntersection::from_bgp(bgp, &self.store);
        wcoj.execute()
    }
}
```

### Week 3: Testing & Benchmarking

**Tests to Add**:
- `crates/wcoj/tests/trie_tests.rs` - Trie correctness
- `crates/wcoj/tests/leapfrog_tests.rs` - LeapFrog correctness
- `crates/wcoj/tests/wcoj_integration_tests.rs` - End-to-end

**Benchmarks**:
- Star queries (1 subject, many predicates)
- Cyclic queries (variables forming cycles)
- Complex 5-way joins

**Expected Results**:
- **Star queries**: 50-100x faster than nested loops
- **Cyclic queries**: 10-50x faster
- **5-way joins**: 20-80x faster

---

## Phase 2: SIMD Vectorization
**Timeline**: 1-2 weeks
**Impact**: 4-8x faster batch operations

### Week 1: Core SIMD Implementation

**Files to Modify**:
- `crates/storage/src/simd.rs` - Complete SIMD implementation
- `crates/storage/Cargo.toml` - Enable SIMD by default

**Implementation Steps**:

#### 2.1 SIMD Quad Encoding (AVX2 for x86_64)
```rust
// crates/storage/src/simd.rs
#[target_feature(enable = "avx2")]
unsafe fn encode_quads_simd_avx2(quads: &[Quad], index_type: IndexType) -> Vec<SmallVec<[u8; 256]>> {
    let mut results = Vec::with_capacity(quads.len());

    // Process 8 quads at a time (256 bits / 32 bytes = 8 quads)
    for chunk in quads.chunks(8) {
        // Load 8 subject pointers into AVX2 register
        let subjects = _mm256_loadu_si256(/* subject pointers */);

        // Encode in parallel
        let encoded = encode_parallel_avx2(subjects, /* predicates, objects */);

        results.extend_from_slice(&encoded);
    }

    results
}
```

#### 2.2 SIMD Quad Encoding (NEON for aarch64/Apple Silicon)
```rust
#[target_feature(enable = "neon")]
unsafe fn encode_quads_simd_neon(quads: &[Quad], index_type: IndexType) -> Vec<SmallVec<[u8; 256]>> {
    let mut results = Vec::with_capacity(quads.len());

    // Process 4 quads at a time (128 bits / 32 bytes = 4 quads)
    for chunk in quads.chunks(4) {
        // Load 4 subject pointers into NEON register
        let subjects = vld1q_u64(/* subject pointers */);

        // Encode in parallel
        let encoded = encode_parallel_neon(subjects, /* predicates, objects */);

        results.extend_from_slice(&encoded);
    }

    results
}
```

#### 2.3 SIMD Filtering
```rust
// Vectorized FILTER evaluation for numeric comparisons
#[target_feature(enable = "avx2")]
unsafe fn filter_numeric_simd(values: &[f64], threshold: f64, op: ComparisonOp) -> Vec<bool> {
    let threshold_vec = _mm256_set1_pd(threshold); // Broadcast threshold
    let mut results = Vec::with_capacity(values.len());

    for chunk in values.chunks(4) { // 4 x f64 = 256 bits
        let vals = _mm256_loadu_pd(chunk.as_ptr());

        let mask = match op {
            ComparisonOp::LessThan => _mm256_cmp_pd(vals, threshold_vec, _CMP_LT_OQ),
            ComparisonOp::GreaterThan => _mm256_cmp_pd(vals, threshold_vec, _CMP_GT_OQ),
            ComparisonOp::Equal => _mm256_cmp_pd(vals, threshold_vec, _CMP_EQ_OQ),
            // ... other ops
        };

        // Extract mask to boolean vector
        let mask_bits = _mm256_movemask_pd(mask);
        results.extend_from_slice(&extract_bits(mask_bits));
    }

    results
}
```

### Week 2: Integration & Benchmarking

**Integration Points**:
- `crates/storage/src/quad_store.rs` - Use SIMD for bulk insert
- `crates/sparql/src/executor.rs` - Use SIMD for FILTER evaluation

**Enable by Default**:
```toml
# crates/storage/Cargo.toml
[features]
default = ["in-memory", "simd"]  # Add simd to default!
```

**Benchmarks to Add**:
- `benches/simd_encoding_benchmark.rs` - Quad encoding speed
- `benches/simd_filtering_benchmark.rs` - FILTER performance

**Expected Results**:
- **Bulk insert**: 146K → 580K triples/sec (4x faster)
- **Numeric filters**: 4-8x faster
- **Batch operations**: 6x faster average

---

## Success Metrics

### WCOJ Implementation
- [ ] All tests passing (trie, leapfrog, integration)
- [ ] 50x faster on star queries (benchmark proof)
- [ ] 20x faster on 5-way joins
- [ ] Auto-detection in query optimizer

### SIMD Implementation
- [ ] 4x faster bulk insert (benchmark proof)
- [ ] 6x faster numeric filtering
- [ ] Works on both x86_64 (AVX2) and aarch64 (NEON)
- [ ] Enabled by default in release builds

### Final Validation
- [ ] All 650+ tests still passing
- [ ] New benchmarks in `docs/benchmarks/`
- [ ] Updated npm README with honest claims
- [ ] Published v0.2.0 with "Now with WCOJ + SIMD"

---

## Risk Mitigation

### WCOJ Risks
**Risk**: Complex algorithm, potential bugs
**Mitigation**:
- Extensive unit tests at each layer (trie, leapfrog, intersection)
- Property-based testing with proptest
- Compare results against nested loop join (must be identical)

### SIMD Risks
**Risk**: Platform-specific code, portability issues
**Mitigation**:
- Feature flag allows disabling SIMD
- Scalar fallback for unsupported platforms
- Runtime CPU feature detection

---

## After Implementation: Marketing Claims

### What We Can Say (v0.2.0+)
✅ **WCOJ Algorithm**:
- "Asymptotically optimal multi-way joins"
- "50-100x faster on star queries" (benchmark-proven)
- "LeapFrog TrieJoin algorithm"

✅ **SIMD Vectorization**:
- "4-8x faster batch operations" (benchmark-proven)
- "Platform-optimized (AVX2, NEON)"
- "580K triples/sec bulk insert" (up from 146K)

✅ **Combined Impact**:
- "Fastest RDF database on the market" (measured)
- "10-100x faster than market leaders on complex queries"

---

## Development Order

**Priority 1: WCOJ** (bigger impact on complex queries)
1. Week 1: Trie + LeapFrog
2. Week 2: Integration
3. Week 3: Testing

**Priority 2: SIMD** (easier, clear wins)
1. Week 1: Core implementation
2. Week 2: Benchmarks + validation

**Total Timeline**: 4-5 weeks to market dominance

---

**Next Step**: Start with WCOJ trie implementation?
