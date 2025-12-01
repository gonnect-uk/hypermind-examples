//! RDF Triple (Subject-Predicate-Object)

use crate::Node;
use std::fmt;

/// RDF Triple: Subject-Predicate-Object
///
/// Zero-copy representation using borrowed nodes.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Triple<'a> {
    /// Subject (must be IRI, BlankNode, or QuotedTriple)
    pub subject: Node<'a>,

    /// Predicate (must be IRI)
    pub predicate: Node<'a>,

    /// Object (can be any node type)
    pub object: Node<'a>,
}

impl<'a> Triple<'a> {
    /// Create a new triple
    ///
    /// # Panics
    ///
    /// Panics if subject is not IRI, BlankNode, or QuotedTriple,
    /// or if predicate is not IRI.
    pub fn new(subject: Node<'a>, predicate: Node<'a>, object: Node<'a>) -> Self {
        assert!(
            subject.is_iri() || subject.is_blank_node() || subject.is_quoted_triple(),
            "Subject must be IRI, BlankNode, or QuotedTriple"
        );
        assert!(predicate.is_iri(), "Predicate must be IRI");

        Triple {
            subject,
            predicate,
            object,
        }
    }

    /// Create a new triple without validation (for performance)
    ///
    /// # Safety
    ///
    /// Caller must ensure subject and predicate are valid node types.
    pub fn new_unchecked(subject: Node<'a>, predicate: Node<'a>, object: Node<'a>) -> Self {
        Triple {
            subject,
            predicate,
            object,
        }
    }

    /// Check if this triple matches a pattern
    ///
    /// None represents a wildcard that matches anything.
    pub fn matches(
        &self,
        subject: Option<&Node<'a>>,
        predicate: Option<&Node<'a>>,
        object: Option<&Node<'a>>,
    ) -> bool {
        let subject_matches = subject.map_or(true, |s| s == &self.subject);
        let predicate_matches = predicate.map_or(true, |p| p == &self.predicate);
        let object_matches = object.map_or(true, |o| o == &self.object);

        subject_matches && predicate_matches && object_matches
    }

    /// Get subject as IRI string (if it's an IRI)
    pub fn subject_iri(&self) -> Option<&str> {
        self.subject.as_iri().map(|iri| iri.as_str())
    }

    /// Get predicate as IRI string
    pub fn predicate_iri(&self) -> Option<&str> {
        self.predicate.as_iri().map(|iri| iri.as_str())
    }

    /// Get object as IRI string (if it's an IRI)
    pub fn object_iri(&self) -> Option<&str> {
        self.object.as_iri().map(|iri| iri.as_str())
    }

    /// Serialize to N-Triples format
    pub fn to_ntriples(&self) -> String {
        format!("{} {} {} .", self.subject, self.predicate, self.object)
    }
}

impl<'a> fmt::Debug for Triple<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Triple")
            .field("subject", &self.subject)
            .field("predicate", &self.predicate)
            .field("object", &self.object)
            .finish()
    }
}

impl<'a> fmt::Display for Triple<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.subject, self.predicate, self.object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triple_creation() {
        let subject = Node::iri("http://example.org/subject");
        let predicate = Node::iri("http://example.org/predicate");
        let object = Node::literal_str("value");

        let triple = Triple::new(subject, predicate, object);

        assert_eq!(triple.subject_iri(), Some("http://example.org/subject"));
        assert_eq!(triple.predicate_iri(), Some("http://example.org/predicate"));
    }

    #[test]
    fn test_triple_matches() {
        let subject = Node::iri("http://example.org/subject");
        let predicate = Node::iri("http://example.org/predicate");
        let object = Node::literal_str("value");

        let triple = Triple::new(subject.clone(), predicate.clone(), object.clone());

        // Exact match
        assert!(triple.matches(Some(&subject), Some(&predicate), Some(&object)));

        // Wildcard matches
        assert!(triple.matches(None, None, None));
        assert!(triple.matches(Some(&subject), None, None));
        assert!(triple.matches(None, Some(&predicate), None));

        // Non-match
        let other_subject = Node::iri("http://example.org/other");
        assert!(!triple.matches(Some(&other_subject), Some(&predicate), Some(&object)));
    }

    #[test]
    fn test_ntriples_format() {
        let subject = Node::iri("http://example.org/s");
        let predicate = Node::iri("http://example.org/p");
        let object = Node::literal_str("value");

        let triple = Triple::new(subject, predicate, object);
        let nt = triple.to_ntriples();

        assert!(nt.contains("<http://example.org/s>"));
        assert!(nt.contains("<http://example.org/p>"));
        assert!(nt.contains("\"value\""));
    }

    #[test]
    #[should_panic(expected = "Subject must be IRI")]
    fn test_invalid_subject() {
        let subject = Node::literal_str("invalid");  // Literal can't be subject
        let predicate = Node::iri("http://example.org/p");
        let object = Node::literal_str("value");

        Triple::new(subject, predicate, object);
    }

    #[test]
    #[should_panic(expected = "Predicate must be IRI")]
    fn test_invalid_predicate() {
        let subject = Node::iri("http://example.org/s");
        let predicate = Node::literal_str("invalid");  // Literal can't be predicate
        let object = Node::literal_str("value");

        Triple::new(subject, predicate, object);
    }
}
