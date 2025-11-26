// Port of Jena Namespace/Prefix tests
// Tests namespace handling and prefix expansion

use rdf_model::{Dictionary, Node};
use std::sync::Arc;

#[test]
fn test_rdf_namespace() {
    let dict = Arc::new(Dictionary::new());

    let rdf_ns = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    let rdf_type = Node::iri(dict.intern(&format!("{}type", rdf_ns)));

    if let Node::Iri(iri) = rdf_type {
        let uri = iri.as_str();
        assert!(uri.starts_with(rdf_ns));
        assert_eq!(uri, "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    }
}

#[test]
fn test_rdfs_namespace() {
    let dict = Arc::new(Dictionary::new());

    let rdfs_ns = "http://www.w3.org/2000/01/rdf-schema#";
    let rdfs_label = Node::iri(dict.intern(&format!("{}label", rdfs_ns)));

    if let Node::Iri(iri) = rdfs_label {
        let uri = iri.as_str();
        assert!(uri.starts_with(rdfs_ns));
    }
}

#[test]
fn test_owl_namespace() {
    let dict = Arc::new(Dictionary::new());

    let owl_ns = "http://www.w3.org/2002/07/owl#";
    let owl_class = Node::iri(dict.intern(&format!("{}Class", owl_ns)));

    if let Node::Iri(iri) = owl_class {
        let uri = iri.as_str();
        assert!(uri.starts_with(owl_ns));
    }
}

#[test]
fn test_xsd_namespace() {
    let dict = Arc::new(Dictionary::new());

    let xsd_ns = "http://www.w3.org/2001/XMLSchema#";
    let xsd_string = dict.intern(&format!("{}string", xsd_ns));

    assert!(xsd_string.starts_with(xsd_ns));
}

#[test]
fn test_custom_namespace() {
    let dict = Arc::new(Dictionary::new());

    let custom_ns = "http://example.org/myont#";
    let custom_class = Node::iri(dict.intern(&format!("{}MyClass", custom_ns)));

    if let Node::Iri(iri) = custom_class {
        let uri = iri.as_str();
        assert!(uri.starts_with(custom_ns));
        assert_eq!(uri, "http://example.org/myont#MyClass");
    }
}

#[test]
fn test_namespace_separator() {
    let dict = Arc::new(Dictionary::new());

    // Hash separator
    let hash_uri = Node::iri(dict.intern("http://example.org/vocab#term"));

    // Slash separator
    let slash_uri = Node::iri(dict.intern("http://example.org/vocab/term"));

    if let Node::Iri(iri) = hash_uri {
        let uri = iri.as_str();
        assert!(uri.contains("#"));
    }

    if let Node::Iri(iri) = slash_uri {
        let uri = iri.as_str();
        assert!(uri.contains("/term"));
    }
}

#[test]
fn test_prefix_expansion() {
    let dict = Arc::new(Dictionary::new());

    // Simulated prefix expansion: rdf:type -> full URI
    let prefix_map = vec![
        ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
        ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
        ("owl", "http://www.w3.org/2002/07/owl#"),
    ];

    for (_prefix, namespace) in prefix_map {
        let full_uri = format!("{}type", namespace);
        let expanded = dict.intern(&full_uri);
        assert!(expanded.contains(namespace));
    }
}

#[test]
fn test_base_uri_resolution() {
    let dict = Arc::new(Dictionary::new());

    let base = "http://example.org/";
    let relative = "resource1";
    let resolved = format!("{}{}", base, relative);

    let uri = Node::iri(dict.intern(&resolved));

    if let Node::Iri(iri) = uri {
        let full = iri.as_str();
        assert_eq!(full, "http://example.org/resource1");
    }
}

#[test]
fn test_namespace_consistency() {
    let dict = Arc::new(Dictionary::new());

    let ns = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

    let type1 = dict.intern(&format!("{}type", ns));
    let type2 = dict.intern(&format!("{}type", ns));

    // Same URI should be interned to same reference
    assert_eq!(type1, type2);
}

#[test]
fn test_multiple_terms_same_namespace() {
    let dict = Arc::new(Dictionary::new());

    let rdf_ns = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

    let terms = vec![
        Node::iri(dict.intern(&format!("{}type", rdf_ns))),
        Node::iri(dict.intern(&format!("{}Property", rdf_ns))),
        Node::iri(dict.intern(&format!("{}Statement", rdf_ns))),
        Node::iri(dict.intern(&format!("{}subject", rdf_ns))),
        Node::iri(dict.intern(&format!("{}predicate", rdf_ns))),
        Node::iri(dict.intern(&format!("{}object", rdf_ns))),
    ];

    for term in terms {
        if let Node::Iri(iri) = term {
        let uri = iri.as_str();
            assert!(uri.starts_with(rdf_ns));
        }
    }
}
