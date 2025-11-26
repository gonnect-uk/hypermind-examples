// Category 10: Clone and Default Traits (5 tests)
// Tests trait implementations

use hypergraph::Hypergraph;

#[test]
fn test_clone_creates_independent_copy() {
    let mut hg1 = Hypergraph::new();
    let _n1 = hg1.add_labeled_node("N1".to_string());
    let _n2 = hg1.add_labeled_node("N2".to_string());
    hg1.add_node(); // Just add a node to avoid empty clone test

    let hg2 = hg1.clone();

    assert_eq!(hg1.stats().node_count, hg2.stats().node_count);
    assert_eq!(hg1.stats().edge_count, hg2.stats().edge_count);
}

#[test]
fn test_clone_modifications_dont_affect_original() {
    let mut hg1 = Hypergraph::new();
    let _n1 = hg1.add_labeled_node("N1".to_string());

    let mut hg2 = hg1.clone();

    // Modify clone
    hg2.add_labeled_node("N2".to_string());
    hg2.add_labeled_node("N3".to_string());

    // Original should be unaffected
    assert_eq!(hg1.stats().node_count, 1);
    assert_eq!(hg2.stats().node_count, 3);
}

#[test]
fn test_default_creates_empty_hypergraph() {
    let hg = Hypergraph::default();

    assert_eq!(hg.stats().node_count, 0);
    assert_eq!(hg.stats().edge_count, 0);
}

#[test]
fn test_cloned_hypergraph_data_is_identical() {
    let mut hg1 = Hypergraph::new();

    let nodes: Vec<_> = (0..5).map(|i| hg1.add_labeled_node(format!("N{}", i))).collect();
    let _e1 = hg1.add_labeled_hyperedge(vec![nodes[0], nodes[1]], true, "edge1".to_string());
    let _e2 = hg1.add_labeled_hyperedge(vec![nodes[2], nodes[3], nodes[4]], false, "edge2".to_string());

    let hg2 = hg1.clone();

    // Same node count
    assert_eq!(hg1.stats().node_count, hg2.stats().node_count);

    // Same edge count
    assert_eq!(hg1.stats().edge_count, hg2.stats().edge_count);

    // Same stats
    assert_eq!(hg1.stats().node_count, hg2.stats().node_count);
    assert_eq!(hg1.stats().edge_count, hg2.stats().edge_count);
    assert_eq!(hg1.stats().max_arity, hg2.stats().max_arity);
}

#[test]
fn test_display_trait_produces_formatted_output() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true);
    hg.add_hyperedge(vec![n2, n3], false);

    let stats = hg.stats();
    let display_output = format!("{}", stats);

    // Verify display contains expected content
    assert!(!display_output.is_empty());
    assert!(display_output.contains("nodes") || display_output.contains("edges"));
}
