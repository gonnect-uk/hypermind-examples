# HyperMind Examples

**5 minutes to your first AI agent with deductive reasoning.**

```bash
git clone https://github.com/gonnect-uk/hypermind-examples.git
cd hypermind-examples
npm install
npm start
```

That's it. No servers. No configuration. Runs entirely in-memory via WASM.

---

## What You Get

```
HyperMind = LLM Planning + Knowledge Graph + Deductive Reasoning
```

- Ask questions in plain English
- Get answers grounded in facts (not hallucinations)
- Every conclusion has a proof you can trace

---

## Run Examples

```bash
# Euroleague basketball play-by-play with OWL reasoning
npm run euroleague

# Fraud detection with reasoning
npm run fraud

# Cross-database federation (KGDB + Snowflake + BigQuery)
npm run federation
npm run federation:3way

# Graph analytics (PageRank, shortest paths)
npm run graphframes

# Datalog reasoning
npm run datalog

# All examples
npm start
```

All examples include sample data. No external databases needed.

See [EUROLEAGUE_ANALYTICS.md](EUROLEAGUE_ANALYTICS.md) for detailed output with 100% test pass rate.

---

## How It Works

```
Your Question                    "Find circular payment patterns"
      |
      v
+------------------+
|  HyperMindAgent  |  Reads schema, generates valid SPARQL
+------------------+
      |
      v
+------------------+
|  ThinkingReasoner|  Applies OWL rules, derives new facts
+------------------+
      |
      v
+------------------+
|  KGDB (Gonnect)  |  Executes query (449ns lookups)
+------------------+
      |
      v
Answer + Derivation Chain + Cryptographic Proofs
```

Every step is traceable. No black boxes.

---

## Quick Code Example

```javascript
const { GraphDB, HyperMindAgent } = require('rust-kgdb')

// 1. Load data with OWL ontology (auto-detected from TTL)
const db = new GraphDB('http://example.org/')  // KGDB (Gonnect) instance
db.loadTtl(`
  @prefix owl: <http://www.w3.org/2002/07/owl#> .
  @prefix ex: <http://example.org/> .

  ex:transfers a owl:TransitiveProperty .
  ex:alice ex:transfers ex:bob .
  ex:bob ex:transfers ex:carol .
`, null)

// 2. Create agent (no API key needed for basic reasoning)
const agent = new HyperMindAgent({ name: 'demo', kg: db })

// 3. Ask a question
const result = await agent.call('Who can alice reach through transfers?')

console.log(result.answer)
// "alice can reach carol (via transitivity through bob)"

console.log(result.thinkingGraph.derivationChain)
// Step 1: Observed alice->bob
// Step 2: Observed bob->carol
// Step 3: Derived alice->carol (TransitiveProperty rule)
```

---

## Examples Overview

| Example | What It Shows | Run Command |
|---------|---------------|-------------|
| `euroleague` | **Basketball KG with OWL reasoning + RDF2Vec** | `npm run euroleague` |
| `boston` | **Real estate analytics with property valuation** | `npm run boston` |
| `legal` | **Legal case analysis with mentorship chains** | `npm run legal` |
| `fraud-detection` | Circular payment detection | `npm run fraud` |
| `federation` | Query KGDB + external DBs | `npm run federation` |
| `graphframes` | PageRank, connected components | `npm run graphframes` |
| `datalog` | Rule-based reasoning | `npm run datalog` |
| `pregel` | Bulk parallel graph processing | `npm run pregel` |
| `embeddings` | Vector similarity search | `npm run embeddings` |
| `deductive` | ThinkingReasoner with proofs | `npm run deductive` |

**Featured Examples**:
- [Euroleague Basketball](EUROLEAGUE_ANALYTICS.md) - OWL reasoning, RDF2Vec, 17 assertions (100% pass)
- [Boston Real Estate](BOSTON_REALESTATE.md) - Property valuation, 19 assertions (100% pass)
- [US Legal Case](LEGAL_CASE.md) - Mentorship chains, 20 assertions (100% pass)
- [Federation Setup](FEDERATION_SETUP.md) - Cross-database queries with credentials

**Featured Example**: The [Euroleague Basketball](EUROLEAGUE_ANALYTICS.md) example demonstrates:
- RDF2Vec embeddings (138 entities, 128D)
- ThinkingReasoner with OWL rules (111 observations → 222 derived facts)
- Prompt optimization with schema extraction
- 100% test pass rate with assertions

---

## Architecture

```
+-------------------------------------------------------+
|                   IN-MEMORY (WASM)                    |
|  +-------------------+  +-------------------------+   |
|  |  KGDB (Gonnect)   |  |   RpcFederationProxy    |   |
|  |  - SPARQL 1.1     |  |   mode: 'inMemory'      |   |
|  |  - 24 bytes/triple|  |   - KGDB queries        |   |
|  |  - 449ns lookups  |  |   - Mock external DBs   |   |
|  +-------------------+  +-------------------------+   |
|           |                        |                  |
|  +--------v------------------------v--------+         |
|  |           ThinkingReasoner               |         |
|  |  - EventStore (observations)             |         |
|  |  - FactStore (derived facts)             |         |
|  |  - DeductiveEngine (rule application)    |         |
|  |  - ProofWriter (SHA-256 audit trail)     |         |
|  +------------------------------------------+         |
+-------------------------------------------------------+
             |
             v
      Runs in Node.js via NAPI-RS
      No external servers needed
```

---

## Key Concepts

### Grounded Reasoning

Every conclusion traces back to data:

```
Observation: alice transfers bob     <- From your data
Observation: bob transfers carol     <- From your data
Rule: TransitiveProperty applied     <- From OWL ontology
Conclusion: alice transfers carol    <- Derived fact
Proof: sha256(...) = 92be3c44        <- Cryptographic audit
```

### OWL Properties → Automatic Rules

| You Define | System Creates |
|------------|----------------|
| `owl:TransitiveProperty` | If A→B and B→C then A→C |
| `owl:SymmetricProperty` | If A→B then B→A |
| `rdfs:subClassOf` | Members of subclass are members of superclass |

Load your ontology. Rules generate automatically.

### Derivation Chain

Like showing your work in math class:

```javascript
result.thinkingGraph.derivationChain
// [
//   { step: 1, type: 'observation', fact: 'transfers(alice,bob)' },
//   { step: 2, type: 'observation', fact: 'transfers(bob,carol)' },
//   { step: 3, rule: 'transitivity', fact: 'transfers(alice,carol)',
//     usedSteps: [1, 2] }
// ]
```

---

## Adding LLM (Optional)

For natural language query understanding:

```bash
OPENAI_API_KEY=your-key npm run fraud
# or
ANTHROPIC_API_KEY=your-key npm run fraud
```

Without an API key, examples still work using schema-based query generation.

---

## Benchmarks

Run the comparison yourself:

```bash
# HyperMind vs vanilla LLM
OPENAI_API_KEY=your-key npm run bench:hypermind

# Compare with LangChain, DSPy
OPENAI_API_KEY=your-key npm run bench:frameworks
```

Results on LUBM dataset (3,272 triples):

| | HyperMind | Vanilla GPT-4 |
|---|-----------|---------------|
| Accuracy | 86.4% | 0% |
| Valid SPARQL | 100% | 12% |
| Latency | 1.2s | 3.8s |

---

## Project Structure

```
hypermind-examples/
├── examples/
│   ├── euroleague-basketball-agent.js    # Basketball KG + RDF2Vec + ThinkingReasoner
│   ├── boston-realestate-agent.js        # Real estate with property valuation
│   ├── legal-case-agent.js               # Legal case analysis with mentorship
│   ├── hyperfederate-hypermind-demo.js   # Federation demo
│   ├── fraud-memory-hypergraph.js        # Fraud detection
│   ├── hypermind-deductive-demo.ts       # Deductive reasoning
│   ├── graphframes-example.ts            # Graph analytics
│   ├── datalog-example.ts                # Datalog rules
│   ├── pregel-example.ts                 # Pregel BSP
│   └── embeddings-example.ts             # Vector search
├── benchmarks/
│   ├── vanilla-vs-hypermind-benchmark.js
│   ├── framework-comparison-benchmark.js
│   └── benchmark-frameworks.py
├── data/
│   ├── euroleague-game.ttl               # Euroleague basketball play-by-play
│   ├── boston-properties.ttl             # Boston property assessments
│   ├── brown-v-board.ttl                 # Brown v. Board legal case
│   ├── fraud-graph.ttl                   # Sample fraud data
│   ├── insurance-claims.ttl              # Insurance claims
│   └── lubm-sample.ttl                   # LUBM benchmark data
├── EUROLEAGUE_ANALYTICS.md               # Detailed Euroleague output
├── BOSTON_REALESTATE.md                  # Boston example documentation
├── LEGAL_CASE.md                         # Legal case documentation
├── FEDERATION_SETUP.md                   # Federation credentials guide
├── package.json
└── README.md
```

---

## Enterprise / K8s Deployment

The examples here use the **in-memory WASM runtime** (free, open source).

For production deployments with:
- Kubernetes orchestration
- Distributed query execution
- Real Snowflake/BigQuery federation
- Enterprise support

Contact: **gonnect.hypermind@gmail.com**

---

## Requirements

- Node.js 14+
- That's it

---

## Links

- [npm package](https://www.npmjs.com/package/rust-kgdb)
- [Full documentation](https://github.com/gonnect-uk/rust-kgdb)

## License

Apache 2.0
