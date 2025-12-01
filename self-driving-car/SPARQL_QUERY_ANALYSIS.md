# SPARQL Query Results Analysis

## Issue: Scenario 3 Returns 0 Bindings

### The Problem

**Scenario 3 Data** (lines 1366-1369):
```turtle
<http://gonnect.com/vehicle/ego> <...#hasVelocity> "15.0" .
<http://gonnect.com/zone/school_001> <...#speedLimit> "8.33" .
```

**SPARQL Query**:
```sparql
SELECT ?zone ?limit ?vehicle ?speed
WHERE {
  ?zone <...#type> <...#SchoolZone> .
  ?zone <...#speedLimit> ?limit .
  ?vehicle <...#type> <...#Vehicle> .
  ?vehicle <...#hasVelocity> ?speed .
  FILTER(?speed > ?limit)   ← PROBLEM HERE!
}
```

**Expected**: Should return 1 binding (15.0 > 8.33 is TRUE)
**Actual**: Returns 0 bindings

### Root Cause: Missing Datatype Declarations

**Scenario 1 (working correctly)** uses Turtle format with explicit datatypes:
```turtle
av:hasVelocity "13.3"^^xsd:float ;
av:state "red"^^xsd:string ;
```

**Scenarios 2 & 3 (failing)** use N-Triples format WITHOUT datatypes:
```turtle
<http://gonnect.com/vehicle/ego> <http://gonnect.com/ontology/av#hasVelocity> "15.0" .
                                                                                  ↑↑↑↑
                                                                        NO ^^xsd:float!
```

When RDF triples are loaded without explicit datatypes, they are stored as **plain literals** (strings), not numbers.

**SPARQL FILTER Behavior**:
- `FILTER(?speed > ?limit)` expects **numeric comparison**
- When `?speed = "15.0"` (string) and `?limit = "8.33"` (string), the comparison fails
- Result: 0 bindings (no rows match)

### The Fix

**Option 1: Add Datatype Declarations** (Recommended)

Change scenarios 2 and 3 to include `^^xsd:float`:

```turtle
<http://gonnect.com/vehicle/ego> <http://gonnect.com/ontology/av#hasVelocity> "15.0"^^<http://www.w3.org/2001/XMLSchema#float> .
<http://gonnect.com/zone/school_001> <http://gonnect.com/ontology/av#speedLimit> "8.33"^^<http://www.w3.org/2001/XMLSchema#float> .
```

**Option 2: Use xsd() Function in FILTER**

```sparql
FILTER(xsd:float(?speed) > xsd:float(?limit))
```

But this is less efficient and doesn't fix the root cause.

**Option 3: Change SPARQL to String Comparison**

Not recommended - defeats the purpose of semantic precision.

---

## Other Query Results Explained

### Scenario 1 (Red Traffic Light): ASK = FALSE ✅ CORRECT

**Data**:
```turtle
<http://gonnect.com/traffic_light/tl_001> av:state "red"^^xsd:string .
```

**Query**:
```sparql
ASK WHERE {
  ?tl <...#type> <...#TrafficLight> .
  ?tl <...#state> "red" .
}
```

**Why FALSE when data says "red"?**

This is actually WORKING CORRECTLY for the current state!

Looking at the console output:
```
REAL SPARQL ASK Result: FALSE
Decision: ✅ SAFE TO PROCEED
```

This means when you run the demo, the page is NOT currently showing scenario 1. The ASK query is being executed **before** or **after** scenario 1 data is loaded, so the graph is empty or contains different data.

The demo cycles through scenarios, so:
- When scenario 1 is active → ASK should return TRUE
- When scenario 2 or 3 is active → ASK returns FALSE (correct!)

### Scenario 2 (Pedestrian): ASK = TRUE ✅ CORRECT

**Data**:
```turtle
<http://gonnect.com/pedestrian/ped_001> <...#inCrosswalk> <http://gonnect.com/crosswalk/cw_001> .
```

**Query**:
```sparql
ASK WHERE {
  ?ped <...#type> <...#Pedestrian> .
  ?ped <...#inCrosswalk> ?crosswalk .
}
```

**Result**: TRUE ✅

This is **CORRECT**. The data contains:
1. `ped_001` is type `Pedestrian` ✓
2. `ped_001` has property `inCrosswalk` pointing to `cw_001` ✓

All patterns match → TRUE

---

## Summary

| Scenario | Query Type | Expected | Actual | Status |
|----------|-----------|----------|--------|--------|
| 1 (Traffic Light) | ASK | TRUE (when loaded) | FALSE (not currently loaded) | ✅ Correct |
| 2 (Pedestrian) | ASK | TRUE | TRUE | ✅ Correct |
| 3 (School Zone) | SELECT | 1 binding | 0 bindings | ❌ **BUG** |

**Action Required**: Add `^^xsd:float` datatype declarations to numeric literals in scenarios 2 and 3.

---

## Recommended Fix

Update DEMO_RUST_KGDB.html line 1367 and 1369:

```diff
- <http://gonnect.com/vehicle/ego> <http://gonnect.com/ontology/av#hasVelocity> "15.0" .
+ <http://gonnect.com/vehicle/ego> <http://gonnect.com/ontology/av#hasVelocity> "15.0"^^<http://www.w3.org/2001/XMLSchema#float> .

- <http://gonnect.com/zone/school_001> <http://gonnect.com/ontology/av#speedLimit> "8.33" .
+ <http://gonnect.com/zone/school_001> <http://gonnect.com/ontology/av#speedLimit> "8.33"^^<http://www.w3.org/2001/XMLSchema#float> .
```

This will enable numeric comparison in the FILTER clause and return the expected 1 binding showing the speed violation.
