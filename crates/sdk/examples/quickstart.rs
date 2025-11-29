//! Quick start example for rust-kgdb-sdk

use rust_kgdb_sdk::{GraphDB, Node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("rust-kgdb SDK Quick Start");
    println!("==========================\n");

    // Create an in-memory graph database
    let mut db = GraphDB::in_memory();
    println!("✓ Created in-memory database");

    // Insert some RDF triples
    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Alice"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Bob"),
        )
        .execute()?;

    println!("✓ Inserted {} triples", db.count());

    // Query the data
    println!("\nQuerying for names:");
    let results = db
        .query()
        .sparql("SELECT ?name WHERE { ?person <http://xmlns.com/foaf/0.1/name> ?name }")
        .execute()?;

    for (i, binding) in results.iter().enumerate() {
        if let Some(name) = binding.get("name") {
            println!("  {}. {:?}", i + 1, name);
        }
    }

    println!("\n✓ Query returned {} results", results.len());
    println!("\nQuick start completed successfully!");

    Ok(())
}
