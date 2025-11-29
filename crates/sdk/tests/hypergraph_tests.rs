//! Hypergraph-specific tests for Rust SDK
//!
//! Tests hypergraph operations that go beyond standard RDF triple operations:
//! - Binary edges (2 nodes)
//! - Ternary edges (3 nodes - standard RDF triples)
//! - Quaternary edges (4 nodes - RDF quads with named graphs)
//! - N-ary edges (arbitrary number of nodes)
//! - Hyperedge traversal and pattern matching
//! - Complex multi-edge patterns

use rust_kgdb_sdk::{GraphDB, Node};

#[test]
fn hypergraph_binary_edge() {
    // Binary hyperedge: Two nodes connected by an edge label
    // In RDF: (subject) -[predicate]-> (object)
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/likes"), // Edge label
            Node::iri("http://example.org/pizza"),
        )
        .execute()
        .expect("Insert should succeed");

    let results = db
        .query()
        .sparql("SELECT ?who ?what WHERE { ?who <http://example.org/likes> ?what }")
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find 1 binary edge");
}

#[test]
fn hypergraph_ternary_edge_standard_triple() {
    // Ternary hyperedge: Standard RDF triple
    // Three nodes: (subject, predicate, object)
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"), // Node 1: Subject
            Node::iri("http://xmlns.com/foaf/0.1/name"), // Node 2: Predicate
            Node::literal("Alice"),                // Node 3: Object
        )
        .execute()
        .expect("Insert should succeed");

    assert_eq!(db.count(), 1, "Should have 1 ternary edge (triple)");
}

#[test]
fn hypergraph_quaternary_edge_named_graph() {
    // Quaternary hyperedge: RDF quad with named graph
    // Four nodes: (subject, predicate, object, graph)
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Alice"),
        )
        .graph(Node::iri("http://example.org/graph1")) // 4th node: Named graph
        .execute()
        .expect("Insert should succeed");

    assert_eq!(db.count(), 1, "Should have 1 quaternary edge (quad)");
}

#[test]
fn hypergraph_multiple_edges_same_subject() {
    // One node (subject) with multiple outgoing hyperedges
    // Star pattern: Alice -[name]-> "Alice"
    //               Alice -[age]-> 30
    //               Alice -[email]-> "alice@example.org"
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Alice"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/age"),
            Node::integer(30),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/email"),
            Node::literal("alice@example.org"),
        )
        .execute()
        .expect("Insert should succeed");

    // Query all properties of Alice
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?p ?o WHERE {
                <http://example.org/alice> ?p ?o
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 3, "Should find 3 outgoing edges");
}

#[test]
fn hypergraph_edge_traversal() {
    // Hyperedge traversal: Following connected edges
    // Path: Alice -[knows]-> Bob -[knows]-> Charlie
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/charlie"),
        )
        .execute()
        .expect("Insert should succeed");

    // Find friends of friends (2-hop traversal)
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?friend_of_friend WHERE {
                <http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> ?friend .
                ?friend <http://xmlns.com/foaf/0.1/knows> ?friend_of_friend
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find Charlie through Bob");
}

#[test]
fn hypergraph_bidirectional_edges() {
    // Bidirectional edges: Two edges in opposite directions
    // Alice -[knows]-> Bob
    // Bob -[knows]-> Alice
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/alice"),
        )
        .execute()
        .expect("Insert should succeed");

    // Find mutual friends
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?person1 ?person2 WHERE {
                ?person1 <http://xmlns.com/foaf/0.1/knows> ?person2 .
                ?person2 <http://xmlns.com/foaf/0.1/knows> ?person1
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should find 2 mutual relationships");
}

#[test]
fn hypergraph_complex_pattern() {
    // Complex hypergraph pattern: Small social network
    // Nodes: Alice, Bob, Charlie
    // Edges: Alice knows Bob, Bob knows Charlie, Alice knows Charlie
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Alice"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/age"),
            Node::integer(30),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://xmlns.com/foaf/0.1/name"),
            Node::literal("Bob"),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://xmlns.com/foaf/0.1/age"),
            Node::integer(25),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .execute()
        .expect("Insert should succeed");

    // Complex pattern: Find people who know someone
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?person ?name ?knows WHERE {
                ?person <http://xmlns.com/foaf/0.1/name> ?name .
                ?person <http://xmlns.com/foaf/0.1/knows> ?knows
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 1, "Alice knows Bob");
}

#[test]
fn hypergraph_triangle_pattern() {
    // Triangle pattern: Three nodes with circular connections
    // Alice -> Bob -> Charlie -> Alice
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/charlie"),
        )
        .triple(
            Node::iri("http://example.org/charlie"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/alice"),
        )
        .execute()
        .expect("Insert should succeed");

    // Find triangles
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?a ?b ?c WHERE {
                ?a <http://xmlns.com/foaf/0.1/knows> ?b .
                ?b <http://xmlns.com/foaf/0.1/knows> ?c .
                ?c <http://xmlns.com/foaf/0.1/knows> ?a
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    // Triangle can be matched starting from any of the 3 nodes, so 3 results expected
    assert_eq!(results.len(), 3, "Should find 3 triangle rotations");
}

#[test]
fn hypergraph_star_pattern() {
    // Star pattern: One central node with multiple connections
    // Alice knows: Bob, Charlie, Dave, Eve
    let mut db = GraphDB::in_memory();

    let people = vec![
        "http://example.org/bob",
        "http://example.org/charlie",
        "http://example.org/dave",
        "http://example.org/eve",
    ];

    for person in &people {
        db.insert()
            .triple(
                Node::iri("http://example.org/alice"),
                Node::iri("http://xmlns.com/foaf/0.1/knows"),
                Node::iri(*person),
            )
            .execute()
            .expect("Insert should succeed");
    }

    // Find all connections from center
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?person WHERE {
                <http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> ?person
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 4, "Alice should know 4 people");
}

#[test]
fn hypergraph_multi_hop_traversal() {
    // Multi-hop traversal: 3-hop path
    // Alice -> Bob -> Charlie -> Dave
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .triple(
            Node::iri("http://example.org/bob"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/charlie"),
        )
        .triple(
            Node::iri("http://example.org/charlie"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/dave"),
        )
        .execute()
        .expect("Insert should succeed");

    // 3-hop query
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?hop3 WHERE {
                <http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> ?hop1 .
                ?hop1 <http://xmlns.com/foaf/0.1/knows> ?hop2 .
                ?hop2 <http://xmlns.com/foaf/0.1/knows> ?hop3
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find Dave 3 hops away");
}

#[test]
fn hypergraph_multiple_edge_types() {
    // Multiple edge types between same nodes
    // Alice -[knows]-> Bob
    // Alice -[worksW ith]-> Bob
    // Alice -[livesNear]-> Bob
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/worksWith"),
            Node::iri("http://example.org/bob"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/livesNear"),
            Node::iri("http://example.org/bob"),
        )
        .execute()
        .expect("Insert should succeed");

    // Find all relationship types
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?relationship WHERE {
                <http://example.org/alice> ?relationship <http://example.org/bob>
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 3, "Should find 3 different edge types");
}

#[test]
fn hypergraph_typed_edges() {
    // Hyperedges with typed nodes (RDF types)
    // Person nodes connected to Organization nodes
    let mut db = GraphDB::in_memory();

    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
            Node::iri("http://xmlns.com/foaf/0.1/Person"),
        )
        .triple(
            Node::iri("http://example.org/acme"),
            Node::iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
            Node::iri("http://xmlns.com/foaf/0.1/Organization"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/worksFor"),
            Node::iri("http://example.org/acme"),
        )
        .execute()
        .expect("Insert should succeed");

    // Find people working for organizations
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?person ?org WHERE {
                ?person a <http://xmlns.com/foaf/0.1/Person> .
                ?org a <http://xmlns.com/foaf/0.1/Organization> .
                ?person <http://example.org/worksFor> ?org
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find typed edge relationship");
}

#[test]
fn hypergraph_property_graph_pattern() {
    // Property graph pattern: Nodes with properties and edges
    // Simulates labeled property graph using RDF
    let mut db = GraphDB::in_memory();

    // Node properties
    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/label"),
            Node::literal("Person"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/firstName"),
            Node::literal("Alice"),
        )
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://example.org/age"),
            Node::integer(30),
        )
        .execute()
        .expect("Insert should succeed");

    // Edge with properties (reified triple)
    db.insert()
        .triple(
            Node::iri("http://example.org/alice"),
            Node::iri("http://xmlns.com/foaf/0.1/knows"),
            Node::iri("http://example.org/bob"),
        )
        .execute()
        .expect("Insert should succeed");

    // Query property graph
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?firstName ?age WHERE {
                ?person <http://example.org/firstName> ?firstName .
                ?person <http://example.org/age> ?age
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find property graph node");
}

#[test]
fn hypergraph_large_n_ary_simulation() {
    // Simulate N-ary relationship using multiple triples
    // Meeting: Alice, Bob, Charlie at location X at time T
    let mut db = GraphDB::in_memory();

    let meeting = Node::blank("meeting1");

    db.insert()
        .triple(
            meeting.clone(),
            Node::iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
            Node::iri("http://example.org/Meeting"),
        )
        .triple(
            meeting.clone(),
            Node::iri("http://example.org/participant"),
            Node::iri("http://example.org/alice"),
        )
        .triple(
            meeting.clone(),
            Node::iri("http://example.org/participant"),
            Node::iri("http://example.org/bob"),
        )
        .triple(
            meeting.clone(),
            Node::iri("http://example.org/participant"),
            Node::iri("http://example.org/charlie"),
        )
        .triple(
            meeting.clone(),
            Node::iri("http://example.org/location"),
            Node::literal("Conference Room A"),
        )
        .triple(
            meeting.clone(),
            Node::iri("http://example.org/time"),
            Node::literal("2024-01-15T10:00:00"),
        )
        .execute()
        .expect("Insert should succeed");

    // Query N-ary relationship
    let results = db
        .query()
        .sparql(
            r#"
            SELECT ?participant WHERE {
                ?meeting a <http://example.org/Meeting> .
                ?meeting <http://example.org/participant> ?participant
            }
        "#,
        )
        .execute()
        .expect("Query should succeed");

    assert_eq!(results.len(), 3, "Should find 3 meeting participants");
}
