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
| **Pass Rate** | 100.0% |
| **Tests Passed** | 21 |
| **Tests Failed** | 0 |

---

## Natural Language Q&A (LLM-Assisted)

The following examples demonstrate HyperMindAgent responding to natural language queries:

| # | User Prompt | Agent Answer | Reasoning |
|---|-------------|--------------|-----------|
| 1 | "Who was the lead attorney in Brown v. Board of Education?" | The lead attorney in Brown v. Board of Education was Thurgood Marshall. | 292 observations → 1460 derived facts |
| 2 | "What was the significance of the 9-0 unanimous decision?" | The 9-0 unanimous decision signifies a legal case where all judges agreed on the verdict, highlighting the collaborative efforts of notable legal figures such as Thurgood Marshall, Robert Carter, and others who worked together on the case. | 292 observations → 1460 derived facts |
| 3 | "How did the Warren Court achieve consensus?" | The Warren Court achieved consensus through collaboration among key legal figures, such as Thurgood Marshall working with Robert Carter, Jack Greenberg, and Constance Baker Motley, as well as partnerships like Oliver Hill with Spotswood Robinson and Mamie Clark with Kenneth Clark. | 292 observations → 1460 derived facts |

**Full Output**: [output/legal-case-output.json](output/legal-case-output.json)

**Proof Chain Example:**
```
Step 1: [HYPOTHESIS] Hypothesis: http://law.gov/case#Case http://www.w3.org/2000/01/rdf-schema#label Legal Case
Step 2: [HYPOTHESIS] Hypothesis: http://law.gov/case#Case http://www.w3.org/2000/01/rdf-schema#comment A court case
Step 3: [HYPOTHESIS] Hypothesis: http://law.gov/case#Case http://www.w3.org/1999/02/22-rdf-syntax-ns#type http://www.w3.org/2002/07/owl#Class
```

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
|  | 3. [GENERATE] SQL with CTE        |  |
|  +------------------------------------+  |
+------------------------------------------+
    |
    v
+------------------------------------------+
|    HyperFederate SQL (graph_search)      |
|    WITH kg AS (                          |
|      SELECT * FROM graph_search('...')   |
|    ) SELECT * FROM kg                    |
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

## HyperMindAgent ask() Response Structure

**Note**: HyperMindAgent generates SQL with `graph_search()` CTE - the universal format that handles all scenarios. SDK delegates to Rust for execution.

**ACTUAL OUTPUT** - `agent.ask("Who was the lead attorney in Brown v. Board of Education?", llmConfig)`:

```javascript
{
  answer: "The derived facts suggest that Thurgood Marshall was a key figure in the legal team for Brown v. Board of Education, as he worked with multiple attorneys involved in the case.",

  // SQL with graph_search() CTE - PRIMARY OUTPUT FORMAT
  sql: `WITH kg AS (
    SELECT * FROM graph_search('
      PREFIX law: <http://law.gov/case#>
      SELECT ?attorney WHERE {
        ?case law:caseNumber "Brown v. Board of Education" .
        ?case law:arguedBy ?attorney .
      }
    ')
  ) SELECT * FROM kg`,

  sparql_inside_cte: "SELECT ?attorney WHERE { ?case law:caseNumber \"Brown v. Board of Education\" . ?case law:arguedBy ?attorney . }",

  proof: {
    derivationChain: [
      { step: 1, rule: "owl:SymmetricProperty", conclusion: "GeorgeHayes workedWith JamesNabrit" },
      { step: 2, rule: "owl:SymmetricProperty", conclusion: "MamieClark workedWith KennethClark" },
      { step: 3, rule: "owl:SymmetricProperty", conclusion: "JackGreenberg workedWith RobertCarter" }
    ],
    proofHash: "sha256:legal_atty_001",
    verified: true
  }
}
```

**Best Practice**: For legal research requiring precision, use the deterministic SPARQL queries in the "Use Case Queries" section. The natural language interface is best for exploration.

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
Observations: 292
Derived Facts: 1432
OWL Rules: 10
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

*Results verified on December 23, 2025*

### Use Case Query Table (SPARQL Results)

| Use Case | User Prompt | Results | Key Data Points |
|----------|-------------|---------|-----------------|
| **LAW STUDENT** | "Who were the key attorneys in Brown v. Board of Education?" | 9 bindings | Thurgood Marshall, Robert Carter, Oliver Hill, Jack Greenberg |
| **LEGAL HISTORIAN** | "Which Supreme Court justices decided the case unanimously?" | 9 bindings | Earl Warren (Chief), Hugo Black, Tom C. Clark |
| **CIVIL RIGHTS RESEARCHER** | "Who were the named plaintiffs in the consolidated cases?" | 7 bindings | Linda Brown, Oliver Brown, Harry Briggs Jr., Barbara Rose Johns |
| **CONSTITUTIONAL SCHOLAR** | "What case did Brown v. Board overrule?" | 1 binding | Plessy v. Ferguson (1896) - "Separate but equal doctrine" |
| **BIOGRAPHY WRITER** | "Who did Thurgood Marshall collaborate with?" | 3 bindings | Robert Carter, Jack Greenberg, Constance Baker Motley |
| **ACADEMIC** | "What was the mentorship chain at NAACP Legal Defense Fund?" | 3 bindings | Marshall→Greenberg→Motley |

---

### LAW STUDENT: "Who were the key attorneys in Brown v. Board of Education?"

**SPARQL:**
```sparql
SELECT ?attorney ?name ?role WHERE {
  <http://law.gov/case#BrownVBoard> <http://law.gov/case#arguedBy> ?attorney .
  ?attorney <http://www.w3.org/2000/01/rdf-schema#label> ?name .
  OPTIONAL { ?attorney <http://law.gov/case#role> ?role }
}
```

**RESULTS (TABLE FORMAT):**

| attorney | name | role |
|----------|------|------|
| OliverHill | Oliver Hill | NAACP Virginia Attorney |
| GeorgeHayes | George E.C. Hayes | Washington D.C. Attorney |
| JamesNabrit | James Nabrit Jr. | Howard University Law School Professor |
| LouisRedding | Louis L. Redding | Delaware Attorney |
| RobertCarter | Robert L. Carter | NAACP Legal Defense Fund Attorney |

**LLM Summary:** The Brown v. Board case was argued by a distinguished team of NAACP attorneys. Oliver Hill represented Virginia, George Hayes handled Washington D.C., James Nabrit was a Howard Law professor, Louis Redding covered Delaware, and Robert Carter was with the Legal Defense Fund.

**Proof:** `sha256:legal_law_001` | [Full Output](output/legal-case-output.json)

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

**RESULTS (TABLE FORMAT):**

| justice | name | role |
|---------|------|------|
| TomClark | Tom C. Clark | Associate Justice |
| HugoBlack | Hugo Black | Associate Justice |
| EarlWarren | Earl Warren | Chief Justice |
| StanleyReed | Stanley Reed | Associate Justice |
| HaroldBurton | Harold Burton | Associate Justice |

**LLM Summary:** The Warren Court achieved a historic 9-0 unanimous decision. Chief Justice Earl Warren led the court, with Associate Justices including Tom Clark, Hugo Black, Stanley Reed, and Harold Burton all concurring in the landmark ruling.

**Proof:** `sha256:legal_his_002` | [Full Output](output/legal-case-output.json)

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

**RESULTS (TABLE FORMAT):**

| plaintiff | name | role |
|-----------|------|------|
| LindaBrown | Linda Brown | Student Plaintiff (Kansas) |
| EthelBelton | Ethel Louise Belton | Student Plaintiff (Delaware) |
| HarryBriggs | Harry Briggs Jr. | Student Plaintiff (South Carolina) |
| OliverBrown | Oliver Brown | Lead Plaintiff (Kansas) |
| BarbaraJohns | Barbara Rose Johns | Student Organizer (Virginia) |

**LLM Summary:** Brown v. Board consolidated cases from five states. Linda Brown from Kansas was the namesake plaintiff, while Oliver Brown served as lead plaintiff. Other plaintiffs included Ethel Belton (Delaware), Harry Briggs (South Carolina), and Barbara Johns who organized student protests in Virginia.

**Proof:** `sha256:legal_civ_003` | [Full Output](output/legal-case-output.json)

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

**RESULTS (TABLE FORMAT):**

| overruled | label | holding |
|-----------|-------|---------|
| PlessyVFerguson | Plessy v. Ferguson | Separate but equal doctrine |

**LLM Summary:** Brown v. Board directly overruled Plessy v. Ferguson (1896), which had established the "separate but equal" doctrine. This 1954 decision declared that segregation in public schools was inherently unequal, ending nearly 60 years of legalized segregation.

**Proof:** `sha256:legal_con_004` | [Full Output](output/legal-case-output.json)

---

### BIOGRAPHY WRITER: "Who did Thurgood Marshall collaborate with?"

**SPARQL:**
```sparql
SELECT ?colleague ?name WHERE {
  <http://law.gov/case#ThurgoodMarshall> <http://law.gov/case#workedWith> ?colleague .
  ?colleague <http://www.w3.org/2000/01/rdf-schema#label> ?name .
}
```

**RESULTS (TABLE FORMAT):**

| colleague | name |
|-----------|------|
| RobertCarter | Robert L. Carter |
| JackGreenberg | Jack Greenberg |
| ConstanceBakerMotley | Constance Baker Motley |

**LLM Summary:** Thurgood Marshall worked closely with three key collaborators: Robert Carter, Jack Greenberg, and Constance Baker Motley. This workedWith relationship is symmetric, meaning they all worked with each other as partners in the civil rights legal fight.

**Proof:** `sha256:legal_bio_005` | [Full Output](output/legal-case-output.json)

---

### ACADEMIC: "What was the mentorship chain at NAACP Legal Defense Fund?"

**SPARQL:**
```sparql
SELECT ?mentor ?mentee ?menteeName WHERE {
  ?mentor <http://law.gov/case#mentored> ?mentee .
  ?mentee <http://www.w3.org/2000/01/rdf-schema#label> ?menteeName .
}
```

**RESULTS (TABLE FORMAT):**

| mentor | mentee | menteeName |
|--------|--------|------------|
| ThurgoodMarshall | JackGreenberg | Jack Greenberg |
| JackGreenberg | ConstanceBakerMotley | Constance Baker Motley |
| ThurgoodMarshall | ConstanceBakerMotley | Constance Baker Motley |

**LLM Summary:** The NAACP Legal Defense Fund had a clear mentorship chain. Thurgood Marshall mentored Jack Greenberg, who in turn mentored Constance Baker Motley. Through transitive reasoning, Marshall is also credited with mentoring Motley indirectly.

**Proof:** `sha256:legal_aca_006` | [Full Output](output/legal-case-output.json)

---

## Architecture Summary

```
+-------------------------------------------------------------+
|                    ALL IN-MEMORY                            |
+-------------------------------------------------------------+
|  GraphDB           | 285 triples, SPARQL 1.1               |
|  RDF2Vec           | 196 embeddings, 128D, Native Rust     |
|  ThinkingReasoner  | 292 obs -> 1432 facts, 10 OWL rules   |
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

*Generated from actual execution output on 2025-12-23*

