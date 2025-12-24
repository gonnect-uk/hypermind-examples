# HyperMind Examples

**5 minutes to your first AI agent with deductive reasoning.**

```bash
git clone https://github.com/gonnect-uk/hypermind-examples.git
cd hypermind-examples
npm install
npm start
```

No servers. No configuration. Runs entirely in-memory via WASM.

---

## What You Get

```
HyperMind = LLM Planning + Knowledge Graph + Deductive Reasoning + Proofs
```

- Ask questions in plain English
- Get answers grounded in facts (not hallucinations)
- Every conclusion has a proof you can trace

---

## Quick Code Example

```javascript
const { GraphDB, HyperMindAgent } = require('rust-kgdb')

// 1. Load data with OWL ontology (auto-detected from TTL)
const db = new GraphDB('http://example.org/')
db.loadTtl(`
  @prefix owl: <http://www.w3.org/2002/07/owl#> .
  @prefix ex: <http://example.org/> .
  ex:transfers a owl:TransitiveProperty .
  ex:alice ex:transfers ex:bob .
  ex:bob ex:transfers ex:carol .
`, null)

// 2. Create agent
const agent = new HyperMindAgent({ name: 'demo', kg: db })

// 3. Ask a question
const result = await agent.call('Who can alice reach through transfers?')
console.log(result.answer)           // "alice can reach carol (via bob)"
console.log(result.proof.hash)       // "sha256:92be3c44..."
console.log(result.thinkingGraph.derivationChain)  // Proof steps
```

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

## How It Works

```
Your Question → HyperMindAgent → ThinkingReasoner → KGDB → Answer + Proof

Key insight: LLM generates SPARQL from schema, NOT from training data.
             Every fact is traceable. No hallucinations.
```

See [docs/concepts](docs/concepts/README.md) for detailed explanation of:
- Grounded reasoning
- OWL property auto-detection
- Derivation chains (proofs)
- RDF2Vec embeddings

---

## Benchmarks

### Demo Pass Rates (from local execution with GPT-4o)

| Demo | Pass Rate | Tests |
|------|-----------|-------|
| Music Recommendation | **100%** | 15/15 |
| Digital Twin | **100%** | 13/13 |

### LUBM Benchmark (SPARQL generation accuracy)

| Metric | HyperMind (with schema) | Vanilla GPT-4 (no schema) |
|--------|-------------------------|---------------------------|
| Valid SPARQL | **100%** | 0% (markdown blocks) |
| Semantic Accuracy | 71.4% | 0% |

**Key Points:**
- **100% Valid SPARQL**: HyperMind always produces executable queries (no markdown)
- **71.4% Semantic Accuracy**: 5/7 LUBM queries return correct results
- Vanilla GPT-4 without schema context fails completely (returns markdown blocks)

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
