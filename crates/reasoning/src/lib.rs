//! Reasoning engines for RDF, RDFS, OWL 2, and custom rule systems
//!
//! Implements Apache Jena-compatible reasoning with mobile optimizations:
//! - RDFS: All 13 W3C entailment rules
//! - OWL 2 RL: 61 production rules
//! - OWL 2 EL: Polynomial-time reasoning
//! - OWL 2 QL: Query rewriting
//! - Custom RETE-based rule engine
//!
//! Zero compromises, production-grade, sub-millisecond inference.

#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

/// RDFS reasoning implementation
pub mod rdfs;
/// OWL 2 reasoning implementation
pub mod owl2;
/// Transitive closure reasoning
pub mod transitive;
/// RETE rule engine
pub mod rete;

pub use rdfs::RDFSReasoner;

/// Reasoner error types
#[derive(Debug, thiserror::Error)]
pub enum ReasonerError {
    /// Inconsistency detected
    #[error("Inconsistency detected: {0}")]
    Inconsistency(String),

    /// Invalid rule
    #[error("Invalid rule: {0}")]
    InvalidRule(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Cycle detected
    #[error("Cycle detected: {0}")]
    Cycle(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
}

/// Result type for reasoning operations
pub type ReasonerResult<T> = Result<T, ReasonerError>;

/// Reasoner configuration
#[derive(Debug, Clone)]
pub struct ReasonerConfig {
    /// Enable rule tracing
    pub trace_rules: bool,
    /// Max inference depth
    pub max_depth: usize,
    /// Max derived triples
    pub max_inferred: usize,
    /// Enable incremental
    pub incremental: bool,
    /// Use parallel
    pub parallel: bool,
}

impl Default for ReasonerConfig {
    fn default() -> Self {
        Self {
            trace_rules: false,
            max_depth: 100,
            max_inferred: 1_000_000,
            incremental: true,
            parallel: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ReasonerConfig::default();
        assert_eq!(config.max_depth, 100);
    }
}
