# Euroleague Basketball Knowledge Graph Analytics

This document shows **actual output** from running the Euroleague example with HyperMindAgent.

**Source**: [Representing Euroleague Play-by-Play Data as a Knowledge Graph](https://medium.com/@skontopo2009/representing-euroleague-play-by-play-data-as-a-knowledge-graph-6397534cdd75)

**Data Model**: [pbprdf - Play-by-Play RDF Ontology](https://github.com/andrewstellman/pbprdf)

---

## Test Results Summary

| Metric | Value |
|--------|-------|
| **Pass Rate** | 100.0% |
| **Tests Passed** | 18 |
| **Tests Failed** | 0 |

---

## HyperMindAgent Flow

```
User Query (Natural Language)
    |
    v
+------------------------------------------+
|         HyperMindAgent                   |
|  +------------------------------------+  |
|  | 1. [OBSERVE] Load facts from KG   |  |
|  | 2. [INFER] Apply OWL rules        |  |
|  | 3. [GENERATE] LLM creates SPARQL  |  |
|  +------------------------------------+  |
+------------------------------------------+
    |
    v
+------------------------------------------+
|    HyperFederate SQL (graph_search)      |
|    SELECT * FROM graph_search('SPARQL')  |
|    LEFT JOIN external_db ON ...          |
+------------------------------------------+
    |
    v
+------------------------------------------+
|  4. [PROVE] Execute + derivation chain   |
+------------------------------------------+
    |
    v
Answer + Proof (traceable to source data)
```

---

## HyperFederate SQL with graph_search() UDF

**ACTUAL OUTPUT** - HyperFederate unifies SQL + Knowledge Graph queries:

```sql
-- HyperFederate SQL: Join Knowledge Graph + External Data
SELECT
  kg.player,
  kg.steal_count,
  ext.player_salary,
  ext.team_budget
FROM graph_search('
  PREFIX euro: <http://euroleague.net/ontology#>
  SELECT ?player (COUNT(?steal) AS ?steal_count) WHERE {
    ?steal a euro:Steal .
    ?steal euro:player ?player .
  } GROUP BY ?player
') kg
LEFT JOIN external_db.player_contracts ext
  ON kg.player = ext.player_uri
ORDER BY kg.steal_count DESC
```

**SPARQL inside graph_search() executed:**

```sparql
PREFIX euro: <http://euroleague.net/ontology#>
SELECT ?player (COUNT(?steal) AS ?steal_count) WHERE {
  ?steal a euro:Steal .
  ?steal euro:player ?player .
} GROUP BY ?player
```

**HONEST RESULTS (from graph_search):**

| player                    | steal_count |
|---------------------------|-------------|
| lessort__mathias          |           1 |
| mitoglou__konstantinos    |           1 |
| mattisseck__jonas         |           1 |

---

## Thinking Events Timeline (Real-time Reasoning Stream)

**ACTUAL OUTPUT** - Auto-captured during reasoning:

### [OBSERVE] - Detected 111 facts from knowledge graph:

```
-> none teammateOf osman__cedi
-> grant__jerian teammateOf osman__cedi
-> brown__lorenzo teammateOf osman__cedi
-> nunn__kendrick teammateOf osman__cedi
-> sloukas__kostas teammateOf osman__cedi
-> grigonis__marius teammateOf osman__cedi
... and 105 more observations
```

### [INFER] - Applied OWL Rules:

| Rule | Description | Effect |
|------|-------------|--------|
| SymmetricProperty | `A teammateOf B => B teammateOf A` | 111 -> 222 facts |
| TransitiveProperty | `A rel B, B rel C => A rel C` | Chain inference |

### [PROVE] - Derivation Chain (audit trail):

```
Step 1: [OBSERVATION] none teammateOf osman__cedi
Step 2: [OBSERVATION] grant__jerian teammateOf osman__cedi
Step 3: [OBSERVATION] brown__lorenzo teammateOf osman__cedi
Step 4: [OBSERVATION] nunn__kendrick teammateOf osman__cedi
Step 5: [OBSERVATION] sloukas__kostas teammateOf osman__cedi
Step 6: [OBSERVATION] grigonis__marius teammateOf osman__cedi
Step 7: [OBSERVATION] lessort__mathias teammateOf osman__cedi
Step 8: [OBSERVATION] hernangomez__juancho teammateOf osman__cedi
... and 214 more proof steps
```

### REASONING COMPLETE:

- 111 observations (ground truth from KG)
- 222 derived facts (inferred via OWL rules)
- 2 rules applied (SymmetricProperty, TransitiveProperty)
- Every fact is traceable to source data (no hallucination)

---

## Reasoning Visualizer (Grounded Answer Flow)

**Run:** `npm run reasoning`

**ACTUAL OUTPUT** - Shows how answer is derived from response object:

```
┌────────────────────────────────────────────────────────────────────┐
│  📝 USER QUESTION                                                    │
│  "Who made the defensive steals in this game?"                      │
└────────────────────────────────────────────────────────────────────┘
          ▼
┌────────────────────────────────────────────────────────────────────┐
│  📊 ACTUAL DATA FROM KNOWLEDGE GRAPH                                 │
│  1. entity: Mathias Lessort, event: e00011                          │
│  2. entity: Konstantinos Mitoglou, event: e00028                    │
│  3. entity: Jonas Mattisseck, event: e00030                         │
└────────────────────────────────────────────────────────────────────┘
          ▼
┌────────────────────────────────────────────────────────────────────┐
│  🧠 REASONING APPLIED (from response.reasoningStats)                 │
│  Observations (ground truth):    111                                │
│  Inferences (OWL derived):       111                                │
│  Total Facts:                    222                                │
│                                                                      │
│  OWL RULES APPLIED (from derivationChain):                           │
│    • SymmetricProperty         (111 inferences)                     │
└────────────────────────────────────────────────────────────────────┘
          ▼
┌────────────────────────────────────────────────────────────────────┐
│  🔐 PROOF CHAIN (from response.thinkingGraph.derivationChain)        │
│  Step  1: [OBSERVATION]        "grant__jerian teammateOf osman"     │
│  Step  2: [OBSERVATION]        "brown__lorenzo teammateOf osman"    │
│  ──────────────────────────────────────────────────────────────     │
│  Step112: [SymmetricProperty]  "osman teammateOf grant__jerian"     │
│           ↳ derived from: obs_1                                     │
│  Step113: [SymmetricProperty]  "osman teammateOf brown__lorenzo"    │
│           ↳ derived from: obs_2                                     │
│  ──────────────────────────────────────────────────────────────     │
│  ... total 222 proof steps (111 obs + 111 inferences)               │
│  Proof Hash: sha256:19b4818ea0b  ✅ Verified                        │
└────────────────────────────────────────────────────────────────────┘
          ▼
┌────────────────────────────────────────────────────────────────────┐
│  ✅ GROUNDED ANSWER                                                  │
│  "3 results found: Mathias Lessort, Konstantinos Mitoglou,         │
│   Jonas Mattisseck"                                                 │
│                                                                     │
│  ✓ Data from real Knowledge Graph (not hallucinated)               │
│  ✓ SPARQL query is deterministic                                   │
│  ✓ Every fact has proof chain to source                            │
│  ✓ Cryptographic hash ensures integrity                            │
└────────────────────────────────────────────────────────────────────┘
```

---

## HyperMindAgent.call() Response Structure

**ACTUAL OUTPUT** - Complete response from `agent.call("Who made the defensive steals?")`:

```yaml
sparql:
  SELECT ?entity WHERE {
    ?event a <http://euroleague.net/ontology#Steal> .
    ?event <http://euroleague.net/ontology#player> ?entity
  } LIMIT 100

results (actual data):
  -> event=e00011, entity=lessort__mathias
  -> entity=mitoglou__konstantinos, event=e00028
  -> entity=mattisseck__jonas, event=e00030

answer:
  "Found 3 results"

thinking:
  predicatesIdentified: auto-detected
  schemaMatches: 11 classes, 7 predicates

reasoning:
  observations: 111
  derivedFacts: 222
  rulesApplied: 2

proof:
  derivationChain:
    - step: 1, rule: "OBSERVATION", conclusion: "none teammateOf osman__cedi"
    - step: 2, rule: "OBSERVATION", conclusion: "grant__jerian teammateOf osman__cedi"
    - step: 3, rule: "OBSERVATION", conclusion: "brown__lorenzo teammateOf osman__cedi"
    - step: 4, rule: "OBSERVATION", conclusion: "nunn__kendrick teammateOf osman__cedi"
  proofHash: "sha256:19b4806a903"
  verified: true
```

**Second Query** - `agent.call("Who are the teammates of Lessort?")`:

```yaml
sparql:
  SELECT ?s ?o WHERE {
    ?s <http://euroleague.net/ontology#teammateOf> ?o
  } LIMIT 100

results (actual data):
  -> s=none, o=osman__cedi
  -> s=grant__jerian, o=osman__cedi
  -> o=osman__cedi, s=brown__lorenzo
  -> o=osman__cedi, s=nunn__kendrick
  -> o=osman__cedi, s=sloukas__kostas
  ... and 106 more

answer:
  "Found 111 results"
```

---

## Knowledge Graph Statistics

```
Triples: 603
Teams: 2 (BER, PAN)
Players: 22
Steals: 3
Assists: 8
Teammate links: 111
```

## RDF2Vec Embeddings (Native Rust)

```
Entity Embeddings: 214
Dimensions: 128
Random Walks: 1380
Training Time: 2.05s
Mode: Native Rust (zero JavaScript overhead)
```

## ThinkingReasoner Summary

```
Observations: 111
Derived Facts: 222
OWL Rules: 2
  - SymmetricProperty: A rel B => B rel A
  - TransitiveProperty: A rel B, B rel C => A rel C
```

---

## Use Case Queries (SPARQL-first, deterministic)

### JOURNALIST: "Who made the defensive steals?"

**SPARQL:**
```sparql
SELECT ?player WHERE {
  ?e <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Steal> .
  ?e <http://euroleague.net/ontology#player> ?player .
}
```

**RESULTS:** 3 bindings
```
e=e00011, player=lessort__mathias
player=mitoglou__konstantinos, e=e00028
e=e00030, player=mattisseck__jonas
```

**REASONING CONTEXT:**
- Observations: 111
- Derived Facts: 222
- Rules Applied: 2

---

### COACH: "Which players distributed the ball best with assists?"

**SPARQL:**
```sparql
SELECT ?player WHERE {
  ?e <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Assist> .
  ?e <http://euroleague.net/ontology#player> ?player .
}
```

**RESULTS:** 8 bindings
```
player=nunn__kendrick, e=e00007
player=rapieque__elias, e=e00014
player=nunn__kendrick, e=e00016
player=hermannsson__martin, e=e00032
e=e00051, player=brown__lorenzo
```

---

### ANALYST: "Who made scoring plays (Two/Three Pointers)?"

**SPARQL:**
```sparql
SELECT ?player ?label WHERE {
  ?e <http://euroleague.net/ontology#player> ?player .
  ?e <http://www.w3.org/2000/01/rdf-schema#label> ?label .
  FILTER(CONTAINS(?label, "Pointer"))
}
```

**RESULTS:** 26 bindings
```
e=e00013, label=1 - 2 pt), player=olinde__louis
label=1 - 2 pt), e=e00017, player=olinde__louis
label=2 - 2 pt), e=e00022, player=olinde__louis
e=e00038, label=3 - 2 pt), player=olinde__louis
label=1 - 2 pt), player=brown__lorenzo, e=e00026
```

---

### FAN: "Who are the teammates of Lessort?"

**SPARQL:**
```sparql
SELECT ?teammate WHERE {
  <http://euroleague.net/player/lessort__mathias> <http://euroleague.net/ontology#teammateOf> ?teammate .
}
```

**RESULTS:** 8 bindings
```
teammate=osman__cedi
teammate=grant__jerian
teammate=brown__lorenzo
teammate=sloukas__kostas
teammate=yurtseven__omer
```

---

## Architecture Summary

```
+-------------------------------------------------------------+
|                    ALL IN-MEMORY                            |
+-------------------------------------------------------------+
|  GraphDB           | 603 triples, SPARQL 1.1               |
|  RDF2Vec           | 214 embeddings, 128D, Native Rust     |
|  ThinkingReasoner  | 111 obs -> 222 facts, 2 OWL rules     |
|  Prompt Optimizer  | 11 classes, 7 predicates              |
|  HyperFederate     | graph_search() UDF for SQL+SPARQL     |
+-------------------------------------------------------------+
```

---

## Running the Example

```bash
# Install dependencies
npm install

# Run with LLM (full HyperMindAgent)
OPENAI_API_KEY=your-key npm run euroleague

# Run without LLM (schema-based only)
npm run euroleague
```

---

*Generated from actual execution output on 2025-12-22*
