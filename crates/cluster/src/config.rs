//! Cluster configuration management.
//!
//! Provides configuration types for both coordinator and executor nodes,
//! with support for environment variable parsing and validation.

use crate::{ClusterError, ClusterResult, NodeId, NodeRole};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Default number of partitions (should be prime for better distribution)
pub const DEFAULT_PARTITION_COUNT: u16 = 9;

/// Default replication factor (1 = no replication)
pub const DEFAULT_REPLICATION_FACTOR: u8 = 1;

/// Default gRPC port
pub const DEFAULT_GRPC_PORT: u16 = 9090;

/// Default HTTP/SPARQL port
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Default heartbeat interval
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 5000;

/// Default heartbeat timeout (node considered dead after this)
pub const DEFAULT_HEARTBEAT_TIMEOUT_MS: u64 = 15000;

/// Configuration for a cluster node (coordinator or executor).
///
/// # Example
///
/// ```rust
/// use cluster::{ClusterConfig, NodeId, NodeRole};
///
/// // Create executor config
/// let config = ClusterConfig::new(NodeId(1), NodeRole::Executor)
///     .with_partition_count(9)
///     .with_coordinator_addr("coordinator:9090")
///     .with_partitions(vec![0, 1, 2]);
/// ```
///
/// Create from environment (requires NODE_ID env var):
/// ```rust,ignore
/// let config = ClusterConfig::from_env().unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Unique node identifier
    pub node_id: NodeId,

    /// Role of this node
    pub role: NodeRole,

    /// Total number of partitions in cluster
    pub partition_count: u16,

    /// Replication factor (copies of each partition)
    pub replication_factor: u8,

    /// Partitions owned by this node (for executors)
    pub partitions: Vec<u16>,

    /// Coordinator address(es) for discovery
    pub coordinator_addrs: Vec<String>,

    /// gRPC listen address
    pub grpc_addr: String,

    /// HTTP/SPARQL listen address
    pub http_addr: String,

    /// Data directory for RocksDB
    pub data_dir: String,

    /// Heartbeat interval
    pub heartbeat_interval: Duration,

    /// Heartbeat timeout
    pub heartbeat_timeout: Duration,

    /// HDRF lambda parameter (balance factor)
    pub hdrf_lambda: f64,

    /// Log level
    pub log_level: String,
}

impl ClusterConfig {
    /// Create a new configuration with required fields
    pub fn new(node_id: NodeId, role: NodeRole) -> Self {
        Self {
            node_id,
            role,
            partition_count: DEFAULT_PARTITION_COUNT,
            replication_factor: DEFAULT_REPLICATION_FACTOR,
            partitions: vec![],
            coordinator_addrs: vec![],
            grpc_addr: format!("0.0.0.0:{}", DEFAULT_GRPC_PORT),
            http_addr: format!("0.0.0.0:{}", DEFAULT_HTTP_PORT),
            data_dir: "/data".to_string(),
            heartbeat_interval: Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS),
            heartbeat_timeout: Duration::from_millis(DEFAULT_HEARTBEAT_TIMEOUT_MS),
            hdrf_lambda: 1.0,
            log_level: "info".to_string(),
        }
    }

    /// Create configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `NODE_ID`: Node identifier (required)
    /// - `NODE_ROLE`: "coordinator" or "executor" (default: executor)
    /// - `PARTITION_COUNT`: Total partitions (default: 9)
    /// - `REPLICATION_FACTOR`: Copies per partition (default: 1)
    /// - `PARTITIONS`: Comma-separated partition IDs for this node
    /// - `COORDINATOR_ADDR`: Coordinator address(es), comma-separated
    /// - `GRPC_ADDR`: gRPC listen address (default: 0.0.0.0:9090)
    /// - `HTTP_ADDR`: HTTP listen address (default: 0.0.0.0:8080)
    /// - `DATA_DIR`: RocksDB data directory (default: /data)
    /// - `HEARTBEAT_INTERVAL_MS`: Heartbeat interval (default: 5000)
    /// - `HEARTBEAT_TIMEOUT_MS`: Heartbeat timeout (default: 15000)
    /// - `HDRF_LAMBDA`: HDRF balance parameter (default: 1.0)
    /// - `RUST_LOG`: Log level (default: info)
    pub fn from_env() -> ClusterResult<Self> {
        let node_id = env::var("NODE_ID")
            .map_err(|_| ClusterError::config("NODE_ID environment variable required"))?
            .parse::<u64>()
            .map_err(|e| ClusterError::config(format!("Invalid NODE_ID: {}", e)))?;

        let role = match env::var("NODE_ROLE").as_deref() {
            Ok("coordinator") => NodeRole::Coordinator,
            Ok("executor") | Err(_) => NodeRole::Executor,
            Ok(other) => {
                return Err(ClusterError::config(format!(
                    "Invalid NODE_ROLE: {} (expected 'coordinator' or 'executor')",
                    other
                )))
            }
        };

        let mut config = Self::new(NodeId(node_id), role);

        // Parse partition count
        if let Ok(count) = env::var("PARTITION_COUNT") {
            config.partition_count = count
                .parse()
                .map_err(|e| ClusterError::config(format!("Invalid PARTITION_COUNT: {}", e)))?;
        }

        // Parse replication factor
        if let Ok(factor) = env::var("REPLICATION_FACTOR") {
            config.replication_factor = factor
                .parse()
                .map_err(|e| ClusterError::config(format!("Invalid REPLICATION_FACTOR: {}", e)))?;
        }

        // Parse partitions (comma-separated)
        if let Ok(partitions) = env::var("PARTITIONS") {
            config.partitions = partitions
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| {
                    s.trim()
                        .parse()
                        .map_err(|e| ClusterError::config(format!("Invalid partition ID: {}", e)))
                })
                .collect::<ClusterResult<Vec<_>>>()?;
        }

        // Parse coordinator addresses (comma-separated)
        if let Ok(addrs) = env::var("COORDINATOR_ADDR") {
            config.coordinator_addrs = addrs
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect();
        }

        // Parse addresses
        if let Ok(addr) = env::var("GRPC_ADDR") {
            config.grpc_addr = addr;
        }

        if let Ok(addr) = env::var("HTTP_ADDR") {
            config.http_addr = addr;
        }

        if let Ok(dir) = env::var("DATA_DIR") {
            config.data_dir = dir;
        }

        // Parse heartbeat settings
        if let Ok(interval) = env::var("HEARTBEAT_INTERVAL_MS") {
            let ms: u64 = interval
                .parse()
                .map_err(|e| ClusterError::config(format!("Invalid HEARTBEAT_INTERVAL_MS: {}", e)))?;
            config.heartbeat_interval = Duration::from_millis(ms);
        }

        if let Ok(timeout) = env::var("HEARTBEAT_TIMEOUT_MS") {
            let ms: u64 = timeout
                .parse()
                .map_err(|e| ClusterError::config(format!("Invalid HEARTBEAT_TIMEOUT_MS: {}", e)))?;
            config.heartbeat_timeout = Duration::from_millis(ms);
        }

        // Parse HDRF lambda
        if let Ok(lambda) = env::var("HDRF_LAMBDA") {
            config.hdrf_lambda = lambda
                .parse()
                .map_err(|e| ClusterError::config(format!("Invalid HDRF_LAMBDA: {}", e)))?;
        }

        if let Ok(level) = env::var("RUST_LOG") {
            config.log_level = level;
        }

        config.validate()?;
        Ok(config)
    }

    /// Builder method: set partition count
    pub fn with_partition_count(mut self, count: u16) -> Self {
        self.partition_count = count;
        self
    }

    /// Builder method: set replication factor
    pub fn with_replication_factor(mut self, factor: u8) -> Self {
        self.replication_factor = factor;
        self
    }

    /// Builder method: set owned partitions
    pub fn with_partitions(mut self, partitions: Vec<u16>) -> Self {
        self.partitions = partitions;
        self
    }

    /// Builder method: add coordinator address
    pub fn with_coordinator_addr(mut self, addr: impl Into<String>) -> Self {
        self.coordinator_addrs.push(addr.into());
        self
    }

    /// Builder method: set gRPC address
    pub fn with_grpc_addr(mut self, addr: impl Into<String>) -> Self {
        self.grpc_addr = addr.into();
        self
    }

    /// Builder method: set HTTP address
    pub fn with_http_addr(mut self, addr: impl Into<String>) -> Self {
        self.http_addr = addr.into();
        self
    }

    /// Builder method: set data directory
    pub fn with_data_dir(mut self, dir: impl Into<String>) -> Self {
        self.data_dir = dir.into();
        self
    }

    /// Builder method: set HDRF lambda
    pub fn with_hdrf_lambda(mut self, lambda: f64) -> Self {
        self.hdrf_lambda = lambda;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> ClusterResult<()> {
        // Partition count must be positive
        if self.partition_count == 0 {
            return Err(ClusterError::config("partition_count must be > 0"));
        }

        // Replication factor must be positive
        if self.replication_factor == 0 {
            return Err(ClusterError::config("replication_factor must be > 0"));
        }

        // Executors should have coordinator addresses
        if self.role.is_executor() && self.coordinator_addrs.is_empty() {
            return Err(ClusterError::config(
                "Executor requires at least one coordinator address",
            ));
        }

        // Validate partition IDs are in range
        for &p in &self.partitions {
            if p >= self.partition_count {
                return Err(ClusterError::config(format!(
                    "Partition {} is out of range (max: {})",
                    p,
                    self.partition_count - 1
                )));
            }
        }

        // HDRF lambda should be positive
        if self.hdrf_lambda <= 0.0 {
            return Err(ClusterError::config("hdrf_lambda must be > 0"));
        }

        Ok(())
    }

    /// Check if this node owns a specific partition
    pub fn owns_partition(&self, partition: u16) -> bool {
        self.partitions.contains(&partition)
    }

    /// Get the data directory for a specific partition
    pub fn partition_data_dir(&self, partition: u16) -> String {
        format!("{}/partition_{}", self.data_dir, partition)
    }
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self::new(NodeId(0), NodeRole::Coordinator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ClusterConfig::new(NodeId(1), NodeRole::Executor)
            .with_partition_count(9)
            .with_partitions(vec![0, 1, 2])
            .with_coordinator_addr("coordinator:9090")
            .with_data_dir("/data/executor-1");

        assert_eq!(config.node_id, NodeId(1));
        assert!(config.role.is_executor());
        assert_eq!(config.partition_count, 9);
        assert_eq!(config.partitions, vec![0, 1, 2]);
        assert_eq!(config.coordinator_addrs, vec!["coordinator:9090"]);
    }

    #[test]
    fn test_config_validation() {
        // Valid executor config
        let config = ClusterConfig::new(NodeId(1), NodeRole::Executor)
            .with_partition_count(9)
            .with_partitions(vec![0, 1, 2])
            .with_coordinator_addr("coordinator:9090");
        assert!(config.validate().is_ok());

        // Invalid: executor without coordinator
        let config = ClusterConfig::new(NodeId(1), NodeRole::Executor);
        assert!(config.validate().is_err());

        // Invalid: partition out of range
        let config = ClusterConfig::new(NodeId(1), NodeRole::Executor)
            .with_partition_count(9)
            .with_partitions(vec![10]) // Out of range
            .with_coordinator_addr("coordinator:9090");
        assert!(config.validate().is_err());

        // Invalid: zero partition count
        let config = ClusterConfig::new(NodeId(0), NodeRole::Coordinator)
            .with_partition_count(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_owns_partition() {
        let config = ClusterConfig::new(NodeId(1), NodeRole::Executor)
            .with_partitions(vec![0, 1, 2]);

        assert!(config.owns_partition(0));
        assert!(config.owns_partition(1));
        assert!(config.owns_partition(2));
        assert!(!config.owns_partition(3));
    }

    #[test]
    fn test_partition_data_dir() {
        let config = ClusterConfig::new(NodeId(1), NodeRole::Executor)
            .with_data_dir("/data/executor-1");

        assert_eq!(
            config.partition_data_dir(0),
            "/data/executor-1/partition_0"
        );
        assert_eq!(
            config.partition_data_dir(5),
            "/data/executor-1/partition_5"
        );
    }
}
