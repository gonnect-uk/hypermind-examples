# 🚗 Self-Driving Car Demo - rust-kgdb

Production-grade self-driving car reasoning demo using **rust-kgdb v0.1.1** with real SPARQL query execution.

## ✨ Features

- ✅ **Real rust-kgdb backend** (2.78 µs lookups, 100% SPARQL 1.1/1.2)
- ✅ **Native hypergraph support** for complex reasoning
- ✅ **3D car animation** with Three.js
- ✅ **3 driving scenarios** (traffic light, pedestrian, school zone)
- ✅ **Sub-millisecond SPARQL queries**
- ✅ **Explainable AI decisions** with reasoning traces

---

## 🚀 Quick Start

```bash
# One command to start everything:
make demo
```

This will build, start the server, and open the demo in your browser!

---

## 📋 All Commands

```bash
make help         # Show all commands
make build        # Build av-server binary
make start        # Start server in background
make stop         # Stop server
make demo         # Start server + open demo (default)
make test         # Run SPARQL test queries
make health       # Check server health
make logs         # Show server logs
make clean        # Stop server and clean build
```

---

## 📍 URLs

- **Demo**: `file:///$(pwd)/DEMO_RUST_KGDB.html`
- **Backend**: `http://localhost:8080`
- **Health**: `http://localhost:8080/health`
- **Stats**: `http://localhost:8080/stats`

---

## 🔧 Manual Start

```bash
# Build
cd av-cli-standalone
cargo build --bin av-server --features server --release

# Start
../target/release/av-server

# Open demo (in another terminal)
open ../DEMO_RUST_KGDB.html
```

---

## 📊 Server API

```bash
# Load RDF data
curl -X POST http://localhost:8080/load \
  -H "Content-Type: application/json" \
  -d '{"turtle_data": "..."}'

# SPARQL ASK query
curl -X POST http://localhost:8080/ask \
  -H "Content-Type: application/json" \
  -d '{"sparql_query": "ASK WHERE { ?s ?p ?o }"}'

# SPARQL SELECT query
curl -X POST http://localhost:8080/select \
  -H "Content-Type: application/json" \
  -d '{"sparql_query": "SELECT * WHERE { ?s ?p ?o }"}'
```

---

## 🎯 Demo Scenarios

### 1. Red Traffic Light Emergency Stop
- **Speed**: 48 km/h
- **Distance**: 30m to red light
- **Query**: Check if traffic light is red
- **Decision**: **BRAKE 80%**

### 2. Pedestrian Crossing
- **Speed**: 36 km/h
- **Scenario**: Pedestrian in crosswalk at 45m
- **Query**: Detect pedestrian in crosswalk
- **Decision**: **EMERGENCY BRAKE**

### 3. School Zone Speeding
- **Speed**: 72 km/h (limit: 30 km/h)
- **Query**: Check speed vs school zone limit
- **Decision**: **SLOW DOWN 50%**

---

## 🐛 Troubleshooting

### Demo shows errors

1. **Hard refresh**: `Cmd+Shift+R` (Mac) or `Ctrl+Shift+R` (Windows)
2. Check server: `make health`
3. View logs: `make logs`

### Port 8080 in use

```bash
# Stop existing server
make stop

# Or kill manually
lsof -i :8080
kill -9 <PID>
```

---

## 📝 Known Limitations (v0.1.1)

**SPARQL Parser**: Must use full URIs in WHERE clauses:
- ✅ **Works**: `<http://example.org/property>`
- ❌ **Fails**: `ex:property` (prefix syntax not supported in patterns)

---

## 🚀 Performance

| Metric | Result |
|--------|--------|
| Triple lookup | 2.78 µs |
| SPARQL query | 1-3 ms |
| Bulk insert | 146K triples/sec |
| Memory/triple | 24 bytes |

---

**Built with ❤️ using Rust and SPARQL**
