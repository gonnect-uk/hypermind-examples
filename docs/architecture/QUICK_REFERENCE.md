# Enterprise Scale Design - Quick Reference

**Full Document**: [ENTERPRISE_SCALE_DESIGN.md](./ENTERPRISE_SCALE_DESIGN.md) (1,931 lines, 62 sections)
**Summary**: [DESIGN_EXECUTIVE_SUMMARY.md](./DESIGN_EXECUTIVE_SUMMARY.md)

---

## 10-Second Pitch

**rust-kgdb** will become the **fastest open-source SPARQL database** by combining:
1. **LeapFrog TrieJoin** (worst-case optimal joins)
2. **Apache DataFusion** (vectorized aggregations)
3. **HDRF Partitioning** (90% query locality)
4. **Arrow Flight** (zero-copy data exchange)

**Target**: 1B+ triples, <10ms p99 latency, 450K triples/sec insert

---

## Key Technical Decisions

### 1. Partitioning Strategy
**Choice**: Subject-anchored with HDRF (High-Degree Replicated First)

```rust
Partition ID = H(subject) mod num_partitions

Example:
<http://example.org/Person123> → Partition 5
<http://example.org/Org456> → Partition 12
```

**Replication**: High-degree subjects (>100 edges) replicated to 2-3 partitions

**Why**: 90% of SPARQL queries filter by subject ("Give me Alice's friends")

### 2. Dual-Path Query Engine

```
Query → Parser → Optimizer → Router
                               ├─→ OLTP (WCOJ) for point queries
                               └─→ OLAP (DataFusion) for aggregations
```

**OLTP Triggers**: <100K results, pattern matching, multi-way joins
**OLAP Triggers**: >100K results, GROUP BY, heavy aggregates

### 3. Storage Backend
**Choice**: RocksDB with 4 column families (SPOC, POCS, OCSP, CSPO)

**Key Encoding**:
```
SPOC: [partition_id:u16][subject_id:u64][predicate_id:u64][object_id:u64][graph_id:u64]
```

**Tuning**:
- Block cache: 4-8 GB shared
- Bloom filters: 10 bits/key (1% false positive)
- Write buffers: 256-512 MB

### 4. Coordination Layer
**Choice**: etcd cluster (3-5 nodes)

**Uses**:
- Distributed dictionary (URI → ID mapping)
- Partition metadata (home partition map)
- Leader election (coordinator high availability)
- Failure detection (heartbeats)

### 5. Data Exchange
**Choice**: Apache Arrow Flight

**Benefits**:
- Zero-copy IPC (no serialization)
- Columnar format (CPU cache friendly)
- 3-10x compression (dictionary + RLE)
- Language-agnostic (future Python/Java clients)

---

## Performance Targets

| Metric | Current | Target | RDFox |
|--------|---------|--------|-------|
| Lookup | 2.78 µs | <10 ms | 5-15 ms |
| Insert | 146K/sec | 450K/sec | 200K/sec |
| 4-way Join | 230 ms | <50 ms | ~100 ms |
| Memory | 24 bytes/triple | 30 bytes | 32 bytes |
| Scale | 10M | **1B+** | 10B+ |

---

## Competitive Matrix

|  | rust-kgdb | RDFox | Neo4j | Neptune |
|--|-----------|-------|-------|---------|
| **Cost** | Free | $$$$ | $$$$ | $$$$ |
| **SPARQL** | 100% | 99%+ | No | 95% |
| **WCOJ** | Yes | No | No | No |
| **Arrow** | Yes | No | No | No |
| **Open-Source** | Yes | No | No | No |

**Sweet Spot**: Organizations needing RDFox performance without license fees

---

## Architecture (ASCII Diagram)

```
┌───────────────────────────────────┐
│    Clients (SPARQL, gRPC, REST)   │
└───────────────┬───────────────────┘
                │
    ┌───────────▼───────────┐
    │  Coordinator Cluster  │
    │  (3 nodes + etcd)     │
    └───────────┬───────────┘
                │
      ┌─────────┼─────────┐
      │         │         │
  ┌───▼───┐ ┌──▼────┐ ┌──▼────┐
  │Part 0 │ │Part 1 │ │Part N │
  │       │ │       │ │       │
  │RocksDB│ │RocksDB│ │RocksDB│
  └───────┘ └───────┘ └───────┘
      │         │         │
      └─────────┼─────────┘
                │
       ┌────────▼────────┐
       │  Arrow Flight   │
       └─────────────────┘
```

**3-Layer Architecture**:
1. **Coordinator**: Query routing, result aggregation
2. **Executor**: Data storage, local execution
3. **Exchange**: Arrow Flight data transfer

---

## Implementation Phases

### Phase 1: Single-Node (3 months)
- SIMD vectorization
- Rayon parallelization
- RocksDB tuning
- **Goal**: Beat RDFox on LUBM

### Phase 2: Distributed (4 months)
- HDRF partitioner
- etcd integration
- Arrow Flight exchange
- **Goal**: 1B triples, <20ms p99

### Phase 3: DataFusion (3 months)
- OLAP path integration
- Cost-based routing
- SIMD kernels
- **Goal**: 10x aggregation speedup

### Phase 4: Production (3 months)
- Monitoring (Prometheus + Grafana)
- Security (TLS + OAuth2)
- Backup/recovery
- **Goal**: Enterprise-ready

**Total**: 12 months to v1.0

---

## Key Algorithms

### HDRF Partitioning
```rust
fn assign_triple(triple: &Triple) -> Vec<PartitionId> {
    let home = hash(triple.subject) % num_partitions;

    if degree(triple.subject) > 100 {
        // High-degree: replicate to 2-3 partitions
        vec![home, replica_1, replica_2]
    } else {
        vec![home]
    }
}
```

### Query Router
```rust
fn route_query(query: &SparqlQuery) -> ExecutionPath {
    if num_aggregates > 2 || cardinality > 100_000 {
        ExecutionPath::Olap // DataFusion
    } else if num_joins >= 3 {
        ExecutionPath::Oltp // WCOJ
    } else {
        ExecutionPath::Auto // Cost-based
    }
}
```

### LeapFrog TrieJoin
```rust
fn leapfrog_join(iterators: &mut [Trie], order: &[Var]) {
    while !at_end() {
        let max_key = iterators.iter().map(|it| it.key()).max();

        for it in iterators {
            it.seek(max_key);
        }

        if all_match() {
            emit_binding();
            advance_all();
        }
    }
}
```

---

## Configuration Examples

### OLTP-Optimized (Transactional)
```toml
[storage]
block_cache_size = "4GB"
bloom_filter_bits = 12

[query]
enable_datafusion = false
oltp_only = true

[replication]
mode = "strict"
sync_writes = true
```

### OLAP-Optimized (Analytics)
```toml
[storage]
block_cache_size = "16GB"
write_buffer_size = "1GB"

[query]
enable_datafusion = true
olap_threshold = 50000

[executor]
batch_size = 100000
parallel_scans = true

[replication]
mode = "eventual"
allow_stale_reads = true
```

---

## Benchmarks

### LUBM (Lehigh University Benchmark)

**Dataset**: LUBM(1000) = 133M triples

| Query | Target | RDFox | Description |
|-------|--------|-------|-------------|
| Q1 | 15 ms | 20 ms | Find all graduate students |
| Q2 | 45 ms | 60 ms | 4-way join (students + universities) |
| Q5 | 30 ms | 100 ms | Chain query (4 patterns) |
| Q14 | 180 ms | 250 ms | Transitive closure |

### SP2Bench (SPARQL Benchmark)

**Dataset**: 10M triples (DBLP publications)

| Query | Target | Description |
|-------|--------|-------------|
| Q1 | 120 ms | COUNT all triples |
| Q3a | 80 ms | Heavy OPTIONAL clauses |
| Q5a | 200 ms | GROUP BY + ORDER BY |

---

## Research Foundation

**18 Academic Papers + Industry Best Practices**:

1. RDFox DMAT (Potter et al., 2016) - Dynamic data exchange
2. HDRF (Petroni et al., 2015) - Graph partitioning
3. LeapFrog TrieJoin (Veldhuizen, 2014) - Worst-case optimal joins
4. DataFusion (Lamb et al., 2024) - Vectorized execution
5. Semi-Naive Datalog - Delta rule generation
6. Consistent Hashing - Virtual nodes for rebalancing
7. RocksDB Tuning - LSM-tree optimization
8. Kubernetes StatefulSet - Distributed database patterns
9. SIMD in Rust - AVX-512 vectorization
10. Sparse Matrix - CSR/CSC for graph operations

**Full References**: See [ENTERPRISE_SCALE_DESIGN.md](./ENTERPRISE_SCALE_DESIGN.md#references)

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| **Dictionary bottleneck** | LRU cache (10M entries) + etcd sharding |
| **Network saturation** | Arrow compression (3-10x) + batching |
| **Rebalancing downtime** | <5 min target, read-only mode |
| **etcd quorum loss** | 5-node cluster + hourly backups |
| **Complex deployment** | Helm charts + one-click install |

---

## Success Criteria

**Technical**:
- ✅ Beat RDFox on LUBM (single node)
- ✅ 1B+ triples, <20ms p99 latency
- ✅ <30 bytes/triple memory overhead
- ✅ 100% W3C SPARQL 1.1 compliance

**Business**:
- 🎯 10+ enterprise pilots by Q4 2026
- 🎯 1K+ GitHub stars, 50+ contributors
- 🎯 ISWC 2026 paper acceptance

---

## Immediate Next Steps

### Week 1-2: Validation
- Share with W3C RDF community
- Get feedback from RDFox team (Oxford)
- Validate Arrow integration with DataFusion maintainers

### Month 1: SIMD + Rayon
- Implement AVX-512 kernels
- Parallel bulk insert (8-16 threads)
- Establish LUBM baseline

### Quarter 1: RDFox Parity
- 450K triples/sec insert
- <50ms 4-way joins
- Publish benchmark report

---

**Document Status**: Ready for Review
**Target Audience**: CTOs, Systems Architects, RDF Researchers
**Confidence**: High (18 cited papers, 10+ years distributed systems research)

**Full Design**: [ENTERPRISE_SCALE_DESIGN.md](./ENTERPRISE_SCALE_DESIGN.md) (63 KB, 1,931 lines)
