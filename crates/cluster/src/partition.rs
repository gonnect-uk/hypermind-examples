//! Partition management for distributed cluster.
//!
//! Provides types for managing partition assignments:
//! - `PartitionId`: Unique identifier for a partition
//! - `PartitionState`: Current state of a partition
//! - `PartitionMap`: Global view of partition-to-node assignments

use crate::{ClusterError, ClusterResult, NodeId};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

/// Unique identifier for a partition.
///
/// Partitions are numbered 0 to N-1 where N is the total partition count.
/// Each partition is assigned to one or more nodes depending on replication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartitionId(pub u16);

impl PartitionId {
    /// Create a new partition ID
    pub fn new(id: u16) -> Self {
        Self(id)
    }

    /// Get the raw u16 value
    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for PartitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "P{}", self.0)
    }
}

impl From<u16> for PartitionId {
    fn from(id: u16) -> Self {
        Self(id)
    }
}

impl From<PartitionId> for u16 {
    fn from(id: PartitionId) -> Self {
        id.0
    }
}

/// State of a partition.
///
/// ```text
/// Initializing → Active → Rebalancing → Active
///                  ↓
///               Offline
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartitionState {
    /// Partition is being initialized (loading data)
    Initializing,
    /// Partition is active and serving queries
    Active,
    /// Partition is being rebalanced to another node
    Rebalancing,
    /// Partition is offline (node failed)
    Offline,
}

impl PartitionState {
    /// Check if partition can serve queries
    pub fn can_serve(&self) -> bool {
        matches!(self, PartitionState::Active | PartitionState::Rebalancing)
    }

    /// Check if partition is healthy
    pub fn is_healthy(&self) -> bool {
        !matches!(self, PartitionState::Offline)
    }
}

impl fmt::Display for PartitionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartitionState::Initializing => write!(f, "Initializing"),
            PartitionState::Active => write!(f, "Active"),
            PartitionState::Rebalancing => write!(f, "Rebalancing"),
            PartitionState::Offline => write!(f, "Offline"),
        }
    }
}

impl Default for PartitionState {
    fn default() -> Self {
        PartitionState::Initializing
    }
}

/// Information about a single partition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    /// Partition identifier
    pub id: PartitionId,
    /// Current state
    pub state: PartitionState,
    /// Primary owner node
    pub owner: NodeId,
    /// Replica nodes (for replication factor > 1)
    pub replicas: Vec<NodeId>,
    /// Number of triples in this partition
    pub triple_count: u64,
    /// Size in bytes
    pub size_bytes: u64,
}

impl PartitionInfo {
    /// Create new partition info
    pub fn new(id: PartitionId, owner: NodeId) -> Self {
        Self {
            id,
            state: PartitionState::Initializing,
            owner,
            replicas: vec![],
            triple_count: 0,
            size_bytes: 0,
        }
    }

    /// Get all nodes that can serve this partition
    pub fn serving_nodes(&self) -> Vec<NodeId> {
        let mut nodes = vec![self.owner];
        nodes.extend(&self.replicas);
        nodes
    }
}

/// Global partition map maintaining partition-to-node assignments.
///
/// Thread-safe using DashMap for concurrent access from multiple
/// query handlers.
///
/// # Example
///
/// ```rust
/// use cluster::{PartitionMap, PartitionId, NodeId};
///
/// let map = PartitionMap::new(9); // 9 partitions
///
/// // Assign partitions to nodes
/// map.assign(PartitionId(0), NodeId(1));
/// map.assign(PartitionId(1), NodeId(1));
/// map.assign(PartitionId(2), NodeId(1));
/// map.assign(PartitionId(3), NodeId(2));
///
/// // Query partition owner
/// let owner = map.get_owner(PartitionId(0));
/// ```
#[derive(Debug)]
pub struct PartitionMap {
    /// Total number of partitions
    partition_count: u16,

    /// Partition ID -> PartitionInfo
    partitions: DashMap<PartitionId, PartitionInfo>,

    /// Node ID -> Set of owned partition IDs
    node_partitions: DashMap<NodeId, HashSet<PartitionId>>,

    /// Version number for cache invalidation
    version: RwLock<u64>,
}

impl PartitionMap {
    /// Create a new partition map with specified partition count
    pub fn new(partition_count: u16) -> Self {
        Self {
            partition_count,
            partitions: DashMap::new(),
            node_partitions: DashMap::new(),
            version: RwLock::new(0),
        }
    }

    /// Get total partition count
    pub fn partition_count(&self) -> u16 {
        self.partition_count
    }

    /// Get current version (for cache invalidation)
    pub fn version(&self) -> u64 {
        *self.version.read()
    }

    /// Assign a partition to a node (as primary owner)
    pub fn assign(&self, partition: PartitionId, node: NodeId) {
        // Remove from old owner if reassigning
        if let Some(old_info) = self.partitions.get(&partition) {
            let old_owner = old_info.owner;
            if old_owner != node {
                if let Some(mut node_parts) = self.node_partitions.get_mut(&old_owner) {
                    node_parts.remove(&partition);
                }
            }
        }

        // Create or update partition info
        self.partitions
            .entry(partition)
            .and_modify(|info| {
                info.owner = node;
                info.state = PartitionState::Active;
            })
            .or_insert_with(|| {
                let mut info = PartitionInfo::new(partition, node);
                info.state = PartitionState::Active;
                info
            });

        // Update node -> partitions mapping
        self.node_partitions
            .entry(node)
            .or_insert_with(HashSet::new)
            .insert(partition);

        // Increment version
        *self.version.write() += 1;
    }

    /// Assign multiple partitions to a node
    pub fn assign_many(&self, partitions: &[PartitionId], node: NodeId) {
        for &partition in partitions {
            self.assign(partition, node);
        }
    }

    /// Get the owner node for a partition
    pub fn get_owner(&self, partition: PartitionId) -> Option<NodeId> {
        self.partitions.get(&partition).map(|info| info.owner)
    }

    /// Get partition info
    pub fn get_info(&self, partition: PartitionId) -> Option<PartitionInfo> {
        self.partitions.get(&partition).map(|r| r.clone())
    }

    /// Get all partitions owned by a node
    pub fn get_node_partitions(&self, node: NodeId) -> Vec<PartitionId> {
        self.node_partitions
            .get(&node)
            .map(|parts| parts.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get all nodes that own at least one partition
    pub fn get_active_nodes(&self) -> Vec<NodeId> {
        self.node_partitions
            .iter()
            .filter(|entry| !entry.value().is_empty())
            .map(|entry| *entry.key())
            .collect()
    }

    /// Update partition state
    pub fn set_partition_state(&self, partition: PartitionId, state: PartitionState) -> ClusterResult<()> {
        if let Some(mut info) = self.partitions.get_mut(&partition) {
            info.state = state;
            *self.version.write() += 1;
            Ok(())
        } else {
            Err(ClusterError::partition_not_found(partition.0))
        }
    }

    /// Update partition statistics
    pub fn update_stats(&self, partition: PartitionId, triple_count: u64, size_bytes: u64) -> ClusterResult<()> {
        if let Some(mut info) = self.partitions.get_mut(&partition) {
            info.triple_count = triple_count;
            info.size_bytes = size_bytes;
            Ok(())
        } else {
            Err(ClusterError::partition_not_found(partition.0))
        }
    }

    /// Get total triple count across all partitions
    pub fn total_triple_count(&self) -> u64 {
        self.partitions
            .iter()
            .map(|entry| entry.triple_count)
            .sum()
    }

    /// Get total size across all partitions
    pub fn total_size_bytes(&self) -> u64 {
        self.partitions
            .iter()
            .map(|entry| entry.size_bytes)
            .sum()
    }

    /// Check if all partitions are assigned
    pub fn is_fully_assigned(&self) -> bool {
        (0..self.partition_count).all(|p| self.partitions.contains_key(&PartitionId(p)))
    }

    /// Get unassigned partitions
    pub fn get_unassigned(&self) -> Vec<PartitionId> {
        (0..self.partition_count)
            .map(PartitionId)
            .filter(|p| !self.partitions.contains_key(p))
            .collect()
    }

    /// Get partition load per node (for rebalancing)
    pub fn get_load_distribution(&self) -> HashMap<NodeId, (usize, u64)> {
        let mut distribution = HashMap::new();

        for entry in self.node_partitions.iter() {
            let node = *entry.key();
            let partitions = entry.value();

            let triple_count: u64 = partitions
                .iter()
                .filter_map(|p| self.partitions.get(p))
                .map(|info| info.triple_count)
                .sum();

            distribution.insert(node, (partitions.len(), triple_count));
        }

        distribution
    }

    /// Export partition map to JSON for persistence
    pub fn to_json(&self) -> ClusterResult<String> {
        let snapshot: Vec<PartitionInfo> = self.partitions
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        serde_json::to_string_pretty(&snapshot)
            .map_err(|e| ClusterError::serialization(e.to_string()))
    }

    /// Import partition map from JSON
    pub fn from_json(json: &str, partition_count: u16) -> ClusterResult<Self> {
        let infos: Vec<PartitionInfo> = serde_json::from_str(json)
            .map_err(|e| ClusterError::serialization(e.to_string()))?;

        let map = Self::new(partition_count);

        for info in infos {
            map.partitions.insert(info.id, info.clone());
            map.node_partitions
                .entry(info.owner)
                .or_insert_with(HashSet::new)
                .insert(info.id);
        }

        Ok(map)
    }
}

impl Clone for PartitionMap {
    fn clone(&self) -> Self {
        let new_map = Self::new(self.partition_count);

        for entry in self.partitions.iter() {
            new_map.partitions.insert(*entry.key(), entry.value().clone());
        }

        for entry in self.node_partitions.iter() {
            new_map.node_partitions.insert(*entry.key(), entry.value().clone());
        }

        *new_map.version.write() = *self.version.read();

        new_map
    }
}

/// Thread-safe shared partition map
pub type SharedPartitionMap = Arc<PartitionMap>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_id() {
        let p = PartitionId::new(5);
        assert_eq!(p.as_u16(), 5);
        assert_eq!(p.to_string(), "P5");
    }

    #[test]
    fn test_partition_state() {
        assert!(PartitionState::Active.can_serve());
        assert!(PartitionState::Rebalancing.can_serve());
        assert!(!PartitionState::Offline.can_serve());
        assert!(!PartitionState::Initializing.can_serve());
    }

    #[test]
    fn test_partition_map_basic() {
        let map = PartitionMap::new(9);

        // Assign partitions to nodes
        map.assign(PartitionId(0), NodeId(1));
        map.assign(PartitionId(1), NodeId(1));
        map.assign(PartitionId(2), NodeId(1));
        map.assign(PartitionId(3), NodeId(2));
        map.assign(PartitionId(4), NodeId(2));
        map.assign(PartitionId(5), NodeId(2));
        map.assign(PartitionId(6), NodeId(3));
        map.assign(PartitionId(7), NodeId(3));
        map.assign(PartitionId(8), NodeId(3));

        // Check assignments
        assert_eq!(map.get_owner(PartitionId(0)), Some(NodeId(1)));
        assert_eq!(map.get_owner(PartitionId(3)), Some(NodeId(2)));
        assert_eq!(map.get_owner(PartitionId(6)), Some(NodeId(3)));

        // Check node partitions
        let node1_parts = map.get_node_partitions(NodeId(1));
        assert_eq!(node1_parts.len(), 3);
        assert!(node1_parts.contains(&PartitionId(0)));

        // Check fully assigned
        assert!(map.is_fully_assigned());
        assert!(map.get_unassigned().is_empty());
    }

    #[test]
    fn test_partition_map_reassign() {
        let map = PartitionMap::new(3);

        map.assign(PartitionId(0), NodeId(1));
        assert_eq!(map.get_owner(PartitionId(0)), Some(NodeId(1)));

        // Reassign to different node
        map.assign(PartitionId(0), NodeId(2));
        assert_eq!(map.get_owner(PartitionId(0)), Some(NodeId(2)));

        // Check old owner no longer has partition
        assert!(!map.get_node_partitions(NodeId(1)).contains(&PartitionId(0)));
        assert!(map.get_node_partitions(NodeId(2)).contains(&PartitionId(0)));
    }

    #[test]
    fn test_partition_map_stats() {
        let map = PartitionMap::new(3);

        map.assign(PartitionId(0), NodeId(1));
        map.assign(PartitionId(1), NodeId(1));
        map.assign(PartitionId(2), NodeId(2));

        map.update_stats(PartitionId(0), 1000, 10000).unwrap();
        map.update_stats(PartitionId(1), 2000, 20000).unwrap();
        map.update_stats(PartitionId(2), 3000, 30000).unwrap();

        assert_eq!(map.total_triple_count(), 6000);
        assert_eq!(map.total_size_bytes(), 60000);

        let distribution = map.get_load_distribution();
        assert_eq!(distribution.get(&NodeId(1)), Some(&(2, 3000)));
        assert_eq!(distribution.get(&NodeId(2)), Some(&(1, 3000)));
    }

    #[test]
    fn test_partition_map_json() {
        let map = PartitionMap::new(3);

        map.assign(PartitionId(0), NodeId(1));
        map.assign(PartitionId(1), NodeId(2));
        map.update_stats(PartitionId(0), 100, 1000).unwrap();

        let json = map.to_json().unwrap();
        let restored = PartitionMap::from_json(&json, 3).unwrap();

        assert_eq!(restored.get_owner(PartitionId(0)), Some(NodeId(1)));
        assert_eq!(restored.get_owner(PartitionId(1)), Some(NodeId(2)));

        let info = restored.get_info(PartitionId(0)).unwrap();
        assert_eq!(info.triple_count, 100);
    }

    #[test]
    fn test_version_increment() {
        let map = PartitionMap::new(3);

        let v1 = map.version();
        map.assign(PartitionId(0), NodeId(1));
        let v2 = map.version();
        map.assign(PartitionId(1), NodeId(2));
        let v3 = map.version();

        assert!(v2 > v1);
        assert!(v3 > v2);
    }
}
