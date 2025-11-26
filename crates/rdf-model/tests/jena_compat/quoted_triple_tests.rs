// Port of Jena RDF-star (Quoted Triple) tests
// Tests RDF-star quoted triples (RDF 1.1 feature)

use rdf_model::{Dictionary, Node, Triple};
use std::sync::Arc;

#[test]
fn test_create_quoted_triple() {
    let dict = Arc::new(Dictionary::new());

    let subject = Node::iri(dict.intern("http://example.org/alice"));
    let predicate = Node::iri(dict.intern("http://example.org/knows"));
    let object = Node::iri(dict.intern("http://example.org/bob"));

    let triple = Triple::new(subject, predicate, object);
    let quoted = Node::quoted_triple(triple);

    assert!(matches!(quoted, Node::QuotedTriple(_)));
}

#[test]
fn test_quoted_triple_as_subject() {
    let dict = Arc::new(Dictionary::new());

    // Inner triple: Alice knows Bob
    let inner = Triple::new(
        Node::iri(dict.intern("http://example.org/alice")),
        Node::iri(dict.intern("http://example.org/knows")),
        Node::iri(dict.intern("http://example.org/bob")),
    );

    // Outer triple: <<Alice knows Bob>> certainty "high"
    let outer = Triple::new(
        Node::quoted_triple(inner),
        Node::iri(dict.intern("http://example.org/certainty")),
        Node::literal_typed("high", dict.intern("http://www.w3.org/2001/XMLSchema#string")),
    );

    assert!(matches!(outer.subject, Node::QuotedTriple(_)));
}

#[test]
fn test_quoted_triple_as_object() {
    let dict = Arc::new(Dictionary::new());

    // Inner triple: Alice age 30
    let inner = Triple::new(
        Node::iri(dict.intern("http://example.org/alice")),
        Node::iri(dict.intern("http://example.org/age")),
        Node::literal_typed("30", dict.intern("http://www.w3.org/2001/XMLSchema#integer")),
    );

    // Outer triple: Document mentions <<Alice age 30>>
    let outer = Triple::new(
        Node::iri(dict.intern("http://example.org/document1")),
        Node::iri(dict.intern("http://example.org/mentions")),
        Node::quoted_triple(inner),
    );

    assert!(matches!(outer.object, Node::QuotedTriple(_)));
}

#[test]
fn test_nested_quoted_triples() {
    let dict = Arc::new(Dictionary::new());

    // Level 1: Alice knows Bob
    let level1 = Triple::new(
        Node::iri(dict.intern("http://example.org/alice")),
        Node::iri(dict.intern("http://example.org/knows")),
        Node::iri(dict.intern("http://example.org/bob")),
    );

    // Level 2: <<Alice knows Bob>> certainty "high"
    let level2 = Triple::new(
        Node::quoted_triple(level1),
        Node::iri(dict.intern("http://example.org/certainty")),
        Node::literal_typed("high", dict.intern("http://www.w3.org/2001/XMLSchema#string")),
    );

    // Level 3: <<<Alice knows Bob> certainty "high">> source "survey"
    let level3 = Node::quoted_triple(level2);

    assert!(matches!(level3, Node::QuotedTriple(_)));
}

#[test]
fn test_quoted_triple_extraction() {
    let dict = Arc::new(Dictionary::new());

    let inner_subject = Node::iri(dict.intern("http://example.org/s"));
    let inner_predicate = Node::iri(dict.intern("http://example.org/p"));
    let inner_object = Node::literal_typed("o", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    let triple = Triple::new(
        inner_subject.clone(),
        inner_predicate.clone(),
        inner_object.clone(),
    );
    let quoted = Node::quoted_triple(triple);

    // Extract components
    if let Node::QuotedTriple(inner) = quoted {
        assert!(matches!(inner.subject, Node::Iri(_)));
        assert!(matches!(inner.predicate, Node::Iri(_)));
        assert!(matches!(inner.object, Node::Literal(_)));
    }
}

#[test]
fn test_quoted_triple_with_blank_node() {
    let dict = Arc::new(Dictionary::new());

    // Quoted triple with blank node
    let triple = Triple::new(
        Node::blank(1),
        Node::iri(dict.intern("http://example.org/name")),
        Node::literal_typed("Anonymous", dict.intern("http://www.w3.org/2001/XMLSchema#string")),
    );

    let quoted = Node::quoted_triple(triple);

    if let Node::QuotedTriple(inner) = quoted {
        assert!(matches!(inner.subject, Node::BlankNode(_)));
    }
}

#[test]
fn test_quoted_triple_provenance() {
    let dict = Arc::new(Dictionary::new());

    // Original claim: Company revenue 1000000
    let claim = Triple::new(
        Node::iri(dict.intern("http://example.org/company")),
        Node::iri(dict.intern("http://example.org/revenue")),
        Node::literal_typed("1000000", dict.intern("http://www.w3.org/2001/XMLSchema#integer")),
    );

    // Add source metadata to claim
    let provenance = Triple::new(
        Node::quoted_triple(claim),
        Node::iri(dict.intern("http://example.org/source")),
        Node::iri(dict.intern("http://example.org/annual_report_2023")),
    );

    assert!(matches!(provenance.subject, Node::QuotedTriple(_)));
}

#[test]
fn test_quoted_triple_annotation() {
    let dict = Arc::new(Dictionary::new());

    // Statement: Alice likes Pizza
    let statement = Triple::new(
        Node::iri(dict.intern("http://example.org/alice")),
        Node::iri(dict.intern("http://example.org/likes")),
        Node::iri(dict.intern("http://example.org/pizza")),
    );

    // Annotate with timestamp
    let annotated = Triple::new(
        Node::quoted_triple(statement),
        Node::iri(dict.intern("http://example.org/timestamp")),
        Node::literal_typed("2023-11-25T10:00:00Z", dict.intern("http://www.w3.org/2001/XMLSchema#dateTime")),
    );

    assert!(matches!(annotated.subject, Node::QuotedTriple(_)));
}

#[test]
fn test_quoted_triple_not_iri() {
    let dict = Arc::new(Dictionary::new());

    let triple = Triple::new(
        Node::iri(dict.intern("http://example.org/s")),
        Node::iri(dict.intern("http://example.org/p")),
        Node::iri(dict.intern("http://example.org/o")),
    );

    let quoted = Node::quoted_triple(triple);

    assert!(!matches!(quoted, Node::Iri(_)));
}

#[test]
fn test_quoted_triple_not_literal() {
    let dict = Arc::new(Dictionary::new());

    let triple = Triple::new(
        Node::iri(dict.intern("http://example.org/s")),
        Node::iri(dict.intern("http://example.org/p")),
        Node::iri(dict.intern("http://example.org/o")),
    );

    let quoted = Node::quoted_triple(triple);

    assert!(!matches!(quoted, Node::Literal(_)));
}
