//! RDF node types and builders

/// Ergonomic wrapper for RDF nodes
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// IRI/URI node
    IRI(String),
    /// Literal value with optional datatype and language tag
    Literal {
        /// The literal value
        value: String,
        /// Optional datatype IRI
        datatype: Option<String>,
        /// Optional language tag
        lang: Option<String>,
    },
    /// Blank node with identifier
    BlankNode(String),
}

/// Node type for pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// IRI node
    IRI,
    /// Literal node
    Literal,
    /// Blank node
    BlankNode,
}

impl Node {
    /// Create an IRI node
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::Node;
    ///
    /// let node = Node::iri("http://example.org/alice");
    /// ```
    pub fn iri<S: Into<String>>(iri: S) -> Self {
        Node::IRI(iri.into())
    }

    /// Create a plain literal node
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::Node;
    ///
    /// let node = Node::literal("Alice");
    /// ```
    pub fn literal<S: Into<String>>(value: S) -> Self {
        Node::Literal {
            value: value.into(),
            datatype: None,
            lang: None,
        }
    }

    /// Create a typed literal node
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::Node;
    ///
    /// let node = Node::typed_literal(
    ///     "42",
    ///     "http://www.w3.org/2001/XMLSchema#integer"
    /// );
    /// ```
    pub fn typed_literal<S: Into<String>, D: Into<String>>(value: S, datatype: D) -> Self {
        Node::Literal {
            value: value.into(),
            datatype: Some(datatype.into()),
            lang: None,
        }
    }

    /// Create a language-tagged literal node
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::Node;
    ///
    /// let node = Node::lang_literal("Hello", "en");
    /// ```
    pub fn lang_literal<S: Into<String>, L: Into<String>>(value: S, lang: L) -> Self {
        Node::Literal {
            value: value.into(),
            datatype: None,
            lang: Some(lang.into()),
        }
    }

    /// Create an integer literal node
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::Node;
    ///
    /// let node = Node::integer(42);
    /// ```
    pub fn integer(value: i64) -> Self {
        Node::typed_literal(
            value.to_string(),
            "http://www.w3.org/2001/XMLSchema#integer",
        )
    }

    /// Create a boolean literal node
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::Node;
    ///
    /// let node = Node::boolean(true);
    /// ```
    pub fn boolean(value: bool) -> Self {
        Node::typed_literal(
            value.to_string(),
            "http://www.w3.org/2001/XMLSchema#boolean",
        )
    }

    /// Create a blank node
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::Node;
    ///
    /// let node = Node::blank("b0");
    /// ```
    pub fn blank<S: Into<String>>(id: S) -> Self {
        Node::BlankNode(id.into())
    }

    /// Get the node type
    pub fn node_type(&self) -> NodeType {
        match self {
            Node::IRI(_) => NodeType::IRI,
            Node::Literal { .. } => NodeType::Literal,
            Node::BlankNode(_) => NodeType::BlankNode,
        }
    }

    /// Check if this is an IRI node
    pub fn is_iri(&self) -> bool {
        matches!(self, Node::IRI(_))
    }

    /// Check if this is a literal node
    pub fn is_literal(&self) -> bool {
        matches!(self, Node::Literal { .. })
    }

    /// Check if this is a blank node
    pub fn is_blank(&self) -> bool {
        matches!(self, Node::BlankNode(_))
    }

    /// Get the IRI value if this is an IRI node
    pub fn as_iri(&self) -> Option<&str> {
        match self {
            Node::IRI(iri) => Some(iri),
            _ => None,
        }
    }

    /// Get the literal value if this is a literal node
    pub fn as_literal(&self) -> Option<(&str, Option<&str>, Option<&str>)> {
        match self {
            Node::Literal {
                value,
                datatype,
                lang,
            } => Some((
                value,
                datatype.as_deref(),
                lang.as_deref(),
            )),
            _ => None,
        }
    }

    /// Get the blank node ID if this is a blank node
    pub fn as_blank(&self) -> Option<&str> {
        match self {
            Node::BlankNode(id) => Some(id),
            _ => None,
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::IRI(iri) => write!(f, "<{}>", iri),
            Node::Literal {
                value,
                datatype,
                lang,
            } => {
                write!(f, "\"{}\"", value)?;
                if let Some(dt) = datatype {
                    write!(f, "^^<{}>", dt)?;
                }
                if let Some(l) = lang {
                    write!(f, "@{}", l)?;
                }
                Ok(())
            }
            Node::BlankNode(id) => write!(f, "_:{}", id),
        }
    }
}
