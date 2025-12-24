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

## Test Scenarios (13 Total)

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
| 13 | HyperMindAgent | LLM-powered natural language queries |

---

## Sample Output

```
================================================================================
  DIGITAL TWIN: Smart Building IoT Example
  University of Sharjah Smart Campus 2024
  rust-kgdb v0.8.18 | Real-Time IoT + Datalog Reasoning
================================================================================

[1] Loading Smart Building Digital Twin Ontology...
    Triples loaded: 123
    [PASS] Building ontology loaded successfully

[2] Ingesting Real-Time IoT Sensor Readings...
    New triples after sensor ingestion: 179
    Sensor readings added: 56
    [PASS] Real-time sensor data ingested

[3] SPARQL: Query All Temperature Readings...
    Temperature readings found: 4
      - InterdisciplinaryLab: 25.1°C
      - InterdisciplinaryLab: 25.5°C
      - ServerRoom: 22.5°C
      - Kitchen: 26.1°C
    [PASS] Temperature sensors reporting

[4] SPARQL: Monitor Critical Zones...
    Critical zones: 1
      - Server Room: 22.5°C
    [PASS] Critical zone monitoring active

[5] SPARQL: Occupancy Analysis...
    Zones with occupancy data: 2
      - InterdisciplinaryLab: 0 people
      - ConferenceRoom: 0 people
    [PASS] Occupancy sensors active

[6] SPARQL: Energy Consumption Analysis...
    Energy meters: 3
      - EnergyMeter_HVAC: 259.24 kWh (3955W current)
      - EnergyMeter_Microwave: 9.71 kWh (0W current)
      - EnergyMeter_CoffeeMachine: 14.48 kWh (0W current)
    Total current power: 3955W
    [PASS] Energy monitoring active

[7] SPARQL: HVAC System Status...
    HVAC systems: 3
    Zones controlled: Kitchen, ConferenceRoom, InterdisciplinaryLab
    [PASS] HVAC system mapped

[8] SPARQL: Zone Adjacency (Heat Propagation)...
    Adjacent zone pairs: 4
      - MailRoom <-> Kitchen
      - Kitchen <-> MailRoom
      - InterdisciplinaryLab <-> ConferenceRoom
      - ConferenceRoom <-> InterdisciplinaryLab
    [PASS] Zone adjacency mapped (OWL SymmetricProperty)

[9] Datalog: Automated HVAC Control Rules...
    Rules evaluated successfully
    Critical alerts: 1
    Temperature alerts: 1
    [ALERT] Server room temperature exceeds threshold!
    [ACTION] Increasing precision AC cooling
    [PASS] Datalog reasoning operational

[10] GraphFrame: Sensor Network Topology...
    Vertices: 9
    Edges: 12
    PageRank (node importance):
      - M5Building: 0.2942
      - ServerRoom: 0.1201
      - ConferenceRoom: 0.0975
      - InterdisciplinaryLab: 0.0949
      - Kitchen: 0.0571
    Connected components: 1
    [PASS] Sensor network analyzed

[11] ThinkingReasoner: Deductive Reasoning...
    [PASS] Reasoning engine available

[12] Real-Time Decision Engine...
    Scenario: Server room temperature spike detected

    INPUT:
      Sensor: TempSensor_Server
      Reading: 25.2°C
      Threshold: 24.0°C
      Status: EXCEEDED

    REASONING CHAIN:
      Step 1: [OBSERVATION] temp(ServerRoom) = 25.2
      Step 2: [RULE] criticalZone(ServerRoom) = true
      Step 3: [INFERENCE] temperatureAlert(ServerRoom, high)
      Step 4: [INFERENCE] criticalAlert(ServerRoom, serverOverheat)
      Step 5: [ACTION] increaseACPower(ServerRoomAC, +20%)

    OUTPUT:
      Decision: INCREASE_COOLING
      Target: ServerRoomAC
      Action: Set cooling to 100% capacity
      Proof: SHA-256 c14c1cfe549be335...

    [PASS] Real-time decision with proof

================================================================================
  TEST RESULTS: 13 PASSED, 0 FAILED - 100.0% PASS RATE
================================================================================

  DIGITAL TWIN CAPABILITIES DEMONSTRATED:
    - Real-time IoT sensor data ingestion
    - SPARQL queries for temperature, occupancy, energy
    - Datalog rules for automated HVAC control
    - OWL reasoning (SymmetricProperty, TransitiveProperty)
    - GraphFrame network analysis (PageRank, components)
    - Cryptographic proof per decision

  DATA SOURCE: University of Sharjah Smart Campus 2024
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

---

## Example Queries

### User Prompt → SPARQL → Answer

**Query 1: "What is the temperature in the server room?"**

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX iot: <http://smartbuilding.org/iot#>

SELECT ?zone ?temp WHERE {
  ?reading rdf:type iot:SensorReading .
  ?reading iot:fromSensor ?sensor .
  ?sensor iot:installedIn ?zone .
  ?zone iot:criticalZone "true" .
  ?reading iot:temperature ?temp .
}
```

**Answer:** Server Room: 20.8°C (Critical zone monitored)

---

**Query 2: "Which zones are adjacent and might share heat?"**

```sparql
PREFIX iot: <http://smartbuilding.org/iot#>

SELECT ?zone1 ?zone2 WHERE {
  ?zone1 iot:adjacentTo ?zone2 .
}
```

**Answer:**
- MailRoom ↔ Kitchen
- InterdisciplinaryLab ↔ ConferenceRoom

(OWL SymmetricProperty automatically infers bidirectional relationships)

---

**Query 3: "What is the current energy consumption?"**

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX iot: <http://smartbuilding.org/iot#>

SELECT ?meter ?total ?current WHERE {
  ?meter rdf:type iot:EnergyMeter .
  ?reading iot:fromMeter ?meter .
  ?reading iot:totalEnergy ?total .
  ?reading iot:currentPower ?current .
}
```

**Answer:**
- HVAC: 270.87 kWh (4178W current)
- Microwave: 9.61 kWh (0W idle)
- Coffee Machine: 13.62 kWh (0W idle)
- **Total current power: 4178W**

---

## HyperMindAgent Interactions (LLM-Powered)

The HyperMindAgent provides natural language querying of the knowledge graph with GPT-4o or Claude. Below are real interactions from local execution:

### Interaction Table (from actual local run)

| User Query | LLM Answer | Knowledge Source | Reasoning | Proof |
|------------|------------|------------------|-----------|-------|
| "What is the current status of the server room and should I be concerned?" | "The server room is a critical zone located on the ground floor of the M5 Building, with an area of 50 square meters and a maximum occupancy of 2. Given its designation as a critical zone, it may warrant concern depending on the current conditions and activities within." | SPARQL on `iot:criticalZone`, `iot:floor`, `iot:maxOccupancy`, `iot:area` | Agent identified server room as critical zone, combined floor/area/occupancy metadata | ✓ Generated |

**Note**: Real-Time Decision Engine (Test 12) proof: `SHA-256: 9fe06436cdf318a6...`

### Sample Agent Output

```
[13] HyperMindAgent: Natural Language Query with LLM...
    Agent: building-assistant
    Model: GPT-4o

    USER QUESTION:
    "What is the current status of the server room and should I be concerned?"

    AGENT ANSWER:
    ------------------------------------------------------------
    The server room is a critical zone located on the ground floor of the
    M5 Building, with an area of 50 square meters and a maximum occupancy
    of 2. Given its designation as a critical zone, it may warrant concern
    depending on the current conditions and activities within.
    ------------------------------------------------------------

    PROOF HASH: [object Object]...

    [PASS] HyperMindAgent query successful
```

### How It Works

1. **User Question** → Natural language input
2. **Schema Extraction** → Agent reads ontology predicates (temperature, criticalZone, etc.)
3. **SPARQL Generation** → LLM generates valid SPARQL from schema
4. **Query Execution** → rust-kgdb executes against knowledge graph
5. **Answer Synthesis** → LLM summarizes results in natural language
6. **Proof Hash** → Cryptographic verification of reasoning chain

---

## Full Output Log

📄 **[View Complete Output Log](output/digital-twin-output.txt)**
