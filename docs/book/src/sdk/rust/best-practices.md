# Rust SDK Best Practices

Proven patterns and recommendations for building robust rust-kgdb applications.

## 1. Dictionary Lifecycle

Always create the Dictionary once and share it across your application.

**Good:**
```rust
let dict = Arc::new(Dictionary::new());

// Share dict everywhere
let executor = Executor::new(store);
executor.execute_query(query, &dict)?;

// Pass to other modules
process_results(&dict, results)?;
```

**Bad:**
```rust
// Creating new dictionary each time wastes memory
let dict1 = Dictionary::new();
let dict2 = Dictionary::new();
// dict1 and dict2 don't share interned strings!
```

## 2. Storage Backend Selection

Choose the right backend for your use case:

**InMemory**: Development, testing, temporary data
```rust
let backend = InMemoryBackend::new();
```

**RocksDB**: Production, persistent, moderate-to-large datasets
```rust
let backend = RocksDBBackend::new("./production-db")?;
```

**LMDB**: Read-heavy workloads, memory-mapped performance
```rust
let backend = LMDBBackend::new("./read-optimized-db")?;
```

## 3. Error Handling Pattern

Always propagate errors with context:

**Good:**
```rust
fn load_knowledge_base(path: &str) -> Result<Arc<dyn StorageBackend>> {
    let parser = TurtleParser::new();
    let triples = parser.parse_file(path)
        .map_err(|e| format!("Failed to parse {}: {}", path, e))?;

    let backend = RocksDBBackend::new("./kb")?;
    let store = Arc::new(backend);

    for triple in triples {
        store.put(&triple)
            .map_err(|e| format!("Failed to store triple: {}", e))?;
    }

    Ok(store)
}
```

**Bad:**
```rust
// Loses error context
let triples = parser.parse_file(path)?;
for triple in triples {
    store.put(&triple)?;  // Why did it fail?
}
```

## 4. Query Parameterization

Build queries dynamically, not with string concatenation:

**Good:**
```rust
fn find_person(store: &Arc<dyn StorageBackend>, name: &str) -> Result<Vec<Binding>> {
    // Use BIND for parameterization
    let query = format!(r#"
        PREFIX ex: <http://example.org/>
        SELECT ?person WHERE {{
            ?person ex:name ?name .
            BIND("{}" AS ?searchName)
            FILTER (?name = ?searchName)
        }}
    "#, name);

    let executor = Executor::new(Arc::clone(store));
    executor.execute_query(&query, &dict)
}
```

**Better (prevents injection):**
```rust
// Use VALUE clause
let query = format!(r#"
    PREFIX ex: <http://example.org/>
    SELECT ?person WHERE {{
        ?person ex:name ?name .
        VALUES ?searchName {{ "{}" }}
        FILTER (?name = ?searchName)
    }}
"#, name);
```

## 5. Memory Efficiency: String Interning

Leverage Dictionary for memory efficiency:

**Good:**
```rust
// Intern common strings once
let rdf_type = dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
let person_class = dict.intern("http://example.org/Person");

// Reuse in many triples (only 8 bytes per reference)
for id in 0..1000 {
    let subject = dict.intern(&format!("http://example.org/person/{}", id));
    let triple = Triple::new(
        subject,
        rdf_type,        // Reused
        person_class     // Reused
    );
    store.put(&triple)?;
}
```

**Bad:**
```rust
// Creating new Node each time wastes memory
for id in 0..1000 {
    let triple = Triple::new(
        Node::IRI(&format!("http://example.org/person/{}", id)),
        Node::IRI("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),  // Not interned!
        Node::IRI("http://example.org/Person")  // Not interned!
    );
}
```

## 6. Batch Operations

Process data in batches, not one at a time:

**Good:**
```rust
let batch_size = 1000;
let mut batch = Vec::new();

for triple in triples {
    batch.push(triple);

    if batch.len() >= batch_size {
        for t in batch.drain(..) {
            store.put(&t)?;
        }
        store.commit()?;
        println!("Committed {} triples", batch_size);
    }
}

// Flush remaining
for t in batch {
    store.put(&t)?;
}
store.commit()?;
```

**Bad:**
```rust
// Commits on every triple (slow)
for triple in triples {
    store.put(&triple)?;
    store.commit()?;
}
```

## 7. Query Optimization

Order SPARQL patterns from most to least selective:

**Good:**
```sparql
# More selective patterns first
SELECT ?person WHERE {
    ?person <http://example.org/age> ?age .
    FILTER (?age > 65) .              # Filters early
    ?person <http://example.org/knows> ?friend .
    ?friend <http://example.org/name> "Bob" .
}
```

**Bad:**
```sparql
# Less selective patterns first
SELECT ?person WHERE {
    ?person <http://example.org/knows> ?friend .
    ?friend <http://example.org/knows> ?mutual .
    ?person <http://example.org/age> ?age .
    FILTER (?age > 65) .              # Filters too late
}
```

## 8. Resource Management

Use Arc for shared ownership, clean up when done:

**Good:**
```rust
fn process_large_file(path: &str) -> Result<()> {
    let dict = Arc::new(Dictionary::new());
    let backend = RocksDBBackend::new("./temp-db")?;
    let store = Arc::new(backend);

    // Process...

    // Explicit cleanup
    drop(store);  // Closes database handles

    // Remove temporary database
    std::fs::remove_dir_all("./temp-db")?;

    Ok(())
}
```

## 9. Testing Patterns

Use InMemory for tests, RocksDB for integration tests:

**Unit Test:**
```rust
#[test]
fn test_query_execution() {
    let dict = Arc::new(Dictionary::new());
    let store = Arc::new(InMemoryBackend::new());

    // Add test data
    let subject = dict.intern("http://example.org/test");
    let predicate = dict.intern("http://example.org/type");
    let object = dict.intern("http://example.org/TestClass");

    store.put(&Triple::new(subject, predicate, object)).unwrap();

    // Execute query
    let executor = Executor::new(store);
    let results = executor.execute_query(
        "SELECT ?x WHERE { ?x ?p ?o }",
        &dict
    ).unwrap();

    assert_eq!(results.len(), 1);
}
```

**Integration Test:**
```rust
#[test]
fn test_persistence() {
    let store = Arc::new(RocksDBBackend::new("./test-db").unwrap());

    // Write data
    let subject = dict.intern("http://example.org/persistent");
    store.put(&Triple::new(subject, p, o)).unwrap();
    store.commit().unwrap();

    drop(store);  // Close database

    // Reopen and verify
    let store = Arc::new(RocksDBBackend::new("./test-db").unwrap());
    let results = executor.execute_query("SELECT * WHERE { ?s ?p ?o }", &dict).unwrap();
    assert!(!results.is_empty());

    // Cleanup
    std::fs::remove_dir_all("./test-db").unwrap();
}
```

## 10. Performance Monitoring

Profile slow queries:

```rust
use std::time::Instant;

fn execute_with_timing(
    executor: &Executor,
    query: &str,
    dict: &Dictionary,
) -> Result<(Vec<Binding>, u128)> {
    let start = Instant::now();
    let results = executor.execute_query(query, dict)?;
    let elapsed = start.elapsed().as_micros();

    if elapsed > 10000 {  // > 10ms
        eprintln!("Slow query ({} µs): {}", elapsed, query);
    }

    Ok((results, elapsed))
}
```

## 11. Handling Large Results

Stream results instead of collecting all:

**Good (streams):**
```rust
let executor = Executor::new(store);
for result in executor.stream_results(query, &dict)? {
    println!("Result: {:?}", result?);
}
```

**Less efficient (collects all):**
```rust
let all_results = executor.execute_query(query, &dict)?;
for result in all_results {
    println!("Result: {:?}", result);
}
```

## 12. Concurrent Access

Use Arc<Mutex<>> for mutable state, Arc<> for immutable:

```rust
use std::sync::{Arc, Mutex};

let store = Arc::new(InMemoryBackend::new());
let dict = Arc::new(Dictionary::new());

let shared_state = Arc::new(Mutex::new(HashMap::new()));

// Safe to share across threads
let store_clone = Arc::clone(&store);
let state_clone = Arc::clone(&shared_state);

std::thread::spawn(move || {
    let mut state = state_clone.lock().unwrap();
    state.insert("key", "value");
});
```

## 13. Logging

Use structured logging for debugging:

```rust
use log::{debug, info, warn, error};

fn process_triple(store: &Arc<dyn StorageBackend>, triple: &Triple) -> Result<()> {
    debug!("Processing triple: {:?}", triple);

    store.put(triple)?;

    info!("Successfully stored triple");

    Ok(())
}
```

## 14. Configuration Management

Externalize configuration:

**config.toml:**
```toml
[database]
backend = "rocksdb"
path = "./production-db"

[query]
timeout_ms = 30000
max_results = 100000
```

**Code:**
```rust
fn load_config() -> Result<Config> {
    let config = std::fs::read_to_string("config.toml")?;
    toml::from_str(&config)
}
```

## Summary Checklist

- [ ] Create Dictionary once, share everywhere
- [ ] Choose appropriate storage backend
- [ ] Always propagate errors with context
- [ ] Parameterize queries (avoid string concatenation)
- [ ] Intern frequently used strings
- [ ] Process in batches
- [ ] Order SPARQL patterns by selectivity
- [ ] Use Arc for shared ownership
- [ ] Test with InMemory, verify with RocksDB
- [ ] Monitor slow queries
- [ ] Stream large result sets
- [ ] Use concurrent access patterns safely
- [ ] Log important operations
- [ ] Externalize configuration

## Next Steps

- [Performance Guide](./performance.md) - Optimization techniques
- [Code Examples](./examples.md) - Real-world patterns
- [API Reference](./api.md) - Complete function list
