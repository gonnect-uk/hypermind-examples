//! Zero-copy RDF/RDF-star type system with string interning
//!
//! This crate provides the core RDF data model with:
//! - Zero-copy node representations using lifetimes
//! - String interning for memory efficiency
//! - RDF-star support (quoted triples)
//! - Arena allocation for fast node creation
//!
//! # Architecture
//!
//! Follows Apache Jena's node hierarchy but optimized for Rust:
//! - No GC overhead (uses lifetimes and arenas)
//! - Zero-copy semantics throughout
//! - Type-safe at compile time
//!
//! # Example
//!
//! ```rust,ignore
//! use rdf_model::{Node, Triple, Dictionary};
//!
//! let dict = Dictionary::new();
//! let subject = Node::iri(dict.intern("http://example.org/subject"));
//! let predicate = Node::iri(dict.intern("http://example.org/predicate"));
//! let object = Node::literal_str(dict.intern("value"));
//!
//! let triple = Triple::new(subject, predicate, object);
//! ```

#![deny(unsafe_code)]  // Can be overridden with #[allow(unsafe_code)] per module
#![warn(missing_docs, rust_2018_idioms)]

mod dictionary;
mod node;
mod quad;
mod triple;
mod vocab;

pub use dictionary::Dictionary;
pub use node::{BlankNodeId, IriRef, Literal, Node, Variable};
pub use quad::Quad;
pub use triple::Triple;
pub use vocab::Vocabulary;

/// Errors that can occur when working with RDF data
#[derive(Debug, thiserror::Error)]
pub enum RdfError {
    /// Invalid IRI format
    #[error("Invalid IRI: {0}")]
    InvalidIri(String),

    /// Invalid literal format
    #[error("Invalid literal: {0}")]
    InvalidLiteral(String),

    /// Invalid blank node ID
    #[error("Invalid blank node ID: {0}")]
    InvalidBlankNode(String),

    /// Invalid quoted triple
    #[error("Invalid quoted triple: {0}")]
    InvalidQuotedTriple(String),
}

/// Result type for RDF operations
pub type Result<T> = std::result::Result<T, RdfError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_compiles() {
        // Basic smoke test
        let dict = Dictionary::new();
        assert!(dict.is_empty());
    }
}
