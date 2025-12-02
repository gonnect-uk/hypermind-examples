//! Integration Tests for Query Plan API
//!
//! Tests based on research from:
//! - Neumann, "Efficiently Compiling Efficient Query Plans for Modern Hardware" (VLDB 2011)
//! - Moerkotte, "Building Query Compilers" (2014)
//! - Leis et al., "How Good Are Query Optimizers, Really?" (VLDB 2015)
//!
//! ## Testing Strategy
//!
//! 1. **Plan Generation Tests**: Verify correct plan structure for queries
//! 2. **Cost Model Tests**: Test cost estimation accuracy
//! 3. **Plan Optimization Tests**: Test optimizer transformations
//! 4. **Distributed Plan Tests**: Test partition-aware planning
//! 5. **Plan Serialization Tests**: Test plan JSON export
//! 6. **Plan Comparison Tests**: Compare different strategies

use cluster::{NodeId, PartitionId, PartitionMap};
use serde::{Deserialize, Serialize};

// ============================================================================
// QUERY PLAN DATA STRUCTURES
// ============================================================================

/// Physical query plan node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryPlan {
    /// Sequential scan of a partition
    Scan {
        partition: u16,
        pattern: TriplePattern,
        estimated_rows: u64,
    },

    /// Index seek (more efficient than scan)
    IndexSeek {
        partition: u16,
        index: IndexType,
        pattern: TriplePattern,
        estimated_rows: u64,
    },

    /// Nested loop join
    NestedLoopJoin {
        left: Box<QueryPlan>,
        right: Box<QueryPlan>,
        join_variables: Vec<String>,
    },

    /// Hash join (build hash table on smaller side)
    HashJoin {
        build: Box<QueryPlan>,
        probe: Box<QueryPlan>,
        join_variables: Vec<String>,
    },

    /// WCOJ (LeapFrog TrieJoin)
    WCOJoin {
        inputs: Vec<Box<QueryPlan>>,
        join_variables: Vec<String>,
    },

    /// Union of multiple plans
    Union {
        branches: Vec<QueryPlan>,
    },

    /// Filter operator
    Filter {
        input: Box<QueryPlan>,
        expression: String,
    },

    /// Projection (select columns)
    Project {
        input: Box<QueryPlan>,
        variables: Vec<String>,
    },

    /// Distributed exchange (shuffle between partitions)
    Exchange {
        input: Box<QueryPlan>,
        target_partitions: Vec<u16>,
        exchange_type: ExchangeType,
    },

    /// Aggregation
    Aggregate {
        input: Box<QueryPlan>,
        group_by: Vec<String>,
        aggregates: Vec<AggregateOp>,
    },

    /// Sort operator
    Sort {
        input: Box<QueryPlan>,
        order_by: Vec<(String, SortOrder)>,
    },

    /// Limit/Offset
    Limit {
        input: Box<QueryPlan>,
        limit: u64,
        offset: u64,
    },
}

/// Triple pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriplePattern {
    pub subject: PatternElement,
    pub predicate: PatternElement,
    pub object: PatternElement,
}

/// Element in a triple pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternElement {
    Variable(String),
    Constant(String),
}

/// Index types for seeks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IndexType {
    SPOC, // Subject-Predicate-Object-Context
    POCS, // Predicate-Object-Context-Subject
    OCSP, // Object-Context-Subject-Predicate
    CSPO, // Context-Subject-Predicate-Object
}

/// Exchange types for distributed queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ExchangeType {
    /// Broadcast to all partitions
    Broadcast,
    /// Hash-based repartition
    HashRepartition,
    /// Gather to coordinator
    Gather,
}

/// Aggregation operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AggregateOp {
    pub function: AggregateFunction,
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    GroupConcat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

// ============================================================================
// COST MODEL
// ============================================================================

/// Cost model parameters (based on SSD + network assumptions)
#[derive(Debug, Clone)]
pub struct CostModel {
    /// Cost per index seek (~5µs for SSD)
    pub c_seek: f64,
    /// Cost per row scan (~0.1µs)
    pub c_scan: f64,
    /// Cost per hash operation (~0.05µs)
    pub c_hash: f64,
    /// Cost per join iteration (~2µs)
    pub c_join: f64,
    /// Cost per network RTT (~100µs)
    pub c_network: f64,
    /// Cost per byte transferred (~0.01µs)
    pub c_transfer: f64,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            c_seek: 5.0,
            c_scan: 0.1,
            c_hash: 0.05,
            c_join: 2.0,
            c_network: 100.0,
            c_transfer: 0.01,
        }
    }
}

impl CostModel {
    /// Estimate cost of a query plan
    pub fn estimate(&self, plan: &QueryPlan) -> f64 {
        match plan {
            QueryPlan::Scan { estimated_rows, .. } => {
                self.c_seek + self.c_scan * *estimated_rows as f64
            }

            QueryPlan::IndexSeek { estimated_rows, .. } => {
                self.c_seek + self.c_scan * *estimated_rows as f64 * 0.1 // Index is 10x faster
            }

            QueryPlan::NestedLoopJoin { left, right, .. } => {
                let left_cost = self.estimate(left);
                let right_cost = self.estimate(right);
                let left_rows = self.estimate_cardinality(left);
                let right_rows = self.estimate_cardinality(right);

                left_cost + left_rows * (right_cost + self.c_join * right_rows)
            }

            QueryPlan::HashJoin { build, probe, .. } => {
                let build_cost = self.estimate(build);
                let probe_cost = self.estimate(probe);
                let build_rows = self.estimate_cardinality(build);
                let probe_rows = self.estimate_cardinality(probe);

                // Build hash table + probe
                build_cost + self.c_hash * build_rows
                    + probe_cost + self.c_hash * probe_rows
            }

            QueryPlan::WCOJoin { inputs, .. } => {
                // WCOJ is optimal for multi-way joins
                let input_costs: f64 = inputs.iter().map(|i| self.estimate(i)).sum();
                let max_rows: f64 = inputs
                    .iter()
                    .map(|i| self.estimate_cardinality(i))
                    .fold(0.0, f64::max);

                input_costs + self.c_join * max_rows.sqrt() // Sub-linear for WCOJ
            }

            QueryPlan::Exchange { input, target_partitions, .. } => {
                let input_cost = self.estimate(input);
                let rows = self.estimate_cardinality(input);
                let partitions = target_partitions.len() as f64;

                input_cost + self.c_network * partitions + self.c_transfer * rows * 100.0
            }

            QueryPlan::Union { branches } => {
                branches.iter().map(|b| self.estimate(b)).sum()
            }

            QueryPlan::Filter { input, .. } => {
                self.estimate(input) * 1.1 // 10% overhead for filtering
            }

            QueryPlan::Project { input, .. } => {
                self.estimate(input) // Projection is cheap
            }

            QueryPlan::Aggregate { input, .. } => {
                let input_cost = self.estimate(input);
                let rows = self.estimate_cardinality(input);
                input_cost + self.c_hash * rows
            }

            QueryPlan::Sort { input, .. } => {
                let input_cost = self.estimate(input);
                let rows = self.estimate_cardinality(input);
                input_cost + rows * (rows as f64).log2() * self.c_scan
            }

            QueryPlan::Limit { input, .. } => {
                self.estimate(input) // Limit doesn't add cost
            }
        }
    }

    /// Estimate cardinality (number of output rows)
    fn estimate_cardinality(&self, plan: &QueryPlan) -> f64 {
        match plan {
            QueryPlan::Scan { estimated_rows, .. } => *estimated_rows as f64,
            QueryPlan::IndexSeek { estimated_rows, .. } => *estimated_rows as f64,
            QueryPlan::NestedLoopJoin { left, right, .. } => {
                // Assume 10% selectivity
                self.estimate_cardinality(left) * self.estimate_cardinality(right) * 0.1
            }
            QueryPlan::HashJoin { build, probe, .. } => {
                self.estimate_cardinality(build).min(self.estimate_cardinality(probe))
            }
            QueryPlan::WCOJoin { inputs, .. } => {
                inputs.iter().map(|i| self.estimate_cardinality(i)).fold(f64::MAX, f64::min)
            }
            QueryPlan::Union { branches } => {
                branches.iter().map(|b| self.estimate_cardinality(b)).sum()
            }
            QueryPlan::Filter { input, .. } => self.estimate_cardinality(input) * 0.5,
            QueryPlan::Project { input, .. } => self.estimate_cardinality(input),
            QueryPlan::Exchange { input, .. } => self.estimate_cardinality(input),
            QueryPlan::Aggregate { input, group_by, .. } => {
                if group_by.is_empty() {
                    1.0
                } else {
                    self.estimate_cardinality(input) * 0.1 // Assume 10% distinct groups
                }
            }
            QueryPlan::Sort { input, .. } => self.estimate_cardinality(input),
            QueryPlan::Limit { limit, .. } => *limit as f64,
        }
    }
}

// ============================================================================
// PLAN GENERATION TESTS
// ============================================================================

#[test]
fn test_simple_scan_plan() {
    let plan = QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("rdf:type".to_string()),
            object: PatternElement::Variable("?o".to_string()),
        },
        estimated_rows: 1000,
    };

    // Verify plan structure
    if let QueryPlan::Scan { partition, pattern, estimated_rows } = &plan {
        assert_eq!(*partition, 0);
        assert_eq!(*estimated_rows, 1000);
        assert!(matches!(pattern.subject, PatternElement::Variable(_)));
    } else {
        panic!("Expected Scan plan");
    }
}

#[test]
fn test_join_plan_generation() {
    let left = Box::new(QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("foaf:knows".to_string()),
            object: PatternElement::Variable("?friend".to_string()),
        },
        estimated_rows: 100,
    });

    let right = Box::new(QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?friend".to_string()),
            predicate: PatternElement::Constant("foaf:name".to_string()),
            object: PatternElement::Variable("?name".to_string()),
        },
        estimated_rows: 50,
    });

    let join = QueryPlan::HashJoin {
        build: right, // Smaller side builds hash table
        probe: left,
        join_variables: vec!["?friend".to_string()],
    };

    // Verify join structure
    if let QueryPlan::HashJoin { join_variables, .. } = &join {
        assert_eq!(join_variables, &vec!["?friend".to_string()]);
    } else {
        panic!("Expected HashJoin plan");
    }
}

#[test]
fn test_distributed_plan_with_exchange() {
    let local_scan = QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("p".to_string()),
            object: PatternElement::Variable("?o".to_string()),
        },
        estimated_rows: 1000,
    };

    let distributed = QueryPlan::Exchange {
        input: Box::new(local_scan),
        target_partitions: vec![0, 1, 2],
        exchange_type: ExchangeType::Gather,
    };

    if let QueryPlan::Exchange { target_partitions, exchange_type, .. } = &distributed {
        assert_eq!(target_partitions.len(), 3);
        assert_eq!(*exchange_type, ExchangeType::Gather);
    } else {
        panic!("Expected Exchange plan");
    }
}

// ============================================================================
// COST MODEL TESTS
// ============================================================================

#[test]
fn test_scan_cost() {
    let cost_model = CostModel::default();

    let plan = QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Variable("?p".to_string()),
            object: PatternElement::Variable("?o".to_string()),
        },
        estimated_rows: 1000,
    };

    let cost = cost_model.estimate(&plan);

    // Cost = c_seek + c_scan * rows = 5 + 0.1 * 1000 = 105
    assert!(
        (cost - 105.0).abs() < 0.1,
        "Scan cost mismatch: expected 105, got {}",
        cost
    );
}

#[test]
fn test_index_seek_cheaper_than_scan() {
    let cost_model = CostModel::default();

    let scan = QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Constant("s1".to_string()),
            predicate: PatternElement::Variable("?p".to_string()),
            object: PatternElement::Variable("?o".to_string()),
        },
        estimated_rows: 1000,
    };

    let seek = QueryPlan::IndexSeek {
        partition: 0,
        index: IndexType::SPOC,
        pattern: TriplePattern {
            subject: PatternElement::Constant("s1".to_string()),
            predicate: PatternElement::Variable("?p".to_string()),
            object: PatternElement::Variable("?o".to_string()),
        },
        estimated_rows: 1000,
    };

    let scan_cost = cost_model.estimate(&scan);
    let seek_cost = cost_model.estimate(&seek);

    assert!(
        seek_cost < scan_cost,
        "Index seek ({}) should be cheaper than scan ({})",
        seek_cost,
        scan_cost
    );
}

#[test]
fn test_hash_join_cheaper_than_nested_loop() {
    let cost_model = CostModel::default();

    let left = Box::new(QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("p1".to_string()),
            object: PatternElement::Variable("?o1".to_string()),
        },
        estimated_rows: 1000,
    });

    let right = Box::new(QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("p2".to_string()),
            object: PatternElement::Variable("?o2".to_string()),
        },
        estimated_rows: 1000,
    });

    let nested_loop = QueryPlan::NestedLoopJoin {
        left: left.clone(),
        right: right.clone(),
        join_variables: vec!["?s".to_string()],
    };

    let hash_join = QueryPlan::HashJoin {
        build: right,
        probe: left,
        join_variables: vec!["?s".to_string()],
    };

    let nl_cost = cost_model.estimate(&nested_loop);
    let hj_cost = cost_model.estimate(&hash_join);

    assert!(
        hj_cost < nl_cost,
        "Hash join ({}) should be cheaper than nested loop ({})",
        hj_cost,
        nl_cost
    );
}

#[test]
fn test_wcoj_efficient_for_multiway_joins() {
    let cost_model = CostModel::default();

    // Triangle query: ?a knows ?b, ?b knows ?c, ?c knows ?a
    let patterns: Vec<Box<QueryPlan>> = (0..3)
        .map(|i| {
            Box::new(QueryPlan::Scan {
                partition: 0,
                pattern: TriplePattern {
                    subject: PatternElement::Variable(format!("?v{}", i)),
                    predicate: PatternElement::Constant("knows".to_string()),
                    object: PatternElement::Variable(format!("?v{}", (i + 1) % 3)),
                },
                estimated_rows: 1000,
            })
        })
        .collect();

    let wcoj = QueryPlan::WCOJoin {
        inputs: patterns,
        join_variables: vec!["?v0".to_string(), "?v1".to_string(), "?v2".to_string()],
    };

    let cost = cost_model.estimate(&wcoj);

    // WCOJ should have sub-linear cost for multi-way joins
    // Cost should be roughly O(sqrt(n)) for triangle queries
    assert!(
        cost < 10000.0,
        "WCOJ cost ({}) should be reasonable for triangle query",
        cost
    );
}

// ============================================================================
// DISTRIBUTED PLAN TESTS
// ============================================================================

#[test]
fn test_partition_aware_plan() {
    let partition_map = PartitionMap::new(9);

    // Setup partitions
    for p in 0..9 {
        partition_map.assign(PartitionId(p), NodeId((p % 3 + 1) as u64));
    }

    // Query that needs data from multiple partitions
    let plans: Vec<QueryPlan> = (0..3)
        .map(|p| QueryPlan::Scan {
            partition: p,
            pattern: TriplePattern {
                subject: PatternElement::Variable("?s".to_string()),
                predicate: PatternElement::Constant("p".to_string()),
                object: PatternElement::Variable("?o".to_string()),
            },
            estimated_rows: 1000,
        })
        .collect();

    // Union of scans from each partition
    let union = QueryPlan::Union { branches: plans };

    // Gather results to coordinator
    let distributed = QueryPlan::Exchange {
        input: Box::new(union),
        target_partitions: vec![0], // Coordinator
        exchange_type: ExchangeType::Gather,
    };

    let cost_model = CostModel::default();
    let cost = cost_model.estimate(&distributed);

    // Should include network cost
    assert!(
        cost > 100.0,
        "Distributed plan should have significant network cost: {}",
        cost
    );
}

#[test]
fn test_broadcast_vs_repartition() {
    let cost_model = CostModel::default();

    let small_table = Box::new(QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("dim".to_string()),
            object: PatternElement::Variable("?o".to_string()),
        },
        estimated_rows: 100, // Small dimension table
    });

    let large_table = Box::new(QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("fact".to_string()),
            object: PatternElement::Variable("?o".to_string()),
        },
        estimated_rows: 1_000_000, // Large fact table
    });

    // Option 1: Broadcast small table
    let broadcast_plan = QueryPlan::Exchange {
        input: small_table.clone(),
        target_partitions: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        exchange_type: ExchangeType::Broadcast,
    };

    // Option 2: Repartition large table
    let repartition_plan = QueryPlan::Exchange {
        input: large_table,
        target_partitions: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        exchange_type: ExchangeType::HashRepartition,
    };

    let broadcast_cost = cost_model.estimate(&broadcast_plan);
    let repartition_cost = cost_model.estimate(&repartition_plan);

    assert!(
        broadcast_cost < repartition_cost,
        "Broadcasting small table ({}) should be cheaper than repartitioning large ({:})",
        broadcast_cost,
        repartition_cost
    );
}

// ============================================================================
// PLAN SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_plan_json_serialization() {
    let plan = QueryPlan::HashJoin {
        build: Box::new(QueryPlan::Scan {
            partition: 0,
            pattern: TriplePattern {
                subject: PatternElement::Variable("?s".to_string()),
                predicate: PatternElement::Constant("p1".to_string()),
                object: PatternElement::Variable("?o".to_string()),
            },
            estimated_rows: 100,
        }),
        probe: Box::new(QueryPlan::Scan {
            partition: 1,
            pattern: TriplePattern {
                subject: PatternElement::Variable("?s".to_string()),
                predicate: PatternElement::Constant("p2".to_string()),
                object: PatternElement::Variable("?x".to_string()),
            },
            estimated_rows: 200,
        }),
        join_variables: vec!["?s".to_string()],
    };

    // Serialize
    let json = serde_json::to_string_pretty(&plan).unwrap();
    assert!(json.contains("HashJoin"));
    assert!(json.contains("Scan"));
    assert!(json.contains("?s"));

    // Deserialize
    let restored: QueryPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(plan, restored);
}

#[test]
fn test_explain_output_format() {
    let plan = QueryPlan::Project {
        input: Box::new(QueryPlan::Filter {
            input: Box::new(QueryPlan::HashJoin {
                build: Box::new(QueryPlan::IndexSeek {
                    partition: 0,
                    index: IndexType::SPOC,
                    pattern: TriplePattern {
                        subject: PatternElement::Constant("Alice".to_string()),
                        predicate: PatternElement::Variable("?p".to_string()),
                        object: PatternElement::Variable("?o".to_string()),
                    },
                    estimated_rows: 10,
                }),
                probe: Box::new(QueryPlan::Scan {
                    partition: 0,
                    pattern: TriplePattern {
                        subject: PatternElement::Variable("?o".to_string()),
                        predicate: PatternElement::Constant("name".to_string()),
                        object: PatternElement::Variable("?name".to_string()),
                    },
                    estimated_rows: 1000,
                }),
                join_variables: vec!["?o".to_string()],
            }),
            expression: "STRLEN(?name) > 5".to_string(),
        }),
        variables: vec!["?p".to_string(), "?name".to_string()],
    };

    let cost_model = CostModel::default();
    let cost = cost_model.estimate(&plan);

    // Create EXPLAIN output
    let explain = serde_json::json!({
        "plan": plan,
        "estimated_cost": cost,
        "estimated_rows": cost_model.estimate(&plan) / cost_model.c_scan,
    });

    let output = serde_json::to_string_pretty(&explain).unwrap();
    assert!(output.contains("Project"));
    assert!(output.contains("Filter"));
    assert!(output.contains("HashJoin"));
    assert!(output.contains("estimated_cost"));
}

// ============================================================================
// PLAN COMPARISON TESTS
// ============================================================================

#[test]
fn test_select_best_join_strategy() {
    let cost_model = CostModel::default();

    let small_left = Box::new(QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Constant("s".to_string()),
            predicate: PatternElement::Variable("?p".to_string()),
            object: PatternElement::Variable("?o".to_string()),
        },
        estimated_rows: 10,
    });

    let large_right = Box::new(QueryPlan::Scan {
        partition: 0,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?o".to_string()),
            predicate: PatternElement::Constant("related".to_string()),
            object: PatternElement::Variable("?x".to_string()),
        },
        estimated_rows: 10000,
    });

    // Compare strategies
    let strategies = vec![
        (
            "NestedLoop",
            QueryPlan::NestedLoopJoin {
                left: small_left.clone(),
                right: large_right.clone(),
                join_variables: vec!["?o".to_string()],
            },
        ),
        (
            "HashJoin",
            QueryPlan::HashJoin {
                build: small_left.clone(),
                probe: large_right.clone(),
                join_variables: vec!["?o".to_string()],
            },
        ),
    ];

    let mut best_strategy = "";
    let mut best_cost = f64::MAX;

    for (name, plan) in &strategies {
        let cost = cost_model.estimate(plan);
        if cost < best_cost {
            best_cost = cost;
            best_strategy = name;
        }
    }

    // Hash join should win for this case
    assert_eq!(
        best_strategy, "HashJoin",
        "HashJoin should be selected for small-large join"
    );
}

#[test]
fn test_index_selection() {
    let cost_model = CostModel::default();

    // Query: ?s foaf:name "Alice"
    // Should use OCSP index (object-first for constant object)

    let spoc_plan = QueryPlan::IndexSeek {
        partition: 0,
        index: IndexType::SPOC,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("foaf:name".to_string()),
            object: PatternElement::Constant("Alice".to_string()),
        },
        estimated_rows: 1000, // Full scan with SPOC
    };

    let pocs_plan = QueryPlan::IndexSeek {
        partition: 0,
        index: IndexType::POCS,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("foaf:name".to_string()),
            object: PatternElement::Constant("Alice".to_string()),
        },
        estimated_rows: 100, // Better with POCS (predicate+object)
    };

    let ocsp_plan = QueryPlan::IndexSeek {
        partition: 0,
        index: IndexType::OCSP,
        pattern: TriplePattern {
            subject: PatternElement::Variable("?s".to_string()),
            predicate: PatternElement::Constant("foaf:name".to_string()),
            object: PatternElement::Constant("Alice".to_string()),
        },
        estimated_rows: 10, // Best with OCSP (object-first)
    };

    let spoc_cost = cost_model.estimate(&spoc_plan);
    let pocs_cost = cost_model.estimate(&pocs_plan);
    let ocsp_cost = cost_model.estimate(&ocsp_plan);

    assert!(
        ocsp_cost < pocs_cost && pocs_cost < spoc_cost,
        "OCSP ({}) should be cheapest, then POCS ({}), then SPOC ({})",
        ocsp_cost,
        pocs_cost,
        spoc_cost
    );
}
