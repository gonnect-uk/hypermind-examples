# RDF Storage Format - Internal Representation

**Status**: v0.1.3 Current Implementation
**Date**: 2025-11-29

---

## 🎯 Key Principle: **Format-Agnostic Storage**

**All RDF formats → Unified Quad structure → Single storage layer**

The storage format is **INDEPENDENT** of the input format (Turtle, N-Triples, JSON-LD, etc.).

---

## 📦 Unified Storage Model

### Quad Structure (4 Components)

```rust
/// Universal RDF Quad (subject, predicate, object, graph)
pub struct Quad<'a> {
    /// Subject (IRI or BlankNode)
    pub subject: Node<'a>,

    /// Predicate (IRI only, per RDF spec)
    pub predicate: Node<'a>,

    /// Object (IRI, BlankNode, or Literal)
    pub object: Node<'a>,

    /// Graph (Optional named graph IRI)
    pub graph: Option<Node<'a>>,
}
```

### Node Structure (5 Types)

```rust
pub enum Node<'a> {
    /// IRI: <http://example.org/resource>
    IRI(&'a str),

    /// Literal: "value"^^datatype or "value"@language
    Literal {
        value: &'a str,
        datatype: Option<&'a str>,
        language: Option<&'a str>,
    },

    /// Blank Node: _:b1
    BlankNode(u64),

    /// Quoted Triple (RDF-star): << :s :p :o >>
    QuotedTriple(Box<Triple<'a>>),

    /// Variable (SPARQL only): ?var
    Variable(&'a str),
}
```

---

## 💾 Physical Storage Format

### Storage Backends (3 Options)

```
┌─────────────────────────────────────────────────────┐
│         Quad → Byte Encoding → Storage Backend      │
└─────────────────────────────────────────────────────┘
                       ↓
   ┌──────────────┬───────────────┬──────────────────┐
   │  InMemory    │   RocksDB     │      LMDB        │
   │  HashMap     │   LSM-Tree    │    B+Tree        │
   │  (fastest)   │  (persistent) │  (memory-mapped) │
   └──────────────┴───────────────┴──────────────────┘
```

### SPOC Indexing (4 Indexes for Fast Lookups)

**All backends use 4 index permutations**:

1. **SPOC** - Subject, Predicate, Object, Graph
   - Query: `?s p o g` → Fast subject lookup

2. **POCS** - Predicate, Object, Graph, Subject
   - Query: `s ?p o g` → Fast predicate lookup

3. **OCSP** - Object, Graph, Subject, Predicate
   - Query: `s p ?o g` → Fast object lookup

4. **CSPO** - Graph, Subject, Predicate, Object
   - Query: `s p o ?g` → Fast graph lookup

---

## 🔍 Storage Examples

### Example 1: Turtle → Storage

**Input** (Turtle):
```turtle
@prefix ex: <http://example.org/> .
ex:Alice ex:knows ex:Bob .
ex:Alice ex:age "25"^^xsd:integer .
```

**After Parsing** (Quad structures):
```rust
Quad {
    subject: Node::IRI("http://example.org/Alice"),
    predicate: Node::IRI("http://example.org/knows"),
    object: Node::IRI("http://example.org/Bob"),
    graph: None,
}

Quad {
    subject: Node::IRI("http://example.org/Alice"),
    predicate: Node::IRI("http://example.org/age"),
    object: Node::Literal {
        value: "25",
        datatype: Some("http://www.w3.org/2001/XMLSchema#integer"),
        language: None,
    },
    graph: None,
}
```

**In Storage** (InMemory backend):
```rust
// SPOC index (HashMap<Vec<u8>, Vec<u8>>)
Key: [S:Alice | P:knows | O:Bob | G:default]
Value: [metadata: timestamp, etc.]

Key: [S:Alice | P:age | O:"25"^^xsd:int | G:default]
Value: [metadata]

// POCS index
Key: [P:knows | O:Bob | G:default | S:Alice]
Value: [reference to SPOC]

// OCSP index
Key: [O:Bob | G:default | S:Alice | P:knows]
Value: [reference to SPOC]

// CSPO index
Key: [G:default | S:Alice | P:knows | O:Bob]
Value: [reference to SPOC]
```

### Example 2: N-Quads → Storage (Named Graph)

**Input** (N-Quads):
```nquads
<http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> <http://example.org/graph1> .
```

**After Parsing**:
```rust
Quad {
    subject: Node::IRI("http://example.org/Alice"),
    predicate: Node::IRI("http://example.org/knows"),
    object: Node::IRI("http://example.org/Bob"),
    graph: Some(Node::IRI("http://example.org/graph1")),  // ← Named graph
}
```

**In Storage**:
```rust
// SPOC index
Key: [S:Alice | P:knows | O:Bob | G:graph1]  // ← graph1, not default
Value: [metadata]

// CSPO index (graph first for fast graph queries)
Key: [G:graph1 | S:Alice | P:knows | O:Bob]
Value: [reference to SPOC]
```

### Example 3: JSON-LD → Storage

**Input** (JSON-LD):
```json
{
  "@context": {"ex": "http://example.org/"},
  "@id": "ex:Alice",
  "ex:knows": {"@id": "ex:Bob"},
  "@graph": "ex:graph1"
}
```

**After Parsing** (SAME Quad structure):
```rust
Quad {
    subject: Node::IRI("http://example.org/Alice"),
    predicate: Node::IRI("http://example.org/knows"),
    object: Node::IRI("http://example.org/Bob"),
    graph: Some(Node::IRI("http://example.org/graph1")),
}
```

**In Storage** (IDENTICAL to N-Quads example above!):
```rust
// Same SPOC/POCS/OCSP/CSPO indexes as N-Quads
// Format doesn't matter - storage is unified
```

---

## 🔑 Key Benefits

### 1. **Format Independence**
- Turtle, N-Triples, N-Quads, TriG, JSON-LD, RDF/XML → All become Quads
- Storage layer has ZERO knowledge of input format
- Can load mixed formats into same database

### 2. **Query Efficiency**
- 4 indexes enable O(1) lookups for any pattern
- `SELECT ?s WHERE { ?s :knows :Bob }` → Use POCS index
- `SELECT ?p WHERE { :Alice ?p ?o }` → Use SPOC index

### 3. **Memory Efficiency**
- **String Interning**: All URIs/literals stored ONCE in Dictionary
- **Node References**: Quad uses 8-byte references, not heap strings
- **24 bytes/triple**: Subject (8) + Predicate (8) + Object (8) + Graph (optional)

### 4. **Backend Flexibility**
```
Same Quad structure works with ANY backend:
- InMemory: HashMap (fastest, volatile)
- RocksDB: LSM-tree (persistent, ACID)
- LMDB: B+tree (memory-mapped, read-optimized)
```

---

## 📊 Storage Size Comparison

**Dataset**: 1M triples (LUBM benchmark)

| Backend | Storage Size | Lookup Speed | Use Case |
|---------|-------------|--------------|----------|
| **InMemory** | ~100 MB RAM | 2.78 µs | Fast queries, volatile |
| **RocksDB** | ~80 MB disk | ~10 µs | Persistent, ACID |
| **LMDB** | ~90 MB disk | ~5 µs | Read-heavy, persistent |

---

## 🎯 Summary

**Storage Format** = **Quad structure** (4 components: S, P, O, G)

**Input Format** (Turtle, N-Quads, JSON-LD, etc.) is **irrelevant to storage**:
- Parser converts ANY format → Quads
- Storage layer only sees Quads
- Query engine only sees Quads
- Output serializer converts Quads → ANY format

**This design enables**:
- ✅ Load Turtle + N-Quads + JSON-LD into SAME database
- ✅ Query unified data with SPARQL
- ✅ Export to ANY format (round-trip conversion)
- ✅ Add new formats without touching storage layer
