# Quickstart Guide

**Get rust-kgdb running in 5 minutes.**

---

## 1. Installation (30 seconds)

```bash
# Clone repository
git clone https://github.com/gonnect-uk/rust-kgdb
cd rust-kgdb

# Build workspace (release mode)
cargo build --workspace --release
```

**Build time**: ~5-6 minutes with full optimizations (LTO, opt-level=3)

---

## 2. Your First Query (2 minutes)

Create `quickstart.rs`:

```rust
use rdf_model::{Dictionary, Node, Triple, Quad};
use storage::QuadStore;
use sparql::{Executor, parse_query};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create string dictionary (interning)
    let dict = Arc::new(Dictionary::new());

    // 2. Create in-memory quad store
    let mut store = QuadStore::new_in_memory(Arc::clone(&dict));

    // 3. Insert data
    let alice = Node::iri(dict.intern("http://example.org/Alice"));
    let knows = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/knows"));
    let bob = Node::iri(dict.intern("http://example.org/Bob"));

    store.insert(Quad::from_triple(Triple::new(alice, knows, bob)))?;

    // 4. Execute SPARQL query
    let query = r#"
        SELECT ?person WHERE {
            ?person <http://xmlns.com/foaf/0.1/knows> <http://example.org/Bob>
        }
    "#;

    let algebra = parse_query(query)?;
    let mut executor = Executor::new(&store);
    let results = executor.execute(&algebra)?;

    // 5. Print results
    for binding in results.bindings() {
        println!("Found: {:?}", binding);
    }

    Ok(())
}
```

**Run**:
```bash
rustc quickstart.rs -L target/release/deps -o quickstart
./quickstart
```

**Output**:
```
Found: {"person": IRI("http://example.org/Alice")}
```

---

## 3. Load TTL Data (1 minute)

```rust
use rdf_io::TurtleParser;

// Parse Turtle file
let ttl_data = r#"
    @prefix : <http://example.org/> .
    @prefix foaf: <http://xmlns.com/foaf/0.1/> .

    :Alice foaf:knows :Bob, :Charlie .
    :Bob foaf:knows :Charlie .
"#;

let parser = TurtleParser::new(Arc::clone(&dict));
let triples = parser.parse_str(ttl_data)?;

// Insert into store
for triple in triples {
    store.insert(Quad::from_triple(triple))?;
}
```

---

## 4. SPARQL Update (1 minute)

```rust
use sparql::{UpdateExecutor, parse_update};

let update_query = r#"
    PREFIX : <http://example.org/>
    PREFIX foaf: <http://xmlns.com/foaf/0.1/>

    INSERT DATA {
        :Alice foaf:name "Alice Smith" .
        :Bob foaf:name "Bob Jones" .
    }
"#;

let algebra = parse_update(update_query)?;
let mut updater = UpdateExecutor::new(&mut store, Arc::clone(&dict));
let count = updater.execute(&algebra)?;

println!("Inserted {} triples", count);
```

---

## Next Steps

📚 **[Architecture Overview](../architecture/OVERVIEW.md)** - Understand system design
⚡ **[Performance Benchmarks](../performance/BENCHMARKS.md)** - See real measurements
✅ **[W3C Compliance](../w3c-compliance/SPARQL_1.1.md)** - Complete feature list
📱 **[Mobile Builds](../../developer/mobile/IOS_BUILD.md)** - iOS/Android deployment

---

## Common Issues

**"cannot find crate `rdf_model`"**
```bash
# Add to Cargo.toml:
[dependencies]
rdf-model = { path = "./crates/rdf-model" }
storage = { path = "./crates/storage" }
sparql = { path = "./crates/sparql" }
rdf-io = { path = "./crates/rdf-io" }
```

**"hidden lifetime parameters deprecated"**
```rust
// Use explicit lifetimes
Node<'_> instead of Node
```

**Slow performance?**
```bash
# Always use --release for real workloads
cargo build --release
```

---

**Production Ready**: This is production-grade software with 521 passing tests and complete W3C compliance.
