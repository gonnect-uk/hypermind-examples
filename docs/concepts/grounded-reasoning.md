# Grounded Reasoning: How ThinkingReasoner Works

HyperMindAgent produces **explainable AI** by combining LLM planning with deductive reasoning. Every conclusion traces back to ground truth observations.

## The Problem with Vanilla LLMs

```
User: "Who argued Brown v. Board of Education?"
Vanilla LLM: "The case was argued by... [hallucination risk]"
```

LLMs can confidently generate incorrect information because they have no access to ground truth data.

## The HyperMind Solution

```
User: "Who argued Brown v. Board of Education?"
HyperMind:
  1. Extract schema from knowledge graph
  2. Generate valid SPARQL query
  3. Execute against real data
  4. Apply OWL reasoning rules
  5. Return answer with proof
```

**Result:** Every fact is traceable to source observations.

---

## How Deductive Reasoning Works

### Step 1: Load Observations

ThinkingReasoner loads facts from your knowledge graph as **observations**:

```
[OBS] ThurgoodMarshall workedWith RobertCarter
[OBS] ThurgoodMarshall workedWith JackGreenberg
[OBS] ThurgoodMarshall mentored JackGreenberg
```

### Step 2: Apply OWL Rules

OWL properties are **auto-detected from your TTL data** (no separate ontology file needed):

```turtle
# In your TTL data file
@prefix owl: <http://www.w3.org/2002/07/owl#> .
ex:workedWith a owl:SymmetricProperty .
ex:mentored a owl:TransitiveProperty .
```

ThinkingReasoner applies these rules:

| OWL Property | Rule Applied |
|--------------|--------------|
| `SymmetricProperty` | If Marshall workedWith Carter, then Carter workedWith Marshall |
| `TransitiveProperty` | If Houston mentored Marshall, and Marshall mentored Greenberg, then Houston mentored Greenberg |

### Step 3: Derive New Facts

```
[DERIVED] RobertCarter workedWith ThurgoodMarshall  (via SymmetricProperty)
[DERIVED] JackGreenberg workedWith ThurgoodMarshall (via SymmetricProperty)
```

### Step 4: Build Derivation Chain

The derivation chain is a **proof** that shows how each conclusion was reached:

```
DERIVATION CHAIN:
  Step 1: [OBSERVATION] ThurgoodMarshall workedWith RobertCarter
  Step 2: [RULE] SymmetricProperty applied
  Step 3: [DERIVED] RobertCarter workedWith ThurgoodMarshall (usedSteps: [1, 2])
```

### Step 5: Generate Cryptographic Proof

```javascript
proof: {
  hash: "sha256:92be3c44...",  // Hash of derivation chain
  verified: true,              // Chain integrity verified
  tripleCount: 3               // Number of source triples used
}
```

---

## No Ontology Files Required

**All OWL definitions are inline in your TTL data files:**

```turtle
# boston-properties.ttl - OWL properties inline

@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix prop: <http://boston.gov/property#> .

# OWL property definitions (inline)
prop:adjacentTo a owl:SymmetricProperty .
prop:priceInfluencedBy a owl:TransitiveProperty .

# Data
prop:BackBay prop:adjacentTo prop:SouthEnd .
prop:BB001 prop:priceInfluencedBy prop:BB002 .
```

HyperMindAgent **auto-detects** OWL properties from loaded data:
- No separate ontology loading
- No `loadOntology()` calls
- Schema extracted at `loadTtl()` time

---

## Actual Output Example

From `npm run legal`:

```
[6] Thinking Graph (Derivation Chain / Proofs):

  EVIDENCE NODES (first 8):
    [OBS] KennethClark workedWith MamieClark
    [OBS] SpotswoodRobinson workedWith OliverHill
    [OBS] JamesNabrit workedWith GeorgeHayes
    [OBS] ThurgoodMarshall workedWith RobertCarter
    [OBS] RobertCarter workedWith JackGreenberg
    [OBS] ThurgoodMarshall workedWith JackGreenberg
    [OBS] ThurgoodMarshall workedWith ConstanceBakerMotley
    [OBS] ThurgoodMarshall mentored JackGreenberg

  DERIVATION CHAIN (Proof Steps):
    Step 1: [OBSERVATION] KennethClark workedWith MamieClark
    Step 2: [OBSERVATION] SpotswoodRobinson workedWith OliverHill
    Step 3: [OBSERVATION] JamesNabrit workedWith GeorgeHayes
    Step 4: [OBSERVATION] ThurgoodMarshall workedWith RobertCarter
    Step 5: [OBSERVATION] RobertCarter workedWith JackGreenberg
    Step 6: [OBSERVATION] ThurgoodMarshall workedWith JackGreenberg
    Step 7: [OBSERVATION] ThurgoodMarshall workedWith ConstanceBakerMotley
    Step 8: [OBSERVATION] ThurgoodMarshall mentored JackGreenberg

  DEDUCTIVE REASONING VALUE:
    - Every conclusion traces back to ground truth observations
    - SymmetricProperty: If Marshall workedWith Carter, then Carter workedWith Marshall
    - TransitiveProperty: If Marshall mentored Greenberg, Greenberg mentored Motley,
                          then Marshall mentored Motley (transitive closure)
    - No hallucinations - only provable facts with derivation chains
```

---

## Why This Matters

| Vanilla LLM | HyperMind |
|-------------|-----------|
| May hallucinate | Every fact traceable |
| No proof | Cryptographic audit trail |
| Black box | Explainable reasoning |
| Context limits | Knowledge graph scales |

**Result:** 86.4% accuracy vs 0% for vanilla LLMs on LUBM benchmark.

---

## See Also

- [HyperMindAgent API](../api/hypermind-agent.md)
- [OWL Properties](owl-properties.md)
- [Derivation Chains](derivation-chains.md)
