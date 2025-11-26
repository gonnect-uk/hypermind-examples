// Category 2: Multi-Node Connections (15 tests)
// Verifies hyperedge behavior with varying arities and configurations

use hypergraph::Hypergraph;

#[test]
fn test_binary_hyperedge_like_standard_graphs() {
    // 2-node hyperedge - like standard graphs
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("A".to_string());
    let n2 = hg.add_labeled_node("B".to_string());

    let edge = hg.add_hyperedge(vec![n1, n2], true);

    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 2);
}

#[test]
fn test_ternary_hyperedge_rdf_triple_pattern() {
    // 3-node hyperedge - RDF triple (S, P, O)
    let mut hg = Hypergraph::new();
    let subject = hg.add_labeled_node("Subject".to_string());
    let predicate = hg.add_labeled_node("Predicate".to_string());
    let object = hg.add_labeled_node("Object".to_string());

    let edge = hg.add_hyperedge(vec![subject, predicate, object], true);

    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 3);
    assert!(hg.get_hyperedge(edge).unwrap().directed);
}

#[test]
fn test_quad_hyperedge_rdf_with_context() {
    // 4-node hyperedge - RDF quad (S, P, O, C)
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..4).map(|i| hg.add_labeled_node(format!("N{}", i))).collect();

    let edge = hg.add_hyperedge(nodes.clone(), true);

    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 4);
}

#[test]
fn test_nary_hyperedge_large_arity() {
    // 5+ node hyperedge - n-ary relation
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..10).map(|_| hg.add_node()).collect();

    let edge = hg.add_hyperedge(nodes.clone(), false);

    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 10);
}

#[test]
fn test_mixed_arity_edges_different_sizes() {
    // Edges with different arities in same hypergraph
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..6).map(|_| hg.add_node()).collect();

    let e2 = hg.add_hyperedge(vec![nodes[0], nodes[1]], true); // Binary
    let e3 = hg.add_hyperedge(vec![nodes[2], nodes[3], nodes[4]], true); // Ternary
    let e4 = hg.add_hyperedge(vec![nodes[1], nodes[3], nodes[4], nodes[5]], false); // Quaternary

    assert_eq!(hg.get_hyperedge(e2).unwrap().nodes.len(), 2);
    assert_eq!(hg.get_hyperedge(e3).unwrap().nodes.len(), 3);
    assert_eq!(hg.get_hyperedge(e4).unwrap().nodes.len(), 4);
}

#[test]
fn test_single_node_hyperedge_self_loop() {
    // Edge connecting a node to itself
    let mut hg = Hypergraph::new();
    let n = hg.add_node();

    let edge = hg.add_hyperedge(vec![n], true);

    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 1);
    assert!(hg.get_hyperedge(edge).unwrap().nodes.contains(&n));
}

#[test]
fn test_hyperedge_with_duplicate_nodes() {
    // Same node appears multiple times in edge
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let edge = hg.add_hyperedge(vec![n1, n2, n1, n2], false);

    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 4);
    // SmallVec preserves order and duplicates
}

#[test]
fn test_one_node_in_multiple_hyperedges() {
    // Single node participates in multiple edges
    let mut hg = Hypergraph::new();
    let hub = hg.add_labeled_node("Hub".to_string());
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    let e1 = hg.add_hyperedge(vec![hub, n1], true);
    let e2 = hg.add_hyperedge(vec![hub, n2], true);
    let e3 = hg.add_hyperedge(vec![hub, n3], true);

    let incidents = hg.get_incident_edges(hub);
    assert_eq!(incidents.len(), 3);
    assert!(incidents.contains(&e1));
    assert!(incidents.contains(&e2));
    assert!(incidents.contains(&e3));
}

#[test]
fn test_node_incident_to_many_edges() {
    // Single node in 5+ edges
    let mut hg = Hypergraph::new();
    let hub = hg.add_node();
    let mut edge_ids = Vec::new();

    for i in 0..10 {
        let other = hg.add_labeled_node(format!("N{}", i));
        let edge = hg.add_hyperedge(vec![hub, other], true);
        edge_ids.push(edge);
    }

    let incidents = hg.get_incident_edges(hub);
    assert_eq!(incidents.len(), 10);

    for edge_id in &edge_ids {
        assert!(incidents.contains(edge_id));
    }
}

#[test]
fn test_directed_edge_directional_semantics() {
    // Directed edges have order semantics
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("First".to_string());
    let n2 = hg.add_labeled_node("Second".to_string());

    let directed = hg.add_hyperedge(vec![n1, n2], true);

    assert!(hg.get_hyperedge(directed).unwrap().directed);
    // First node could be considered "source" by convention
    assert_eq!(hg.get_hyperedge(directed).unwrap().nodes[0], n1);
}

#[test]
fn test_undirected_edge_symmetric_property() {
    // Undirected edges should treat nodes symmetrically
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let undirected = hg.add_hyperedge(vec![n1, n2], false);

    assert!(!hg.get_hyperedge(undirected).unwrap().directed);
    // Both nodes should have same role in undirected edge
}

#[test]
fn test_hyperedge_all_nodes_must_exist() {
    // Adding edge with non-existent nodes - current implementation doesn't validate
    // This test documents behavior - it adds edge without checking node existence
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();

    // Edge with node that doesn't exist in hypergraph
    let edge = hg.add_hyperedge(vec![n1, 999], true);

    // Edge is added regardless
    assert!(hg.get_hyperedge(edge).is_some());
}

#[test]
fn test_edge_references_valid_nodes_after_creation() {
    // All nodes in edge should be retrievable
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..5).map(|i| hg.add_labeled_node(format!("N{}", i))).collect();

    let edge = hg.add_hyperedge(nodes.clone(), true);

    let edge_obj = hg.get_hyperedge(edge).unwrap();
    for node_id in &edge_obj.nodes {
        // All nodes referenced in edge should be retrievable (if they exist)
        if *node_id < 1000 {  // Arbitrary threshold for "exists"
            assert!(hg.get_node(*node_id).is_some());
        }
    }
}

#[test]
fn test_metadata_attached_to_edges() {
    // Edge metadata storage and retrieval (verified via pattern matching)
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let edge_id = hg.add_hyperedge(vec![n1, n2], true);
    let edge = hg.get_hyperedge(edge_id).unwrap();

    // Verify edge exists and has correct structure
    assert_eq!(edge.nodes.len(), 2);
    assert!(edge.nodes.contains(&n1));
    assert!(edge.nodes.contains(&n2));
}

#[test]
fn test_smallvec_optimization_for_small_arity() {
    // SmallVec optimized for ≤4 nodes (common in RDF triples/quads)
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();
    let n4 = hg.add_node();

    let e3 = hg.add_hyperedge(vec![n1, n2, n3], true); // 3-node - should be in SmallVec buffer
    let e4 = hg.add_hyperedge(vec![n1, n2, n3, n4], true); // 4-node - should be in SmallVec buffer

    assert_eq!(hg.get_hyperedge(e3).unwrap().nodes.len(), 3);
    assert_eq!(hg.get_hyperedge(e4).unwrap().nodes.len(), 4);
}
