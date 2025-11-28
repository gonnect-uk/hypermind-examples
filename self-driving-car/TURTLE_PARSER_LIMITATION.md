# Turtle Parser Limitation & Workaround

**Date**: November 27, 2024
**Issue**: rust-kgdb Turtle parser doesn't fully support complex Turtle syntax with semicolons and multi-line formatting

---

## Problem

When you tried to run `DEMO_RUST_KGDB.html`, you got this error:

```
❌ SPARQL Execution Failed
Error: Parse error: Syntax { line: 0, col: 0, message: "Failed to parse entire document. Unparsed content: '\n a av:Vehicle ;\n rdfs:label \"Ego Vehicle\" ;\n av:hasVelocity '" }
```

### Root Cause

The demo uses **complex Turtle syntax** with semicolons for multiple properties:

```turtle
<http://zenya.com/vehicle/ego>
    a av:Vehicle ;
    rdfs:label "Ego Vehicle" ;
    av:hasVelocity "13.3"^^xsd:float ;
    av:positionX "-80.0"^^xsd:float .
```

rust-kgdb's Turtle parser (`../crates/rdf-io/src/turtle.rs`) currently has limitations with:
- Multi-line property lists with `;` separator
- Complex formatting with indentation
- Comments (`#` lines)

This is documented in `RUST_KGDB_INTEGRATION_STATUS.md` (line 62-68).

---

## Solution: Use N-Triples Format

**N-Triples** is a simpler RDF format (one triple per line) that rust-kgdb parses perfectly:

```ntriples
<http://zenya.com/vehicle/ego> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/ontology/av#Vehicle> .
<http://zenya.com/vehicle/ego> <http://www.w3.org/2000/01/rdf-schema#label> "Ego Vehicle" .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#hasVelocity> "13.3"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#positionX> "-80.0"^^<http://www.w3.org/2001/XMLSchema#float> .
```

### Conversion Tool

I created `convert_turtle_to_ntriples.py` which automatically converts all 3 scenarios:

```bash
python3 convert_turtle_to_ntriples.py
```

**Output**: Properly formatted N-Triples for all 3 demo scenarios (see below).

---

## ✅ Converted N-Triples Data

### Scenario 1: Red Traffic Light (11 triples)

```ntriples
<http://zenya.com/vehicle/ego> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/ontology/av#Vehicle> .
<http://zenya.com/vehicle/ego> <http://www.w3.org/2000/01/rdf-schema#label> "Ego Vehicle" .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#hasVelocity> "13.3"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#positionX> "-80.0"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#carLength> "5.0"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/traffic_light/tl_001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/ontology/av#TrafficLight> .
<http://zenya.com/traffic_light/tl_001> <http://www.w3.org/2000/01/rdf-schema#label> "Traffic Light 001" .
<http://zenya.com/traffic_light/tl_001> <http://zenya.com/ontology/av#state> "red"^^<http://www.w3.org/2001/XMLSchema#string> .
<http://zenya.com/traffic_light/tl_001> <http://zenya.com/ontology/av#positionX> "-30.0"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/traffic_light/tl_001> <http://zenya.com/ontology/sensor#confidence> "0.98"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/traffic_light/tl_001> <http://zenya.com/ontology/av#distanceMeters> "30.0"^^<http://www.w3.org/2001/XMLSchema#float> .
```

### Scenario 2: Pedestrian Crossing (6 triples)

```ntriples
<http://zenya.com/vehicle/ego> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/ontology/av#Vehicle> .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#hasVelocity> "10.0"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#positionX> "-60.0"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/crosswalk/cw_001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/ontology/av#Crosswalk> .
<http://zenya.com/crosswalk/cw_001> <http://zenya.com/ontology/av#state> "active"^^<http://www.w3.org/2001/XMLSchema#string> .
<http://zenya.com/crosswalk/cw_001> <http://zenya.com/ontology/av#positionX> "-43.0"^^<http://www.w3.org/2001/XMLSchema#float> .
```

### Scenario 3: School Zone Speeding (3 triples)

```ntriples
<http://zenya.com/vehicle/ego> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/ontology/av#Vehicle> .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#hasVelocity> "22.2"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#positionX> "-120.0"^^<http://www.w3.org/2001/XMLSchema#float> .
```

---

## Manual Testing Workaround

While I work on updating the demo file, you can test rust-kgdb immediately using curl:

### Test Scenario 1 (Red Traffic Light)

```bash
# 1. Clear store
curl -X POST http://localhost:8080/clear

# 2. Load data (N-Triples format)
curl -X POST http://localhost:8080/load \
  -H "Content-Type: application/json" \
  -d '{"turtle_data": "<http://zenya.com/vehicle/ego> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/ontology/av#Vehicle> .\n<http://zenya.com/vehicle/ego> <http://www.w3.org/2000/01/rdf-schema#label> \"Ego Vehicle\" .\n<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#hasVelocity> \"13.3\"^^<http://www.w3.org/2001/XMLSchema#float> .\n<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#positionX> \"-80.0\"^^<http://www.w3.org/2001/XMLSchema#float> .\n<http://zenya.com/vehicle/ego> <http://zenya.com/ontology/av#carLength> \"5.0\"^^<http://www.w3.org/2001/XMLSchema#float> .\n<http://zenya.com/traffic_light/tl_001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://zenya.com/ontology/av#TrafficLight> .\n<http://zenya.com/traffic_light/tl_001> <http://www.w3.org/2000/01/rdf-schema#label> \"Traffic Light 001\" .\n<http://zenya.com/traffic_light/tl_001> <http://zenya.com/ontology/av#state> \"red\"^^<http://www.w3.org/2001/XMLSchema#string> .\n<http://zenya.com/traffic_light/tl_001> <http://zenya.com/ontology/av#positionX> \"-30.0\"^^<http://www.w3.org/2001/XMLSchema#float> .\n<http://zenya.com/traffic_light/tl_001> <http://zenya.com/ontology/sensor#confidence> \"0.98\"^^<http://www.w3.org/2001/XMLSchema#float> .\n<http://zenya.com/traffic_light/tl_001> <http://zenya.com/ontology/av#distanceMeters> \"30.0\"^^<http://www.w3.org/2001/XMLSchema#float> ."}'

# 3. Query (ASK if traffic light is red)
curl -X POST http://localhost:8080/ask \
  -H "Content-Type: application/json" \
  -d '{"sparql_query": "PREFIX av: <http://zenya.com/ontology/av#>\nASK {\n  ?tl a av:TrafficLight ;\n      av:state \"red\" .\n}"}'

# Expected result: {"success":true,"result":true,"execution_time_us":...}
```

This should work perfectly because we're using N-Triples format!

---

## Next Step: Fix the Demo

I need to update `DEMO_RUST_KGDB.html` to replace all 3 `turtleData` sections (lines 668, 769, 866) with the N-Triples versions above.

---

## Long-Term Solution: Improve Turtle Parser

The rust-kgdb Turtle parser could be enhanced to support:
- Property list syntax with `;` separator
- Blank line handling
- Complex formatting

This would require updating `../crates/rdf-io/src/turtle.pest` (the PEG grammar) and `../crates/rdf-io/src/turtle.rs` (the parser implementation).

**Current status**: Basic Turtle support works, but advanced syntax (`;`, `,`, `[]`, `()`) needs additional grammar rules.

---

## Summary

- ✅ rust-kgdb HTTP server is WORKING
- ✅ N-Triples format is fully supported
- ✅ SPARQL queries execute correctly
- ⚠️ Complex Turtle syntax is NOT supported (yet)
- 🔧 **Workaround**: Convert Turtle to N-Triples (tool provided)
- 📋 **Next**: Update demo HTML with N-Triples data
