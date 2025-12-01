# SDK Folder Structure - Consistent Organization

**Last Updated**: 2025-11-29

This document defines the **canonical folder structure** for all SDKs in the rust-kgdb project.

---

## ✅ Consistent SDK Structure

All SDKs follow this standardized pattern:

```
sdks/
├── python/              # Python SDK (UniFFI 0.30)
│   ├── rust_kgdb_py/   # Package directory
│   │   ├── __init__.py
│   │   └── gonnect.py  # UniFFI generated bindings
│   ├── tests/          # Test directory (root level)
│   │   └── test_regression.py
│   ├── docs/           # SDK documentation
│   ├── dist/           # Built packages
│   ├── setup.py
│   ├── pyproject.toml
│   └── README.md
│
├── typescript/         # TypeScript SDK (NAPI-RS)
│   ├── native/
│   │   └── rust-kgdb-napi/  # NAPI-RS Rust crate
│   │       ├── src/
│   │       │   └── lib.rs
│   │       ├── Cargo.toml
│   │       └── build.rs
│   ├── src/            # TypeScript source (if needed)
│   ├── tests/          # Test directory (root level)
│   │   └── regression.test.ts
│   ├── docs/           # SDK documentation
│   ├── index.js
│   ├── index.d.ts
│   ├── package.json
│   └── README.md
│
├── kotlin/             # Kotlin SDK (UniFFI 0.30)
│   ├── src/
│   │   ├── main/kotlin/
│   │   │   └── uniffi/gonnect/
│   │   │       └── gonnect.kt  # UniFFI generated
│   │   └── test/kotlin/        # Test directory (Gradle convention)
│   │       └── direct/
│   │           └── DirectBindingsTest.kt
│   ├── docs/           # SDK documentation
│   ├── build.gradle.kts
│   ├── settings.gradle.kts
│   └── README.md
│
└── rust/               # Rust SDK (native, no bindings needed)
    ├── src/
    ├── tests/
    ├── examples/
    ├── benches/
    └── README.md
```

---

## 📋 Key Principles

### 1. Test Location Consistency

**Rule**: Tests should be at SDK root level (except Kotlin which uses Gradle conventions)

- ✅ `sdks/python/tests/` - Python tests at root
- ✅ `sdks/typescript/tests/` - TypeScript tests at root
- ✅ `sdks/kotlin/src/test/kotlin/` - Kotlin tests follow Gradle convention
- ❌ `sdks/python/rust_kgdb_py/tests/` - WRONG (nested in package)

**Why**:
- Easier to find and run tests
- Separates test dependencies from package dependencies
- Consistent across all SDKs (except language-specific conventions like Gradle)

### 2. Documentation Consistency

**Rule**: Each SDK has its own `docs/` directory at root level

- ✅ `sdks/python/docs/` - Python-specific docs
- ✅ `sdks/typescript/docs/` - TypeScript-specific docs
- ✅ `sdks/kotlin/docs/` - Kotlin-specific docs

**Why**: SDK-specific documentation separate from package code

### 3. No Redundant Directories

**Rule**: No nested `sdks/` directories within SDK folders

- ❌ `sdks/python/sdks/typescript/` - WRONG (redundant, removed)
- ✅ `sdks/python/` and `sdks/typescript/` - CORRECT (siblings)

**Why**: Clear separation of concerns, no nesting confusion

### 4. Built Artifacts Location

**Rule**: Built packages go in SDK-specific directories

- ✅ `sdks/python/dist/` - Python built packages (.tar.gz, .whl)
- ✅ `sdks/typescript/node_modules/` - npm dependencies
- ✅ `sdks/kotlin/build/` - Gradle build outputs

**Why**: Isolate build artifacts per SDK

---

## 📂 Complete Directory Listing

### Python SDK (`sdks/python/`)

```
python/
├── rust_kgdb_py/           # Package directory
│   ├── __init__.py         # Public API exports
│   └── gonnect.py          # UniFFI generated (77KB)
├── tests/                  # ✅ Tests at root level
│   └── test_regression.py  # 29 regression tests
├── docs/                   # SDK documentation
├── dist/                   # Built packages
│   └── rust_kgdb-0.1.3.tar.gz
├── rust_kgdb.egg-info/     # Build metadata
├── setup.py                # PyPI packaging
├── pyproject.toml          # Modern Python packaging
├── MANIFEST.in             # Package includes
└── README.md               # SDK README
```

**Test Command**: `python3 -m pytest tests/test_regression.py -v`

### TypeScript SDK (`sdks/typescript/`)

```
typescript/
├── native/
│   └── rust-kgdb-napi/     # NAPI-RS crate
│       ├── src/
│       │   └── lib.rs      # Complete GraphDB implementation
│       ├── Cargo.toml
│       └── build.rs
├── src/                    # TypeScript source (optional)
├── tests/                  # ✅ Tests at root level
│   └── regression.test.ts  # 28 regression tests
├── docs/                   # SDK documentation
├── index.js                # Entry point
├── index.d.ts              # TypeScript types
├── package.json            # npm configuration
└── README.md               # SDK README
```

**Test Command**: `npm test`

### Kotlin SDK (`sdks/kotlin/`)

```
kotlin/
├── src/
│   ├── main/kotlin/
│   │   └── uniffi/gonnect/
│   │       └── gonnect.kt          # UniFFI generated (81KB)
│   └── test/kotlin/                # ✅ Tests follow Gradle convention
│       └── direct/
│           └── DirectBindingsTest.kt  # 5 tests
├── build/                          # Gradle build directory
├── docs/                           # SDK documentation
├── gradle/                         # Gradle wrapper
├── build.gradle.kts                # Gradle build config
├── settings.gradle.kts             # Gradle settings
└── README.md                       # SDK README
```

**Test Command**: `./gradlew test`

### Rust SDK (`sdks/rust/`)

```
rust/
├── src/                    # Rust library source
├── tests/                  # Integration tests
├── examples/               # Example programs
├── benches/                # Benchmarks
├── Cargo.toml              # Crate configuration
└── README.md               # SDK README
```

**Test Command**: `cargo test`

---

## 🔧 Cleanup Actions Performed

### Removed Redundant Directories

1. ❌ **Removed**: `sdks/python/sdks/typescript/`
   - **Reason**: Redundant nesting - TypeScript SDK already exists at `sdks/typescript/`
   - **Action**: `rm -rf sdks/python/sdks`

2. ✅ **Verified**: No other redundant `sdks/` directories exist

### Verified Test Locations

1. ✅ **Python**: Tests correctly at `sdks/python/tests/test_regression.py`
2. ✅ **TypeScript**: Tests correctly at `sdks/typescript/tests/regression.test.ts`
3. ✅ **Kotlin**: Tests correctly at `sdks/kotlin/src/test/kotlin/direct/DirectBindingsTest.kt`

---

## 📊 SDK Test Summary

| SDK | Test Location | Test Count | Test Command |
|-----|--------------|------------|--------------|
| **Python** | `tests/test_regression.py` | 29 tests | `pytest tests/` |
| **TypeScript** | `tests/regression.test.ts` | 28 tests | `npm test` |
| **Kotlin** | `src/test/kotlin/direct/` | 5 tests | `./gradlew test` |
| **Rust** | `tests/` | 61 tests | `cargo test` |

---

## 🎯 Future SDK Guidelines

When adding new SDKs, follow this checklist:

- [ ] Create SDK directory at `sdks/NEW_SDK/`
- [ ] Place tests at `sdks/NEW_SDK/tests/` (unless language convention differs)
- [ ] Add `docs/` directory for SDK-specific documentation
- [ ] Create `README.md` with SDK usage examples
- [ ] Add test command to this document
- [ ] Verify no redundant `sdks/` nesting

---

## Summary

**Consistent Structure Achieved**: ✅
- All SDKs have tests at consistent locations
- No redundant `sdks/` directories
- Documentation organized per SDK
- Build artifacts isolated

**Test Discovery**:
- Python: `sdks/python/tests/`
- TypeScript: `sdks/typescript/tests/`
- Kotlin: `sdks/kotlin/src/test/kotlin/` (Gradle convention)
- Rust: `sdks/rust/tests/`

This structure ensures:
1. Easy navigation across all SDKs
2. Consistent test execution patterns
3. Clear separation of concerns
4. No redundancy or confusion
