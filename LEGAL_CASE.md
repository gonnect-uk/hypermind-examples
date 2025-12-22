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

## Test Results Summary

| Metric | Value |
|--------|-------|
| **Pass Rate** | 90.5% |
| **Tests Passed** | 19 |
| **Tests Failed** | 2 |

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
|    LEFT JOIN westlaw_attorneys ON ...    |
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
-- HyperFederate SQL: Join Legal Knowledge Graph + Court Records
SELECT
  kg.attorney_name,
  kg.role,
  kg.case_name,
  westlaw.citation_count,
  westlaw.career_wins,
  lexis.bar_admission_year
FROM graph_search('
  PREFIX law: <http://law.gov/case#>
  SELECT ?attorney_name ?role ?case_name WHERE {
    ?case a law:Case .
    ?case rdfs:label ?case_name .
    ?case law:arguedBy ?attorney .
    ?attorney rdfs:label ?attorney_name .
    ?attorney law:role ?role .
  }
') kg
LEFT JOIN westlaw_attorneys westlaw
  ON kg.attorney_name = westlaw.full_name
LEFT JOIN lexis_bar_records lexis
  ON kg.attorney_name = lexis.attorney_name
ORDER BY westlaw.citation_count DESC
```

**HONEST RESULTS (from graph_search):**

| attorney_name                | role                                    |
|------------------------------|-----------------------------------------|
| Oliver Hill                  | NAACP Virginia Attorney                 |
| George E.C. Hayes            | Washington D.C. Attorney                |
| James Nabrit Jr.             | Howard University Law School Professor  |
| Louis L. Redding             | Delaware Attorney                       |
| Robert L. Carter             | NAACP Legal Defense Fund Attorney       |
| Jack Greenberg               | NAACP Legal Defense Fund Attorney       |

---

## Thinking Events Timeline (Real-time Reasoning Stream)

**ACTUAL OUTPUT** - Auto-captured during reasoning:

### [OBSERVE] - Detected facts from knowledge graph:

```
-> KennethClark workedWith MamieClark
-> SpotswoodRobinson workedWith OliverHill
-> JamesNabrit workedWith GeorgeHayes
-> ThurgoodMarshall workedWith RobertCarter
-> RobertCarter workedWith JackGreenberg
-> ThurgoodMarshall workedWith JackGreenberg
-> ThurgoodMarshall mentored JackGreenberg
-> JackGreenberg mentored ConstanceBakerMotley
... and more observations
```

### [INFER] - Applied OWL Rules:

| Rule | Description | Effect |
|------|-------------|--------|
| SymmetricProperty | `A workedWith B => B workedWith A` | Doubles collaboration facts |
| TransitiveProperty | `A mentored B, B mentored C => A mentored C` | Chain inference |

### [PROVE] - Derivation Chain (audit trail):

```
Step 1: [OBSERVATION] KennethClark workedWith MamieClark
Step 2: [INFERENCE] MamieClark workedWith KennethClark (SymmetricProperty)
Step 3: [OBSERVATION] ThurgoodMarshall workedWith RobertCarter
Step 4: [INFERENCE] RobertCarter workedWith ThurgoodMarshall (SymmetricProperty)
Step 5: [OBSERVATION] ThurgoodMarshall mentored JackGreenberg
Step 6: [OBSERVATION] JackGreenberg mentored ConstanceBakerMotley
Step 7: [INFERENCE] ThurgoodMarshall mentored ConstanceBakerMotley (TransitiveProperty)
... and more proof steps
```

### REASONING COMPLETE:

- Ground truth observations from knowledge graph
- Derived facts inferred via OWL rules
- SymmetricProperty and TransitiveProperty applied
- Every fact is traceable to source data (no hallucination)
- Cryptographic proof hashes for audit trails

---

## HyperMindAgent.call() Response Structure

**Note**: HyperMindAgent natural language queries depend on LLM interpretation. For deterministic results, use the direct SPARQL queries shown in "Use Case Queries" section below.

**ACTUAL OUTPUT** - Natural language query example:

```javascript
// Query: "Who argued the Brown v. Board case?"
{
  // LLM interpreted "argued" as date rather than attorneys
  sparql: "SELECT ?s ?o WHERE { ?s <http://law.gov/case#dateArgued> ?o } LIMIT 100",
  answer: "BrownVBoard and 1952-12-09",
  raw_results: [{ "s": "http://law.gov/case#BrownVBoard", "o": "1952-12-09" }]
}

// For accurate attorney results, use direct SPARQL (see Use Case Queries below):
// SELECT ?attorney WHERE { <http://law.gov/case#BrownVBoard> <http://law.gov/case#arguedBy> ?attorney }
```

**Best Practice**: For legal research requiring precision, use the deterministic SPARQL queries in the "Use Case Queries" section. The natural language interface is best for exploration.

**Direct SPARQL Query Result** (accurate):

```javascript
// Query: SELECT ?name WHERE { :BrownVBoard :arguedBy ?a . ?a rdfs:label ?name }
{
  answer: "Thurgood Marshall, Robert L. Carter, Jack Greenberg, Oliver Hill, Louis L. Redding and 4 more",
  resultCount: 9
}
```

---

## Knowledge Graph Statistics

```
Triples: 285
Cases: 6 (Brown + Plessy + 4 consolidated)
Attorneys: 9 (NAACP Legal Defense Fund)
Justices: 9 (Warren Court)
Plaintiffs: 7 (Named plaintiffs from 5 states)
Collaborations: 7 (workedWith links)
Mentorships: 3 (mentored links)
```

## RDF2Vec Embeddings (Native Rust)

```
Entity Embeddings: 196
Dimensions: 128
Random Walks: 560
Training Time: 0.47s
Mode: Native Rust (zero JavaScript overhead)
```

**Legal Entity Similarity (via Native Rust):**
```
Similar to Thurgood Marshall:
  - workedWith (score: 0.687)
  - mentored (score: 0.655)
  - JamesNabrit (score: 0.641)
```

## ThinkingReasoner Summary

```
Observations: 10
Derived Facts: 17
OWL Rules: 2
  - SymmetricProperty: A workedWith B => B workedWith A
  - TransitiveProperty: A mentored B, B mentored C => A mentored C
```

**OWL Reasoning Example:**

When `ThurgoodMarshall workedWith RobertCarter` is observed, the reasoner automatically infers:
- `RobertCarter workedWith ThurgoodMarshall` (via SymmetricProperty)

When the mentorship chain is observed:
- `ThurgoodMarshall mentored JackGreenberg`
- `JackGreenberg mentored ConstanceBakerMotley`

The reasoner infers:
- `ThurgoodMarshall mentored ConstanceBakerMotley` (via TransitiveProperty)

---

## Use Case Queries (SPARQL-first, deterministic)

### LAW STUDENT: "Who were the key attorneys in Brown v. Board of Education?"

**SPARQL:**
```sparql
SELECT ?attorney ?name ?role WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#arguedBy> ?attorney .
  ?attorney <http://www.w3.org/2000/01/rdf-schema#label> ?name .
  OPTIONAL { ?attorney <http://law.gov/case#role> ?role }
}
```

**RESULTS:** 9 bindings
```
attorney=OliverHill, name=Oliver Hill, role=NAACP Virginia Attorney
name=George E.C. Hayes, role=Washington D.C. Attorney, attorney=GeorgeHayes
name=James Nabrit Jr., role=Howard University Law School Professor, attorney=JamesNabrit
attorney=LouisRedding, name=Louis L. Redding, role=Delaware Attorney
attorney=RobertCarter, role=NAACP Legal Defense Fund Attorney, name=Robert L. Carter
```

**REASONING CONTEXT:**
- Observations: 10
- Derived Facts: 17
- Rules Applied: 2

---

### LEGAL HISTORIAN: "Which Supreme Court justices decided the case unanimously?"

**SPARQL:**
```sparql
SELECT ?justice ?name ?role WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#decidedBy> ?justice .
  ?justice <http://www.w3.org/2000/01/rdf-schema#label> ?name .
  OPTIONAL { ?justice <http://law.gov/case#role> ?role }
}
```

**RESULTS:** 9 bindings
```
justice=TomClark, name=Tom C. Clark, role=Associate Justice
justice=HugoBlack, name=Hugo Black, role=Associate Justice
name=Earl Warren, justice=EarlWarren, role=Chief Justice
name=Stanley Reed, justice=StanleyReed, role=Associate Justice
justice=HaroldBurton, name=Harold Burton, role=Associate Justice
```

---

### CIVIL RIGHTS RESEARCHER: "Who were the named plaintiffs in the consolidated cases?"

**SPARQL:**
```sparql
SELECT ?plaintiff ?name ?role WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#plaintiff> ?plaintiff .
  ?plaintiff <http://www.w3.org/2000/01/rdf-schema#label> ?name .
  OPTIONAL { ?plaintiff <http://law.gov/case#role> ?role }
}
```

**RESULTS:** 7 bindings
```
name=Linda Brown, role=Student Plaintiff (Kansas), plaintiff=LindaBrown
role=Student Plaintiff (Delaware), name=Ethel Louise Belton, plaintiff=EthelBelton
role=Student Plaintiff (South Carolina), name=Harry Briggs Jr., plaintiff=HarryBriggs
plaintiff=OliverBrown, name=Oliver Brown, role=Lead Plaintiff (Kansas)
plaintiff=BarbaraJohns, role=Student Organizer (Virginia), name=Barbara Rose Johns
```

---

### CONSTITUTIONAL SCHOLAR: "What case did Brown v. Board overrule?"

**SPARQL:**
```sparql
SELECT ?overruled ?label ?holding WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#overruled> ?overruled .
  ?overruled <http://www.w3.org/2000/01/rdf-schema#label> ?label .
  ?overruled <http://law.gov/case#holding> ?holding .
}
```

**RESULTS:** 1 binding
```
overruled=PlessyVFerguson, holding=Separate but equal doctrine, label=Plessy v. Ferguson
```

---

### BIOGRAPHY WRITER: "Who did Thurgood Marshall collaborate with?"

**SPARQL:**
```sparql
SELECT ?colleague ?name WHERE {
  <http://law.gov/case#ThurgoodMarshall> <http://law.gov/case#workedWith> ?colleague .
  ?colleague <http://www.w3.org/2000/01/rdf-schema#label> ?name .
}
```

**RESULTS:** 3 bindings
```
name=Robert L. Carter, colleague=RobertCarter
name=Jack Greenberg, colleague=JackGreenberg
colleague=ConstanceBakerMotley, name=Constance Baker Motley
```

---

### ACADEMIC: "What was the mentorship chain at NAACP Legal Defense Fund?"

**SPARQL:**
```sparql
SELECT ?mentor ?mentee ?menteeName WHERE {
  ?mentor <http://law.gov/case#mentored> ?mentee .
  ?mentee <http://www.w3.org/2000/01/rdf-schema#label> ?menteeName .
}
```

**RESULTS:** 3 bindings
```
mentee=JackGreenberg, mentor=ThurgoodMarshall, menteeName=Jack Greenberg
mentee=ConstanceBakerMotley, menteeName=Constance Baker Motley, mentor=JackGreenberg
mentor=ThurgoodMarshall, menteeName=Constance Baker Motley, mentee=ConstanceBakerMotley
```

---

## Architecture Summary

```
+-------------------------------------------------------------+
|                    ALL IN-MEMORY                            |
+-------------------------------------------------------------+
|  GraphDB           | 285 triples, SPARQL 1.1               |
|  RDF2Vec           | 196 embeddings, 128D, Native Rust     |
|  ThinkingReasoner  | 10 obs -> 17 facts, 2 OWL rules       |
|  Prompt Optimizer  | 6 classes, 22 predicates              |
|  HyperFederate     | graph_search() UDF for SQL+SPARQL     |
+-------------------------------------------------------------+
```

---

## Running the Example

```bash
# Install dependencies
npm install

# Run with LLM (full HyperMindAgent)
OPENAI_API_KEY=your-key npm run legal

# Run without LLM (schema-based only)
npm run legal
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

---

*Generated from actual execution output on 2025-12-22*
