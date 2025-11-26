// Category 4: Pattern Matching and Queries (15 tests)
// Tests flexible pattern matching with wildcards (essential for SPARQL)

use hypergraph::Hypergraph;

#[test]
fn test_find_edges_with_fixed_first_position() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let knows = hg.add_labeled_node("knows".to_string());

    hg.add_hyperedge(vec![alice, knows, bob], true);

    // Find all edges where first node is Alice
    let matches = hg.find_edges(&[Some(alice), None, None]);

    assert_eq!(matches.len(), 1);
}

#[test]
fn test_find_edges_with_fixed_middle_position() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let charlie = hg.add_labeled_node("Charlie".to_string());
    let knows = hg.add_labeled_node("knows".to_string());

    hg.add_hyperedge(vec![alice, knows, bob], true);
    hg.add_hyperedge(vec![charlie, knows, alice], true);

    // Find all edges where middle node (predicate) is "knows"
    let matches = hg.find_edges(&[None, Some(knows), None]);

    assert_eq!(matches.len(), 2);
}

#[test]
fn test_find_edges_with_fixed_last_position() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let charlie = hg.add_labeled_node("Charlie".to_string());
    let knows = hg.add_labeled_node("knows".to_string());

    hg.add_hyperedge(vec![alice, knows, bob], true);
    hg.add_hyperedge(vec![charlie, knows, bob], true);

    // Find all edges where object is Bob
    let matches = hg.find_edges(&[None, None, Some(bob)]);

    assert_eq!(matches.len(), 2);
}

#[test]
fn test_find_edges_with_all_wildcards() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true);
    hg.add_hyperedge(vec![n2, n3], true);
    hg.add_hyperedge(vec![n1, n3], true);

    // Find all 2-node edges
    let matches = hg.find_edges(&[None, None]);

    assert_eq!(matches.len(), 3);
}

#[test]
fn test_find_edges_with_two_fixed_positions() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let charlie = hg.add_labeled_node("Charlie".to_string());
    let knows = hg.add_labeled_node("knows".to_string());

    hg.add_hyperedge(vec![alice, knows, bob], true);
    hg.add_hyperedge(vec![alice, knows, charlie], true);
    hg.add_hyperedge(vec![bob, knows, alice], true);

    // Find edges with Alice as subject and knows as predicate
    let matches = hg.find_edges(&[Some(alice), Some(knows), None]);

    assert_eq!(matches.len(), 2);
}

#[test]
fn test_find_edges_with_one_position_match() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let charlie = hg.add_labeled_node("Charlie".to_string());

    hg.add_hyperedge(vec![alice, bob], true);
    hg.add_hyperedge(vec![alice, charlie], true);
    hg.add_hyperedge(vec![bob, charlie], true);

    let matches = hg.find_edges(&[Some(alice), None]);

    assert_eq!(matches.len(), 2);
}

#[test]
fn test_pattern_match_no_matches_returns_empty() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let charlie = hg.add_labeled_node("Charlie".to_string());

    hg.add_hyperedge(vec![alice, bob], true);

    let matches = hg.find_edges(&[Some(charlie), None]);

    assert_eq!(matches.len(), 0);
}

#[test]
fn test_pattern_match_ternary_edges() {
    let mut hg = Hypergraph::new();
    let s1 = hg.add_labeled_node("S1".to_string());
    let p1 = hg.add_labeled_node("P1".to_string());
    let o1 = hg.add_labeled_node("O1".to_string());
    let o2 = hg.add_labeled_node("O2".to_string());

    hg.add_hyperedge(vec![s1, p1, o1], true);
    hg.add_hyperedge(vec![s1, p1, o2], true);

    let matches = hg.find_edges(&[Some(s1), Some(p1), None]);

    assert_eq!(matches.len(), 2);
}

#[test]
fn test_pattern_match_binary_edges() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n1, n3], false);

    let matches = hg.find_edges(&[Some(n1), None]);

    assert_eq!(matches.len(), 2);
}

#[test]
fn test_pattern_match_nary_edges() {
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..5).map(|i| hg.add_labeled_node(format!("N{}", i))).collect();

    hg.add_hyperedge(nodes[0..5].to_vec(), false);

    let matches = hg.find_edges(&[Some(nodes[0]), None, None, None, None]);

    assert_eq!(matches.len(), 1);
}

#[test]
fn test_multiple_edges_matching_same_pattern() {
    let mut hg = Hypergraph::new();
    let predicate = hg.add_labeled_node("hasFriend".to_string());
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let charlie = hg.add_labeled_node("Charlie".to_string());

    hg.add_hyperedge(vec![alice, predicate, bob], true);
    hg.add_hyperedge(vec![alice, predicate, charlie], true);
    hg.add_hyperedge(vec![bob, predicate, alice], true);

    let matches = hg.find_edges(&[None, Some(predicate), None]);

    assert_eq!(matches.len(), 3);
}

#[test]
fn test_pattern_match_with_labeled_edges() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_labeled_hyperedge(vec![n1, n2], true, "connects".to_string());
    hg.add_labeled_hyperedge(vec![n1, n3], true, "connects".to_string());

    let matches = hg.find_edges(&[Some(n1), None]);

    assert_eq!(matches.len(), 2);
    // Labels don't affect pattern matching - based on nodes only
}

#[test]
fn test_wildcard_position_independence() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();
    let n4 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2, n3], true);
    hg.add_hyperedge(vec![n1, n4, n3], true);

    // All wildcards gives all ternary edges
    let all = hg.find_edges(&[None, None, None]);
    assert_eq!(all.len(), 2);

    // Fix middle position
    let fixed_middle = hg.find_edges(&[None, Some(n2), None]);
    assert_eq!(fixed_middle.len(), 1);

    // Fix first and last
    let fixed_ends = hg.find_edges(&[Some(n1), None, Some(n3)]);
    assert_eq!(fixed_ends.len(), 2);
}

#[test]
fn test_pattern_different_arity_no_matches() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    // Add ternary edge
    hg.add_hyperedge(vec![n1, n2, n3], true);

    // Query with 2-node pattern
    let matches = hg.find_edges(&[Some(n1), None]);

    assert_eq!(matches.len(), 0); // Arity mismatch
}

#[test]
fn test_pattern_matching_is_deterministic() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("N1".to_string());
    let n2 = hg.add_labeled_node("N2".to_string());
    let n3 = hg.add_labeled_node("N3".to_string());

    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n1, n3], false);

    // Run query multiple times
    let matches1 = hg.find_edges(&[Some(n1), None]);
    let matches2 = hg.find_edges(&[Some(n1), None]);
    let matches3 = hg.find_edges(&[Some(n1), None]);

    assert_eq!(matches1, matches2);
    assert_eq!(matches2, matches3);
    assert_eq!(matches1.len(), 2);
}
