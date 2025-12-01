//! N-Quads serializer tests
//!
//! Tests N-Quads serialization and roundtrip parsing/serialization.

use rdf_io::{RDFSerializer, RDFParser, SerializerFactory, ParserFactory, RDFFormat};
use rdf_model::{Node, Quad, Dictionary};
use std::sync::Arc;

/// Helper to create a parser
fn create_parser() -> Box<dyn RDFParser> {
    let dict = Arc::new(Dictionary::new());
    ParserFactory::create(RDFFormat::NQuads, dict)
}

/// Helper to create a serializer
fn create_serializer() -> Box<dyn RDFSerializer> {
    SerializerFactory::create(RDFFormat::NQuads)
}

#[test]
fn test_serialize_simple_quad() {
    let dict = Arc::new(Dictionary::new());
    let subject = Node::iri(dict.intern("http://example.org/s"));
    let predicate = Node::iri(dict.intern("http://example.org/p"));
    let object = Node::iri(dict.intern("http://example.org/o"));
    let quad = Quad { subject, predicate, object, graph: None };

    let serializer = create_serializer();
    let nquads = serializer.serialize(&[quad]).expect("Should serialize");

    assert_eq!(nquads, "http://example.org/s http://example.org/p http://example.org/o .\n");
}

#[test]
fn test_serialize_quad_with_graph() {
    let dict = Arc::new(Dictionary::new());
    let subject = Node::iri(dict.intern("http://example.org/s"));
    let predicate = Node::iri(dict.intern("http://example.org/p"));
    let object = Node::iri(dict.intern("http://example.org/o"));
    let graph = Some(Node::iri(dict.intern("http://example.org/g")));
    let quad = Quad { subject, predicate, object, graph };

    let serializer = create_serializer();
    let nquads = serializer.serialize(&[quad]).expect("Should serialize");

    assert_eq!(
        nquads,
        "http://example.org/s http://example.org/p http://example.org/o http://example.org/g .\n"
    );
}

#[test]
fn test_serialize_literal() {
    let dict = Arc::new(Dictionary::new());
    let subject = Node::iri(dict.intern("http://example.org/s"));
    let predicate = Node::iri(dict.intern("http://example.org/p"));
    let object = Node::literal_str(dict.intern("Hello World"));
    let quad = Quad { subject, predicate, object, graph: None };

    let serializer = create_serializer();
    let nquads = serializer.serialize(&[quad]).expect("Should serialize");

    assert_eq!(nquads, "http://example.org/s http://example.org/p \"Hello World\" .\n");
}

#[test]
fn test_serialize_literal_with_language() {
    let dict = Arc::new(Dictionary::new());
    let subject = Node::iri(dict.intern("http://example.org/s"));
    let predicate = Node::iri(dict.intern("http://example.org/p"));
    let object = Node::literal_lang(dict.intern("Hello"), dict.intern("en"));
    let quad = Quad { subject, predicate, object, graph: None };

    let serializer = create_serializer();
    let nquads = serializer.serialize(&[quad]).expect("Should serialize");

    assert_eq!(nquads, "http://example.org/s http://example.org/p \"Hello\"@en .\n");
}

#[test]
fn test_serialize_literal_with_datatype() {
    let dict = Arc::new(Dictionary::new());
    let subject = Node::iri(dict.intern("http://example.org/s"));
    let predicate = Node::iri(dict.intern("http://example.org/p"));
    let object = Node::literal_typed(
        dict.intern("42"),
        dict.intern("http://www.w3.org/2001/XMLSchema#integer")
    );
    let quad = Quad { subject, predicate, object, graph: None };

    let serializer = create_serializer();
    let nquads = serializer.serialize(&[quad]).expect("Should serialize");

    assert_eq!(
        nquads,
        "http://example.org/s http://example.org/p \"42\"^^http://www.w3.org/2001/XMLSchema#integer .\n"
    );
}

#[test]
fn test_serialize_blank_nodes() {
    let dict = Arc::new(Dictionary::new());
    let subject = Node::blank(123);
    let predicate = Node::iri(dict.intern("http://example.org/p"));
    let object = Node::blank(456);
    let quad = Quad { subject, predicate, object, graph: None };

    let serializer = create_serializer();
    let nquads = serializer.serialize(&[quad]).expect("Should serialize");

    assert_eq!(nquads, "_:b123 http://example.org/p _:b456 .\n");
}

#[test]
fn test_serialize_escape_sequences() {
    let dict = Arc::new(Dictionary::new());
    let subject = Node::iri(dict.intern("http://example.org/s"));
    let predicate = Node::iri(dict.intern("http://example.org/p"));
    let object = Node::literal_str(dict.intern("Line 1\nLine 2\tTabbed"));
    let quad = Quad { subject, predicate, object, graph: None };

    let serializer = create_serializer();
    let nquads = serializer.serialize(&[quad]).expect("Should serialize");

    assert_eq!(
        nquads,
        "http://example.org/s http://example.org/p \"Line 1\\nLine 2\\tTabbed\" .\n"
    );
}

#[test]
fn test_serialize_multiple_quads() {
    let dict = Arc::new(Dictionary::new());

    let quad1 = Quad {
        subject: Node::iri(dict.intern("http://example.org/s1")),
        predicate: Node::iri(dict.intern("http://example.org/p1")),
        object: Node::iri(dict.intern("http://example.org/o1")),
        graph: None,
    };

    let quad2 = Quad {
        subject: Node::iri(dict.intern("http://example.org/s2")),
        predicate: Node::iri(dict.intern("http://example.org/p2")),
        object: Node::literal_str(dict.intern("value")),
        graph: Some(Node::iri(dict.intern("http://example.org/g"))),
    };

    let serializer = create_serializer();
    let nquads = serializer.serialize(&[quad1, quad2]).expect("Should serialize");

    let expected = concat!(
        "http://example.org/s1 http://example.org/p1 http://example.org/o1 .\n",
        "http://example.org/s2 http://example.org/p2 \"value\" http://example.org/g .\n"
    );

    assert_eq!(nquads, expected);
}

/// Roundtrip test: parse → serialize → parse again
#[test]
fn test_roundtrip_nquads() {
    let original = r#"<http://example.org/s> <http://example.org/p> <http://example.org/o> <http://example.org/g> .
<http://example.org/s2> <http://example.org/p2> "literal"@en .
<http://example.org/s3> <http://example.org/p3> "42"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;

    // Parse original
    let mut parser = create_parser();
    let quads1 = parser.parse(original).expect("Should parse");

    // Serialize
    let serializer = create_serializer();
    let serialized = serializer.serialize(&quads1).expect("Should serialize");

    // Parse again
    let mut parser2 = create_parser();
    let quads2 = parser2.parse(&serialized).expect("Should parse again");

    // Verify same number of quads
    assert_eq!(quads1.len(), quads2.len(), "Should have same number of quads");
    assert_eq!(quads1.len(), 3, "Should have 3 quads");
}

/// Test that serializer format is correct
#[test]
fn test_serializer_factory() {
    let serializer = SerializerFactory::create(RDFFormat::NQuads);
    assert_eq!(serializer.format(), RDFFormat::NQuads);
}
