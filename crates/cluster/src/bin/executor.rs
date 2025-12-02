//! rust-kgdb Cluster Executor
//!
//! The executor node is responsible for:
//! - Storing partitioned RDF data in RocksDB
//! - Executing local SPARQL queries
//! - Returning results to the coordinator
//! - Handling inter-partition queries
//!
//! ## Usage
//!
//! ```bash
//! # Start executor with environment variables
//! NODE_ID=1 NODE_ROLE=executor PARTITIONS=0,1,2 COORDINATOR_ADDR=coordinator:9090 executor
//!
//! # Or use command-line arguments
//! executor --node-id 1 --partitions 0,1,2 --coordinator coordinator:9090
//! ```

use cluster::{ClusterConfig, ClusterResult, NodeId, NodeRole, PartitionId, PartitionMap};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// Executor service state
struct Executor {
    config: ClusterConfig,
    // TODO: Add RocksDB stores per partition
    // TODO: Add SPARQL executor
}

impl Executor {
    /// Create a new executor from configuration
    async fn new(config: ClusterConfig) -> ClusterResult<Self> {
        // TODO: Initialize RocksDB for each owned partition
        // TODO: Load partition data if exists
        // TODO: Connect to coordinator

        Ok(Self { config })
    }

    /// Start the executor server
    async fn serve(self) -> ClusterResult<()> {
        info!(
            "Executor {} starting on gRPC {}",
            self.config.node_id, self.config.grpc_addr
        );
        info!(
            "Assigned partitions: {:?}",
            self.config.partitions
        );
        info!(
            "Coordinator address: {:?}",
            self.config.coordinator_addrs
        );

        // TODO: Start gRPC server for query handling
        // TODO: Register with coordinator
        // TODO: Start heartbeat loop
        // TODO: Initialize partition stores

        info!("Executor ready - connected to cluster");

        // Keep running until shutdown signal
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");

        info!("Executor shutting down gracefully");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("rust-kgdb Executor v{}", cluster::VERSION);

    // Load configuration from environment
    let config = match ClusterConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            eprintln!("Required environment variables:");
            eprintln!("  NODE_ID - Unique node identifier (e.g., 1)");
            eprintln!("  PARTITIONS - Comma-separated partition IDs (e.g., 0,1,2)");
            eprintln!("  COORDINATOR_ADDR - Coordinator address (e.g., coordinator:9090)");
            std::process::exit(1);
        }
    };

    // Validate configuration
    if !config.role.is_executor() {
        eprintln!("Error: NODE_ROLE must be 'executor'");
        std::process::exit(1);
    }

    if config.partitions.is_empty() {
        eprintln!("Error: PARTITIONS must not be empty");
        std::process::exit(1);
    }

    // Create and start executor
    let executor = Executor::new(config).await?;
    executor.serve().await?;

    Ok(())
}
