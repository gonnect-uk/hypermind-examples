# Rust SDK Code Examples

Practical examples demonstrating common rust-kgdb usage patterns.

## Example 1: Basic Setup and Simple Query

```rust
use rust_kgdb::storage::InMemoryBackend;
use rust_kgdb::rdf_model::{Node, Triple, Dictionary};
use rust_kgdb::sparql::Executor;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create shared dictionary
    let dict = Arc::new(Dictionary::new());

    // Create in-memory store
    let backend = InMemoryBackend::new();
    let store = Arc::new(backend);

    // Add sample triple
    let alice = dict.intern("http://example.org/Alice");
    let knows = dict.intern("http://example.org/knows");
    let bob = dict.intern("http://example.org/Bob");

    let triple = Triple::new(alice, knows, bob);
    store.put(&triple)?;

    // Query
    let executor = Executor::new(Arc::clone(&store));
    let query = "SELECT ?x WHERE { ?x <http://example.org/knows> ?y }";
    let results = executor.execute_query(query, &dict)?;

    for binding in results {
        if let Some(node) = binding.get("x") {
            println!("Found: {:?}", node);
        }
    }

    Ok(())
}
```

## Example 2: Loading RDF from File

```rust
use rust_kgdb::rdf_io::TurtleParser;
use rust_kgdb::storage::InMemoryBackend;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse Turtle file
    let parser = TurtleParser::new();
    let triples = parser.parse_file("data.ttl")?;

    // Store in backend
    let store = Arc::new(InMemoryBackend::new());
    for triple in triples {
        store.put(&triple)?;
    }

    println!("Loaded {} triples", store.count()?);

    Ok(())
}
```

**Sample `data.ttl` file:**
```turtle
@prefix ex: <http://example.org/> .

ex:Alice ex:knows ex:Bob ;
         ex:name "Alice" ;
         ex:age 30 .

ex:Bob ex:knows ex:Charlie ;
       ex:name "Bob" ;
       ex:age 28 .
```

## Example 3: Complex SPARQL Query with Filters

```rust
use rust_kgdb::storage::InMemoryBackend;
use rust_kgdb::sparql::Executor;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(InMemoryBackend::new());
    let executor = Executor::new(store);

    let query = r#"
        PREFIX ex: <http://example.org/>

        SELECT ?person ?name ?age WHERE {
            ?person ex:name ?name ;
                    ex:age ?age ;
                    ex:knows ?friend .
            ?friend ex:name "Bob" .
            FILTER (?age >= 25)
        }
        ORDER BY ?name
        LIMIT 10
    "#;

    let results = executor.execute_query(query, &dict)?;

    println!("Results ({} found):", results.len());
    for binding in results {
        let person = binding.get("person").unwrap();
        let name = binding.get("name").unwrap();
        let age = binding.get("age").unwrap();
        println!("  {} ({}): {}", person, name, age);
    }

    Ok(())
}
```

## Example 4: CONSTRUCT Query (Building New RDF)

```rust
use rust_kgdb::sparql::Executor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(InMemoryBackend::new());
    let executor = Executor::new(store);

    // Create derived relationships
    let query = r#"
        PREFIX ex: <http://example.org/>

        CONSTRUCT {
            ?person1 ex:friendship ?person2 .
        }
        WHERE {
            ?person1 ex:knows ?person2 .
            ?person2 ex:knows ?person1 .
        }
    "#;

    let derived_triples = executor.execute_construct(query, &dict)?;

    println!("Created {} new triples via CONSTRUCT", derived_triples.len());

    Ok(())
}
```

## Example 5: SPARQL UPDATE (INSERT/DELETE)

```rust
use rust_kgdb::sparql::Executor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(InMemoryBackend::new());
    let executor = Executor::new(store);

    // Add new data
    let insert = r#"
        PREFIX ex: <http://example.org/>

        INSERT DATA {
            ex:Diana ex:knows ex:Eve ;
                     ex:name "Diana" ;
                     ex:age 32 .
        }
    "#;

    executor.execute_update(insert, &dict)?;

    // Update data
    let update = r#"
        PREFIX ex: <http://example.org/>

        DELETE { ?person ex:age ?oldAge }
        INSERT { ?person ex:age ?newAge }
        WHERE {
            ?person ex:age ?oldAge .
            BIND (?oldAge + 1 AS ?newAge)
        }
    "#;

    executor.execute_update(update, &dict)?;

    Ok(())
}
```

## Example 6: Using RocksDB for Persistence

```rust
use rust_kgdb::storage::RocksDBBackend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Persistent storage
    let backend = RocksDBBackend::new("./my_database")?;
    let store = Arc::new(backend);

    // Add data
    // ... (same as InMemory)

    // Data persists across program runs
    Ok(())
}
```

## Example 7: Semantic Reasoning with RDFS

```rust
use rust_kgdb::reasoning::RDFSReasoner;
use rust_kgdb::storage::InMemoryBackend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(InMemoryBackend::new());

    // Add ontology assertions
    let query = r#"
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX ex: <http://example.org/>

        INSERT DATA {
            ex:Person rdfs:label "Person" .
            ex:Employee rdfs:subClassOf ex:Person .
            ex:Alice a ex:Employee .
        }
    "#;

    let executor = Executor::new(Arc::clone(&store));
    executor.execute_update(query, &dict)?;

    // Apply RDFS reasoning
    let reasoner = RDFSReasoner::new(&store);
    reasoner.apply_rules()?;

    // Now we can infer: ex:Alice a ex:Person
    // (because Employee is subclass of Person)

    Ok(())
}
```

## Example 8: Custom SPARQL Function

```rust
use rust_kgdb::sparql::Executor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(InMemoryBackend::new());
    let mut executor = Executor::new(store);

    // Register custom function: DOUBLE(num)
    executor.register_function(
        "DOUBLE",
        Arc::new(|args: &[Node]| {
            match &args[0] {
                Node::Literal(val, dtype) => {
                    if let Ok(num) = val.parse::<f64>() {
                        let doubled = (num * 2.0).to_string();
                        Ok(Node::Literal(doubled, dtype.clone()))
                    } else {
                        Err("Invalid number".into())
                    }
                },
                _ => Err("Expected literal".into()),
            }
        })
    )?;

    // Use it in query
    let query = r#"
        SELECT ?doubled WHERE {
            BIND (DOUBLE("21"^^xsd:integer) AS ?doubled)
        }
    "#;

    let results = executor.execute_query(query, &dict)?;
    // Result: ?doubled = 42

    Ok(())
}
```

## Example 9: Batch Operations

```rust
use rust_kgdb::storage::InMemoryBackend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(InMemoryBackend::new());
    let dict = Arc::new(Dictionary::new());

    // Efficient batch insert
    let triples = vec![
        // Create many triples...
    ];

    for triple in triples {
        store.put(&triple)?;
    }

    store.commit()?;  // Flush to storage

    Ok(())
}
```

## Example 10: Thread-Safe Concurrent Access

```rust
use rust_kgdb::storage::InMemoryBackend;
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dict = Arc::new(Dictionary::new());
    let store = Arc::new(InMemoryBackend::new());

    let mut handles = vec![];

    // Spawn multiple threads
    for i in 0..4 {
        let store = Arc::clone(&store);
        let dict = Arc::clone(&dict);

        let handle = thread::spawn(move || {
            let s = dict.intern(&format!("http://example.org/item{}", i));
            let p = dict.intern("http://example.org/prop");
            let o = dict.intern(&format!("value{}", i));

            store.put(&Triple::new(s, p, o))
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap()?;
    }

    println!("All threads completed");

    Ok(())
}
```

## Next Steps

- [Best Practices](./best-practices.md) - Design patterns
- [Performance Guide](./performance.md) - Optimization strategies
- [API Reference](./api.md) - Complete API documentation
