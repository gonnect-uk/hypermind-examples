# Digital Twin: Smart Building IoT Demo

**Real-time HVAC automation with explainable AI decisions.**

```bash
npm run digital-twin
```

---

## Overview

This demo implements a Smart Building Digital Twin using the University of Sharjah Smart Campus 2024 dataset patterns. It demonstrates:

- Real-time sensor data ingestion (temperature, humidity, occupancy, energy)
- Automated HVAC control using Datalog rules
- Energy optimization with graph analytics
- Cryptographic proofs for every control decision

---

## Architecture

```
Physical Building          Digital Twin (KGDB)           AI Control
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│   Sensors       │ ──→   │  Knowledge      │ ──→   │  ThinkingGraph  │
│   - Temperature │       │  Graph          │       │  Reasoner       │
│   - Humidity    │       │  (RDF + OWL)    │       │                 │
│   - Occupancy   │       │                 │       │  Datalog Rules  │
│   - Energy      │       │  Datalog        │       │                 │
└─────────────────┘       │  Reasoning      │       │  Proofs         │
                          └─────────────────┘       └─────────────────┘
                                   ↓
                          HVAC Control Commands
                          (with cryptographic proofs)
```

---

## Data Model

### Building Ontology

```turtle
@prefix bldg: <http://smartbuilding.org/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

# Zone adjacency is symmetric
bldg:adjacentTo a owl:SymmetricProperty .

# Building structure
bldg:M5Building a bldg:Building ;
    bldg:hasZone bldg:Zone_Lecture_Hall_1 ,
                 bldg:Zone_Office_Wing_A ,
                 bldg:Zone_Lab_Computer .

# Sensors
bldg:Sensor_TH_001 a bldg:TempHumiditySensor ;
    bldg:installedIn bldg:Zone_Lecture_Hall_1 ;
    bldg:sensorId "TH_001" .
```

### Real-Time Sensor Readings

```turtle
bldg:Reading_TH_001_1735036800 a bldg:SensorReading ;
    bldg:fromSensor bldg:Sensor_TH_001 ;
    bldg:temperature "28.5"^^xsd:decimal ;
    bldg:humidity "65.2"^^xsd:decimal ;
    bldg:timestamp "2024-12-24T08:00:00Z"^^xsd:dateTime .
```

---

## Datalog Rules

### Temperature Alert Detection

```prolog
% High temperature alert when > 26°C
highTempAlert(?zone, ?reading, ?temp) :-
    sensorReading(?reading),
    fromSensor(?reading, ?sensor),
    installedIn(?sensor, ?zone),
    temperature(?reading, ?temp),
    greaterThan(?temp, 26).
```

### HVAC Control Automation

```prolog
% Activate cooling when temperature exceeds threshold
activateCooling(?zone) :-
    highTempAlert(?zone, _, ?temp),
    occupied(?zone, true).

% Reduce cooling for unoccupied zones
reduceCooling(?zone) :-
    highTempAlert(?zone, _, _),
    occupied(?zone, false).
```

### Energy Optimization

```prolog
% Identify energy waste patterns
energyWaste(?zone) :-
    hvacStatus(?zone, "cooling"),
    occupied(?zone, false),
    temperature(?zone, ?temp),
    lessThan(?temp, 24).
```

---

## Test Scenarios (12 Total)

| # | Scenario | What It Tests |
|---|----------|---------------|
| 1 | Building Load | Ontology structure, zones, sensors |
| 2 | Sensor Ingestion | Real-time reading storage |
| 3 | Temperature Query | SPARQL aggregation (AVG, MAX) |
| 4 | Critical Zones | Temperature threshold detection |
| 5 | Occupancy Status | Current occupancy by zone |
| 6 | Energy Consumption | Total energy by zone |
| 7 | HVAC Status | Control system state |
| 8 | Zone Adjacency | OWL SymmetricProperty reasoning |
| 9 | Datalog Reasoning | highTempAlert rule derivation |
| 10 | GraphFrame Analysis | PageRank for zone importance |
| 11 | ThinkingReasoner | Natural language queries with proofs |
| 12 | Real-Time Control | Automated HVAC decisions |

---

## Sample Output

```
=== Smart Building Digital Twin Demo ===
Using University of Sharjah Smart Campus 2024 dataset patterns

✓ Building ontology loaded: 156 triples
✓ Real-time sensor data: 48 readings ingested

--- Temperature Analysis ---
Zone: Lecture Hall 1
  Current: 28.5°C (HIGH)
  Humidity: 65.2%
  Status: COOLING ACTIVATED
  Proof: sha256:7f3a...

--- Energy Optimization ---
Zone: Office Wing A
  Status: ENERGY WASTE DETECTED
  Reason: Cooling active, zone unoccupied
  Recommendation: Reduce HVAC to 50%
  Proof: sha256:9b2c...

--- GraphFrame Insights ---
Most Critical Zones (PageRank):
  1. Lecture Hall 1 (0.342)
  2. Lab Computer (0.287)
  3. Office Wing A (0.198)

=== 12/12 SCENARIOS PASSED ===
```

---

## Dataset Reference

Based on patterns from:
- **University of Sharjah Smart Campus 2024**
- [GitHub: Dataset-of-IoT-Based-Energy-and-Environmental-Parameters](https://github.com/adel8641/Dataset-of-IoT-Based-Energy-and-Environmental-Parameters)

Features:
- Temperature/humidity readings every 5 minutes
- Occupancy detection via motion sensors
- Energy consumption per zone
- HVAC operational status

---

## Key Features Demonstrated

1. **OWL Reasoning**: Symmetric property for zone adjacency
2. **Datalog Rules**: Automated HVAC control logic
3. **GraphFrame Analytics**: Zone importance via PageRank
4. **ThinkingReasoner**: Natural language queries
5. **Cryptographic Proofs**: SHA-256 hash for every decision
6. **Real-Time Processing**: Sensor data ingestion and response

---

## Run the Demo

```bash
cd hypermind-examples
npm install
npm run digital-twin
```

No API key required - runs entirely in-memory with simulated sensor data.
