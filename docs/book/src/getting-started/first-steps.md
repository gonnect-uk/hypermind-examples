# First Steps Tutorial

A hands-on tutorial to learn rust-kgdb step-by-step. This guide demonstrates core operations: loading data, querying, and reasoning.

## Step 1: Setting Up Your Project

Start with the quick start setup:

```bash
cargo new my_kg_app
cd my_kg_app
```

Update `Cargo.toml`:
```toml
[dependencies]
rust-kgdb = { version = "0.1", features = ["all-backends"] }
tokio = { version = "1", features = ["full"] }
```

## Step 2: Creating a Triple Store

Create `src/main.rs` with basic store initialization:

```rust
use rust_kgdb::storage::InMemoryBackend;
use rust_kgdb::rdf_model::Dictionary;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a dictionary for string interning (memory efficient)
    let dict = Arc::new(Dictionary::new());

    // Create an in-memory triple store
    let backend = InMemoryBackend::new();
    let store = Arc::new(backend);

    println!("Triple store created!");

    Ok(())
}
```

Run: `cargo run`

## Step 3: Adding Triples

Extend the program to add RDF triples:

```rust
use rust_kgdb::rdf_model::{Triple, Node};

// Inside main(), after store creation:

// Create URIs
let alice = dict.intern("http://example.org/Alice");
let bob = dict.intern("http://example.org/Bob");
let knows = dict.intern("http://example.org/knows");
let name = dict.intern("http://example.org/name");

// Add triples
let triple1 = Triple::new(alice, knows, bob);
store.put(&triple1)?;

let name_alice = Node::Literal("Alice".to_string(), "http://www.w3.org/2001/XMLSchema#string".to_string());
let triple2 = Triple::new(alice, name, name_alice);
store.put(&triple2)?;

println!("Added 2 triples to the store");
```

## Step 4: Querying with SPARQL

Execute a simple SPARQL query:

```rust
use rust_kgdb::sparql::Executor;

// Inside main(), after adding triples:

let sparql_query = r#"
    SELECT ?person ?knows WHERE {
        ?person <http://example.org/knows> ?knows
    }
"#;

let executor = Executor::new(Arc::clone(&store));
let results = executor.execute_query(sparql_query, &dict)?;

println!("Query Results:");
for binding in results {
    println!("  {:?}", binding);
}
```

## Step 5: Using Different Backends

Switch between storage backends:

```rust
// InMemory (default, fastest)
let backend = InMemoryBackend::new();

// RocksDB (persistent, requires feature flag)
#[cfg(feature = "rocksdb-backend")]
let backend = RocksDBBackend::new("/path/to/db")?;

// LMDB (memory-mapped, requires feature flag)
#[cfg(feature = "lmdb-backend")]
let backend = LMDBBackend::new("/path/to/db")?;
```

## Step 6: Complex Queries

Try a more complex query with filters and joins:

```rust
let complex_query = r#"
    SELECT ?person ?knownBy WHERE {
        ?person <http://example.org/knows> <http://example.org/Bob> .
        ?knownBy <http://example.org/knows> ?person
        FILTER(?knownBy != <http://example.org/Alice>)
    }
    LIMIT 10
"#;

let results = executor.execute_query(complex_query, &dict)?;
println!("Complex query results: {:?}", results);
```

## Step 7: Loading RDF Files

Load data from Turtle files:

```rust
use rust_kgdb::rdf_io::TurtleParser;

let parser = TurtleParser::new();
let triples = parser.parse_file("data.ttl")?;

for triple in triples {
    store.put(&triple)?;
}
println!("Loaded {} triples from file", triples.len());
```

## Next Steps

- Explore the [Rust SDK API](../sdk/rust/api.md) for complete reference
- Learn about [Best Practices](../sdk/rust/best-practices.md)
- Review [Core Concepts](./core-concepts.md) for deeper understanding
- Check [Performance Guide](../sdk/rust/performance.md) for optimization tips
