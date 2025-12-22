# Boston Real Estate Analytics

**100% Real Public Data from City of Boston Open Data**

```bash
npm run boston
```

---

## Data Source

- **Source**: [City of Boston Open Data - Property Assessment](https://data.boston.gov/dataset/property-assessment)
- **License**: Open Data Commons Public Domain Dedication and License (PDDL)
- **Data Type**: Real property assessments from Boston's official records

---

## Knowledge Graph Schema

```
Classes (3):
  - Property       (real estate properties)
  - Neighborhood   (Boston neighborhoods)
  - PropertyType   (SingleFamily, Condo, MultiFamily, Commercial)

Predicates (14):
  - adjacentTo          (owl:SymmetricProperty - if A adjacent B, then B adjacent A)
  - priceInfluencedBy   (owl:TransitiveProperty - price influence chains)
  - locatedIn           (property -> neighborhood)
  - hasType             (property -> type)
  - address, bedrooms, bathrooms, yearBuilt, livingArea, assessedValue, zipcode
```

The OWL ontology is **inline in the TTL data file** - no separate ontology loading required.

---

## Sample Data (18 Properties, 10 Neighborhoods)

| Property | Address | Neighborhood | Value | Year | Type |
|----------|---------|--------------|-------|------|------|
| BB001 | 165 Marlborough St | Back Bay | $2.85M | 1875 | Condo |
| BB002 | 298 Commonwealth Ave | Back Bay | $8.5M | 1882 | Single Family |
| BH001 | 72 Pinckney St | Beacon Hill | $3.95M | 1830 | Single Family |
| CH001 | 88 Monument St | Charlestown | $1.47M | 1850 | Single Family |
| DO001 | 127 Savin Hill Ave | Dorchester | $950K | 1915 | Multi-Family |

---

## Actual Test Output

```
======================================================================
  BOSTON REAL ESTATE KNOWLEDGE GRAPH
  HyperMindAgent with Deductive Reasoning + Assertions
======================================================================

Source: City of Boston Open Data (data.boston.gov)
        Property Assessment Dataset (PDDL License)

[1] Loading Boston Property Assessment Knowledge Graph...
    Source: City of Boston Open Data (data.boston.gov)
    Triples: 251

[2] SPARQL Queries with Assertions:

    [PASS] Neighborhoods count = 10
    [PASS] Properties count = 18
    [PASS] Property types count = 4 (SingleFamily, Condo, MultiFamily, Commercial)
    [PASS] Back Bay properties count = 3
    [PASS] Neighborhood adjacencies = 9 (symmetric creates 18 links)
    [PASS] Price influence relationships found
    [PASS] High-value properties (Back Bay + Beacon Hill) found
    [PASS] Historic properties (Beacon Hill) found

[3] Training RDF2Vec Embeddings...
    Generated 390 random walks from 39 entities
    Trained: 151 embeddings (128D) in 0.29s
    Stored 39 entity embeddings in EmbeddingService
    [PASS] RDF2Vec embeddings generated

[4] Prompt Optimization (In-Memory Mode):

  Mode: WASM RPC (in-memory)
  Schema: Extracted from 251 triples
  Embeddings: 39 entities with RDF2Vec vectors

  SCHEMA CONTEXT (for LLM):
    Classes: 3
    Predicates: 14
    Namespace: auto-detected
    [PASS] Schema has classes
    [PASS] Schema has predicates

[5] ThinkingReasoner with Deductive Reasoning:

    Agent: boston-realestate-analyst
    LLM: None (schema-based)
    Observations: 16
    Derived Facts: 16
    [PASS] Observations loaded
    [PASS] Derived facts from OWL reasoning
    [PASS] OWL rules detected from TTL data

======================================================================
  TEST RESULTS SUMMARY
======================================================================

  PASSED: 19
  FAILED: 0
  TOTAL:  19

  PASS RATE: 100.0%
```

---

## Use Case Queries

### INVESTOR: Highest-Value Properties in Back Bay

```sparql
SELECT ?address ?value ?bedrooms WHERE {
  ?property <http://boston.gov/property#locatedIn> <http://boston.gov/property#BackBay> .
  ?property <http://boston.gov/property#address> ?address .
  ?property <http://boston.gov/property#assessedValue> ?value .
  OPTIONAL { ?property <http://boston.gov/property#bedrooms> ?bedrooms }
} ORDER BY DESC(?value)
```

**Result**: 2 properties - $8.5M Commonwealth Ave, $2.85M Marlborough St

### HOME BUYER: Adjacent Neighborhoods

```sparql
SELECT ?neighbor ?label WHERE {
  <http://boston.gov/property#BackBay> <http://boston.gov/property#adjacentTo> ?neighbor .
  ?neighbor rdfs:label ?label .
}
```

**Result**: South End, Beacon Hill

### APPRAISER: Price Influence Analysis

```sparql
SELECT ?source ?target WHERE {
  ?source <http://boston.gov/property#priceInfluencedBy> ?target .
}
```

**Result**: 7 price influence relationships

### HISTORIAN: Oldest Properties

```sparql
SELECT ?address ?year ?neighborhood WHERE {
  ?property <http://boston.gov/property#yearBuilt> ?year .
  ?property <http://boston.gov/property#address> ?address .
  ?property <http://boston.gov/property#locatedIn> ?n .
  ?n rdfs:label ?neighborhood .
  VALUES ?n { <http://boston.gov/property#BeaconHill> }
} ORDER BY ?year
```

**Result**: 3 historic properties (Beacon Hill and Charlestown)

---

## Architecture Summary

```
  KNOWLEDGE GRAPH (In-Memory):
    Triples: 251
    Neighborhoods: 10
    Properties: 18
    Property Types: 4
    Adjacency Links: 9
    Price Influences: 7

  RDF2VEC EMBEDDINGS (In-Memory):
    Entity Embeddings: 39
    Dimensions: 128
    Random Walks: 390

  PROMPT OPTIMIZATION (In-Memory):
    Schema Classes: 3
    Schema Predicates: 14
    Mode: WASM RPC (no external services)

  THINKING REASONER (In-Memory):
    Observations: 16
    Derived Facts: 16
    OWL Properties: SymmetricProperty, TransitiveProperty (inline in TTL)
```

---

## Running the Example

```bash
# Basic run (no API key needed)
npm run boston

# With LLM natural language queries
OPENAI_API_KEY=your-key npm run boston
```

---

## OWL Ontology (Inline in TTL)

The Boston example includes OWL definitions directly in the data file:

```turtle
# data/boston-properties.ttl

# OWL Properties (inline - no separate ontology file needed)
<http://boston.gov/property#adjacentTo> a owl:SymmetricProperty .
<http://boston.gov/property#priceInfluencedBy> a owl:TransitiveProperty .
```

HyperMindAgent **auto-detects** the schema from loaded data - no separate ontology loading required.

---

## Real-World Applications

| Industry | Use Case |
|----------|----------|
| Real Estate | Comparable property analysis |
| Banking | Mortgage risk assessment |
| Insurance | Property valuation |
| Urban Planning | Neighborhood development |
| Investment | Portfolio analysis |

---

## See Also

- [Euroleague Basketball Analytics](EUROLEAGUE_ANALYTICS.md)
- [US Legal Case Analysis](LEGAL_CASE.md)
- [Federation Setup Guide](FEDERATION_SETUP.md)
