# HyperMind Core Concepts

## What is HyperMind?

```
HyperMind = LLM Planning + Knowledge Graph + Deductive Reasoning + Proofs
```

HyperMind solves the hallucination problem by grounding LLM responses in verifiable facts.

---

## 1. Grounded Reasoning

### The Problem
```
User: "Who argued Brown v. Board of Education?"
Vanilla LLM: "[confident but potentially wrong answer]"
```

### The Solution
```
User: "Who argued Brown v. Board of Education?"
HyperMind:
  1. Schema extracted from knowledge graph
  2. SPARQL query generated: SELECT ?attorney WHERE { ... }
  3. Query executed against real data
  4. OWL reasoning applied
  5. Answer returned with proof chain

Answer: "Thurgood Marshall (Lead Counsel), Robert L. Carter..."
Proof: sha256:92be3c44... (verified)
```

**Every fact traceable to source observations. No hallucinations.**

---

## 2. OWL Properties (Auto-Detected)

OWL properties are **inline in your TTL data** - no separate ontology files needed.

```turtle
# In your data file (e.g., boston-properties.ttl)
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

# OWL definitions inline
ex:adjacentTo a owl:SymmetricProperty .
ex:transfers a owl:TransitiveProperty .

# Data
ex:BackBay ex:adjacentTo ex:SouthEnd .
ex:alice ex:transfers ex:bob .
```

**Auto-detected rules:**

| OWL Property | Rule |
|--------------|------|
| `SymmetricProperty` | If A→B then B→A |
| `TransitiveProperty` | If A→B and B→C then A→C |
| `subClassOf` | Members inherit parent class |

---

## 3. Derivation Chains (Proofs)

Every conclusion has a derivation chain showing how it was reached:

```
DERIVATION CHAIN:
  Step 1: [OBSERVATION] alice transfers bob        <- From your data
  Step 2: [OBSERVATION] bob transfers carol        <- From your data
  Step 3: [RULE] TransitiveProperty applied
  Step 4: [DERIVED] alice transfers carol          <- Inference (usedSteps: [1,2,3])

PROOF:
  hash: sha256:92be3c44...
  verified: true
  tripleCount: 2
```

---

## 4. ThinkingReasoner

The ThinkingReasoner loads facts as observations and derives new facts:

```javascript
// From npm run legal output:

ThinkingReasoner:
  Agent: legal-research-analyst
  Observations: 10
  Derived Facts: 17
  Rules Applied: 2

EVIDENCE NODES:
  [OBS] ThurgoodMarshall workedWith RobertCarter
  [OBS] ThurgoodMarshall workedWith JackGreenberg
  [OBS] ThurgoodMarshall mentored JackGreenberg

DERIVED:
  [DERIVED] RobertCarter workedWith ThurgoodMarshall  (SymmetricProperty)
  [DERIVED] JackGreenberg workedWith ThurgoodMarshall (SymmetricProperty)
```

---

## 5. RDF2Vec Embeddings

Semantic similarity via random walk embeddings:

```javascript
// From npm run euroleague output:

RDF2Vec Embeddings:
  Entity Embeddings: 214
  Dimensions: 128
  Random Walks: 1380
  Training Time: ~1.94s

Use: Find similar entities (players, teams, events)
```

---

## 6. Schema-Aware Query Generation

HyperMindAgent extracts schema from your data automatically:

```
SCHEMA CONTEXT (for LLM):
  Classes: 6 (Case, Person, Attorney, Justice, Plaintiff, Organization)
  Predicates: 22 (arguedBy, decidedBy, plaintiff, workedWith, mentored)
  Namespace: http://law.gov/case#
```

The LLM uses this schema to generate **valid SPARQL** that matches your data structure.

---

## No Ontology Files Required

All examples use **inline OWL in TTL data files**:

```turtle
# Everything in one file - no separate ontology

@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex: <http://example.org/> .

# Classes
ex:Person a rdfs:Class .
ex:Attorney rdfs:subClassOf ex:Person .

# OWL Properties
ex:workedWith a owl:SymmetricProperty .
ex:mentored a owl:TransitiveProperty .

# Data
ex:marshall a ex:Attorney ;
            rdfs:label "Thurgood Marshall" ;
            ex:workedWith ex:carter .
```

**HyperMindAgent auto-detects schema at `loadTtl()` time.**

---

## Benchmark Results

| Metric | HyperMind | Vanilla GPT-4 |
|--------|-----------|---------------|
| Accuracy (LUBM) | 86.4% | 0% |
| Valid SPARQL | 100% | 12% |
| Latency | 1.2s | 3.8s |

---

## See Also

- [GraphDB API](../api/graphdb.md) - KGDB reference
- [HyperMindAgent API](../api/hypermind-agent.md) - Agent API
- [Examples](../../README.md) - Run the examples
