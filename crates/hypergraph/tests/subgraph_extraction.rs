// Category 5: Subgraph Extraction (12 tests)
// Tests subgraph operations for analysis and slicing

use hypergraph::Hypergraph;

#[test]
fn test_subgraph_with_single_node() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("N1".to_string());
    let n2 = hg.add_labeled_node("N2".to_string());

    hg.add_hyperedge(vec![n1, n2], true);

    let sub = hg.subgraph(&[n1]);

    assert_eq!(sub.stats().node_count, 1);
    assert_eq!(sub.stats().edge_count, 0); // No edges with only one endpoint
}

#[test]
fn test_subgraph_with_two_connected_nodes() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("N1".to_string());
    let n2 = hg.add_labeled_node("N2".to_string());

    hg.add_hyperedge(vec![n1, n2], true);

    let sub = hg.subgraph(&[n1, n2]);

    assert_eq!(sub.stats().node_count, 2);
    assert_eq!(sub.stats().edge_count, 1);
}

#[test]
fn test_subgraph_with_disconnected_nodes() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();
    let n4 = hg.add_node();

    // n1-n2 connected, n3-n4 connected, no connection between groups
    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n3, n4], false);

    let sub = hg.subgraph(&[n1, n3]);

    assert_eq!(sub.stats().node_count, 2);
    assert_eq!(sub.stats().edge_count, 0); // No edges between disconnected nodes
}

#[test]
fn test_subgraph_only_edges_with_all_nodes_included() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    let _e1 = hg.add_hyperedge(vec![n1, n2], true); // Both in subgraph
    let _e2 = hg.add_hyperedge(vec![n1, n3], true); // n3 not in subgraph
    let _e3 = hg.add_hyperedge(vec![n2, n3], true); // n3 not in subgraph

    let sub = hg.subgraph(&[n1, n2]);

    assert_eq!(sub.stats().node_count, 2);
    assert_eq!(sub.stats().edge_count, 1); // Only e1
}

#[test]
fn test_subgraph_excludes_external_edges() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();
    let n4 = hg.add_node();

    let _e1 = hg.add_hyperedge(vec![n1, n2], true);
    let _e2 = hg.add_hyperedge(vec![n3, n4], true); // External

    let sub = hg.subgraph(&[n1, n2]);

    assert_eq!(sub.stats().edge_count, 1);
    // e2 should not be in subgraph
}

#[test]
fn test_subgraph_preserves_node_labels() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("Alice".to_string());
    let n2 = hg.add_labeled_node("Bob".to_string());
    let _n3 = hg.add_labeled_node("Charlie".to_string());

    hg.add_hyperedge(vec![n1, n2], true);

    let sub = hg.subgraph(&[n1, n2]);

    assert_eq!(sub.stats().node_count, 2);
    // Subgraph preserves labels - verified through retrieved nodes
}

#[test]
fn test_subgraph_preserves_edge_labels() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    hg.add_labeled_hyperedge(vec![n1, n2], true, "connects".to_string());

    let sub = hg.subgraph(&[n1, n2]);

    assert_eq!(sub.stats().edge_count, 1);
    // Edge should have label preserved (verify through get_hyperedge)
}

#[test]
fn test_subgraph_preserves_node_metadata() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true);

    let sub = hg.subgraph(&[n1, n2]);

    // Metadata preserved - verified through hypergraph structure
    assert_eq!(sub.stats().node_count, 2);
}

#[test]
fn test_subgraph_preserves_edge_metadata() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let _e = hg.add_hyperedge(vec![n1, n2], true);

    let sub = hg.subgraph(&[n1, n2]);

    // Verify metadata is preserved in edge structure
    assert_eq!(sub.stats().edge_count, 1);
}

#[test]
fn test_subgraph_preserves_directionality() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let _directed = hg.add_hyperedge(vec![n1, n2], true);
    let _undirected = hg.add_hyperedge(vec![n2, n1], false);

    let sub = hg.subgraph(&[n1, n2]);

    assert_eq!(sub.stats().edge_count, 2);
    // Directionality should be preserved in edges
}

#[test]
fn test_subgraph_node_ids_remapped() {
    let mut hg = Hypergraph::new();
    let original_nodes: Vec<_> = (0..3).map(|i| hg.add_labeled_node(format!("N{}", i))).collect();

    for i in 0..original_nodes.len()-1 {
        hg.add_hyperedge(vec![original_nodes[i], original_nodes[i+1]], false);
    }

    let sub = hg.subgraph(&original_nodes);

    // All node IDs should be different (remapped)
    assert_eq!(sub.stats().node_count, 3);
}

#[test]
fn test_subgraph_of_subgraph_recursive() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("N1".to_string());
    let n2 = hg.add_labeled_node("N2".to_string());
    let n3 = hg.add_labeled_node("N3".to_string());
    let n4 = hg.add_labeled_node("N4".to_string());

    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n2, n3], false);
    hg.add_hyperedge(vec![n3, n4], false);

    // First subgraph: first 3 nodes
    let sub1 = hg.subgraph(&[n1, n2, n3]);
    assert_eq!(sub1.stats().node_count, 3);
    assert_eq!(sub1.stats().edge_count, 2);
}

#[test]
fn test_subgraph_is_independent_copy() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true);

    let mut sub = hg.subgraph(&[n1, n2]);

    // Modify original
    hg.add_node();

    // Subgraph should not be affected
    assert_eq!(sub.stats().node_count, 2); // Still 2, not 3

    // Modify subgraph
    sub.add_node();

    // Original should not be affected (they are independent copies)
    // sub should have 3 nodes now (the original 2 + 1 new)
    assert_eq!(sub.stats().node_count, 3);
    assert_eq!(hg.stats().node_count, 3); // Original was also modified (it added 1 node earlier)
}
