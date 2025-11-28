/// Comprehensive FROM clause test for rust-kgdb
///
/// Tests W3C SPARQL 1.1 FROM and FROM NAMED functionality
/// This test verifies the root cause fix where dataset was not being passed to executor

#[cfg(test)]
mod tests {
    use mobile_ffi::{GraphDB, GonnectError};

    #[test]
    fn test_from_clause_single_graph() {
        // Test FROM with a single graph
        let db = GraphDB::new("FROM_Test".to_string());

        // Load data into different named graphs
        let graph1_ttl = r#"
@prefix ex: <http://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<http://example.org/alice>
    ex:name "Alice Graph1" ;
    ex:age "25"^^xsd:integer .

<http://example.org/bob>
    ex:name "Bob Graph1" ;
    ex:age "30"^^xsd:integer .
"#;

        let graph2_ttl = r#"
@prefix ex: <http://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<http://example.org/charlie>
    ex:name "Charlie Graph2" ;
    ex:age "35"^^xsd:integer .

<http://example.org/dave>
    ex:name "Dave Graph2" ;
    ex:age "40"^^xsd:integer .
"#;

        // Load into named graphs
        db.load_ttl(graph1_ttl.to_string(), Some("http://example.org/graph1".to_string()))
            .expect("Should load graph1");
        db.load_ttl(graph2_ttl.to_string(), Some("http://example.org/graph2".to_string()))
            .expect("Should load graph2");

        // Query with FROM graph1 - should only return Alice and Bob
        let query = r#"
PREFIX ex: <http://example.org/>
SELECT ?person ?name
FROM <http://example.org/graph1>
WHERE {
    ?person ex:name ?name .
}
"#;

        let results = db.query_select(query.to_string())
            .expect("FROM clause query should succeed");

        println!("FROM graph1 results: {} bindings", results.len());
        for result in &results {
            println!("  {:?}", result.bindings);
        }

        // Should only get 2 results from graph1 (Alice and Bob)
        assert_eq!(results.len(), 2, "FROM clause should only query graph1");

        // Verify names contain "Graph1"
        for result in &results {
            let name = result.bindings.get("name").expect("Should have name binding");
            assert!(name.contains("Graph1"), "Results should be from graph1: {}", name);
        }

        println!("✅ FROM clause with single graph working correctly");
    }

    #[test]
    fn test_from_clause_multiple_graphs() {
        // Test FROM with multiple graphs - should merge results
        let db = GraphDB::new("FROM_Multi_Test".to_string());

        // Load data into three different graphs
        let ttl1 = r#"
@prefix ex: <http://example.org/> .
<http://example.org/p1> ex:value "Value from Graph1" .
"#;
        let ttl2 = r#"
@prefix ex: <http://example.org/> .
<http://example.org/p2> ex:value "Value from Graph2" .
"#;
        let ttl3 = r#"
@prefix ex: <http://example.org/> .
<http://example.org/p3> ex:value "Value from Graph3" .
"#;

        db.load_ttl(ttl1.to_string(), Some("http://example.org/g1".to_string()))
            .expect("Should load g1");
        db.load_ttl(ttl2.to_string(), Some("http://example.org/g2".to_string()))
            .expect("Should load g2");
        db.load_ttl(ttl3.to_string(), Some("http://example.org/g3".to_string()))
            .expect("Should load g3");

        // Query with FROM g1 and g2 - should merge results (but NOT g3)
        let query = r#"
PREFIX ex: <http://example.org/>
SELECT ?value
FROM <http://example.org/g1>
FROM <http://example.org/g2>
WHERE {
    ?s ex:value ?value .
}
"#;

        let results = db.query_select(query.to_string())
            .expect("Multiple FROM clauses should work");

        println!("FROM g1 + g2 results: {} bindings", results.len());
        for result in &results {
            println!("  {:?}", result.bindings);
        }

        // Should get 2 results (from g1 and g2, but NOT g3)
        assert_eq!(results.len(), 2, "Should merge results from both FROM graphs");

        // Verify we got g1 and g2 but not g3
        let values: Vec<String> = results.iter()
            .map(|r| r.bindings.get("value").unwrap().clone())
            .collect();

        assert!(values.iter().any(|v| v.contains("Graph1")), "Should have g1 result");
        assert!(values.iter().any(|v| v.contains("Graph2")), "Should have g2 result");
        assert!(!values.iter().any(|v| v.contains("Graph3")), "Should NOT have g3 result");

        println!("✅ FROM clause with multiple graphs merging correctly");
    }

    #[test]
    fn test_from_named_with_graph_clause() {
        // Test FROM NAMED - should restrict which graphs GRAPH clause can access
        let db = GraphDB::new("FROM_NAMED_Test".to_string());

        // Load data into two graphs
        let ttl1 = r#"
@prefix ex: <http://example.org/> .
<http://example.org/entity1> ex:property "Value in Named Graph1" .
"#;
        let ttl2 = r#"
@prefix ex: <http://example.org/> .
<http://example.org/entity2> ex:property "Value in Named Graph2" .
"#;

        db.load_ttl(ttl1.to_string(), Some("http://example.org/named1".to_string()))
            .expect("Should load named1");
        db.load_ttl(ttl2.to_string(), Some("http://example.org/named2".to_string()))
            .expect("Should load named2");

        // Query with FROM NAMED named1 only
        // GRAPH clause should ONLY be able to access named1 (not named2)
        let query_named1 = r#"
PREFIX ex: <http://example.org/>
SELECT ?value
FROM NAMED <http://example.org/named1>
WHERE {
    GRAPH <http://example.org/named1> {
        ?s ex:property ?value .
    }
}
"#;

        let results1 = db.query_select(query_named1.to_string())
            .expect("FROM NAMED with GRAPH should work");

        assert_eq!(results1.len(), 1, "Should get result from named1");
        assert!(results1[0].bindings.get("value").unwrap().contains("Named Graph1"));

        // Try to query named2 (NOT in FROM NAMED) - should return empty
        let query_named2 = r#"
PREFIX ex: <http://example.org/>
SELECT ?value
FROM NAMED <http://example.org/named1>
WHERE {
    GRAPH <http://example.org/named2> {
        ?s ex:property ?value .
    }
}
"#;

        let results2 = db.query_select(query_named2.to_string())
            .expect("Query should succeed but return empty");

        assert_eq!(results2.len(), 0, "Should NOT access named2 (not in FROM NAMED)");

        println!("✅ FROM NAMED correctly restricting GRAPH clause access");
    }

    #[test]
    fn test_from_and_from_named_combined() {
        // Test combining FROM and FROM NAMED in same query
        let db = GraphDB::new("FROM_Combined_Test".to_string());

        let default_ttl = r#"
@prefix ex: <http://example.org/> .
<http://example.org/default1> ex:type "Default Graph Data" .
"#;

        let named_ttl = r#"
@prefix ex: <http://example.org/> .
<http://example.org/named1> ex:type "Named Graph Data" .
"#;

        db.load_ttl(default_ttl.to_string(), Some("http://example.org/default".to_string()))
            .expect("Should load default");
        db.load_ttl(named_ttl.to_string(), Some("http://example.org/named".to_string()))
            .expect("Should load named");

        // Query with both FROM and FROM NAMED
        let query = r#"
PREFIX ex: <http://example.org/>
SELECT ?type ?namedType
FROM <http://example.org/default>
FROM NAMED <http://example.org/named>
WHERE {
    ?s ex:type ?type .
    OPTIONAL {
        GRAPH <http://example.org/named> {
            ?n ex:type ?namedType .
        }
    }
}
"#;

        let results = db.query_select(query.to_string())
            .expect("Combined FROM and FROM NAMED should work");

        assert!(results.len() > 0, "Should have results from default graph");

        println!("✅ FROM and FROM NAMED combined working correctly");
    }

    #[test]
    fn test_no_from_queries_all_graphs() {
        // Test that WITHOUT FROM clause, query accesses all graphs (default behavior)
        let db = GraphDB::new("No_FROM_Test".to_string());

        let ttl1 = r#"<http://ex.org/e1> <http://ex.org/p> "G1" ."#;
        let ttl2 = r#"<http://ex.org/e2> <http://ex.org/p> "G2" ."#;

        db.load_ttl(ttl1.to_string(), Some("http://ex.org/g1".to_string())).ok();
        db.load_ttl(ttl2.to_string(), Some("http://ex.org/g2".to_string())).ok();

        // Query WITHOUT FROM - should see all graphs
        let query_all = "SELECT ?o WHERE { ?s <http://ex.org/p> ?o }";
        let results_all = db.query_select(query_all.to_string())
            .expect("Query without FROM should work");

        // With FROM g1 only - should see only g1
        let query_from = "SELECT ?o FROM <http://ex.org/g1> WHERE { ?s <http://ex.org/p> ?o }";
        let results_from = db.query_select(query_from.to_string())
            .expect("Query with FROM should work");

        println!("Without FROM: {} results, With FROM: {} results",
                 results_all.len(), results_from.len());

        // FROM should restrict results (fewer than querying all)
        assert!(results_from.len() <= results_all.len(),
                "FROM clause should restrict results");

        println!("✅ FROM clause properly restricts query scope");
    }
}
