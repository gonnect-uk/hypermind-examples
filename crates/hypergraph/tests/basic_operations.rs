// Category 1: Basic Hyperedge Operations (20 tests)
// Tests core functionality of node and edge creation/retrieval

use hypergraph::Hypergraph;

#[test]
fn test_create_empty_hypergraph() {
    let hg = Hypergraph::new();
    let stats = hg.stats();
    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
}

#[test]
fn test_add_single_node() {
    let mut hg = Hypergraph::new();
    let node_id = hg.add_node();

    assert_eq!(hg.stats().node_count, 1);
    assert!(hg.get_node(node_id).is_some());
}

#[test]
fn test_add_multiple_nodes_with_unique_ids() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    assert_eq!(hg.stats().node_count, 3);
    assert_ne!(n1, n2);
    assert_ne!(n2, n3);
    assert_ne!(n1, n3);
}

#[test]
fn test_add_labeled_node() {
    let mut hg = Hypergraph::new();
    let node_id = hg.add_labeled_node("Alice".to_string());

    let node = hg.get_node(node_id).unwrap();
    assert_eq!(node.label, Some("Alice".to_string()));
}

#[test]
fn test_add_multiple_labeled_nodes() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let charlie = hg.add_labeled_node("Charlie".to_string());

    assert_eq!(hg.get_node(alice).unwrap().label, Some("Alice".to_string()));
    assert_eq!(hg.get_node(bob).unwrap().label, Some("Bob".to_string()));
    assert_eq!(hg.get_node(charlie).unwrap().label, Some("Charlie".to_string()));
}

#[test]
fn test_retrieve_node_by_id() {
    let mut hg = Hypergraph::new();
    let node_id = hg.add_labeled_node("Test".to_string());

    let retrieved = hg.get_node(node_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, node_id);
}

#[test]
fn test_retrieve_non_existent_node_returns_none() {
    let hg = Hypergraph::new();
    let result = hg.get_node(999);

    assert!(result.is_none());
}

#[test]
fn test_add_binary_hyperedge() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let edge_id = hg.add_hyperedge(vec![n1, n2], true);

    assert_eq!(hg.stats().edge_count, 1);
    let edge = hg.get_hyperedge(edge_id).unwrap();
    assert_eq!(edge.nodes.len(), 2);
    assert!(edge.nodes.contains(&n1));
    assert!(edge.nodes.contains(&n2));
}

#[test]
fn test_add_ternary_hyperedge() {
    let mut hg = Hypergraph::new();
    let alice = hg.add_labeled_node("Alice".to_string());
    let knows = hg.add_labeled_node("knows".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());

    let edge_id = hg.add_hyperedge(vec![alice, knows, bob], true);

    assert_eq!(hg.stats().edge_count, 1);
    let edge = hg.get_hyperedge(edge_id).unwrap();
    assert_eq!(edge.nodes.len(), 3);
}

#[test]
fn test_add_nary_hyperedge() {
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..6).map(|_| hg.add_node()).collect();

    let edge_id = hg.add_hyperedge(nodes.clone(), false);

    let edge = hg.get_hyperedge(edge_id).unwrap();
    assert_eq!(edge.nodes.len(), 6);
}

#[test]
fn test_retrieve_hyperedge_by_id() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let edge_id = hg.add_hyperedge(vec![n1, n2], true);
    let edge = hg.get_hyperedge(edge_id);

    assert!(edge.is_some());
    assert_eq!(edge.unwrap().id, edge_id);
}

#[test]
fn test_retrieve_non_existent_edge_returns_none() {
    let hg = Hypergraph::new();
    let result = hg.get_hyperedge(999);

    assert!(result.is_none());
}

#[test]
fn test_add_directed_hyperedge() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let edge = hg.add_hyperedge(vec![n1, n2], true);

    assert!(hg.get_hyperedge(edge).unwrap().directed);
}

#[test]
fn test_add_undirected_hyperedge() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let edge = hg.add_hyperedge(vec![n1, n2], false);

    assert!(!hg.get_hyperedge(edge).unwrap().directed);
}

#[test]
fn test_add_labeled_hyperedge() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let edge_id = hg.add_labeled_hyperedge(vec![n1, n2], true, "connects".to_string());

    let edge = hg.get_hyperedge(edge_id).unwrap();
    assert_eq!(edge.label, Some("connects".to_string()));
}

#[test]
fn test_edge_id_uniqueness() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    let e1 = hg.add_hyperedge(vec![n1, n2], true);
    let e2 = hg.add_hyperedge(vec![n2, n3], true);
    let e3 = hg.add_hyperedge(vec![n1, n3], true);

    assert_ne!(e1, e2);
    assert_ne!(e2, e3);
    assert_ne!(e1, e3);
}

#[test]
fn test_node_id_uniqueness() {
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..10).map(|_| hg.add_node()).collect();

    for i in 0..nodes.len() {
        for j in (i+1)..nodes.len() {
            assert_ne!(nodes[i], nodes[j]);
        }
    }
}

#[test]
fn test_hyperedge_contains_correct_nodes() {
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..5).map(|i| hg.add_labeled_node(format!("N{}", i))).collect();

    let edge_id = hg.add_hyperedge(nodes.clone(), true);
    let edge = hg.get_hyperedge(edge_id).unwrap();

    for node_id in &nodes {
        assert!(edge.nodes.contains(node_id));
    }
}

#[test]
fn test_multiple_hyperedges_with_same_nodes() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let e1 = hg.add_hyperedge(vec![n1, n2], true);
    let e2 = hg.add_hyperedge(vec![n1, n2], false); // Same nodes, different direction

    assert_ne!(e1, e2);
    assert_eq!(hg.stats().edge_count, 2);
}
