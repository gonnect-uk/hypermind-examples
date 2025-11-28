//! SHACL Shape Definitions
//!
//! Core types for W3C SHACL shapes and constraints.

use rdf_model::Node;

/// SHACL Shape - defines constraints on RDF data
#[derive(Debug, Clone)]
pub enum Shape<'a> {
    /// Node shape - validates entire nodes
    NodeShape {
        /// Shape IRI
        id: Option<Node<'a>>,
        /// Target definitions
        targets: Vec<Target<'a>>,
        /// Property shapes
        properties: Vec<PropertyShape<'a>>,
        /// Node constraints
        constraints: Vec<Constraint<'a>>,
        /// Nested shape constraints
        shapes: Vec<ShapeConstraint<'a>>,
        /// Deactivated flag
        deactivated: bool,
        /// Severity level
        severity: Option<&'a str>,
        /// Custom message
        message: Option<&'a str>,
    },
    /// Property shape - validates property values
    PropertyShape {
        /// Shape IRI
        id: Option<Node<'a>>,
        /// Property path
        path: PropertyPath<'a>,
        /// Value constraints
        constraints: Vec<Constraint<'a>>,
        /// Nested shape constraints
        shapes: Vec<ShapeConstraint<'a>>,
        /// Deactivated flag
        deactivated: bool,
        /// Severity level
        severity: Option<&'a str>,
        /// Custom message
        message: Option<&'a str>,
    },
}

/// Target selector - defines which nodes to validate
#[derive(Debug, Clone)]
pub enum Target<'a> {
    /// sh:targetClass - all instances of a class
    TargetClass(Node<'a>),
    /// sh:targetNode - specific nodes
    TargetNode(Node<'a>),
    /// sh:targetSubjectsOf - subjects of a predicate
    TargetSubjectsOf(Node<'a>),
    /// sh:targetObjectsOf - objects of a predicate
    TargetObjectsOf(Node<'a>),
}

/// Property path for SHACL (simplified subset of SPARQL property paths)
#[derive(Debug, Clone)]
pub enum PropertyPath<'a> {
    /// Direct predicate
    Predicate(Node<'a>),
    /// Sequence path (p1 / p2)
    Sequence(Vec<PropertyPath<'a>>),
    /// Alternative path (p1 | p2)
    Alternative(Vec<PropertyPath<'a>>),
    /// Inverse path (^p)
    Inverse(Box<PropertyPath<'a>>),
    /// Zero or more path (p*)
    ZeroOrMore(Box<PropertyPath<'a>>),
    /// One or more path (p+)
    OneOrMore(Box<PropertyPath<'a>>),
    /// Zero or one path (p?)
    ZeroOrOne(Box<PropertyPath<'a>>),
}

/// SHACL Constraint Component
#[derive(Debug, Clone)]
pub enum Constraint<'a> {
    // Value Type Constraints
    /// sh:class - value must be instance of class
    Class(Node<'a>),
    /// sh:datatype - literal must have datatype
    Datatype(Node<'a>),
    /// sh:nodeKind - value must be IRI, BlankNode, Literal, etc
    NodeKind(NodeKind),

    // Cardinality Constraints
    /// sh:minCount - minimum number of values
    MinCount(usize),
    /// sh:maxCount - maximum number of values
    MaxCount(usize),

    // Value Range Constraints
    /// sh:minExclusive - value must be greater than
    MinExclusive(Node<'a>),
    /// sh:minInclusive - value must be greater than or equal
    MinInclusive(Node<'a>),
    /// sh:maxExclusive - value must be less than
    MaxExclusive(Node<'a>),
    /// sh:maxInclusive - value must be less than or equal
    MaxInclusive(Node<'a>),

    // String Constraints
    /// sh:minLength - minimum string length
    MinLength(usize),
    /// sh:maxLength - maximum string length
    MaxLength(usize),
    /// sh:pattern - regex pattern
    Pattern { pattern: String, flags: Option<String> },
    /// sh:languageIn - language tag must be in list
    LanguageIn(Vec<String>),
    /// sh:uniqueLang - at most one value per language
    UniqueLang(bool),

    // Property Pair Constraints
    /// sh:equals - values must equal values of another property
    Equals(Node<'a>),
    /// sh:disjoint - values must be disjoint from another property
    Disjoint(Node<'a>),
    /// sh:lessThan - values must be less than another property
    LessThan(Node<'a>),
    /// sh:lessThanOrEquals - values must be less than or equal
    LessThanOrEquals(Node<'a>),

    // Value Constraints
    /// sh:in - value must be in list
    In(Vec<Node<'a>>),
    /// sh:hasValue - must have specific value
    HasValue(Node<'a>),

    // Logical Constraints
    /// sh:closed - only specified properties allowed
    Closed { ignored_properties: Vec<Node<'a>> },
}

/// SHACL Node Kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// sh:IRI
    IRI,
    /// sh:BlankNode
    BlankNode,
    /// sh:Literal
    Literal,
    /// sh:BlankNodeOrIRI
    BlankNodeOrIRI,
    /// sh:BlankNodeOrLiteral
    BlankNodeOrLiteral,
    /// sh:IRIOrLiteral
    IRIOrLiteral,
}

/// Shape-based constraint (references another shape)
#[derive(Debug, Clone)]
pub enum ShapeConstraint<'a> {
    /// sh:node - value must conform to shape
    Node(Node<'a>),
    /// sh:property - property must conform to property shape
    Property(PropertyShape<'a>),
    /// sh:and - must conform to all shapes
    And(Vec<Node<'a>>),
    /// sh:or - must conform to at least one shape
    Or(Vec<Node<'a>>),
    /// sh:xone - must conform to exactly one shape
    Xone(Vec<Node<'a>>),
    /// sh:not - must NOT conform to shape
    Not(Node<'a>),
}

/// Property shape definition (separate from enum for nesting)
#[derive(Debug, Clone)]
pub struct PropertyShape<'a> {
    /// Shape IRI
    pub id: Option<Node<'a>>,
    /// Property path
    pub path: PropertyPath<'a>,
    /// Value constraints
    pub constraints: Vec<Constraint<'a>>,
    /// Nested shape constraints
    pub shapes: Vec<ShapeConstraint<'a>>,
    /// Deactivated flag
    pub deactivated: bool,
    /// Severity level
    pub severity: Option<&'a str>,
    /// Custom message
    pub message: Option<&'a str>,
}

impl<'a> PropertyShape<'a> {
    /// Create a new property shape
    pub fn new(path: PropertyPath<'a>) -> Self {
        Self {
            id: None,
            path,
            constraints: Vec::new(),
            shapes: Vec::new(),
            deactivated: false,
            severity: None,
            message: None,
        }
    }

    /// Add a constraint
    pub fn with_constraint(mut self, constraint: Constraint<'a>) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: &'a str) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set message
    pub fn with_message(mut self, message: &'a str) -> Self {
        self.message = Some(message);
        self
    }
}

impl<'a> Shape<'a> {
    /// Create a new node shape
    pub fn node_shape() -> Self {
        Shape::NodeShape {
            id: None,
            targets: Vec::new(),
            properties: Vec::new(),
            constraints: Vec::new(),
            shapes: Vec::new(),
            deactivated: false,
            severity: None,
            message: None,
        }
    }

    /// Add a target
    pub fn with_target(mut self, target: Target<'a>) -> Self {
        if let Shape::NodeShape { ref mut targets, .. } = self {
            targets.push(target);
        }
        self
    }

    /// Add a property shape
    pub fn with_property(mut self, property: PropertyShape<'a>) -> Self {
        if let Shape::NodeShape { ref mut properties, .. } = self {
            properties.push(property);
        }
        self
    }

    /// Add a constraint
    pub fn with_constraint(mut self, constraint: Constraint<'a>) -> Self {
        match &mut self {
            Shape::NodeShape { constraints, .. } => constraints.push(constraint),
            Shape::PropertyShape { constraints, .. } => constraints.push(constraint),
        }
        self
    }

    /// Check if shape is deactivated
    pub fn is_deactivated(&self) -> bool {
        match self {
            Shape::NodeShape { deactivated, .. } => *deactivated,
            Shape::PropertyShape { deactivated, .. } => *deactivated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdf_model::Dictionary;
    use std::sync::Arc;

    #[test]
    fn test_node_shape_builder() {
        let shape = Shape::node_shape()
            .with_constraint(Constraint::MinCount(1))
            .with_constraint(Constraint::MaxCount(10));

        match shape {
            Shape::NodeShape { constraints, .. } => {
                assert_eq!(constraints.len(), 2);
            }
            _ => panic!("Expected NodeShape"),
        }
    }

    #[test]
    fn test_property_shape_builder() {
        let dict = Arc::new(Dictionary::new());
        let name_iri = dict.intern("http://example.org/name");

        let prop = PropertyShape::new(PropertyPath::Predicate(Node::iri(name_iri)))
            .with_constraint(Constraint::MinLength(1))
            .with_constraint(Constraint::MaxLength(100))
            .with_severity("Violation")
            .with_message("Name must be between 1 and 100 characters");

        assert_eq!(prop.constraints.len(), 2);
        assert_eq!(prop.severity, Some("Violation"));
        assert_eq!(prop.message, Some("Name must be between 1 and 100 characters"));
    }
}
