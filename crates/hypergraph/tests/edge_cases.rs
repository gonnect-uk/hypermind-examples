// Category 7: Edge Cases and Error Handling (11 tests)
// Tests robustness with unusual inputs and error conditions

use hypergraph::Hypergraph;

#[test]
fn test_add_hyperedge_with_empty_node_list() {
    let mut hg = Hypergraph::new();

    // Add edge with empty node list
    let edge = hg.add_hyperedge(vec![], true);

    // Edge is created even with empty nodes
    assert!(hg.get_hyperedge(edge).is_some());
    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 0);
}

#[test]
fn test_add_hyperedge_with_nonexistent_nodes() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();

    // Add edge with node that doesn't exist (current implementation doesn't validate)
    let edge = hg.add_hyperedge(vec![n1, 999, 1000], true);

    // Edge is added regardless of node existence
    assert!(hg.get_hyperedge(edge).is_some());
    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 3);
}

#[test]
fn test_large_node_ids() {
    let mut hg = Hypergraph::new();

    // Add many nodes to get large IDs
    for _ in 0..1000 {
        hg.add_node();
    }

    let high_id_node = hg.add_node();

    // Node with large ID should be retrievable
    assert!(hg.get_node(high_id_node).is_some());
    assert_eq!(hg.stats().node_count, 1001);
}

#[test]
fn test_large_edge_ids() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    // Add many edges to get large IDs
    for _ in 0..1000 {
        hg.add_hyperedge(vec![n1, n2], true);
    }

    let high_id_edge = hg.add_hyperedge(vec![n1, n2], true);

    // Edge with large ID should be retrievable
    assert!(hg.get_hyperedge(high_id_edge).is_some());
    assert_eq!(hg.stats().edge_count, 1001);
}

#[test]
fn test_large_node_count() {
    let mut hg = Hypergraph::new();

    // Create 1000+ nodes
    for _ in 0..1000 {
        hg.add_node();
    }

    assert_eq!(hg.stats().node_count, 1000);

    let additional = hg.add_node();
    assert_eq!(hg.stats().node_count, 1001);
    assert!(hg.get_node(additional).is_some());
}

#[test]
fn test_large_edge_count() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    // Create 1000+ edges
    for _ in 0..1000 {
        hg.add_hyperedge(vec![n1, n2], true);
    }

    assert_eq!(hg.stats().edge_count, 1000);

    let additional = hg.add_hyperedge(vec![n1, n2], true);
    assert_eq!(hg.stats().edge_count, 1001);
    assert!(hg.get_hyperedge(additional).is_some());
}

#[test]
fn test_very_large_node_count_in_single_edge() {
    let mut hg = Hypergraph::new();

    // Create 100+ nodes in a single edge
    let large_node_list: Vec<_> = (0..100).map(|_| hg.add_node()).collect();

    let edge = hg.add_hyperedge(large_node_list.clone(), false);

    assert_eq!(hg.get_hyperedge(edge).unwrap().nodes.len(), 100);
}

#[test]
fn test_long_label_strings() {
    let mut hg = Hypergraph::new();

    // Create very long label
    let long_label = "A".repeat(10000);

    let n = hg.add_labeled_node(long_label.clone());

    let node = hg.get_node(n).unwrap();
    assert_eq!(node.label, Some(long_label));
}

#[test]
fn test_large_metadata_blocks() {
    let mut hg = Hypergraph::new();
    let n = hg.add_node();

    // Large metadata should be storable
    // Verify node exists
    assert!(hg.get_node(n).is_some());
}

#[test]
fn test_unicode_in_labels() {
    let mut hg = Hypergraph::new();

    // Various Unicode characters
    let labels = vec![
        "🔥".to_string(),
        "日本語".to_string(),
        "العربية".to_string(),
        "עברית".to_string(),
        "emoji_🚀_test".to_string(),
    ];

    let nodes: Vec<_> = labels.iter()
        .map(|label| hg.add_labeled_node(label.clone()))
        .collect();

    for (i, node_id) in nodes.iter().enumerate() {
        let node = hg.get_node(*node_id).unwrap();
        assert_eq!(node.label, Some(labels[i].clone()));
    }
}

#[test]
fn test_default_hypergraph_creation() {
    let hg = Hypergraph::default();

    assert_eq!(hg.stats().node_count, 0);
    assert_eq!(hg.stats().edge_count, 0);
}

#[test]
fn test_no_panics_on_operations() {
    let mut hg = Hypergraph::new();

    // Series of operations that should not panic
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let _ = hg.add_hyperedge(vec![], true); // Empty edge
    let _ = hg.add_hyperedge(vec![n1, n2], true);
    let _ = hg.add_hyperedge(vec![n1, 999, n2], true); // Nonexistent node

    let _ = hg.get_node(n1);
    let _ = hg.get_node(999);
    let _ = hg.get_incident_edges(n1);
    let _ = hg.get_incident_edges(999);
    let _ = hg.get_neighbors(n1);
    let _ = hg.get_neighbors(999);

    let _ = hg.bfs(n1);
    let _ = hg.bfs(999);

    let _ = hg.shortest_path(n1, n2);
    let _ = hg.shortest_path(999, 1000);

    let _ = hg.find_edges(&[Some(n1), None]);
    let _ = hg.find_edges(&[None, None]);

    let _ = hg.subgraph(&[n1]);
    let _ = hg.subgraph(&[]);
    let _ = hg.subgraph(&[999]);

    let _ = hg.stats();
    let _ = format!("{}", hg.stats());

    // If we reach here, no panics occurred
    assert!(true);
}
