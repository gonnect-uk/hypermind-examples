// Port of Jena Vocabulary tests
// Tests standard RDF/RDFS/OWL/XSD vocabulary constants

use rdf_model::{Dictionary, Node};
use std::sync::Arc;

#[test]
fn test_rdf_type() {
    let dict = Arc::new(Dictionary::new());
    let rdf_type = Node::iri(dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"));

    if let Some(iri) = rdf_type.as_iri() {
        let uri = iri.as_str();
        assert_eq!(uri, "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    }
}

#[test]
fn test_rdfs_label() {
    let dict = Arc::new(Dictionary::new());
    let rdfs_label = Node::iri(dict.intern("http://www.w3.org/2000/01/rdf-schema#label"));

    if let Some(iri) = rdfs_label.as_iri() {
        assert_eq!(iri.as_str(), "http://www.w3.org/2000/01/rdf-schema#label");
    }
}

#[test]
fn test_rdfs_comment() {
    let dict = Arc::new(Dictionary::new());
    let rdfs_comment = Node::iri(dict.intern("http://www.w3.org/2000/01/rdf-schema#comment"));

    if let Some(iri) = rdfs_comment.as_iri() {
        let uri = iri.as_str();
        assert_eq!(uri, "http://www.w3.org/2000/01/rdf-schema#comment");
    }
}

#[test]
fn test_owl_class() {
    let dict = Arc::new(Dictionary::new());
    let owl_class = Node::iri(dict.intern("http://www.w3.org/2002/07/owl#Class"));

    if let Some(iri) = owl_class.as_iri() {
        let uri = iri.as_str();
        assert_eq!(uri, "http://www.w3.org/2002/07/owl#Class");
    }
}

#[test]
fn test_owl_thing() {
    let dict = Arc::new(Dictionary::new());
    let owl_thing = Node::iri(dict.intern("http://www.w3.org/2002/07/owl#Thing"));

    if let Some(iri) = owl_thing.as_iri() {
        let uri = iri.as_str();
        assert_eq!(uri, "http://www.w3.org/2002/07/owl#Thing");
    }
}

#[test]
fn test_xsd_string() {
    let dict = Arc::new(Dictionary::new());
    let xsd_string = dict.intern("http://www.w3.org/2001/XMLSchema#string");
    assert_eq!(xsd_string, "http://www.w3.org/2001/XMLSchema#string");
}

#[test]
fn test_xsd_integer() {
    let dict = Arc::new(Dictionary::new());
    let xsd_integer = dict.intern("http://www.w3.org/2001/XMLSchema#integer");
    assert_eq!(xsd_integer, "http://www.w3.org/2001/XMLSchema#integer");
}

#[test]
fn test_xsd_boolean() {
    let dict = Arc::new(Dictionary::new());
    let xsd_boolean = dict.intern("http://www.w3.org/2001/XMLSchema#boolean");
    assert_eq!(xsd_boolean, "http://www.w3.org/2001/XMLSchema#boolean");
}

#[test]
fn test_xsd_datetime() {
    let dict = Arc::new(Dictionary::new());
    let xsd_datetime = dict.intern("http://www.w3.org/2001/XMLSchema#dateTime");
    assert_eq!(xsd_datetime, "http://www.w3.org/2001/XMLSchema#dateTime");
}

#[test]
fn test_vocabulary_consistency() {
    let dict = Arc::new(Dictionary::new());

    // Same URI should intern to same reference
    let ref1 = dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    let ref2 = dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");

    assert_eq!(ref1, ref2);
}
