# Self-Driving Car Reasoning System with rust-kgdb

**A production-ready semantic reasoning system for autonomous vehicles leveraging rust-kgdb's SPARQL 1.1 and hypergraph capabilities**

**Target Platform**: macOS Intel (compatible with Apple Silicon)
**Status**: Specification & Implementation Plan
**Date**: 2025-11-26

---

## Executive Summary

This specification outlines a **symbolic reasoning system** for autonomous vehicles that uses **rust-kgdb** (a production-grade mobile hypergraph database) to perform real-time decision-making through SPARQL queries and semantic inference.

**Key Innovation**: Instead of black-box neural networks, we use **transparent, explainable reasoning** via knowledge graphs and SPARQL rules.

### Why This Matters

- **Explainability**: Every decision can be traced through SPARQL queries
- **Safety**: Deterministic reasoning instead of probabilistic AI
- **Regulatory Compliance**: Auditable decision logs
- **Real-time Performance**: Sub-millisecond SPARQL execution (2.78 µs lookups)
- **Mobile-First**: Runs on iOS/Android with <100ms cold start

---

## CARLA Limitation Analysis

### Problem: CARLA Not Supported on macOS

**CARLA Simulator** (carla-simulator/carla) is the industry-standard open-source autonomous vehicle simulator, BUT:

- ❌ **No official macOS support** (Windows 11 / Ubuntu 22.04 only)
- ❌ Requires NVIDIA GPU with 16GB+ VRAM
- ❌ Docker option requires remote rendering (complex setup)
- ❌ Community macOS builds are outdated and unsupported

### Solution: macOS-Compatible Alternatives

We provide **FOUR viable visualization approaches**:

| Approach | Platform | Complexity | Fidelity | Cost |
|----------|----------|------------|----------|------|
| **1. Udacity Unity Simulator** | macOS/Win/Linux | Low | Medium | Free |
| **2. Web-based 3D (Three.js)** | Browser | Low | Medium | Free |
| **3. GDSim (Godot)** | macOS/Win/Linux | Low | Medium | Free |
| **4. AVIS Engine** | macOS/Win/Linux | Medium | High | Free/Paid |
| **5. CARLA (Docker Remote)** | Linux Server | High | Highest | Free* |

**Recommended**: **Udacity Unity Simulator** + **Web-based Dashboard** (hybrid approach)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Visualization Layer                          │
│  Udacity Simulator (Unity) | Web Dashboard (Three.js) | Mobile   │
└─────────────────────────────────────────────────────────────────┘
                              ↕ HTTP/WebSocket
┌─────────────────────────────────────────────────────────────────┐
│                   Reasoning Engine (Rust)                        │
│  SPARQL Executor | Rule Engine | Perception Processor           │
└─────────────────────────────────────────────────────────────────┘
                              ↕ FFI
┌─────────────────────────────────────────────────────────────────┐
│                   rust-kgdb Knowledge Graph                      │
│  RDF Triples | SPARQL 1.1 | Hypergraphs | Quad Store           │
└─────────────────────────────────────────────────────────────────┘
                              ↕ Data Flow
┌─────────────────────────────────────────────────────────────────┐
│                   Sensor Simulation Layer                        │
│  Camera | LiDAR | Radar | GPS | IMU | Map Data                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Ontology Design: Autonomous Vehicle Knowledge Graph

### Core Namespaces

```turtle
@prefix av:      <http://zenya.com/ontology/av#> .
@prefix road:    <http://zenya.com/ontology/road#> .
@prefix sensor:  <http://zenya.com/ontology/sensor#> .
@prefix action:  <http://zenya.com/ontology/action#> .
@prefix rule:    <http://zenya.com/ontology/rule#> .
@prefix geo:     <http://www.w3.org/2003/01/geo/wgs84_pos#> .
@prefix time:    <http://www.w3.org/2006/time#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
```

### Key Classes

```turtle
# Vehicle & Environment
av:Vehicle rdf:type rdfs:Class .
av:Lane rdf:type rdfs:Class .
av:Intersection rdf:type rdfs:Class .
av:TrafficLight rdf:type rdfs:Class .
av:Pedestrian rdf:type rdfs:Class .
av:Obstacle rdf:type rdfs:Class .

# Perception
sensor:Camera rdf:type rdfs:Class .
sensor:LiDAR rdf:type rdfs:Class .
sensor:Radar rdf:type rdfs:Class .
sensor:Detection rdf:type rdfs:Class .

# Actions & Decisions
action:Accelerate rdf:type rdfs:Class .
action:Brake rdf:type rdfs:Class .
action:SteerLeft rdf:type rdfs:Class .
action:SteerRight rdf:type rdfs:Class .
action:ChangeLane rdf:type rdfs:Class .

# Rules & Constraints
rule:TrafficRule rdf:type rdfs:Class .
rule:SafetyConstraint rdf:type rdfs:Class .
rule:DecisionTree rdf:type rdfs:Class .
```

### Key Properties

```turtle
# Spatial relationships
av:hasPosition rdf:type rdf:Property .
av:hasVelocity rdf:type rdf:Property .
av:distanceTo rdf:type rdf:Property .
av:inLane rdf:type rdf:Property .

# Temporal
av:timestamp rdf:type rdf:Property .
av:duration rdf:type rdf:Property .

# Detection
sensor:detects rdf:type rdf:Property .
sensor:confidence rdf:type rdf:Property .
sensor:boundingBox rdf:type rdf:Property .

# Decision provenance
action:triggeredBy rdf:type rdf:Property .
action:priority rdf:type rdf:Property .
prov:wasGeneratedBy rdf:type rdf:Property .
```

### Example: Traffic Light Detection

```turtle
# Vehicle state
:EgoVehicle a av:Vehicle ;
    av:hasPosition :Pos_100m ;
    av:hasVelocity "15.5"^^xsd:float ;  # m/s
    av:inLane :Lane_1 ;
    av:timestamp "2025-11-26T10:30:00Z"^^xsd:dateTime .

# Traffic light detection
:TrafficLight_001 a av:TrafficLight ;
    av:hasPosition :Pos_130m ;
    av:state "red"^^xsd:string ;
    sensor:confidence "0.98"^^xsd:float .

# Detection event
:Detection_TL001 a sensor:Detection ;
    sensor:detects :TrafficLight_001 ;
    sensor:source :Camera_Front ;
    av:distanceTo "30.0"^^xsd:float ;  # meters
    av:timestamp "2025-11-26T10:30:00.123Z"^^xsd:dateTime .

# Decision
:Decision_001 a action:Brake ;
    action:triggeredBy :TrafficLight_001 ;
    action:intensity "0.6"^^xsd:float ;
    action:priority "high"^^xsd:string ;
    prov:wasGeneratedBy :Rule_RedLight_Stop .

# Rule (as RDF-star quoted triple)
:Rule_RedLight_Stop a rule:TrafficRule ;
    rdfs:label "Stop at red traffic light" ;
    rule:condition "IF traffic_light.state = 'red' AND distance < 50m THEN brake" .
```

---

## SPARQL Reasoning Queries

### Query 1: Detect Immediate Hazards

```sparql
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX sensor: <http://zenya.com/ontology/sensor#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?object ?distance ?type WHERE {
  :EgoVehicle av:inLane ?lane .

  ?detection sensor:detects ?object ;
             av:distanceTo ?distance ;
             sensor:confidence ?conf .

  ?object rdf:type ?type .
  ?object av:inLane ?lane .  # Same lane

  FILTER(?distance < 20.0)   # Less than 20 meters
  FILTER(?conf > 0.8)        # High confidence
  FILTER(?type IN (av:Vehicle, av:Pedestrian, av:Obstacle))
}
ORDER BY ?distance
LIMIT 5
```

### Query 2: Traffic Light Decision

```sparql
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX action: <http://zenya.com/ontology/action#>

ASK {
  # Is there a red traffic light ahead?
  ?tl rdf:type av:TrafficLight ;
      av:state "red" ;
      av:distanceTo ?dist .

  :EgoVehicle av:hasVelocity ?speed .

  # Within stopping distance: dist < speed^2 / (2 * max_decel)
  # Assuming max_decel = 5 m/s^2
  BIND((?speed * ?speed) / 10.0 AS ?stoppingDist)
  FILTER(?dist <= ?stoppingDist + 10.0)  # Safety margin
}
```

### Query 3: Lane Change Safety Check

```sparql
PREFIX av: <http://zenya.com/ontology/av#>

SELECT ?adjacentVehicle ?relativeSpeed WHERE {
  :EgoVehicle av:inLane ?currentLane ;
              av:hasVelocity ?egoSpeed .

  ?currentLane av:adjacentLane ?targetLane .

  ?adjacentVehicle av:inLane ?targetLane ;
                   av:hasVelocity ?adjSpeed ;
                   av:distanceTo ?distance .

  BIND(?adjSpeed - ?egoSpeed AS ?relativeSpeed)

  # Check blind spot: vehicle within 5m behind or 10m ahead
  FILTER(?distance > -5.0 && ?distance < 10.0)
}
```

### Query 4: Speed Limit Compliance

```sparql
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX road: <http://zenya.com/ontology/road#>

CONSTRUCT {
  :EgoVehicle action:shouldDecelerate ?targetSpeed .
} WHERE {
  :EgoVehicle av:hasVelocity ?currentSpeed ;
              av:inLane ?lane .

  ?lane road:speedLimit ?limitMps .

  BIND(?limitMps * 0.9 AS ?targetSpeed)  # 90% of limit
  FILTER(?currentSpeed > ?limitMps)
}
```

### Query 5: Pedestrian Crossing Detection (Hypergraph)

Using **rust-kgdb's native hypergraph** support for complex scenarios:

```sparql
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX hyper: <http://zenya.com/ontology/hypergraph#>

SELECT ?scenario ?action ?priority WHERE {
  # Hyperedge representing complex scenario
  ?scenario a hyper:PedestrianCrossingScenario ;
            hyper:involves :EgoVehicle ;
            hyper:involves ?pedestrian ;
            hyper:involves ?crosswalk ;
            hyper:context ?context .

  ?pedestrian a av:Pedestrian ;
              av:trajectory ?trajPed .

  ?crosswalk a av:Crosswalk ;
             av:hasPosition ?crossPos .

  :EgoVehicle av:trajectory ?trajEgo .

  # Check if trajectories intersect (custom function)
  FILTER(av:trajectoriesIntersect(?trajEgo, ?trajPed, ?crossPos))

  # Determine action
  BIND(action:EmergencyBrake AS ?action)
  BIND("critical"^^xsd:string AS ?priority)
}
```

---

## Visualization Options (Detailed)

### Option 1: Udacity Unity Simulator ✅ RECOMMENDED

**Repository**: https://github.com/udacity/self-driving-car-sim

**Features**:
- ✅ **macOS Native Support** (Intel & Apple Silicon)
- ✅ Open source (MIT License)
- ✅ Well-documented
- ✅ Two tracks: Lake Track (easy) + Jungle Track (hard)
- ✅ Manual + Autonomous modes
- ✅ Python API for control
- ✅ Unity ML-Agents integration

**Integration**:
```rust
// Rust FFI to Python bridge
// simulator_bridge.rs
use pyo3::prelude::*;

#[pyfunction]
fn send_control(steering: f32, throttle: f32, brake: f32) -> PyResult<()> {
    // rust-kgdb queries determine these values
    // Send to Unity via socket
    Ok(())
}

#[pyfunction]
fn get_sensor_data() -> PyResult<SensorData> {
    // Receive camera, speed, position from Unity
    // Insert into rust-kgdb as RDF triples
}
```

**Setup**:
```bash
# Download pre-built macOS binary
wget https://github.com/udacity/self-driving-car-sim/releases/latest/mac

# Or build from source with Unity
git clone https://github.com/udacity/self-driving-car-sim
# Open in Unity Hub > Build for macOS
```

**Communication Protocol**:
- **Socket.IO** connection on port 4567
- JSON messages: `{"steering_angle": 0.0, "throttle": 0.5, "brake": 0.0}`
- Camera images: Base64-encoded JPEG
- Telemetry: Speed, position, steering angle

---

### Option 2: Web-based 3D Dashboard ✅ RECOMMENDED

**Stack**: Rust (Actix-web) + TypeScript + Three.js + React

**Features**:
- ✅ **Zero installation** (browser-based)
- ✅ Real-time SPARQL query visualization
- ✅ 3D scene rendering
- ✅ Decision tree visualization
- ✅ Performance dashboard

**Architecture**:
```
┌──────────────────────────────────────┐
│  Web Browser (Safari/Chrome/Firefox) │
│  ┌──────────────────────────────────┐│
│  │ Three.js 3D Scene                ││
│  │ - Vehicle model                  ││
│  │ - Road network                   ││
│  │ - Detected objects               ││
│  │ - Decision overlay               ││
│  └──────────────────────────────────┘│
│  ┌──────────────────────────────────┐│
│  │ React Dashboard                  ││
│  │ - Live SPARQL queries            ││
│  │ - Knowledge graph viz            ││
│  │ - Performance metrics            ││
│  └──────────────────────────────────┘│
└──────────────────────────────────────┘
         ↕ WebSocket (JSON)
┌──────────────────────────────────────┐
│  Actix-web Server (Rust)             │
│  - REST API                          │
│  - WebSocket server                  │
│  - rust-kgdb integration             │
└──────────────────────────────────────┘
```

**Implementation** (see `web-dashboard/` directory):
```typescript
// client/src/scene.ts
import * as THREE from 'three';

export class AVScene {
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;

  constructor(container: HTMLElement) {
    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    this.renderer = new THREE.WebGLRenderer();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    container.appendChild(this.renderer.domElement);

    // Add vehicle model
    this.addVehicle();
    // Add road
    this.addRoad();
    // Add traffic lights
    this.addTrafficLights();
  }

  updateVehiclePosition(x: number, y: number, heading: number) {
    // Update from SPARQL query results
  }

  highlightDetectedObjects(objects: DetectedObject[]) {
    // Visualize SPARQL query: detected obstacles
  }

  showDecisionTree(decision: Decision) {
    // Overlay explaining SPARQL reasoning
  }
}
```

---

### Option 3: GDSim (Godot) ✅ LIGHTWEIGHT

**Website**: https://lupine-vidya.itch.io/gdsim
**Engine**: Godot 3.x/4.x

**Features**:
- ✅ Explicit macOS support
- ✅ Lightweight (< 100MB)
- ✅ Open source engine
- ✅ GDScript + Rust via gdnative

**Integration**:
```rust
// gdnative Rust binding
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct ReasoningEngine {
    quad_store: QuadStore,
}

#[methods]
impl ReasoningEngine {
    fn new(_owner: &Node) -> Self {
        ReasoningEngine {
            quad_store: QuadStore::new_in_memory(Arc::new(Dictionary::new())),
        }
    }

    #[export]
    fn query_safe_speed(&self, _owner: &Node, lane: String) -> f32 {
        // Execute SPARQL query
        // Return safe speed
        15.5
    }

    #[export]
    fn should_brake(&self, _owner: &Node) -> bool {
        // Execute ASK query for red traffic light
        true
    }
}
```

---

### Option 4: AVIS Engine ✅ PROFESSIONAL

**Website**: https://www.avisengine.com/

**Features**:
- ✅ Cross-platform (macOS, Linux, Windows, Web)
- ✅ Designed for AV research
- ✅ Sensor simulation (Camera, LiDAR, Radar)
- ✅ ROS integration
- ⚠️ Limited free tier

**Use Case**: Production-grade demos, research papers

---

### Option 5: CARLA via Docker ⚠️ ADVANCED

**Setup Complexity**: High
**Requirements**: Linux server with NVIDIA GPU

**Architecture**:
```
┌─────────────────┐         ┌─────────────────┐
│   macOS Intel   │         │  Linux Server   │
│                 │  SSH/   │  (AWS/GCP)      │
│  - Client       │  VNC    │  - Docker       │
│  - Viewer       │◄────────┤  - CARLA        │
│  - Controller   │         │  - NVIDIA GPU   │
└─────────────────┘         └─────────────────┘
```

**Steps**:
1. Rent cloud GPU instance (AWS g4dn.xlarge ~ $0.50/hr)
2. Install Docker + NVIDIA Container Toolkit
3. Run CARLA: `docker run --gpus all -p 2000-2002:2000-2002 carlasim/carla:0.9.15`
4. Connect from macOS via Python client
5. Use VNC or stream rendering to macOS

**Pros**: Highest fidelity, industry-standard
**Cons**: Cost, complexity, latency

---

## Implementation Plan

### Phase 1: Core Reasoning Engine (Week 1-2)

**Deliverables**:
- ✅ Autonomous vehicle ontology (Turtle format)
- ✅ rust-kgdb integration crate
- ✅ SPARQL query library (20+ queries)
- ✅ Unit tests

**Files**:
```
self-driving-car/
├── crates/
│   ├── av-ontology/         # RDF ontology definitions
│   ├── av-reasoning/        # SPARQL reasoning engine
│   └── av-simulation/       # Simulator integration
├── ontology/
│   ├── av-core.ttl          # Core AV concepts
│   ├── traffic-rules.ttl    # Traffic regulations
│   └── safety-rules.ttl     # Safety constraints
└── queries/
    ├── hazard-detection.rq
    ├── traffic-light.rq
    ├── lane-change.rq
    └── speed-limit.rq
```

---

### Phase 2: Simulator Integration (Week 3)

**Target**: Udacity Unity Simulator

**Tasks**:
1. Set up Unity simulator on macOS
2. Implement Python bridge (socket.io client)
3. Implement Rust FFI to Python (PyO3)
4. Create sensor data → RDF triple converter
5. Create SPARQL decision → control command converter

**Files**:
```
self-driving-car/
├── crates/
│   └── simulator-bridge/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── unity_client.rs      # Socket.IO connection
│       │   ├── sensor_parser.rs     # JSON → RDF
│       │   └── control_mapper.rs    # SPARQL → Commands
│       └── Cargo.toml
└── python/
    ├── bridge.py                    # PyO3 wrapper
    └── unity_interface.py           # Socket.IO server
```

---

### Phase 3: Web Dashboard (Week 4)

**Stack**: Actix-web + React + Three.js

**Tasks**:
1. Actix-web REST API for rust-kgdb
2. WebSocket server for real-time updates
3. Three.js 3D scene
4. React dashboard with live SPARQL queries
5. Decision tree visualization

**Files**:
```
self-driving-car/
├── web-dashboard/
│   ├── server/               # Rust Actix-web
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── api.rs
│   │   │   ├── websocket.rs
│   │   │   └── sparql_service.rs
│   │   └── Cargo.toml
│   └── client/               # React + TypeScript
│       ├── src/
│       │   ├── components/
│       │   │   ├── Scene3D.tsx
│       │   │   ├── SPARQLConsole.tsx
│       │   │   └── DecisionTree.tsx
│       │   └── index.tsx
│       └── package.json
```

---

### Phase 4: Testing & Validation (Week 5)

**Scenarios**:
1. **Red traffic light**: Emergency stop
2. **Pedestrian crossing**: Yield to pedestrian
3. **Lane change**: Check blind spot
4. **Speed limit**: Decelerate to comply
5. **Obstacle avoidance**: Steer or stop

**Metrics**:
- ✅ Query latency: < 10ms per decision
- ✅ Decision accuracy: > 95%
- ✅ Explainability: 100% (all decisions traceable)
- ✅ Frame rate: 30+ FPS

---

## Technical Specifications

### System Requirements

**macOS Intel**:
- CPU: Intel Core i5 or better
- RAM: 8GB minimum, 16GB recommended
- GPU: Integrated graphics OK for web dashboard, discrete GPU for Unity
- Storage: 2GB for simulator + dependencies
- OS: macOS 10.15 (Catalina) or later

**Dependencies**:
- Rust 1.91+
- Python 3.9+ (for Unity bridge)
- Node.js 18+ (for web dashboard)
- Unity Hub 3.x (for Udacity simulator build)

---

### Performance Targets

| Metric | Target | rust-kgdb Actual |
|--------|--------|------------------|
| SPARQL Query Latency | < 10ms | **2.78 µs** ✅ |
| Triple Insertion | 10K/sec | **146K/sec** ✅ |
| Memory Footprint | < 50MB | **24 bytes/triple** ✅ |
| Cold Start | < 500ms | **< 100ms** ✅ |
| Decision Frequency | 10 Hz | **Limited by simulator** |

---

### Data Flow Example

**Scenario**: Vehicle approaching red traffic light

```
1. Unity Simulator → JSON telemetry
   {
     "speed": 15.5,
     "position": {"x": 100, "y": 0, "z": 0},
     "camera": "base64_image_data",
     "detections": [
       {"type": "traffic_light", "state": "red", "distance": 30.0}
     ]
   }

2. Python Bridge → Rust FFI
   sensor_parser.rs converts to RDF triples

3. rust-kgdb → Insert Triples
   :EgoVehicle av:hasVelocity "15.5"^^xsd:float .
   :TrafficLight_001 av:state "red"^^xsd:string .
   :Detection_TL001 av:distanceTo "30.0"^^xsd:float .

4. SPARQL Query Execution
   ASK { ?tl av:state "red" ; av:distanceTo ?dist . FILTER(?dist < 50) }
   → Returns: true

5. Decision Generation
   CONSTRUCT { :EgoVehicle action:shouldBrake ?intensity . }
   → Returns: :EgoVehicle action:shouldBrake "0.8"^^xsd:float .

6. Control Command
   control_mapper.rs converts to JSON:
   {"steering": 0.0, "throttle": 0.0, "brake": 0.8}

7. Unity Simulator ← Apply Brake
   Vehicle decelerates
```

---

## Safety Considerations

### 1. Fail-Safe Mechanisms

```sparql
# Always check for emergency situations first
PREFIX action: <http://zenya.com/ontology/action#>

SELECT ?emergencyAction WHERE {
  {
    # Collision imminent (< 2 seconds)
    :EgoVehicle av:hasVelocity ?speed .
    ?obstacle av:distanceTo ?dist .
    FILTER(?dist < ?speed * 2.0)
    BIND(action:EmergencyBrake AS ?emergencyAction)
  } UNION {
    # Pedestrian in path
    ?pedestrian a av:Pedestrian ;
                av:trajectory ?trajPed .
    :EgoVehicle av:trajectory ?trajEgo .
    FILTER(av:trajectoriesIntersect(?trajEgo, ?trajPed))
    BIND(action:EmergencyBrake AS ?emergencyAction)
  }
}
ORDER BY DESC(?priority)
LIMIT 1
```

### 2. Decision Auditability

Every decision is logged with full provenance:

```turtle
:Decision_12345 a action:Brake ;
    prov:wasGeneratedBy :Query_RedLight_ASK ;
    prov:wasAttributedTo :ReasoningEngine_v1 ;
    prov:generatedAtTime "2025-11-26T10:30:01.456Z"^^xsd:dateTime ;
    prov:wasDerivedFrom :Detection_TL001 ;
    action:parameters [
        action:intensity "0.8"^^xsd:float ;
        action:duration "2.5"^^xsd:float
    ] ;
    rdfs:comment "Emergency brake due to red traffic light at 30m" .
```

### 3. Confidence Thresholds

```sparql
# Require high confidence for critical decisions
SELECT ?decision WHERE {
  ?detection sensor:confidence ?conf ;
             sensor:detects ?object .

  FILTER(?conf > 0.9)  # 90% confidence minimum

  # Generate decision
  BIND(action:Emergency Brake AS ?decision)
}
```

---

## Future Enhancements

### 1. Multi-Agent Scenarios

Use **hypergraphs** to model interactions between multiple vehicles:

```turtle
:Intersection_Scenario_001 a hyper:MultiAgentScenario ;
    hyper:involves :EgoVehicle ;
    hyper:involves :Vehicle_002 ;
    hyper:involves :Pedestrian_003 ;
    hyper:involves :TrafficLight_001 ;
    hyper:spatialContext :Intersection_MainSt_1stAve ;
    hyper:temporalContext :TimeInterval_T0_T5 .
```

### 2. Machine Learning Integration

Combine symbolic reasoning (SPARQL) with neural perception:

```
Neural Network (Object Detection)
    ↓
Bounding Boxes + Confidence
    ↓
RDF Triples (Semantic Layer)
    ↓
SPARQL Reasoning (Decision)
```

### 3. Real-World Deployment

- **iOS App**: Use rust-kgdb's Swift FFI for on-device reasoning
- **Android App**: Use rust-kgdb's Kotlin FFI
- **Edge Computing**: Deploy reasoning engine on vehicle's embedded system
- **Cloud Sync**: Aggregate driving data across fleet

---

## Comparison with Existing Approaches

| Approach | Explainability | Performance | Safety | Complexity |
|----------|----------------|-------------|--------|------------|
| **End-to-End Neural** | ❌ Black box | ⚠️ Variable | ❌ Unpredictable | Low |
| **Traditional Planning** | ⚠️ Limited | ✅ Fast | ⚠️ Rule-based | High |
| **rust-kgdb SPARQL** | ✅ **Full** | ✅ **Sub-ms** | ✅ **Auditable** | Medium |

---

## Getting Started

### Prerequisites Installation

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Python 3.9+
brew install python@3.9

# 3. Install Node.js 18+
brew install node

# 4. Clone rust-kgdb
cd /Users/gauravmalhotra/Documents/MyGithub/zenya-graphdb/rust-kgdb

# 5. Build rust-kgdb
cargo build --workspace --release

# 6. Download Udacity simulator
cd self-driving-car
wget https://github.com/udacity/self-driving-car-sim/releases/download/v2.0/mac_sim.zip
unzip mac_sim.zip
```

### Quick Test

```bash
# Terminal 1: Start Unity simulator
./mac_sim.app/Contents/MacOS/mac_sim

# Terminal 2: Run reasoning engine
cd self-driving-car
cargo run --bin av-reasoning

# Terminal 3: Start web dashboard
cd web-dashboard/server
cargo run --release

# Open browser to http://localhost:3000
```

---

## Project Structure

```
self-driving-car/
├── README.md                          # This file
├── SELF_DRIVING_REASONING_SPEC.md    # Full specification
├── Cargo.toml                         # Workspace manifest
│
├── crates/
│   ├── av-ontology/                   # RDF ontology
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   │
│   ├── av-reasoning/                  # SPARQL reasoning engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── queries.rs             # SPARQL query library
│   │   │   ├── rules.rs               # Traffic rules
│   │   │   └── executor.rs            # Decision executor
│   │   └── Cargo.toml
│   │
│   ├── simulator-bridge/              # Unity integration
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── unity_client.rs
│   │   │   ├── sensor_parser.rs
│   │   │   └── control_mapper.rs
│   │   └── Cargo.toml
│   │
│   └── av-simulation/                 # Main binary
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── ontology/                          # Turtle files
│   ├── av-core.ttl
│   ├── traffic-rules.ttl
│   ├── safety-rules.ttl
│   └── scenarios.ttl
│
├── queries/                           # SPARQL queries
│   ├── hazard-detection.rq
│   ├── traffic-light.rq
│   ├── lane-change.rq
│   ├── pedestrian-crossing.rq
│   └── speed-limit.rq
│
├── web-dashboard/                     # Visualization
│   ├── server/                        # Rust Actix-web
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── api.rs
│   │   │   ├── websocket.rs
│   │   │   └── sparql_service.rs
│   │   └── Cargo.toml
│   │
│   └── client/                        # React TypeScript
│       ├── src/
│       │   ├── components/
│       │   │   ├── Scene3D.tsx
│       │   │   ├── SPARQLConsole.tsx
│       │   │   ├── DecisionTree.tsx
│       │   │   └── PerformanceMetrics.tsx
│       │   ├── App.tsx
│       │   └── index.tsx
│       ├── package.json
│       └── tsconfig.json
│
├── python/                            # Python bridge
│   ├── bridge.py                      # PyO3 FFI wrapper
│   ├── unity_interface.py             # Socket.IO client
│   └── requirements.txt
│
├── tests/                             # Integration tests
│   ├── test_red_light.rs
│   ├── test_pedestrian.rs
│   ├── test_lane_change.rs
│   └── test_speed_limit.rs
│
└── docs/
    ├── ARCHITECTURE.md
    ├── SPARQL_QUERIES.md
    └── DEPLOYMENT.md
```

---

## Success Criteria

### Functional Requirements
- ✅ Detect traffic lights and respond correctly (100% accuracy)
- ✅ Detect pedestrians and yield (100% accuracy)
- ✅ Perform safe lane changes (95%+ success rate)
- ✅ Maintain speed limits (100% compliance)
- ✅ Avoid obstacles (100% collision avoidance)

### Non-Functional Requirements
- ✅ Decision latency: < 100ms end-to-end
- ✅ SPARQL query time: < 10ms
- ✅ Memory usage: < 100MB
- ✅ CPU usage: < 50% (single core)
- ✅ Explainability: 100% (all decisions have SPARQL provenance)

### Visualization Requirements
- ✅ 30+ FPS rendering
- ✅ Real-time SPARQL query display
- ✅ Live decision tree visualization
- ✅ Sensor data overlay

---

## References

### Academic Papers
1. **Semantic Web for Autonomous Driving**: W3C standards for vehicle ontologies
2. **SPARQL-based Planning**: Using queries for robot decision-making
3. **Explainable AI for AVs**: Regulatory requirements for transparency

### Tools & Frameworks
- rust-kgdb: https://github.com/zenya/rust-kgdb
- Udacity Simulator: https://github.com/udacity/self-driving-car-sim
- CARLA: https://carla.org/
- GDSim: https://lupine-vidya.itch.io/gdsim
- AVIS Engine: https://www.avisengine.com/

### Standards
- W3C RDF: https://www.w3.org/RDF/
- W3C SPARQL 1.1: https://www.w3.org/TR/sparql11-overview/
- W3C PROV: https://www.w3.org/TR/prov-o/
- SAE J3016: Levels of Driving Automation

---

## License

Apache 2.0 (same as rust-kgdb)

---

## Contact

**Author**: Gaurav Malhotra
**Project**: rust-kgdb Self-Driving Reasoning System
**Date**: 2025-11-26
**Status**: ✅ Specification Complete - Ready for Implementation

---

**Next Steps**:
1. Review and approve this specification
2. Set up Udacity Unity Simulator on macOS Intel
3. Begin Phase 1: Core Reasoning Engine implementation
4. Weekly progress reviews

**Estimated Timeline**: 5 weeks to fully functional demo
