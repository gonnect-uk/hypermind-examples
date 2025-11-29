# Rust SDK Detailed Overview

Comprehensive guide to rust-kgdb's architecture, design principles, and core components.

## Architecture Overview

rust-kgdb follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────┐
│  Application Layer (Your Code)      │
├─────────────────────────────────────┤
│  SPARQL Query/Update Engine         │
│  - Parser (pest PEG)                │
│  - Optimizer (cost-based)           │
│  - Executor (zero-copy)             │
├─────────────────────────────────────┤
│  Reasoning Layer                    │
│  - RDFS Reasoner                    │
│  - OWL 2 RL Reasoner                │
│  - SHACL Validator                  │
├─────────────────────────────────────┤
│  Storage Layer                      │
│  - QuadStore (unified API)          │
│  - Triple Indexes (SPOC, POCS, etc) │
├─────────────────────────────────────┤
│  Storage Backends                   │
│  - InMemory | RocksDB | LMDB        │
├─────────────────────────────────────┤
│  RDF Model                          │
│  - Node, Triple, Quad               │
│  - Dictionary (string interning)    │
└─────────────────────────────────────┘
```

## Core Components

### 1. RDF Model (`rdf-model` crate)

Foundation types for RDF data:

```rust
pub enum Node<'a> {
    IRI(&'a str),                    // Resource identifier
    Literal(&'a str, &'a str),       // Value with datatype
    BlankNode(u64),                  // Anonymous node
    QuotedTriple(Box<Triple<'a>>),   // RDF 1.2 quoted triples
    Variable(&'a str),               // Query variable
}

pub struct Triple<'a> {
    pub subject: Node<'a>,
    pub predicate: Node<'a>,
    pub object: Node<'a>,
}

pub struct Quad<'a> {
    pub subject: Node<'a>,
    pub predicate: Node<'a>,
    pub object: Node<'a>,
    pub graph: Node<'a>,
}
```

**Dictionary**: Interns all strings once, reducing memory by 60%:

```rust
let dict = Dictionary::new();
let uri1 = dict.intern("http://example.org/Alice");
let uri2 = dict.intern("http://example.org/Alice"); // Same ID
```

### 2. Storage Layer (`storage` crate)

Three pluggable backends with unified API:

```rust
pub trait StorageBackend {
    fn put(&self, quad: &Quad) -> Result<()>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn scan(&self, prefix: &[u8]) -> Result<Iterator>;
    fn delete(&self, key: &[u8]) -> Result<()>;
}
```

**Backend Comparison**:

| Feature | InMemory | RocksDB | LMDB |
|---------|----------|---------|------|
| Speed | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| Persistence | None | Disk | Memory-mapped |
| ACID | No | Yes | Yes |
| Best For | Dev/Test | General | Read-heavy |

### 3. SPARQL Engine (`sparql` crate)

Complete SPARQL 1.1 implementation:

```rust
pub struct Executor {
    store: Arc<dyn StorageBackend>,
}

impl Executor {
    pub fn execute_query(
        &self,
        query: &str,
        dict: &Dictionary
    ) -> Result<Vec<Binding>>;

    pub fn execute_update(
        &self,
        update: &str,
        dict: &Dictionary
    ) -> Result<()>;
}
```

**64 Builtin Functions** including:
- String functions: STR, CONCAT, SUBSTR, REGEX, etc.
- Numeric functions: ABS, ROUND, CEIL, FLOOR
- Date functions: NOW, YEAR, MONTH, etc.
- Hash functions: MD5, SHA1, SHA256
- Test functions: isIRI, isBlank, BOUND, EXISTS

### 4. Parser Layer (`rdf-io` crate)

Parses RDF data from multiple formats:

```rust
let parser = TurtleParser::new();
let triples = parser.parse_file("data.ttl")?;

let parser = NTriplesParser::new();
let triples = parser.parse_file("data.nt")?;
```

**Supported Formats**:
- Turtle (TTL)
- N-Triples (NT)
- RDF/XML (RDF)
- N-Quads (NQ)

### 5. Reasoning (`reasoning` crate)

Semantic inference engines:

```rust
let rdfs_reasoner = RDFSReasoner::new(&store);
rdfs_reasoner.apply_rules()?;  // Infer subclass relations

let owl_reasoner = OWL2RLReasoner::new(&store);
owl_reasoner.apply_rules()?;   // Infer from OWL axioms
```

## Memory Model: Zero-Copy

rust-kgdb uses Rust's borrow checker for memory safety:

```rust
// These share the same underlying string
let uri1: Node = dict.intern("http://example.org/resource");
let uri2: Node = dict.intern("http://example.org/resource");

// Both reference the same dictionary entry (8 bytes each)
// Total memory: 1 × string + 2 × 8-byte references
```

**Triple Memory Layout** (24 bytes):
```
┌─────────────┬─────────────┬─────────────┐
│ Subject ID  │ Predicate   │ Object ID   │
│ 8 bytes     │ 8 bytes     │ 8 bytes     │
└─────────────┴─────────────┴─────────────┘
```

## Indexing Strategy: SPOC

Four indexes enable efficient queries for any triple pattern:

```
SPOC: (Subject, Predicate, Object, Context) - standard
POCS: (Predicate, Object, Context, Subject) - for ?p ?o ?s
OCSP: (Object, Context, Subject, Predicate) - for ?o patterns
CSPO: (Context, Subject, Predicate, Object) - named graphs
```

**Query Pattern Routing**:
- `?x <p> <o>` → Use POCS index (seek by predicate/object)
- `<s> ?p ?o` → Use SPOC index (seek by subject)
- `?s ?p ?o` → Full scan (no constraint)

## Thread Safety

rust-kgdb is thread-safe by default:

```rust
let dict = Arc::new(Dictionary::new());
let store = Arc::new(InMemoryBackend::new());

// Safe to share across threads
std::thread::spawn({
    let dict = Arc::clone(&dict);
    let store = Arc::clone(&store);
    move || {
        // Use dict and store in parallel
    }
});
```

## Performance Characteristics

**Lookup Speed**: 2.78 µs (359K per second)
**Memory Efficiency**: 24 bytes per triple (vs 32-50 in other systems)
**Bulk Insert**: 146K triples per second (682ms for 100K)
**Query Optimization**: Cost-based planner with cardinality estimation

## Next Steps

- [API Guide](./api.md) - Complete function reference
- [Code Examples](./examples.md) - Real-world patterns
- [Performance Guide](./performance.md) - Optimization tips
- [First Steps Tutorial](../../getting-started/first-steps.md) - Hands-on learning
