# ThinkingReasoner API Reference

ThinkingReasoner applies OWL deductive reasoning to derive new facts from observations.

## Constructor

```javascript
const { ThinkingReasoner } = require('rust-kgdb')

const reasoner = new ThinkingReasoner(kg)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `kg` | `GraphDB` | Knowledge graph instance |

---

## Methods

### reason()

Apply OWL rules to derive new facts from the knowledge graph.

```javascript
const result = reasoner.reason()
```

**Returns:** `ReasoningResult`

```javascript
interface ReasoningResult {
  observations: number        // Count of ground truth facts
  derivedFacts: number        // Count of inferred facts
  totalFacts: number          // observations + derivedFacts
  rulesApplied: string[]      // OWL rules used
  derivationChain: Step[]     // Complete proof chain
  proofHash: string           // SHA-256 hash for verification
}
```

---

## Supported OWL Rules

ThinkingReasoner auto-detects and applies these OWL properties:

| OWL Property | Rule | Example |
|--------------|------|---------|
| `owl:SymmetricProperty` | A→B implies B→A | `knows`, `adjacentTo`, `workedWith` |
| `owl:TransitiveProperty` | A→B, B→C implies A→C | `mentored`, `priceInfluencedBy` |

---

## Derivation Chain Structure

Each step in the derivation chain:

```javascript
interface Step {
  step: number          // Sequential step number
  rule: string          // 'OBSERVATION' or OWL rule name
  conclusion: string    // The fact (e.g., "alice knows bob")
  premises: string[]    // References to prior steps used
}
```

**Example chain:**
```javascript
[
  { step: 1, rule: "OBSERVATION", conclusion: "alice knows bob", premises: [] },
  { step: 2, rule: "OBSERVATION", conclusion: "bob knows carol", premises: [] },
  { step: 3, rule: "owl:SymmetricProperty", conclusion: "bob knows alice", premises: ["obs_1"] },
  { step: 4, rule: "owl:SymmetricProperty", conclusion: "carol knows bob", premises: ["obs_2"] }
]
```

---

## Example

```javascript
const { GraphDB, ThinkingReasoner } = require('rust-kgdb')
const fs = require('fs')

// Load data with OWL properties inline
const db = new GraphDB('http://example.org/')
const ttl = `
  @prefix owl: <http://www.w3.org/2002/07/owl#> .
  @prefix ex: <http://example.org/> .

  ex:knows a owl:SymmetricProperty .
  ex:alice ex:knows ex:bob .
  ex:bob ex:knows ex:carol .
`
db.loadTtl(ttl, null)

// Run reasoning
const reasoner = new ThinkingReasoner(db)
const result = reasoner.reason()

console.log(`Observations: ${result.observations}`)     // 2
console.log(`Derived Facts: ${result.derivedFacts}`)    // 2
console.log(`Rules Applied: ${result.rulesApplied}`)    // ['SymmetricProperty']
console.log(`Proof Hash: ${result.proofHash}`)          // sha256:...
```

---

## Integration with HyperMindAgent

ThinkingReasoner runs automatically inside HyperMindAgent:

```javascript
const agent = new HyperMindAgent({ name: 'demo', kg: db })
const result = await agent.call("Who knows who?")

// Access reasoning stats from response
console.log(result.thinkingGraph.derivationChain)
console.log(result.reasoningStats)
```

The `thinkingGraph.derivationChain` contains the complete proof chain from ThinkingReasoner.

---

## See Also

- [HyperMindAgent API](hypermind-agent.md)
- [GraphDB API](graphdb.md)
- [OWL Properties](../concepts/owl-properties.md)
