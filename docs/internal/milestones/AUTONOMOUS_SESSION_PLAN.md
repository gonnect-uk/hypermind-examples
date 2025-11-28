# Autonomous Session Plan - 2025-11-20

**User Goal**: See new generated apps in action on iPhone simulators
**Success Criteria**:
1. ✅ All 3 new apps deployed to simulators
2. ✅ GraphDBAdmin bugs resolved (graphs displayed, mobile-friendly tables)
3. ✅ TTL files bundled with apps for offline GraphDB loading
4. ✅ Apps showing queries, triples in proper mobile format
5. ✅ Universal-mobile-kg-engine supports structured + unstructured data

## Architecture Decision: TTL Data Management

### Problem
How to bundle RDF/TTL datasets with iOS apps for offline GraphDB usage?

### Solution: Bundle Resources Architecture
```
AppName.app/
├── AppName (binary)
├── Info.plist
├── Resources/
│   └── datasets/
│       ├── insurance-policies.ttl     # Domain-specific data
│       ├── compliance-rules.ttl       # Regulatory data
│       └── ontology.ttl              # Schema definitions
└── Frameworks/
    └── rust-kgdb.xcframework          # GraphDB engine
```

**Implementation**:
1. Generate Xcode project with Resources folder
2. Add TTL files to Copy Bundle Resources build phase
3. Load at runtime: `Bundle.main.url(forResource: "policies", withExtension: "ttl")`
4. GraphDB initializes from bundled data on first launch

### Best Practices
- **Offline-First**: All data bundled, no network required
- **Lazy Loading**: Load TTL only when needed
- **Incremental Updates**: Delta sync for updates
- **Compression**: Gzip TTL files to reduce app size

## Parallel Work Streams

### Stream 1: iOS App Deployment (High Priority)
1. Generate Xcode projects for 3 apps
2. Bundle TTL datasets
3. Build for simulator
4. Deploy to both iPhone 16e and 17 Pro
5. Test SPARQL queries in-app

### Stream 2: GraphDBAdmin Bug Fixes (High Priority)
**Bugs to Fix**:
1. Graph visualization not displaying
2. Query results not in mobile-friendly format
3. Triple display needs table structure

**Solution**:
- Use SwiftUI List with custom row views
- Add graph visualization using Swift Charts or custom Canvas
- Implement pagination for large result sets
- Add search/filter for triples

### Stream 3: Universal-Mobile-KG-Engine (Medium Priority)
**Structured Data Support**: ✅ DONE
- RDF/OWL ontologies
- SPARQL queries
- Triple stores

**Unstructured Data Support**: ⏳ PENDING
- Text documents (PDF, DOCX)
- Images with OCR
- Audio transcriptions
- Vector embeddings for semantic search
- Entity extraction from unstructured text

**Architecture**:
```rust
UniversalKGEngine {
    structured: RDFStore,      // Existing
    unstructured: VectorStore, // New
    ontology: UniversalOntology // Bridge
}
```

## Timeline (Autonomous Execution)

**Hour 1-2**: iOS App Deployment
- Generate Xcode projects
- Bundle TTL files
- Build and deploy to simulators

**Hour 2-3**: GraphDBAdmin Fixes
- Fix graph display
- Implement mobile-friendly tables
- Add pagination

**Hour 3-4**: Universal-Mobile-KG-Engine
- Add vector store integration
- Implement unstructured data ingestion
- Test with sample documents

**Hour 4-5**: Integration & Testing
- Test all apps on simulators
- Verify SPARQL queries work
- Validate graph displays
- Create demo video/screenshots

## Success Metrics
- [ ] 3 apps visible in iPhone simulator
- [ ] Can launch each app
- [ ] SPARQL queries return results
- [ ] Graphs display correctly
- [ ] Tables are mobile-friendly
- [ ] No crashes or errors
- [ ] Universal engine handles PDFs/docs

---

**Status**: Starting autonomous execution
**Expected Completion**: 4-5 hours
**Next Checkpoint**: Xcode project generation complete
