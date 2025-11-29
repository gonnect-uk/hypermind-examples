# Rust SDK API Guide

Complete reference for the rust-kgdb Rust API.

## Core Types

### Node

Represents any RDF node (IRI, literal, blank node, variable).

```rust
pub enum Node<'a> {
    IRI(&'a str),
    Literal(&'a str, &'a str),      // value, datatype
    BlankNode(u64),
    QuotedTriple(Box<Triple<'a>>),
    Variable(&'a str),
}

// Creating nodes
let iri = Node::IRI("http://example.org/resource");
let literal = Node::Literal("42", "http://www.w3.org/2001/XMLSchema#integer");
let blank = Node::BlankNode(1);
let var = Node::Variable("x");
```

### Triple

Three-component RDF statement.

```rust
pub struct Triple<'a> {
    pub subject: Node<'a>,
    pub predicate: Node<'a>,
    pub object: Node<'a>,
}

// Creating a triple
let triple = Triple::new(
    Node::IRI("http://example.org/Alice"),
    Node::IRI("http://example.org/knows"),
    Node::IRI("http://example.org/Bob")
);
```

### Quad

Four-component RDF statement with graph context.

```rust
pub struct Quad<'a> {
    pub subject: Node<'a>,
    pub predicate: Node<'a>,
    pub object: Node<'a>,
    pub graph: Node<'a>,
}
```

### Dictionary

String interning for memory efficiency.

```rust
pub struct Dictionary {
    // Implementation detail
}

impl Dictionary {
    pub fn new() -> Self;
    pub fn intern(&self, s: &str) -> &str;
    pub fn lookup(&self, id: u64) -> Option<&str>;
}

// Usage
let dict = Dictionary::new();
let uri = dict.intern("http://example.org/Alice");
```

## Storage Backends

### StorageBackend Trait

```rust
pub trait StorageBackend {
    fn put(&self, quad: &Quad) -> Result<()>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn scan(&self, prefix: &[u8]) -> Result<ScanIterator>;
    fn delete(&self, key: &[u8]) -> Result<()>;
    fn commit(&self) -> Result<()>;
}
```

### InMemoryBackend

```rust
pub struct InMemoryBackend { }

impl InMemoryBackend {
    pub fn new() -> Self;
    // Implements StorageBackend
}

// Usage
let backend = InMemoryBackend::new();
let store = Arc::new(backend);
```

### RocksDBBackend (feature: `rocksdb-backend`)

```rust
pub struct RocksDBBackend { }

impl RocksDBBackend {
    pub fn new(path: impl AsRef<Path>) -> Result<Self>;
    pub fn with_options(path: impl AsRef<Path>, options: Options) -> Result<Self>;
}
```

### LMDBBackend (feature: `lmdb-backend`)

```rust
pub struct LMDBBackend { }

impl LMDBBackend {
    pub fn new(path: impl AsRef<Path>) -> Result<Self>;
    pub fn with_max_dbs(path: impl AsRef<Path>, max_dbs: u32) -> Result<Self>;
}
```

## SPARQL Execution

### Executor

Executes SPARQL queries and updates.

```rust
pub struct Executor {
    store: Arc<dyn StorageBackend>,
}

impl Executor {
    pub fn new(store: Arc<dyn StorageBackend>) -> Self;

    // Execute SELECT query
    pub fn execute_query(
        &self,
        query: &str,
        dict: &Dictionary
    ) -> Result<Vec<Binding>>;

    // Execute CONSTRUCT query
    pub fn execute_construct(
        &self,
        query: &str,
        dict: &Dictionary
    ) -> Result<Vec<Triple>>;

    // Execute ASK query (boolean)
    pub fn execute_ask(
        &self,
        query: &str,
        dict: &Dictionary
    ) -> Result<bool>;

    // Execute UPDATE operations
    pub fn execute_update(
        &self,
        update: &str,
        dict: &Dictionary
    ) -> Result<()>;

    // Register custom function
    pub fn register_function(
        &mut self,
        name: &str,
        func: Arc<dyn Fn(&[Node]) -> Result<Node>>
    ) -> Result<()>;
}
```

### Binding

Solution mapping for query results.

```rust
pub struct Binding {
    pub bindings: HashMap<String, Node>,
}

impl Binding {
    pub fn get(&self, var: &str) -> Option<&Node>;
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Node)>;
}
```

## SPARQL Functions

### String Functions

```rust
STR(x)                      // Convert to string
CONCAT(str1, str2, ...)     // Concatenate strings
SUBSTR(str, start)          // Get substring
STRLEN(str)                 // String length
REGEX(str, pattern)         // Pattern matching
REPLACE(str, pat, repl)     // Replace pattern
LCASE(str)                  // Lowercase
UCASE(str)                  // Uppercase
STRBEFORE(str1, str2)       // Text before match
STRAFTER(str1, str2)        // Text after match
TRIM(str)                   // Remove whitespace
```

### Numeric Functions

```rust
ABS(num)                    // Absolute value
ROUND(num)                  // Round to nearest
CEIL(num)                   // Round up
FLOOR(num)                  // Round down
RAND()                      // Random number (0-1)
```

### Date/Time Functions

```rust
NOW()                       // Current timestamp
YEAR(date), MONTH, DAY      // Extract components
HOURS, MINUTES, SECONDS     // Time components
TIMEZONE(date)              // Timezone
```

### Hash Functions

```rust
MD5(str)                    // MD5 hash
SHA1(str)                   // SHA1 hash
SHA256(str)                 // SHA256 hash
SHA384(str)                 // SHA384 hash
SHA512(str)                 // SHA512 hash
```

### Test Functions

```rust
isIRI(term)                 // Check if IRI
isBlank(term)               // Check if blank node
isLiteral(term)             // Check if literal
isNumeric(term)             // Check if numeric
BOUND(var)                  // Check if bound
EXISTS(pattern)             // Check if pattern matches
NOT EXISTS(pattern)         // Check if pattern doesn't match
```

### Aggregate Functions

```rust
COUNT(?var)                 // Count distinct
COUNT(*)                    // Count all
SUM(?var)                   // Sum values
AVG(?var)                   // Average
MIN(?var)                   // Minimum
MAX(?var)                   // Maximum
GROUP_CONCAT(?var)          // Concatenate strings
```

## Reasoning APIs

### RDFS Reasoner

```rust
pub struct RDFSReasoner { }

impl RDFSReasoner {
    pub fn new(store: &Arc<dyn StorageBackend>) -> Self;
    pub fn apply_rules(&self) -> Result<()>;
}
```

### OWL 2 RL Reasoner

```rust
pub struct OWL2RLReasoner { }

impl OWL2RLReasoner {
    pub fn new(store: &Arc<dyn StorageBackend>) -> Self;
    pub fn apply_rules(&self) -> Result<()>;
}
```

## RDF Parsing

### TurtleParser

```rust
pub struct TurtleParser { }

impl TurtleParser {
    pub fn new() -> Self;
    pub fn parse_file(&self, path: impl AsRef<Path>) -> Result<Vec<Triple>>;
    pub fn parse_string(&self, content: &str) -> Result<Vec<Triple>>;
}
```

### NTriplesParser

```rust
pub struct NTriplesParser { }

impl NTriplesParser {
    pub fn new() -> Self;
    pub fn parse_file(&self, path: impl AsRef<Path>) -> Result<Vec<Triple>>;
    pub fn parse_string(&self, content: &str) -> Result<Vec<Triple>>;
}
```

## Error Handling

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    StorageError(String),
    ParseError(String),
    QueryError(String),
    ReasoningError(String),
    IOError(std::io::Error),
}

impl std::error::Error for Error { }
impl std::fmt::Display for Error { }
```

## Next Steps

- [Code Examples](./examples.md) - See practical usage
- [Best Practices](./best-practices.md) - Design patterns
- [Performance Guide](./performance.md) - Optimization tips
