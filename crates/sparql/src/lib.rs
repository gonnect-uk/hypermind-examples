//! SPARQL 1.1 Query Language Implementation
//!
//! Production-grade SPARQL parser and algebra for mobile hypergraph database.
//!
//! Features:
//! - Complete SPARQL 1.1 support (SELECT, CONSTRUCT, DESCRIBE, ASK)
//! - Zero-copy parsing with lifetimes
//! - Grammar-based (pest) - NO string manipulation
//! - Visitor pattern for algebra traversal
//! - Property paths (*, +, ?, |, ^, /)
//! - All aggregates (COUNT, SUM, AVG, MIN, MAX, SAMPLE, GROUP_CONCAT)
//! - All builtin functions (100+ functions)
//! - SPARQL Update (INSERT, DELETE, LOAD, CLEAR, etc.)
//!
//! Based on:
//! - W3C SPARQL 1.1 Specification
//! - Apache Jena ARQ query engine

#![warn(unsafe_code, missing_docs, rust_2018_idioms)]

pub mod algebra;
pub mod bindings;
pub mod executor;
pub mod optimizer;
pub mod parser;

pub use algebra::*;
pub use bindings::*;
pub use executor::*;
pub use optimizer::*;
pub use parser::*;
