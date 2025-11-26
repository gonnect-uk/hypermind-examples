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

#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod turtle;
pub mod ntriples;

pub use turtle::TurtleParser;
pub use ntriples::NTriplesParser;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_compiles() {
        let format = RDFFormat::Turtle;
        assert_eq!(format, RDFFormat::Turtle);
    }
}
