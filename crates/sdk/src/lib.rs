//! # rust-kgdb-sdk: High-Level Ergonomic API
//!
//! This crate provides a user-friendly, type-safe API for working with rust-kgdb,
//! wrapping the low-level components with ergonomic builders and convenience methods.
//!
//! ## Features
//!
//! - **Fluent API**: Builder pattern for queries and updates
//! - **Type Safety**: Compile-time guarantees for SPARQL operations
//! - **Zero Cost**: Thin wrapper with no runtime overhead
//! - **Ergonomic**: Rust idioms (iterators, Result, Option)
//!
//! ## Quick Start
//!
//! ```rust
//! use rust_kgdb_sdk::{GraphDB, Node};
//!
//! // Create in-memory database
//! let mut db = GraphDB::in_memory();
//!
//! // Insert triples
//! db.insert()
//!     .triple(
//!         Node::iri("http://example.org/alice"),
//!         Node::iri("http://xmlns.com/foaf/0.1/name"),
//!         Node::literal("Alice"),
//!     )
//!     .execute()?;
//!
//! // Query with SPARQL
//! let results = db.query()
//!     .sparql("SELECT ?name WHERE { ?person <http://xmlns.com/foaf/0.1/name> ?name }")
//!     .execute()?;
//!
//! for binding in results {
//!     println!("Name: {:?}", binding.get("name"));
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![warn(missing_docs)]

mod error;
mod graphdb;
mod node;
mod query_builder;
mod transaction;
mod update_builder;

pub use error::{Error, Result};
pub use graphdb::GraphDB;
pub use node::{Node, NodeType};
pub use query_builder::{OwnedBinding, QueryBuilder, QueryResult};
pub use transaction::Transaction;
pub use update_builder::UpdateBuilder;

// Re-export commonly used types
pub use rdf_model::{Dictionary, Quad, Triple};
pub use sparql::Query;
pub use storage::QuadStore;

/// SDK version matching rust-kgdb core
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{Error, GraphDB, Node, NodeType, QueryBuilder, Result, UpdateBuilder};
}
