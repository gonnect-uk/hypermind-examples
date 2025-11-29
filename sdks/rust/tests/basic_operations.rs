//! Basic SDK operations tests

use rust_kgdb_sdk::{GraphDB, Node};

#[test]
fn test_create_database() {
    let db = GraphDB::in_memory();
    assert_eq!(db.count(), 0);
    assert!(db.is_empty());
}

#[test]
fn test_insert_single_triple() {
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Alice"),
        )
        .execute()
        .expect("Insert should succeed");

    assert_eq!(db.count(), 1);
    assert!(!db.is_empty());
}

#[test]
fn test_insert_multiple_triples() {
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Alice"),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Bob"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .execute()
        .expect("Insert should succeed");

    assert_eq!(db.count(), 3);
}

#[test]
fn test_typed_literals() {
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/age"),
            Node::integer(30),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://example.org/active"),
            Node::boolean(true),
        )
        .execute()
        .expect("Insert should succeed");

    assert_eq!(db.count(), 2);
}

#[test]
fn test_language_tagged_literals() {
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/greeting"),
            Node::lang_literal("Hello", "en"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/greeting"),
            Node::lang_literal("Bonjour", "fr"),
        )
        .execute()
        .expect("Insert should succeed");

    assert_eq!(db.count(), 2);
}

#[test]
fn test_node_types() {
    let iri = Node::iri("http://example.org/test");
    assert!(iri.is_iri());
    assert!(!iri.is_literal());
    assert!(!iri.is_blank());
    assert_eq!(iri.as_iri(), Some("http://example.org/test"));

    let lit = Node::literal("test");
    assert!(!lit.is_iri());
    assert!(lit.is_literal());
    assert!(!lit.is_blank());

    let blank = Node::blank("b0");
    assert!(!blank.is_iri());
    assert!(!blank.is_literal());
    assert!(blank.is_blank());
    assert_eq!(blank.as_blank(), Some("b0"));
}

#[test]
fn test_node_display() {
    assert_eq!(
        Node::iri("http://example.org/test").to_string(),
        "<http://example.org/test>"
    );
    assert_eq!(Node::literal("test").to_string(), "\"test\"");
    assert_eq!(Node::blank("b0").to_string(), "_:b0");
    assert_eq!(
        Node::typed_literal("42", "http://www.w3.org/2001/XMLSchema#integer").to_string(),
        "\"42\"^^<http://www.w3.org/2001/XMLSchema#integer>"
    );
    assert_eq!(Node::lang_literal("Hello", "en").to_string(), "\"Hello\"@en");
}
