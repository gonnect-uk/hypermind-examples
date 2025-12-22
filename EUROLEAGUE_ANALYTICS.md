# Euroleague Basketball Knowledge Graph Analytics

This document shows **actual output** from running the Euroleague example with HyperMindAgent.

**Source**: [Representing Euroleague Play-by-Play Data as a Knowledge Graph](https://medium.com/@skontopo2009/representing-euroleague-play-by-play-data-as-a-knowledge-graph-6397534cdd75)

**Data Model**: [pbprdf - Play-by-Play RDF Ontology](https://github.com/andrewstellman/pbprdf)

## Test Results Summary

| Metric | Value |
|--------|-------|
| **Pass Rate** | 100.0% |
| **Tests Passed** | 17 |
| **Tests Failed** | 0 |

## Knowledge Graph Statistics

```
Triples: 603
Teams: 2 (BER, PAN)
Players: 22
Steals: 3
Assists: 8
Teammate links: 111
```

## RDF2Vec Embeddings

```
Entity Embeddings: 138
Dimensions: 128
Random Walks: 1380
Training Time: ~2.43s
```

## ThinkingReasoner (Deductive Reasoning)

```
Observations: 111
Derived Facts: 222
OWL Rules: 2
  - SymmetricProperty: A rel B => B rel A
  - TransitiveProperty: A rel B, B rel C => A rel C
```

## Prompt Optimization

```
Schema Classes: 11
Schema Predicates: 7
Mode: WASM RPC (in-memory)
```

---

## Actual Output

This is the **complete unedited output** from running `npm run euroleague`:

```
======================================================================
  EUROLEAGUE BASKETBALL KNOWLEDGE GRAPH
  HyperMindAgent with Deductive Reasoning + Assertions
======================================================================

Source: https://medium.com/@skontopo2009/
        representing-euroleague-play-by-play-data-as-a-knowledge-graph
Data Model: https://github.com/andrewstellman/pbprdf

[1] Loading Play-by-Play Knowledge Graph...
    Source: euroleague-api (pip install euroleague-api)
    Triples: 603

[2] SPARQL Queries with Assertions:

    [PASS] Teams count = 2 (BER, PAN)
    [PASS] Players count = 22
    [PASS] Steals count = 3 (Lessort, Mitoglou, Mattisseck)
    [PASS] Steal players are correct
    [PASS] Assist events count = 8
    [PASS] Teammate links = 111
    [PASS] Scoring events found

[3] Training RDF2Vec Embeddings...
    Generated 1380 random walks from 138 entities
    Trained: 213 embeddings (128D) in 2.43s
    Stored 138 entity embeddings in EmbeddingService
    [PASS] RDF2Vec embeddings generated

[4] Prompt Optimization (In-Memory Mode):

  Mode: WASM RPC (in-memory)
  Schema: Extracted from 603 triples
  Embeddings: 138 entities with RDF2Vec vectors

  SCHEMA CONTEXT (for LLM):
    Classes: 11
    Predicates: 7
    Namespace: auto-detected
    [PASS] Schema has classes
    [PASS] Schema has predicates

  GENERATED PROMPT (first 500 chars):
  You are a SQL query generator for HyperFederate.

  HyperFederate is a federated query engine that unifies:
  - SQL queries over structured data
  - Knowledge Graph queries via the `graph_search()` UDF
  - Cross-source joins between SQL tables and graph data

  ## HyperFederate SQL Grammar

  ```sql
  -- Basic Query Structure
  SELECT [DISTINCT] columns FROM source [WHERE cond] [GROUP BY cols] [ORDER BY cols] [LIMIT n]

  -- Knowledge Graph Access (the graph_search UDF)
  graph_search('SPARQL_QUERY') -> returns a t...

[5] ThinkingReasoner with Deductive Reasoning:

  Loading observations into ThinkingReasoner...
  Running deductive reasoning...
    Agent: euroleague-analyst
    LLM: None (schema-based)
    Observations: 111
    Derived Facts: 222
    Rules Applied: 2
    [PASS] Observations loaded = 111
    [PASS] Derived facts = 222 (symmetric property doubles links)
    [PASS] Rules applied = 2 (SymmetricProperty + TransitiveProperty)

[6] Thinking Graph (Derivation Chain / Proofs):

  EVIDENCE NODES (first 8):
    [OBS] none is teammate of osman__cedi
    [OBS] grant__jerian is teammate of osman__cedi
    [OBS] brown__lorenzo is teammate of osman__cedi
    [OBS] nunn__kendrick is teammate of osman__cedi
    [OBS] sloukas__kostas is teammate of osman__cedi
    [OBS] grigonis__marius is teammate of osman__cedi
    [OBS] lessort__mathias is teammate of osman__cedi
    [OBS] hernangomez__juancho is teammate of osman__cedi

  DERIVATION CHAIN (Proof Steps):
    Step 1: [OBSERVATION] none teammateOf osman__cedi
    Step 2: [OBSERVATION] grant__jerian teammateOf osman__cedi
    Step 3: [OBSERVATION] brown__lorenzo teammateOf osman__cedi
    Step 4: [OBSERVATION] nunn__kendrick teammateOf osman__cedi
    Step 5: [OBSERVATION] sloukas__kostas teammateOf osman__cedi
    Step 6: [OBSERVATION] grigonis__marius teammateOf osman__cedi
    Step 7: [OBSERVATION] lessort__mathias teammateOf osman__cedi
    Step 8: [OBSERVATION] hernangomez__juancho teammateOf osman__cedi

  DEDUCTIVE REASONING VALUE:
    - Every conclusion traces back to ground truth observations
    - SymmetricProperty: If A teammateOf B, then B teammateOf A
    - TransitiveProperty: If A assistedBy B, B assistedBy C, then A assistedBy C
    - No hallucinations - only provable facts with derivation chains

[7] Use Case Queries (SPARQL-first, deterministic):

------------------------------------------------------------
JOURNALIST: "Who made the defensive steals?"
VALUE: Uncover storylines beyond surface-level stats
------------------------------------------------------------

SPARQL:
```sparql
SELECT ?player WHERE {
    ?e <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Steal> .
    ?e <http://euroleague.net/ontology#player> ?player .
  }
```

RESULTS: 3 bindings
SAMPLE (first 5):
  player=lessort__mathias, e=e00011
  e=e00028, player=mitoglou__konstantinos
  e=e00030, player=mattisseck__jonas
    [PASS] JOURNALIST: Who made the defensive steals?

REASONING CONTEXT:
  Observations: 111
  Derived Facts: 222
  Rules Applied: 2

------------------------------------------------------------
COACH: "Which players distributed the ball best with assists?"
VALUE: Identify team chemistry for strategic planning
------------------------------------------------------------

SPARQL:
```sparql
SELECT ?player WHERE {
    ?e <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://euroleague.net/ontology#Assist> .
    ?e <http://euroleague.net/ontology#player> ?player .
  }
```

RESULTS: 8 bindings
SAMPLE (first 5):
  e=e00007, player=nunn__kendrick
  player=rapieque__elias, e=e00014
  e=e00016, player=nunn__kendrick
  e=e00032, player=hermannsson__martin
  player=brown__lorenzo, e=e00051
    [PASS] COACH: Which players distributed the ball best with assists?

REASONING CONTEXT:
  Observations: 111
  Derived Facts: 222
  Rules Applied: 2

------------------------------------------------------------
ANALYST: "Who made scoring plays (Two/Three Pointers)?"
VALUE: Enriched interconnected data for modeling
------------------------------------------------------------

SPARQL:
```sparql
SELECT ?player ?label WHERE {
    ?e <http://euroleague.net/ontology#player> ?player .
    ?e <http://www.w3.org/2000/01/rdf-schema#label> ?label .
    FILTER(CONTAINS(?label, "Pointer"))
  }
```

RESULTS: 26 bindings
SAMPLE (first 5):
  e=e00013, player=olinde__louis, label=1 - 2 pt)
  player=olinde__louis, label=1 - 2 pt), e=e00017
  e=e00022, player=olinde__louis, label=2 - 2 pt)
  e=e00038, player=olinde__louis, label=3 - 2 pt)
  e=e00026, label=1 - 2 pt), player=brown__lorenzo
    [PASS] ANALYST: Who made scoring plays (Two/Three Pointers)?

REASONING CONTEXT:
  Observations: 111
  Derived Facts: 222
  Rules Applied: 2

------------------------------------------------------------
FAN: "Who are the teammates of Lessort?"
VALUE: Interactive exploration of team dynamics
------------------------------------------------------------

SPARQL:
```sparql
SELECT ?teammate WHERE {
        <http://euroleague.net/player/lessort__mathias> <http://euroleague.net/ontology#teammateOf> ?teammate .
      }
```

RESULTS: 8 bindings
SAMPLE (first 5):
  teammate=osman__cedi
  teammate=grant__jerian
  teammate=brown__lorenzo
  teammate=sloukas__kostas
  teammate=yurtseven__omer
    [PASS] FAN: Who are the teammates of Lessort?

REASONING CONTEXT:
  Observations: 111
  Derived Facts: 222
  Rules Applied: 2

[8] HyperMindAgent Natural Language: Skipped (no OPENAI_API_KEY)
    Set OPENAI_API_KEY environment variable to enable LLM-assisted queries.

======================================================================
  TEST RESULTS SUMMARY
======================================================================

  PASSED: 17
  FAILED: 0
  TOTAL:  17

  PASS RATE: 100.0%

======================================================================
  ARCHITECTURE SUMMARY - ALL IN-MEMORY
======================================================================

  KNOWLEDGE GRAPH (In-Memory):
    Triples: 603
    Teams: 2
    Players: 22
    Steals: 3
    Assists: 8
    Teammate links: 111

  RDF2VEC EMBEDDINGS (In-Memory):
    Entity Embeddings: 138
    Dimensions: 128
    Random Walks: 1380

  PROMPT OPTIMIZATION (In-Memory):
    Schema Classes: 11
    Schema Predicates: 7
    Mode: WASM RPC (no external services)

  THINKING REASONER (In-Memory):
    Observations: 111
    Derived Facts: 222
    OWL Rules: 2
    - SymmetricProperty: A rel B => B rel A
    - TransitiveProperty: A rel B, B rel C => A rel C

  BENEFITS:
    - Zero latency: No network I/O
    - Offline capable: Works without internet
    - Privacy: All data in process memory
    - Verifiable: Assertions prove correctness
```

---

## Key Insights

### 1. Deductive Reasoning with Proofs

The ThinkingReasoner loads 111 observations from the knowledge graph and derives 222 facts using OWL rules:

- **SymmetricProperty**: `teammateOf` is symmetric, so if A is teammate of B, then B is teammate of A
- **TransitiveProperty**: `assistedBy` is transitive, enabling assist chain inference

Every derived fact has a **derivation chain** (proof) that traces back to ground truth observations.

### 2. RDF2Vec Semantic Embeddings

The example trains 128-dimensional embeddings on 1,380 random walks:

- Entity embeddings capture graph structure
- Enables semantic similarity search
- Works entirely in-memory (~2.43s training)

### 3. Prompt Optimization

The schema is extracted at load time and used to build LLM prompts:

- 11 OWL classes extracted
- 7 predicates available
- SQL-first approach with `graph_search()` UDF

### 4. Use Case Coverage

| Persona | Query | Results | Status |
|---------|-------|---------|--------|
| Journalist | Who made steals? | 3 players (lessort, mitoglou, mattisseck) | PASS |
| Coach | Who had assists? | 8 assists (nunn x2, rapieque, hermannsson, brown) | PASS |
| Analyst | Scoring plays? | 26 events (olinde, brown, etc.) | PASS |
| Fan | Lessort's teammates? | 8 teammates (osman, grant, brown, sloukas, yurtseven) | PASS |

---

## Running the Example

```bash
# Install dependencies
npm install

# Run with optional LLM
OPENAI_API_KEY=your-key npm run euroleague

# Run without LLM (schema-based only)
npm run euroleague
```

## Architecture

```
+-------------------------------------------------------------+
|                    ALL IN-MEMORY                            |
+-------------------------------------------------------------+
|  GraphDB           | 603 triples, SPARQL 1.1               |
|  RDF2Vec           | 138 embeddings, 128D                  |
|  ThinkingReasoner  | 111 obs -> 222 facts, 2 OWL rules     |
|  Prompt Optimizer  | 11 classes, 7 predicates              |
+-------------------------------------------------------------+
```

---

*Generated from actual execution output on 2025-12-22*
