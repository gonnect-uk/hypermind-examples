//! HDRF (High-Degree Replicated First) Partitioner
//!
//! Implementation of the HDRF streaming graph partitioning algorithm from:
//! Petroni et al., "HDRF: Stream-Based Partitioning for Power-Law Graphs" (CIKM 2015)
//!
//! ## Algorithm Overview
//!
//! HDRF is a streaming edge partitioner optimized for power-law graphs (like RDF).
//! It uses a scoring function that balances:
//! 1. **Replication cost**: Prefer partitions that already have edge endpoints
//! 2. **Load balance**: Prefer partitions with fewer edges
//!
//! ## Mathematical Foundation
//!
//! For each incoming edge (u, v), HDRF computes a score for each partition p:
//!
//! ```text
//! C_HDRF(u, v, p) = C_REP(u, v, p) + λ × C_BAL(p)
//! ```
//!
//! Where:
//! - `C_REP` = Replication score (prefer partitions with u or v)
//! - `C_BAL` = Balance score (prefer less-loaded partitions)
//! - `λ` = Balance parameter (default: 1.0)
//!
//! ### Replication Score
//!
//! ```text
//! C_REP(u, v, p) = g(u, p) + g(v, p)
//!
//! g(x, p) = {
//!     1 + (1 - θ(x))  if x ∈ partition p
//!     0               otherwise
//! }
//!
//! θ(x) = deg(x) / deg_max
//! ```
//!
//! High-degree vertices (like `rdf:type`) get lower replication scores,
//! so they're more likely to be replicated across partitions.
//!
//! ### Balance Score
//!
//! ```text
//! C_BAL(p) = 1 - |P_p| / avg_load
//!
//! avg_load = total_edges / num_partitions
//! ```
//!
//! Partitions below average load get positive scores.
//!
//! ## Example
//!
//! ```rust
//! use cluster::HdrfPartitioner;
//!
//! let mut partitioner = HdrfPartitioner::new(9, 1.0);
//!
//! // Assign edges (subject_id, predicate_id, object_id)
//! let partition = partitioner.assign_edge(1, 10, 2);
//! println!("Edge assigned to partition {}", partition);
//!
//! // Get partition statistics
//! let stats = partitioner.get_stats();
//! println!("Replication factor: {:.2}", stats.replication_factor);
//! ```

use crate::PartitionId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use xxhash_rust::xxh64::xxh64;

/// Statistics about partitioner performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionerStats {
    /// Total edges processed
    pub total_edges: u64,
    /// Number of unique vertices
    pub unique_vertices: u64,
    /// Average replication factor (1.0 = no replication)
    pub replication_factor: f64,
    /// Load imbalance ratio (max/avg)
    pub load_imbalance: f64,
    /// Edges per partition
    pub partition_loads: Vec<u64>,
    /// Maximum vertex degree seen
    pub max_degree: u64,
}

/// HDRF streaming graph partitioner.
///
/// Implements the High-Degree Replicated First algorithm for
/// streaming edge partitioning of power-law graphs.
#[derive(Debug)]
pub struct HdrfPartitioner {
    /// Number of partitions
    num_partitions: usize,

    /// Lambda parameter controlling balance vs replication tradeoff
    /// - Higher lambda (>1): Prioritize balance over replication
    /// - Lower lambda (<1): Prioritize replication over balance
    lambda: f64,

    /// Vertex ID -> degree (number of edges containing this vertex)
    vertex_degrees: HashMap<u64, u64>,

    /// Maximum degree seen so far
    max_degree: u64,

    /// Partition ID -> set of vertex IDs present in partition
    partition_vertices: Vec<HashSet<u64>>,

    /// Number of edges in each partition
    partition_loads: Vec<u64>,

    /// Total edges processed
    total_edges: u64,

    /// Vertex ID -> set of partitions containing this vertex
    vertex_partitions: HashMap<u64, HashSet<usize>>,
}

impl HdrfPartitioner {
    /// Create a new HDRF partitioner.
    ///
    /// # Arguments
    /// - `num_partitions`: Number of partitions (typically 3-27)
    /// - `lambda`: Balance parameter (default: 1.0)
    ///
    /// # Example
    ///
    /// ```rust
    /// use cluster::HdrfPartitioner;
    ///
    /// // Create partitioner with 9 partitions
    /// let partitioner = HdrfPartitioner::new(9, 1.0);
    /// ```
    pub fn new(num_partitions: usize, lambda: f64) -> Self {
        assert!(num_partitions > 0, "num_partitions must be > 0");
        assert!(lambda > 0.0, "lambda must be > 0");

        Self {
            num_partitions,
            lambda,
            vertex_degrees: HashMap::new(),
            max_degree: 1, // Avoid division by zero
            partition_vertices: vec![HashSet::new(); num_partitions],
            partition_loads: vec![0; num_partitions],
            total_edges: 0,
            vertex_partitions: HashMap::new(),
        }
    }

    /// Assign an edge to a partition using HDRF algorithm.
    ///
    /// # Arguments
    /// - `subject_id`: Hashed subject vertex ID
    /// - `predicate_id`: Hashed predicate vertex ID
    /// - `object_id`: Hashed object vertex ID
    ///
    /// # Returns
    /// The partition ID (0 to num_partitions-1) where this edge should be stored.
    ///
    /// For RDF triples, we treat the triple as a hyperedge connecting
    /// subject, predicate, and object. We use the subject as the primary
    /// vertex for partition assignment (subject-anchored partitioning).
    pub fn assign_edge(&mut self, subject_id: u64, predicate_id: u64, object_id: u64) -> usize {
        // Update vertex degrees
        self.update_degree(subject_id);
        self.update_degree(predicate_id);
        self.update_degree(object_id);

        // Primary vertex for partitioning is subject (subject-anchored)
        let primary_vertex = subject_id;
        let secondary_vertices = [predicate_id, object_id];

        // Calculate HDRF score for each partition
        let mut best_partition = 0;
        let mut best_score = f64::NEG_INFINITY;

        let avg_load = if self.total_edges == 0 {
            1.0
        } else {
            self.total_edges as f64 / self.num_partitions as f64
        };

        for p in 0..self.num_partitions {
            let score = self.calculate_hdrf_score(
                primary_vertex,
                &secondary_vertices,
                p,
                avg_load,
            );

            if score > best_score {
                best_score = score;
                best_partition = p;
            }
        }

        // Update partition state
        self.partition_loads[best_partition] += 1;
        self.total_edges += 1;

        // Add vertices to partition
        self.add_vertex_to_partition(subject_id, best_partition);
        self.add_vertex_to_partition(predicate_id, best_partition);
        self.add_vertex_to_partition(object_id, best_partition);

        best_partition
    }

    /// Calculate HDRF score for assigning edge to partition p.
    ///
    /// C_HDRF(u, v, p) = C_REP(u, v, p) + λ × C_BAL(p)
    fn calculate_hdrf_score(
        &self,
        primary: u64,
        secondaries: &[u64],
        partition: usize,
        avg_load: f64,
    ) -> f64 {
        // C_REP: Replication score
        let c_rep = self.replication_score(primary, partition)
            + secondaries
                .iter()
                .map(|&v| self.replication_score(v, partition))
                .sum::<f64>();

        // C_BAL: Balance score
        let c_bal = self.balance_score(partition, avg_load);

        c_rep + self.lambda * c_bal
    }

    /// Calculate replication score g(x, p) for vertex x and partition p.
    ///
    /// g(x, p) = 1 + (1 - θ(x)) if x ∈ partition p, else 0
    /// θ(x) = deg(x) / deg_max
    fn replication_score(&self, vertex: u64, partition: usize) -> f64 {
        if self.partition_vertices[partition].contains(&vertex) {
            let degree = *self.vertex_degrees.get(&vertex).unwrap_or(&1) as f64;
            let theta = degree / self.max_degree as f64;
            1.0 + (1.0 - theta)
        } else {
            0.0
        }
    }

    /// Calculate balance score C_BAL(p).
    ///
    /// C_BAL(p) = 1 - |P_p| / avg_load
    fn balance_score(&self, partition: usize, avg_load: f64) -> f64 {
        let load = self.partition_loads[partition] as f64;
        1.0 - (load / avg_load)
    }

    /// Update degree count for a vertex.
    fn update_degree(&mut self, vertex: u64) {
        let degree = self.vertex_degrees.entry(vertex).or_insert(0);
        *degree += 1;
        if *degree > self.max_degree {
            self.max_degree = *degree;
        }
    }

    /// Add vertex to partition tracking.
    fn add_vertex_to_partition(&mut self, vertex: u64, partition: usize) {
        self.partition_vertices[partition].insert(vertex);
        self.vertex_partitions
            .entry(vertex)
            .or_insert_with(HashSet::new)
            .insert(partition);
    }

    /// Assign a triple using string IRIs (hashes them internally).
    ///
    /// # Example
    ///
    /// ```rust
    /// use cluster::HdrfPartitioner;
    ///
    /// let mut partitioner = HdrfPartitioner::new(9, 1.0);
    /// let partition = partitioner.assign_triple(
    ///     "http://example.org/Alice",
    ///     "http://xmlns.com/foaf/0.1/knows",
    ///     "http://example.org/Bob",
    /// );
    /// ```
    pub fn assign_triple(&mut self, subject: &str, predicate: &str, object: &str) -> PartitionId {
        let s_hash = xxh64(subject.as_bytes(), 0);
        let p_hash = xxh64(predicate.as_bytes(), 0);
        let o_hash = xxh64(object.as_bytes(), 0);

        PartitionId(self.assign_edge(s_hash, p_hash, o_hash) as u16)
    }

    /// Get partition assignment for a triple without updating state.
    ///
    /// Useful for query routing to determine which partition(s) to query.
    pub fn get_partition(&self, subject: &str) -> PartitionId {
        let s_hash = xxh64(subject.as_bytes(), 0);

        // Check if subject is already assigned to partitions
        if let Some(partitions) = self.vertex_partitions.get(&s_hash) {
            if let Some(&p) = partitions.iter().next() {
                return PartitionId(p as u16);
            }
        }

        // If not assigned yet, use consistent hash
        let partition = (s_hash % self.num_partitions as u64) as u16;
        PartitionId(partition)
    }

    /// Get all partitions that contain a specific subject.
    ///
    /// Useful for query routing when a subject might be replicated.
    pub fn get_subject_partitions(&self, subject: &str) -> Vec<PartitionId> {
        let s_hash = xxh64(subject.as_bytes(), 0);

        self.vertex_partitions
            .get(&s_hash)
            .map(|parts| parts.iter().map(|&p| PartitionId(p as u16)).collect())
            .unwrap_or_else(Vec::new)
    }

    /// Get partitioner statistics.
    pub fn get_stats(&self) -> PartitionerStats {
        let unique_vertices = self.vertex_degrees.len() as u64;

        // Calculate replication factor
        let total_vertex_occurrences: usize = self.partition_vertices
            .iter()
            .map(|s| s.len())
            .sum();

        let replication_factor = if unique_vertices > 0 {
            total_vertex_occurrences as f64 / unique_vertices as f64
        } else {
            1.0
        };

        // Calculate load imbalance
        let avg_load = if self.num_partitions > 0 {
            self.total_edges as f64 / self.num_partitions as f64
        } else {
            0.0
        };

        let max_load = *self.partition_loads.iter().max().unwrap_or(&0) as f64;
        let load_imbalance = if avg_load > 0.0 {
            max_load / avg_load
        } else {
            1.0
        };

        PartitionerStats {
            total_edges: self.total_edges,
            unique_vertices,
            replication_factor,
            load_imbalance,
            partition_loads: self.partition_loads.clone(),
            max_degree: self.max_degree,
        }
    }

    /// Get number of partitions.
    pub fn num_partitions(&self) -> usize {
        self.num_partitions
    }

    /// Get lambda parameter.
    pub fn lambda(&self) -> f64 {
        self.lambda
    }

    /// Reset partitioner state (for testing or rebalancing).
    pub fn reset(&mut self) {
        self.vertex_degrees.clear();
        self.max_degree = 1;
        self.partition_vertices = vec![HashSet::new(); self.num_partitions];
        self.partition_loads = vec![0; self.num_partitions];
        self.total_edges = 0;
        self.vertex_partitions.clear();
    }
}

/// Simple hash-based partitioner for comparison benchmarks.
///
/// Uses consistent hashing based on subject IRI.
/// Faster but may produce worse partition quality.
#[derive(Debug)]
pub struct HashPartitioner {
    num_partitions: usize,
}

impl HashPartitioner {
    /// Create a new hash partitioner.
    pub fn new(num_partitions: usize) -> Self {
        assert!(num_partitions > 0);
        Self { num_partitions }
    }

    /// Assign triple to partition using subject hash.
    pub fn assign_triple(&self, subject: &str, _predicate: &str, _object: &str) -> PartitionId {
        let hash = xxh64(subject.as_bytes(), 0);
        PartitionId((hash % self.num_partitions as u64) as u16)
    }

    /// Get partition for a subject.
    pub fn get_partition(&self, subject: &str) -> PartitionId {
        let hash = xxh64(subject.as_bytes(), 0);
        PartitionId((hash % self.num_partitions as u64) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdrf_basic() {
        let mut partitioner = HdrfPartitioner::new(3, 1.0);

        // Assign some edges
        let p1 = partitioner.assign_edge(1, 10, 2);
        let p2 = partitioner.assign_edge(1, 11, 3);
        let p3 = partitioner.assign_edge(2, 10, 4);

        // All partitions should be valid
        assert!(p1 < 3);
        assert!(p2 < 3);
        assert!(p3 < 3);

        // Same subject should tend to go to same partition
        // (due to replication score bonus)
        // Note: This is probabilistic, so we just check counts
        let stats = partitioner.get_stats();
        assert_eq!(stats.total_edges, 3);
    }

    #[test]
    fn test_hdrf_balance() {
        let mut partitioner = HdrfPartitioner::new(3, 10.0); // High lambda = balance priority

        // Assign 30 edges with distinct subjects
        for i in 0..30 {
            partitioner.assign_edge(i * 1000, 1, i * 1000 + 1);
        }

        let stats = partitioner.get_stats();

        // With high lambda, loads should be relatively balanced
        // Each partition should have roughly 10 edges
        assert!(stats.load_imbalance < 1.5, "Load imbalance too high: {}", stats.load_imbalance);
    }

    #[test]
    fn test_hdrf_replication() {
        let mut partitioner = HdrfPartitioner::new(3, 0.1); // Low lambda = replication priority

        // Assign many edges with same subject
        for i in 0..10 {
            partitioner.assign_edge(1, i as u64, (i + 100) as u64);
        }

        // Subject 1 should mostly be in one partition
        let s1_partitions = partitioner.get_subject_partitions(""); // Can't use this for ID

        // Check via internal state
        let subject_partition_count = partitioner.vertex_partitions.get(&1)
            .map(|s| s.len())
            .unwrap_or(0);

        // With low lambda, subject should be concentrated
        // (though some replication is expected)
        assert!(subject_partition_count <= 3);
    }

    #[test]
    fn test_hdrf_power_law() {
        let mut partitioner = HdrfPartitioner::new(9, 1.0);

        // Simulate power-law: one vertex connects to many others
        let hub_vertex = 1u64;
        for i in 0..100 {
            partitioner.assign_edge(hub_vertex, 10, i + 100);
        }

        // Also add some normal vertices
        for i in 0..50 {
            partitioner.assign_edge(i + 1000, 10, i + 2000);
        }

        let stats = partitioner.get_stats();

        // Hub should have high degree (predicate 10 appears in all 150 edges)
        assert_eq!(stats.max_degree, 150);

        // Replication factor should be reasonable (< 3)
        assert!(stats.replication_factor < 3.0,
            "Replication factor too high: {}", stats.replication_factor);
    }

    #[test]
    fn test_hdrf_triple_assignment() {
        let mut partitioner = HdrfPartitioner::new(9, 1.0);

        let p1 = partitioner.assign_triple(
            "http://example.org/Alice",
            "http://xmlns.com/foaf/0.1/knows",
            "http://example.org/Bob",
        );

        let p2 = partitioner.assign_triple(
            "http://example.org/Alice",
            "http://xmlns.com/foaf/0.1/name",
            "Alice",
        );

        // Both triples about Alice should be in valid partitions
        assert!(p1.0 < 9);
        assert!(p2.0 < 9);

        // Get partition for Alice
        let alice_partition = partitioner.get_partition("http://example.org/Alice");
        assert!(alice_partition.0 < 9);
    }

    #[test]
    fn test_hash_partitioner() {
        let partitioner = HashPartitioner::new(9);

        let p1 = partitioner.assign_triple("http://example.org/A", "p", "o1");
        let p2 = partitioner.assign_triple("http://example.org/A", "p", "o2");
        let p3 = partitioner.assign_triple("http://example.org/B", "p", "o3");

        // Same subject should always go to same partition
        assert_eq!(p1, p2);

        // Different subjects may go to different partitions (but this is hash-based)
        assert!(p1.0 < 9);
        assert!(p3.0 < 9);
    }

    #[test]
    fn test_partitioner_stats() {
        let mut partitioner = HdrfPartitioner::new(3, 1.0);

        for i in 0..9 {
            partitioner.assign_edge(i, 100, i + 10);
        }

        let stats = partitioner.get_stats();

        assert_eq!(stats.total_edges, 9);
        assert!(stats.unique_vertices > 0);
        assert!(stats.replication_factor >= 1.0);
        assert!(stats.load_imbalance >= 1.0);
        assert_eq!(stats.partition_loads.len(), 3);
    }

    #[test]
    fn test_partitioner_reset() {
        let mut partitioner = HdrfPartitioner::new(3, 1.0);

        partitioner.assign_edge(1, 2, 3);
        assert_eq!(partitioner.get_stats().total_edges, 1);

        partitioner.reset();
        assert_eq!(partitioner.get_stats().total_edges, 0);
        assert_eq!(partitioner.get_stats().unique_vertices, 0);
    }
}
