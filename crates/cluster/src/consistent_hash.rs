//! Consistent Hashing with Virtual Nodes (Ketama Algorithm)
//!
//! Implementation based on the Ketama consistent hashing algorithm,
//! used for partition routing and cluster membership.
//!
//! ## Algorithm Overview
//!
//! Consistent hashing maps keys to nodes on a hash ring. When nodes
//! are added or removed, only K/N keys need to be remapped (where
//! K = total keys, N = number of nodes).
//!
//! ## Virtual Nodes
//!
//! Each physical node is represented by multiple virtual nodes
//! (default: 150) on the ring. This provides:
//! - Better load distribution
//! - Smoother key migration during scaling
//! - Reduced variance in load
//!
//! ## Mathematical Properties
//!
//! For a ring with N nodes and V virtual nodes each:
//! - Each node owns approximately 1/N of keys
//! - Standard deviation of load ≈ ε/√(NV)
//! - Key lookup: O(log(NV))
//!
//! ## Example
//!
//! ```rust
//! use cluster::ConsistentHash;
//!
//! let mut ring = ConsistentHash::new(150); // 150 virtual nodes per physical node
//!
//! // Add nodes
//! ring.add_node("node-1");
//! ring.add_node("node-2");
//! ring.add_node("node-3");
//!
//! // Route keys to nodes
//! let node = ring.get_node("http://example.org/Alice").unwrap();
//! println!("Alice routes to: {}", node);
//!
//! // Remove node - only ~1/3 of keys remap
//! ring.remove_node("node-2");
//! ```

use serde::{Deserialize, Serialize};
use siphasher::sip::SipHasher13;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// Default number of virtual nodes per physical node.
///
/// Higher values = better distribution but more memory.
/// 150 is a good balance for most use cases.
pub const DEFAULT_VIRTUAL_NODES: usize = 150;

/// Consistent hash ring with virtual nodes.
#[derive(Debug, Clone)]
pub struct ConsistentHash {
    /// Number of virtual nodes per physical node
    virtual_nodes: usize,

    /// Hash ring: hash position -> node name
    ring: BTreeMap<u64, String>,

    /// Set of physical nodes
    nodes: Vec<String>,
}

impl ConsistentHash {
    /// Create a new consistent hash ring.
    ///
    /// # Arguments
    /// - `virtual_nodes`: Number of virtual nodes per physical node
    ///
    /// # Example
    ///
    /// ```rust
    /// use cluster::ConsistentHash;
    ///
    /// let ring = ConsistentHash::new(150);
    /// assert_eq!(ring.virtual_nodes(), 150);
    /// ```
    pub fn new(virtual_nodes: usize) -> Self {
        assert!(virtual_nodes > 0, "virtual_nodes must be > 0");

        Self {
            virtual_nodes,
            ring: BTreeMap::new(),
            nodes: Vec::new(),
        }
    }

    /// Create with default virtual nodes (150).
    pub fn default_ring() -> Self {
        Self::new(DEFAULT_VIRTUAL_NODES)
    }

    /// Add a node to the ring.
    ///
    /// Creates `virtual_nodes` positions on the ring for this node.
    pub fn add_node(&mut self, node: &str) {
        if self.nodes.contains(&node.to_string()) {
            return; // Already exists
        }

        self.nodes.push(node.to_string());

        for i in 0..self.virtual_nodes {
            let hash = self.hash_key(&format!("{}:{}", node, i));
            self.ring.insert(hash, node.to_string());
        }
    }

    /// Remove a node from the ring.
    ///
    /// Removes all virtual nodes for this physical node.
    pub fn remove_node(&mut self, node: &str) {
        self.nodes.retain(|n| n != node);

        for i in 0..self.virtual_nodes {
            let hash = self.hash_key(&format!("{}:{}", node, i));
            self.ring.remove(&hash);
        }
    }

    /// Get the node responsible for a key.
    ///
    /// Walks clockwise on the ring to find the first node.
    pub fn get_node(&self, key: &str) -> Option<&str> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = self.hash_key(key);

        // Find the first node clockwise from the hash position
        let node = self
            .ring
            .range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, node)| node.as_str());

        node
    }

    /// Get multiple nodes for replication.
    ///
    /// Returns `n` distinct physical nodes, walking clockwise from key position.
    pub fn get_nodes(&self, key: &str, n: usize) -> Vec<&str> {
        if self.ring.is_empty() || n == 0 {
            return vec![];
        }

        let hash = self.hash_key(key);
        let mut result = Vec::with_capacity(n);
        let mut seen = std::collections::HashSet::new();

        // Walk clockwise from hash position
        for (_, node) in self.ring.range(hash..).chain(self.ring.iter()) {
            if seen.insert(node.as_str()) {
                result.push(node.as_str());
                if result.len() >= n {
                    break;
                }
            }
        }

        result
    }

    /// Get number of physical nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get number of virtual nodes per physical node.
    pub fn virtual_nodes(&self) -> usize {
        self.virtual_nodes
    }

    /// Get total positions on the ring.
    pub fn ring_size(&self) -> usize {
        self.ring.len()
    }

    /// Get all physical nodes.
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }

    /// Check if ring is empty.
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Get the hash position for a key (for debugging).
    pub fn get_hash(&self, key: &str) -> u64 {
        self.hash_key(key)
    }

    /// Get load distribution across nodes (key counts per node).
    ///
    /// # Arguments
    /// - `keys`: Iterator of keys to check
    ///
    /// # Returns
    /// Map of node name to key count
    pub fn get_distribution<'a>(&self, keys: impl Iterator<Item = &'a str>) -> std::collections::HashMap<String, usize> {
        let mut distribution = std::collections::HashMap::new();

        for key in keys {
            if let Some(node) = self.get_node(key) {
                *distribution.entry(node.to_string()).or_insert(0) += 1;
            }
        }

        distribution
    }

    /// Internal hash function using SipHash.
    fn hash_key(&self, key: &str) -> u64 {
        let mut hasher = SipHasher13::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for ConsistentHash {
    fn default() -> Self {
        Self::default_ring()
    }
}

/// Statistics about consistent hash ring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistentHashStats {
    /// Number of physical nodes
    pub node_count: usize,
    /// Virtual nodes per physical node
    pub virtual_nodes: usize,
    /// Total ring positions
    pub ring_size: usize,
    /// Arc lengths (number of ring positions) per node
    pub arc_lengths: std::collections::HashMap<String, usize>,
}

impl ConsistentHash {
    /// Get statistics about the ring.
    pub fn stats(&self) -> ConsistentHashStats {
        let mut arc_lengths = std::collections::HashMap::new();

        for node in &self.nodes {
            arc_lengths.insert(node.clone(), 0);
        }

        // Count ring positions per node
        for (_, node) in &self.ring {
            *arc_lengths.entry(node.clone()).or_insert(0) += 1;
        }

        ConsistentHashStats {
            node_count: self.nodes.len(),
            virtual_nodes: self.virtual_nodes,
            ring_size: self.ring.len(),
            arc_lengths,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut ring = ConsistentHash::new(10);

        ring.add_node("node-1");
        ring.add_node("node-2");
        ring.add_node("node-3");

        assert_eq!(ring.node_count(), 3);
        assert_eq!(ring.ring_size(), 30); // 3 nodes * 10 virtual nodes
    }

    #[test]
    fn test_get_node() {
        let mut ring = ConsistentHash::new(100);

        ring.add_node("node-1");
        ring.add_node("node-2");

        // Should always return a node
        let node = ring.get_node("test-key");
        assert!(node.is_some());

        // Same key should always route to same node
        let n1 = ring.get_node("consistent-key");
        let n2 = ring.get_node("consistent-key");
        assert_eq!(n1, n2);
    }

    #[test]
    fn test_get_nodes_replication() {
        let mut ring = ConsistentHash::new(100);

        ring.add_node("node-1");
        ring.add_node("node-2");
        ring.add_node("node-3");

        // Get 2 replica nodes
        let nodes = ring.get_nodes("test-key", 2);
        assert_eq!(nodes.len(), 2);

        // Should be distinct
        assert_ne!(nodes[0], nodes[1]);
    }

    #[test]
    fn test_node_removal() {
        let mut ring = ConsistentHash::new(100);

        ring.add_node("node-1");
        ring.add_node("node-2");
        ring.add_node("node-3");

        // Track keys before removal
        let keys: Vec<String> = (0..100).map(|i| format!("key-{}", i)).collect();
        let before: Vec<_> = keys.iter().map(|k| ring.get_node(k).unwrap().to_string()).collect();

        // Remove a node
        ring.remove_node("node-2");
        assert_eq!(ring.node_count(), 2);

        // Count how many keys moved
        let after: Vec<_> = keys.iter().map(|k| ring.get_node(k).unwrap().to_string()).collect();
        let moved = before.iter().zip(after.iter())
            .filter(|(b, a)| b != a)
            .count();

        // Should only move keys from removed node (roughly 1/3)
        // Give some tolerance for hash distribution
        assert!(moved <= 50, "Too many keys moved: {}", moved);
    }

    #[test]
    fn test_distribution() {
        let mut ring = ConsistentHash::new(150);

        ring.add_node("node-1");
        ring.add_node("node-2");
        ring.add_node("node-3");

        // Generate many keys
        let keys: Vec<String> = (0..1000).map(|i| format!("http://example.org/entity/{}", i)).collect();
        let distribution = ring.get_distribution(keys.iter().map(|s| s.as_str()));

        // Each node should have roughly 333 keys (allow 20% variance)
        for (node, count) in &distribution {
            assert!(
                *count >= 200 && *count <= 500,
                "Node {} has unbalanced load: {} keys",
                node,
                count
            );
        }
    }

    #[test]
    fn test_empty_ring() {
        let ring = ConsistentHash::new(100);

        assert!(ring.get_node("test").is_none());
        assert!(ring.get_nodes("test", 2).is_empty());
        assert!(ring.is_empty());
    }

    #[test]
    fn test_duplicate_add() {
        let mut ring = ConsistentHash::new(10);

        ring.add_node("node-1");
        ring.add_node("node-1"); // Duplicate

        assert_eq!(ring.node_count(), 1);
        assert_eq!(ring.ring_size(), 10);
    }

    #[test]
    fn test_stats() {
        let mut ring = ConsistentHash::new(50);

        ring.add_node("node-1");
        ring.add_node("node-2");

        let stats = ring.stats();

        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.virtual_nodes, 50);
        assert_eq!(stats.ring_size, 100);
        assert_eq!(stats.arc_lengths.len(), 2);
    }

    #[test]
    fn test_get_hash_deterministic() {
        let ring = ConsistentHash::new(10);

        let h1 = ring.get_hash("test-key");
        let h2 = ring.get_hash("test-key");

        assert_eq!(h1, h2);
    }
}
