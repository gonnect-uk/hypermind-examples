# HyperMind Documentation

## Quick Start

```bash
npm install rust-kgdb
```

```javascript
const { GraphDB, HyperMindAgent } = require('rust-kgdb')

const db = new GraphDB('http://example.org/')
db.loadTtl('@prefix ex: <http://example.org/> . ex:alice ex:knows ex:bob .', null)

const agent = new HyperMindAgent({ name: 'demo', kg: db })
const result = await agent.call('Who does alice know?')
console.log(result.answer)
```

---

## Documentation Index

### Getting Started
- [Installation](getting-started/installation.md) - npm install, requirements
- [Quick Start](getting-started/quickstart.md) - 5 minutes to first agent
- [Your First Query](getting-started/your-first-query.md) - Step-by-step tutorial

### API Reference
- [GraphDB](api/graphdb.md) - KGDB (Gonnect) API reference
- [HyperMindAgent](api/hypermind-agent.md) - AI agent framework
- [ThinkingReasoner](api/thinking-reasoner.md) - Deductive reasoning engine
- [EmbeddingService](api/embedding-service.md) - RDF2Vec vector embeddings

### Concepts
- [Grounded Reasoning](concepts/grounded-reasoning.md) - LLM + KG = no hallucination
- [OWL Properties](concepts/owl-properties.md) - SymmetricProperty, TransitiveProperty
- [Derivation Chains](concepts/derivation-chains.md) - Proof generation
- [RDF2Vec Embeddings](concepts/rdf2vec-embeddings.md) - Vector similarity

### Architecture
- [Overview](architecture/overview.md) - High-level architecture
- [WASM Runtime](architecture/wasm-runtime.md) - In-memory execution
- [Enterprise](architecture/enterprise.md) - K8s deployment (contact for setup)

### Examples
- [Euroleague Basketball](../EUROLEAGUE_ANALYTICS.md) - OWL reasoning, RDF2Vec
- [Boston Real Estate](../BOSTON_REALESTATE.md) - Property valuation
- [US Legal Case](../LEGAL_CASE.md) - Mentorship chains
- [Federation Setup](../FEDERATION_SETUP.md) - Cross-database queries

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
