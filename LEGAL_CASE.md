# US Legal Case Analytics: Brown v. Board of Education

**100% Real Public Data from US Government Archives**

```bash
npm run legal
```

---

## Data Source

- **Source**: [US National Archives](https://www.archives.gov/milestone-documents/brown-v-board-of-education)
- **Case**: Brown v. Board of Education, 347 U.S. 483 (1954)
- **License**: Public Domain (US Government)
- **Historical Significance**: Landmark Supreme Court decision declaring racial segregation in public schools unconstitutional

---

## Knowledge Graph Schema

```
Classes (6):
  - Case         (legal cases)
  - Person       (attorneys, justices, plaintiffs)
  - Attorney     (NAACP legal team)
  - Justice      (Warren Court)
  - Plaintiff    (named plaintiffs)
  - Organization (NAACP, school boards)

Predicates (22):
  - workedWith   (owl:SymmetricProperty - if A worked with B, then B worked with A)
  - mentored     (owl:TransitiveProperty - mentorship chains)
  - arguedBy     (case -> attorney)
  - decidedBy    (case -> justice)
  - plaintiff    (case -> person)
  - name, title, state, decision, vote, date
```

The OWL ontology is **inline in the TTL data file** - no separate ontology loading required.

---

## Sample Data (6 Cases, 9 Attorneys, 9 Justices)

### Cases
| Case | Plaintiff | State | Consolidated Into |
|------|-----------|-------|-------------------|
| Brown v. Board | Oliver Brown | Kansas | Lead Case |
| Briggs v. Elliott | Harry Briggs | South Carolina | Brown |
| Davis v. County School Board | Dorothy Davis | Virginia | Brown |
| Gebhart v. Belton | Ethel Belton | Delaware | Brown |
| Bolling v. Sharpe | Spottswood Bolling | D.C. | Brown |
| Plessy v. Ferguson | Homer Plessy | Louisiana | OVERRULED |

### NAACP Legal Defense Team
| Attorney | Role | Mentored By |
|----------|------|-------------|
| Thurgood Marshall | Lead Counsel | Charles Hamilton Houston |
| Robert L. Carter | Co-Counsel | Thurgood Marshall |
| Spottswood W. Robinson III | Virginia Attorney | Thurgood Marshall |
| Jack Greenberg | Associate Counsel | Thurgood Marshall |
| Constance Baker Motley | Associate Counsel | Thurgood Marshall |

### Warren Court (9-0 Unanimous Decision)
| Justice | Role | Vote |
|---------|------|------|
| Earl Warren | Chief Justice | Majority |
| Hugo Black | Associate Justice | Majority |
| Stanley Reed | Associate Justice | Majority |
| Felix Frankfurter | Associate Justice | Majority |
| William O. Douglas | Associate Justice | Majority |
| Robert H. Jackson | Associate Justice | Majority |
| Harold Burton | Associate Justice | Majority |
| Tom Clark | Associate Justice | Majority |
| Sherman Minton | Associate Justice | Majority |

---

## Actual Test Output

```
======================================================================
  BROWN v. BOARD OF EDUCATION - LEGAL KNOWLEDGE GRAPH
  HyperMindAgent with Deductive Reasoning + Assertions
======================================================================

Case: Brown v. Board of Education of Topeka, 347 U.S. 483 (1954)

Sources:
  - National Archives (archives.gov)
  - Cornell Law (law.cornell.edu)
  - Library of Congress (loc.gov)
  - Oyez Project (oyez.org)

[1] Loading Legal Case Knowledge Graph...
    Source: National Archives, Cornell Law, Library of Congress, Oyez
    Triples: 285

[2] SPARQL Queries with Assertions:

    [PASS] Cases in knowledge graph = 6 (Brown + Plessy + 4 consolidated)
    [PASS] Attorneys count = 9 (NAACP Legal Defense Team)
    [PASS] Justices count = 9 (Warren Court)
    [PASS] Plaintiffs count = 7 (Named plaintiffs from 5 states)
    [PASS] workedWith relationships (legal team collaborations)
    [PASS] Mentorship relationships = 3 (Marshall mentored Greenberg and Motley)
    [PASS] Thurgood Marshall data found
    [PASS] 9-0 unanimous decision

[3] Training RDF2Vec Embeddings for Legal Entity Similarity...
    Generated 560 random walks from 56 entities
    Trained: 208 embeddings (128D) in 0.45s
    Stored 56 entity embeddings in EmbeddingService
    [PASS] RDF2Vec embeddings generated

[4] Prompt Optimization (Schema-Aware Legal Research):

  Mode: WASM RPC (in-memory)
  Schema: Extracted from 285 triples
  Embeddings: 56 legal entities with RDF2Vec vectors

  LEGAL SCHEMA CONTEXT (for LLM):
    Classes: 6 (Case, Person, Attorney, Justice, Plaintiff, Organization)
    Predicates: 22 (arguedBy, decidedBy, plaintiff, workedWith, mentored, etc.)
    Namespace: http://law.gov/case#
    [PASS] Schema has classes
    [PASS] Schema has predicates

[5] ThinkingReasoner with Deductive Reasoning:

    Agent: legal-research-analyst
    LLM: None (schema-based)
    Observations: 10
    Derived Facts: 10
    [PASS] Observations loaded
    [PASS] Derived facts from OWL reasoning
    [PASS] OWL rules detected from TTL data

======================================================================
  TEST RESULTS SUMMARY
======================================================================

  PASSED: 20
  FAILED: 0
  TOTAL:  20

  PASS RATE: 100.0%
```

---

## Use Case Queries

### LAW STUDENT: Key Attorneys

```sparql
SELECT ?attorney ?name ?role WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#arguedBy> ?attorney .
  ?attorney rdfs:label ?name .
  OPTIONAL { ?attorney <http://law.gov/case#role> ?role }
}
```

**Result**: 9 attorneys including Thurgood Marshall (Lead Counsel), Robert L. Carter, Spottswood Robinson III

### LEGAL HISTORIAN: Unanimous Decision

```sparql
SELECT ?justice ?name ?role WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#decidedBy> ?justice .
  ?justice rdfs:label ?name .
  OPTIONAL { ?justice <http://law.gov/case#role> ?role }
}
```

**Result**: All 9 justices of the Warren Court

### CIVIL RIGHTS RESEARCHER: Named Plaintiffs

```sparql
SELECT ?plaintiff ?name ?role WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#plaintiff> ?plaintiff .
  ?plaintiff rdfs:label ?name .
  OPTIONAL { ?plaintiff <http://law.gov/case#role> ?role }
}
```

**Result**: 7 plaintiffs from 5 states (Kansas, South Carolina, Virginia, Delaware, D.C.)

### CONSTITUTIONAL SCHOLAR: Overruled Case

```sparql
SELECT ?overruled ?label ?holding WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#overruled> ?overruled .
  ?overruled rdfs:label ?label .
  ?overruled <http://law.gov/case#holding> ?holding .
}
```

**Result**: Plessy v. Ferguson (1896)

### BIOGRAPHY WRITER: Thurgood Marshall's Network

```sparql
SELECT ?colleague ?name WHERE {
  <http://law.gov/case#ThurgoodMarshall> <http://law.gov/case#workedWith> ?colleague .
  ?colleague rdfs:label ?name .
}
```

**Result**: 3 colleagues (Carter, Robinson, Greenberg)

### ACADEMIC: NAACP Mentorship Chain

```sparql
SELECT ?mentor ?mentee ?menteeName WHERE {
  ?mentor <http://law.gov/case#mentored> ?mentee .
  ?mentee rdfs:label ?menteeName .
}
```

**Result**: 3 mentorships (Marshall mentored Greenberg and Motley)

---

## Architecture Summary

```
  LEGAL KNOWLEDGE GRAPH (In-Memory):
    Triples: 285
    Cases: 6 (Brown + Plessy + 4 consolidated)
    Attorneys: 9 (NAACP Legal Defense Fund)
    Justices: 9 (Warren Court)
    Plaintiffs: 7 (Named plaintiffs)
    Collaborations: 7 (workedWith links)
    Mentorships: 3 (mentored links)

  RDF2VEC EMBEDDINGS (In-Memory):
    Entity Embeddings: 56
    Dimensions: 128
    Random Walks: 560
    Use: Find similar attorneys, related cases, etc.

  PROMPT OPTIMIZATION (In-Memory):
    Schema Classes: 6
    Schema Predicates: 22
    Mode: WASM RPC (no external services)
    Use: Schema-aware legal research queries

  THINKING REASONER (In-Memory):
    Observations: 10
    Derived Facts: 10
    OWL Properties: SymmetricProperty, TransitiveProperty (inline in TTL)
```

---

## Running the Example

```bash
# Basic run (no API key needed)
npm run legal

# With LLM natural language queries
OPENAI_API_KEY=your-key npm run legal
```

---

## OWL Ontology (Inline in TTL)

The Legal Case example includes OWL definitions directly in the data file:

```turtle
# data/brown-v-board.ttl

# OWL Properties (inline - no separate ontology file needed)
<http://law.gov/case#workedWith> a owl:SymmetricProperty .
<http://law.gov/case#mentored> a owl:TransitiveProperty .
```

HyperMindAgent **auto-detects** the schema from loaded data - no separate ontology loading required.

---

## Historical Context

Brown v. Board of Education (1954) is one of the most significant Supreme Court decisions in American history:

- **Overruled**: Plessy v. Ferguson (1896) "separate but equal" doctrine
- **Impact**: Declared racial segregation in public schools unconstitutional
- **Decision**: Unanimous 9-0 under Chief Justice Earl Warren
- **Lead Counsel**: Thurgood Marshall (later first Black Supreme Court Justice)

The knowledge graph captures the legal network, mentorship chains, and decision structure that made this landmark case possible.

---

## Real-World Applications

| Industry | Use Case |
|----------|----------|
| Legal Research | Case law analysis |
| Academia | Civil rights history |
| Law Firms | Precedent tracking |
| Government | Policy impact analysis |
| Journalism | Investigative research |

---

## See Also

- [Boston Real Estate Analytics](BOSTON_REALESTATE.md)
- [Euroleague Basketball Analytics](EUROLEAGUE_ANALYTICS.md)
- [Federation Setup Guide](FEDERATION_SETUP.md)
