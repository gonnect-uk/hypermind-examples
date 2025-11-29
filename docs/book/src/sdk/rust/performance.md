# Rust SDK Performance Guide

Strategies for optimizing rust-kgdb applications for maximum performance.

## Performance Benchmarks

Measured on Apple Silicon with InMemory backend:

| Operation | Speed | Throughput | Notes |
|-----------|-------|-----------|-------|
| Single Triple Lookup | 2.78 µs | 359K/sec | SPOC index |
| BGP Query (3 triples) | <100 µs | | Index scans + join |
| Bulk Insert 100K | 682 ms | 146K/sec | Batch operations |
| Dictionary Lookup (cached) | 60.4 µs | 1.66M/sec | String interning |
| Memory per Triple | 24 bytes | | 25% better than RDFox |

See `BENCHMARK_RESULTS_REPORT.md` in docs for full analysis.

## Optimization Techniques

### 1. Choose the Right Backend

**InMemory** for optimal speed:
```rust
// Fastest for reads/queries
let backend = InMemoryBackend::new();
let store = Arc::new(backend);

let executor = Executor::new(store);
let results = executor.execute_query(query, &dict)?;  // 2.78 µs/lookup
```

**RocksDB** for balanced performance:
```rust
// Good throughput with persistence
let backend = RocksDBBackend::new("./db")?;

// Speeds up with larger dataset sizes and good cache locality
```

**LMDB** for read-heavy loads:
```rust
// Memory-mapped for consistent read performance
let backend = LMDBBackend::new("./mmapped-db")?;
```

### 2. Efficient String Interning

Reuse interned strings to reduce memory allocation:

**Good:**
```rust
// Intern once, reuse many times
let rdf_type = dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
let name_pred = dict.intern("http://example.org/name");

for person_id in 0..100_000 {
    let subject = dict.intern(&format!("http://example.org/p{}", person_id));

    let type_triple = Triple::new(subject, rdf_type, person_class);
    let name_triple = Triple::new(subject, name_pred, name_value);

    store.put(&type_triple)?;
    store.put(&name_triple)?;
}
```

**Memory Savings**: ~2 bytes per reuse (instead of string duplication)

### 3. Batch Operations

Process data in optimal batch sizes:

```rust
const BATCH_SIZE: usize = 10_000;

fn bulk_load(store: &Arc<dyn StorageBackend>, triples: Vec<Triple>) -> Result<()> {
    for chunk in triples.chunks(BATCH_SIZE) {
        for triple in chunk {
            store.put(triple)?;
        }
        store.commit()?;  // Flush batch
    }
    Ok(())
}
```

**Performance Impact**: 10-20% faster bulk inserts

### 4. Query Pattern Optimization

Order patterns by selectivity (most to least restrictive):

**Optimized Query:**
```sparql
PREFIX ex: <http://example.org/>

SELECT ?name WHERE {
    # Most selective: specific value match
    ?person ex:department "Engineering" .

    # Moderately selective: range filter
    ?person ex:age ?age .
    FILTER (?age > 30 && ?age < 65) .

    # Least selective: general pattern
    ?person ex:name ?name .
}
```

**Impact**: 5-10x faster query execution

### 5. Index-Friendly Triple Patterns

Use patterns that align with SPOC indexes:

**Patterns Ranked by Speed:**

1. **Fastest** - All bound: `<s> <p> <o>`
   ```sparql
   ASK { <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> }
   ```

2. **Fast** - Subject + Predicate: `?o` where `<s> <p> ?o`
   ```sparql
   SELECT ?friend WHERE { <http://example.org/Alice> <http://example.org/knows> ?friend }
   ```

3. **Fast** - Subject only: `?p ?o` where `<s> ?p ?o`
   ```sparql
   SELECT ?predicate ?object WHERE { <http://example.org/Alice> ?predicate ?object }
   ```

4. **Moderate** - Predicate + Object: `?s` where `?s <p> <o>`
   ```sparql
   SELECT ?person WHERE { ?person <http://example.org/type> <http://example.org/Person> }
   ```

5. **Slower** - Subject + Object: `?p` where `<s> ?p <o>`
   ```sparql
   SELECT ?relation WHERE { <http://example.org/Alice> ?relation <http://example.org/Bob> }
   ```

6. **Slowest** - Only subject: `?p ?o` where `<s> ?p ?o`
   ```sparql
   SELECT ?anything WHERE { <http://example.org/Alice> ?p ?o }
   ```

7. **Slowest** - Full scan: `?s ?p ?o`
   ```sparql
   SELECT ?subject ?predicate ?object WHERE { ?subject ?predicate ?object }
   ```

### 6. Filter Placement

Place FILTER clauses after highly selective patterns:

**Optimized:**
```sparql
PREFIX ex: <http://example.org/>

SELECT ?name WHERE {
    # Selective pattern first
    ?person ex:type ex:HighValueCustomer .

    # Then filter
    ?person ex:name ?name .
    ?person ex:revenue ?revenue .
    FILTER (?revenue > 1000000) .
}
```

**Less Optimized:**
```sparql
SELECT ?name WHERE {
    ?person ex:name ?name .
    ?person ex:revenue ?revenue .
    FILTER (?revenue > 1000000) .  # Evaluated too late
    ?person ex:type ex:HighValueCustomer .
}
```

### 7. Avoid Property Paths When Possible

Direct patterns are faster than property paths:

**Fast:**
```sparql
# Direct triple pattern
SELECT ?friend WHERE {
    <http://example.org/Alice> <http://example.org/knows> ?friend
}
```

**Slower:**
```sparql
# Property path (requires graph traversal)
SELECT ?friend WHERE {
    <http://example.org/Alice> <http://example.org/knows>+ ?friend
}
```

### 8. Use CONSTRUCT Instead of SELECT + Processing

Build results in SPARQL rather than post-processing:

**Efficient:**
```sparql
# All work done in SPARQL
CONSTRUCT {
    ?person1 ex:colleague ?person2 .
}
WHERE {
    ?person1 ex:department ?dept .
    ?person2 ex:department ?dept .
    FILTER (?person1 != ?person2)
}
```

**Less Efficient:**
```rust
// SELECT then post-process in Rust
let results = executor.execute_query(
    "SELECT ?person1 ?person2 ?dept WHERE { ... }",
    &dict
)?;

// Process in application code
for binding in results {
    // ... expensive post-processing
}
```

### 9. Memory Efficiency

Monitor and minimize memory usage:

```rust
// Check memory before and after
fn estimate_memory_usage(dict: &Dictionary) -> usize {
    // Dictionary stores interned strings
    dict.estimate_memory() + 24 * triple_count
}

// Process in chunks for large files
for chunk in file_chunks(path, 1_000_000) {
    let triples = parse_chunk(&chunk)?;
    for triple in triples {
        store.put(&triple)?;
    }
    store.commit()?;

    // Free parsed data
    drop(triples);
}
```

### 10. Parallel Processing

Use Rayon for multi-threaded operations (requires `rayon` feature):

```rust
use rayon::prelude::*;

fn parallel_load(triples: Vec<Triple>, store: Arc<dyn StorageBackend>) -> Result<()> {
    // Partition work
    let chunks: Vec<Vec<Triple>> = triples
        .chunks(1000)
        .map(|c| c.to_vec())
        .collect();

    // Process in parallel
    let results: Result<Vec<()>, _> = chunks
        .into_par_iter()
        .try_for_each(|chunk| {
            for triple in chunk {
                store.put(&triple)?;
            }
            Ok(())
        });

    results?;
    store.commit()?;
    Ok(())
}
```

### 11. Connection Pooling

For production deployments, pool storage connections:

```rust
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

struct StoragePool {
    available: Mutex<VecDeque<Arc<dyn StorageBackend>>>,
    size: usize,
}

impl StoragePool {
    fn new(path: &str, pool_size: usize) -> Result<Self> {
        let mut available = VecDeque::new();
        for _ in 0..pool_size {
            available.push_back(Arc::new(
                RocksDBBackend::new(path)?
            ));
        }
        Ok(StoragePool {
            available: Mutex::new(available),
            size: pool_size,
        })
    }

    fn get(&self) -> Arc<dyn StorageBackend> {
        let mut pool = self.available.lock().unwrap();
        pool.pop_front()
            .unwrap_or_else(|| Arc::new(InMemoryBackend::new()))
    }

    fn return_connection(&self, conn: Arc<dyn StorageBackend>) {
        let mut pool = self.available.lock().unwrap();
        if pool.len() < self.size {
            pool.push_back(conn);
        }
    }
}
```

### 12. Query Result Caching

Cache expensive query results:

```rust
use std::collections::HashMap;
use std::sync::RwLock;

struct QueryCache {
    cache: RwLock<HashMap<String, Vec<Binding>>>,
}

impl QueryCache {
    fn get_or_execute(
        &self,
        query: &str,
        executor: &Executor,
        dict: &Dictionary,
    ) -> Result<Vec<Binding>> {
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.get(query) {
                return Ok(cached.clone());
            }
        }

        // Execute and cache
        let results = executor.execute_query(query, dict)?;

        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(query.to_string(), results.clone());
        }

        Ok(results)
    }
}
```

### 13. Lazy Evaluation

Stream results instead of materializing all at once:

```rust
impl Executor {
    pub fn stream_results(
        &self,
        query: &str,
        dict: &Dictionary,
    ) -> Result<impl Iterator<Item = Result<Binding>>> {
        // Returns iterator instead of Vec
        let results = self.execute_query(query, dict)?;
        Ok(results.into_iter().map(Ok))
    }
}
```

## Profiling Your Application

### Enable Debug Info in Release Build

```toml
[profile.release]
debug = true
opt-level = 3
```

### Profile with Flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bin your_app --release
# Generates flamegraph.svg
```

### Profile with Perf (Linux)

```bash
perf record --call-graph=dwarf ./target/release/your_app
perf report
```

## Expected Performance Targets

| Scenario | Expected Time | Optimization |
|----------|---------------|--------------|
| Load 100K triples | <1 second | Batch operations |
| Simple SPARQL query | <10 µs | Index alignment |
| Complex JOIN (5-way) | <10 ms | Selectivity ordering |
| Full text search | <100 ms | Filter optimization |

## Optimization Checklist

- [ ] Use InMemory for development, RocksDB for production
- [ ] Intern frequently used strings
- [ ] Process data in batches (1-10K triples)
- [ ] Order SPARQL patterns by selectivity
- [ ] Use aligned index patterns
- [ ] Place FILTERs after selective patterns
- [ ] Prefer direct patterns over property paths
- [ ] Use CONSTRUCT for result building
- [ ] Monitor memory usage
- [ ] Profile slow operations
- [ ] Cache expensive queries
- [ ] Use connection pooling
- [ ] Consider parallel processing
- [ ] Test with realistic data sizes

## Next Steps

- [Best Practices](./best-practices.md) - Design patterns
- [Code Examples](./examples.md) - Real-world usage
- [API Reference](./api.md) - Complete API
