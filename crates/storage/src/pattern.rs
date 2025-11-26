//! Query pattern types for quad matching

use rdf_model::Node;

/// Pattern for matching nodes in queries
///
/// `None` represents a wildcard that matches any node.
#[derive(Clone, Debug, PartialEq)]
pub enum NodePattern<'a> {
    /// Match any node (wildcard)
    Any,

    /// Match a specific node
    Concrete(Node<'a>),
}

impl<'a> NodePattern<'a> {
    /// Check if this pattern matches a node
    pub fn matches(&self, node: &Node<'a>) -> bool {
        match self {
            NodePattern::Any => true,
            NodePattern::Concrete(n) => n == node,
        }
    }

    /// Check if this is a wildcard pattern
    pub fn is_wildcard(&self) -> bool {
        matches!(self, NodePattern::Any)
    }

    /// Check if this is a concrete pattern
    pub fn is_concrete(&self) -> bool {
        matches!(self, NodePattern::Concrete(_))
    }

    /// Get the concrete node, if any
    pub fn as_node(&self) -> Option<&Node<'a>> {
        match self {
            NodePattern::Concrete(n) => Some(n),
            NodePattern::Any => None,
        }
    }
}

impl<'a> From<Option<Node<'a>>> for NodePattern<'a> {
    fn from(opt: Option<Node<'a>>) -> Self {
        match opt {
            Some(node) => NodePattern::Concrete(node),
            None => NodePattern::Any,
        }
    }
}

impl<'a> From<Node<'a>> for NodePattern<'a> {
    fn from(node: Node<'a>) -> Self {
        NodePattern::Concrete(node)
    }
}

/// Pattern for matching quads in queries
///
/// Each field can be either a concrete node or a wildcard.
#[derive(Clone, Debug)]
pub struct QuadPattern<'a> {
    /// Subject pattern (can be wildcard)
    pub subject: NodePattern<'a>,

    /// Predicate pattern (can be wildcard)
    pub predicate: NodePattern<'a>,

    /// Object pattern (can be wildcard)
    pub object: NodePattern<'a>,

    /// Graph pattern (can be wildcard)
    pub graph: NodePattern<'a>,
}

impl<'a> QuadPattern<'a> {
    /// Create a new quad pattern
    pub fn new(
        subject: impl Into<NodePattern<'a>>,
        predicate: impl Into<NodePattern<'a>>,
        object: impl Into<NodePattern<'a>>,
        graph: impl Into<NodePattern<'a>>,
    ) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            graph: graph.into(),
        }
    }

    /// Create a pattern that matches everything (all wildcards)
    pub fn all() -> Self {
        Self {
            subject: NodePattern::Any,
            predicate: NodePattern::Any,
            object: NodePattern::Any,
            graph: NodePattern::Any,
        }
    }

    /// Check if a quad matches this pattern
    pub fn matches(&self, quad: &rdf_model::Quad<'a>) -> bool {
        self.subject.matches(&quad.subject)
            && self.predicate.matches(&quad.predicate)
            && self.object.matches(&quad.object)
            && match (&self.graph, &quad.graph) {
                (NodePattern::Any, _) => true,
                (NodePattern::Concrete(pg), Some(qg)) => pg == qg,
                (NodePattern::Concrete(_), None) => false,
                _ => true,
            }
    }

    /// Count how many positions are bound (not wildcards)
    pub fn bound_count(&self) -> usize {
        let mut count = 0;
        if !self.subject.is_wildcard() {
            count += 1;
        }
        if !self.predicate.is_wildcard() {
            count += 1;
        }
        if !self.object.is_wildcard() {
            count += 1;
        }
        if !self.graph.is_wildcard() {
            count += 1;
        }
        count
    }

    /// Check if this is a fully bound pattern (no wildcards)
    pub fn is_fully_bound(&self) -> bool {
        self.bound_count() == 4
    }

    /// Check if this is a wildcard pattern (all wildcards)
    pub fn is_wildcard(&self) -> bool {
        self.bound_count() == 0
    }
}

impl<'a> Default for QuadPattern<'a> {
    fn default() -> Self {
        Self::all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdf_model::Dictionary;

    #[test]
    fn test_node_pattern_wildcard() {
        let pattern = NodePattern::Any;
        let dict = Dictionary::new();
        let node = Node::iri(dict.intern("http://example.org/test"));

        assert!(pattern.matches(&node));
        assert!(pattern.is_wildcard());
        assert!(!pattern.is_concrete());
    }

    #[test]
    fn test_node_pattern_concrete() {
        let dict = Dictionary::new();
        let node1 = Node::iri(dict.intern("http://example.org/test"));
        let node2 = Node::iri(dict.intern("http://example.org/other"));

        let pattern = NodePattern::Concrete(node1.clone());

        assert!(pattern.matches(&node1));
        assert!(!pattern.matches(&node2));
        assert!(pattern.is_concrete());
        assert!(!pattern.is_wildcard());
    }

    #[test]
    fn test_quad_pattern_bound_count() {
        let dict = Dictionary::new();
        let node = Node::iri(dict.intern("http://example.org/test"));

        // All wildcards
        let pattern = QuadPattern::all();
        assert_eq!(pattern.bound_count(), 0);
        assert!(pattern.is_wildcard());

        // One bound
        let pattern = QuadPattern::new(node.clone(), None, None, None);
        assert_eq!(pattern.bound_count(), 1);

        // Two bound
        let pattern = QuadPattern::new(node.clone(), node.clone(), None, None);
        assert_eq!(pattern.bound_count(), 2);

        // Fully bound
        let pattern = QuadPattern::new(node.clone(), node.clone(), node.clone(), node.clone());
        assert_eq!(pattern.bound_count(), 4);
        assert!(pattern.is_fully_bound());
    }
}
