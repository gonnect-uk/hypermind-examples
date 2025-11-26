// Port of Jena Equality tests
// Tests equality semantics for nodes and triples

use rdf_model::{Dictionary, Node, Triple};
use std::sync::Arc;

#[test]
fn test_iri_equality() {
    let dict = Arc::new(Dictionary::new());

    let iri1 = Node::iri(dict.intern("http://example.org/resource"));
    let iri2 = Node::iri(dict.intern("http://example.org/resource"));

    assert_eq!(iri1.as_iri(), iri2.as_iri());
}

#[test]
fn test_iri_inequality() {
    let dict = Arc::new(Dictionary::new());

    let iri1 = Node::iri(dict.intern("http://example.org/resource1"));
    let iri2 = Node::iri(dict.intern("http://example.org/resource2"));

    assert_ne!(iri1.as_iri(), iri2.as_iri());
}

#[test]
fn test_literal_value_equality() {
    let dict = Arc::new(Dictionary::new());

    let lit1 = Node::literal_typed("hello", dict.intern("http://www.w3.org/2001/XMLSchema#string"));
    let lit2 = Node::literal_typed("hello", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    if let (Some(l1), Some(l2)) = (lit1.as_literal(), lit2.as_literal()) {
        assert_eq!(l1.lexical_form, l2.lexical_form);
        assert_eq!(l1.datatype, l2.datatype);
    }
}

#[test]
fn test_literal_value_inequality() {
    let dict = Arc::new(Dictionary::new());

    let lit1 = Node::literal_typed("hello", dict.intern("http://www.w3.org/2001/XMLSchema#string"));
    let lit2 = Node::literal_typed("world", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    if let (Some(l1), Some(l2)) = (lit1.as_literal(), lit2.as_literal()) {
        assert_ne!(l1.lexical_form, l2.lexical_form);
    }
}

#[test]
fn test_literal_datatype_inequality() {
    let dict = Arc::new(Dictionary::new());

    let int_lit = Node::literal_typed("42", dict.intern("http://www.w3.org/2001/XMLSchema#integer"));
    let str_lit = Node::literal_typed("42", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    if let (Some(l1), Some(l2)) = (int_lit.as_literal(), str_lit.as_literal()) {
        assert_eq!(l1.lexical_form, l2.lexical_form); // Same value
        assert_ne!(l1.datatype, l2.datatype); // Different datatypes
    }
}

#[test]
fn test_blank_node_equality() {
    let blank1 = Node::blank(42);
    let blank2 = Node::blank(42);

    assert_eq!(blank1.as_blank_node(), blank2.as_blank_node());
}

#[test]
fn test_blank_node_inequality() {
    let blank1 = Node::blank(1);
    let blank2 = Node::blank(2);

    assert_ne!(blank1.as_blank_node(), blank2.as_blank_node());
}

#[test]
fn test_triple_equality() {
    let dict = Arc::new(Dictionary::new());

    let s = Node::iri(dict.intern("http://example.org/s"));
    let p = Node::iri(dict.intern("http://example.org/p"));
    let o = Node::literal_typed("o", dict.intern("http://www.w3.org/2001/XMLSchema#string"));

    let triple1 = Triple {
        subject: s.clone(),
        predicate: p.clone(),
        object: o.clone(),
    };

    let triple2 = Triple {
        subject: s.clone(),
        predicate: p.clone(),
        object: o.clone(),
    };

    // Compare components
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
fn test_triple_subject_inequality() {
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

    assert_ne!(
        format!("{:?}", triple1.subject),
        format!("{:?}", triple2.subject)
    );
}

#[test]
fn test_node_type_inequality() {
    let dict = Arc::new(Dictionary::new());

    let iri = Node::iri(dict.intern("http://example.org/resource"));
    let literal = Node::literal_typed("resource", dict.intern("http://www.w3.org/2001/XMLSchema#string"));
    let blank = Node::blank(1);

    // Different node types should not be equal
    assert!(!iri.is_literal());
    assert!(!literal.is_iri());
    assert!(!blank.is_iri());
}

#[test]
fn test_case_sensitive_iri_equality() {
    let dict = Arc::new(Dictionary::new());

    let iri1 = Node::iri(dict.intern("http://example.org/Resource"));
    let iri2 = Node::iri(dict.intern("http://example.org/resource"));

    // IRIs are case-sensitive
    assert_ne!(iri1.as_iri(), iri2.as_iri());
}
