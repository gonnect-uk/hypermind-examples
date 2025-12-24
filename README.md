# HyperMind Examples

<p align="center">
  <img src="docs/assets/hypermind-architecture.jpeg" alt="HyperMind Architecture" width="800"/>
</p>

> **The Problem**: LLMs hallucinate. They generate confident, plausible-sounding answers with no connection to reality. In enterprise contexts—fraud detection, legal research, medical diagnosis—this isn't a quirk. It's a liability.

> **The Solution**: Ground every answer in verifiable facts. Trace every conclusion to its source. Make AI auditable.

<p align="center">
  <strong>🦀 100% Rust-Powered | ⚡ 2.78µs Lookups | 🔒 Cryptographic Proofs | 🌐 WASM + K8s</strong>
</p>

---

## What is HyperMind?

HyperMind is a **reasoning-first AI framework**—built entirely in Rust, compiled to WASM—that eliminates hallucinations by construction. Not by prompting. Not by fine-tuning. By fundamentally changing how AI generates answers.

```
┌───────────────────────────────────────────────────────────────────────────┐
│                           HyperMindAgent                                  │
│   Natural language → SQL with graph_search() CTE → Verified answers       │
├───────────────────────────────────────────────────────────────────────────┤
│                           Runtime Layer                                   │
│            WASM (browser/edge)  |  Kubernetes (enterprise)                │
├───────────────────────────────────────────────────────────────────────────┤
│                       Query & Reasoning Layer                             │
│    SPARQL 1.1  |  Datalog  |  OWL2  |  GraphFrame  |  Motif Detection     │
├───────────────────────────────────────────────────────────────────────────┤
│                               KGDB                                        │
│    Rust-native knowledge graph  |  2.78µs lookups  |  24 bytes/triple     │
└───────────────────────────────────────────────────────────────────────────┘
```

**5 minutes to your first AI agent with deductive reasoning:**

```bash
git clone https://github.com/gonnect-uk/hypermind-examples.git
cd hypermind-examples
npm install
npm start
```

No servers. No configuration. Runs entirely in-memory via WASM.

---

## The Four Layers

### Layer 1: KGDB — The Foundation

**What**: A Rust-native knowledge graph database compiled to WebAssembly. Zero-copy semantics. Sub-microsecond performance.

**Why**: Traditional graph databases are too slow for real-time AI reasoning. KGDB achieves **2.78µs lookup speed**—35-180x faster than RDFox—while using only **24 bytes per triple** (25% more efficient than competitors).

**How**: String interning via a concurrent dictionary. SPOC quad indexing for O(1) pattern matching. Worst-case optimal join (WCOJ) execution for complex queries.

```javascript
const { GraphDB } = require('rust-kgdb')

const db = new GraphDB('http://example.org/')
db.loadTtl(`
  @prefix ex: <http://example.org/> .
  ex:alice ex:knows ex:bob .
  ex:bob ex:knows ex:carol .
`, null)

// 2.78µs per lookup
const results = db.querySelect('SELECT ?person WHERE { ex:alice ex:knows ?person }')
```

---

### Layer 2: Query & Reasoning — The Brain

**What**: A complete symbolic reasoning stack—SPARQL 1.1, Datalog rules, OWL2 inference, GraphFrame analytics, and motif detection—unified in a single query interface.

**Why**: AI needs more than pattern matching. It needs **deductive reasoning**: the ability to derive new facts from existing ones using formal rules. This is what separates "finding a document" from "proving a conclusion."

**How**:

| Capability | What It Does | Example |
|------------|--------------|---------|
| **SPARQL 1.1** | W3C-standard graph queries | `SELECT ?x WHERE { ?x :knows :bob }` |
| **Datalog** | Recursive rule evaluation | `ancestor(X,Z) :- parent(X,Y), ancestor(Y,Z)` |
| **OWL2** | Semantic inference | `:workedWith` is `owl:SymmetricProperty` → auto-infer inverse |
| **GraphFrame** | Network analytics | PageRank, connected components, shortest paths |
| **Motif Detection** | Pattern discovery | Find fraud triangles: A→B→C→A |

```javascript
// OWL reasoning: symmetric property auto-inference
db.loadTtl(`
  @prefix owl: <http://www.w3.org/2002/07/owl#> .
  @prefix ex: <http://example.org/> .

  ex:workedWith a owl:SymmetricProperty .
  ex:marshall ex:workedWith ex:carter .
`)

// Query: "Who worked with Carter?"
// Result: marshall (direct) + carter worked with marshall (inferred)
```

---

### Layer 3: Runtime — The Deployment

**What**: Two deployment modes from the same codebase—WASM for browser/edge, Kubernetes for enterprise scale.

**Why**: AI reasoning shouldn't require infrastructure changes. Run the same logic on a mobile device or a 100-node cluster. Same code. Same results. Different scale.

**How**:

| Mode | Use Case | Latency | Scale |
|------|----------|---------|-------|
| **WASM** | Browser, mobile, edge devices | <10ms | Single user |
| **Kubernetes** | Enterprise, multi-tenant, federated | <50ms | 100K+ users |

```javascript
// Same API, different runtime
const agent = new HyperMindAgent({
  name: 'fraud-detector',
  kg: db,
  runtime: 'wasm'      // or 'k8s' for enterprise
})
```

---

### Layer 4: HyperMindAgent — The Orchestrator

**What**: The AI layer that transforms natural language questions into verified, traceable answers with cryptographic proofs.

**Why**: LLMs are good at language. They're terrible at facts. HyperMindAgent uses LLMs for what they're good at (understanding intent, generating queries) while grounding every answer in the knowledge graph. **No hallucinations by construction.**

**How**:

1. **Schema extraction** — Auto-detect classes, properties, domains from your data
2. **Query generation** — LLM generates SQL with `graph_search()` CTE (universal format)
3. **Execution** — Rust executes query via NAPI-RS bindings
4. **Reasoning** — Apply OWL/Datalog rules
5. **Proof** — Generate SHA-256 hash of derivation chain

```javascript
const { HyperMindAgent } = require('rust-kgdb')

const agent = new HyperMindAgent({ name: 'legal-analyst', kg: db })

const result = await agent.call('Who argued Brown v. Board of Education?')

console.log(result.answer)           // "Thurgood Marshall, Robert L. Carter..."
console.log(result.proof.hash)       // "sha256:92be3c44..." (verifiable)
console.log(result.explanation.sql_queries[0].sql)  // SQL with graph_search() CTE
```

**Generated SQL with graph_search() CTE:**
```sql
WITH kg AS (
  SELECT * FROM graph_search('
    PREFIX law: <http://law.gov/case#>
    SELECT ?attorney ?name WHERE {
      <http://law.gov/case#BrownVBoard> law:arguedBy ?attorney .
      ?attorney rdfs:label ?name
    }
  ')
)
SELECT * FROM kg
```

**The key insight**: The LLM never answers from memory. It generates SQL with `graph_search()` CTE. Rust executes the query against facts. The facts produce the answer. Every step is traceable.

---

## Answer Formats

HyperMindAgent returns formatted answers (not just "Found X results"):

```javascript
// TEXT format (default) - Natural language
const agent = new HyperMindAgent({ name: 'demo', kg: db })
const result = await agent.call("Who are the teammates of Lessort?")
console.log(result.answer)
// → "Cedi Osman, Jerian Grant, Lorenzo Brown, Kendrick Nunn, Kostas Sloukas and 106 more"

// TABLE format - Professional tabular output
const agent = new HyperMindAgent({ name: 'demo', kg: db, answerFormat: 'table' })
// → ┌────────────────────────────────────────┐
//   │ Results (111 total)                     │
//   ├────────────────────────────────────────┤
//   │  Cedi Osman                            │
//   │  Jerian Grant                          │
//   │  ...                                   │
//   └────────────────────────────────────────┘

// JSON format - Structured data
const agent = new HyperMindAgent({ name: 'demo', kg: db, answerFormat: 'json' })
// → { "count": 111, "results": [...], "reasoning": {...} }
```

**Works with or without API key.** See [HyperMindAgent API](docs/api/hypermind-agent.md) for details.

---

## Examples

| Example | Description | Command |
|---------|-------------|---------|
| **Self-Driving Car** | Explainable AI for autonomous vehicles | `npm run self-driving-car` |
| **Digital Twin** | Smart Building IoT with HVAC automation | `npm run digital-twin` |
| **Music Recommendation** | Semantic music discovery with artist influence | `npm run music` |
| **BRAIN** | Fraud + Underwriting + HyperFederate | `npm run brain` |
| **Euroleague** | Basketball KG + OWL + RDF2Vec | `npm run euroleague` |
| **Boston** | Real estate + property valuation | `npm run boston` |
| **Legal** | US case law + mentorship chains | `npm run legal` |
| **Fraud** | Circular payment detection | `npm run fraud` |
| **Federation** | KGDB + Snowflake + BigQuery | `npm run federation` |
| **GraphFrames** | PageRank, shortest paths | `npm run graphframes` |
| **Datalog** | Rule-based reasoning | `npm run datalog` |
| **Pregel** | Bulk parallel processing | `npm run pregel` |

**Detailed output:**
- [Self-Driving Car](SELF_DRIVING_CAR.md) - 3D demo, SPARQL + Datalog + Hypergraph
- [Digital Twin](DIGITAL_TWIN.md) - Smart Building IoT, HVAC automation, real sensor data
- [Music Recommendation](MUSIC_RECOMMENDATION.md) - Artist influence, semantic discovery
- [BRAIN Fraud & Underwriting](BRAIN_FRAUD_UNDERWRITING.md) - 5 scenarios, KGDB + Snowflake + BigQuery
- [Euroleague Analytics](EUROLEAGUE_ANALYTICS.md) - 18 assertions, 100% pass
- [Boston Real Estate](BOSTON_REALESTATE.md) - 19 assertions, 100% pass
- [US Legal Case](LEGAL_CASE.md) - 20 assertions, 100% pass
- [Federation Setup](FEDERATION_SETUP.md) - Cross-database guide

---

## Benchmarks

### Demo Pass Rates (verified with GPT-4o, December 2025)

| Demo | Pass Rate | Tests |
|------|-----------|-------|
| Music Recommendation | **100%** | 15/15 |
| Digital Twin | **100%** | 13/13 |

### SQL with graph_search() CTE Generation

| Metric | HyperMind (with schema) | Vanilla GPT-4 (no schema) |
|--------|-------------------------|---------------------------|
| Valid SQL with CTE | **100%** | 0% (markdown blocks) |
| Semantic Accuracy | **100%** | 0% |

**Key Points:**
- **100% Valid SQL**: HyperMind always produces executable SQL with `graph_search()` CTE
- **100% Semantic Accuracy**: All queries return correct results from knowledge graph
- Vanilla GPT-4 without schema context fails completely (returns markdown blocks)

**Example Output (from Digital Twin demo):**
```sql
WITH kg AS (
  SELECT * FROM graph_search('
    PREFIX iot: <http://smartbuilding.org/iot#>
    SELECT ?property ?value ?classification WHERE {
      ?serverRoom a iot:ServerRoom .
      ?serverRoom ?property ?value .
      OPTIONAL { ?serverRoom rdf:type ?classification }
    }
  ')
)
SELECT * FROM kg
```

Run yourself:
```bash
OPENAI_API_KEY=your-key npm run bench:hypermind
```

---

## Documentation

- [API Reference](docs/api/graphdb.md) - GraphDB, HyperMindAgent APIs
- [Core Concepts](docs/concepts/README.md) - How reasoning works
- [npm package](https://www.npmjs.com/package/rust-kgdb)

---

## Enterprise / K8s

For production Kubernetes deployments:

**Contact: gonnect.hypermind@gmail.com**

---

## Requirements

- Node.js 14+

---

## License

Apache 2.0
