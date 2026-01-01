# HyperMind Documentation

## Quick Start

```bash
npm install rust-kgdb
```

```javascript
const { GraphDB, HyperMindAgent, getVersion } = require('rust-kgdb')

// Create in-memory knowledge graph
const db = new GraphDB('http://example.org/')
db.loadTtl('@prefix ex: <http://example.org/> . ex:alice ex:knows ex:bob .', null)

// Create HyperMindAgent with native Rust runtime
const agent = new HyperMindAgent()
agent.loadTtl('@prefix ex: <http://example.org/> . ex:alice ex:knows ex:bob .')

// Train embeddings (configurable)
agent.trainEmbeddingsWithConfig(50, 6, 3)  // 50 walks, 6 length, 3 epochs

// Use ask() - Dynamic Proxy with full reasoning
const llmConfig = {
  provider: 'openai',
  apiKey: process.env.OPENAI_API_KEY,
  model: 'gpt-4o'
}
const result = agent.ask('Who does alice know?', llmConfig)
console.log(result.answer)      // "bob"
console.log(result.reasoning)   // "To find who alice knows..."
console.log(result.rhaiCode)    // Generated Rhai code
console.log(result.proofHash)   // SHA-256 hash
```

---

## Key Features

### ask() vs askAgentic()

Both methods provide **complete explainability, thinking, and proof generation**:

| Feature | ask() (Dynamic Proxy) | askAgentic() (Tool Calling) |
|---------|----------------------|----------------------------|
| Execution Mode | Rhai Code Generation | Tool Calling Loop |
| Reasoning | LLM generates code | Multi-turn dialogue |
| Proof Generation | ✓ SHA-256 hash | ✓ SHA-256 hash |
| Capabilities Used | ✓ Tracked | ✓ Tracked |
| Latency | Fast (~1-5s) | Slower (~5-15s) |
| Use Case | Simple queries | Complex analysis |

### ask() Output Fields

```javascript
const result = agent.ask(question, llmConfig)
// result.answer           - Natural language answer
// result.reasoning        - LLM's reasoning for the approach
// result.rhaiCode         - Generated Rhai code
// result.capabilitiesUsed - ["query", "count", ...]
// result.proofHash        - SHA-256 proof hash
// result.executionTimeUs  - Execution time in microseconds
```

### askAgentic() Output Fields

```javascript
const result = agent.askAgentic(question, llmConfig)
// result.answer           - Natural language answer
// result.reasoning        - "Completed in N turns"
// result.toolCalls        - JSON of tool calls made
// result.capabilitiesUsed - ["query", ...]
// result.proofHash        - SHA-256 proof hash
// result.executionTimeUs  - Execution time in microseconds
```

---

## Demo Results (100% Pass Rate - January 2026)

| Example | Tests | Status |
|---------|-------|--------|
| [Music Recommendation](../examples/music-recommendation-agent.js) | 14/14 | ✅ 100% |
| [Euroleague Basketball](../examples/euroleague-basketball-agent.js) | 18/18 | ✅ 100% |
| [Legal Case](../examples/legal-case-agent.js) | 21/21 | ✅ 100% |
| [Digital Twin](../examples/digital-twin-smart-building.js) | 12/12 | ✅ 100% |
| [Boston Real Estate](../examples/boston-realestate-agent.js) | 19/19 | ✅ 100% |

---

## Documentation Index

### API Reference
- [GraphDB](api/graphdb.md) - KGDB (Gonnect) API reference
- [HyperMindAgent](api/hypermind-agent.md) - AI agent framework
- [ThinkingReasoner](api/thinking-reasoner.md) - Deductive reasoning engine

### Concepts
- [Core Concepts](concepts/README.md) - Grounded reasoning, OWL properties, proofs
- [Grounded Reasoning](concepts/grounded-reasoning.md) - LLM + KG = no hallucination

### Examples
- [Euroleague Basketball](../EUROLEAGUE_ANALYTICS.md) - OWL reasoning, RDF2Vec
- [Boston Real Estate](../BOSTON_REALESTATE.md) - Property valuation
- [US Legal Case](../LEGAL_CASE.md) - Mentorship chains
- [Music Recommendation](../MUSIC_RECOMMENDATION.md) - Genre-based similarity
- [Digital Twin](../DIGITAL_TWIN.md) - IoT + Datalog reasoning

---

## Runtime Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    YOUR APPLICATION                          │
│  const agent = new HyperMindAgent()                          │
│  const result = agent.ask("Find high-risk customers")        │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Native NAPI-RS bindings
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    RUST RUNTIME (KGDB)                       │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  In-Memory  │  │  RDF2Vec    │  │   HyperFederate     │ │
│  │    KGDB     │  │  Embeddings │  │   SQL + SPARQL      │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│                                                             │
│  Performance: <50ns per call | 24 bytes/triple              │
└─────────────────────────────────────────────────────────────┘
```

---

## npm Package

```bash
npm install rust-kgdb
```

- [npm package](https://www.npmjs.com/package/rust-kgdb)
- [GitHub repository](https://github.com/gonnect-uk/rust-kgdb)

---

## Enterprise / K8s Deployment

For production deployments with Kubernetes orchestration and real database federation:

**Contact: gonnect.hypermind@gmail.com**
