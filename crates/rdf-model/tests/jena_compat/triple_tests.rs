// Port of Jena TestTriple.java and TestStatements.java
// Tests Triple creation, manipulation, and equality

use rdf_model::{Dictionary, Node, Triple};
use std::sync::Arc;

#[test]
fn test_create_triple() {
    let dict = Arc::new(Dictionary::new());

    let subject = Node::iri(dict.intern("http://example.org/subject"));
    let predicate = Node::iri(dict.intern("http://example.org/predicate"));
    let object = Node::literal_typed("value", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    let triple = Triple {
        subject: subject.clone(),
        predicate: predicate.clone(),
        object: object.clone(),
    };

    // Verify components
    assert!(matches!(triple.subject, Node::Iri(_)));
    assert!(matches!(triple.predicate, Node::Iri(_)));
    assert!(matches!(triple.object, Node::Literal(_)));
}

#[test]
fn test_triple_with_blank_node_subject() {
    let dict = Arc::new(Dictionary::new());

    let subject = Node::blank(1);
    let predicate = Node::iri(dict.intern("http://example.org/name"));
    let object = Node::literal_typed("Anonymous", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    let triple = Triple {
        subject,
        predicate,
        object,
    };

    // Blank nodes are valid subjects
    assert!(matches!(triple.subject, Node::BlankNode(_)));
}

#[test]
fn test_triple_with_literal_object() {
    let dict = Arc::new(Dictionary::new());

    let subject = Node::iri(dict.intern("http://example.org/person"));
    let predicate = Node::iri(dict.intern("http://example.org/age"));
    let object = Node::literal_typed("30", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));

    let triple = Triple {
        subject,
        predicate,
        object,
    };

    // Verify literal object
    if let Node::Literal(lit) = triple.object {
        assert_eq!(lit.lexical_form, "30");
        assert_eq!(lit.datatype, Some("http://www.w3.org/2001/XMLSchema#integer"));
    } else {
        panic!("Expected Literal object");
    }
}

#[test]
fn test_triple_with_iri_object() {
    let dict = Arc::new(Dictionary::new());

    let subject = Node::iri(dict.intern("http://example.org/book"));
    let predicate = Node::iri(dict.intern("http://example.org/author"));
    let object = Node::iri(dict.intern("http://example.org/person/john"));

    let triple = Triple {
        subject,
        predicate,
        object,
    };

    // Verify IRI object
    assert!(matches!(triple.object, Node::Iri(_)));
}

#[test]
fn test_triple_equality() {
    let dict = Arc::new(Dictionary::new());

    let subject1 = Node::iri(dict.intern("http://example.org/s"));
    let predicate1 = Node::iri(dict.intern("http://example.org/p"));
    let object1 = Node::literal_typed("o", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    let triple1 = Triple {
        subject: subject1.clone(),
        predicate: predicate1.clone(),
        object: object1.clone(),
    };

    let triple2 = Triple {
        subject: subject1.clone(),
        predicate: predicate1.clone(),
        object: object1.clone(),
    };

    // Triples with same components should be equal
    assert_eq!(
        format!("{:?}", triple1.subject),
        format!("{:?}", triple2.subject)
    );
    assert_eq!(
        format!("{:?}", triple1.predicate),
        format!("{:?}", triple2.predicate)
    );
    assert_eq!(
        format!("{:?}", triple1.object),
        format!("{:?}", triple2.object)
    );
}

#[test]
fn test_triple_inequality() {
    let dict = Arc::new(Dictionary::new());

    let triple1 = Triple {
        subject: Node::iri(dict.intern("http://example.org/s1")),
        predicate: Node::iri(dict.intern("http://example.org/p")),
        object: Node::literal_typed("o", dict.intern("http://www.w3.org/2001/XMLSchema#string")),
    };

    let triple2 = Triple {
        subject: Node::iri(dict.intern("http://example.org/s2")),
        predicate: Node::iri(dict.intern("http://example.org/p")),
        object: Node::literal_typed("o", dict.intern("http://www.w3.org/2001/XMLSchema#string")),
    };

    // Triples with different subjects should not be equal
    assert_ne!(
        format!("{:?}", triple1.subject),
        format!("{:?}", triple2.subject)
    );
}

#[test]
fn test_triple_with_multiple_datatypes() {
    let dict = Arc::new(Dictionary::new());

    let subject = Node::iri(dict.intern("http://example.org/data"));

    // Integer literal
    let triple1 = Triple {
        subject: subject.clone(),
        predicate: Node::iri(dict.intern("http://example.org/intProp")),
        object: Node::literal_typed("42", dict.intern("http://www.w3.org/2001/XMLSchema#integer")),
    };

    // String literal
    let triple2 = Triple {
        subject: subject.clone(),
        predicate: Node::iri(dict.intern("http://example.org/strProp")),
        object: Node::literal_typed("text", dict.intern("http://www.w3.org/2001/XMLSchema#string")),
    };

    // Boolean literal
    let triple3 = Triple {
        subject: subject.clone(),
        predicate: Node::iri(dict.intern("http://example.org/boolProp")),
        object: Node::literal_typed("true", dict.intern("http://www.w3.org/2001/XMLSchema#boolean")),
    };

    // All should be valid triples
    assert!(matches!(triple1.object, Node::Literal(_)));
    assert!(matches!(triple2.object, Node::Literal(_)));
    assert!(matches!(triple3.object, Node::Literal(_)));
}

#[test]
fn test_triple_pattern_matching() {
    let dict = Arc::new(Dictionary::new());

    let triple = Triple {
        subject: Node::iri(dict.intern("http://example.org/subject")),
        predicate: Node::iri(dict.intern("http://example.org/predicate")),
        object: Node::literal_typed("value", dict.intern("http://www.w3.org/2001/XMLSchema#string")),
    };

    // Pattern matching on components
    match (&triple.subject, &triple.predicate, &triple.object) {
        (Node::Iri(s), Node::Iri(p), Node::Literal(lit)) => {
            assert_eq!(s.as_str(), "http://example.org/subject");
            assert_eq!(p.as_str(), "http://example.org/predicate");
            assert_eq!(lit.lexical_form, "value");
        }
        _ => panic!("Unexpected triple structure"),
    }
}

#[test]
fn test_triple_cloning() {
    let dict = Arc::new(Dictionary::new());

    let subject = Node::iri(dict.intern("http://example.org/s"));
    let predicate = Node::iri(dict.intern("http://example.org/p"));
    let object = Node::literal_typed("o", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    let triple1 = Triple {
        subject: subject.clone(),
        predicate: predicate.clone(),
        object: object.clone(),
    };

    let triple2 = Triple {
        subject: subject.clone(),
        predicate: predicate.clone(),
        object: object.clone(),
    };

    // Cloned triples should have equal components
    assert_eq!(
        format!("{:?}", triple1.subject),
        format!("{:?}", triple2.subject)
    );
}

#[test]
fn test_triple_with_rdf_star() {
    let dict = Arc::new(Dictionary::new());

    // Create inner triple
    let inner_subject = Node::iri(dict.intern("http://example.org/alice"));
    let inner_predicate = Node::iri(dict.intern("http://example.org/knows"));
    let inner_object = Node::iri(dict.intern("http://example.org/bob"));

    let inner_triple = Triple {
        subject: inner_subject.clone(),
        predicate: inner_predicate.clone(),
        object: inner_object.clone(),
    };

    // Use quoted triple as subject (RDF-star)
    let outer_triple = Triple {
        subject: Node::quoted_triple(inner_triple),
        predicate: Node::iri(dict.intern("http://example.org/certainty")),
        object: Node::literal_typed("0.9", dict.intern("http://www.w3.org/2001/XMLSchema#double")),
    };

    // Verify RDF-star triple
    assert!(matches!(outer_triple.subject, Node::QuotedTriple(_)));
}
