# Error Handling

Guide to error handling and troubleshooting in rust-kgdb.

## Error Types

All rust-kgdb APIs return `Result<T>` with the following error types:

```rust
pub enum Error {
    StorageError(String),
    ParseError(String),
    QueryError(String),
    ReasoningError(String),
    IOError(io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Storage Errors

Occur when reading/writing to the triple store.

### Common Causes

| Error | Cause | Solution |
|-------|-------|----------|
| "Database locked" | Another process using DB | Close other processes |
| "Permission denied" | No write access | Check file permissions |
| "Out of memory" | Store too large | Use RocksDB instead of InMemory |
| "Corruption detected" | Database corruption | Rebuild database |

### Example Handling

```rust
use rust_kgdb::storage::InMemoryBackend;

match store.put(&triple) {
    Ok(_) => println!("Triple stored"),
    Err(Error::StorageError(msg)) => {
        eprintln!("Storage failed: {}", msg);
        // Retry or use fallback
    },
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

## Parse Errors

Occur when parsing RDF files or SPARQL queries.

### Common Causes

| Error | Cause | Solution |
|-------|-------|----------|
| "Unexpected token" | Syntax error in file | Check TTL/SPARQL syntax |
| "Unknown prefix" | Undefined namespace | Add `@prefix` declaration |
| "Invalid URI" | Malformed IRI | Fix URI syntax |
| "Unmatched quote" | Unclosed string literal | Close string properly |

### Example Handling

```rust
use rust_kgdb::rdf_io::TurtleParser;

let parser = TurtleParser::new();
match parser.parse_file("data.ttl") {
    Ok(triples) => println!("Parsed {} triples", triples.len()),
    Err(Error::ParseError(msg)) => {
        eprintln!("Parse failed: {}", msg);
        eprintln!("Check TTL syntax");
    },
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

### Fixing Parse Errors

**Common TTL syntax issues**:

```turtle
# Missing period
<http://example.org/s> <http://example.org/p> <http://example.org/o>
#                                                                   ^ Add .

# Undefined prefix
ex:resource ex:knows ex:person .
# ^ Add: @prefix ex: <http://example.org/> .

# Invalid literal
"String without type"^^<invalid-type> .
# Fix: "String"^^xsd:string .
```

## Query Errors

Occur when executing SPARQL queries.

### Common Causes

| Error | Cause | Solution |
|-------|-------|----------|
| "Unknown variable" | Variable used but not bound | Check variable scope |
| "Type mismatch" | Wrong type in function | Check function signature |
| "Invalid aggregate" | Aggregate outside GROUP BY | Use GROUP BY if needed |
| "Syntax error" | Invalid SPARQL syntax | Check SPARQL grammar |

### Example Handling

```rust
use rust_kgdb::sparql::Executor;

let query = "SELECT ?x WHERE { ?x <p> ?y FILTER (?x > 5) }";

match executor.execute_query(query, &dict) {
    Ok(results) => println!("Found {} results", results.len()),
    Err(Error::QueryError(msg)) => {
        eprintln!("Query failed: {}", msg);
        eprintln!("Check SPARQL syntax");
    },
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

### Debugging SPARQL Errors

```rust
// Enable logging for SPARQL parsing/execution
#[cfg(debug_assertions)]
{
    env_logger::builder()
        .filter_module("sparql", log::LevelFilter::Debug)
        .init();
}

// Now you'll see detailed SPARQL debug output
```

## Reasoning Errors

Occur during semantic inference.

### Common Causes

| Error | Cause | Solution |
|-------|-------|----------|
| "Cyclic ontology" | Circular inheritance | Fix ontology definition |
| "Invalid rule" | Malformed rule | Check reasoning rule syntax |
| "Resource limit" | Inference too expensive | Limit rule depth |

### Example Handling

```rust
use rust_kgdb::reasoning::RDFSReasoner;

let reasoner = RDFSReasoner::new(&store);
match reasoner.apply_rules() {
    Ok(_) => println!("Reasoning complete"),
    Err(Error::ReasoningError(msg)) => {
        eprintln!("Reasoning failed: {}", msg);
        // Fallback: query without reasoning
    },
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

## IO Errors

Occur during file operations.

### Common Causes

| Error | Cause | Solution |
|-------|-------|----------|
| "No such file" | File doesn't exist | Check file path |
| "Permission denied" | No read access | Check permissions |
| "Is a directory" | Tried to open folder | Specify file path |
| "Disk full" | Out of space | Free disk space |

## Error Recovery Patterns

### Retry with Backoff

```rust
fn execute_with_retry<T, F>(mut f: F) -> Result<T>
where
    F: FnMut() -> Result<T>
{
    let mut retries = 3;
    let mut backoff = Duration::from_millis(100);

    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if retries > 0 => {
                eprintln!("Error: {}. Retrying...", e);
                std::thread::sleep(backoff);
                backoff *= 2;
                retries -= 1;
            }
            Err(e) => return Err(e),
        }
    }
}

// Usage
let results = execute_with_retry(|| {
    executor.execute_query("SELECT ?x WHERE { ?x ?p ?o }", &dict)
})?;
```

### Fallback Strategy

```rust
fn query_with_fallback(executor: &Executor, query: &str, dict: &Dictionary) -> Result<Vec<Binding>> {
    // Try complex query
    match executor.execute_query(query, dict) {
        Ok(results) => Ok(results),
        Err(Error::QueryError(_)) => {
            // Fall back to simpler query
            eprintln!("Complex query failed, trying simplified version");
            executor.execute_query("SELECT ?x WHERE { ?x ?p ?o }", dict)
        }
        Err(e) => Err(e),
    }
}
```

### Error Logging and Reporting

```rust
use log::{debug, warn, error};

fn process_triple(store: &Arc<dyn StorageBackend>, triple: &Triple) -> Result<()> {
    match store.put(triple) {
        Ok(_) => {
            debug!("Stored triple: {:?}", triple);
            Ok(())
        }
        Err(Error::StorageError(msg)) => {
            error!("Storage failed for {:?}: {}", triple, msg);
            Err(Error::StorageError(msg))
        }
        Err(e) => {
            error!("Unexpected error: {}", e);
            Err(e)
        }
    }
}
```

## Panic Prevention

rust-kgdb APIs never panic in user code (panics indicate bugs). Handle errors instead:

```rust
// Good: Handles potential errors
fn load_data(path: &str) -> Result<usize> {
    let parser = TurtleParser::new();
    let triples = parser.parse_file(path)?;

    let store = InMemoryBackend::new();
    let mut count = 0;

    for triple in triples {
        match store.put(&triple) {
            Ok(_) => count += 1,
            Err(e) => warn!("Failed to store triple: {}", e),
        }
    }

    Ok(count)
}

// Bad: Panics on error
fn load_data_panicky(path: &str) -> usize {
    let parser = TurtleParser::new();
    let triples = parser.parse_file(path).unwrap();  // Panics on error!

    let store = InMemoryBackend::new();
    for triple in triples {
        store.put(&triple).unwrap();  // Panics on error!
    }

    triples.len()
}
```

## Debugging Techniques

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
RUST_LOG=sparql=trace cargo run
```

### Use Assertions in Development

```rust
#[cfg(debug_assertions)]
{
    let results = executor.execute_query(query, &dict)?;
    assert!(!results.is_empty(), "Expected non-empty results");
}
```

### Inspect Intermediate Values

```rust
let triples = parser.parse_file("data.ttl")?;
eprintln!("Parsed {} triples", triples.len());
eprintln!("First triple: {:?}", triples.first());

for triple in &triples {
    store.put(triple)?;
}
eprintln!("All triples stored successfully");
```

### Validate Data

```rust
fn validate_triple(triple: &Triple) -> Result<()> {
    // Custom validation
    match (&triple.subject, &triple.predicate, &triple.object) {
        (Node::IRI(s), Node::IRI(p), _) if !s.starts_with("http") => {
            Err(Error::ParseError("Subject URI must use http scheme".into()))
        }
        _ => Ok(()),
    }
}
```

## Common Error Messages

### "Key not found"
- Trying to get data that doesn't exist
- **Fix**: Check if key exists before accessing

### "Cannot infer type"
- Type checker can't determine type
- **Fix**: Use explicit type annotations

### "Index out of bounds"
- Accessing invalid index
- **Fix**: Check bounds before accessing

### "Deadlock detected"
- Circular lock dependencies
- **Fix**: Acquire locks in consistent order

## Performance Under Errors

Error handling adds minimal overhead:
- Result enum: 1-2 CPU cycles
- Error propagation: Single branch
- Logging: Only when enabled

## See Also

- [Best Practices](../sdk/rust/best-practices.md) - Error handling patterns
- [Code Examples](../sdk/rust/examples.md) - Real usage patterns
- [API Reference](./api.md) - Function signatures
