# Developer Documentation

**Internal documentation for rust-kgdb contributors and maintainers.**

---

## Quick Navigation

### 🤝 [Contributing](contributing/)
- **[CODE_STANDARDS.md](contributing/CODE_STANDARDS.md)** - Coding conventions and style
- **[TESTING_GUIDE.md](contributing/TESTING_GUIDE.md)** - Test requirements and best practices
- **[PR_CHECKLIST.md](contributing/PR_CHECKLIST.md)** - Pull request review criteria

### 📱 [Mobile Development](mobile/)
- **[IOS_BUILD.md](mobile/IOS_BUILD.md)** - iOS XCFramework build process
- **[ANDROID_BUILD.md](mobile/ANDROID_BUILD.md)** - Android AAR build process
- **[UNIFFI_GUIDE.md](mobile/UNIFFI_GUIDE.md)** - UniFFI 0.30 custom CLI usage

### 🔧 [Implementation Guides](implementation/)
- **[ADDING_SPARQL_FUNCTIONS.md](implementation/ADDING_SPARQL_FUNCTIONS.md)** - Extend SPARQL with custom functions
- **[ADDING_STORAGE_BACKEND.md](implementation/ADDING_STORAGE_BACKEND.md)** - Implement new storage backends
- **[PARSER_DEVELOPMENT.md](implementation/PARSER_DEVELOPMENT.md)** - Pest grammar development

### 🐛 [Troubleshooting](troubleshooting/)
- **[BUILD_ISSUES.md](troubleshooting/BUILD_ISSUES.md)** - Common build failures and solutions
- **[TEST_FAILURES.md](troubleshooting/TEST_FAILURES.md)** - Debugging test failures
- **[PLATFORM_SPECIFIC.md](troubleshooting/PLATFORM_SPECIFIC.md)** - iOS/Android/Desktop issues

---

## Development Workflow

### 1. Setup
```bash
# Clone and build
git clone https://github.com/gonnect-uk/rust-kgdb
cd rust-kgdb
cargo build --workspace

# Install iOS targets (optional)
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
```

### 2. Make Changes
- Follow [Code Standards](contributing/CODE_STANDARDS.md)
- Add tests (see [Testing Guide](contributing/TESTING_GUIDE.md))
- Run `cargo fmt` and `cargo clippy`

### 3. Test
```bash
# Run all tests
cargo test --workspace

# Run W3C conformance tests
cargo test --test w3c_conformance -- --ignored

# Run benchmarks
cargo bench --workspace
```

### 4. Submit PR
- Review [PR Checklist](contributing/PR_CHECKLIST.md)
- Ensure 100% test pass rate
- Update documentation as needed

---

## Key Principles

### Zero Compromises
Every feature must work perfectly:
- No stubs or mocks in production code
- Complete W3C compliance
- Production-grade error handling
- Comprehensive test coverage

### Memory Safety First
- Minimize `unsafe` code
- Document safety invariants when `unsafe` is necessary
- Zero allocations in hot paths where possible
- Use Rust's borrow checker to enforce correctness

### Performance Obsession
- Zero-copy semantics throughout
- String interning via Dictionary
- Aggressive compiler optimization (LTO, opt-level=3)
- Benchmark every change

### Mobile-First Design
- <100ms cold start (vs 2-5s for JVM)
- <20MB for 100K triples
- Battery-efficient algorithms
- Offline-capable

---

## Workspace Structure

```
crates/
├── rdf-model/             # Core RDF types
│   ├── Node, Triple, Quad
│   ├── Dictionary (string interning)
│   └── Vocabulary constants
├── storage/               # Pluggable backends
│   ├── InMemoryBackend (default)
│   ├── RocksDBBackend (feature-gated)
│   └── LMDBBackend (feature-gated)
├── sparql/                # SPARQL 1.1 engine
│   ├── Parser (pest PEG)
│   ├── Executor (zero-copy)
│   └── 64 builtin functions
├── rdf-io/                # RDF parsers
│   ├── Turtle, N-Triples
│   ├── RDF/XML, JSON-LD
│   └── N-Quads, TriG
├── reasoning/             # RDFS, OWL 2 RL
├── hypergraph/            # Native hypergraph model
├── shacl/                 # W3C SHACL validation
├── prov/                  # W3C PROV provenance
├── wcoj/                  # Worst-case optimal joins
├── datalog/               # Datalog engine
└── mobile-ffi/            # iOS/Android FFI
    ├── lib.rs (FFI types)
    └── bin/uniffi-bindgen.rs (custom CLI)
```

---

## Common Tasks

### Add SPARQL Function
See: [implementation/ADDING_SPARQL_FUNCTIONS.md](implementation/ADDING_SPARQL_FUNCTIONS.md)

1. Add enum variant to `BuiltinFunction` (`sparql/src/algebra.rs`)
2. Implement in `eval_builtin()` (`sparql/src/executor.rs`)
3. Add parser support (`sparql/src/parser.rs`)
4. Add tests (`sparql/tests/`)

### Add Storage Backend
See: [implementation/ADDING_STORAGE_BACKEND.md](implementation/ADDING_STORAGE_BACKEND.md)

1. Implement `StorageBackend` trait (`storage/src/backend.rs`)
2. Add feature flag (`storage/Cargo.toml`)
3. Conditional compilation (`storage/src/lib.rs`)
4. Add tests (`storage/tests/`)

### Debug Performance
```bash
# Flamegraph profiling
cargo install flamegraph
cargo flamegraph --bench triple_store_benchmark

# Criterion benchmarks
cargo bench --package storage --bench triple_store_benchmark

# Profile with perf (Linux)
perf record --call-graph=dwarf ./target/release/bench
perf report
```

---

## Testing Requirements

### Unit Tests
- Every public API must have tests
- Test edge cases and error conditions
- Property-based testing with proptest for core types

### Integration Tests
- W3C SPARQL 1.1 conformance tests
- Apache Jena compatibility tests
- W3C RDF 1.2 conformance tests

### Benchmarks
- Criterion for micro-benchmarks
- LUBM and SP2Bench for macro-benchmarks
- Compare against RDFox and Jena baselines

### Test Pass Rate
**100% required** - No merging with failing tests

---

## Release Process

### Version Bumping
```toml
# Cargo.toml
[workspace.package]
version = "0.2.0"  # Increment here
```

### Pre-Release Checklist
- [ ] All 521 tests passing
- [ ] Benchmarks run successfully
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped

### Release
```bash
# Tag release
git tag -a v0.2.0 -m "Version 0.2.0"
git push origin v0.2.0

# Publish to crates.io (if applicable)
cargo publish --package rdf-model
# ... (publish in dependency order)
```

---

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Lookup | <3 µs | 2.78 µs | ✅ **BEAT** |
| Bulk Insert | >200K/sec | 146K/sec | ⚠️ 73% (optimizing) |
| Memory | <25 bytes/triple | 24 bytes | ✅ **BEAT** |
| Cold Start | <100 ms | <100 ms | ✅ **MEET** |
| Mobile Memory | <20 MB (100K triples) | <20 MB | ✅ **MEET** |

---

## Support

- **Internal Slack**: #rust-kgdb-dev
- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For design discussions
