//! Multi-Format Side-by-Side Tests
//!
//! Tests the same RDF data in multiple formats (Turtle, N-Triples, N-Quads)
//! to verify that all parsers produce identical quads.
//!
//! This demonstrates the Strategy Pattern in action - all formats go through
//! the unified RDFParser trait and produce the same results.

use rdf_io::{ParserFactory, RDFFormat, RDFParser};
use rdf_model::{Dictionary, Node};
use std::sync::Arc;

/// Helper to create a parser for a specific format
fn create_parser(format: RDFFormat) -> Box<dyn RDFParser> {
    let dict = Arc::new(Dictionary::new());
    ParserFactory::create(format, dict)
}

/// Test 1: Simple triple in all formats
#[test]
fn test_simple_triple_all_formats() {
    // Same data in 3 formats
    let turtle = r#"
        @prefix ex: <http://example.org/> .
        ex:Alice ex:knows ex:Bob .
    "#;

    let ntriples = r#"
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .
    "#;

    let nquads = r#"
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .
    "#;

    // Parse with all three parsers
    let mut turtle_parser = create_parser(RDFFormat::Turtle);
    let mut ntriples_parser = create_parser(RDFFormat::NTriples);
    let mut nquads_parser = create_parser(RDFFormat::NQuads);

    let turtle_quads = turtle_parser.parse(turtle).expect("Turtle should parse");
    let ntriples_quads = ntriples_parser.parse(ntriples).expect("N-Triples should parse");
    let nquads_quads = nquads_parser.parse(nquads).expect("N-Quads should parse");

    // All should produce 1 quad
    assert_eq!(turtle_quads.len(), 1, "Turtle should produce 1 quad");
    assert_eq!(ntriples_quads.len(), 1, "N-Triples should produce 1 quad");
    assert_eq!(nquads_quads.len(), 1, "N-Quads should produce 1 quad");

    // All should have IRI nodes
    assert!(matches!(turtle_quads[0].subject, Node::Iri(_)));
    assert!(matches!(ntriples_quads[0].subject, Node::Iri(_)));
    assert!(matches!(nquads_quads[0].subject, Node::Iri(_)));

    // None should have a graph (default graph)
    assert!(turtle_quads[0].graph.is_none());
    assert!(ntriples_quads[0].graph.is_none());
    assert!(nquads_quads[0].graph.is_none());
}

/// Test 2: Literals with language tags
#[test]
fn test_literals_with_language_all_formats() {
    let turtle = r#"
        @prefix ex: <http://example.org/> .
        ex:Alice ex:name "Alice Smith"@en .
    "#;

    let ntriples = r#"
        <http://example.org/Alice> <http://example.org/name> "Alice Smith"@en .
    "#;

    let nquads = r#"
        <http://example.org/Alice> <http://example.org/name> "Alice Smith"@en .
    "#;

    let mut turtle_parser = create_parser(RDFFormat::Turtle);
    let mut ntriples_parser = create_parser(RDFFormat::NTriples);
    let mut nquads_parser = create_parser(RDFFormat::NQuads);

    let turtle_quads = turtle_parser.parse(turtle).expect("Turtle should parse");
    let ntriples_quads = ntriples_parser.parse(ntriples).expect("N-Triples should parse");
    let nquads_quads = nquads_parser.parse(nquads).expect("N-Quads should parse");

    // All should have literals with language tag
    for quads in [&turtle_quads, &ntriples_quads, &nquads_quads] {
        match &quads[0].object {
            Node::Literal(lit) => {
                assert!(lit.lexical_form.contains("Alice Smith"));
                assert_eq!(lit.language, Some("en"));
                assert_eq!(lit.datatype, None);
            }
            _ => panic!("Expected Literal with language tag"),
        }
    }
}

/// Test 3: Typed literals
#[test]
fn test_typed_literals_all_formats() {
    let turtle = r#"
        @prefix ex: <http://example.org/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        ex:Alice ex:age "30"^^xsd:integer .
    "#;

    let ntriples = r#"
        <http://example.org/Alice> <http://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
    "#;

    let nquads = r#"
        <http://example.org/Alice> <http://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
    "#;

    let mut turtle_parser = create_parser(RDFFormat::Turtle);
    let mut ntriples_parser = create_parser(RDFFormat::NTriples);
    let mut nquads_parser = create_parser(RDFFormat::NQuads);

    let turtle_quads = turtle_parser.parse(turtle).expect("Turtle should parse");
    let ntriples_quads = ntriples_parser.parse(ntriples).expect("N-Triples should parse");
    let nquads_quads = nquads_parser.parse(nquads).expect("N-Quads should parse");

    // All should have typed literals
    for quads in [&turtle_quads, &ntriples_quads, &nquads_quads] {
        match &quads[0].object {
            Node::Literal(lit) => {
                assert!(lit.lexical_form.contains("30"));
                assert_eq!(lit.language, None);
                assert!(lit.datatype.is_some());
                assert!(lit.datatype.unwrap().contains("integer"));
            }
            _ => panic!("Expected typed Literal"),
        }
    }
}

/// Test 4: Blank nodes
#[test]
fn test_blank_nodes_all_formats() {
    let turtle = r#"
        @prefix ex: <http://example.org/> .
        _:person ex:name "Anonymous" .
    "#;

    let ntriples = r#"
        _:person <http://example.org/name> "Anonymous" .
    "#;

    let nquads = r#"
        _:person <http://example.org/name> "Anonymous" .
    "#;

    let mut turtle_parser = create_parser(RDFFormat::Turtle);
    let mut ntriples_parser = create_parser(RDFFormat::NTriples);
    let mut nquads_parser = create_parser(RDFFormat::NQuads);

    let turtle_quads = turtle_parser.parse(turtle).expect("Turtle should parse");
    let ntriples_quads = ntriples_parser.parse(ntriples).expect("N-Triples should parse");
    let nquads_quads = nquads_parser.parse(nquads).expect("N-Quads should parse");

    // All should have blank node subjects
    assert!(matches!(turtle_quads[0].subject, Node::BlankNode(_)));
    assert!(matches!(ntriples_quads[0].subject, Node::BlankNode(_)));
    assert!(matches!(nquads_quads[0].subject, Node::BlankNode(_)));
}

/// Test 5: Named graphs (N-Quads only)
#[test]
fn test_named_graphs_nquads() {
    // Only N-Quads supports named graphs in the syntax
    let nquads = r#"
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> <http://example.org/graph1> .
        <http://example.org/Bob> <http://example.org/age> "30" <http://example.org/graph2> .
    "#;

    let mut nquads_parser = create_parser(RDFFormat::NQuads);
    let quads = nquads_parser.parse(nquads).expect("N-Quads should parse");

    assert_eq!(quads.len(), 2);

    // First quad has graph1
    match &quads[0].graph {
        Some(Node::Iri(g)) => assert!(g.0.contains("graph1")),
        _ => panic!("Expected named graph IRI"),
    }

    // Second quad has graph2
    match &quads[1].graph {
        Some(Node::Iri(g)) => assert!(g.0.contains("graph2")),
        _ => panic!("Expected named graph IRI"),
    }
}

/// Test 6: Multiple triples
#[test]
fn test_multiple_triples_all_formats() {
    let turtle = r#"
        @prefix ex: <http://example.org/> .
        ex:Alice ex:knows ex:Bob .
        ex:Bob ex:knows ex:Charlie .
        ex:Charlie ex:age "25" .
    "#;

    let ntriples = r#"
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .
        <http://example.org/Bob> <http://example.org/knows> <http://example.org/Charlie> .
        <http://example.org/Charlie> <http://example.org/age> "25" .
    "#;

    let nquads = r#"
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .
        <http://example.org/Bob> <http://example.org/knows> <http://example.org/Charlie> .
        <http://example.org/Charlie> <http://example.org/age> "25" .
    "#;

    let mut turtle_parser = create_parser(RDFFormat::Turtle);
    let mut ntriples_parser = create_parser(RDFFormat::NTriples);
    let mut nquads_parser = create_parser(RDFFormat::NQuads);

    let turtle_quads = turtle_parser.parse(turtle).expect("Turtle should parse");
    let ntriples_quads = ntriples_parser.parse(ntriples).expect("N-Triples should parse");
    let nquads_quads = nquads_parser.parse(nquads).expect("N-Quads should parse");

    // All should produce 3 quads
    assert_eq!(turtle_quads.len(), 3);
    assert_eq!(ntriples_quads.len(), 3);
    assert_eq!(nquads_quads.len(), 3);
}

/// Test 7: ParserFactory auto-detection from filename
#[test]
fn test_parser_factory_auto_detection() {
    let dict = Arc::new(Dictionary::new());

    // Test auto-detection for implemented formats
    let turtle_parser = ParserFactory::create_from_file("data.ttl", dict.clone());
    assert_eq!(turtle_parser.format(), RDFFormat::Turtle);

    let nt_parser = ParserFactory::create_from_file("data.nt", dict.clone());
    assert_eq!(nt_parser.format(), RDFFormat::NTriples);

    let nq_parser = ParserFactory::create_from_file("data.nq", dict.clone());
    assert_eq!(nq_parser.format(), RDFFormat::NQuads);

    // Note: RDF/XML, JSON-LD, and TriG are not yet implemented
    // These will be added in future releases
}

/// Test 8: Comments and whitespace handling
#[test]
fn test_comments_and_whitespace() {
    let turtle = r#"
        # This is a comment
        @prefix ex: <http://example.org/> .

        # Another comment
        ex:Alice ex:knows ex:Bob .
    "#;

    let ntriples = r#"
        # N-Triples comment
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .

    "#;

    let nquads = r#"
        # N-Quads comment

        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .
    "#;

    let mut turtle_parser = create_parser(RDFFormat::Turtle);
    let mut ntriples_parser = create_parser(RDFFormat::NTriples);
    let mut nquads_parser = create_parser(RDFFormat::NQuads);

    let turtle_quads = turtle_parser.parse(turtle).expect("Turtle should parse");
    let ntriples_quads = ntriples_parser.parse(ntriples).expect("N-Triples should parse");
    let nquads_quads = nquads_parser.parse(nquads).expect("N-Quads should parse");

    // All should ignore comments and produce 1 quad
    assert_eq!(turtle_quads.len(), 1);
    assert_eq!(ntriples_quads.len(), 1);
    assert_eq!(nquads_quads.len(), 1);
}

/// Test 9: Empty input
#[test]
fn test_empty_input_all_formats() {
    let empty = "";

    let mut turtle_parser = create_parser(RDFFormat::Turtle);
    let mut ntriples_parser = create_parser(RDFFormat::NTriples);
    let mut nquads_parser = create_parser(RDFFormat::NQuads);

    let turtle_quads = turtle_parser.parse(empty).expect("Empty Turtle should parse");
    let ntriples_quads = ntriples_parser.parse(empty).expect("Empty N-Triples should parse");
    let nquads_quads = nquads_parser.parse(empty).expect("Empty N-Quads should parse");

    // All should produce 0 quads
    assert_eq!(turtle_quads.len(), 0);
    assert_eq!(ntriples_quads.len(), 0);
    assert_eq!(nquads_quads.len(), 0);
}

/// Test 10: Unicode support
#[test]
fn test_unicode_all_formats() {
    let turtle = r#"
        @prefix ex: <http://example.org/> .
        ex:Alice ex:name "アリス"@ja .
    "#;

    let ntriples = r#"
        <http://example.org/Alice> <http://example.org/name> "アリス"@ja .
    "#;

    let nquads = r#"
        <http://example.org/Alice> <http://example.org/name> "アリス"@ja .
    "#;

    let mut turtle_parser = create_parser(RDFFormat::Turtle);
    let mut ntriples_parser = create_parser(RDFFormat::NTriples);
    let mut nquads_parser = create_parser(RDFFormat::NQuads);

    let turtle_quads = turtle_parser.parse(turtle).expect("Unicode Turtle should parse");
    let ntriples_quads = ntriples_parser.parse(ntriples).expect("Unicode N-Triples should parse");
    let nquads_quads = nquads_parser.parse(nquads).expect("Unicode N-Quads should parse");

    // All should preserve Unicode
    for quads in [&turtle_quads, &ntriples_quads, &nquads_quads] {
        match &quads[0].object {
            Node::Literal(lit) => {
                assert!(lit.lexical_form.contains("アリス"));
            }
            _ => panic!("Expected Literal with Unicode"),
        }
    }
}
