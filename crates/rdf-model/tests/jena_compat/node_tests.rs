// Port of Jena TestRDFNodes.java
// Tests core Node functionality: IRI, Literal, BlankNode, QuotedTriple

use rdf_model::{Dictionary, Node};
use std::sync::Arc;

#[test]
fn test_is_iri() {
    let dict = Arc::new(Dictionary::new());

    // IRI nodes should identify as IRI
    let iri_node = Node::iri(dict.intern("http://example.org/foo"));
    assert!(iri_node.is_iri());

    // Literals should not be IRIs
    let literal_node = Node::literal_str(dict.intern("hello"));
    assert!(!literal_node.is_iri());

    // Blank nodes should not be IRIs
    let blank_node = Node::blank(1);
    assert!(!blank_node.is_iri());
}

#[test]
fn test_is_literal() {
    let dict = Arc::new(Dictionary::new());

    // Literals should identify as literals
    let int_literal = Node::literal_typed(
        dict.intern("17"),
        dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
    );
    assert!(int_literal.is_literal());

    let string_literal = Node::literal_str(dict.intern("hello"));
    assert!(string_literal.is_literal());

    // IRIs should not be literals
    let iri_node = Node::iri(dict.intern("http://example.org/foo"));
    assert!(!iri_node.is_literal());

    // Blank nodes should not be literals
    let blank_node = Node::blank(1);
    assert!(!blank_node.is_literal());
}

#[test]
fn test_is_blank_node() {
    let dict = Arc::new(Dictionary::new());

    // Blank nodes should identify as blank
    let blank1 = Node::blank(1);
    assert!(blank1.is_blank_node());

    let blank2 = Node::blank(42);
    assert!(blank2.is_blank_node());

    // IRIs should not be blank nodes
    let iri_node = Node::iri(dict.intern("http://example.org/foo"));
    assert!(!iri_node.is_blank_node());

    // Literals should not be blank nodes
    let literal_node = Node::literal_str(dict.intern("test"));
    assert!(!literal_node.is_blank_node());
}

#[test]
fn test_blank_node_uniqueness() {
    // Each blank node should have unique ID
    let blank1 = Node::blank(1);
    let blank2 = Node::blank(2);
    let blank3 = Node::blank(1);

    assert_ne!(blank1.as_blank_node().unwrap(), blank2.as_blank_node().unwrap());
    assert_eq!(blank1.as_blank_node().unwrap(), blank3.as_blank_node().unwrap());
}

#[test]
fn test_iri_equality() {
    let dict = Arc::new(Dictionary::new());

    let iri1 = Node::iri(dict.intern("http://example.org/resource"));
    let iri2 = Node::iri(dict.intern("http://example.org/resource"));
    let iri3 = Node::iri(dict.intern("http://example.org/different"));

    // Same IRI should be equal
    assert_eq!(iri1.as_iri().unwrap().as_str(), iri2.as_iri().unwrap().as_str());

    // Different IRIs should not be equal
    assert_ne!(iri1.as_iri().unwrap().as_str(), iri3.as_iri().unwrap().as_str());
}

#[test]
fn test_literal_equality() {
    let dict = Arc::new(Dictionary::new());

    let lit1 = Node::literal_str(dict.intern("hello"));
    let lit2 = Node::literal_str(dict.intern("hello"));
    let lit3 = Node::literal_str(dict.intern("world"));

    // Same literal values should be equal
    let l1 = lit1.as_literal().unwrap();
    let l2 = lit2.as_literal().unwrap();
    let l3 = lit3.as_literal().unwrap();

    assert_eq!(l1.lexical_form, l2.lexical_form);
    assert_ne!(l1.lexical_form, l3.lexical_form);
}

#[test]
fn test_as_iri_extraction() {
    let dict = Arc::new(Dictionary::new());

    let iri_node = Node::iri(dict.intern("http://example.org/resource"));

    // Extract IRI string
    let iri = iri_node.as_iri().expect("Should be IRI");
    assert_eq!(iri.as_str(), "http://example.org/resource");
}

#[test]
fn test_as_literal_extraction() {
    let dict = Arc::new(Dictionary::new());

    let literal_node = Node::literal_typed(
        dict.intern("42"),
        dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
    );

    // Extract literal value and datatype
    let literal = literal_node.as_literal().expect("Should be Literal");
    assert_eq!(literal.lexical_form, "42");
    assert_eq!(literal.datatype, Some("http://www.w3.org/2001/XMLSchema#integer"));
}

#[test]
fn test_quoted_triple_node() {
    use rdf_model::Triple;

    let dict = Arc::new(Dictionary::new());

    // Create a quoted triple (RDF-star feature)
    let subject = Node::iri(dict.intern("http://example.org/subject"));
    let predicate = Node::iri(dict.intern("http://example.org/predicate"));
    let object = Node::literal_str(dict.intern("value"));

    let triple = Triple {
        subject: subject.clone(),
        predicate: predicate.clone(),
        object: object.clone(),
    };

    let quoted = Node::quoted_triple(triple);

    // Verify it's recognized as QuotedTriple
    assert!(quoted.is_quoted_triple());
}

#[test]
fn test_variable_node() {
    let dict = Arc::new(Dictionary::new());

    // SPARQL variables
    let var1 = Node::variable(dict.intern("x"));
    let var2 = Node::variable(dict.intern("subject"));

    assert!(var1.is_variable());
    assert!(var2.is_variable());

    // Extract variable names
    let v1 = var1.as_variable().expect("Should be Variable");
    let v2 = var2.as_variable().expect("Should be Variable");

    assert_eq!(v1.0, "x");
    assert_eq!(v2.0, "subject");
}
