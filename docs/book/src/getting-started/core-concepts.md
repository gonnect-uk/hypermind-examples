# Core Concepts

Understanding RDF, SPARQL, and the knowledge graph model is essential for effective use of rust-kgdb.

## RDF Basics

**RDF (Resource Description Framework)** is a standard data model for knowledge representation. It represents information as triples:

### Triples: Subject-Predicate-Object

Every RDF statement is a triple:
```
<subject> <predicate> <object>
```

**Example:**
```
<http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob>
```

This states: "Alice knows Bob"

### Node Types

RDF nodes can be:

1. **IRI (Internationalized Resource Identifier)**
   - Globally unique identifier
   - Example: `<http://example.org/Alice>`

2. **Literal**
   - Strings, numbers, dates, etc.
   - Format: `"value"` or `"value"^^<datatype>`
   - Example: `"Alice"` or `"42"^^<http://www.w3.org/2001/XMLSchema#integer>`

3. **Blank Node**
   - Anonymous node with local identifier
   - Example: `_:b1`

## SPARQL: Graph Querying

**SPARQL** is the standard query language for RDF. It uses a pattern-matching approach.

### Basic Query Structure

```sparql
SELECT ?variable WHERE {
    # Graph pattern (triple patterns)
    ?subject <predicate> ?object
}
```

**Example:**
```sparql
SELECT ?person WHERE {
    ?person <http://example.org/knows> <http://example.org/Alice>
}
```

This returns all people who know Alice.

### Variables

Variables start with `?`:
- `?person` - matches any node
- `?predicate` - matches any predicate (with property paths)

### Filters

Restrict results with conditions:

```sparql
SELECT ?name ?age WHERE {
    ?person <http://example.org/name> ?name ;
            <http://example.org/age> ?age .
    FILTER (?age > 30)
}
```

## The Dictionary: Memory Efficiency

rust-kgdb uses **string interning** for efficiency:

1. All strings (URIs, literals) are stored once
2. Nodes reference strings by ID (8 bytes)
3. Reduces memory by ~60% compared to naive approaches

**Memory Layout:**
- Triple: 24 bytes (3 × 8-byte node IDs)
- Compared to other systems: 32-50 bytes per triple

## Quad vs Triple

**Triples**: Subject, Predicate, Object (3 components)
**Quads**: Subject, Predicate, Object, Graph (4 components)

Quads allow organizing statements into named graphs:

```sparql
SELECT ?s ?p ?o FROM <http://example.org/graph1> WHERE {
    ?s ?p ?o
}
```

## Triple Stores and Indexes

rust-kgdb uses **SPOC indexing** - four permutations of triple components:

- **SPOC**: Subject-Predicate-Object-Context (standard order)
- **POCS**: Predicate-Object-Context-Subject (pattern: `?p ?o ?s`)
- **OCSP**: Object-Context-Subject-Predicate (pattern: `?o ?s ?p`)
- **CSPO**: Context-Subject-Predicate-Object (for named graphs)

This enables **sub-millisecond lookups** for any triple pattern.

## Storage Backends

Choose storage based on your needs:

### InMemory
- **Best for**: Development, testing, datasets <100MB
- **Speed**: Fastest (2.78 µs per lookup)
- **Persistence**: None

### RocksDB
- **Best for**: Persistent storage, large datasets
- **Speed**: ~10x slower than InMemory
- **Persistence**: Disk-based with ACID guarantees

### LMDB
- **Best for**: Read-heavy workloads
- **Speed**: Memory-mapped, very fast reads
- **Persistence**: File-based with memory mapping

## SPARQL Operations

### SELECT
Returns variables from matching patterns:
```sparql
SELECT ?name ?age WHERE { ... }
```

### CONSTRUCT
Builds new RDF data from query results:
```sparql
CONSTRUCT { ?s <http://example.org/likes> ?o }
WHERE { ?s <http://example.org/knows> ?o }
```

### ASK
Boolean query (returns true/false):
```sparql
ASK { ?person <http://example.org/knows> <http://example.org/Alice> }
```

### UPDATE
Modifies the triple store:
```sparql
INSERT DATA {
    <http://example.org/Charlie> <http://example.org/knows> <http://example.org/Diana>
}
```

## Reasoning

rust-kgdb supports semantic reasoning with RDFS and OWL 2 RL:

**RDFS Example:**
```sparql
# If X is a subclass of Y and Y is a subclass of Z,
# then X is a subclass of Z (transitivity)
```

**OWL 2 RL Example:**
```sparql
# Inverse functional properties
# If X knowsInverse Y and knowsInverse is inverse functional,
# then Y knows X
```

See [Core Concepts in API Guide](../sdk/rust/api.md) for implementation details.

## Next Steps

- [Quick Start](./quick-start.md) - Try these concepts in code
- [First Steps](./first-steps.md) - Hands-on tutorial
- [Rust SDK API](../sdk/rust/api.md) - Complete API reference
- [SPARQL Guide](../reference/api.md) - Detailed SPARQL documentation
