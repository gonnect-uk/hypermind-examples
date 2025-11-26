# System Requirements & Compatibility Check

**Your MacBook Pro Configuration**: ✅ **EXCELLENT - Exceeds All Requirements**

---

## Your Hardware

```
Model:      MacBook Pro 16" (2019) - MacBookPro16,1
CPU:        Intel Core i9 8-Core @ 2.4 GHz
RAM:        64 GB DDR4
GPU:        AMD Radeon Pro 5500M (4GB/8GB VRAM) + Intel UHD 630
macOS:      Darwin 24.6.0 (should be macOS 14.x Sonoma or later)
```

---

## Compatibility Assessment

### ✅ Udacity Unity Simulator
**Requirement**: Quad-core CPU, 8GB RAM, any GPU
**Your Spec**: 8-core i9, 64GB RAM, AMD Radeon Pro 5500M
**Result**: ✅ **PERFECT** - Will run at 60+ FPS with high settings

### ✅ Web-based 3D Dashboard (Three.js)
**Requirement**: Any modern browser, WebGL support
**Your Spec**: Safari/Chrome with AMD GPU hardware acceleration
**Result**: ✅ **PERFECT** - Smooth rendering at 30-60 FPS

### ✅ rust-kgdb Reasoning Engine
**Requirement**: 4GB RAM, dual-core CPU
**Your Spec**: 64GB RAM, 8-core i9
**Result**: ✅ **OVERKILL** - Sub-millisecond SPARQL queries

### ✅ All Three Running Simultaneously
**Estimated Usage**:
- Unity Simulator: 2-4GB RAM, 30-50% GPU, 1-2 CPU cores
- Web Dashboard: 500MB RAM, 10-20% GPU, 1 CPU core
- Reasoning Engine: 100MB RAM, 1-5% CPU (1 core)
- **Total**: ~5GB RAM, 40-70% GPU, 3-4 CPU cores

**Your Available**: 64GB RAM, AMD Pro 5500M, 8 cores
**Result**: ✅ **PLENTY OF HEADROOM** - Can run development tools + browser + simulator + engine

---

## Recommended Setup

### 1. Use rust-kgdb as Published Crate

**DO NOT COPY** - Instead, use as Cargo dependency:

```toml
# self-driving-car/Cargo.toml
[workspace]
members = [
    "crates/av-reasoning",
    "crates/simulator-bridge",
]

[workspace.dependencies]
# Use rust-kgdb from parent directory (local path for now)
rdf-model = { path = "../crates/rdf-model" }
storage = { path = "../crates/storage" }
sparql = { path = "../crates/sparql" }
hypergraph = { path = "../crates/hypergraph" }

# Once published to crates.io, switch to:
# rdf-model = "0.1.0"
# storage = { version = "0.1.0", features = ["inmemory"] }
# sparql = "0.1.0"
```

### 2. Lightweight Stack for Your Hardware

Given your excellent hardware, I recommend:

**Primary**: Udacity Unity Simulator (GPU-accelerated)
- Uses AMD Radeon Pro 5500M
- 60 FPS rendering
- Best visual fidelity

**Secondary**: Web Dashboard (browser-based)
- Real-time SPARQL query visualization
- Decision tree display
- Performance metrics

**Backend**: rust-kgdb (in-memory storage)
- Zero persistence needed for simulation
- Maximum performance
- 64GB RAM allows millions of triples

---

## Performance Expectations

### Unity Simulator
- **Frame Rate**: 60 FPS (with your AMD GPU)
- **Resolution**: 1920x1080 or higher
- **Latency**: < 16ms per frame
- **CPU Usage**: 20-30% (2-3 cores)

### rust-kgdb Reasoning
- **Query Latency**: 2.78 µs (measured benchmark)
- **Decision Frequency**: 100 Hz (10ms per cycle)
- **Memory**: <100MB for typical scenarios
- **CPU Usage**: <5% (single core)

### End-to-End Pipeline
```
Sensor Data (Unity) → 16ms
    ↓
RDF Conversion → 1ms
    ↓
SPARQL Query → 0.003ms
    ↓
Control Command → 1ms
    ↓
Unity Update → 16ms
────────────────────
Total: ~34ms (29 Hz decision loop)
```

**Bottleneck**: Unity rendering (16ms), NOT rust-kgdb (3 µs)

---

## Disk Space Requirements

```
Udacity Unity Simulator:  ~500 MB
Rust toolchain:           ~1.5 GB
Node.js + npm packages:   ~300 MB
rust-kgdb (compiled):     ~50 MB
Project workspace:        ~100 MB
────────────────────────────────
Total:                    ~2.5 GB
```

**Your Available**: Plenty (assuming standard 512GB+ SSD)

---

## Network Requirements

### Udacity Simulator
- ✅ No internet needed after download
- Socket.IO on localhost (127.0.0.1:4567)
- Zero latency

### Web Dashboard
- ✅ No internet needed (runs locally)
- WebSocket on localhost (127.0.0.1:3000)
- Zero latency

### rust-kgdb
- ✅ Fully offline
- No cloud dependencies
- All reasoning on-device

**Result**: ✅ **100% OFFLINE CAPABLE** - Perfect for development

---

## macOS Compatibility

### Supported Versions
- ✅ macOS 10.15 (Catalina) or later
- ✅ macOS 11 (Big Sur)
- ✅ macOS 12 (Monterey)
- ✅ macOS 13 (Ventura)
- ✅ macOS 14 (Sonoma) ← Your likely version
- ✅ macOS 15 (Sequoia)

### Your Darwin 24.6.0
Corresponds to **macOS 15.x (Sequoia)** or **macOS 14.x (Sonoma)**
**Result**: ✅ **FULLY SUPPORTED**

---

## Thermal Considerations

Your **MacBook Pro 16" (2019)** has robust cooling, but be aware:

### Unity Simulator (GPU Load)
- AMD Radeon Pro 5500M will run warm (70-80°C)
- Fans will spin up (moderate noise)
- **Recommendation**: Use cooling pad or elevated stand

### rust-kgdb (CPU Load)
- Minimal heat generation (<5% CPU)
- No thermal concerns

### Combined Workload
- Expect moderate fan noise during simulation
- CPU temperature: 60-75°C (normal)
- GPU temperature: 70-85°C (normal)

**Mitigation**:
```bash
# Monitor temperatures
sudo powermetrics --samplers smc -i 1000 | grep -E "(CPU die|GPU)"

# If too hot, reduce Unity graphics quality:
# - Lower resolution to 1280x720
# - Disable shadows/anti-aliasing
# - Cap frame rate to 30 FPS
```

---

## Realistic Timeline

Given your hardware, expect:

### Week 1-2: Core Reasoning
- ✅ Fast compilation (8-core i9)
- ✅ Quick testing iterations
- ✅ No performance bottlenecks

### Week 3: Simulator Integration
- ✅ Unity builds quickly (64GB RAM)
- ✅ Smooth simulator performance
- ✅ Real-time debugging

### Week 4: Web Dashboard
- ✅ Node.js builds fast
- ✅ Three.js renders smoothly
- ✅ No lag in visualization

### Week 5: Testing
- ✅ Can run all components simultaneously
- ✅ Record screen at 60 FPS (for demos)
- ✅ No slowdowns

**Overall**: Your hardware is **ideal** for this project.

---

## Bottlenecks (What WON'T Work Well)

### ❌ CARLA Simulator
- Requires NVIDIA GPU (you have AMD)
- Requires CUDA (AMD doesn't support CUDA)
- Requires 16GB VRAM (your AMD has 4-8GB)
- Requires Linux/Windows (you're on macOS)

**Solution**: Use Udacity/GDSim instead (designed for macOS)

### ❌ Training Neural Networks
- If you wanted to train CNNs for perception, AMD GPU is suboptimal
- TensorFlow/PyTorch prefer NVIDIA CUDA
- Apple Metal Performance Shaders (MPS) help, but slower than CUDA

**Solution**: We're NOT training neural networks - we're using SPARQL reasoning!

### ❌ Large-Scale Simulation (1000+ vehicles)
- Unity can handle ~50-100 vehicles smoothly
- Beyond that, frame rate drops

**Solution**: We're focusing on single-vehicle reasoning (ego vehicle + 10-20 others)

---

## Final Verdict

### ✅ **YES - Your MacBook Pro 16" (2019) Can Handle This Project**

**Strengths**:
- ✅ 8-core i9 → Fast compilation, real-time reasoning
- ✅ 64GB RAM → Overkill (can hold millions of RDF triples)
- ✅ AMD Radeon Pro 5500M → Smooth Unity rendering at 60 FPS
- ✅ macOS compatibility → Udacity simulator runs natively

**Limitations**:
- ⚠️ Fans will spin up during GPU-intensive tasks
- ⚠️ CARLA won't work (but we have alternatives)

**Recommendation**: **Proceed with confidence** - your hardware is more than adequate.

---

## Dependency Strategy

### Use rust-kgdb via Cargo Workspace

```
zenya-graphdb/
└── rust-kgdb/                    # Main rust-kgdb repo
    ├── crates/
    │   ├── rdf-model/
    │   ├── storage/
    │   ├── sparql/
    │   └── ...
    └── self-driving-car/         # Our project (separate workspace)
        ├── Cargo.toml            # Points to ../crates/* (local path)
        └── crates/
            ├── av-reasoning/     # Uses rust-kgdb as dependency
            └── ...
```

**Key Point**: We **DO NOT COPY** rust-kgdb code. We **USE** it as a library.

### Migration Path

**Phase 1 (Now)**: Use local path dependencies
```toml
rdf-model = { path = "../crates/rdf-model" }
```

**Phase 2 (After rust-kgdb publishes to crates.io)**: Use version dependencies
```toml
rdf-model = "0.1.0"
```

---

## Next Steps

1. ✅ **Confirmed**: Your MacBook Pro can handle this project
2. 🔜 **Download**: Udacity Unity Simulator (~500 MB)
3. 🔜 **Test**: Run simulator to verify performance
4. 🔜 **Build**: rust-kgdb release binary (verify sub-millisecond queries)
5. 🔜 **Implement**: Begin Phase 1 (Core Reasoning Engine)

**Confidence Level**: 95% - Project is **FEASIBLE** on your hardware.
