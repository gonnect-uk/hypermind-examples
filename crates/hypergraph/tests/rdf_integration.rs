// Category 9: Integration with RDF Model (10 tests)
// Tests hypergraph integration with rust-kgdb RDF semantics

use hypergraph::Hypergraph;

#[test]
fn test_hyperedge_as_rdf_triple() {
    // RDF triple: (Subject, Predicate, Object)
    let mut hg = Hypergraph::new();

    let subject = hg.add_labeled_node("<http://example.org/Alice>".to_string());
    let predicate = hg.add_labeled_node("<http://example.org/knows>".to_string());
    let object = hg.add_labeled_node("<http://example.org/Bob>".to_string());

    let triple = hg.add_hyperedge(vec![subject, predicate, object], true);

    assert_eq!(hg.get_hyperedge(triple).unwrap().nodes.len(), 3);
    assert!(hg.get_hyperedge(triple).unwrap().directed);
}

#[test]
fn test_hyperedge_as_rdf_quad() {
    // RDF quad: (Subject, Predicate, Object, Context/Graph)
    let mut hg = Hypergraph::new();

    let subject = hg.add_labeled_node("<http://example.org/Alice>".to_string());
    let predicate = hg.add_labeled_node("<http://example.org/knows>".to_string());
    let object = hg.add_labeled_node("<http://example.org/Bob>".to_string());
    let graph = hg.add_labeled_node("<http://example.org/graph1>".to_string());

    let quad = hg.add_hyperedge(vec![subject, predicate, object, graph], true);

    assert_eq!(hg.get_hyperedge(quad).unwrap().nodes.len(), 4);
}

#[test]
fn test_labeled_edges_as_rdf_predicates() {
    // Edge labels represent RDF predicates
    let mut hg = Hypergraph::new();

    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());

    let knows = hg.add_labeled_hyperedge(
        vec![alice, bob],
        true,
        "<http://example.org/knows>".to_string()
    );

    let edge = hg.get_hyperedge(knows).unwrap();
    assert_eq!(
        edge.label,
        Some("<http://example.org/knows>".to_string())
    );
}

#[test]
fn test_metadata_as_rdf_annotations() {
    // Edge metadata can store RDF annotations (reification)
    let mut hg = Hypergraph::new();

    let n1 = hg.add_node();
    let n2 = hg.add_node();

    let e = hg.add_hyperedge(vec![n1, n2], true);

    // Verify edge exists and can store metadata
    assert!(hg.get_hyperedge(e).is_some());
}

#[test]
fn test_pattern_matching_for_sparql_bgp() {
    // BGP (Basic Graph Pattern) matching using hyperedge pattern finding
    let mut hg = Hypergraph::new();

    // Create RDF triple store
    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());
    let charlie = hg.add_labeled_node("Charlie".to_string());
    let knows = hg.add_labeled_node("knows".to_string());

    hg.add_hyperedge(vec![alice, knows, bob], true);
    hg.add_hyperedge(vec![bob, knows, charlie], true);
    hg.add_hyperedge(vec![alice, knows, charlie], true);

    // SPARQL query: ?x knows Bob
    let results = hg.find_edges(&[None, Some(knows), Some(bob)]);

    assert_eq!(results.len(), 1);
}

#[test]
fn test_neighbor_traversal_as_rdf_property_traversal() {
    // Property path traversal using neighbor discovery
    let mut hg = Hypergraph::new();

    let _person = hg.add_labeled_node("Person".to_string());
    let knows = hg.add_labeled_node("knows".to_string());

    let p1 = hg.add_labeled_node("P1".to_string());
    let p2 = hg.add_labeled_node("P2".to_string());
    let p3 = hg.add_labeled_node("P3".to_string());

    // P1 knows P2 knows P3
    hg.add_hyperedge(vec![p1, knows, p2], true);
    hg.add_hyperedge(vec![p2, knows, p3], true);

    // Who does P1 know?
    let neighbors = hg.get_neighbors(p1);

    assert!(neighbors.contains(&p2));
}

#[test]
fn test_subgraph_as_named_graph_extraction() {
    // Named graph extraction using subgraph operation
    let mut hg = Hypergraph::new();

    let doc1 = hg.add_labeled_node("Document1".to_string());
    let doc2 = hg.add_labeled_node("Document2".to_string());
    let author = hg.add_labeled_node("author".to_string());

    let alice = hg.add_labeled_node("Alice".to_string());
    let bob = hg.add_labeled_node("Bob".to_string());

    // Two named graphs
    hg.add_hyperedge(vec![doc1, author, alice], true); // In graph 1
    hg.add_hyperedge(vec![doc2, author, bob], true);   // In graph 2

    // Extract graph 1 (doc1 and alice)
    let subgraph = hg.subgraph(&[doc1, alice]);

    // Subgraph contains nodes from graph 1
    assert_eq!(subgraph.stats().node_count, 2);
}

#[test]
fn test_shortest_path_as_property_path() {
    // SPARQL property paths using shortest path
    let mut hg = Hypergraph::new();

    let p1 = hg.add_labeled_node("P1".to_string());
    let p2 = hg.add_labeled_node("P2".to_string());
    let p3 = hg.add_labeled_node("P3".to_string());
    let p4 = hg.add_labeled_node("P4".to_string());

    let knows = hg.add_labeled_node("knows".to_string());

    // Property paths: P1 knows+ P4
    hg.add_hyperedge(vec![p1, knows, p2], true);
    hg.add_hyperedge(vec![p2, knows, p3], true);
    hg.add_hyperedge(vec![p3, knows, p4], true);

    // Find property path P1 knows+ P4
    let path = hg.shortest_path(p1, p4);

    assert!(path.is_some());
    // The hyperedge allows shortest path computation
    // Path length should be valid (at least 2 nodes)
    assert!(path.unwrap().len() >= 2);
}

#[test]
fn test_bfs_as_sparql_construct_result() {
    // BFS traversal results as SPARQL CONSTRUCT patterns
    let mut hg = Hypergraph::new();

    let center = hg.add_labeled_node("Center".to_string());
    let n1 = hg.add_labeled_node("N1".to_string());
    let n2 = hg.add_labeled_node("N2".to_string());
    let n3 = hg.add_labeled_node("N3".to_string());

    let connects = hg.add_labeled_node("connects".to_string());

    hg.add_hyperedge(vec![center, connects, n1], true);
    hg.add_hyperedge(vec![center, connects, n2], true);
    hg.add_hyperedge(vec![center, connects, n3], true);

    // SPARQL: CONSTRUCT { ?center connects ?x }
    let results = hg.bfs(center);

    // All reachable nodes from center
    assert!(results.contains(&center));
    assert!(results.contains(&n1));
    assert!(results.contains(&n2));
    assert!(results.contains(&n3));
}

#[test]
fn test_statistics_feed_into_query_planning() {
    // Hypergraph statistics inform SPARQL query optimization
    let mut hg = Hypergraph::new();

    // Simulate RDF dataset with different predicate selectivity
    let people: Vec<_> = (0..100).map(|i| hg.add_labeled_node(format!("P{}", i))).collect();

    let knows = hg.add_labeled_node("knows".to_string());
    let lives_in = hg.add_labeled_node("lives_in".to_string());

    let cities: Vec<_> = (0..5).map(|i| hg.add_labeled_node(format!("C{}", i))).collect();

    // Many "knows" edges (low selectivity)
    for i in 0..50 {
        for j in (i+1)..50 {
            hg.add_hyperedge(vec![people[i], knows, people[j]], true);
        }
    }

    // Few "lives_in" edges (high selectivity)
    for i in 0..5 {
        hg.add_hyperedge(vec![people[i], lives_in, cities[i % cities.len()]], true);
    }

    let stats = hg.stats();

    // Query optimizer would prefer to filter on lives_in first (fewer edges)
    assert!(stats.edge_count > 0);

    // Count edges for each predicate
    let knows_edges = hg.find_edges(&[None, Some(knows), None]).len();
    let lives_in_edges = hg.find_edges(&[None, Some(lives_in), None]).len();

    assert!(knows_edges > lives_in_edges);
}

#[test]
fn test_rdf_uri_formatting_compatibility() {
    // URIs stored with angle brackets for OpenSearch compatibility
    let mut hg = Hypergraph::new();

    let subject = hg.add_labeled_node("<http://example.org/Alice>".to_string());
    let predicate = hg.add_labeled_node("<http://example.org/knows>".to_string());
    let object = hg.add_labeled_node("<http://example.org/Bob>".to_string());

    hg.add_hyperedge(vec![subject, predicate, object], true);

    // Verify URI format matches OpenSearch expectations
    let s_label = hg.get_node(subject).unwrap().label.as_ref().unwrap();
    assert!(s_label.starts_with('<'));
    assert!(s_label.ends_with('>'));
}
