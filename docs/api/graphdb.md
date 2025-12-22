# GraphDB (KGDB) API Reference

KGDB (Gonnect) is a high-performance RDF triple store with SPARQL 1.1 support.

## Constructor

```javascript
const db = new GraphDB(baseUri)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `baseUri` | `string` | Base URI namespace for the graph |

**Example:**
```javascript
const { GraphDB } = require('rust-kgdb')
const db = new GraphDB('http://example.org/')
```

---

## Methods

### loadTtl(data, graphUri)

Load Turtle/TTL data into the knowledge graph.

```javascript
db.loadTtl(data, graphUri)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `data` | `string` | TTL format RDF data |
| `graphUri` | `string \| null` | Optional named graph URI |

**Example:**
```javascript
db.loadTtl(`
  @prefix foaf: <http://xmlns.com/foaf/0.1/> .
  @prefix ex: <http://example.org/> .

  ex:alice foaf:name "Alice" ;
           foaf:knows ex:bob .
`, null)
```

**OWL Properties (Auto-Detected):**
```javascript
db.loadTtl(`
  @prefix owl: <http://www.w3.org/2002/07/owl#> .
  @prefix ex: <http://example.org/> .

  ex:knows a owl:SymmetricProperty .
  ex:transfers a owl:TransitiveProperty .
`, null)
```

---

### querySelect(sparql)

Execute SPARQL SELECT query.

```javascript
const results = db.querySelect(sparql)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `sparql` | `string` | SPARQL SELECT query |

**Returns:** `QueryResult[]`

```javascript
interface QueryResult {
  bindings: { [variable: string]: string }
}
```

**Example:**
```javascript
const results = db.querySelect(`
  SELECT ?name WHERE {
    ?person <http://xmlns.com/foaf/0.1/name> ?name .
  }
`)

for (const row of results) {
  console.log(row.bindings.name)  // "Alice"
}
```

---

### queryConstruct(sparql)

Execute SPARQL CONSTRUCT query to create new triples.

```javascript
const triples = db.queryConstruct(sparql)
```

**Returns:** `TripleResult[]`

```javascript
interface TripleResult {
  subject: string
  predicate: string
  object: string
}
```

---

### getTripleCount()

Get total number of triples in the graph.

```javascript
const count = db.getTripleCount()
```

**Returns:** `number`

---

### getStats()

Get database statistics.

```javascript
const stats = db.getStats()
```

**Returns:** `DatabaseStats`

```javascript
interface DatabaseStats {
  tripleCount: number
  subjectCount: number
  predicateCount: number
  objectCount: number
}
```

---

## Performance

| Operation | Speed |
|-----------|-------|
| Triple lookup | 449ns |
| Memory per triple | 24 bytes |
| Bulk insert | 146K triples/sec |

---

## See Also

- [HyperMindAgent API](hypermind-agent.md)
- [ThinkingReasoner API](thinking-reasoner.md)
