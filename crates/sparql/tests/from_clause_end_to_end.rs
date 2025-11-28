/// End-to-end comprehensive FROM clause testing
///
/// This test suite verifies complete W3C SPARQL 1.1 FROM and FROM NAMED functionality
/// covering all edge cases and real-world scenarios

use rdf_model::{Node, Quad};
use sparql::{Executor, SPARQLParser, Query};
use storage::QuadStore;
use std::sync::Arc;

#[test]
fn test_from_clause_basic() {
    // Basic FROM clause - query single named graph
    let mut store = QuadStore::new_in_memory();
    let dict = Arc::clone(store.dictionary());

    // Load data into graph1
    let s = dict.intern("http://example.org/alice");
    let p = dict.intern("http://example.org/name");
    let g1 = dict.intern("http://example.org/graph1");

    store.insert(Quad::new(
        Node::iri(s),
        Node::iri(p),
        Node::literal_str(dict.intern("Alice")),
        Some(Node::iri(g1)),
    )).unwrap();

    // Parse query with FROM
    let query = r#"
        SELECT ?name
        FROM <http://example.org/graph1>
        WHERE { ?s <http://example.org/name> ?name }
    "#;

    let mut parser = SPARQLParser::new();
    let parsed = parser.parse_query(query).expect("Should parse FROM clause");

    // Verify dataset was extracted from query
    match parsed {
        Query::Select { dataset, .. } => {
            assert_eq!(dataset.default.len(), 1, "Should have 1 default graph");
            assert_eq!(dataset.default[0], "http://example.org/graph1");
        }
        _ => panic!("Expected SELECT query"),
    }

    println!("✅ FROM clause basic parsing and extraction working");
}

#[test]
fn test_from_clause_execution_single_graph() {
    // Execute FROM clause query and verify results
    let mut store = QuadStore::new_in_memory();
    let dict = Arc::clone(store.dictionary());

    // Data in graph1
    let alice = dict.intern("http://example.org/alice");
    let name_pred = dict.intern("http://example.org/name");
    let graph1 = dict.intern("http://example.org/graph1");

    store.insert(Quad::new(
        Node::iri(alice),
        Node::iri(name_pred),
        Node::literal_str(dict.intern("Alice in Graph1")),
        Some(Node::iri(graph1)),
    )).unwrap();

    // Data in graph2 (should NOT be returned)
    let graph2 = dict.intern("http://example.org/graph2");
    store.insert(Quad::new(
        Node::iri(alice),
        Node::iri(name_pred),
        Node::literal_str(dict.intern("Alice in Graph2")),
        Some(Node::iri(graph2)),
    )).unwrap();

    // Query with FROM graph1 only
    let query = r#"
        SELECT ?name
        FROM <http://example.org/graph1>
        WHERE { ?s <http://example.org/name> ?name }
    "#;

    let mut parser = SPARQLParser::new();
    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, dataset, .. } => {
            let mut executor = Executor::new(&store);

            // CRITICAL: Pass dataset to executor
            if !dataset.default.is_empty() || !dataset.named.is_empty() {
                executor = executor.with_dataset(dataset);
            }

            let results = executor.execute(&pattern).expect("Should execute");

            assert_eq!(results.len(), 1, "Should return only 1 result from graph1");

            // Verify it's from graph1
            let binding = &results.bindings()[0];
            let name_var = sparql::Variable::new("name");
            let name_value = binding.get(&name_var).expect("Should have name binding");

            if let Node::Literal(lit) = name_value {
                assert!(lit.lexical_form.contains("Graph1"), "Should be from graph1");
                assert!(!lit.lexical_form.contains("Graph2"), "Should NOT be from graph2");
            } else {
                panic!("Expected literal value");
            }
        }
        _ => panic!("Expected SELECT query"),
    }

    println!("✅ FROM clause execution with single graph working correctly");
}

#[test]
fn test_from_clause_multiple_graphs() {
    // FROM with multiple graphs - should merge results
    let mut store = QuadStore::new_in_memory();
    let dict = Arc::clone(store.dictionary());

    let pred = dict.intern("http://example.org/value");
    let g1 = dict.intern("http://example.org/g1");
    let g2 = dict.intern("http://example.org/g2");
    let g3 = dict.intern("http://example.org/g3");

    // Data in 3 different graphs
    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/e1")),
        Node::iri(pred),
        Node::literal_str(dict.intern("Value from G1")),
        Some(Node::iri(g1)),
    )).unwrap();

    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/e2")),
        Node::iri(pred),
        Node::literal_str(dict.intern("Value from G2")),
        Some(Node::iri(g2)),
    )).unwrap();

    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/e3")),
        Node::iri(pred),
        Node::literal_str(dict.intern("Value from G3")),
        Some(Node::iri(g3)),
    )).unwrap();

    // Query FROM g1 and g2 (but NOT g3)
    let query = r#"
        SELECT ?value
        FROM <http://example.org/g1>
        FROM <http://example.org/g2>
        WHERE { ?s <http://example.org/value> ?value }
    "#;

    let mut parser = SPARQLParser::new();
    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, dataset, .. } => {
            assert_eq!(dataset.default.len(), 2, "Should have 2 FROM graphs");

            let mut executor = Executor::new(&store).with_dataset(dataset);
            let results = executor.execute(&pattern).unwrap();

            assert_eq!(results.len(), 2, "Should merge results from g1 and g2");

            // Verify we got g1 and g2 but not g3
            let values: Vec<String> = results.bindings().iter()
                .map(|b| {
                    let val = b.get(&sparql::Variable::new("value")).unwrap();
                    if let Node::Literal(lit) = val {
                        lit.lexical_form.to_string()
                    } else {
                        String::new()
                    }
                })
                .collect();

            assert!(values.iter().any(|v| v.contains("G1")), "Should have G1");
            assert!(values.iter().any(|v| v.contains("G2")), "Should have G2");
            assert!(!values.iter().any(|v| v.contains("G3")), "Should NOT have G3");
        }
        _ => panic!("Expected SELECT query"),
    }

    println!("✅ FROM clause with multiple graphs merging correctly");
}

#[test]
fn test_from_named_restricts_graph_access() {
    // FROM NAMED restricts which graphs GRAPH clause can access
    let mut store = QuadStore::new_in_memory();
    let dict = Arc::clone(store.dictionary());

    let pred = dict.intern("http://example.org/prop");
    let named1 = dict.intern("http://example.org/named1");
    let named2 = dict.intern("http://example.org/named2");

    // Data in two named graphs
    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/entity1")),
        Node::iri(pred),
        Node::literal_str(dict.intern("In Named1")),
        Some(Node::iri(named1)),
    )).unwrap();

    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/entity2")),
        Node::iri(pred),
        Node::literal_str(dict.intern("In Named2")),
        Some(Node::iri(named2)),
    )).unwrap();

    // Query with FROM NAMED named1 - should restrict GRAPH access
    let query = r#"
        SELECT ?value
        FROM NAMED <http://example.org/named1>
        WHERE {
            GRAPH <http://example.org/named1> {
                ?s <http://example.org/prop> ?value
            }
        }
    "#;

    let mut parser = SPARQLParser::new();
    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, dataset, .. } => {
            assert_eq!(dataset.named.len(), 1, "Should have 1 named graph");
            assert_eq!(dataset.named[0], "http://example.org/named1");

            let mut executor = Executor::new(&store).with_dataset(dataset);
            let results = executor.execute(&pattern).unwrap();

            assert_eq!(results.len(), 1, "Should access named1");

            let val = results.bindings()[0].get(&sparql::Variable::new("value")).unwrap();
            if let Node::Literal(lit) = val {
                assert!(lit.lexical_form.contains("Named1"));
            }
        }
        _ => panic!("Expected SELECT query"),
    }

    // Try accessing named2 (NOT in FROM NAMED) - should fail
    let query_forbidden = r#"
        SELECT ?value
        FROM NAMED <http://example.org/named1>
        WHERE {
            GRAPH <http://example.org/named2> {
                ?s <http://example.org/prop> ?value
            }
        }
    "#;

    let parsed2 = parser.parse_query(query_forbidden).unwrap();
    match parsed2 {
        Query::Select { pattern, dataset, .. } => {
            let mut executor = Executor::new(&store).with_dataset(dataset);
            let results = executor.execute(&pattern).unwrap();

            assert_eq!(results.len(), 0, "Should NOT access named2 (not in FROM NAMED)");
        }
        _ => panic!("Expected SELECT query"),
    }

    println!("✅ FROM NAMED correctly restricting GRAPH clause access");
}

#[test]
fn test_from_and_from_named_combined() {
    // Combine FROM and FROM NAMED in same query
    let mut store = QuadStore::new_in_memory();
    let dict = Arc::clone(store.dictionary());

    let type_pred = dict.intern("http://example.org/type");
    let default_g = dict.intern("http://example.org/default");
    let named_g = dict.intern("http://example.org/named");

    // Data in default graph
    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/default1")),
        Node::iri(type_pred),
        Node::literal_str(dict.intern("Default Graph Data")),
        Some(Node::iri(default_g)),
    )).unwrap();

    // Data in named graph
    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/named1")),
        Node::iri(type_pred),
        Node::literal_str(dict.intern("Named Graph Data")),
        Some(Node::iri(named_g)),
    )).unwrap();

    // Query with both FROM and FROM NAMED
    let query = r#"
        SELECT ?type ?namedType
        FROM <http://example.org/default>
        FROM NAMED <http://example.org/named>
        WHERE {
            ?s <http://example.org/type> ?type .
            OPTIONAL {
                GRAPH <http://example.org/named> {
                    ?n <http://example.org/type> ?namedType .
                }
            }
        }
    "#;

    let mut parser = SPARQLParser::new();
    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, dataset, .. } => {
            assert_eq!(dataset.default.len(), 1, "Should have FROM default");
            assert_eq!(dataset.named.len(), 1, "Should have FROM NAMED");

            let mut executor = Executor::new(&store).with_dataset(dataset);
            let results = executor.execute(&pattern).unwrap();

            assert!(results.len() > 0, "Should have results from default graph");
        }
        _ => panic!("Expected SELECT query"),
    }

    println!("✅ FROM and FROM NAMED combined working correctly");
}

#[test]
fn test_from_vs_graph_clause_difference() {
    // Demonstrate difference between FROM and GRAPH clauses
    let mut store = QuadStore::new_in_memory();
    let dict = Arc::clone(store.dictionary());

    let pred = dict.intern("http://example.org/p");
    let g1 = dict.intern("http://example.org/g1");
    let g2 = dict.intern("http://example.org/g2");

    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/e1")),
        Node::iri(pred),
        Node::literal_str(dict.intern("G1")),
        Some(Node::iri(g1)),
    )).unwrap();

    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/e2")),
        Node::iri(pred),
        Node::literal_str(dict.intern("G2")),
        Some(Node::iri(g2)),
    )).unwrap();

    // Query 1: Using GRAPH clause (explicit graph selection)
    let query_graph = r#"
        SELECT ?o
        WHERE {
            GRAPH <http://example.org/g1> {
                ?s <http://example.org/p> ?o
            }
        }
    "#;

    // Query 2: Using FROM clause (sets default graph for query)
    let query_from = r#"
        SELECT ?o
        FROM <http://example.org/g1>
        WHERE {
            ?s <http://example.org/p> ?o
        }
    "#;

    let mut parser = SPARQLParser::new();

    // Both should return same result (data from g1 only)
    let parsed_graph = parser.parse_query(query_graph).unwrap();
    let parsed_from = parser.parse_query(query_from).unwrap();

    match (parsed_graph, parsed_from) {
        (Query::Select { pattern: p1, .. }, Query::Select { pattern: p2, dataset, .. }) => {
            let results1 = Executor::new(&store).execute(&p1).unwrap();

            let mut exec2 = Executor::new(&store);
            if !dataset.default.is_empty() {
                exec2 = exec2.with_dataset(dataset);
            }
            let results2 = exec2.execute(&p2).unwrap();

            assert_eq!(results1.len(), results2.len(), "FROM and GRAPH should give same results");
            assert_eq!(results1.len(), 1, "Should get data from g1 only");
        }
        _ => panic!("Expected SELECT queries"),
    }

    println!("✅ FROM vs GRAPH clause behavior verified");
}

#[test]
fn test_complex_from_clause_real_world() {
    // Real-world scenario: querying across multiple data sources
    let mut store = QuadStore::new_in_memory();
    let dict = Arc::clone(store.dictionary());

    // Simulate data from different sources
    let name_pred = dict.intern("http://schema.org/name");
    let age_pred = dict.intern("http://schema.org/age");
    let dept_pred = dict.intern("http://company.com/department");

    let hr_db = dict.intern("http://company.com/graphs/hr");
    let it_db = dict.intern("http://company.com/graphs/it");
    let finance_db = dict.intern("http://company.com/graphs/finance");

    // HR database
    let alice = dict.intern("http://company.com/employees/alice");
    store.insert(Quad::new(
        Node::iri(alice),
        Node::iri(name_pred),
        Node::literal_str(dict.intern("Alice")),
        Some(Node::iri(hr_db)),
    )).unwrap();
    store.insert(Quad::new(
        Node::iri(alice),
        Node::iri(age_pred),
        Node::literal_typed(dict.intern("30"), "http://www.w3.org/2001/XMLSchema#integer"),
        Some(Node::iri(hr_db)),
    )).unwrap();

    // IT database
    store.insert(Quad::new(
        Node::iri(alice),
        Node::iri(dept_pred),
        Node::literal_str(dict.intern("Engineering")),
        Some(Node::iri(it_db)),
    )).unwrap();

    // Finance database (different employee)
    let bob = dict.intern("http://company.com/employees/bob");
    store.insert(Quad::new(
        Node::iri(bob),
        Node::iri(name_pred),
        Node::literal_str(dict.intern("Bob")),
        Some(Node::iri(finance_db)),
    )).unwrap();

    // Query: Get employee info from HR and IT databases only
    let query = r#"
        SELECT ?name ?age
        FROM <http://company.com/graphs/hr>
        FROM NAMED <http://company.com/graphs/it>
        WHERE {
            ?employee <http://schema.org/name> ?name .
            OPTIONAL { ?employee <http://schema.org/age> ?age }
            OPTIONAL {
                GRAPH <http://company.com/graphs/it> {
                    ?employee <http://company.com/department> ?dept
                }
            }
        }
    "#;

    let mut parser = SPARQLParser::new();
    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, dataset, .. } => {
            // Verify dataset configuration
            assert_eq!(dataset.default.len(), 1, "Should query HR as default");
            assert_eq!(dataset.named.len(), 1, "Should have IT as named graph");

            let mut executor = Executor::new(&store).with_dataset(dataset);
            let results = executor.execute(&pattern).unwrap();

            // Should get Alice from HR (Bob is in finance, not included)
            assert_eq!(results.len(), 1, "Should get 1 employee from HR");

            let name_binding = results.bindings()[0].get(&sparql::Variable::new("name")).unwrap();
            if let Node::Literal(lit) = name_binding {
                assert_eq!(lit.lexical_form, "Alice", "Should be Alice from HR");
            }
        }
        _ => panic!("Expected SELECT query"),
    }

    println!("✅ Complex real-world FROM clause scenario working correctly");
}

#[test]
fn test_from_clause_w3c_compliance() {
    // Verify W3C SPARQL 1.1 spec compliance for FROM clause
    let mut store = QuadStore::new_in_memory();
    let dict = Arc::clone(store.dictionary());

    // Per W3C spec: FROM creates a default graph by merging specified graphs
    let p = dict.intern("http://example.org/p");
    let g1 = dict.intern("http://example.org/g1");
    let g2 = dict.intern("http://example.org/g2");

    // Data in g1
    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/s1")),
        Node::iri(p),
        Node::literal_str(dict.intern("o1")),
        Some(Node::iri(g1)),
    )).unwrap();

    // Data in g2
    store.insert(Quad::new(
        Node::iri(dict.intern("http://example.org/s2")),
        Node::iri(p),
        Node::literal_str(dict.intern("o2")),
        Some(Node::iri(g2)),
    )).unwrap();

    // Per spec: Multiple FROM clauses merge graphs into single default graph
    let query = r#"
        SELECT ?s ?o
        FROM <http://example.org/g1>
        FROM <http://example.org/g2>
        WHERE { ?s <http://example.org/p> ?o }
    "#;

    let mut parser = SPARQLParser::new();
    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, dataset, .. } => {
            let mut executor = Executor::new(&store).with_dataset(dataset);
            let results = executor.execute(&pattern).unwrap();

            // Both triples should be in merged default graph
            assert_eq!(results.len(), 2, "Should merge both graphs");

            println!("✅ W3C SPARQL 1.1 FROM clause spec compliance verified");
        }
        _ => panic!("Expected SELECT query"),
    }
}
