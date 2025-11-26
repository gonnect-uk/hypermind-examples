// Port of Jena TestResources.java
// Tests Resource creation and IRI handling

use rdf_model::{Dictionary, Node};
use std::sync::Arc;

#[test]
fn test_create_resource() {
    let dict = Arc::new(Dictionary::new());

    let resource = Node::iri(dict.intern("http://example.org/resource1"));

    if let Node::Iri(iri) = resource {
        let uri = iri.as_str();
        assert_eq!(uri, "http://example.org/resource1");
    }
}

#[test]
fn test_resource_uri_schemes() {
    let dict = Arc::new(Dictionary::new());

    let http_resource = Node::iri(dict.intern("http://example.org/r"));
    let https_resource = Node::iri(dict.intern("https://example.org/r"));
    let ftp_resource = Node::iri(dict.intern("ftp://example.org/r"));
    let urn_resource = Node::iri(dict.intern("urn:isbn:0-486-27557-4"));

    // All should be valid IRIs
    assert!(matches!(http_resource, Node::Iri(_)));
    assert!(matches!(https_resource, Node::Iri(_)));
    assert!(matches!(ftp_resource, Node::Iri(_)));
    assert!(matches!(urn_resource, Node::Iri(_)));
}

#[test]
fn test_resource_with_fragment() {
    let dict = Arc::new(Dictionary::new());

    let resource = Node::iri(dict.intern("http://example.org/resource#section1"));

    if let Node::Iri(iri) = resource {
        let uri = iri.as_str();
        assert!(uri.contains("#section1"));
        assert_eq!(uri, "http://example.org/resource#section1");
    }
}

#[test]
fn test_resource_with_query_params() {
    let dict = Arc::new(Dictionary::new());

    let resource = Node::iri(dict.intern("http://example.org/api?param=value&key=123"));

    if let Node::Iri(iri) = resource {
        let uri = iri.as_str();
        assert!(uri.contains("?param=value"));
        assert!(uri.contains("&key=123"));
    }
}

#[test]
fn test_resource_equality() {
    let dict = Arc::new(Dictionary::new());

    let res1 = Node::iri(dict.intern("http://example.org/resource"));
    let res2 = Node::iri(dict.intern("http://example.org/resource"));

    assert_eq!(res1.as_iri(), res2.as_iri());
}

#[test]
fn test_resource_inequality() {
    let dict = Arc::new(Dictionary::new());

    let res1 = Node::iri(dict.intern("http://example.org/resource1"));
    let res2 = Node::iri(dict.intern("http://example.org/resource2"));

    assert_ne!(res1.as_iri(), res2.as_iri());
}

#[test]
fn test_resource_case_sensitive() {
    let dict = Arc::new(Dictionary::new());

    let res1 = Node::iri(dict.intern("http://example.org/Resource"));
    let res2 = Node::iri(dict.intern("http://example.org/resource"));

    // URIs are case-sensitive (except scheme and host, but we test full string)
    assert_ne!(res1.as_iri(), res2.as_iri());
}

#[test]
fn test_resource_unicode_iri() {
    let dict = Arc::new(Dictionary::new());

    let resource = Node::iri(dict.intern("http://example.org/resource/日本"));

    if let Node::Iri(iri) = resource {
        let uri = iri.as_str();
        assert!(uri.contains("日本"));
    }
}

#[test]
fn test_vocabulary_resources() {
    let dict = Arc::new(Dictionary::new());

    // Common RDF vocabulary IRIs
    let rdf_type = Node::iri(dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"));
    let rdfs_label = Node::iri(dict.intern("http://www.w3.org/2000/01/rdf-schema#label"));
    let owl_class = Node::iri(dict.intern("http://www.w3.org/2002/07/owl#Class"));

    assert!(matches!(rdf_type, Node::Iri(_)));
    assert!(matches!(rdfs_label, Node::Iri(_)));
    assert!(matches!(owl_class, Node::Iri(_)));
}

#[test]
fn test_relative_uri_handling() {
    let dict = Arc::new(Dictionary::new());

    // In practice, relative URIs should be resolved against base URI
    // Here we just test that the system accepts them
    let relative = Node::iri(dict.intern("./relative/path"));

    if let Node::Iri(iri) = relative {
        assert_eq!(iri.as_str(), "./relative/path");
    }
}
