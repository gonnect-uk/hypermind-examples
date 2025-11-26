//! RDF Quad (Subject-Predicate-Object-Graph)

use crate::{Node, Triple};
use std::fmt;

/// RDF Quad: Subject-Predicate-Object-Graph
///
/// Extends Triple with optional graph name for named graphs.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Quad<'a> {
    /// Subject (must be IRI, BlankNode, or QuotedTriple)
    pub subject: Node<'a>,

    /// Predicate (must be IRI)
    pub predicate: Node<'a>,

    /// Object (can be any node type)
    pub object: Node<'a>,

    /// Graph name (None = default graph, Some = named graph)
    pub graph: Option<Node<'a>>,
}

impl<'a> Quad<'a> {
    /// Create a new quad
    pub fn new(
        subject: Node<'a>,
        predicate: Node<'a>,
        object: Node<'a>,
        graph: Option<Node<'a>>,
    ) -> Self {
        assert!(
            subject.is_iri() || subject.is_blank_node() || subject.is_quoted_triple(),
            "Subject must be IRI, BlankNode, or QuotedTriple"
        );
        assert!(predicate.is_iri(), "Predicate must be IRI");

        if let Some(ref g) = graph {
            assert!(
                g.is_iri() || g.is_blank_node(),
                "Graph must be IRI or BlankNode"
            );
        }

        Quad {
            subject,
            predicate,
            object,
            graph,
        }
    }

    /// Create a quad from a triple (default graph)
    pub fn from_triple(triple: Triple<'a>) -> Self {
        Quad {
            subject: triple.subject,
            predicate: triple.predicate,
            object: triple.object,
            graph: None,
        }
    }

    /// Create a quad from a triple with graph name
    pub fn from_triple_with_graph(triple: Triple<'a>, graph: Node<'a>) -> Self {
        Quad {
            subject: triple.subject,
            predicate: triple.predicate,
            object: triple.object,
            graph: Some(graph),
        }
    }

    /// Convert to triple (drops graph)
    pub fn to_triple(&self) -> Triple<'a> {
        Triple {
            subject: self.subject.clone(),
            predicate: self.predicate.clone(),
            object: self.object.clone(),
        }
    }

    /// Check if this is in the default graph
    pub fn is_default_graph(&self) -> bool {
        self.graph.is_none()
    }

    /// Check if this quad matches a pattern
    ///
    /// None represents a wildcard that matches anything.
    pub fn matches(
        &self,
        subject: Option<&Node<'a>>,
        predicate: Option<&Node<'a>>,
        object: Option<&Node<'a>>,
        graph: Option<&Node<'a>>,
    ) -> bool {
        let subject_matches = subject.map_or(true, |s| s == &self.subject);
        let predicate_matches = predicate.map_or(true, |p| p == &self.predicate);
        let object_matches = object.map_or(true, |o| o == &self.object);

        let graph_matches = match (graph, &self.graph) {
            (None, _) => true,  // Wildcard matches any graph
            (Some(g), Some(ref self_g)) => g == self_g,
            (Some(_), None) => false,
        };

        subject_matches && predicate_matches && object_matches && graph_matches
    }

    /// Serialize to N-Quads format
    pub fn to_nquads(&self) -> String {
        if let Some(ref graph) = self.graph {
            format!(
                "{} {} {} {} .",
                self.subject, self.predicate, self.object, graph
            )
        } else {
            format!("{} {} {} .", self.subject, self.predicate, self.object)
        }
    }
}

impl<'a> fmt::Debug for Quad<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Quad")
            .field("subject", &self.subject)
            .field("predicate", &self.predicate)
            .field("object", &self.object)
            .field("graph", &self.graph)
            .finish()
    }
}

impl<'a> fmt::Display for Quad<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref graph) = self.graph {
            write!(
                f,
                "{} {} {} {}",
                self.subject, self.predicate, self.object, graph
            )
        } else {
            write!(f, "{} {} {}", self.subject, self.predicate, self.object)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quad_default_graph() {
        let subject = Node::iri("http://example.org/s");
        let predicate = Node::iri("http://example.org/p");
        let object = Node::literal_str("value");

        let quad = Quad::new(subject, predicate, object, None);

        assert!(quad.is_default_graph());
    }

    #[test]
    fn test_quad_named_graph() {
        let subject = Node::iri("http://example.org/s");
        let predicate = Node::iri("http://example.org/p");
        let object = Node::literal_str("value");
        let graph = Node::iri("http://example.org/graph");

        let quad = Quad::new(subject, predicate, object, Some(graph));

        assert!(!quad.is_default_graph());
    }

    #[test]
    fn test_quad_from_triple() {
        let subject = Node::iri("http://example.org/s");
        let predicate = Node::iri("http://example.org/p");
        let object = Node::literal_str("value");

        let triple = Triple::new(subject, predicate, object);
        let quad = Quad::from_triple(triple);

        assert!(quad.is_default_graph());
    }

    #[test]
    fn test_quad_matches() {
        let subject = Node::iri("http://example.org/s");
        let predicate = Node::iri("http://example.org/p");
        let object = Node::literal_str("value");
        let graph = Node::iri("http://example.org/graph");

        let quad = Quad::new(subject.clone(), predicate.clone(), object.clone(), Some(graph.clone()));

        // Exact match
        assert!(quad.matches(Some(&subject), Some(&predicate), Some(&object), Some(&graph)));

        // Wildcard matches
        assert!(quad.matches(None, None, None, None));
        assert!(quad.matches(Some(&subject), None, None, None));

        // Graph mismatch
        let other_graph = Node::iri("http://example.org/other");
        assert!(!quad.matches(Some(&subject), Some(&predicate), Some(&object), Some(&other_graph)));
    }

    #[test]
    fn test_nquads_format() {
        let subject = Node::iri("http://example.org/s");
        let predicate = Node::iri("http://example.org/p");
        let object = Node::literal_str("value");
        let graph = Node::iri("http://example.org/g");

        let quad = Quad::new(subject, predicate, object, Some(graph));
        let nq = quad.to_nquads();

        assert!(nq.contains("<http://example.org/s>"));
        assert!(nq.contains("<http://example.org/p>"));
        assert!(nq.contains("\"value\""));
        assert!(nq.contains("<http://example.org/g>"));
    }
}
