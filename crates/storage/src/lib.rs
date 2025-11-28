//! Pluggable storage backends for RDF quad stores
//!
//! This crate provides a trait-based abstraction over different storage backends,
//! allowing the same quad store API to work with:
//! - In-memory storage (ultra-fast, no persistence)
//! - RocksDB (persistent, ACID transactions)
//! - LMDB (alternative persistent backend)
//!
//! # Design Principles
//!
//! 1. **Storage Trait**: Abstract interface for all backends
//! 2. **Quad Indexes**: SPOC, POCS, OCSP, CSPO for optimal query patterns
//! 3. **Zero-Copy**: Minimize allocations in hot paths
//! 4. **ACID Transactions**: Full transaction support for persistent backends
//!
//! # Example
//!
//! ```rust,ignore
//! use storage::{QuadStore, StorageBackend, InMemoryBackend};
//! use rdf_model::{Node, Triple, Quad, Dictionary};
//!
//! // Create in-memory quad store
//! let dict = Dictionary::new();
//! let mut store = QuadStore::new(InMemoryBackend::new());
//!
//! // Insert quad
//! let subject = Node::iri(dict.intern("http://example.org/s"));
//! let predicate = Node::iri(dict.intern("http://example.org/p"));
//! let object = Node::literal_str(dict.intern("value"));
//! let quad = Quad::new(subject, predicate, object, None);
//!
//! store.insert(quad).unwrap();
//!
//! // Query with pattern
//! let results: Vec<_> = store.find(&QuadPattern::default()).collect();
//! ```

#![cfg_attr(feature = "simd", feature(portable_simd))]
#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

mod backend;
mod indexes;
mod inmemory;
mod observability;
mod pattern;
mod quad_store;
mod transaction;

// SIMD optimizations (optional, requires nightly)
#[cfg(feature = "simd")]
pub mod simd_encode;

#[cfg(feature = "simd")]
pub mod simd;

// Optional persistent storage backends
#[cfg(feature = "rocksdb-backend")]
mod rocksdb_backend;

#[cfg(feature = "lmdb-backend")]
mod lmdb_backend;

pub use backend::{StorageBackend, StorageError, StorageResult, StorageStats};
pub use indexes::{Index, IndexType, QuadIndex};
pub use inmemory::InMemoryBackend;
pub use observability::{
    track_operation, record_error, track_batch, record_throughput,
    OperationType, HealthStatus, PerformanceMetrics,
};
pub use pattern::{NodePattern, QuadPattern};
pub use quad_store::QuadStore;
pub use transaction::Transaction;

// Export persistent backends when features enabled
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb_backend::RocksDbBackend;

#[cfg(feature = "lmdb-backend")]
pub use lmdb_backend::LmdbBackend;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_compiles() {
        // Basic smoke test
        let _backend = InMemoryBackend::new();
    }
}
