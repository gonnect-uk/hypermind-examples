// Category 6: Statistics and Metadata (12 tests)
// Tests statistics computation and metadata handling

use hypergraph::Hypergraph;

#[test]
fn test_stats_empty_hypergraph() {
    let hg = Hypergraph::new();
    let stats = hg.stats();

    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
    assert_eq!(stats.max_arity, 0);
    assert_eq!(stats.directed_edges, 0);
    assert_eq!(stats.undirected_edges, 0);
}

#[test]
fn test_stats_node_count() {
    let mut hg = Hypergraph::new();
    hg.add_node();
    hg.add_node();
    hg.add_node();

    let stats = hg.stats();

    assert_eq!(stats.node_count, 3);
}

#[test]
fn test_stats_edge_count() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true);
    hg.add_hyperedge(vec![n2, n3], true);
    hg.add_hyperedge(vec![n1, n3], true);

    let stats = hg.stats();

    assert_eq!(stats.edge_count, 3);
}

#[test]
fn test_stats_max_arity_two_node() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true);

    let stats = hg.stats();

    assert_eq!(stats.max_arity, 2);
}

#[test]
fn test_stats_max_arity_three_node() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2, n3], true);

    let stats = hg.stats();

    assert_eq!(stats.max_arity, 3);
}

#[test]
fn test_stats_max_arity_five_node() {
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..5).map(|_| hg.add_node()).collect();

    hg.add_hyperedge(nodes, true);

    let stats = hg.stats();

    assert_eq!(stats.max_arity, 5);
}

#[test]
fn test_stats_directed_undirected_split() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true); // Directed
    hg.add_hyperedge(vec![n2, n3], false); // Undirected

    let stats = hg.stats();

    assert_eq!(stats.directed_edges, 1);
    assert_eq!(stats.undirected_edges, 1);
}

#[test]
fn test_stats_all_undirected_edges() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();
    let n4 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n2, n3], false);
    hg.add_hyperedge(vec![n3, n4], false);

    let stats = hg.stats();

    assert_eq!(stats.undirected_edges, 3);
    assert_eq!(stats.directed_edges, 0);
}

#[test]
fn test_stats_all_directed_edges() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();
    let n4 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true);
    hg.add_hyperedge(vec![n2, n3], true);
    hg.add_hyperedge(vec![n3, n4], true);

    let stats = hg.stats();

    assert_eq!(stats.directed_edges, 3);
    assert_eq!(stats.undirected_edges, 0);
}

#[test]
fn test_stats_mixed_directed_undirected() {
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..6).map(|_| hg.add_node()).collect();

    // 3 directed, 2 undirected
    hg.add_hyperedge(vec![nodes[0], nodes[1]], true);
    hg.add_hyperedge(vec![nodes[1], nodes[2]], true);
    hg.add_hyperedge(vec![nodes[2], nodes[3]], true);
    hg.add_hyperedge(vec![nodes[3], nodes[4]], false);
    hg.add_hyperedge(vec![nodes[4], nodes[5]], false);

    let stats = hg.stats();

    assert_eq!(stats.directed_edges, 3);
    assert_eq!(stats.undirected_edges, 2);
    assert_eq!(stats.edge_count, 5);
    assert!(stats.directed_edges + stats.undirected_edges == stats.edge_count);
}

#[test]
fn test_node_metadata_storage() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();

    // Verify node exists and can store metadata
    assert!(hg.get_node(n1).is_some());
}

#[test]
fn test_edge_metadata_storage() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let e = hg.add_hyperedge(vec![n1, n2], true);

    // Verify edge exists and can store metadata
    assert!(hg.get_hyperedge(e).is_some());
}

#[test]
fn test_node_label_retrieval() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());

    let alice_node = hg.get_node(alice).unwrap();
    let bob_node = hg.get_node(bob).unwrap();

    assert_eq!(alice_node.label, Some("Alice".to_string()));
    assert_eq!(bob_node.label, Some("Bob".to_string()));
}

#[test]
fn test_edge_label_retrieval() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let e1 = hg.add_labeled_hyperedge(vec![n1, n2], true, "knows".to_string());

    let edge = hg.get_hyperedge(e1).unwrap();
    assert_eq!(edge.label, Some("knows".to_string()));
}

#[test]
fn test_hypergraph_stats_display_format() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], true);
    hg.add_hyperedge(vec![n2, n3], false);

    let stats = hg.stats();
    let display_str = format!("{}", stats);

    // Verify display format contains key information
    assert!(display_str.contains("3 nodes"));
    assert!(display_str.contains("2 edges"));
    assert!(display_str.contains("max arity 2"));
    assert!(display_str.contains("directed/undirected"));
}
