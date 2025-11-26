# Documentation Index

This directory contains all project documentation organized by category.

## Structure

```
docs/
├── benchmarks/           # Performance benchmarks and comparisons
├── session-reports/      # Development session summaries
└── archive/              # Historical/superseded documentation
```

## Quick Links

### Current Documentation (Root)
- **[README.md](../README.md)** - Main project overview and quick start
- **[CLAUDE.md](../CLAUDE.md)** - Guide for Claude Code AI assistant
- **[ARCHITECTURE_SPEC.md](../ARCHITECTURE_SPEC.md)** - Detailed technical architecture
- **[ACCEPTANCE_CRITERIA.md](../ACCEPTANCE_CRITERIA.md)** - Apache Jena feature parity checklist

### Benchmarks
- **[BENCHMARK_RESULTS_REPORT.md](benchmarks/BENCHMARK_RESULTS_REPORT.md)** - Latest performance measurements (2025-11-18)
- **[HONEST_BENCHMARK_PLAN.md](benchmarks/HONEST_BENCHMARK_PLAN.md)** - 4-week optimization roadmap
- **[COMPLETE_FEATURE_COMPARISON.md](benchmarks/COMPLETE_FEATURE_COMPARISON.md)** - Rust KGDB vs Jena vs RDFox

### Test Porting & W3C Conformance
- **[COMPLETE_JENA_TEST_ANALYSIS.md](COMPLETE_JENA_TEST_ANALYSIS.md)** - Complete Apache Jena test suite analysis (2025-11-25)
- **[TEST_PORTING_QUICK_START.md](TEST_PORTING_QUICK_START.md)** - Week 1 action items and templates

### Session Reports
- **[SESSION_SUMMARY.md](session-reports/SESSION_SUMMARY.md)** - Latest development session
- **[TODAY_ACCOMPLISHMENTS.md](session-reports/TODAY_ACCOMPLISHMENTS.md)** - Daily progress log

### Archive
Historical documentation and superseded reports (see [archive/README.md](archive/README.md))

---

## Documentation Maintenance

### Adding New Documentation
- **Benchmarks**: Place in `docs/benchmarks/`
- **Session Reports**: Place in `docs/session-reports/`
- **Architecture**: Update root `ARCHITECTURE_SPEC.md` or `CLAUDE.md`
- **Old Documents**: Move to `docs/archive/` with date prefix

### Archiving Process
When a document becomes outdated:
1. Move to `docs/archive/`
2. Add date prefix: `YYYY-MM-DD_original_name.md`
3. Update this index if necessary

---

**Last Updated**: 2025-11-25
