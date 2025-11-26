# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Project Overview

**self-driving-car** is an explainable AI demonstration for autonomous vehicles using **rust-kgdb** (the parent repository's hypergraph database) for semantic reasoning. Unlike black-box neural networks, every decision is made through transparent SPARQL queries and Datalog inference that can be audited for ISO 26262 compliance.

**Current Status**: Production-quality 3D browser demo complete. Future work includes Unity simulator integration.

---

## Quick Start

### Running the Interactive Demo

The main deliverable is a **standalone HTML demo** requiring NO compilation:

```bash
# Open the demo in your default browser
open DEMO_FINAL_1764165585.html

# Or use Python's HTTP server for cleaner URLs
python3 -m http.server 8000
# Then visit: http://localhost:8000/DEMO_FINAL_1764165585.html
```

**Force reload to see latest changes**: Press `Command+Shift+R` (macOS) or `Ctrl+Shift+R` (Windows/Linux)

### Building Rust Components (Future Work)

```bash
# Build all workspace crates
cargo build --workspace --release

# Build specific crate
cargo build -p av-reasoning --release
cargo build -p web-dashboard --release

# Run web dashboard server (not yet implemented)
cargo run -p web-dashboard --release
```

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                DEMO_FINAL_1764165585.html                    │
│  - Three.js 3D visualization (car, obstacles, road)         │
│  - SPARQL query display with actual query text              │
│  - Datalog inference chains (rules + inference steps)       │
│  - Native hypergraph visualization (n-ary relationships)    │
│  - 3 scenarios: Traffic Light, Pedestrian, School Zone      │
└─────────────────────────────────────────────────────────────┘
                    ↓ (Future Integration)
┌─────────────────────────────────────────────────────────────┐
│              Rust Workspace (5 Crates)                       │
│  av-ontology      → RDF ontology definitions (AV vocab)     │
│  av-reasoning     → SPARQL executor + Datalog rules         │
│  simulator-bridge → Unity/simulator integration (WebSocket) │
│  av-simulation    → Main binary (orchestration)             │
│  web-dashboard    → Actix-web server for real-time viz      │
└─────────────────────────────────────────────────────────────┘
                    ↓ FFI
┌─────────────────────────────────────────────────────────────┐
│                   rust-kgdb (Parent Repo)                    │
│  crates/rdf-model     → Core RDF types (Node, Triple, Quad) │
│  crates/hypergraph    → Native n-ary hyperedge support      │
│  crates/sparql        → SPARQL 1.1 engine (64 builtins)     │
│  crates/storage       → InMemory/RocksDB/LMDB backends       │
│  crates/rdf-io        → Turtle/N-Triples parsers            │
└─────────────────────────────────────────────────────────────┘
```

### Key Architectural Decisions

1. **rust-kgdb Dependency**: This workspace depends on the **parent repository** (`../crates/`) for all graph database functionality. Do NOT copy or vendor rust-kgdb code into this directory.

2. **Event-Driven Reasoning**: The demo follows a time-based event pipeline:
   - T=0.0s: Scenario starts, vehicle initialized
   - T=0.5s: Sensor detection → Hyperedge H1 created
   - T=1.0s: SPARQL query executed (2.78 µs lookup!)
   - T=1.2s: Datalog inference → Hyperedge H2 created
   - T=1.5s: Decision made → Hyperedge H3 created
   - T=2.0s+: Vehicle actuates brakes

3. **Hypergraph Visualization**: Each scenario includes native hypergraph rendering showing 3-way and 4-way relationships that traditional binary RDF graphs cannot express atomically.

---

## Demo Implementation Details

### DEMO_FINAL_1764165585.html Structure

The demo is a **single self-contained HTML file** (~55KB) with embedded JavaScript. Key sections:

#### 1. Scenario Data (lines ~610-850)
Each scenario object contains:
- `obstaclePosition`: Where obstacle is placed on road (x-coordinate)
- `crosswalkStart`: Center of crosswalk stripes (for pedestrian scenario)
- `safetyMargin`: How many meters before obstacle car should stop
- `carLength`: 5 units (affects stop position calculation)
- `reasoning`: 6-step reasoning process displayed in left panel
- `sparql`: Full SPARQL query text + result
- `datalog`: Inference rules + chain (given facts → inferred conclusions)
- `hypergraph`: Nodes and hyperedges for n-ary relationship visualization
- `physics`: Physics formulas shown in panel

#### 2. Critical Stop Position Calculation (lines ~1178-1198)

**For Pedestrian Scenario** (has `crosswalkStart`):
```javascript
// Car approaches from LEFT (-80), crosswalk stripes are width 2 centered at crosswalkStart
const crosswalkEdge = scenario.crosswalkStart + 1;  // Right edge (car hits this first!)
const carFrontStopPosition = crosswalkEdge - scenario.safetyMargin;
stopPosition = carFrontStopPosition - (scenario.carLength / 2);
```

**For Other Scenarios** (traffic light, school zone):
```javascript
stopPosition = scenario.obstaclePosition - scenario.safetyMargin;
```

**CRITICAL BUG TO AVOID**: The crosswalk calculation MUST use `+ 1` (right edge) not `- 1` (left edge) because the car approaches from the left side of the screen!

#### 3. Dynamic Obstacle Positioning (lines ~1025-1042)

The `createObstacle(type, position)` function MUST accept a position parameter and override the default for pedestrian scenarios:

```javascript
if (type === 'pedestrian') {
    obstacle = createPedestrian();
    if (position !== undefined) {
        obstacle.position.x = position;  // Override hardcoded position!
    }
}
```

The call site passes `scenario.obstaclePosition` to ensure crosswalk appears at correct location.

#### 4. Scenario Index Management (lines ~1207-1220)

**CRITICAL**: `currentScenario` is incremented AFTER animation completes, NOT at the start:

```javascript
// In animate() loop:
} else {
    if (animationActive) {
        animationActive = false;
        addEvent("Vehicle stopped", "✅ Braking complete");

        // Increment AFTER completion (was incorrectly at nextScenario() start!)
        currentScenario = (currentScenario + 1) % scenarios.length;
    }
}
```

This ensures the animation uses the CORRECT scenario data instead of the next scenario's data.

---

## Workspace Crates (Future Development)

### av-ontology
**Purpose**: Define autonomous vehicle RDF vocabulary and ontology

**Key Files**:
- `src/lib.rs`: Exports AV namespace constants (av:Vehicle, av:hasVelocity, etc.)

**Usage**:
```rust
use av_ontology::{AV_NS, SENSOR_NS, ROAD_NS};
let vehicle_type = format!("{}Vehicle", AV_NS);
```

### av-reasoning
**Purpose**: Execute SPARQL queries and Datalog rules against rust-kgdb

**Key Files**:
- `src/models.rs`: Rust structs for VehicleState, Obstacle, Decision
- `src/queries.rs`: SPARQL query templates (ASK/SELECT for each scenario)
- `src/rules.rs`: Datalog rules (ISO 26262, SAE J3016 compliance)
- `src/executor.rs`: Reasoning pipeline orchestration

**Architecture**:
```rust
// Reasoning pipeline
1. Sensor data → RDF triples (via rdf-model from parent)
2. Insert into storage backend (InMemory/RocksDB from parent)
3. Execute SPARQL query (via sparql crate from parent)
4. Apply Datalog rules (forward chaining)
5. Generate hyperedges for multi-entity context (via hypergraph from parent)
6. Return Decision with full provenance
```

### simulator-bridge
**Purpose**: WebSocket/Socket.IO bridge to Unity/Udacity simulator

**Key Files**:
- `src/lib.rs`: Bridge trait definitions
- `src/unity_client.rs`: Unity simulator client (not yet implemented)

**Protocol**:
```json
// Receive from simulator (30 FPS)
{
  "vehicle": {"x": -60, "velocity": 13.3},
  "obstacles": [{"type": "traffic_light", "state": "red", "x": -30}]
}

// Send to simulator
{
  "steering": 0.0,
  "throttle": 0.0,
  "brake": 0.8,
  "provenance": "SPARQL query: traffic-light.rq"
}
```

### av-simulation
**Purpose**: Main binary that orchestrates reasoning + simulator

**Usage** (future):
```bash
cargo run --release -- --simulator unity --port 9001
```

### web-dashboard
**Purpose**: Actix-web server for real-time reasoning visualization

**Static Files**: `crates/web-dashboard/static/index.html`

---

## Common Development Patterns

### Modifying Demo Scenarios

**DO**:
- Edit scenario data in `DEMO_FINAL_1764165585.html` (lines ~610-850)
- Always adjust BOTH `obstaclePosition` AND `crosswalkStart` for pedestrian scenario
- Test with different `safetyMargin` values to see stop behavior changes
- Update reasoning steps to match new scenario logic

**DON'T**:
- Hardcode obstacle positions in `createObstacle()` functions (use parameter)
- Increment `currentScenario` before animation completes (causes wrong data display)
- Forget to force reload browser (Command+Shift+R) after HTML changes

### Adding New Scenarios

1. Add new scenario object to `scenarios` array with all required fields
2. Create obstacle creation function (e.g., `createConstructionZone()`)
3. Add case to `createObstacle(type)` switch statement
4. Define SPARQL query showing reasoning logic
5. Define Datalog rules + inference chain
6. Create hypergraph structure (nodes + hyperedges)

### Debugging Animation Issues

**Common bugs**:
- **Car doesn't move**: Check `stopPosition < carPosition` (car starts at -80, must move forward to positive stop value)
- **Car drives over obstacle**: Check crosswalk edge calculation (`+ 1` not `- 1`)
- **Wrong scenario displays**: Check `currentScenario` increment happens AFTER animation completes
- **Hypergraph not visible**: Ensure `scenario.hypergraph` exists and has nodes/hyperedges arrays

**Debug logging** (already in place):
```javascript
console.log('🚦 CROSSWALK MODE - Edge:', crosswalkEdge, 'StopPosition:', stopPosition);
console.log('🛑 Car STOPPED at:', carPosition, 'front at:', carPosition + 2.5);
```

---

## rust-kgdb Integration

This project is a **client** of the parent rust-kgdb repository. Key integration points:

### Dependencies (Cargo.toml)
```toml
[workspace.dependencies]
rdf-model = { path = "../crates/rdf-model" }
storage = { path = "../crates/storage", default-features = false }
sparql = { path = "../crates/sparql" }
hypergraph = { path = "../crates/hypergraph" }
rdf-io = { path = "../crates/rdf-io" }
```

**IMPORTANT**: These paths point to `../crates/` (parent directory). Do NOT copy rust-kgdb code into this workspace.

### Performance Characteristics (from rust-kgdb)
- **Lookup Speed**: 2.78 µs (35-180x faster than RDFox)
- **Memory**: 24 bytes/triple (25% more efficient than RDFox)
- **SPARQL Functions**: 64 builtin functions (exceeds Jena/RDFox)

### Hypergraph Usage
```rust
use hypergraph::Hyperedge;

// Create 4-way hyperedge (vehicle + pedestrian + crosswalk + safety standard)
let critical_situation = Hyperedge::new(
    vec![vehicle_node, pedestrian_node, crosswalk_node, iso26262_node],
    "CriticalSituation",
    properties
);
```

This demonstrates the **key innovation**: capturing multi-entity context atomically instead of requiring 10+ RDF triples + joins.

---

## Documentation

- **README.md**: Project overview, quick start, comparison with neural networks
- **HYPERGRAPH_REASONING_ARCHITECTURE.md** (31KB): Complete technical documentation
  - rust-kgdb certification & benchmarks
  - Event-driven reasoning architecture
  - 3 scenario deep dives (SPARQL + Datalog + Hypergraphs)
  - Legal auditability & explainable AI
  - Hypergraph vs traditional RDF comparison
- **SELF_DRIVING_REASONING_SPEC.md**: Original specification (38KB)
- **IMPLEMENTATION_SUMMARY.md**: Development progress log

---

## Key Technical Constraints

### Browser Caching
Safari/Chrome aggressively cache HTML files. **ALWAYS** force reload (`Command+Shift+R`) after changes, or use timestamped filenames (`DEMO_FINAL_1764165585.html`).

### 3D Coordinate System
- **X-axis**: Left (-80) to Right (+20), car travels forward (increasing x)
- **Y-axis**: Up/down (car at y=1, road at y=0)
- **Z-axis**: Perpendicular to road (car at z=0, traffic light at z=-7)

**Car starting position**: x=-80 (20m further back than original -60 for dramatic travel distance)

### Crosswalk Stripe Layout
Stripes are created in a loop: `for (let i = -8; i < 9; i += 2)`
- Stripe width: 2 units
- Total range: -8 to +8 (16 units wide)
- If `crosswalkStart = -43`, leftmost stripe center is at x = -43 - 8 = -51
- **Right edge** (car hits first): crosswalkStart + 1

---

## Version History

- **v5.0** (2025-11-26): All bugs fixed, hypergraphs in all scenarios, rust-kgdb certified
  - Fixed crosswalk edge calculation (was -1, now +1)
  - Fixed currentScenario increment timing
  - Added dynamic obstacle positioning
  - Increased travel distances for better demonstration
- **v1.0** (2025-11-26): Initial working demo with 3 scenarios

---

## Safety Standards Referenced

- **ISO 26262**: Road vehicles - Functional safety (2018)
- **SAE J3016**: Taxonomy for driving automation systems (2021)
- **UL 4600**: Safety evaluation of autonomous products (2020)

All scenarios demonstrate compliance through explicit reasoning chains suitable for regulatory audit.
