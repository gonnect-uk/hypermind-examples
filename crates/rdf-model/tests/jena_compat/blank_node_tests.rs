// Port of Jena BlankNode tests
// Tests blank node creation, uniqueness, and identity

use rdf_model::{Node, BlankNodeId};

#[test]
fn test_create_blank_node() {
    let blank = Node::blank(1);
    assert!(matches!(blank, Node::BlankNode(_)));
}

#[test]
fn test_blank_node_unique_ids() {
    let blank1 = Node::blank(1);
    let blank2 = Node::blank(2);
    let blank3 = Node::blank(3);

    if let (Node::BlankNode(id1), Node::BlankNode(id2), Node::BlankNode(id3)) =
        (&blank1, &blank2, &blank3)
    {
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
}

#[test]
fn test_blank_node_same_id() {
    let blank1 = Node::blank(42);
    let blank2 = Node::blank(42);

    if let (Node::BlankNode(id1), Node::BlankNode(id2)) = (&blank1, &blank2) {
        assert_eq!(id1, id2);
    }
}

#[test]
fn test_blank_node_sequential_creation() {
    let blanks: Vec<Node> = (1..=100).map(|i| Node::blank(i)).collect();

    assert_eq!(blanks.len(), 100);

    for (i, blank) in blanks.iter().enumerate() {
        if let Node::BlankNode(id) = blank {
            assert_eq!(*id, BlankNodeId((i + 1) as u64));
        }
    }
}

#[test]
fn test_blank_node_not_iri() {
    let blank = Node::blank(1);
    assert!(!matches!(blank, Node::Iri(_)));
}

#[test]
fn test_blank_node_not_literal() {
    let blank = Node::blank(1);
    assert!(!matches!(blank, Node::Literal(_)));
}

#[test]
fn test_blank_node_in_triple_subject() {
    use rdf_model::{Dictionary, Triple};
    use std::sync::Arc;

    let dict = Arc::new(Dictionary::new());

    let triple = Triple {
        subject: Node::blank(1),
        predicate: Node::iri(dict.intern("http://example.org/name")),
        object: Node::literal_typed("Anonymous", dict.intern("http://www.w3.org/2001/XMLSchema#string")),
    };

    // Blank nodes are valid subjects
    assert!(matches!(triple.subject, Node::BlankNode(_)));
}

#[test]
fn test_blank_node_in_triple_object() {
    use rdf_model::{Dictionary, Triple};
    use std::sync::Arc;

    let dict = Arc::new(Dictionary::new());

    let triple = Triple {
        subject: Node::iri(dict.intern("http://example.org/person")),
        predicate: Node::iri(dict.intern("http://example.org/knows")),
        object: Node::blank(1),
    };

    // Blank nodes are valid objects
    assert!(matches!(triple.object, Node::BlankNode(_)));
}

#[test]
fn test_blank_node_large_ids() {
    let blank1 = Node::blank(u64::MAX);
    let blank2 = Node::blank(u64::MAX - 1);

    if let (Node::BlankNode(id1), Node::BlankNode(id2)) = (&blank1, &blank2) {
        assert_eq!(*id1, BlankNodeId(u64::MAX));
        assert_eq!(*id2, BlankNodeId(u64::MAX - 1));
        assert_ne!(id1, id2);
    }
}

#[test]
fn test_blank_node_zero_id() {
    let blank = Node::blank(0);

    if let Node::BlankNode(id) = blank {
        assert_eq!(id, BlankNodeId(0));
    }
}
