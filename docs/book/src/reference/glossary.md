# Glossary

Terms and concepts used throughout rust-kgdb documentation.

## RDF & Semantic Web

### Triple
A fundamental RDF statement consisting of subject-predicate-object (S-P-O), representing a single fact. Example: "Alice knows Bob" is stored as:
- Subject: `<http://example.org/Alice>`
- Predicate: `<http://example.org/knows>`
- Object: `<http://example.org/Bob>`

### Quad
A triple with an additional graph context component, forming subject-predicate-object-graph (S-P-O-G). Used to organize triples into named graphs.

### RDF Graph
A collection of triples forming a directed labeled graph where nodes are IRIs, blank nodes, or literals, and edges are predicates.

### IRI (Internationalized Resource Identifier)
A globally unique identifier for resources in RDF, similar to but broader than URLs. Example: `<http://example.org/Alice>`

### Literal
A data value in RDF with optional datatype and language tag. Examples:
- `"Alice"` (plain string)
- `"42"^^xsd:integer` (integer)
- `"Bonjour"@fr` (French string)

### Blank Node
An anonymous node without a global identifier, used for representing unnamed entities. Example: `_:b1`

### Named Graph
A set of RDF triples identified by an IRI, allowing grouping and context. Example: `<http://example.org/graph1>`

### SPARQL
**Semantic Protocol and RDF Query Language** - Standard query language for RDF data. Provides SELECT, CONSTRUCT, ASK, and UPDATE operations.

### Property Path
SPARQL expression for traversing relationships in an RDF graph with operators:
- Single step: `ex:knows`
- Alternative: `ex:knows|ex:likes`
- Transitive: `ex:knows+` (one or more)
- Kleene star: `ex:knows*` (zero or more)

## Data Structures & Storage

### Triple Store
A database optimized for storing and querying RDF triples. rust-kgdb is a triple store with 24 bytes per triple.

### Index
Data structure enabling fast lookups. rust-kgdb uses SPOC indexing with 4 permutations (SPOC, POCS, OCSP, CSPO).

### SPOC Index
Primary indexing strategy using four quad indexes:
- **SPOC**: (Subject, Predicate, Object, Context)
- **POCS**: (Predicate, Object, Context, Subject)
- **OCSP**: (Object, Context, Subject, Predicate)
- **CSPO**: (Context, Subject, Predicate, Object)

### Dictionary
String interning mechanism that stores each unique string once and references it by ID. rust-kgdb's Dictionary reduces memory by 60%.

### Storage Backend
Pluggable storage engine:
- **InMemory**: HashMap-based, fast, volatile
- **RocksDB**: LSM-tree, persistent, ACID
- **LMDB**: B+-tree, memory-mapped, read-optimized

## Query & Optimization

### Basic Graph Pattern (BGP)
A set of triple patterns that must all match for a solution. Core pattern matching mechanism in SPARQL.

### Graph Pattern Matching
Process of finding all triples in the store that match a query's triple patterns.

### Query Optimization
Reordering and restructuring query patterns to minimize computation. rust-kgdb uses cost-based optimization.

### Cardinality Estimation
Predicting how many results a pattern will produce, used to optimize join order.

### Join
Combining results from multiple patterns, like SQL joins. Binary operations combining two pattern results.

### Filter
SPARQL constraint that keeps only results matching a condition. Applied after pattern matching.

### Aggregate Function
Function combining multiple values into one (COUNT, SUM, AVG, MIN, MAX, GROUP_CONCAT).

### CONSTRUCT
SPARQL query form that returns new RDF triples built from query results (vs SELECT which returns variable bindings).

## Reasoning & Semantics

### Reasoning
Inferring new facts from existing facts and rules. Examples:
- RDFS: Subclass transitivity
- OWL: Inverse properties, symmetric properties

### RDFS (RDF Schema)
Basic reasoning system providing class and property hierarchies.

### OWL (Web Ontology Language)
More expressive reasoning system supporting complex class and property relationships.

### OWL 2 RL (Rule Language)
Tractable subset of OWL suitable for rule-based inference.

### Ontology
Formal specification of entities, properties, and relationships in a domain.

### Inference Rule
Logical rule deriving new facts. Example: "If X is subclass of Y and Z is instance of X, then Z is instance of Y"

### Subsumption
When one class is more general than another (subclass relationship).

## Performance & Benchmarking

### Throughput
Operations completed per unit time. Example: "146K triples/sec" means 146,000 triples inserted per second.

### Latency
Time to complete a single operation. Example: "2.78 µs" per lookup means 2.78 microseconds per triple lookup.

### Benchmark
Standardized test measuring performance characteristics:
- **LUBM**: Lehigh University Benchmark (university/student data)
- **SP2Bench**: Semantic Publishing benchmark (RDF publishing data)

### Criterion
Statistical benchmarking framework detecting regressions with confidence intervals.

### Baseline
Reference performance measurement for comparison. rust-kgdb compares against previous versions.

### Regression
Performance degradation (becoming slower) due to code changes.

## Testing & Quality

### Unit Test
Test of a single component in isolation.

### Integration Test
Test of multiple components working together.

### W3C Conformance
Compliance with official W3C standards for SPARQL, RDF, Turtle, etc.

### Test Suite
Collection of related tests, like the official W3C test suite.

### Smoke Test
Quick test ensuring basic functionality works.

### Regression Test
Test preventing previous bugs from reoccurring.

## Development & Deployment

### Feature Flag
Compile-time option enabling/disabling code. Example: `features = ["rocksdb-backend"]`

### WASM (WebAssembly)
Bytecode format running code in browsers and runtimes at near-native speed.

### FFI (Foreign Function Interface)
Mechanism for calling code written in other languages (like Rust from Python/Kotlin).

### Cargo
Rust package manager and build system.

### Clippy
Rust linter providing code quality suggestions.

### ACID
Database properties:
- **Atomicity**: All or nothing
- **Consistency**: Valid state before/after
- **Isolation**: No interference between transactions
- **Durability**: Persisted after commit

## Performance Terminology

### Big-O Notation
Asymptotic complexity:
- **O(1)**: Constant time
- **O(log n)**: Logarithmic
- **O(n)**: Linear
- **O(n²)**: Quadratic

### Amortized Cost
Average cost per operation over multiple operations.

### Cache Locality
How often accessed data is in CPU cache, affecting performance.

### Zero-Copy
Design minimizing data copying by using references instead.

## Standards & Specifications

### W3C (World Wide Web Consortium)
Standards organization specifying RDF, SPARQL, OWL, etc.

### RDF 1.1
Modern RDF specification including:
- RDF Concepts and Abstract Syntax
- Turtle (TTL)
- N-Triples (NT)
- N-Quads (NQ)
- RDF/XML

### SPARQL 1.1
Latest SPARQL standard with Query and Update specifications.

### Turtle
Human-friendly RDF serialization format using prefixes and compact syntax.

### N-Triples
Simple line-based RDF format with one triple per line.

## Abbreviations

| Term | Meaning |
|------|---------|
| BGP | Basic Graph Pattern |
| HTTP | HyperText Transfer Protocol |
| IRI | Internationalized Resource Identifier |
| JSON | JavaScript Object Notation |
| LSM | Log-Structured Merge-tree |
| LMDB | Lightning Memory-Mapped Database |
| RDF | Resource Description Framework |
| RDFS | RDF Schema |
| OWL | Web Ontology Language |
| SPARQL | Semantic Protocol and RDF Query Language |
| TTL | Turtle |
| W3C | World Wide Web Consortium |
| WASM | WebAssembly |
| XSD | XML Schema Definition |
| URI | Uniform Resource Identifier |
| URL | Uniform Resource Locator |

## See Also

- [Core Concepts](../getting-started/core-concepts.md) - Detailed explanations
- [API Reference](./api.md) - Function and type documentation
- [SPARQL Reference](../sdk/rust/api.md#sparql-functions) - SPARQL builtin functions
