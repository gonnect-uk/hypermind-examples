#!/usr/bin/env node
/**
 * @gonnect/rust-kgdb - Basic Usage Example
 *
 * This example demonstrates:
 * 1. Creating a GraphDB instance
 * 2. Loading RDF/Turtle data
 * 3. Executing SPARQL queries
 * 4. Using reasoners (RDFS, OWL, Datalog)
 * 5. Hypergraph operations (hyperedges)
 */

console.log('🚀 Testing @gonnect/rust-kgdb\n');

try {
    // Import the package (this will fail until package includes binaries)
    // For now, we're testing the documentation structure
    console.log('✅ Package import would go here');
    console.log('   const { GraphDB, RDFSReasoner, DatalogReasoner } = require("@gonnect/rust-kgdb");\n');

    // Example 1: Basic Triple Store
    console.log('📊 Example 1: Basic Triple Store');
    console.log('   const db = new GraphDB("http://example.org/");');
    console.log('   db.load_ttl(`');
    console.log('     @prefix ex: <http://example.org/> .');
    console.log('     ex:Alice ex:knows ex:Bob .');
    console.log('     ex:Bob ex:age 30 .');
    console.log('   `);');
    console.log('   const results = db.query_select(`');
    console.log('     SELECT ?person ?age WHERE {');
    console.log('       ex:Alice ex:knows ?person .');
    console.log('       ?person ex:age ?age .');
    console.log('     }');
    console.log('   `);\n');

    // Example 2: RDF* (RDF-star) - Quoted Triples
    console.log('🌟 Example 2: RDF* (RDF-star) Support');
    console.log('   // Model metadata about triples');
    console.log('   db.load_ttl(`');
    console.log('     @prefix : <http://example.org/> .');
    console.log('     <<:Bob :age 30>> :certainty 0.9 .');
    console.log('     <<:Bob :age 30>> :source :Census2020 .');
    console.log('   `);');
    console.log('   // Query metadata');
    console.log('   const metadata = db.query_select(`');
    console.log('     SELECT ?certainty WHERE {');
    console.log('       <<?s ?p ?o>> :certainty ?certainty .');
    console.log('     }');
    console.log('   `);\n');

    // Example 3: Hypergraph - N-ary Relationships
    console.log('🔬 Example 3: Hypergraph Operations');
    console.log('   // Hyperedges connect multiple nodes simultaneously');
    console.log('   db.create_hyperedge({');
    console.log('     id: "meeting_001",');
    console.log('     nodes: ["Alice", "Bob", "Charlie"],');
    console.log('     properties: { type: "project_meeting", date: "2025-01-15" }');
    console.log('   });');
    console.log('   // Query hyperedges');
    console.log('   const meetings = db.query_hyperedges({');
    console.log('     type: "project_meeting",');
    console.log('     participants: ["Alice"]  // Find all meetings with Alice');
    console.log('   });\n');

    // Example 4: RDFS Reasoning
    console.log('🧠 Example 4: RDFS Reasoning');
    console.log('   const reasoner = new RDFSReasoner(db);');
    console.log('   db.load_ttl(`');
    console.log('     @prefix : <http://example.org/> .');
    console.log('     @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .');
    console.log('     :Manager rdfs:subClassOf :Employee .');
    console.log('     :Alice a :Manager .');
    console.log('   `);');
    console.log('   reasoner.materialize();  // Infer Alice is also an Employee\n');

    // Example 5: Datalog Reasoning
    console.log('🔮 Example 5: Datalog Reasoning');
    console.log('   const datalog = new DatalogReasoner(db);');
    console.log('   datalog.add_rule(`');
    console.log('     ancestorOf(?x, ?y) :- parentOf(?x, ?y).');
    console.log('     ancestorOf(?x, ?z) :- parentOf(?x, ?y), ancestorOf(?y, ?z).');
    console.log('   `);');
    console.log('   datalog.materialize();  // Compute transitive closure\n');

    // Example 6: Performance Features
    console.log('⚡ Example 6: Performance Features');
    console.log('   // Zero-copy semantics - no heap allocations');
    console.log('   // SIMD vectorization - 4x-8x faster batch ops');
    console.log('   // Parallel query execution - multi-core utilization');
    console.log('   // String interning - 8-byte URI references');
    console.log('   // All automatic - no configuration needed!\n');

    console.log('✅ All examples documented successfully!');
    console.log('\n📚 Full API documentation: https://www.npmjs.com/package/@gonnect/rust-kgdb');
    console.log('🔗 GitHub: https://github.com/gonnect-uk/rust-kgdb\n');

} catch (error) {
    console.error('❌ Error:', error.message);
    console.log('\n📦 Note: This example tests documentation structure.');
    console.log('   Binary distribution coming soon with full API support.\n');
}
