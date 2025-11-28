# Hypergraph Test Suite Plan

## Executive Summary

This document outlines a comprehensive test plan for the rust-kgdb hypergraph crate (a cornerstone feature of the product). The plan targets **100+ tests** organized into 6 categories with 100% pass rate, covering all public APIs, edge cases, performance characteristics, and SPARQL integration.

## Hypergraph Implementation Overview

The rust-kgdb hypergraph (`crates/hypergraph/src/lib.rs`) is a production-grade implementation supporting:

- **Arbitrary arity hyperedges**: Not limited to binary edges like standard graphs
- **Directed and undirected semantics**: Flexible edge directionality
- **Labeled edges and metadata**: Optional labels for RDF integration
- **Efficient traversal and querying**: O(1) node/edge lookup, O(d) traversal
- **RDF* (quoted triples) integration**: Support for nested statements

### Key Data Structures

```rust
pub struct Hypergraph {
    nodes: FxHashMap<NodeId, Node>,
    hyperedges: FxHashMap<EdgeId, Hyperedge>,
    node_to_edges: FxHashMap<NodeId, FxHashSet<EdgeId>>,  // Index for fast traversal
    next_node_id: NodeId,
    next_edge_id: EdgeId,
}

pub struct Hyperedge {
    pub id: EdgeId,
    pub nodes: SmallVec<[NodeId; 4]>,  // Optimized for ≤4 nodes (common in RDF triples/quads)
    pub directed: bool,
    pub label: Option<String>,
    pub metadata: Option<Vec<u8>>,
}
```

### Public API (24 methods)

1. `new()` - Create empty hypergraph
2. `add_node()` - Add unlabeled node
3. `add_labeled_node(label)` - Add labeled node
4. `add_hyperedge(nodes, directed)` - Add hyperedge
5. `add_labeled_hyperedge(nodes, directed, label)` - Add labeled hyperedge
6. `get_node(id)` - Retrieve node by ID
7. `get_hyperedge(id)` - Retrieve hyperedge by ID
8. `get_incident_edges(node_id)` - Get all edges touching a node (O(1))
9. `find_edges(pattern)` - Pattern matching with wildcards
10. `get_neighbors(node_id)` - Get connected nodes
11. `subgraph(node_ids)` - Extract subgraph
12. `stats()` - Get hypergraph statistics
13. `bfs(start)` - Breadth-first search traversal
14. `shortest_path(start, end)` - Find shortest path

---

## Test Plan Structure (6 Categories)

### 1. Basic Hyperedge Operations (20 tests)
**Goal**: Verify core functionality of node and edge creation/retrieval.

**Tests**:
- ✅ Create empty hypergraph
- ✅ Add single node
- ✅ Add multiple nodes with unique IDs
- ✅ Add labeled node (label stored correctly)
- ✅ Add multiple labeled nodes
- ✅ Retrieve node by ID
- ✅ Retrieve non-existent node returns None
- ✅ Add binary hyperedge (2 nodes)
- ✅ Add ternary hyperedge (3 nodes)
- ✅ Add n-ary hyperedge (5+ nodes)
- ✅ Retrieve hyperedge by ID
- ✅ Retrieve non-existent edge returns None
- ✅ Add directed hyperedge
- ✅ Add undirected hyperedge
- ✅ Add labeled hyperedge (label stored)
- ✅ Edge ID uniqueness
- ✅ Node ID uniqueness
- ✅ Hyperedge contains correct nodes
- ✅ Hyperedge directed flag set correctly
- ✅ Multiple hyperedges with same nodes

**Key Assertions**:
- Node/Edge counts match additions
- IDs are unique and sequential
- Retrieval returns exact object added
- Labels are preserved

---

### 2. Multi-Node Connections (15 tests)
**Goal**: Verify hyperedge behavior with varying arities and configurations.

**Tests**:
- ✅ 2-node hyperedge (binary, like standard graphs)
- ✅ 3-node hyperedge (ternary, RDF triple pattern)
- ✅ 4-node hyperedge (quad, RDF with context)
- ✅ 5-node hyperedge (n-ary relation)
- ✅ 10-node hyperedge (large arity)
- ✅ Mixed arity: edges with different arities
- ✅ Single node hyperedge (self-loop)
- ✅ Hyperedge with duplicate nodes (same node multiple times)
- ✅ One node in multiple hyperedges
- ✅ Node incident to 5+ edges
- ✅ Directed edge directional semantics
- ✅ Undirected edge symmetric property
- ✅ All nodes must exist before adding edge
- ✅ Edge references valid nodes after creation
- ✅ Metadata attached to edges

**Key Assertions**:
- SmallVec optimization works (≤4 nodes)
- Arity (node count) correct for each edge
- Incident edges map maintained
- Directionality flag respected

---

### 3. Hypergraph Traversal (15 tests)
**Goal**: Verify graph traversal, navigation, and path-finding algorithms.

**Tests**:
- ✅ BFS from single node
- ✅ BFS visits all reachable nodes
- ✅ BFS order is breadth-first
- ✅ BFS on disconnected node
- ✅ BFS on isolated component
- ✅ Shortest path between adjacent nodes
- ✅ Shortest path multi-hop
- ✅ Shortest path start == end (self-path)
- ✅ Shortest path no path exists (disconnected)
- ✅ Get neighbors of single node
- ✅ Get neighbors: directed vs undirected
- ✅ Get neighbors with multiple hyperedges
- ✅ Incident edges returns all touching edges
- ✅ Incident edges empty for isolated node
- ✅ Degree distribution computation

**Key Assertions**:
- All reachable nodes visited
- Shortest path is actually shortest
- Neighbors exclude self-references
- Directionality affects neighbor results
- Isolated nodes have empty incident edge sets

---

### 4. Pattern Matching and Queries (15 tests)
**Goal**: Verify flexible pattern matching with wildcards (essential for SPARQL).

**Tests**:
- ✅ Find edges with fixed first position
- ✅ Find edges with fixed middle position (predicate)
- ✅ Find edges with fixed last position
- ✅ Find edges with all wildcards (all edges)
- ✅ Find edges with two fixed positions
- ✅ Find edges with one position match (no wildcards)
- ✅ Pattern match: no matches returns empty
- ✅ Pattern match ternary (3-node) edges
- ✅ Pattern match binary (2-node) edges
- ✅ Pattern match n-ary (5+ node) edges
- ✅ Multiple edges matching same pattern
- ✅ Pattern match with labeled edges
- ✅ Wildcard position independence
- ✅ Mixed arity: pattern doesn't match different arity
- ✅ Pattern matching is deterministic

**Key Assertions**:
- Pattern matching correct for all arities
- Wildcard (None) matches any node
- Fixed patterns (Some) must match exactly
- No false positives/negatives
- Results deterministic across calls

---

### 5. Subgraph Extraction (12 tests)
**Goal**: Verify subgraph operations for analysis and slicing.

**Tests**:
- ✅ Subgraph with single node
- ✅ Subgraph with two connected nodes
- ✅ Subgraph with disconnected nodes
- ✅ Subgraph: only edges with all nodes included
- ✅ Subgraph excludes external edges
- ✅ Subgraph preserves node labels
- ✅ Subgraph preserves edge labels
- ✅ Subgraph preserves node metadata
- ✅ Subgraph preserves edge metadata
- ✅ Subgraph preserves directionality
- ✅ Subgraph: node IDs remapped (new IDs)
- ✅ Subgraph of subgraph (recursive)

**Key Assertions**:
- Subgraph contains only specified nodes
- All edges have both endpoints in subgraph
- Labels and metadata preserved
- Node IDs are new (not same as original)
- Subgraph is independent copy

---

### 6. Statistics and Metadata (12 tests)
**Goal**: Verify statistics computation and metadata handling.

**Tests**:
- ✅ Stats: empty hypergraph
- ✅ Stats: node count
- ✅ Stats: edge count
- ✅ Stats: max arity (2-node, 3-node, 5-node)
- ✅ Stats: directed/undirected split
- ✅ Stats: all undirected edges
- ✅ Stats: all directed edges
- ✅ Stats: mixed directed/undirected
- ✅ Node metadata storage
- ✅ Edge metadata storage
- ✅ Node label retrieval
- ✅ Edge label retrieval

**Key Assertions**:
- Counts match actual structure
- Max arity correctly identified
- Directed/undirected counts sum to total edges
- Metadata preserved and retrievable
- Display format (Display trait) works

---

### 7. Edge Cases and Error Handling (11 tests)
**Goal**: Verify robustness with unusual inputs and error conditions.

**Tests**:
- ✅ Add hyperedge with empty node list
- ✅ Add hyperedge with non-existent nodes
- ✅ Large node IDs (u64::MAX)
- ✅ Large edge IDs (u64::MAX)
- ✅ 1000+ nodes in hypergraph
- ✅ 1000+ edges in hypergraph
- ✅ Very large node count in single edge (100+ nodes)
- ✅ Long label strings
- ✅ Large metadata blocks (MB-sized)
- ✅ Unicode in labels
- ✅ Default Hypergraph creation

**Key Assertions**:
- Empty edge list handling
- ID overflow behavior
- Scale limits respected
- No panics on edge cases

---

### 8. Performance Benchmarks (10 tests)
**Goal**: Verify performance meets production requirements.

**Run with**: `cargo bench --package hypergraph --bench hypergraph_benchmark`

**Benchmarks**:
- ⏱️ Add node: <1µs per node
- ⏱️ Add 2-node edge: <5µs
- ⏱️ Add 4-node edge: <10µs
- ⏱️ Retrieve node: <2µs
- ⏱️ Retrieve edge: <2µs
- ⏱️ Get incident edges: <1µs (indexed)
- ⏱️ BFS on 1000-node graph: <10ms
- ⏱️ Shortest path: <5ms
- ⏱️ Pattern matching: <2ms for 1000 edges
- ⏱️ Subgraph extraction: <5ms for 100-node subgraph

**Performance Targets**:
- O(1) node/edge lookup
- O(1) incident edges (via index)
- O(d) neighbor traversal where d = degree
- O(n) BFS where n = nodes visited
- O(m) pattern matching where m = edges

---

### 9. Integration with RDF Model (10 tests)
**Goal**: Verify hypergraph works with rust-kgdb RDF integration.

**Tests**:
- ✅ Hyperedge as RDF triple (S-P-O)
- ✅ Hyperedge as RDF quad (S-P-O-C)
- ✅ Labeled edges as RDF predicates
- ✅ Metadata as RDF annotations
- ✅ Pattern matching for SPARQL BGP
- ✅ Neighbor traversal as RDF property traversal
- ✅ Subgraph as named graph extraction
- ✅ Shortest path as property path (^)
- ✅ BFS as SPARQL CONSTRUCT result
- ✅ Statistics feed into query planning

**Key Assertions**:
- RDF semantics preserved
- URI formatting compatible
- Pattern matching compatible with BGP

---

### 10. Clone and Default Traits (5 tests)
**Goal**: Verify traits are correctly implemented.

**Tests**:
- ✅ Clone creates independent copy
- ✅ Clone: modifications don't affect original
- ✅ Default creates empty hypergraph
- ✅ Cloned hypergraph data is identical
- ✅ Display trait produces formatted output

**Key Assertions**:
- Clone deep copy verified
- Default == new()
- Display format is readable

---

## Test File Organization

```
crates/hypergraph/tests/
├── integration_test.rs          (main entry point)
├── basic_operations.rs          (Category 1: 20 tests)
├── multi_node_connections.rs    (Category 2: 15 tests)
├── traversal.rs                 (Category 3: 15 tests)
├── pattern_matching.rs          (Category 4: 15 tests)
├── subgraph_extraction.rs       (Category 5: 12 tests)
├── statistics_metadata.rs       (Category 6: 12 tests)
├── edge_cases.rs                (Category 7: 11 tests)
└── rdf_integration.rs           (Category 9: 10 tests)

crates/hypergraph/benches/
└── hypergraph_benchmark.rs      (Category 8: 10 benchmarks)
```

**Total Tests**: 110+ tests + 10 benchmarks

---

## Execution Strategy

### Phase 1: Core Functionality (Week 1)
```bash
# Write and test basic operations
cargo test -p hypergraph --lib                 # Unit tests in lib.rs
cargo test -p hypergraph basic_operations     # Category 1
cargo test -p hypergraph multi_node_connections # Category 2
```

### Phase 2: Traversal and Queries (Week 1-2)
```bash
cargo test -p hypergraph traversal             # Category 3
cargo test -p hypergraph pattern_matching      # Category 4
```

### Phase 3: Advanced Features (Week 2)
```bash
cargo test -p hypergraph subgraph_extraction   # Category 5
cargo test -p hypergraph statistics_metadata   # Category 6
```

### Phase 4: Edge Cases and Performance (Week 2-3)
```bash
cargo test -p hypergraph edge_cases           # Category 7
cargo bench -p hypergraph                     # Category 8
```

### Phase 5: Integration (Week 3)
```bash
cargo test -p hypergraph rdf_integration      # Category 9
```

### Full Test Suite
```bash
# Run all hypergraph tests
cargo test -p hypergraph

# Run all tests with output
cargo test -p hypergraph -- --nocapture

# Run specific test
cargo test -p hypergraph test_add_node

# Run benchmarks
cargo bench -p hypergraph
```

---

## Test Coverage Goals

| Category | Tests | Coverage | Status |
|----------|-------|----------|--------|
| Basic Operations | 20 | Core APIs | ✅ Planned |
| Multi-Node Connections | 15 | Arity handling | ✅ Planned |
| Traversal | 15 | Navigation | ✅ Planned |
| Pattern Matching | 15 | SPARQL BGP | ✅ Planned |
| Subgraph Extraction | 12 | Slicing | ✅ Planned |
| Statistics/Metadata | 12 | Metadata | ✅ Planned |
| Edge Cases | 11 | Robustness | ✅ Planned |
| Benchmarks | 10 | Performance | ✅ Planned |
| RDF Integration | 10 | Compatibility | ✅ Planned |
| Traits | 5 | Clone/Default | ✅ Planned |
| **TOTAL** | **125** | **100%** | ✅ **Comprehensive** |

---

## Success Criteria

1. **100% Pass Rate**: All 125 tests pass consistently
2. **Zero Panics**: No unhandled panics in any test
3. **Performance**: All benchmarks meet targets (O(1) / O(d) / O(n) complexity)
4. **Code Coverage**: >95% of hypergraph code covered by tests
5. **Documentation**: All tests have clear comments explaining what they verify
6. **RDF Compatibility**: Pattern matching works with SPARQL BGP patterns
7. **Production Ready**: No memory leaks, no unsafe code required

---

## Research Summary

### Existing Hypergraph Test Suites

**Rust Hypergraph Crates Reviewed**:
1. **yamafaktory/hypergraph** (crates.io)
   - 100% safe Rust
   - Proper error handling
   - Parallelism with Rayon
   - Uses tests directory (pattern we're following)

2. **open-hypergraphs** (crates.io)
   - Differentiable and data-parallel
   - Syntax focus

3. **mhgl** (Matt's HyperGraph Library)
   - Undirected hypergraph focus
   - ID-based node/edge references

**Python Libraries Reviewed**:
1. **HypergraphX (HGX)** - Oxford Academic publication
   - Hyperdegree distributions
   - Hyperedge centrality measures
   - Local and mesoscale statistics
   - Triangle counting in hypergraphs

2. **Hypergraph-DB**
   - Stress tests for scalability
   - Timing measurements
   - Vertex/hyperedge addition performance

3. **halp** (Hypergraph Algorithms Package)
   - Directed hypergraph support
   - Attribute metadata

### Standard Hypergraph Operations (From Academic Literature)

**Hypergraph Traversal** (ACM/IEEE research):
- B-connectivity: All nodes in edge tail visited before traversal
- Hyperpaths: Connections among nodes using hyperarcs
- Optimal path finding: NP-hard in general case

**Our Implementation Covers**:
- ✅ BFS traversal
- ✅ Shortest path finding
- ✅ Neighbor discovery
- ✅ Incident edge queries
- ✅ Pattern matching (SPARQL BGP semantics)

---

## Next Steps

1. **Create test files** (this sprint):
   - `crates/hypergraph/tests/integration_test.rs`
   - `crates/hypergraph/tests/basic_operations.rs`
   - `crates/hypergraph/tests/multi_node_connections.rs`
   - etc.

2. **Create benchmark file**:
   - `crates/hypergraph/benches/hypergraph_benchmark.rs`

3. **Run full test suite**:
   - Target: 125+ tests, 100% pass rate
   - Performance verified within targets

4. **Document results**:
   - Test coverage report
   - Performance report
   - Test execution log

---

## Contact & Questions

This test plan ensures rust-kgdb's hypergraph implementation (a cornerstone feature) is production-grade, fully tested, and meets all performance requirements.

**Generated**: 2025-11-25
**Crate**: rust-kgdb/crates/hypergraph
**Target**: 125 tests covering all public APIs and edge cases
