//! Query Optimizer Integration Tests
//!
//! Verifies that the SPARQL optimizer correctly analyzes queries
//! and generates appropriate query plans.

use rdf_model::{Node, Quad};
use sparql::{Algebra, Executor, TriplePattern, VarOrNode, Variable, optimizer::JoinStrategy};
use storage::{InMemoryBackend, QuadStore};
use std::sync::Arc;

/// Helper to create a test quad store with sample data
fn create_test_store() -> QuadStore<InMemoryBackend> {
    let mut store = QuadStore::new(InMemoryBackend::new());
    let dict = Arc::clone(store.dictionary());

    // Create sample data: a simple social network
    let alice = Node::iri(dict.intern("http://example.org/alice"));
    let bob = Node::iri(dict.intern("http://example.org/bob"));
    let charlie = Node::iri(dict.intern("http://example.org/charlie"));

    let knows = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/knows"));
    let name = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/name"));
    let age = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/age"));
    let email = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/email"));

    // Names (3 people)
    store
        .insert(Quad::new(
            alice.clone(),
            name.clone(),
            Node::literal_str(dict.intern("Alice")),
            None,
        ))
        .unwrap();

    store
        .insert(Quad::new(
            bob.clone(),
            name.clone(),
            Node::literal_str(dict.intern("Bob")),
            None,
        ))
        .unwrap();

    store
        .insert(Quad::new(
            charlie.clone(),
            name.clone(),
            Node::literal_str(dict.intern("Charlie")),
            None,
        ))
        .unwrap();

    // Ages (only Alice and Bob have ages)
    store
        .insert(Quad::new(
            alice.clone(),
            age.clone(),
            Node::literal_typed(
                dict.intern("30"),
                dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
            ),
            None,
        ))
        .unwrap();

    store
        .insert(Quad::new(
            bob.clone(),
            age.clone(),
            Node::literal_typed(
                dict.intern("25"),
                dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
            ),
            None,
        ))
        .unwrap();

    // Emails (only Alice and Bob have emails)
    store
        .insert(Quad::new(
            alice.clone(),
            email.clone(),
            Node::literal_str(dict.intern("alice@example.org")),
            None,
        ))
        .unwrap();

    store
        .insert(Quad::new(
            bob.clone(),
            email.clone(),
            Node::literal_str(dict.intern("bob@example.org")),
            None,
        ))
        .unwrap();

    // Social connections
    store
        .insert(Quad::new(alice.clone(), knows.clone(), bob.clone(), None))
        .unwrap();

    store
        .insert(Quad::new(
            alice.clone(),
            knows.clone(),
            charlie.clone(),
            None,
        ))
        .unwrap();

    store
        .insert(Quad::new(bob.clone(), knows.clone(), charlie.clone(), None))
        .unwrap();

    store
}

#[test]
fn test_single_pattern_uses_nested_loop() {
    // Single pattern should use nested loop
    let store = create_test_store();
    let dict = Arc::clone(store.dictionary());
    let mut executor = Executor::new(&store);

    let patterns = vec![TriplePattern {
        subject: VarOrNode::Var(Variable::new("person")),
        predicate: VarOrNode::Node(Node::iri(
            dict.intern("http://xmlns.com/foaf/0.1/name"),
        )),
        object: VarOrNode::Var(Variable::new("name")),
    }];

    let bgp = Algebra::BGP(patterns);
    let results = executor.execute(&bgp).unwrap();

    // Should find 3 people
    assert_eq!(results.len(), 3, "Should find 3 people with names");

    // Verify query plan used nested loop
    let plan = executor.get_query_plan().unwrap();
    assert_eq!(plan.strategy, JoinStrategy::NestedLoop);
}

#[test]
fn test_two_pattern_query() {
    // Two-pattern star query (shared variable makes it a star)
    let store = create_test_store();
    let dict = Arc::clone(store.dictionary());
    let mut executor = Executor::new(&store);

    let patterns = vec![
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/name"),
            )),
            object: VarOrNode::Var(Variable::new("name")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/age"),
            )),
            object: VarOrNode::Var(Variable::new("age")),
        },
    ];

    let bgp = Algebra::BGP(patterns);
    let results = executor.execute(&bgp).unwrap();

    assert_eq!(results.len(), 2, "Should find 2 people with name and age");

    let plan = executor.get_query_plan().unwrap();

    // This is a star query (shared ?person variable)
    assert!(plan.analysis.is_star, "Should detect star pattern");

    // Star queries use WCOJ
    assert_eq!(plan.strategy, JoinStrategy::WCOJ);
}

#[test]
fn test_three_pattern_star_query() {
    // 3-pattern star query (below threshold of 4, uses nested loop)
    let store = create_test_store();
    let dict = Arc::clone(store.dictionary());
    let mut executor = Executor::new(&store);

    let patterns = vec![
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/name"),
            )),
            object: VarOrNode::Var(Variable::new("n")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/age"),
            )),
            object: VarOrNode::Var(Variable::new("a")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/email"),
            )),
            object: VarOrNode::Var(Variable::new("e")),
        },
    ];

    let bgp = Algebra::BGP(patterns.clone());
    let results = executor.execute(&bgp).unwrap();

    assert_eq!(results.len(), 2, "Should find 2 people with all properties");

    // Verify optimizer analysis
    let plan = executor.get_query_plan().unwrap();

    // Optimizer should detect star pattern
    assert!(
        plan.analysis.is_star,
        "Optimizer should detect 3-pattern star query"
    );

    // Star queries always use WCOJ (they benefit most from it)
    assert_eq!(
        plan.strategy,
        JoinStrategy::WCOJ,
        "Star queries should always use WCOJ strategy"
    );
}

#[test]
fn test_four_pattern_star_query_triggers_wcoj() {
    // 4-pattern star query (meets threshold, optimizer recommends WCOJ)
    let store = create_test_store();
    let dict = Arc::clone(store.dictionary());
    let mut executor = Executor::new(&store);

    let patterns = vec![
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("p")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/name"),
            )),
            object: VarOrNode::Var(Variable::new("n")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("p")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/age"),
            )),
            object: VarOrNode::Var(Variable::new("a")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("p")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/email"),
            )),
            object: VarOrNode::Var(Variable::new("e")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("p")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/knows"),
            )),
            object: VarOrNode::Var(Variable::new("friend")),
        },
    ];

    let bgp = Algebra::BGP(patterns);
    let results = executor.execute(&bgp).unwrap();

    // Should find Alice and Bob (both have all 4 properties)
    assert!(results.len() >= 2, "Should find at least 2 people");

    let plan = executor.get_query_plan().unwrap();

    // Optimizer should detect star pattern and recommend WCOJ
    assert!(
        plan.analysis.is_star,
        "Optimizer should detect 4-pattern star query"
    );

    assert_eq!(
        plan.strategy,
        JoinStrategy::WCOJ,
        "4+ pattern star query should recommend WCOJ"
    );

    println!("4-pattern query plan:\n{}", plan.explanation);
}

#[test]
fn test_query_plan_api() {
    // Test the query plan visualization API
    let store = create_test_store();
    let dict = Arc::clone(store.dictionary());
    let mut executor = Executor::new(&store);

    let patterns = vec![
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("p")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/name"),
            )),
            object: VarOrNode::Var(Variable::new("n")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("p")),
            predicate: VarOrNode::Node(Node::iri(
                dict.intern("http://xmlns.com/foaf/0.1/age"),
            )),
            object: VarOrNode::Var(Variable::new("a")),
        },
    ];

    // Test explain() API (without executing)
    let explanation = executor.explain(&patterns);
    assert!(!explanation.is_empty(), "Explanation should not be empty");
    println!("Explain API output:\n{}", explanation);

    // Execute query
    let bgp = Algebra::BGP(patterns);
    let _ = executor.execute(&bgp).unwrap();

    // Get plan after execution
    let plan = executor.get_query_plan();
    assert!(plan.is_some(), "Plan should be available after execution");

    let plan = plan.unwrap();

    // Verify plan contains useful information
    assert!(!plan.explanation.is_empty());
    assert!(plan.estimated_cardinality > 0);
    assert!(plan.estimated_cost > 0.0);

    println!("\nQuery plan after execution:");
    println!("  Strategy: {:?}", plan.strategy);
    println!("  Star query: {}", plan.analysis.is_star);
    println!("  Cyclic query: {}", plan.analysis.is_cyclic);
    println!("  Estimated cardinality: {}", plan.estimated_cardinality);
    println!("  Estimated cost: {:.2}", plan.estimated_cost);
    println!("  Explanation:\n{}", plan.explanation);
}

#[test]
fn test_optimizer_detects_star_pattern() {
    // Verify optimizer correctly identifies star patterns
    let store = create_test_store();
    let dict = Arc::clone(store.dictionary());
    let mut executor = Executor::new(&store);

    // Create clear star pattern: central ?person variable
    let patterns = vec![
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://p1"))),
            object: VarOrNode::Var(Variable::new("o1")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://p2"))),
            object: VarOrNode::Var(Variable::new("o2")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://p3"))),
            object: VarOrNode::Var(Variable::new("o3")),
        },
    ];

    let bgp = Algebra::BGP(patterns);
    let _ = executor.execute(&bgp);

    let plan = executor.get_query_plan().unwrap();
    assert!(
        plan.analysis.is_star,
        "Should detect star pattern with shared variable"
    );
}

#[test]
fn test_multi_hop_join_query() {
    // Friend-of-friend pattern
    let store = create_test_store();
    let dict = Arc::clone(store.dictionary());
    let mut executor = Executor::new(&store);

    let knows_uri = dict.intern("http://xmlns.com/foaf/0.1/knows");

    let patterns = vec![
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("person1")),
            predicate: VarOrNode::Node(Node::iri(knows_uri)),
            object: VarOrNode::Var(Variable::new("friend1")),
        },
        TriplePattern {
            subject: VarOrNode::Var(Variable::new("friend1")),
            predicate: VarOrNode::Node(Node::iri(knows_uri)),
            object: VarOrNode::Var(Variable::new("friend2")),
        },
    ];

    let bgp = Algebra::BGP(patterns);
    let results = executor.execute(&bgp).unwrap();

    assert!(results.len() > 0, "Should find friend-of-friend connections");

    let plan = executor.get_query_plan().unwrap();
    println!("Multi-hop query plan:\n{}", plan.explanation);
}
