//! RDF Node types with zero-copy semantics
//!
//! Implements the core RDF data model with lifetime-bound references
//! for maximum performance and minimal allocations.

use crate::Triple;
use std::fmt;

/// RDF Node - core type in the RDF model
///
/// Uses borrowed references ('a lifetime) for zero-copy semantics.
/// All strings are expected to be interned via Dictionary.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Node<'a> {
    /// IRI/URI reference
    /// Example: <http://example.org/resource>
    Iri(IriRef<'a>),

    /// Literal value (string, number, date, etc.)
    /// Example: "John"@en or "42"^^xsd:integer
    Literal(Literal<'a>),

    /// Blank node with unique identifier
    /// Example: _:b0
    BlankNode(BlankNodeId),

    /// Quoted triple (RDF-star)
    /// Example: << :alice :knows :bob >>
    QuotedTriple(Box<Triple<'a>>),

    /// SPARQL variable (for patterns)
    /// Example: ?x
    Variable(Variable<'a>),
}

impl<'a> Node<'a> {
    /// Create an IRI node
    pub fn iri(iri: &'a str) -> Self {
        Node::Iri(IriRef(iri))
    }

    /// Create a simple string literal
    pub fn literal_str(value: &'a str) -> Self {
        Node::Literal(Literal {
            lexical_form: value,
            language: None,
            datatype: None,
        })
    }

    /// Create a language-tagged literal
    pub fn literal_lang(value: &'a str, language: &'a str) -> Self {
        Node::Literal(Literal {
            lexical_form: value,
            language: Some(language),
            datatype: None,
        })
    }

    /// Create a typed literal
    pub fn literal_typed(value: &'a str, datatype: &'a str) -> Self {
        Node::Literal(Literal {
            lexical_form: value,
            language: None,
            datatype: Some(datatype),
        })
    }

    /// Create a blank node
    pub fn blank(id: u64) -> Self {
        Node::BlankNode(BlankNodeId(id))
    }

    /// Create a SPARQL variable
    pub fn variable(name: &'a str) -> Self {
        Node::Variable(Variable(name))
    }

    /// Create a quoted triple (RDF-star)
    pub fn quoted_triple(triple: Triple<'a>) -> Self {
        Node::QuotedTriple(Box::new(triple))
    }

    /// Check if this is an IRI
    pub fn is_iri(&self) -> bool {
        matches!(self, Node::Iri(_))
    }

    /// Check if this is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self, Node::Literal(_))
    }

    /// Check if this is a blank node
    pub fn is_blank_node(&self) -> bool {
        matches!(self, Node::BlankNode(_))
    }

    /// Check if this is a quoted triple
    pub fn is_quoted_triple(&self) -> bool {
        matches!(self, Node::QuotedTriple(_))
    }

    /// Check if this is a variable
    pub fn is_variable(&self) -> bool {
        matches!(self, Node::Variable(_))
    }

    /// Get as IRI reference
    pub fn as_iri(&self) -> Option<&IriRef<'a>> {
        match self {
            Node::Iri(iri) => Some(iri),
            _ => None,
        }
    }

    /// Get as literal
    pub fn as_literal(&self) -> Option<&Literal<'a>> {
        match self {
            Node::Literal(lit) => Some(lit),
            _ => None,
        }
    }

    /// Get as blank node ID
    pub fn as_blank_node(&self) -> Option<BlankNodeId> {
        match self {
            Node::BlankNode(id) => Some(*id),
            _ => None,
        }
    }

    /// Get as variable
    pub fn as_variable(&self) -> Option<&Variable<'a>> {
        match self {
            Node::Variable(var) => Some(var),
            _ => None,
        }
    }

    /// Get string value for display/comparison
    pub fn value(&self) -> String {
        match self {
            Node::Iri(iri) => iri.0.to_string(),
            Node::Literal(lit) => lit.lexical_form.to_string(),
            Node::BlankNode(id) => format!("_:b{}", id.0),
            Node::QuotedTriple(triple) => format!("<< {} >>", triple),
            Node::Variable(var) => format!("?{}", var.0),
        }
    }
}

impl<'a> fmt::Debug for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Iri(iri) => write!(f, "Iri({})", iri.0),
            Node::Literal(lit) => write!(f, "Literal({:?})", lit),
            Node::BlankNode(id) => write!(f, "BlankNode({})", id.0),
            Node::QuotedTriple(triple) => write!(f, "QuotedTriple({:?})", triple),
            Node::Variable(var) => write!(f, "Variable({})", var.0),
        }
    }
}

impl<'a> fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Iri(iri) => write!(f, "<{}>", iri.0),
            Node::Literal(lit) => write!(f, "{}", lit),
            Node::BlankNode(id) => write!(f, "_:b{}", id.0),
            Node::QuotedTriple(triple) => write!(f, "<< {} >>", triple),
            Node::Variable(var) => write!(f, "?{}", var.0),
        }
    }
}

/// IRI reference (borrowed string)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IriRef<'a>(pub &'a str);

impl<'a> IriRef<'a> {
    /// Get the full IRI string
    pub fn as_str(&self) -> &'a str {
        self.0
    }

    /// Extract namespace (part before # or last /)
    pub fn namespace(&self) -> &'a str {
        let s = self.0;
        if let Some(pos) = s.rfind('#') {
            &s[..=pos]
        } else if let Some(pos) = s.rfind('/') {
            &s[..=pos]
        } else {
            ""
        }
    }

    /// Extract local name (part after # or last /)
    pub fn local_name(&self) -> &'a str {
        let s = self.0;
        if let Some(pos) = s.rfind('#') {
            &s[pos + 1..]
        } else if let Some(pos) = s.rfind('/') {
            &s[pos + 1..]
        } else {
            s
        }
    }
}

impl<'a> fmt::Debug for IriRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IriRef({})", self.0)
    }
}

impl<'a> fmt::Display for IriRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.0)
    }
}

/// RDF Literal with optional language or datatype
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Literal<'a> {
    /// Lexical form (string representation)
    pub lexical_form: &'a str,

    /// Language tag (e.g., "en", "fr")
    pub language: Option<&'a str>,

    /// Datatype IRI (e.g., "http://www.w3.org/2001/XMLSchema#integer")
    pub datatype: Option<&'a str>,
}

impl<'a> Literal<'a> {
    /// Check if this is a plain literal (no language or datatype)
    pub fn is_plain(&self) -> bool {
        self.language.is_none() && self.datatype.is_none()
    }

    /// Check if this has a language tag
    pub fn has_language(&self) -> bool {
        self.language.is_some()
    }

    /// Check if this has a datatype
    pub fn has_datatype(&self) -> bool {
        self.datatype.is_some()
    }

    /// Try to parse as integer
    pub fn as_i64(&self) -> Option<i64> {
        self.lexical_form.parse().ok()
    }

    /// Try to parse as float
    pub fn as_f64(&self) -> Option<f64> {
        self.lexical_form.parse().ok()
    }

    /// Try to parse as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self.lexical_form {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        }
    }
}

impl<'a> fmt::Debug for Literal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Literal")
            .field("lexical_form", &self.lexical_form)
            .field("language", &self.language)
            .field("datatype", &self.datatype)
            .finish()
    }
}

impl<'a> fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.lexical_form)?;

        if let Some(lang) = self.language {
            write!(f, "@{}", lang)?;
        } else if let Some(dt) = self.datatype {
            write!(f, "^^<{}>", dt)?;
        }

        Ok(())
    }
}

/// Blank node identifier (numeric ID)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlankNodeId(pub u64);

impl BlankNodeId {
    /// Create a new blank node ID
    pub fn new(id: u64) -> Self {
        BlankNodeId(id)
    }

    /// Get the numeric ID
    pub fn id(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for BlankNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlankNodeId({})", self.0)
    }
}

impl fmt::Display for BlankNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_:b{}", self.0)
    }
}

/// SPARQL variable
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variable<'a>(pub &'a str);

impl<'a> Variable<'a> {
    /// Create a new variable
    pub fn new(name: &'a str) -> Self {
        Variable(name)
    }

    /// Get the variable name
    pub fn name(&self) -> &'a str {
        self.0
    }
}

impl<'a> fmt::Debug for Variable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Variable({})", self.0)
    }
}

impl<'a> fmt::Display for Variable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "?{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iri_node() {
        let node = Node::iri("http://example.org/resource");

        assert!(node.is_iri());
        assert_eq!(node.value(), "http://example.org/resource");
    }

    #[test]
    fn test_literal_str() {
        let node = Node::literal_str("Hello World");

        assert!(node.is_literal());
        if let Some(lit) = node.as_literal() {
            assert_eq!(lit.lexical_form, "Hello World");
            assert!(lit.is_plain());
        }
    }

    #[test]
    fn test_literal_lang() {
        let node = Node::literal_lang("Hello", "en");

        if let Some(lit) = node.as_literal() {
            assert_eq!(lit.language, Some("en"));
            assert!(lit.has_language());
        }
    }

    #[test]
    fn test_literal_typed() {
        let node = Node::literal_typed("42", "http://www.w3.org/2001/XMLSchema#integer");

        if let Some(lit) = node.as_literal() {
            assert_eq!(lit.datatype, Some("http://www.w3.org/2001/XMLSchema#integer"));
            assert!(lit.has_datatype());
            assert_eq!(lit.as_i64(), Some(42));
        }
    }

    #[test]
    fn test_blank_node() {
        let node = Node::blank(123);

        assert!(node.is_blank_node());
        assert_eq!(node.as_blank_node(), Some(BlankNodeId(123)));
    }

    #[test]
    fn test_variable() {
        let node = Node::variable("x");

        assert!(node.is_variable());
        if let Some(var) = node.as_variable() {
            assert_eq!(var.name(), "x");
        }
    }

    #[test]
    fn test_iri_namespace_local_name() {
        let iri = IriRef("http://example.org/ns#localName");

        assert_eq!(iri.namespace(), "http://example.org/ns#");
        assert_eq!(iri.local_name(), "localName");
    }

    #[test]
    fn test_literal_parse_values() {
        let int_lit = Literal {
            lexical_form: "42",
            language: None,
            datatype: None,
        };
        assert_eq!(int_lit.as_i64(), Some(42));

        let float_lit = Literal {
            lexical_form: "3.14",
            language: None,
            datatype: None,
        };
        assert_eq!(float_lit.as_f64(), Some(3.14));

        let bool_lit = Literal {
            lexical_form: "true",
            language: None,
            datatype: None,
        };
        assert_eq!(bool_lit.as_bool(), Some(true));
    }
}
