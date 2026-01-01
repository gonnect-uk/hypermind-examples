# HyperMindAgent API Reference

HyperMindAgent combines LLM planning with knowledge graph reasoning for grounded, explainable AI.

**npm package:** [`rust-kgdb`](https://www.npmjs.com/package/rust-kgdb)

---

## Quick Start

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

## Constructor

```javascript
const agent = new HyperMindAgent(options)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `options.name` | `string?` | Agent name for identification |

**Example:**
```javascript
const { HyperMindAgent } = require('rust-kgdb')

const agent = new HyperMindAgent()
// or
const agent = new HyperMindAgent({ name: 'my-agent' })
```

---

## Key Methods

### loadTtl(ttlData)

Load Turtle data into the agent's knowledge graph.

```javascript
agent.loadTtl(`
  @prefix ex: <http://example.org/> .
  @prefix owl: <http://www.w3.org/2002/07/owl#> .

  ex:knows a owl:SymmetricProperty .
  ex:alice ex:knows ex:bob .
`)
```

### trainEmbeddingsWithConfig(walksPerEntity, walkLength, epochs)

Train RDF2Vec embeddings with configurable parameters.

```javascript
// Fast training (for demos)
agent.trainEmbeddingsWithConfig(50, 6, 3)

// Full training (for production)
agent.trainEmbeddingsWithConfig(200, 10, 5)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `walksPerEntity` | `number` | Random walks per entity (default: 50) |
| `walkLength` | `number` | Length of each walk (default: 6) |
| `epochs` | `number` | Training epochs (default: 3) |

---

## ask() - Dynamic Proxy

Generate code dynamically to answer questions. LLM generates Rhai code that executes against the knowledge graph.

```javascript
const llmConfig = {
  provider: 'openai',
  apiKey: process.env.OPENAI_API_KEY,
  model: 'gpt-4o'
}

const result = agent.ask('What are the most expensive properties in Boston?', llmConfig)
```

**Returns:** `AskResult`

```javascript
{
  answer: string,           // Natural language answer
  reasoning: string,        // LLM's reasoning for the approach
  rhaiCode: string,         // Generated Rhai code
  capabilitiesUsed: string[], // ["query", "count", ...]
  proofHash: string,        // SHA-256 proof hash
  executionTimeUs: number   // Execution time in microseconds
}
```

**Example Output:**
```javascript
{
  answer: 'Found 5 results: "property_BB002", "property_BH001", ...',
  reasoning: 'To find the most expensive properties, we query for properties and their assessed values, then sort by value descending...',
  rhaiCode: 'let sparql = "SELECT ?p ?v WHERE { ?p a :Property . ?p :assessedValue ?v } ORDER BY DESC(?v)";\nlet results = query(sparql);...',
  capabilitiesUsed: ['query', 'count', 'load_ttl', 'federate', 'graph_search'],
  proofHash: 'd027713343646b18...',
  executionTimeUs: 2760
}
```

---

## askAgentic() - Tool Calling

Multi-turn tool calling loop for complex analysis. The LLM iteratively calls tools to gather information.

```javascript
const result = agent.askAgentic(
  'Analyze property values in Back Bay and adjacent neighborhoods. What trends do you see?',
  llmConfig
)
```

**Returns:** `AgenticResult`

```javascript
{
  answer: string,           // Detailed natural language answer
  reasoning: string,        // "Completed in N turns"
  toolCalls: string,        // JSON of tool calls made
  capabilitiesUsed: string[], // ["query", ...]
  proofHash: string,        // SHA-256 proof hash
  executionTimeUs: number   // Execution time in microseconds
}
```

**Example Output:**
```javascript
{
  answer: 'The analysis of property values in Back Bay and adjacent neighborhoods reveals:\n\n### Back Bay:\n- Properties have higher assessed values ($2.8M to $8.5M)\n\n### South End:\n- Lower values ($875K to $2.4M)\n\n### Beacon Hill:\n- Strong values ($1.6M to $3.9M)...',
  reasoning: 'Completed in 4 turns',
  toolCalls: 'query({"sparql":"SELECT ?neighborhood ?property ?value WHERE {..."})',
  capabilitiesUsed: ['query'],
  proofHash: '8ca3e570c0b96b14...',
  executionTimeUs: 10699650
}
```

---

## ask() vs askAgentic() Comparison

| Feature | ask() (Dynamic Proxy) | askAgentic() (Tool Calling) |
|---------|----------------------|----------------------------|
| Execution Mode | Rhai Code Generation | Multi-turn dialogue |
| Reasoning | LLM generates code | Step-by-step tool calls |
| Proof Generation | SHA-256 hash | SHA-256 hash |
| Capabilities Tracked | Yes | Yes |
| Latency | Fast (~1-5s) | Slower (~5-15s) |
| Use Case | Simple queries | Complex analysis |

---

## Available Capabilities

Both `ask()` and `askAgentic()` have access to these capabilities:

| Capability | Description |
|------------|-------------|
| `query` | Execute SPARQL SELECT queries |
| `construct` | Execute SPARQL CONSTRUCT queries |
| `count` | Count query results |
| `is_empty` | Check if results are empty |
| `load_ttl` | Load Turtle data |
| `federate` | Execute federated queries |
| `graph_search` | Search across graphs |
| `apply_rules` | Apply OWL reasoning rules |
| `derive` | Derive new facts |
| `explain` | Explain reasoning |
| `extract_schema` | Extract ontology schema |
| `get_classes` | List OWL classes |
| `get_properties` | List properties |
| `store_memory` | Store to working memory |
| `recall_memory` | Recall from memory |
| `persist_memory` | Persist memory |
| `embed` | Get entity embedding |
| `similar` | Find similar entities |
| `similarity` | Calculate similarity score |
| `pagerank` | Run PageRank algorithm |
| `triangle_count` | Count triangles |
| `connected_components` | Find connected components |
| `shortest_paths` | Find shortest paths |
| `label_propagation` | Run label propagation |
| `show_definition` | Show capability definition |
| `list_all_capabilities` | List all capabilities |

---

## OWL Property Auto-Detection

HyperMindAgent automatically detects OWL properties in your TTL data:

| OWL Property | Reasoning Rule |
|--------------|----------------|
| `owl:SymmetricProperty` | If A→B then B→A |
| `owl:TransitiveProperty` | If A→B and B→C then A→C |
| `rdfs:subClassOf` | Members of subclass are members of superclass |

**No separate ontology loading required.** OWL properties are inline in TTL:

```turtle
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:adjacentTo a owl:SymmetricProperty .
ex:priceInfluencedBy a owl:TransitiveProperty .

ex:BackBay ex:adjacentTo ex:SouthEnd .
```

---

## Demo Results (100% Pass Rate)

| Example | Tests | Status |
|---------|-------|--------|
| [Music Recommendation](../../examples/music-recommendation-agent.js) | 15/15 | 100% |
| [Euroleague Basketball](../../examples/euroleague-basketball-agent.js) | 18/18 | 100% |
| [Legal Case](../../examples/legal-case-agent.js) | 21/21 | 100% |
| [Digital Twin](../../examples/digital-twin-smart-building.js) | 13/13 | 100% |
| [Boston Real Estate](../../examples/boston-realestate-agent.js) | 21/21 | 100% |

---

## Run the Examples

```bash
# Clone the repository
git clone https://github.com/gonnect-uk/hypermind-examples.git
cd hypermind-examples

# Install dependencies
npm install

# Run examples (requires OPENAI_API_KEY for ask/askAgentic)
export OPENAI_API_KEY=your-key
npm run euroleague   # Euroleague basketball analytics
npm run boston       # Boston real estate
npm run legal        # Brown v. Board of Education
npm run music        # Music recommendation
npm run digital-twin # Smart building IoT
```

---

## See Also

- [Core Concepts](../concepts/README.md)
- [GraphDB API](graphdb.md)
- [ThinkingReasoner API](thinking-reasoner.md)
- [npm Package](https://www.npmjs.com/package/rust-kgdb)
