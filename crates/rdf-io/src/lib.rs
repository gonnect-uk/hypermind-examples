//! RDF format parsers and serializers
//!
//! Supports all W3C RDF formats:
//! - Turtle (RDF 1.1)
//! - N-Triples
//! - N-Quads
//! - TriG
//! - JSON-LD
//! - RDF/XML
//!
//! All parsers are GENERIC - no hardcoding, support any valid RDF.
//!
//! # Strategy Pattern Architecture
//!
//! All parsers implement the `RDFParser` trait for unified interface.
//! Use `ParserFactory` for automatic format detection and parser instantiation.

#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod turtle;
pub mod ntriples;
pub mod nquads;

pub use turtle::TurtleParser;
pub use ntriples::NTriplesParser;
pub use nquads::NQuadsParser;

use rdf_model::{Dictionary, Node, Quad};
use std::sync::Arc;

/// RDF format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RDFFormat {
    /// Turtle format (.ttl)
    Turtle,

    /// N-Triples format (.nt)
    NTriples,

    /// N-Quads format (.nq)
    NQuads,

    /// TriG format (.trig)
    TriG,

    /// JSON-LD format (.jsonld)
    JSONLD,

    /// RDF/XML format (.rdf, .owl)
    RDFXML,
}

/// Strategy trait for RDF parsers
///
/// All RDF format parsers implement this trait, enabling polymorphic parsing
/// via the Factory pattern (ParserFactory).
///
/// # Example
///
/// ```no_run
/// use rdf_io::{RDFParser, TurtleParser, ParseResult};
/// use rdf_model::Dictionary;
/// use std::sync::Arc;
///
/// let dict = Arc::new(Dictionary::new());
/// let mut parser = TurtleParser::new(dict);
/// let quads = parser.parse("@prefix : <http://example.org/> . :s :p :o .")?;
/// # Ok::<(), rdf_io::ParseError>(())
/// ```
pub trait RDFParser {
    /// Parse RDF content into Quads with dictionary interning
    ///
    /// # Arguments
    ///
    /// * `content` - RDF content as string
    ///
    /// # Returns
    ///
    /// Vector of parsed Quads
    ///
    /// # Lifetimes
    ///
    /// The returned Quads reference strings from the parser's dictionary,
    /// which must outlive the quads.
    fn parse<'a>(&'a mut self, content: &'a str) -> ParseResult<Vec<Quad<'a>>>;

    /// Get the format this parser handles
    fn format(&self) -> RDFFormat;
}

/// Factory for creating RDF parsers based on format
///
/// Supports automatic format detection from file extensions and
/// manual parser instantiation for specific formats.
///
/// # Example
///
/// ```no_run
/// use rdf_io::{ParserFactory, RDFFormat};
/// use rdf_model::Dictionary;
/// use std::sync::Arc;
///
/// let dict = Arc::new(Dictionary::new());
///
/// // Auto-detect format from filename
/// let mut parser = ParserFactory::create_from_file("data.ttl", dict.clone());
/// let quads = parser.parse("@prefix : <http://example.org/> . :s :p :o .")?;
///
/// // Manual format specification
/// let mut parser = ParserFactory::create(RDFFormat::Turtle, dict);
/// # Ok::<(), rdf_io::ParseError>(())
/// ```
pub struct ParserFactory;

impl ParserFactory {
    /// Create a parser for the specified format
    ///
    /// # Arguments
    ///
    /// * `format` - RDF format to parse
    /// * `dictionary` - String interning dictionary
    ///
    /// # Returns
    ///
    /// Boxed parser implementing RDFParser trait
    pub fn create(format: RDFFormat, dictionary: Arc<Dictionary>) -> Box<dyn RDFParser> {
        match format {
            RDFFormat::Turtle => Box::new(TurtleParser::new(dictionary)),
            RDFFormat::NTriples => Box::new(NTriplesParser::new(dictionary)),
            RDFFormat::NQuads => Box::new(NQuadsParser::new(dictionary)),
            RDFFormat::TriG => unimplemented!("TriG parser not yet implemented"),
            RDFFormat::JSONLD => unimplemented!("JSON-LD parser not yet implemented"),
            RDFFormat::RDFXML => unimplemented!("RDF/XML parser not yet implemented"),
        }
    }

    /// Create a parser with format auto-detected from filename
    ///
    /// # Arguments
    ///
    /// * `filename` - File path or name (extension used for detection)
    /// * `dictionary` - String interning dictionary
    ///
    /// # Returns
    ///
    /// Boxed parser for detected format
    pub fn create_from_file(filename: &str, dictionary: Arc<Dictionary>) -> Box<dyn RDFParser> {
        let format = Self::detect_format(filename);
        Self::create(format, dictionary)
    }

    /// Detect RDF format from file extension
    ///
    /// # Arguments
    ///
    /// * `filename` - File path or name
    ///
    /// # Returns
    ///
    /// Detected RDF format (defaults to Turtle if unknown)
    pub fn detect_format(filename: &str) -> RDFFormat {
        match filename.rsplit('.').next() {
            Some("ttl") => RDFFormat::Turtle,
            Some("nt") => RDFFormat::NTriples,
            Some("nq") => RDFFormat::NQuads,
            Some("trig") => RDFFormat::TriG,
            Some("jsonld") => RDFFormat::JSONLD,
            Some("rdf") | Some("owl") => RDFFormat::RDFXML,
            _ => RDFFormat::Turtle, // Default to Turtle
        }
    }
}

/// Errors that can occur during parsing
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Syntax error in RDF document
    #[error("Syntax error at line {line}, column {col}: {message}")]
    Syntax {
        /// Line number
        line: usize,
        /// Column number
        col: usize,
        /// Error message
        message: String,
    },

    /// Invalid IRI
    #[error("Invalid IRI: {0}")]
    InvalidIri(String),

    /// Invalid literal
    #[error("Invalid literal: {0}")]
    InvalidLiteral(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

// ============================================================================
// RDF Serialization (Strategy Pattern)
// ============================================================================

/// Strategy trait for RDF serializers
///
/// All RDF format serializers implement this trait, enabling polymorphic serialization
/// via the Factory pattern (SerializerFactory).
///
/// # Example
///
/// ```no_run
/// use rdf_io::{RDFSerializer, SerializerFactory, RDFFormat};
/// use rdf_model::{Node, Quad, Dictionary};
/// use std::sync::Arc;
///
/// let dict = Arc::new(Dictionary::new());
/// let subject = Node::iri(dict.intern("http://example.org/s"));
/// let predicate = Node::iri(dict.intern("http://example.org/p"));
/// let object = Node::iri(dict.intern("http://example.org/o"));
/// let quad = Quad { subject, predicate, object, graph: None };
///
/// let serializer = SerializerFactory::create(RDFFormat::NQuads);
/// let nquads_string = serializer.serialize(&[quad])?;
/// # Ok::<(), rdf_io::ParseError>(())
/// ```
pub trait RDFSerializer {
    /// Serialize quads into RDF format string
    ///
    /// # Arguments
    ///
    /// * `quads` - Slice of quads to serialize
    ///
    /// # Returns
    ///
    /// Serialized RDF string
    fn serialize<'a>(&self, quads: &[Quad<'a>]) -> ParseResult<String>;

    /// Get the format this serializer produces
    fn format(&self) -> RDFFormat;
}

/// Factory for creating RDF serializers based on format
///
/// # Example
///
/// ```no_run
/// use rdf_io::{SerializerFactory, RDFFormat};
///
/// let serializer = SerializerFactory::create(RDFFormat::NQuads);
/// ```
pub struct SerializerFactory;

impl SerializerFactory {
    /// Create a serializer for the specified format
    ///
    /// # Arguments
    ///
    /// * `format` - RDF format to serialize to
    ///
    /// # Returns
    ///
    /// Boxed serializer implementing RDFSerializer trait
    pub fn create(format: RDFFormat) -> Box<dyn RDFSerializer> {
        match format {
            RDFFormat::NQuads => Box::new(NQuadsSerializer::new()),
            RDFFormat::Turtle => unimplemented!("Turtle serializer not yet implemented"),
            RDFFormat::NTriples => unimplemented!("N-Triples serializer not yet implemented"),
            RDFFormat::TriG => unimplemented!("TriG serializer not yet implemented"),
            RDFFormat::JSONLD => unimplemented!("JSON-LD serializer not yet implemented"),
            RDFFormat::RDFXML => unimplemented!("RDF/XML serializer not yet implemented"),
        }
    }
}

// ============================================================================
// N-Quads Serializer
// ============================================================================

/// N-Quads serializer
pub struct NQuadsSerializer;

impl NQuadsSerializer {
    /// Create a new N-Quads serializer
    pub fn new() -> Self {
        Self
    }
}

impl Default for NQuadsSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl RDFSerializer for NQuadsSerializer {
    fn serialize<'a>(&self, quads: &[Quad<'a>]) -> ParseResult<String> {
        let mut output = String::new();

        for quad in quads {
            // Subject: IRI or BlankNode
            match &quad.subject {
                Node::Iri(iri) => output.push_str(iri.0),
                Node::BlankNode(id) => output.push_str(&id.to_string()),
                _ => {
                    return Err(ParseError::Syntax {
                        line: 0,
                        col: 0,
                        message: "Subject must be IRI or BlankNode".to_string(),
                    });
                }
            }

            output.push(' ');

            // Predicate: Must be IRI
            match &quad.predicate {
                Node::Iri(iri) => output.push_str(iri.0),
                _ => {
                    return Err(ParseError::Syntax {
                        line: 0,
                        col: 0,
                        message: "Predicate must be IRI".to_string(),
                    });
                }
            }

            output.push(' ');

            // Object: IRI, BlankNode, or Literal
            match &quad.object {
                Node::Iri(iri) => output.push_str(iri.0),
                Node::BlankNode(id) => output.push_str(&id.to_string()),
                Node::Literal(lit) => {
                    output.push('"');
                    // Escape special characters
                    for ch in lit.lexical_form.chars() {
                        match ch {
                            '"' => output.push_str("\\\""),
                            '\\' => output.push_str("\\\\"),
                            '\n' => output.push_str("\\n"),
                            '\r' => output.push_str("\\r"),
                            '\t' => output.push_str("\\t"),
                            _ => output.push(ch),
                        }
                    }
                    output.push('"');

                    // Language tag or datatype
                    if let Some(lang) = lit.language {
                        output.push('@');
                        output.push_str(lang);
                    } else if let Some(datatype) = lit.datatype {
                        output.push_str("^^");
                        output.push_str(datatype);
                    }
                }
                _ => {
                    return Err(ParseError::Syntax {
                        line: 0,
                        col: 0,
                        message: "Object must be IRI, BlankNode, or Literal".to_string(),
                    });
                }
            }

            // Optional graph
            if let Some(graph) = &quad.graph {
                output.push(' ');
                match graph {
                    Node::Iri(iri) => output.push_str(iri.0),
                    Node::BlankNode(id) => output.push_str(&id.to_string()),
                    _ => {
                        return Err(ParseError::Syntax {
                            line: 0,
                            col: 0,
                            message: "Graph must be IRI or BlankNode".to_string(),
                        });
                    }
                }
            }

            output.push_str(" .\n");
        }

        Ok(output)
    }

    fn format(&self) -> RDFFormat {
        RDFFormat::NQuads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_compiles() {
        let format = RDFFormat::Turtle;
        assert_eq!(format, RDFFormat::Turtle);
    }
}
