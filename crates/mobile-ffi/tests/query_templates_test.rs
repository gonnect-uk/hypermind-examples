// Query Templates Integration Test
// Tests all GraphDBAdmin reasoner and quick template queries return proper results

use mobile_ffi::GraphDB;

// Sample TTL data with entities, types, and labels (matching database-catalog.ttl structure)
const TEST_TTL: &str = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix : <http://zenya.com/> .

# Database entities
:CUSTOMERS rdf:type rdfs:Class ;
    rdfs:label "CUSTOMERS" ;
    rdfs:comment "Customer table" .

:ORDERS rdf:type rdfs:Class ;
    rdfs:label "ORDERS" ;
    rdfs:comment "Orders table" .

:LINEITEM rdf:type rdfs:Class ;
    rdfs:label "LINEITEM" ;
    rdfs:comment "Line items table" .

# Properties
:hasOrderkey rdf:type rdf:Property ;
    rdfs:label "hasOrderkey" ;
    rdfs:domain :ORDERS ;
    rdfs:range xsd:integer .

:hasCustomerkey rdf:type rdf:Property ;
    rdfs:label "hasCustomerkey" ;
    rdfs:domain :CUSTOMERS ;
    rdfs:range xsd:integer .

# Sample data instances
:customer_1 rdf:type :CUSTOMERS ;
    rdfs:label "Customer 1" ;
    :hasCustomerkey 1001 .

:order_1 rdf:type :ORDERS ;
    rdfs:label "Order 1" ;
    :hasOrderkey 5001 .

:order_2 rdf:type :ORDERS ;
    rdfs:label "Order 2" ;
    :hasOrderkey 5002 .
"#;

fn setup_test_db() -> GraphDB {
    let db = GraphDB::new("http://zenya.com/test".to_string());
    db.load_ttl(TEST_TTL.to_string(), None).expect("Failed to load test data");
    db
}

#[test]
fn test_rdfs_reasoner_query() {
    let db = setup_test_db();

    // RDFS reasoner query: Find entities with both type and label
    let query = "SELECT ?entity ?type ?label WHERE {
        ?entity <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type .
        ?entity <http://www.w3.org/2000/01/rdf-schema#label> ?label
    } LIMIT 50";

    let results = db.query_select(query.to_string()).expect("Query failed");

    // Should find: 3 classes (CUSTOMERS, ORDERS, LINEITEM) + 2 properties + 3 instances = 8 entities
    assert!(!results.is_empty(), "RDFS query returned no results");
    assert!(results.len() >= 3, "Expected at least 3 results (classes + instances), got {}", results.len());

    // Verify result structure
    let first = &results[0];
    assert!(first.bindings.contains_key("entity"), "Result missing 'entity' binding");
    assert!(first.bindings.contains_key("type"), "Result missing 'type' binding");
    assert!(first.bindings.contains_key("label"), "Result missing 'label' binding");

    println!("✅ RDFS reasoner query: {} results", results.len());
}

#[test]
fn test_owl_rl_reasoner_query() {
    let db = setup_test_db();

    // OWL RL query: Return all triples
    let query = "SELECT ?subject ?predicate ?object WHERE {
        ?subject ?predicate ?object
    } LIMIT 50";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert!(!results.is_empty(), "OWL RL query returned no results");
    assert!(results.len() >= 10, "Expected at least 10 triples, got {}", results.len());

    let first = &results[0];
    assert!(first.bindings.contains_key("subject"), "Result missing 'subject' binding");
    assert!(first.bindings.contains_key("predicate"), "Result missing 'predicate' binding");
    assert!(first.bindings.contains_key("object"), "Result missing 'object' binding");

    println!("✅ OWL RL reasoner query: {} results", results.len());
}

#[test]
fn test_owl_el_reasoner_query() {
    let db = setup_test_db();

    // OWL EL query: Return unique predicates
    let query = "SELECT DISTINCT ?predicate WHERE {
        ?s ?predicate ?o
    } LIMIT 50";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert!(!results.is_empty(), "OWL EL query returned no results");
    // Should find predicates: rdf:type, rdfs:label, rdfs:comment, rdfs:domain, rdfs:range, :hasOrderkey, :hasCustomerkey
    assert!(results.len() >= 5, "Expected at least 5 distinct predicates, got {}", results.len());

    let first = &results[0];
    assert!(first.bindings.contains_key("predicate"), "Result missing 'predicate' binding");

    println!("✅ OWL EL reasoner query: {} distinct predicates", results.len());
}

#[test]
fn test_owl_ql_reasoner_query() {
    let db = setup_test_db();

    // OWL QL query: Count distinct types
    let query = "SELECT (COUNT(DISTINCT ?type) AS ?count) WHERE {
        ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type
    }";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert_eq!(results.len(), 1, "Expected exactly 1 aggregate result");

    let first = &results[0];
    assert!(first.bindings.contains_key("count"), "Result missing 'count' binding");

    let count_str = first.bindings.get("count").expect("Missing count");
    let count: i64 = count_str.trim_matches('"').parse().expect("Count not a number");
    assert!(count >= 3, "Expected at least 3 distinct types, got {}", count);

    println!("✅ OWL QL reasoner query: {} distinct types", count);
}

#[test]
fn test_shacl_reasoner_query() {
    let db = setup_test_db();

    // SHACL query: Entities with labels
    let query = "SELECT ?entity ?label WHERE {
        ?entity <http://www.w3.org/2000/01/rdf-schema#label> ?label
    } LIMIT 50";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert!(!results.is_empty(), "SHACL query returned no results");
    assert!(results.len() >= 5, "Expected at least 5 entities with labels, got {}", results.len());

    let first = &results[0];
    assert!(first.bindings.contains_key("entity"), "Result missing 'entity' binding");
    assert!(first.bindings.contains_key("label"), "Result missing 'label' binding");

    println!("✅ SHACL reasoner query: {} entities with labels", results.len());
}

#[test]
fn test_datalog_reasoner_query() {
    let db = setup_test_db();

    // Datalog query: Subjects with types (no inference)
    let query = "SELECT ?subject ?type WHERE {
        ?subject <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type
    } LIMIT 50";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert!(!results.is_empty(), "Datalog query returned no results");
    assert!(results.len() >= 5, "Expected at least 5 typed subjects, got {}", results.len());

    let first = &results[0];
    assert!(first.bindings.contains_key("subject"), "Result missing 'subject' binding");
    assert!(first.bindings.contains_key("type"), "Result missing 'type' binding");

    println!("✅ Datalog reasoner query: {} typed subjects", results.len());
}

#[test]
fn test_quick_template_select_all() {
    let db = setup_test_db();

    // SELECT ALL template
    let query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 100";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert!(!results.is_empty(), "SELECT ALL returned no results");
    assert!(results.len() >= 10, "Expected at least 10 triples, got {}", results.len());

    let first = &results[0];
    assert!(first.bindings.contains_key("s"), "Result missing 's' binding");
    assert!(first.bindings.contains_key("p"), "Result missing 'p' binding");
    assert!(first.bindings.contains_key("o"), "Result missing 'o' binding");

    println!("✅ SELECT ALL template: {} triples", results.len());
}

#[test]
fn test_quick_template_count() {
    let db = setup_test_db();

    // COUNT template
    let query = "SELECT (COUNT(*) AS ?count) WHERE { ?s ?p ?o }";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert_eq!(results.len(), 1, "Expected exactly 1 aggregate result");

    let first = &results[0];
    assert!(first.bindings.contains_key("count"), "Result missing 'count' binding");

    let count_str = first.bindings.get("count").expect("Missing count");
    let count: i64 = count_str.trim_matches('"').parse().expect("Count not a number");
    assert!(count >= 10, "Expected at least 10 triples, got {}", count);

    println!("✅ COUNT template: {} triples", count);
}

#[test]
fn test_quick_template_types() {
    let db = setup_test_db();

    // TYPES template
    let query = "SELECT DISTINCT ?type WHERE {
        ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type
    }";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert!(!results.is_empty(), "TYPES query returned no results");
    // Should find: rdfs:Class, rdf:Property, :CUSTOMERS, :ORDERS, :LINEITEM
    assert!(results.len() >= 3, "Expected at least 3 distinct types, got {}", results.len());

    let first = &results[0];
    assert!(first.bindings.contains_key("type"), "Result missing 'type' binding");

    println!("✅ TYPES template: {} distinct types", results.len());
}

#[test]
fn test_quick_template_predicates() {
    let db = setup_test_db();

    // PREDICATES template
    let query = "SELECT DISTINCT ?p WHERE { ?s ?p ?o }";

    let results = db.query_select(query.to_string()).expect("Query failed");

    assert!(!results.is_empty(), "PREDICATES query returned no results");
    assert!(results.len() >= 5, "Expected at least 5 distinct predicates, got {}", results.len());

    let first = &results[0];
    assert!(first.bindings.contains_key("p"), "Result missing 'p' binding");

    println!("✅ PREDICATES template: {} distinct predicates", results.len());
}

#[test]
fn test_all_queries_return_data() {
    let db = setup_test_db();

    let queries = vec![
        ("RDFS", "SELECT ?entity ?type ?label WHERE { ?entity <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type . ?entity <http://www.w3.org/2000/01/rdf-schema#label> ?label } LIMIT 50"),
        ("OWL_RL", "SELECT ?subject ?predicate ?object WHERE { ?subject ?predicate ?object } LIMIT 50"),
        ("OWL_EL", "SELECT DISTINCT ?predicate WHERE { ?s ?predicate ?o } LIMIT 50"),
        ("SELECT_ALL", "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 100"),
        ("TYPES", "SELECT DISTINCT ?type WHERE { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type }"),
        ("PREDICATES", "SELECT DISTINCT ?p WHERE { ?s ?p ?o }"),
    ];

    for (name, query) in queries {
        let results = db.query_select(query.to_string()).expect(&format!("{} query failed", name));
        assert!(!results.is_empty(), "{} query returned no results", name);
        println!("✅ {}: {} results", name, results.len());
    }

    println!("\n🎉 ALL QUERIES VALIDATED - Ready for deployment!");
}
