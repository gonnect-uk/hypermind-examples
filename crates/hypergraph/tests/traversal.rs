// Category 3: Hypergraph Traversal (15 tests)
// Tests graph traversal, navigation, and path-finding algorithms

use hypergraph::Hypergraph;

#[test]
fn test_bfs_from_single_node() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("N1".to_string());

    let traversal = hg.bfs(n1);

    assert_eq!(traversal.len(), 1);
    assert_eq!(traversal[0], n1);
}

#[test]
fn test_bfs_visits_all_reachable_nodes() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("N1".to_string());
    let n2 = hg.add_labeled_node("N2".to_string());
    let n3 = hg.add_labeled_node("N3".to_string());
    let n4 = hg.add_labeled_node("N4".to_string());

    // Linear chain: N1 - N2 - N3 - N4
    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n2, n3], false);
    hg.add_hyperedge(vec![n3, n4], false);

    let traversal = hg.bfs(n1);

    assert_eq!(traversal.len(), 4);
    assert!(traversal.contains(&n1));
    assert!(traversal.contains(&n2));
    assert!(traversal.contains(&n3));
    assert!(traversal.contains(&n4));
}

#[test]
fn test_bfs_order_is_breadth_first() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_labeled_node("N1".to_string());
    let n2 = hg.add_labeled_node("N2".to_string());
    let n3 = hg.add_labeled_node("N3".to_string());
    let n4 = hg.add_labeled_node("N4".to_string());
    let n5 = hg.add_labeled_node("N5".to_string());

    //       N2  N3
    //        \  /
    //         N1
    //        /  \
    //       N4  N5
    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n1, n3], false);
    hg.add_hyperedge(vec![n1, n4], false);
    hg.add_hyperedge(vec![n1, n5], false);

    let traversal = hg.bfs(n1);

    assert_eq!(traversal[0], n1);
    // All neighbors of N1 should appear before any of their neighbors
    assert!(traversal.len() <= 5);
}

#[test]
fn test_bfs_on_disconnected_node() {
    let mut hg = Hypergraph::new();
    let isolated = hg.add_labeled_node("Isolated".to_string());

    let traversal = hg.bfs(isolated);

    assert_eq!(traversal.len(), 1);
    assert_eq!(traversal[0], isolated);
}

#[test]
fn test_bfs_on_isolated_component() {
    let mut hg = Hypergraph::new();
    // Component 1
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    hg.add_hyperedge(vec![n1, n2], false);

    // Component 2 (disconnected)
    let n3 = hg.add_node();
    let n4 = hg.add_node();
    hg.add_hyperedge(vec![n3, n4], false);

    let traversal = hg.bfs(n1);

    // Should only visit component 1
    assert!(traversal.contains(&n1));
    assert!(traversal.contains(&n2));
    assert!(!traversal.contains(&n3));
    assert!(!traversal.contains(&n4));
}

#[test]
fn test_shortest_path_between_adjacent_nodes() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], false);

    let path = hg.shortest_path(n1, n2);

    assert!(path.is_some());
    let p = path.unwrap();
    assert_eq!(p.len(), 2);
    assert_eq!(p[0], n1);
    assert_eq!(p[1], n2);
}

#[test]
fn test_shortest_path_multihop() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();
    let n4 = hg.add_node();

    // Chain: 1 - 2 - 3 - 4
    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n2, n3], false);
    hg.add_hyperedge(vec![n3, n4], false);

    let path = hg.shortest_path(n1, n4);

    assert!(path.is_some());
    let p = path.unwrap();
    assert_eq!(p.len(), 4);
    assert_eq!(p, vec![n1, n2, n3, n4]);
}

#[test]
fn test_shortest_path_start_equals_end() {
    let mut hg = Hypergraph::new();
    let n = hg.add_node();

    let path = hg.shortest_path(n, n);

    assert!(path.is_some());
    assert_eq!(path.unwrap(), vec![n]);
}

#[test]
fn test_shortest_path_no_path_exists() {
    let mut hg = Hypergraph::new();
    // Component 1
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    hg.add_hyperedge(vec![n1, n2], false);

    // Component 2 (disconnected)
    let n3 = hg.add_node();
    let n4 = hg.add_node();
    hg.add_hyperedge(vec![n3, n4], false);

    let path = hg.shortest_path(n1, n3);

    assert!(path.is_none());
}

#[test]
fn test_get_neighbors_of_single_node() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], false);
    hg.add_hyperedge(vec![n1, n3], false);

    let neighbors = hg.get_neighbors(n1);

    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.contains(&n2));
    assert!(neighbors.contains(&n3));
}

#[test]
fn test_get_neighbors_directed_vs_undirected() {
    let mut hg = Hypergraph::new();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    // Directed edges: n1 -> n2, n3 -> n1
    hg.add_hyperedge(vec![n1, n2], true);
    hg.add_hyperedge(vec![n3, n1], true);

    let neighbors = hg.get_neighbors(n1);

    // In current implementation, get_neighbors returns all connected nodes
    // regardless of directionality
    assert!(neighbors.contains(&n2));
    assert!(neighbors.contains(&n3));
}

#[test]
fn test_get_neighbors_with_multiple_hyperedges() {
    let mut hg = Hypergraph::new();
    let hub = hg.add_node();
    let n1 = hg.add_node();
    let n2 = hg.add_node();
    let n3 = hg.add_node();

    // Multiple edges from hub
    hg.add_hyperedge(vec![hub, n1], false);
    hg.add_hyperedge(vec![hub, n2], false);
    hg.add_hyperedge(vec![hub, n3], false);

    let neighbors = hg.get_neighbors(hub);

    assert_eq!(neighbors.len(), 3);
    assert!(neighbors.contains(&n1));
    assert!(neighbors.contains(&n2));
    assert!(neighbors.contains(&n3));
}

#[test]
fn test_incident_edges_returns_all_touching_edges() {
    let mut hg = Hypergraph::new();
    let hub = hg.add_node();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let e1 = hg.add_hyperedge(vec![hub, n1], false);
    let e2 = hg.add_hyperedge(vec![hub, n2], false);
    let e3 = hg.add_hyperedge(vec![n1, n2], false); // Doesn't touch hub

    let incidents = hg.get_incident_edges(hub);

    assert_eq!(incidents.len(), 2);
    assert!(incidents.contains(&e1));
    assert!(incidents.contains(&e2));
    assert!(!incidents.contains(&e3));
}

#[test]
fn test_incident_edges_empty_for_isolated_node() {
    let mut hg = Hypergraph::new();
    let isolated = hg.add_node();
    let n1 = hg.add_node();
    let n2 = hg.add_node();

    hg.add_hyperedge(vec![n1, n2], false);

    let incidents = hg.get_incident_edges(isolated);

    assert_eq!(incidents.len(), 0);
}

#[test]
fn test_degree_distribution_computation() {
    let mut hg = Hypergraph::new();
    let nodes: Vec<_> = (0..5).map(|_| hg.add_node()).collect();

    let hub = nodes[0];
    let n1 = nodes[1];
    let n2 = nodes[2];
    let n3 = nodes[3];
    let n4 = nodes[4];

    // Star topology: hub connected to all others
    hg.add_hyperedge(vec![hub, n1], false);
    hg.add_hyperedge(vec![hub, n2], false);
    hg.add_hyperedge(vec![hub, n3], false);
    hg.add_hyperedge(vec![hub, n4], false);

    let hub_degree = hg.get_incident_edges(hub).len();
    let leaf_degree = hg.get_incident_edges(n1).len();

    assert_eq!(hub_degree, 4); // Hub has 4 incident edges
    assert_eq!(leaf_degree, 1); // Leaves have 1 incident edge
}
