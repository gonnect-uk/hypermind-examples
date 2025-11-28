#!/usr/bin/env rustc --test

use std::sync::Arc;

// Minimal test to reproduce the semicolon parsing bug
#[test]
fn test_semicolon_syntax() {
    // Load the rdf-io and rdf-model crates
    extern crate rdf_model;
    extern crate rdf_io;

    use rdf_model::Dictionary;
    use rdf_io::turtle::TurtleParser;

    let dict = Arc::new(Dictionary::new());
    let mut parser = TurtleParser::new(Arc::clone(&dict));

    // Test simple semicolon syntax
    let turtle = r#"
        @prefix ex: <http://example.org/> .

        ex:subject
            ex:pred1 ex:obj1 ;
            ex:pred2 ex:obj2 .
    "#;

    match parser.parse(turtle) {
        Ok(quads) => {
            println!("✅ SUCCESS: Parsed {} triples", quads.len());
            for quad in &quads {
                println!("  {} {} {}", quad.subject, quad.predicate, quad.object);
            }
            assert_eq!(quads.len(), 2, "Should parse 2 triples");
        }
        Err(e) => {
            println!("❌ FAILED: {:?}", e);
            panic!("Parser should handle semicolon syntax");
        }
    }
}

fn main() {
    test_semicolon_syntax();
}
