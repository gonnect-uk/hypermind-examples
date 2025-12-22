# Euroleague Basketball Analytics with Knowledge Graphs

> **Deductive reasoning on play-by-play data. No hallucinations. Every insight has a proof.**

Inspired by my friend Stratos Kontopanagos: [Representing Euroleague Play-by-Play Data as a Knowledge Graph](https://medium.com/@skontopo2009/representing-euroleague-play-by-play-data-as-a-knowledge-graph-6397534cdd75)

**Data Source:** Same as Stratos's article - [euroleague-api](https://pypi.org/project/euroleague-api/) Python package

**Engine:** [KGDB (Gonnect)](https://www.npmjs.com/package/rust-kgdb) - 449ns lookups, 24 bytes/triple

---

## The Problem with Traditional Analytics

Traditional basketball analytics tools give you numbers. They don't give you *reasoning*.

Ask GPT-4: "Who made the most steals in this game?"

You get: *"Based on general basketball knowledge, point guards typically..."* - a guess based on patterns, not data.

Ask HyperMind the same question:

```
ANSWER: Lessort Mathias (1 steal), Mitoglou Konstantinos (1 steal), Mattisseck Jonas (1 steal)

PROOF:
  Step 1: [OBSERVATION] e00042 is Steal, player = lessort__mathias
  Step 2: [OBSERVATION] e00047 is Steal, player = mitoglou__konstantinos
  Step 3: [OBSERVATION] e00051 is Steal, player = mattisseck__jonas
  Step 4: [INFERENCE] steal_count(lessort, 1), steal_count(mitoglou, 1), steal_count(mattisseck, 1)

  Evidence: KG triples from euroleague-api
  Hash: sha256:7a3f2b1c...
```

Every conclusion is traceable. Auditable. Provable.

---

## Actual Demo Output (euroleague-api Data)

```
======================================================================
  EUROLEAGUE BASKETBALL KNOWLEDGE GRAPH
  HyperMindAgent with Deductive Reasoning
======================================================================

Source: https://medium.com/@skontopo2009/
        representing-euroleague-play-by-play-data-as-a-knowledge-graph
Data Model: https://github.com/andrewstellman/pbprdf

[1] Loading Play-by-Play Knowledge Graph...
    Source: euroleague-api (pip install euroleague-api)
    Triples: 596

[2] Teams in the Graph:
    - Ber (Alba Berlin)
    - Pan (Panathinaikos)

[3] Players and Teams:
    - Nunn Kendrick (pan)
    - Sloukas Kostas (pan)
    - Lessort Mathias (pan)
    - Grigonis Marius (pan)
    - Mitoglou Konstantinos (pan)
    - Williams Trevion (ber)
    - Spagnolo Matteo (ber)
    - Koumadje Khalifa (ber)
    - Hermannsson Martin (ber)
    ... (22 players total)

[4] Play-by-Play Events (sample):
    - Steal (1) by lessort__mathias
    - Steal (1) by mitoglou__konstantinos
    - Steal (1) by mattisseck__jonas
    - Assist (1) by nunn__kendrick
    - Assist (2) by nunn__kendrick
    - Assist (1) by williams__trevion
    - Assist (2) by williams__trevion
    - Three Pointer (1/1 - 3 pt) by mitoglou__konstantinos
    - Three Pointer (1/1 - 3 pt) by grigonis__marius
    - Two Pointer (3/3 - 8 pt) by lessort__mathias
    - Free Throw In (2/2 - 6 pt) by lessort__mathias
    ... (100 events total)

[7] Creating HyperMindAgent with ThinkingReasoner...
    Agent: euroleague-analyst
    Observations: 111
    Derived Facts: 222
    Rules Applied: 2

[9] Thinking Graph (Explainable AI):

  EVIDENCE NODES:
  [OBS] nunn__kendrick is teammate of sloukas__kostas
  [OBS] lessort__mathias is teammate of grigonis__marius
  [OBS] williams__trevion is teammate of spagnolo__matteo
  ...

  DERIVATION CHAIN (Proof Steps):
  Step 1: [OBSERVATION] nunn__kendrick teammateOf sloukas__kostas
  Step 2: [OBSERVATION] lessort__mathias teammateOf grigonis__marius
  Step 3: [OBSERVATION] williams__trevion teammateOf spagnolo__matteo
  ...

  VALUE: Every conclusion traces back to ground truth observations.
         No hallucinations - only provable facts.

======================================================================
  SUMMARY
======================================================================

  Knowledge Graph: 596 triples
  Teams: 2 (Berlin vs Panathinaikos)
  Players: 22
  Events: 100
  Teammate links: 111
  Observations: 111
  Derived Facts: 222
  Rules Applied: 2
```

---

## Analytics Questions We Can Answer

### 1. Who made defensive plays (steals + blocks)?

**From the actual data:**

| Player | Steals | Blocks |
|--------|--------|--------|
| Lessort Mathias | 1 | - |
| Mitoglou Konstantinos | 1 | - |
| Mattisseck Jonas | 1 | - |

**SPARQL Query:**
```sparql
SELECT ?player (COUNT(?e) AS ?defensive_actions) WHERE {
  { ?e a <http://euroleague.net/ontology#Steal> }
  UNION
  { ?e a <http://euroleague.net/ontology#Block> }
  ?e <http://euroleague.net/ontology#player> ?player .
}
GROUP BY ?player
ORDER BY DESC(?defensive_actions)
```

---

### 2. Who made the most assists?

**From the actual data:**

| Player | Assists |
|--------|---------|
| Nunn Kendrick | 2 |
| Williams Trevion | 2 |
| Rapieque Elias | 1 |
| Hermannsson Martin | 1 |
| Brown Lorenzo | 1 |
| Lessort Mathias | 1 |

---

### 3. Teammate Chemistry (OWL SymmetricProperty)

The `teammateOf` property is defined as `owl:SymmetricProperty`:

```
euro:teammateOf a owl:SymmetricProperty .
```

**Automatic Deduction:** If Nunn `teammateOf` Sloukas, then Sloukas `teammateOf` Nunn.

**From the actual data (111 teammate links):**
```
nunn__kendrick <-> sloukas__kostas
nunn__kendrick <-> lessort__mathias
nunn__kendrick <-> grigonis__marius
lessort__mathias <-> grigonis__marius
williams__trevion <-> spagnolo__matteo
...
```

---

### 4. Top Scorers

**From the actual data:**

| Player | 2PT | 3PT | FT | Total |
|--------|-----|-----|-----|-------|
| Lessort Mathias | 8 | 0 | 6 | 14 |
| Mitoglou Konstantinos | 0 | 3 | 5 | 8 |
| Nunn Kendrick | 2 | 0 | 4 | 6 |
| Spagnolo Matteo | 4 | 0 | 0 | 4 |

---

## Questions Requiring Additional Data Fields

The current euroleague-api data includes basic play-by-play. These advanced queries require additional fields:

| Question | Required Field | Status |
|----------|---------------|--------|
| Clutch performance (last 2 min) | `gameTime`, `scoreDifferential` | Not in current API response |
| Plus-minus chemistry | `plusMinus` per lineup | Needs lineup tracking |
| Performance after timeouts | `timeout` event + `nextPlay` | Partial (TV timeouts exist) |
| Free throw % under pressure | `gameTime`, `scoreDifferential` | Needs game clock data |

**To add these fields:** Extend `scripts/euroleague-to-ttl.py` to extract additional columns from the euroleague-api DataFrame.

---

## Architecture: KGDB + HyperMind

```
Natural Language Query          "Find players with most assists"
        |
+------------------+
|  HyperMindAgent  |  Schema extraction -> valid SPARQL
+------------------+
        |
+------------------+
| ThinkingReasoner |  OWL rules -> derived facts (222 from 111)
+------------------+
        |
+------------------+
|   KGDB (Gonnect) |  449ns lookups, 24 bytes/triple
+------------------+
        |
Answer + Derivation Chain + SHA-256 Proof
```

**Key differentiator:** Every step is deterministic. No probability. No hallucination.

---

## OWL Properties Enable Automatic Reasoning

| You Define | System Derives |
|------------|----------------|
| `owl:SymmetricProperty` | If A rel B -> B rel A |
| `owl:TransitiveProperty` | If A rel B, B rel C -> A rel C |
| `rdfs:subClassOf` | Steal subClassOf Event -> all steals are events |

Load your ontology. Rules generate automatically.

---

## Run It Yourself

```bash
git clone https://github.com/gonnect-uk/hypermind-examples.git
cd hypermind-examples

# Generate data from euroleague-api
uv run --with euroleague-api python3 scripts/euroleague-to-ttl.py

# Run the demo (without LLM - uses schema-based reasoning)
npm install
npm run euroleague

# Run with OpenAI for natural language -> SPARQL generation
OPENAI_API_KEY=your-key npm run euroleague
```

### With LLM vs Without LLM

| Mode | How to Run | What You Get |
|------|------------|--------------|
| **With OpenAI** | `OPENAI_API_KEY=your-key npm run euroleague` | Natural language -> generated SPARQL, LLM answers |
| **Without LLM** | `npm run euroleague` | Schema-based reasoning, OWL rule deduction, proofs |

Both modes show:
- Knowledge graph with 596 triples
- OWL SymmetricProperty (teammateOf) automatic deduction
- ThinkingReasoner with 111 observations, 222 derived facts
- Derivation chains (provable facts)

---

## Data Pipeline

```
euroleague-api (Python)
        |
        v
scripts/euroleague-to-ttl.py  -->  data/euroleague-game.ttl
        |
        v
examples/euroleague-basketball-agent.js
        |
        v
KGDB (596 triples) --> HyperMindAgent --> Reasoning Output
```

---

## Why Knowledge Graphs for Sports Analytics

From Stratos's article:

> "Knowledge Graphs provide a powerful framework for representing complex relationships in basketball data. Unlike tabular data, KGs enable semantic queries that understand the meaning of relationships."

**Traditional approach:** SQL tables with joins.
**KG approach:** `?player teammateOf ?other . ?player madeShot ?shot .`

The difference: KGs understand that "teammates" is a symmetric relationship. SQL doesn't.

---

## Summary

| Metric | Value |
|--------|-------|
| Data Source | euroleague-api (pip install) |
| Knowledge Graph | 596 triples |
| Teams | 2 (BER vs PAN) |
| Players | 22 |
| Events | 100 |
| Observations | 111 |
| Derived Facts | 222 |
| Rules Applied | 2 |

---

## Contact

For enterprise deployments with full season data:
**gonnect.hypermind@gmail.com**

---

*Built with [KGDB (Gonnect)](https://www.npmjs.com/package/rust-kgdb) - 449ns lookups, 24 bytes/triple*
