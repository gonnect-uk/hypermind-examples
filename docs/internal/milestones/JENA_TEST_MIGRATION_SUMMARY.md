# Apache Jena Test Migration Plan - Executive Summary

**Document**: COMPREHENSIVE_JENA_TEST_MIGRATION_PLAN.md (2,015 lines)
**Date**: November 22, 2025
**Status**: Research & Planning Complete - Ready for Implementation

---

## Key Findings

### 1. Test Coverage Analysis

**Total W3C RDF/SPARQL/SHACL Tests Available**: 1,872+ tests

| Category | Tests | Status | Effort | Timeline |
|----------|-------|--------|--------|----------|
| **RDF Parsing** (6 formats) | 299 | 10% done | HIGH | 2-3 weeks |
| **SPARQL Query** (14 features) | 842 | 15% done | VERY HIGH | 3-4 weeks |
| **SPARQL Update** (graph ops) | 116 | 0% done | MEDIUM | 1 week |
| **SHACL Validation** (3 levels) | 247 | 0% done | HIGH | 1-1.5 weeks |
| **RDFS Reasoning** | 83 | 0% done | MEDIUM | 1 week |
| **OWL 2 RL Reasoning** | 135 | 0% done | VERY HIGH | 2-3 weeks |
| **OPTIONAL: OWL 2 EL/DL** | 85 | — | VERY HIGH | 2 weeks |
| **OPTIONAL: RDF-star** | 65 | — | MEDIUM | 1 week |

**Current Status**: ~112/1,872 tests passing (6%)

### 2. Critical Gaps (Must Fix First)

1. **RDF/XML Parser** (71 tests) - Complex XML handling required
2. **JSON-LD Parser** (89 tests) - Sophisticated context processing
3. **SPARQL Aggregates** (76 tests) - GROUP BY, HAVING clauses
4. **Property Paths** (93 tests) - Cycle detection, performance critical
5. **SHACL Validation** (247 tests) - Constraint engine needed
6. **Reasoning** (218 tests) - RDFS/OWL entailment rules

### 3. Implementation Phases

**Phase 1: Critical (6 weeks)**
- RDF Parsing: Turtle, N-Triples, RDF/XML, JSON-LD, N-Quads, TriG (299 tests)
- SPARQL Query: All major features (842 tests)
- SPARQL Update: INSERT/DELETE/Graph ops (116 tests)
- Result Formats: JSON/CSV/TSV (54 tests)
- SHACL Validation: Core & Advanced (247 tests)
- **Total**: 1,558 tests
- **Target**: 100% pass rate

**Phase 2: Essential (4 weeks)**
- RDFS Reasoning: Entailment rules (83 tests)
- OWL 2 RL: Ontology axioms (135 tests)
- **Total**: 218 tests
- **Target**: 90%+ pass rate

**Phase 3: Optional (4-5 weeks)**
- OWL 2 EL/DL profiles (85 tests)
- RDF-star/quoted triples (65 tests)
- Performance benchmarks

### 4. Test Infrastructure Required

**New Files to Create**:
- 18 test runner modules
- 3 W3C manifest parsers (Turtle, JSON-LD, RDF)
- 8 format-specific test suites
- Result comparison engine (RDF graph isomorphism)
- CI/CD integration (GitHub Actions)

**Test Data Volume**: ~700 MB (W3C repositories)
- https://github.com/w3c/rdf-tests (~500 MB)
- https://github.com/json-ld/json-ld.org (~200 MB)

**Dependencies to Add**:
- proptest, criterion (testing)
- tempfile, glob, walkdir (test discovery)
- reqwest, tokio (downloading test data)
- rayon (parallel execution)
- pretty_assertions, similar (test output)
- junit (CI/CD reporting)

### 5. Major Technical Challenges

| Challenge | Risk | Solution |
|-----------|------|----------|
| **Grammar Exactness** | HIGH | Weekly validation against W3C BNF |
| **RDF Graph Isomorphism** | MEDIUM | Implement proper comparison algorithm |
| **Property Path Cycles** | HIGH | Cycle detection, timeout handling |
| **Result Format Precision** | MEDIUM | Exact JSON/CSV/XML/Turtle serialization |
| **SHACL Constraint Interaction** | HIGH | Incremental testing per constraint type |
| **Test Execution Speed** | MEDIUM | Parallel execution with rayon |

### 6. Directory Structure Changes

```
crates/
├── rdf-io/
│   ├── src/
│   │   ├── lib.rs (existing)
│   │   ├── turtle.rs (enhance)
│   │   ├── ntriples.rs (enhance)
│   │   ├── rdfxml.rs (NEW - 71 tests)
│   │   ├── nquads.rs (NEW - 28 tests)
│   │   ├── trig.rs (NEW - 31 tests)
│   │   ├── jsonld.rs (NEW - 89 tests)
│   │   ├── w3c_test_runner.rs (NEW)
│   │   └── test_utils.rs (NEW)
│   └── tests/ (8 NEW test modules, ~299 tests)
│
├── sparql/
│   ├── src/
│   │   ├── test_runner.rs (NEW)
│   │   └── result_format.rs (NEW)
│   └── tests/ (11 NEW test modules, ~842 tests)
│
├── shacl/
│   ├── src/
│   │   └── test_runner.rs (NEW)
│   └── tests/ (3 NEW test modules, ~247 tests)
│
└── reasoning/
    ├── src/
    │   └── test_runner.rs (NEW)
    └── tests/ (2 NEW test modules, ~218 tests)

test-data/
├── w3c-rdf-tests/ (cloned)
├── w3c-jsonld-tests/ (cloned)
└── manifest-downloader.sh (NEW)
```

### 7. Recommended Implementation Order

**Week 1-2**: RDF Parsing Foundation
1. Enhance Turtle parser → 99 tests passing
2. Complete N-Triples parser → 40 tests passing
3. Build W3C manifest parser (reusable)

**Week 3-4**: Core SPARQL
4. Implement SPARQL filters & expressions → 97 tests
5. Implement aggregates & GROUP BY → 76 tests
6. Implement CONSTRUCT queries → 46 tests

**Week 5-6**: SPARQL Completion & SHACL
7. Implement property paths → 93 tests
8. Implement SPARQL update → 116 tests
9. Build SHACL validator → 247 tests
10. Implement result formats → 54 tests

**Week 7-10**: Reasoning & Advanced
11. Implement RDFS reasoning → 83 tests
12. Implement OWL 2 RL → 135 tests

### 8. Success Metrics

**Primary Success Criteria**:
- ✓ 1,558/1,558 Phase 1 tests passing (100%)
- ✓ 0 known failures
- ✓ <5 minute full test suite execution time
- ✓ CI/CD integration working

**Stretch Goals**:
- ✓ 218/218 Phase 2 tests passing (RDFS+OWL)
- ✓ Property path performance <100ms/query
- ✓ Memory footprint <1GB during testing
- ✓ Test execution parallelization working

### 9. Known Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Grammar doesn't match W3C | 40% | HIGH | Weekly spec review |
| RDF/XML parser bugs | 35% | MEDIUM | Early implementation |
| JSON-LD context processing | 40% | MEDIUM | Phased implementation |
| Property path performance | 30% | HIGH | Optimize algorithm |
| SHACL constraint bugs | 35% | HIGH | Constraint-by-constraint testing |
| Test execution too slow | 30% | MEDIUM | Parallel execution |

### 10. Certification Path

**Target**: W3C RDF/SPARQL Conformance Certification

```
Week 6:  Complete Phase 1 (1,558 tests)
         → Generate W3C test report
         → Submit to W3C test suite
         → Claim compliance

Week 10: Complete Phase 2 (218 tests)
         → Full RDFS+OWL support
         → Updated compliance report

Week 14: Complete Phase 3 (150 tests)
         → RDF-star support
         → OWL 2 EL/DL support
```

---

## Document Contents

The full 2,015-line document includes:

### Part 1: Test Glossary (600+ lines)
- Detailed breakdown of 1,872 tests by category
- Test count estimates for each subcategory
- W3C repository references
- Jena equivalent mappings

### Part 2: Detailed TODO List (800+ lines)
- 3 implementation phases with specific tasks
- Test count breakdown per phase
- Effort estimates (days/weeks)
- Blocker analysis
- Success criteria

### Part 3: Directory Structure Mapping (100+ lines)
- Jena → rust-kgdb file mapping
- Directory structure for new tests
- File organization strategy

### Part 4: Compliance Gap Analysis (150+ lines)
- 7 critical gaps identified
- Test coverage percentages
- Timeline to fix each gap

### Part 5: Crate Version Updates (50+ lines)
- Recommended dependency versions
- New dependencies needed
- Justification for each

### Part 6: Implementation Strategy (200+ lines)
- Recommended implementation order
- Major challenges & complexity
- Risk assessment matrix

### Part 7: Test Data Organization (100+ lines)
- Download URLs for all test suites
- File structure (700 MB total)
- Manifest format examples

### Part 8: CI/CD & Metrics (100+ lines)
- GitHub Actions pipeline template
- Test metrics dashboard
- Performance tracking

---

## Quick Reference

**Total Test Count**: 1,872 (Core: 1,410, Optional: 462)
**Current Coverage**: 6% (~112 tests)
**Gap**: 1,760 tests to implement
**Estimated Effort**: 14-18 weeks (full team)
**Critical Path**: RDF → SPARQL → SHACL → Reasoning
**File Size**: 2,015 lines (comprehensive markdown)

---

## Next Steps

1. **Review the full document** (2,015 lines) in COMPREHENSIVE_JENA_TEST_MIGRATION_PLAN.md
2. **Prioritize Phase 1 tasks** based on team capacity
3. **Setup CI/CD pipeline** for automated test reporting
4. **Download W3C test data** (700 MB)
5. **Begin RDF-IO enhancements** (Weeks 1-2)
6. **Track progress** using the provided metrics dashboard

---

**Status**: Document Complete - Ready for Implementation  
**Location**: `/rust-kgdb/COMPREHENSIVE_JENA_TEST_MIGRATION_PLAN.md`  
**Audience**: Architecture Review, Development Team Lead, Project Manager
