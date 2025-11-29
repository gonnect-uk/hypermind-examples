# Quick Start Guide

Get up and running with rust-kgdb in minutes. This guide will help you create your first RDF triple store and execute SPARQL queries.

## Prerequisites

- Rust 1.70+ ([install here](https://rustup.rs/))
- Cargo (included with Rust)
- Basic familiarity with RDF and SPARQL

## Installation

Add rust-kgdb to your `Cargo.toml`:

```toml
[dependencies]
rust-kgdb = { version = "0.1", features = ["all-backends"] }
tokio = { version = "1", features = ["full"] }
```

## Your First Program

Create a new Rust project:

```bash
cargo new my_kg_app
cd my_kg_app
```

Add the dependencies above to `Cargo.toml`, then create `src/main.rs`:

```rust
use rust_kgdb::storage::InMemoryBackend;
use rust_kgdb::rdf_model::{Node, Triple, Dictionary};
use rust_kgdb::sparql::Executor;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a dictionary for string interning
    let dict = Arc::new(Dictionary::new());

    // Initialize the in-memory backend
    let backend = InMemoryBackend::new();
    let store = Arc::new(backend);

    // Add some sample data
    let subject = dict.intern("http://example.org/Alice");
    let predicate = dict.intern("http://example.org/knows");
    let object = dict.intern("http://example.org/Bob");

    let triple = Triple::new(subject, predicate, object);
    store.put(&triple)?;

    // Query the data
    let sparql_query = r#"
        SELECT ?person ?knows WHERE {
            ?person <http://example.org/knows> ?knows
        }
    "#;

    let executor = Executor::new(store);
    let results = executor.execute_query(sparql_query, &dict)?;

    println!("Query results: {:?}", results);

    Ok(())
}
```

Run your program:

```bash
cargo run
```

## Next Steps

- Read [Installation](./installation.md) for detailed setup options
- Follow the [First Steps](./first-steps.md) tutorial for hands-on examples
- Learn [Core Concepts](./core-concepts.md) to understand RDF and SPARQL fundamentals
- Explore [Rust SDK](../sdk/rust/index.md) for the complete API reference
