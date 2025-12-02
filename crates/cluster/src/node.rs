//! Node identification and role management for distributed cluster.
//!
//! Provides types for identifying and managing cluster nodes:
//! - `NodeId`: Unique identifier for each node
//! - `NodeRole`: Whether node is Coordinator or Executor
//! - `NodeState`: Current health/activity state

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a node in the cluster.
///
/// Node IDs are assigned at startup and remain stable throughout
/// the node's lifetime. They are used for:
/// - Routing messages between nodes
/// - Tracking partition ownership
/// - Health monitoring
///
/// # Example
///
/// ```rust
/// use cluster::NodeId;
///
/// let node = NodeId(1);
/// assert_eq!(node.as_u64(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create a new NodeId
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw u64 value
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Special ID for the coordinator (always 0)
    pub const COORDINATOR: NodeId = NodeId(0);
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

impl From<u64> for NodeId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<NodeId> for u64 {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

/// Role of a node in the cluster.
///
/// - **Coordinator**: Routes queries, manages partition map, handles cluster membership
/// - **Executor**: Stores partitioned data, executes local queries
///
/// # Architecture
///
/// ```text
/// Coordinator (1 per cluster)
///     │
///     ├── Receives all client queries
///     ├── Routes to appropriate executors
///     ├── Aggregates results
///     └── Manages partition assignments
///
/// Executor (N per cluster)
///     │
///     ├── Owns subset of partitions
///     ├── Executes local SPARQL queries
///     ├── Returns results to coordinator
///     └── Handles local RocksDB storage
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeRole {
    /// Central query router and cluster manager
    Coordinator,
    /// Partition-local query executor with RocksDB storage
    Executor,
}

impl NodeRole {
    /// Check if this is the coordinator role
    pub fn is_coordinator(&self) -> bool {
        matches!(self, NodeRole::Coordinator)
    }

    /// Check if this is an executor role
    pub fn is_executor(&self) -> bool {
        matches!(self, NodeRole::Executor)
    }
}

impl fmt::Display for NodeRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeRole::Coordinator => write!(f, "Coordinator"),
            NodeRole::Executor => write!(f, "Executor"),
        }
    }
}

impl Default for NodeRole {
    fn default() -> Self {
        NodeRole::Executor
    }
}

/// Current state of a node in the cluster.
///
/// State transitions:
/// ```text
/// Starting → Active → Draining → Stopped
///              ↓
///           Failed
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeState {
    /// Node is starting up, not yet ready for queries
    Starting,
    /// Node is active and processing queries
    Active,
    /// Node is draining (graceful shutdown), finishing existing queries
    Draining,
    /// Node has stopped cleanly
    Stopped,
    /// Node has failed unexpectedly
    Failed,
}

impl NodeState {
    /// Check if node can accept new queries
    pub fn can_accept_queries(&self) -> bool {
        matches!(self, NodeState::Active)
    }

    /// Check if node is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, NodeState::Active | NodeState::Draining)
    }

    /// Check if node is terminated
    pub fn is_terminated(&self) -> bool {
        matches!(self, NodeState::Stopped | NodeState::Failed)
    }
}

impl fmt::Display for NodeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeState::Starting => write!(f, "Starting"),
            NodeState::Active => write!(f, "Active"),
            NodeState::Draining => write!(f, "Draining"),
            NodeState::Stopped => write!(f, "Stopped"),
            NodeState::Failed => write!(f, "Failed"),
        }
    }
}

impl Default for NodeState {
    fn default() -> Self {
        NodeState::Starting
    }
}

/// Metadata about a node in the cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique node identifier
    pub id: NodeId,
    /// Node's role in cluster
    pub role: NodeRole,
    /// Current state
    pub state: NodeState,
    /// gRPC address for inter-node communication
    pub grpc_addr: String,
    /// Partitions owned by this node (only for Executors)
    pub partitions: Vec<u16>,
    /// Last heartbeat timestamp (Unix millis)
    pub last_heartbeat: u64,
}

impl NodeInfo {
    /// Create a new NodeInfo for an executor
    pub fn new_executor(id: NodeId, grpc_addr: String, partitions: Vec<u16>) -> Self {
        Self {
            id,
            role: NodeRole::Executor,
            state: NodeState::Starting,
            grpc_addr,
            partitions,
            last_heartbeat: 0,
        }
    }

    /// Create a new NodeInfo for the coordinator
    pub fn new_coordinator(grpc_addr: String) -> Self {
        Self {
            id: NodeId::COORDINATOR,
            role: NodeRole::Coordinator,
            state: NodeState::Starting,
            grpc_addr,
            partitions: vec![],
            last_heartbeat: 0,
        }
    }

    /// Update the heartbeat timestamp
    pub fn update_heartbeat(&mut self, timestamp: u64) {
        self.last_heartbeat = timestamp;
        if self.state == NodeState::Starting {
            self.state = NodeState::Active;
        }
    }

    /// Check if heartbeat is stale (older than threshold_ms)
    pub fn is_heartbeat_stale(&self, current_time: u64, threshold_ms: u64) -> bool {
        self.last_heartbeat > 0 && (current_time - self.last_heartbeat) > threshold_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_creation() {
        let id = NodeId::new(42);
        assert_eq!(id.as_u64(), 42);
        assert_eq!(id.to_string(), "Node(42)");
    }

    #[test]
    fn test_node_id_coordinator() {
        assert_eq!(NodeId::COORDINATOR, NodeId(0));
    }

    #[test]
    fn test_node_role() {
        assert!(NodeRole::Coordinator.is_coordinator());
        assert!(!NodeRole::Coordinator.is_executor());
        assert!(NodeRole::Executor.is_executor());
        assert!(!NodeRole::Executor.is_coordinator());
    }

    #[test]
    fn test_node_state_transitions() {
        assert!(NodeState::Active.can_accept_queries());
        assert!(!NodeState::Draining.can_accept_queries());
        assert!(!NodeState::Failed.can_accept_queries());

        assert!(NodeState::Active.is_healthy());
        assert!(NodeState::Draining.is_healthy());
        assert!(!NodeState::Failed.is_healthy());

        assert!(NodeState::Stopped.is_terminated());
        assert!(NodeState::Failed.is_terminated());
        assert!(!NodeState::Active.is_terminated());
    }

    #[test]
    fn test_node_info_executor() {
        let info = NodeInfo::new_executor(
            NodeId(1),
            "executor-1:9090".to_string(),
            vec![0, 1, 2],
        );
        assert_eq!(info.id, NodeId(1));
        assert!(info.role.is_executor());
        assert_eq!(info.partitions, vec![0, 1, 2]);
    }

    #[test]
    fn test_node_info_heartbeat() {
        let mut info = NodeInfo::new_executor(
            NodeId(1),
            "executor-1:9090".to_string(),
            vec![0, 1, 2],
        );

        assert_eq!(info.state, NodeState::Starting);
        info.update_heartbeat(1000);
        assert_eq!(info.state, NodeState::Active);
        assert_eq!(info.last_heartbeat, 1000);

        assert!(!info.is_heartbeat_stale(1500, 1000));
        assert!(info.is_heartbeat_stale(2500, 1000));
    }
}
