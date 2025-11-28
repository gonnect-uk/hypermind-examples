// Standalone test to verify GraphDB Admin TTL loading
use std::sync::Arc;
use rdf_model::{Dictionary, Quad, Node};
use rdf_io::TurtleParser;

fn main() {
    println!("Testing GraphDB Admin database-catalog.ttl parsing...\n");

    // Read the TTL file
    let ttl_path = "../../ios/GraphDBAdmin/GraphDBAdmin/Resources/datasets/database-catalog.ttl";
    let content = match std::fs::read_to_string(ttl_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Cannot read {}: {}", ttl_path, e);
            std::process::exit(1);
        }
    };

    println!("✓ Read {} bytes from {}", content.len(), ttl_path);

    // Create dictionary and parser
    let dictionary = Arc::new(Dictionary::new());
    let mut parser = TurtleParser::new(Arc::clone(&dictionary));

    println!("✓ Created parser");

    // Parse the TTL content
    println!("\nParsing Turtle content...");
    let quads = match parser.parse(&content) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("ERROR: Turtle parsing failed: {}", e);
            std::process::exit(1);
        }
    };

    println!("✓ Parsed {} quads", quads.len());

    // Show first 10 quads
    println!("\nFirst 10 quads:");
    for (i, quad) in quads.iter().take(10).enumerate() {
        println!("  {}. S: {:?}", i+1, node_to_string(&quad.subject));
        println!("     P: {:?}", node_to_string(&quad.predicate));
        println!("     O: {:?}", node_to_string(&quad.object));
        if let Some(g) = &quad.graph {
            println!("     G: {:?}", node_to_string(g));
        }
        println!();
    }

    // Count unique subjects (entities)
    let mut subjects = std::collections::HashSet::new();
    for quad in &quads {
        subjects.insert(node_to_string(&quad.subject));
    }

    println!("\n=== STATISTICS ===");
    println!("Total triples: {}", quads.len());
    println!("Unique entities (subjects): {}", subjects.len());
    println!("Dictionary size: {}", dictionary.len());

    if quads.len() > 0 {
        println!("\n✅ SUCCESS: database-catalog.ttl can be parsed!");
    } else {
        println!("\n⚠️  WARNING: 0 triples parsed - check file content");
    }
}

fn node_to_string(node: &Node) -> String {
    match node {
        Node::IRI(iri) => iri.to_string(),
        Node::Literal(value, datatype) => format!("\"{}\"^^<{}>", value, datatype),
        Node::BlankNode(id) => format!("_:{}", id),
        Node::QuotedTriple(t) => format!("<<{} {} {}>>",
            node_to_string(&t.subject),
            node_to_string(&t.predicate),
            node_to_string(&t.object)),
        Node::Variable(v) => format!("?{}", v),
    }
}
