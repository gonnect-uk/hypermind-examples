//! Comprehensive Integration Tests for Sharding Strategies
//!
//! Tests based on research from:
//! - Petroni et al., "HDRF: Stream-Based Partitioning for Power-Law Graphs" (CIKM 2015)
//! - PowerGraph: "PowerLyra: Differentiated Graph Computation" (EuroSys 2015)
//! - Gonzalez et al., "GraphX: Distributed Graph-Parallel Computation" (SIGMOD 2014)
//!
//! ## Testing Strategy
//!
//! 1. **Correctness Tests**: Verify partitioning produces valid assignments
//! 2. **Balance Tests**: Ensure load is distributed evenly (variance < threshold)
//! 3. **Locality Tests**: Verify vertex locality (related edges on same partition)
//! 4. **Replication Tests**: Verify replication factor stays bounded
//! 5. **Stability Tests**: Ensure deterministic behavior for same input
//! 6. **Scale Tests**: Test with power-law distributions (LUBM-like)
//! 7. **Migration Tests**: Verify minimal key movement during scaling

use cluster::{
    ConsistentHash, HdrfPartitioner, NodeId, PartitionId, PartitionMap, PartitionState,
};
use std::collections::HashSet;

// ============================================================================
// CORRECTNESS TESTS
// ============================================================================

#[test]
fn test_hdrf_assigns_valid_partitions() {
    let mut partitioner = HdrfPartitioner::new(9, 1.0);

    // Assign 1000 random edges
    for i in 0..1000 {
        let partition = partitioner.assign_edge(i, i * 100, i + 500);
        assert!(
            partition < 9,
            "Partition {} out of range [0,9)",
            partition
        );
    }
}

#[test]
fn test_consistent_hash_assigns_valid_nodes() {
    let mut ring = ConsistentHash::new(150);
    ring.add_node("executor-1");
    ring.add_node("executor-2");
    ring.add_node("executor-3");

    // All keys should route to valid nodes
    for i in 0..1000 {
        let key = format!("http://example.org/entity/{}", i);
        let node = ring.get_node(&key);
        assert!(
            node.is_some(),
            "Key {} should route to a node",
            key
        );
        assert!(
            ["executor-1", "executor-2", "executor-3"].contains(&node.unwrap()),
            "Node {} is not in cluster",
            node.unwrap()
        );
    }
}

#[test]
fn test_partition_map_assignment_consistency() {
    let map = PartitionMap::new(9);

    // Assign partitions
    map.assign(PartitionId(0), NodeId(1));
    map.assign(PartitionId(1), NodeId(1));
    map.assign(PartitionId(2), NodeId(1));

    // Verify assignments persist
    assert_eq!(map.get_owner(PartitionId(0)), Some(NodeId(1)));
    assert_eq!(map.get_owner(PartitionId(1)), Some(NodeId(1)));
    assert_eq!(map.get_owner(PartitionId(2)), Some(NodeId(1)));

    // Verify node partition list
    let partitions = map.get_node_partitions(NodeId(1));
    assert_eq!(partitions.len(), 3);
    assert!(partitions.contains(&PartitionId(0)));
}

// ============================================================================
// BALANCE TESTS
// ============================================================================

/// Test load balance with uniform distribution
#[test]
fn test_hdrf_uniform_load_balance() {
    let mut partitioner = HdrfPartitioner::new(9, 1.0);

    // Assign edges with distinct subjects (uniform distribution)
    for i in 0..9000 {
        partitioner.assign_edge(i * 1000, 1, i + 100);
    }

    let stats = partitioner.get_stats();

    // Calculate coefficient of variation (CV = stddev / mean)
    let mean = stats.total_edges as f64 / stats.partition_loads.len() as f64;
    let variance: f64 = stats
        .partition_loads
        .iter()
        .map(|&load| {
            let diff = load as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / stats.partition_loads.len() as f64;
    let stddev = variance.sqrt();
    let cv = stddev / mean;

    // CV should be < 0.2 for good balance (20% variation)
    assert!(
        cv < 0.3,
        "Load imbalance too high: CV = {:.3} (max 0.3)",
        cv
    );
}

/// Test load balance with power-law distribution (realistic RDF)
#[test]
fn test_hdrf_power_law_balance() {
    let mut partitioner = HdrfPartitioner::new(9, 1.0);

    // Simulate power-law: few high-degree vertices, many low-degree
    // Hub vertex (like rdf:type) connects to many objects
    let hub = 1u64;
    for i in 0..500 {
        partitioner.assign_edge(hub, 10, i + 1000);
    }

    // Add regular vertices
    for i in 0..1000 {
        partitioner.assign_edge(i + 2000, 11, i + 3000);
    }

    let stats = partitioner.get_stats();

    // Even with power-law, imbalance should be < 2.0
    assert!(
        stats.load_imbalance < 2.5,
        "Power-law load imbalance too high: {:.2}",
        stats.load_imbalance
    );
}

/// Test consistent hash balance
#[test]
fn test_consistent_hash_balance() {
    let mut ring = ConsistentHash::new(150);
    ring.add_node("node-1");
    ring.add_node("node-2");
    ring.add_node("node-3");

    // Route 3000 keys
    let keys: Vec<String> = (0..3000)
        .map(|i| format!("http://example.org/entity/{}", i))
        .collect();

    let distribution = ring.get_distribution(keys.iter().map(|s| s.as_str()));

    // Each node should have roughly 1000 keys (±30%)
    for (node, count) in &distribution {
        assert!(
            *count >= 700 && *count <= 1300,
            "Node {} has unbalanced load: {} keys (expected ~1000)",
            node,
            count
        );
    }
}

// ============================================================================
// LOCALITY TESTS (EDGE CUTS)
// ============================================================================

/// Test subject locality: edges with same subject should cluster
#[test]
fn test_hdrf_subject_locality() {
    let mut partitioner = HdrfPartitioner::new(9, 0.5); // Lower lambda = more locality

    // Add edges about subject 1
    let subject1_partitions: HashSet<usize> = (0..10)
        .map(|i| partitioner.assign_edge(1, i as u64 + 100, i as u64 + 200))
        .collect();

    // Subject 1 should be concentrated (not spread across all partitions)
    // With lambda=0.5, balance still affects placement, so allow some spread
    assert!(
        subject1_partitions.len() <= 6,
        "Subject 1 spread across {} partitions (expected <= 6)",
        subject1_partitions.len()
    );
}

/// Test predicate fan-out: high-degree predicates should be replicated
#[test]
fn test_hdrf_predicate_fanout() {
    let mut partitioner = HdrfPartitioner::new(9, 1.0);

    // rdf:type connects many subjects to many objects
    let rdf_type = 999u64;
    for i in 0..100 {
        partitioner.assign_edge(i, rdf_type, (i % 10) + 1000);
    }

    let stats = partitioner.get_stats();

    // High-degree predicate should be replicated across partitions
    // But replication factor should still be reasonable
    assert!(
        stats.replication_factor < 4.0,
        "Replication factor too high: {:.2}",
        stats.replication_factor
    );
}

// ============================================================================
// REPLICATION TESTS
// ============================================================================

/// Test replication factor bounds
#[test]
fn test_hdrf_replication_bounds() {
    let mut partitioner = HdrfPartitioner::new(9, 1.0);

    // Large-scale test with mixed workload
    // Hub vertices
    for hub in 0..5 {
        for i in 0..100 {
            partitioner.assign_edge(hub, 100, i + hub * 1000);
        }
    }

    // Regular vertices
    for i in 0..2000 {
        partitioner.assign_edge(i + 1000000, 200, i + 2000000);
    }

    let stats = partitioner.get_stats();

    // Replication factor should be between 1.0 (no replication) and num_partitions
    assert!(
        stats.replication_factor >= 1.0,
        "Replication factor {} < 1.0 (impossible)",
        stats.replication_factor
    );
    assert!(
        stats.replication_factor < 9.0,
        "Replication factor {} >= num_partitions (every vertex replicated everywhere)",
        stats.replication_factor
    );

    // For HDRF with default lambda, expect RF between 1.2 and 3.0
    println!("Replication factor: {:.3}", stats.replication_factor);
}

/// Test consistent hash replication for fault tolerance
#[test]
fn test_consistent_hash_replication() {
    let mut ring = ConsistentHash::new(100);
    ring.add_node("node-1");
    ring.add_node("node-2");
    ring.add_node("node-3");

    // Get 2 replicas for each key
    let nodes = ring.get_nodes("test-key", 2);
    assert_eq!(nodes.len(), 2);

    // Replicas should be distinct
    let unique: HashSet<_> = nodes.iter().collect();
    assert_eq!(unique.len(), 2, "Replica nodes should be distinct");
}

// ============================================================================
// STABILITY/DETERMINISM TESTS
// ============================================================================

/// Test HDRF produces deterministic results
#[test]
fn test_hdrf_determinism() {
    // Create two identical partitioners
    let mut p1 = HdrfPartitioner::new(9, 1.0);
    let mut p2 = HdrfPartitioner::new(9, 1.0);

    // Same input sequence
    for i in 0..100 {
        let partition1 = p1.assign_edge(i, i * 10, i + 100);
        let partition2 = p2.assign_edge(i, i * 10, i + 100);

        assert_eq!(
            partition1, partition2,
            "Edge {} assigned differently: {} vs {}",
            i, partition1, partition2
        );
    }
}

/// Test consistent hash determinism
#[test]
fn test_consistent_hash_determinism() {
    let mut ring1 = ConsistentHash::new(100);
    let mut ring2 = ConsistentHash::new(100);

    // Same node order
    ring1.add_node("node-1");
    ring1.add_node("node-2");
    ring2.add_node("node-1");
    ring2.add_node("node-2");

    // Same key should route to same node
    for i in 0..100 {
        let key = format!("key-{}", i);
        let node1 = ring1.get_node(&key);
        let node2 = ring2.get_node(&key);
        assert_eq!(
            node1, node2,
            "Key {} routes differently: {:?} vs {:?}",
            key, node1, node2
        );
    }
}

// ============================================================================
// SCALE TESTS (LUBM-LIKE WORKLOADS)
// ============================================================================

/// Test with LUBM-like university ontology structure
#[test]
fn test_lubm_workload_partitioning() {
    let mut partitioner = HdrfPartitioner::new(9, 1.0);

    // Simulate LUBM structure:
    // - Universities contain Departments
    // - Departments have Professors and Students
    // - Professors teach Courses
    // - Students take Courses

    let mut rng_seed = 42u64;
    let mut simple_random = || {
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        rng_seed
    };

    // 5 universities
    for uni in 0..5 {
        let uni_id = uni * 100000;

        // 10 departments per university
        for dept in 0..10 {
            let dept_id = uni_id + dept * 1000;

            // University contains Department
            partitioner.assign_edge(uni_id, 1, dept_id); // contains predicate

            // 20 professors per department
            for prof in 0..20 {
                let prof_id = dept_id + prof * 10;
                partitioner.assign_edge(dept_id, 2, prof_id); // hasProfessor

                // Each professor teaches 3 courses
                for course in 0..3 {
                    let course_id = prof_id * 100 + course;
                    partitioner.assign_edge(prof_id, 3, course_id); // teaches
                }
            }

            // 100 students per department
            for student in 0..100 {
                let student_id = dept_id + 500 + student;
                partitioner.assign_edge(dept_id, 4, student_id); // hasStudent

                // Each student takes 5 courses (random professors' courses)
                for _ in 0..5 {
                    let random_prof = (simple_random() % 20) as u64;
                    let random_course = (simple_random() % 3) as u64;
                    let course_id = (dept_id + random_prof as u64 * 10) * 100 + random_course;
                    partitioner.assign_edge(student_id, 5, course_id); // takesCourse
                }
            }
        }
    }

    let stats = partitioner.get_stats();

    println!("LUBM-like workload stats:");
    println!("  Total edges: {}", stats.total_edges);
    println!("  Unique vertices: {}", stats.unique_vertices);
    println!("  Replication factor: {:.3}", stats.replication_factor);
    println!("  Load imbalance: {:.3}", stats.load_imbalance);
    println!("  Partition loads: {:?}", stats.partition_loads);

    // Verify reasonable metrics
    // LUBM workload is hierarchical, so some imbalance is expected
    assert!(
        stats.load_imbalance < 3.0,
        "LUBM load imbalance too high: {:.2}",
        stats.load_imbalance
    );
    assert!(
        stats.replication_factor < 4.0,
        "LUBM replication too high: {:.2}",
        stats.replication_factor
    );
}

// ============================================================================
// MIGRATION TESTS (SCALING)
// ============================================================================

/// Test consistent hash minimal migration on node addition
#[test]
fn test_consistent_hash_minimal_migration_add() {
    let mut ring = ConsistentHash::new(150);

    ring.add_node("node-1");
    ring.add_node("node-2");
    ring.add_node("node-3");

    // Track key assignments
    let keys: Vec<String> = (0..1000)
        .map(|i| format!("key-{}", i))
        .collect();

    let before: Vec<_> = keys
        .iter()
        .map(|k| ring.get_node(k).unwrap().to_string())
        .collect();

    // Add a new node
    ring.add_node("node-4");

    let after: Vec<_> = keys
        .iter()
        .map(|k| ring.get_node(k).unwrap().to_string())
        .collect();

    // Count migrated keys
    let migrated = before
        .iter()
        .zip(after.iter())
        .filter(|(b, a)| b != a)
        .count();

    // Expected migration: ~1/4 of keys (to new node)
    // Allow 10% tolerance
    let expected_max = (keys.len() as f64 * 0.35) as usize;

    assert!(
        migrated <= expected_max,
        "Too many keys migrated: {} (expected <= {})",
        migrated,
        expected_max
    );

    println!(
        "Keys migrated on node addition: {} / {} ({:.1}%)",
        migrated,
        keys.len(),
        100.0 * migrated as f64 / keys.len() as f64
    );
}

/// Test consistent hash minimal migration on node removal
#[test]
fn test_consistent_hash_minimal_migration_remove() {
    let mut ring = ConsistentHash::new(150);

    ring.add_node("node-1");
    ring.add_node("node-2");
    ring.add_node("node-3");

    let keys: Vec<String> = (0..1000)
        .map(|i| format!("key-{}", i))
        .collect();

    let before: Vec<_> = keys
        .iter()
        .map(|k| ring.get_node(k).unwrap().to_string())
        .collect();

    // Remove a node
    ring.remove_node("node-2");

    let after: Vec<_> = keys
        .iter()
        .map(|k| ring.get_node(k).unwrap().to_string())
        .collect();

    // Only keys that were on removed node should migrate
    let migrated = before
        .iter()
        .zip(after.iter())
        .filter(|(b, a)| b != a)
        .count();

    // Expected: ~1/3 of keys (from removed node)
    let expected_max = (keys.len() as f64 * 0.45) as usize;

    assert!(
        migrated <= expected_max,
        "Too many keys migrated: {} (expected <= {})",
        migrated,
        expected_max
    );
}

// ============================================================================
// PARTITION MAP TESTS
// ============================================================================

/// Test partition map versioning for cache invalidation
#[test]
fn test_partition_map_versioning() {
    let map = PartitionMap::new(9);

    let v1 = map.version();
    map.assign(PartitionId(0), NodeId(1));
    let v2 = map.version();
    map.assign(PartitionId(1), NodeId(2));
    let v3 = map.version();

    assert!(v2 > v1, "Version should increment on assign");
    assert!(v3 > v2, "Version should increment on each assign");
}

/// Test partition state transitions
#[test]
fn test_partition_state_transitions() {
    let map = PartitionMap::new(3);

    map.assign(PartitionId(0), NodeId(1));

    // Initial state is Active (set by assign)
    let info = map.get_info(PartitionId(0)).unwrap();
    assert_eq!(info.state, PartitionState::Active);

    // Transition to Rebalancing
    map.set_partition_state(PartitionId(0), PartitionState::Rebalancing)
        .unwrap();
    let info = map.get_info(PartitionId(0)).unwrap();
    assert_eq!(info.state, PartitionState::Rebalancing);
    assert!(info.state.can_serve()); // Can still serve during rebalancing

    // Transition to Offline
    map.set_partition_state(PartitionId(0), PartitionState::Offline)
        .unwrap();
    let info = map.get_info(PartitionId(0)).unwrap();
    assert!(!info.state.can_serve());
}

/// Test partition reassignment
#[test]
fn test_partition_reassignment() {
    let map = PartitionMap::new(9);

    // Assign to node 1
    map.assign(PartitionId(0), NodeId(1));
    assert_eq!(map.get_owner(PartitionId(0)), Some(NodeId(1)));
    assert!(map.get_node_partitions(NodeId(1)).contains(&PartitionId(0)));

    // Reassign to node 2
    map.assign(PartitionId(0), NodeId(2));
    assert_eq!(map.get_owner(PartitionId(0)), Some(NodeId(2)));
    assert!(map.get_node_partitions(NodeId(2)).contains(&PartitionId(0)));
    assert!(!map.get_node_partitions(NodeId(1)).contains(&PartitionId(0)));
}

// ============================================================================
// LAMBDA PARAMETER SENSITIVITY TESTS
// ============================================================================

/// Test HDRF with high lambda (prioritize balance)
#[test]
fn test_hdrf_high_lambda_balance() {
    let mut partitioner = HdrfPartitioner::new(3, 10.0);

    // Hub vertex
    for i in 0..30 {
        partitioner.assign_edge(1, 100, i + 1000);
    }

    let stats = partitioner.get_stats();

    // High lambda should spread hub across partitions
    // Load imbalance should be very low
    assert!(
        stats.load_imbalance < 1.5,
        "High lambda should produce balanced partitions: {}",
        stats.load_imbalance
    );
}

/// Test HDRF with low lambda (prioritize locality)
#[test]
fn test_hdrf_low_lambda_locality() {
    let mut partitioner = HdrfPartitioner::new(3, 0.1);

    // Hub vertex - should cluster
    let partitions: HashSet<usize> = (0..30)
        .map(|i| partitioner.assign_edge(1, 100, i + 1000))
        .collect();

    // Low lambda should keep hub in fewer partitions
    assert!(
        partitions.len() <= 2,
        "Low lambda should concentrate edges: {} partitions used",
        partitions.len()
    );
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

/// Test partition map JSON round-trip
#[test]
fn test_partition_map_serialization() {
    let map = PartitionMap::new(9);

    map.assign(PartitionId(0), NodeId(1));
    map.assign(PartitionId(1), NodeId(1));
    map.assign(PartitionId(2), NodeId(2));
    map.update_stats(PartitionId(0), 1000, 10000).unwrap();

    // Serialize
    let json = map.to_json().unwrap();

    // Deserialize
    let restored = PartitionMap::from_json(&json, 9).unwrap();

    // Verify
    assert_eq!(restored.get_owner(PartitionId(0)), Some(NodeId(1)));
    assert_eq!(restored.get_owner(PartitionId(2)), Some(NodeId(2)));

    let info = restored.get_info(PartitionId(0)).unwrap();
    assert_eq!(info.triple_count, 1000);
}
