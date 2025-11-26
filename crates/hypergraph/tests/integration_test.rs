// Integration Test Suite for rust-kgdb Hypergraph Crate
//
// Comprehensive test coverage for hypergraph operations:
// - Basic node/edge creation and retrieval
// - Multi-node connections (arbitrary arity)
// - Traversal and pathfinding
// - Pattern matching (SPARQL BGP)
// - Subgraph extraction
// - Statistics and metadata
//
// Run all tests: cargo test -p hypergraph
// Run specific test: cargo test -p hypergraph test_name

mod basic_operations;
mod multi_node_connections;
mod traversal;
mod pattern_matching;
mod subgraph_extraction;
mod statistics_metadata;
mod edge_cases;
mod rdf_integration;
mod traits;
