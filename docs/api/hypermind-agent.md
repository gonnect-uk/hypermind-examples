# HyperMindAgent API Reference

HyperMindAgent combines LLM planning with knowledge graph reasoning for grounded, explainable AI.

## Constructor

```javascript
const agent = new HyperMindAgent(options)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `options.name` | `string` | Agent name for identification |
| `options.kg` | `GraphDB` | Knowledge graph instance |
| `options.apiKey` | `string?` | Optional OpenAI/Anthropic API key |
| `options.model` | `string?` | LLM model (default: 'gpt-4o') |

**Example:**
```javascript
const { GraphDB, HyperMindAgent } = require('rust-kgdb')

const db = new GraphDB('http://example.org/')
db.loadTtl('...', null)

const agent = new HyperMindAgent({
  name: 'my-agent',
  kg: db,
  apiKey: process.env.OPENAI_API_KEY  // Optional
})
```

---

## Methods

### call(question)

Ask a natural language question. Returns answer with full reasoning trace.

```javascript
const result = await agent.call(question)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `question` | `string` | Natural language question |

**Returns:** `AgentResult`

```javascript
interface AgentResult {
  answer: string              // Natural language answer
  sparql: string              // Generated SPARQL query
  results: QueryResult[]      // Raw query results
  thinkingGraph: ThinkingGraph  // Reasoning trace
  proof: Proof                // Cryptographic proof
}
```

---

## ThinkingGraph Structure

The `thinkingGraph` contains the complete reasoning trace:

```javascript
interface ThinkingGraph {
  observations: Observation[]     // Ground truth facts from KG
  derivedFacts: DerivedFact[]     // Facts derived via OWL rules
  derivationChain: Step[]         // Proof steps
  rulesApplied: number            // Count of OWL rules used
}

interface Observation {
  type: 'observation'
  fact: string                    // e.g., "alice knows bob"
  source: 'knowledge_graph'
}

interface DerivedFact {
  fact: string                    // e.g., "bob knows alice"
  rule: string                    // e.g., "SymmetricProperty"
  usedObservations: number[]      // References to source observations
}

interface Step {
  step: number
  type: 'observation' | 'inference' | 'rule'
  fact: string
  rule?: string                   // OWL rule applied
  usedSteps?: number[]            // Previous steps used
}
```

---

## Proof Structure

Every answer includes a cryptographic proof:

```javascript
interface Proof {
  hash: string           // SHA-256 hash of derivation chain
  verified: boolean      // Proof verification status
  tripleCount: number    // Number of triples used
}
```

---

## Example with Full Output

```javascript
const result = await agent.call("Who does alice know?")

console.log(result)
// {
//   answer: "Alice knows Bob",
//   sparql: "SELECT ?person WHERE { ex:alice ex:knows ?person }",
//   results: [{ bindings: { person: "ex:bob" } }],
//   thinkingGraph: {
//     observations: [
//       { type: "observation", fact: "alice knows bob" }
//     ],
//     derivedFacts: [
//       { fact: "bob knows alice", rule: "SymmetricProperty" }
//     ],
//     derivationChain: [
//       { step: 1, type: "observation", fact: "alice knows bob" },
//       { step: 2, type: "rule", rule: "SymmetricProperty",
//         fact: "bob knows alice", usedSteps: [1] }
//     ],
//     rulesApplied: 1
//   },
//   proof: {
//     hash: "sha256:92be3c44...",
//     verified: true,
//     tripleCount: 1
//   }
// }
```

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

ex:knows a owl:SymmetricProperty .
ex:alice ex:knows ex:bob .
```

---

## Without API Key

HyperMindAgent works without an LLM API key using schema-based query generation:

```javascript
const agent = new HyperMindAgent({
  name: 'offline-agent',
  kg: db
  // No apiKey - uses schema-based reasoning only
})
```

Schema is extracted from loaded TTL data and used to generate valid SPARQL queries deterministically.

---

## See Also

- [Core Concepts](../concepts/README.md)
- [GraphDB API](graphdb.md)
- [ThinkingReasoner API](thinking-reasoner.md)
