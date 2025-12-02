//! # rust-kgdb Cluster
//!
//! Distributed clustering support for rust-kgdb RDF database.
//!
//! This crate provides:
//! - **HDRF Partitioner**: High-Degree Replicated First edge partitioning
//! - **Consistent Hashing**: Ketama-style hashing with virtual nodes
//! - **Partition Management**: Assignment and rebalancing
//! - **Node Management**: Coordinator and executor roles
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    Coordinator                       │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
//! │  │Query Router │  │ Cost Model  │  │Partition Map│ │
//! │  └─────────────┘  └─────────────┘  └─────────────┘ │
//! └─────────────────────────────────────────────────────┘
//!                           │
//!            ┌──────────────┼──────────────┐
//!            ▼              ▼              ▼
//! ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
//! │  Executor 1  │ │  Executor 2  │ │  Executor 3  │
//! │ Partitions   │ │ Partitions   │ │ Partitions   │
//! │   0,1,2      │ │   3,4,5      │ │   6,7,8      │
//! │  [RocksDB]   │ │  [RocksDB]   │ │  [RocksDB]   │
//! └──────────────┘ └──────────────┘ └──────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use cluster::{ClusterConfig, HdrfPartitioner, NodeRole};
//!
//! // Create configuration
//! let config = ClusterConfig::new(NodeId(1), NodeRole::Executor)
//!     .with_partition_count(9)
//!     .with_coordinator_addr("coordinator:9090");
//!
//! // Create HDRF partitioner
//! let partitioner = HdrfPartitioner::new(9, 1.0);
//!
//! // Assign triple to partition
//! let partition = partitioner.assign_triple(&triple);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod consistent_hash;
pub mod error;
pub mod hdrf;
pub mod node;
pub mod partition;

// Re-exports for convenience
pub use config::ClusterConfig;
pub use consistent_hash::ConsistentHash;
pub use error::{ClusterError, ClusterResult};
pub use hdrf::HdrfPartitioner;
pub use node::{NodeId, NodeRole, NodeState};
pub use partition::{PartitionId, PartitionMap, PartitionState};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.2.0");
    }
}
