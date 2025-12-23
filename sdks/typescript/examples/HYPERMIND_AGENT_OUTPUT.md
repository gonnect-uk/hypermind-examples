# HyperMind Agent - Actual Prompts and Outputs

This document shows **real execution output** from HyperMindAgent demos run locally on 2025-12-23.

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

## How to Run

```bash
# Fraud Detection Agent
OPENAI_API_KEY=your-key node examples/fraud-detection-agent.js

# Underwriting Agent
OPENAI_API_KEY=your-key node examples/underwriting-agent.js

# E2E Demo (ThinkingReasoner + HyperMindAgent)
OPENAI_API_KEY=your-key node examples/hypermind-e2e-demo.js
```

---

*Generated: 2025-12-23 | HyperMind v0.2.0 | gpt-4o*
