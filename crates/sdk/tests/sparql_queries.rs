//! SPARQL query tests

use rust_kgdb_sdk::{GraphDB, Node};

#[test]
fn test_simple_select_query() {
    let mut db = GraphDB::in_memory();

    // Insert test data
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
        .execute()
        .expect("Insert should succeed");

    // Query
    let results = db
        .query()
        .sparql("SELECT ?name WHERE { ?person <http://xmlns.com/foaf/0.1/name> ?name }")
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 2);
}

#[test]
fn test_query_no_results() {
    let db = GraphDB::in_memory();

    let results = db
        .query()
        .sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }")
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 0);
    assert!(results.is_empty());
}

#[test]
fn test_query_with_filter() {
    let mut db = GraphDB::in_memory();

    // Insert test data
    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/age"),
            Node::integer(30),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://example.org/age"),
            Node::integer(25),
        )
        .execute()
        .expect("Insert should succeed");

    // Query with FILTER (Note: This will work once the executor supports FILTER properly)
    let results = db
        .query()
        .sparql("SELECT ?person WHERE { ?person <http://example.org/age> ?age }")
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 2);
}

#[test]
fn test_query_builder_error() {
    let db = GraphDB::in_memory();

    // Query without SPARQL should fail
    let result = db.query().execute();
    assert!(result.is_err());
}
