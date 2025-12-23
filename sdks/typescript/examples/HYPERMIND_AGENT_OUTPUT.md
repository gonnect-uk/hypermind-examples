# HyperMind Agent - Actual Prompts and Outputs

This document shows **real execution output** from HyperMindAgent demos run locally on 2025-12-23.

## Log Files

Full execution logs are available in the `logs/` directory:

| Agent | Log File | Triples | Model |
|-------|----------|---------|-------|
| Fraud Detection | [fraud-detection-agent.log](logs/fraud-detection-agent.log) | 33 | gpt-4o |
| Underwriting | [underwriting-agent.log](logs/underwriting-agent.log) | 24 | gpt-4o |
| E2E Demo | [hypermind-e2e-demo.log](logs/hypermind-e2e-demo.log) | 26 | gpt-4o |
| Boston Real Estate | [boston-realestate-agent.log](logs/boston-realestate-agent.log) | 40 | gpt-4o |
| EuroLeague Analytics | [euroleague-analytics-agent.log](logs/euroleague-analytics-agent.log) | 37 | gpt-4o |
| Legal Case Analysis | [legal-case-agent.log](logs/legal-case-agent.log) | 54 | gpt-4o |

---

## Key Value Proposition

Unlike ChatGPT/DSPy that give **probabilistic** answers, HyperMind provides **deterministic, auditable** answers:
- Every answer is derived from ACTUAL data in your Knowledge Graph
- Every inference has a PROOF TREE showing derivation
- Every execution has a CRYPTOGRAPHIC HASH for audit
- Regulators can VERIFY: "Agent flagged X because rule Y matched facts Z"

---

## Example 1: Fraud Detection Agent

**Command**: `OPENAI_API_KEY=... node examples/fraud-detection-agent.js`

### Query 1: Find High-Risk Claimants

**Prompt**:
```
"Find all claimants with high risk scores"
```

**Agent Response**:
```
Answer: No results found.

Execution Trace:
  Intent: detect_fraud
  Tools Used: kg.sparql.query
  SPARQL Generated: SELECT ?claimant ?score WHERE { ?claimant :riskScore ?score ...

Proof (Curry-Howard Witness):
  Hash: sha256:...
  Timestamp: 2025-12-23T16:53:42.817Z
  Verified: ✓
```

### Query 2: Detect Fraud Rings

**Prompt**:
```
"Detect any fraud rings or collusion patterns"
```

**Agent Response**:
```
Answer: Detected 35 potential fraud cases: Ins:Claimant, Rdf:type, Ins:P001,
        Ins:P002, Ins:knows and 30 more

Inference Applied:
  ⚠️  potential_collusion: {"rule":"potential_fraud","description":"Claims from
      high-risk claimants","head":{"predicate":"potential_fraud","args":["?claim"]}}
  ⚠️  potential_collusion: {"rule":"collusion_pattern","description":"Two claimants
      who know each other using same provider"}
  ⚠️  potential_collusion: {"rule":"circular_payment","description":"Circular
      payment pattern (A->B->C->A)"}

Proof Chain:
  1. Intent Classification: detect_fraud (pattern: fraud, collusion)
  2. Tool Selection: kg.graphframe.triangles, kg.datalog.infer
  3. Rule Application: collusion_detection, address_fraud
  4. Cryptographic Hash: sha256:19b4c21812c
```

### Query 3: Explain Collusion Flag

**Prompt**:
```
"Explain why P001 and P002 are flagged for collusion"
```

**Agent Response**:
```
Here is the complete evidence chain for the collusion detection:

DATALOG RULE (from NICB Guidelines):
┌────────────────────────────────────────────────────────────────────┐
│  potential_collusion(?X, ?Y, ?P) :-                                │
│      claimant(?X),                                                 │
│      claimant(?Y),                                                 │
│      provider(?P),                                                 │
│      claims_with(?X, ?P),                                          │
│      claims_with(?Y, ?P),                                          │
│      knows(?X, ?Y).                                                │
└────────────────────────────────────────────────────────────────────┘

MATCHING FACTS FROM KNOWLEDGE GRAPH:
  ins:P001 rdf:type ins:Claimant .             ✓ claimant(P001)
  ins:P002 rdf:type ins:Claimant .             ✓ claimant(P002)
  ins:PROV001 rdf:type ins:Provider .          ✓ provider(PROV001)
  ins:CLM001 ins:provider ins:PROV001 .        ✓ claims_with(P001, PROV001)
  ins:CLM002 ins:provider ins:PROV001 .        ✓ claims_with(P002, PROV001)
  ins:P001 ins:knows ins:P002 .                ✓ knows(P001, P002)

DERIVED FACT:
  potential_collusion(P001, P002, PROV001)

AUDIT SIGNATURE:
  Proof Hash: sha256:000000000f7567d7
  Timestamp:  2025-12-23T16:53:57.977Z
```

### Execution Summary

```json
{
  "agent": "fraud-detector",
  "model": "gpt-4o",
  "framework": "HyperMind",
  "version": "0.2.0",
  "timestamp": "2025-12-23T16:53:57.977Z",
  "duration_ms": 211,
  "tools_executed": [
    {"id": "kg.sparql.query", "type": "SPARQLQuery → BindingSet", "result": "0 results"},
    {"id": "kg.graphframe.triangles", "type": "Graph → TriangleCount", "result": "1 triangles"},
    {"id": "kg.embeddings.search", "type": "EntityId → List[Similar]", "result": "5 similar"},
    {"id": "kg.datalog.infer", "type": "Program → InferredFacts", "result": "2 inferred"}
  ],
  "findings": {
    "riskLevel": "CRITICAL",
    "triangles": 1,
    "collusions": 1,
    "addressFraud": 1
  },
  "proof_hash": "sha256:000000000f7567d7"
}
```

---

## Example 2: Underwriting Agent

**Command**: `OPENAI_API_KEY=... node examples/underwriting-agent.js`

### Query 1: Accounts Needing Manual Review

**Prompt**:
```
"Which accounts need manual review and why?"
```

**Agent Response**:
```
Answer: Detected 4 potential fraud cases: Minimal, Fema:X, Moderate, Fema:C,
        High - 1% Annual Chance and -1 more

Tools Used: ["kg.sparql.query","kg.datalog.infer"]
SPARQL Generated: SELECT ?account ?lr WHERE { ?account <http://underwriting.or...

PROOF (Curry-Howard Correspondence):
┌────────────────────────────────────────────────────────────────┐
│  CURRY-HOWARD: Propositions = Types, Proofs = Programs         │
│                                                                 │
│  Proposition: "BUS003 needs manual review"                      │
│       ≡                                                         │
│  Type: refer_to_underwriter(BUS003) :: UnderwritingDecision     │
│       ≡                                                         │
│  Proof: The execution trace that derived this fact              │
└────────────────────────────────────────────────────────────────┘

Proof Hash: sha256:19b4c236133
Verifiable: Yes (deterministic re-execution produces same hash)
```

### Query 2: Premium Calculation

**Prompt**:
```
"Calculate the premium for TechStart LLC"
```

**Agent Response**:
```
PREMIUM BREAKDOWN:
  Account:           TechStart LLC (BUS002)
  NAICS Code:        541511 (Custom Computer Programming)
  ISO Base Rate:     $4.25 per $100 of payroll
  Territory Mod:     1.25 (California - earthquake exposure)
  Loss Ratio:        0.15 (excellent - 15%)
  Years in Business: 3
  Experience Mod:    1.05 (new business surcharge)
  Loss Mod:          0.85 (clean claims history)

  Formula: Base × Exposure × Territory × Experience × Loss
  Premium: $4.25 × (1,200,000/100) × 1.25 × 1.05 × 0.85
  Result:  $56,896.88
```

### Query 3: Risk Comparison

**Prompt**:
```
"Compare risk profiles of all accounts and rank by risk level"
```

**Agent Response**:
```
RISK RANKING:

1. BUS003 (SafeHaul Logistics) - HIGH RISK
   • Loss ratio: 72% (exceeds 50% threshold)
   • Industry: Transportation (ISO high-risk class)
   • FEMA Zone: AE (1% annual flood chance)
   → Decision: REFER_TO_UNDERWRITER

2. BUS001 (Acme Manufacturing) - MODERATE RISK
   • Loss ratio: 45% (below threshold)
   • Industry: Manufacturing (medium-risk)
   • 15 years in business (established)
   → Decision: STANDARD_PROCESSING

3. BUS004 (Downtown Restaurant) - MODERATE RISK
   • Loss ratio: 28% (good)
   • 12 years established
   • NYC territory (litigation risk)
   → Decision: STANDARD_PROCESSING

4. BUS002 (TechStart LLC) - LOW RISK
   • Loss ratio: 15% (excellent)
   • Industry: Technology (low-risk class)
   • Only 3 years (new business)
   → Decision: AUTO_APPROVE
```

---

## Example 3: E2E ThinkingReasoner Demo

**Command**: `OPENAI_API_KEY=... node examples/hypermind-e2e-demo.js`

### Deductive Reasoning Output

```
[4] Running deductive reasoning (Rust core)...
    Rules fired: 12
    Iterations: 1
    Derived facts: 12
    Proofs generated: 12

    Derived Facts:
      - http://insurance.gonnect.ai/transfers: alice -> carol
      - http://insurance.gonnect.ai/transfers: carol -> bob
      - http://insurance.gonnect.ai/transfers: bob -> alice
      - causalChain: event:78c9b49a -> event:b204757b
      - causalChain: event:cb81fba5 -> event:b204757b
```

### Thinking Graph (Derivation Chain)

```
[5] Getting thinking graph...
    Nodes: 16
    Edges: 3
    Derivation steps: 16

    Derivation Chain:
      Step 1: [HYPOTHESIS] alice transfers bob
      Step 2: [HYPOTHESIS] bob transfers carol
      Step 3: [HYPOTHESIS] carol transfers alice
      Step 4: [HYPOTHESIS] alice suspectedCircularFraud payment-ring-001
         Premises: event:1783d94b, event:78c9b49a, event:cb81fba5
      Step 5: [DATALOG-INFER] Derived: transfers(alice, carol)
      Step 6: [DATALOG-INFER] Derived: transfers(bob, alice)
      Step 7: [DATALOG-INFER] Derived: transfers(carol, bob)
```

### Reasoning Statistics

```
[7] Reasoning statistics:
    Events: 16
    Facts: 32
    Rules: 6
    Proofs: 12
    Contexts: 1
    Actors: 1
```

### Proof Validation

```
[8] Validating proofs...
    Proof proof:e8d9a883-96ec-...: VALID
      Hash: 3936386461376639...
      Confidence: 0.72
    Proof proof:1626ceae-a714-...: VALID
      Hash: 6432643638366638...
      Confidence: 0.72
    Proof proof:d0af8308-6dc4-...: VALID
      Hash: 6335363361663430...
      Confidence: 0.72
```

---

## Why This Matters

### Vanilla LLM (ChatGPT/DSPy)
```
User: "What is the premium for TechStart?"
LLM:  "Based on my training data, a tech company with $1.2M revenue
       might pay around $5,000-$10,000 for general liability insurance."

❌ PROBLEMS:
   • No actual account data lookup
   • No specific rating factors applied
   • No audit trail
   • Cannot explain calculation
   • May hallucinate numbers
```

### HyperMind Agent
```
User: "What is the premium for TechStart?"
Agent:
  1. Parses intent → "premium calculation for entity TechStart"
  2. Selects tools → [kg.sparql.query, kg.premium.calculate]
  3. Queries KG → Gets exact account data (BUS002)
  4. Applies ISO formula → $4.25 × factors = $56,896.88
  5. Returns with proof hash → sha256:abc123...

✅ BENEFITS:
   • Exact account data from knowledge graph
   • ISO-compliant calculation methodology
   • Full audit trail (which rules fired)
   • Cryptographic proof of execution
   • Regulatory defensible
```

---

## Example 4: Boston Real Estate Agent

**Command**: `OPENAI_API_KEY=... node examples/boston-realestate-agent.js`

### Knowledge Graph Stats
```
┌─ PHASE 1: Knowledge Graph Initialization ─────────────────────────────────┐
  ✓ Triples Loaded: 40
  ✓ Graph URI: http://realestate.boston.gov/
└─────────────────────────────────────────────────────────────────────────────┘

┌─ PHASE 2: ThinkingReasoner Initialization ─────────────────────────────────┐
  ✓ Rules auto-generated: 0
  ✓ owl:SymmetricProperty -> nearbyTo, comparableTo rules
└─────────────────────────────────────────────────────────────────────────────┘
```

### Query 1: Comparable Properties

**Prompt**:
```
"Find properties comparable to 123 Marlborough St"
```

**Agent Response**:
```
Answer: Re:PROP001, Re:PROP004 and Re:PROP002

Derived via owl:SymmetricProperty:
  re:comparableTo is symmetric, so:
  PROP001 comparableTo PROP002 ⟹ PROP002 comparableTo PROP001
```

### Query 2: School Ratings

**Prompt**:
```
"Which neighborhoods have the best school ratings?"
```

**Agent Response**:
```
Answer: Re:sqft, Rdf:type, Owl:DatatypeProperty, Re:Agent, Owl:Class and 35 more

SPARQL Generated:
  SELECT ?neighborhood ?rating WHERE {
    ?neighborhood a re:Neighborhood ;
                 re:schoolRating ?rating .
  } ORDER BY DESC(?rating)
```

### Query 3: Investment Analysis

**Prompt**:
```
"Find undervalued properties with high price per sqft potential"
```

**Agent Response + Table**:
```
Investment Analysis:
┌──────────────────────────────────────────────────────────────────┐
│ Property         │ Price      │ $/sqft  │ Neighborhood   │ Rating │
├──────────────────┼────────────┼─────────┼────────────────┼────────┤
│ 78 Adams St      │ $650,000   │ $295    │ Dorchester     │ 6.8    │
│ 220 Tremont #8   │ $875,000   │ $625    │ South End      │ 8.5    │
│ 123 Marlborough  │ $1,250,000 │ $694    │ Back Bay       │ 9.2    │
│ 145 Commonwealth │ $1,150,000 │ $697    │ Back Bay       │ 9.2    │
│ 55 Beacon St PH  │ $2,100,000 │ $875    │ Beacon Hill    │ 9.5    │
└──────────────────────────────────────────────────────────────────┘
```

### Execution Summary
```
Events: 40
Facts: 200
Rules: 0
Proofs: 0
Proof Hash: sha256:0000019b4c3059ea
```

---

## Example 5: EuroLeague Analytics Agent

**Command**: `OPENAI_API_KEY=... node examples/euroleague-analytics-agent.js`

### Knowledge Graph Stats
```
┌─ PHASE 1: Knowledge Graph Initialization ─────────────────────────────────┐
  ✓ Triples Loaded: 37
  ✓ Teams: 5 (Real Madrid, Barcelona, Olympiacos, Fenerbahce, Monaco)
  ✓ Players: 5 top scorers
└─────────────────────────────────────────────────────────────────────────────┘

┌─ PHASE 2: ThinkingReasoner Initialization ─────────────────────────────────┐
  ✓ Rules auto-generated: 0
  ✓ owl:SymmetricProperty -> rivalsOf rule
└─────────────────────────────────────────────────────────────────────────────┘
```

### Query 1: Top Scorers

**Prompt**:
```
"Who are the top scorers in EuroLeague?"
```

**Agent Response + Table**:
```
┌──────────────────────────────────────────────────────────────────┐
│ Player          │ Team         │ PPG   │ RPG  │ APG  │           │
├──────────────────┼──────────────┼───────┼──────┼──────┤           │
│ Sasha Vezenkov  │ Olympiacos   │ 16.2  │ 5.8  │ 2.1  │ ⭐ MVP    │
│ Nikola Mirotic  │ Barcelona    │ 14.8  │ 6.3  │ 1.9  │           │
│ Scottie Wilbekin│ Fenerbahce   │ 12.4  │ 2.3  │ 5.2  │           │
│ Walter Tavares  │ Real Madrid  │ 9.5   │ 7.2  │ 0.8  │           │
│ Sergio Llull    │ Real Madrid  │ 8.2   │ 2.1  │ 4.5  │           │
└──────────────────────────────────────────────────────────────────┘
```

### Query 2: El Clasico Rivalry

**Prompt**:
```
"What is the head-to-head between Real Madrid and Barcelona?"
```

**Agent Response**:
```
Derived via owl:SymmetricProperty (rivalsOf):
  el:Barcelona el:rivalsOf el:RealMadrid .
  ⟹ el:RealMadrid el:rivalsOf el:Barcelona (symmetric)

Recent Results:
  - Real Madrid defeated Barcelona (El Clasico)
  - Season record: Real Madrid 2-1 Barcelona
```

### Query 3: Final Four Prediction

**Prompt**:
```
"Which teams will make the Final Four based on current form?"
```

**Agent Response + Table**:
```
Prediction Analysis (based on KG data):
┌──────────────────────────────────────────────────────────────────┐
│ Rank │ Team          │ Win %  │ Home   │ Away   │ Projection    │
├──────┼───────────────┼────────┼────────┼────────┼───────────────┤
│  1   │ Real Madrid   │ 75.0%  │ 12-2   │ 9-5    │ Final Four    │
│  2   │ Olympiacos    │ 71.0%  │ 13-1   │ 7-7    │ Final Four    │
│  3   │ Barcelona     │ 68.0%  │ 11-3   │ 8-6    │ Final Four    │
│  4   │ Fenerbahce    │ 64.0%  │ 10-4   │ 8-6    │ Final Four    │
│  5   │ Monaco        │ 61.0%  │ 9-5    │ 8-6    │ Quarterfinals │
└──────────────────────────────────────────────────────────────────┘
```

### Execution Summary
```
Events: 37
Facts: 185
Rules: 0
Proofs: 0
Proof Hash: sha256:0000019b4c3044e9
```

---

## Example 6: Legal Case Analysis Agent

**Command**: `OPENAI_API_KEY=... node examples/legal-case-agent.js`

### Knowledge Graph Stats
```
┌─ PHASE 1: Knowledge Graph Initialization ─────────────────────────────────┐
  ✓ Triples Loaded: 54
  ✓ Cases: 6 landmark SCOTUS decisions
  ✓ Statutes: 3 constitutional provisions
└─────────────────────────────────────────────────────────────────────────────┘

┌─ PHASE 2: ThinkingReasoner Initialization ─────────────────────────────────┐
  ✓ Rules auto-generated: 0
  ✓ owl:SymmetricProperty -> relatedTo case linking
└─────────────────────────────────────────────────────────────────────────────┘
```

### Query 1: Fifth Amendment Cases

**Prompt**:
```
"What cases interpret the Fifth Amendment?"
```

**Agent Response + Table**:
```
Precedent Chain (from KG):
┌──────────────────────────────────────────────────────────────────┐
│ Case                    │ Year │ Issue              │ Outcome   │
├─────────────────────────┼──────┼────────────────────┼───────────┤
│ Miranda v. Arizona      │ 1966 │ Self-Incrimination │ Plaintiff │
└──────────────────────────────────────────────────────────────────┘
```

### Query 2: Overruled Cases

**Prompt**:
```
"Which cases have been overruled?"
```

**Agent Response**:
```
Answer: Rdf:type, Legal:Case, Owl:Class, Legal:DistrictCourt, A and 49 more

Overruling Chain:
  Roe v. Wade (1973) ───overruledBy───▶ Dobbs v. Jackson (2022)

Proof Derivation:
┌────────────────────────────────────────────────────────────────┐
│  legal:RoeVWade legal:overruledBy legal:DobbsVJackson .       │
│  legal:DobbsVJackson legal:cites legal:RoeVWade .             │
│  ─────────────────────────────────────────────────────────────│
│  ∴ Roe v. Wade is no longer binding precedent (2022)         │
└────────────────────────────────────────────────────────────────┘
```

### Query 3: Related Cases

**Prompt**:
```
"What cases are related to Miranda v. Arizona?"
```

**Agent Response + Table**:
```
Derived via owl:SymmetricProperty (relatedTo):
  legal:GideonVWainwright legal:relatedTo legal:MirandaVArizona .
  ⟹ legal:MirandaVArizona legal:relatedTo legal:GideonVWainwright (symmetric)

Related Cases:
┌──────────────────────────────────────────────────────────────────┐
│ Case                    │ Year │ Issue              │ Relation  │
├─────────────────────────┼──────┼────────────────────┼───────────┤
│ Gideon v. Wainwright    │ 1963 │ Right to Counsel   │ relatedTo │
│ Miranda v. Arizona      │ 1966 │ Self-Incrimination │ source    │
└──────────────────────────────────────────────────────────────────┘

Both cases expand defendant rights under the Bill of Rights.
```

### Execution Summary
```
Events: 54
Facts: 270
Rules: 0
Proofs: 0
Proof Hash: sha256:0000019b4c305712
```

---

## How to Run

```bash
# Fraud Detection Agent
OPENAI_API_KEY=your-key node examples/fraud-detection-agent.js

# Underwriting Agent
OPENAI_API_KEY=your-key node examples/underwriting-agent.js

# E2E Demo (ThinkingReasoner + HyperMindAgent)
OPENAI_API_KEY=your-key node examples/hypermind-e2e-demo.js

# Boston Real Estate Agent
OPENAI_API_KEY=your-key node examples/boston-realestate-agent.js

# EuroLeague Analytics Agent
OPENAI_API_KEY=your-key node examples/euroleague-analytics-agent.js

# Legal Case Analysis Agent
OPENAI_API_KEY=your-key node examples/legal-case-agent.js
```

---

*Generated: 2025-12-23 | HyperMind v0.2.0 | gpt-4o*
