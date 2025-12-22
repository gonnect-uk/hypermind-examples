# HyperMindAgent API Reference

HyperMindAgent combines LLM planning with knowledge graph reasoning for grounded, explainable AI.

**npm package:** [`rust-kgdb`](https://www.npmjs.com/package/rust-kgdb) (v0.8.16+)

---

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
| `options.answerFormat` | `string?` | Output format: 'text' (default), 'table', 'json' |

**Example:**
```javascript
const { GraphDB, HyperMindAgent } = require('rust-kgdb')

const db = new GraphDB('http://example.org/')
db.loadTtl('...', null)

const agent = new HyperMindAgent({
  name: 'my-agent',
  kg: db,
  apiKey: process.env.OPENAI_API_KEY,  // Optional
  answerFormat: 'text'                  // 'text' | 'table' | 'json'
})
```

---

## Answer Formats

HyperMindAgent supports three output formats. Entity names are automatically extracted from URIs (e.g., `lessort__mathias` → `Mathias Lessort`).

### TEXT Format (Default)

Natural language listing of entities:

```javascript
const agent = new HyperMindAgent({ name: 'demo', kg: db, answerFormat: 'text' })
const result = await agent.call("Who are the teammates of Lessort?")

console.log(result.answer)
// "Cedi Osman, Jerian Grant, Lorenzo Brown, Kendrick Nunn, Kostas Sloukas and 106 more"
```

### TABLE Format

Professional tabular output:

```javascript
const agent = new HyperMindAgent({ name: 'demo', kg: db, answerFormat: 'table' })
const result = await agent.call("Who are the teammates of Lessort?")

console.log(result.answer)
// ┌────────────────────────────────────────┐
// │ Results (111 total)                     │
// ├────────────────────────────────────────┤
// │  Cedi Osman                            │
// │  Jerian Grant                          │
// │  Lorenzo Brown                         │
// │  Kendrick Nunn                         │
// │  Kostas Sloukas                        │
// │  Marius Grigonis                       │
// │  Mathias Lessort                       │
// │  Juancho Hernangomez                   │
// │  Konstantinos Mitoglou                 │
// │  ... and 96 more                       │
// └────────────────────────────────────────┘
```

### JSON Format

Structured data for programmatic use:

```javascript
const agent = new HyperMindAgent({ name: 'demo', kg: db, answerFormat: 'json' })
const result = await agent.call("Who are the teammates of Lessort?")

console.log(result.answer)
// {
//   "count": 111,
//   "results": [
//     { "s": "Jerian Grant", "o": "Cedi Osman" },
//     { "s": "Lorenzo Brown", "o": "Cedi Osman" },
//     { "s": "Mathias Lessort", "o": "Cedi Osman" },
//     ...
//   ],
//   "reasoning": {
//     "observations": 0,
//     "derivedFacts": 0,
//     "rulesApplied": 0
//   }
// }
```

---

## With vs Without API Key

### With API Key

When an API key is provided, HyperMindAgent uses the LLM for intent classification and planning, but query generation remains deterministic (schema-based).

```javascript
const agent = new HyperMindAgent({
  name: 'euroleague',
  kg: db,
  apiKey: process.env.OPENAI_API_KEY,
  model: 'gpt-4o'
})

const result = await agent.call("Who are the teammates of Lessort?")
console.log(result.answer)
// "Cedi Osman, Jerian Grant, Lorenzo Brown, Kendrick Nunn, Kostas Sloukas and 106 more"
```

### Without API Key

Works identically - schema-based query generation produces the same results:

```javascript
const agent = new HyperMindAgent({
  name: 'euroleague',
  kg: db
  // No apiKey - uses schema-based reasoning only
})

const result = await agent.call("Who are the teammates of Lessort?")
console.log(result.answer)
// "Cedi Osman, Jerian Grant, Lorenzo Brown, Kendrick Nunn, Kostas Sloukas and 106 more"
```

**Key Point:** The LLM is optional. Query generation is deterministic from schema, ensuring reproducible results.

---

## Real Output Examples

### Euroleague Basketball

**Question:** "Who are the teammates of Lessort?"

```javascript
{
  "answer": "Cedi Osman, Jerian Grant, Lorenzo Brown, Kendrick Nunn, Kostas Sloukas and 106 more",
  "sparql": "SELECT ?s ?o WHERE { ?s <http://euroleague.net/ontology#teammateOf> ?o } LIMIT 100",
  "resultCount": 111,
  "thinkingGraph": {
    "observations": 0,
    "derivedFacts": 0,
    "derivationChain": 0
  }
}
```

**Raw Results (first 5):**
```json
[
  { "o": "http://euroleague.net/player/osman__cedi", "s": "http://euroleague.net/player/none" },
  { "o": "http://euroleague.net/player/osman__cedi", "s": "http://euroleague.net/player/grant__jerian" },
  { "o": "http://euroleague.net/player/osman__cedi", "s": "http://euroleague.net/player/brown__lorenzo" },
  { "o": "http://euroleague.net/player/osman__cedi", "s": "http://euroleague.net/player/nunn__kendrick" },
  { "s": "http://euroleague.net/player/sloukas__kostas", "o": "http://euroleague.net/player/osman__cedi" }
]
```

### Legal Case (Brown v. Board)

**Question:** "Who argued the Brown v. Board case?"

```javascript
{
  "answer": "BrownVBoard, OliverHill, GeorgeHayes, JamesNabrit, LouisRedding and 4 more",
  "sparql": "SELECT ?s ?o WHERE { ?s <http://law.gov/case#arguedBy> ?o } LIMIT 100"
}
```

**Raw Results (first 5):**
```json
[
  { "s": "http://law.gov/case#BrownVBoard", "o": "http://law.gov/case#OliverHill" },
  { "o": "http://law.gov/case#GeorgeHayes", "s": "http://law.gov/case#BrownVBoard" },
  { "o": "http://law.gov/case#JamesNabrit", "s": "http://law.gov/case#BrownVBoard" },
  { "o": "http://law.gov/case#LouisRedding", "s": "http://law.gov/case#BrownVBoard" },
  { "o": "http://law.gov/case#RobertCarter", "s": "http://law.gov/case#BrownVBoard" }
]
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
  answer: string              // Formatted answer (text/table/json)
  explanation: object         // Execution trace with SPARQL queries
  raw_results: object[]       // Raw SPARQL query results
  inferences: object[]        // Applied reasoning rules
  thinkingGraph: ThinkingGraph  // Reasoning trace
  derivedFacts: object[]      // Facts derived via OWL rules
  proofs: object[]            // Cryptographic proofs
  reasoningStats: object      // Reasoning statistics
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

## Explore the Examples

Run the examples to see HyperMindAgent in action:

```bash
# Clone the repository
git clone https://github.com/gonnect-uk/hypermind-examples.git
cd hypermind-examples

# Install dependencies
npm install

# Run examples
npm run euroleague   # Euroleague basketball analytics
npm run boston       # Boston real estate
npm run legal        # Brown v. Board of Education
```

**Live Examples:**
- [Euroleague Basketball](https://github.com/gonnect-uk/hypermind-examples/blob/main/examples/euroleague-basketball-agent.js)
- [Boston Real Estate](https://github.com/gonnect-uk/hypermind-examples/blob/main/examples/boston-realestate-agent.js)
- [Legal Case Analysis](https://github.com/gonnect-uk/hypermind-examples/blob/main/examples/legal-case-agent.js)

---

## See Also

- [Core Concepts](../concepts/README.md)
- [GraphDB API](graphdb.md)
- [ThinkingReasoner API](thinking-reasoner.md)
- [npm Package](https://www.npmjs.com/package/rust-kgdb)
