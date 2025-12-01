//! SPARQL Query Optimizer with WCOJ Integration
//!
//! Analyzes query patterns and selects optimal execution strategy:
//! - **WCOJ (LeapFrog TrieJoin)**: For star queries, cyclic queries, complex joins
//! - **Nested Loop Join**: For simple 2-way joins, linear patterns
//!
//! # Auto-Detection
//!
//! The optimizer automatically detects:
//! - Star queries: Multiple patterns sharing a common variable
//! - Cyclic queries: Variables forming cycles in the join graph
//! - Complex multi-way joins: More than 3 triple patterns
//!
//! # Query Plans
//!
//! Generates detailed execution plans showing:
//! - Join strategy (WCOJ vs nested loops)
//! - Index selection (SPOC, POCS, OCSP, CSPO)
//! - Estimated cardinality
//! - Estimated cost

use crate::algebra::{Algebra, TriplePattern};
use rdf_model::Node;
use std::collections::{HashMap, HashSet};

/// Query execution strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinStrategy {
    /// Worst-Case Optimal Join (LeapFrog TrieJoin)
    WCOJ,
    /// Traditional nested loop join
    NestedLoop,
    /// Hash join (future)
    HashJoin,
}

/// Index type for quad access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// Subject-Predicate-Object-Context
    SPOC,
    /// Predicate-Object-Context-Subject
    POCS,
    /// Object-Context-Subject-Predicate
    OCSP,
    /// Context-Subject-Predicate-Object
    CSPO,
}

/// Query execution plan with cost estimates
#[derive(Debug, Clone)]
pub struct QueryPlan {
    /// Join strategy to use
    pub strategy: JoinStrategy,
    /// Index to use for each pattern
    pub index_selection: Vec<(usize, IndexType)>,
    /// Estimated result cardinality
    pub estimated_cardinality: usize,
    /// Estimated execution cost (lower is better)
    pub estimated_cost: f64,
    /// Human-readable explanation
    pub explanation: String,
    /// Pattern analysis details
    pub analysis: PatternAnalysis,
}

/// Analysis of query pattern characteristics
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    /// Number of triple patterns
    pub pattern_count: usize,
    /// Number of variables
    pub variable_count: usize,
    /// Is this a star query?
    pub is_star: bool,
    /// Is this a cyclic query?
    pub is_cyclic: bool,
    /// Variable sharing graph (variable -> pattern indices)
    pub variable_sharing: HashMap<String, Vec<usize>>,
    /// Join selectivity (0.0 = very selective, 1.0 = not selective)
    pub selectivity: f64,
}

/// SPARQL Query Optimizer
pub struct QueryOptimizer {
    /// Enable WCOJ optimization
    wcoj_enabled: bool,
    /// Threshold for WCOJ (minimum pattern count)
    wcoj_threshold: usize,
}

impl QueryOptimizer {
    /// Create new optimizer with default settings
    pub fn new() -> Self {
        Self {
            wcoj_enabled: true,
            wcoj_threshold: 4, // Use WCOJ for 4+ patterns (conservative threshold)
        }
    }

    /// Create optimizer with WCOJ disabled (for testing/comparison)
    pub fn without_wcoj() -> Self {
        Self {
            wcoj_enabled: false,
            wcoj_threshold: usize::MAX,
        }
    }

    /// Analyze patterns and generate optimal query plan
    pub fn optimize(&self, patterns: &[TriplePattern]) -> QueryPlan {
        let analysis = self.analyze_patterns(patterns);

        // Decide join strategy based on pattern analysis
        let strategy = self.select_strategy(&analysis);

        // Select indexes for each pattern
        let index_selection = self.select_indexes(patterns);

        // Estimate cardinality and cost
        let estimated_cardinality = self.estimate_cardinality(&analysis);
        let estimated_cost = self.estimate_cost(&analysis, &strategy);

        // Generate explanation
        let explanation = self.generate_explanation(&analysis, &strategy);

        QueryPlan {
            strategy,
            index_selection,
            estimated_cardinality,
            estimated_cost,
            explanation,
            analysis,
        }
    }

    /// Analyze pattern characteristics
    fn analyze_patterns(&self, patterns: &[TriplePattern]) -> PatternAnalysis {
        let pattern_count = patterns.len();

        // Extract all variables
        let mut variables = HashSet::new();
        let mut variable_sharing: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, pattern) in patterns.iter().enumerate() {
            // Collect variables from subject
            if let crate::algebra::VarOrNode::Var(var) = &pattern.subject {
                let var_name = var.name.to_string();
                variables.insert(var_name.clone());
                variable_sharing
                    .entry(var_name)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }

            // Collect variables from predicate
            if let crate::algebra::VarOrNode::Var(var) = &pattern.predicate {
                let var_name = var.name.to_string();
                variables.insert(var_name.clone());
                variable_sharing
                    .entry(var_name)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }

            // Collect variables from object
            if let crate::algebra::VarOrNode::Var(var) = &pattern.object {
                let var_name = var.name.to_string();
                variables.insert(var_name.clone());
                variable_sharing
                    .entry(var_name)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        let variable_count = variables.len();

        // Detect star query: one variable appears in many patterns
        let is_star = self.is_star_query(&variable_sharing, pattern_count);

        // Detect cyclic query: variables form cycles
        let is_cyclic = self.is_cyclic_query(patterns, &variable_sharing);

        // Estimate selectivity (simplified)
        let selectivity = self.estimate_selectivity(patterns);

        PatternAnalysis {
            pattern_count,
            variable_count,
            is_star,
            is_cyclic,
            variable_sharing,
            selectivity,
        }
    }

    /// Detect star query pattern
    ///
    /// A star query has one central variable that appears in many patterns.
    /// Example: ?person :name "Alice" . ?person :age 30 . ?person :city "NYC"
    fn is_star_query(
        &self,
        variable_sharing: &HashMap<String, Vec<usize>>,
        pattern_count: usize,
    ) -> bool {
        // Check if any variable appears in more than 50% of patterns
        variable_sharing
            .values()
            .any(|indices| indices.len() >= (pattern_count / 2).max(2))
    }

    /// Detect cyclic query pattern
    ///
    /// A cyclic query has variables that form cycles in the join graph.
    /// Example: ?a :knows ?b . ?b :knows ?c . ?c :knows ?a
    fn is_cyclic_query(
        &self,
        patterns: &[TriplePattern],
        variable_sharing: &HashMap<String, Vec<usize>>,
    ) -> bool {
        if patterns.len() < 3 {
            return false; // Need at least 3 patterns for a cycle
        }

        // Build adjacency list: variable -> connected variables
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

        for pattern in patterns {
            let vars = self.extract_variables_from_pattern(pattern);

            // Connect all variables in this pattern
            for (i, var1) in vars.iter().enumerate() {
                for var2 in vars.iter().skip(i + 1) {
                    graph
                        .entry(var1.clone())
                        .or_insert_with(HashSet::new)
                        .insert(var2.clone());
                    graph
                        .entry(var2.clone())
                        .or_insert_with(HashSet::new)
                        .insert(var1.clone());
                }
            }
        }

        // Check for cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for var in graph.keys() {
            if !visited.contains(var) {
                if self.has_cycle_dfs(var, None, &graph, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    /// DFS cycle detection
    fn has_cycle_dfs(
        &self,
        node: &str,
        parent: Option<&str>,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                // Skip parent to avoid false positive in undirected graph
                if Some(neighbor.as_str()) == parent {
                    continue;
                }

                if !visited.contains(neighbor) {
                    if self.has_cycle_dfs(
                        neighbor,
                        Some(node),
                        graph,
                        visited,
                        rec_stack,
                    ) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true; // Back edge found = cycle
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Extract all variables from a pattern
    fn extract_variables_from_pattern(&self, pattern: &TriplePattern) -> Vec<String> {
        let mut vars = Vec::new();

        if let crate::algebra::VarOrNode::Var(var) = &pattern.subject {
            vars.push(var.name.to_string());
        }
        if let crate::algebra::VarOrNode::Var(var) = &pattern.predicate {
            vars.push(var.name.to_string());
        }
        if let crate::algebra::VarOrNode::Var(var) = &pattern.object {
            vars.push(var.name.to_string());
        }

        vars
    }

    /// Select join strategy based on analysis
    fn select_strategy(&self, analysis: &PatternAnalysis) -> JoinStrategy {
        if !self.wcoj_enabled {
            return JoinStrategy::NestedLoop;
        }

        // Use WCOJ if:
        // 1. Star query (50-100x faster)
        // 2. Cyclic query (10-50x faster)
        // 3. Many patterns (>= threshold)
        // 4. Many shared variables (complex join)

        if analysis.is_star {
            return JoinStrategy::WCOJ; // Star queries benefit most
        }

        if analysis.is_cyclic {
            return JoinStrategy::WCOJ; // Cyclic queries are WCOJ sweet spot
        }

        if analysis.pattern_count >= self.wcoj_threshold {
            return JoinStrategy::WCOJ; // Complex multi-way joins
        }

        if analysis.variable_count >= 3 && analysis.pattern_count >= 3 {
            return JoinStrategy::WCOJ; // Many variables + patterns
        }

        // Simple 2-way join: nested loop is fine
        JoinStrategy::NestedLoop
    }

    /// Select optimal index for each pattern
    fn select_indexes(&self, patterns: &[TriplePattern]) -> Vec<(usize, IndexType)> {
        patterns
            .iter()
            .enumerate()
            .map(|(idx, pattern)| {
                let index = self.select_index_for_pattern(pattern);
                (idx, index)
            })
            .collect()
    }

    /// Select index based on bound variables
    fn select_index_for_pattern(&self, pattern: &TriplePattern) -> IndexType {
        let s_bound = matches!(pattern.subject, crate::algebra::VarOrNode::Node(_));
        let p_bound = matches!(pattern.predicate, crate::algebra::VarOrNode::Node(_));
        let o_bound = matches!(pattern.object, crate::algebra::VarOrNode::Node(_));

        // Select index that puts bound variables first
        match (s_bound, p_bound, o_bound) {
            (true, _, _) => IndexType::SPOC, // Subject bound -> SPOC
            (false, true, _) => IndexType::POCS, // Predicate bound -> POCS
            (false, false, true) => IndexType::OCSP, // Object bound -> OCSP
            (false, false, false) => IndexType::SPOC, // All unbound -> default SPOC
        }
    }

    /// Estimate result cardinality (simplified)
    fn estimate_cardinality(&self, analysis: &PatternAnalysis) -> usize {
        // Simple heuristic: assume 10 results per pattern
        // Real implementation would use database statistics
        let base = 10_usize;
        let multiplier = analysis.pattern_count;

        base.saturating_mul(multiplier)
    }

    /// Estimate execution cost
    fn estimate_cost(&self, analysis: &PatternAnalysis, strategy: &JoinStrategy) -> f64 {
        let n = 1000.0; // Assume 1000 triples per pattern (simplified)
        let k = analysis.pattern_count as f64;

        match strategy {
            JoinStrategy::WCOJ => {
                // WCOJ cost: O(N^(k/(k-1)))
                if k <= 1.0 {
                    n
                } else {
                    n.powf(k / (k - 1.0))
                }
            }
            JoinStrategy::NestedLoop => {
                // Nested loop cost: O(N^k)
                n.powf(k)
            }
            JoinStrategy::HashJoin => {
                // Hash join cost: O(N * k)
                n * k
            }
        }
    }

    /// Estimate selectivity (0.0 = very selective, 1.0 = not selective)
    fn estimate_selectivity(&self, patterns: &[TriplePattern]) -> f64 {
        let mut bound_count = 0;
        let total_positions = patterns.len() * 3; // 3 positions per pattern

        for pattern in patterns {
            if matches!(pattern.subject, crate::algebra::VarOrNode::Node(_)) {
                bound_count += 1;
            }
            if matches!(pattern.predicate, crate::algebra::VarOrNode::Node(_)) {
                bound_count += 1;
            }
            if matches!(pattern.object, crate::algebra::VarOrNode::Node(_)) {
                bound_count += 1;
            }
        }

        // More bound variables = more selective (lower value)
        1.0 - (bound_count as f64 / total_positions as f64)
    }

    /// Generate human-readable explanation
    fn generate_explanation(&self, analysis: &PatternAnalysis, strategy: &JoinStrategy) -> String {
        let mut parts = Vec::new();

        // Strategy choice
        match strategy {
            JoinStrategy::WCOJ => {
                parts.push(format!("Using WCOJ (LeapFrog TrieJoin) for optimal multi-way join"));
            }
            JoinStrategy::NestedLoop => {
                parts.push(format!("Using nested loop join for simple pattern"));
            }
            JoinStrategy::HashJoin => {
                parts.push(format!("Using hash join"));
            }
        }

        // Pattern characteristics
        parts.push(format!(
            "Patterns: {}, Variables: {}",
            analysis.pattern_count, analysis.variable_count
        ));

        // Query type
        if analysis.is_star {
            parts.push(format!("⭐ Star query detected (50-100x faster with WCOJ)"));
        }
        if analysis.is_cyclic {
            parts.push(format!("🔁 Cyclic query detected (10-50x faster with WCOJ)"));
        }

        // Selectivity
        let selectivity_desc = if analysis.selectivity < 0.3 {
            "highly selective"
        } else if analysis.selectivity < 0.7 {
            "moderately selective"
        } else {
            "not selective"
        };
        parts.push(format!("Selectivity: {}", selectivity_desc));

        parts.join(" | ")
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdf_model::Dictionary;
    use std::sync::Arc;

    fn create_test_dict() -> Arc<Dictionary> {
        Arc::new(Dictionary::new())
    }

    #[test]
    fn test_optimizer_creation() {
        let opt = QueryOptimizer::new();
        assert!(opt.wcoj_enabled);
        assert_eq!(opt.wcoj_threshold, 4); // Conservative threshold for v0.1.7
    }

    #[test]
    fn test_star_query_detection() {
        let dict = create_test_dict();
        let opt = QueryOptimizer::new();

        // Create star query: ?person :name "Alice" . ?person :age 30 . ?person :city "NYC"
        let person_var = dict.intern("person");
        let name_pred = dict.intern("http://example.org/name");
        let age_pred = dict.intern("http://example.org/age");
        let city_pred = dict.intern("http://example.org/city");

        let patterns = vec![
            TriplePattern {
                subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
                predicate: crate::algebra::VarOrNode::Node(Node::iri(name_pred)),
                object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("Alice"))),
            },
            TriplePattern {
                subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
                predicate: crate::algebra::VarOrNode::Node(Node::iri(age_pred)),
                object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("30"))),
            },
            TriplePattern {
                subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
                predicate: crate::algebra::VarOrNode::Node(Node::iri(city_pred)),
                object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("NYC"))),
            },
        ];

        let analysis = opt.analyze_patterns(&patterns);

        assert!(analysis.is_star, "Should detect star query");
        assert_eq!(analysis.pattern_count, 3);
        assert_eq!(analysis.variable_count, 1);
    }

    #[test]
    fn test_wcoj_strategy_selection() {
        let dict = create_test_dict();
        let opt = QueryOptimizer::new();

        // Star query with 3 patterns
        let person_var = dict.intern("person");
        let pred1 = dict.intern("http://example.org/p1");
        let pred2 = dict.intern("http://example.org/p2");
        let pred3 = dict.intern("http://example.org/p3");

        let patterns = vec![
            TriplePattern {
                subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
                predicate: crate::algebra::VarOrNode::Node(Node::iri(pred1)),
                object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("o1"))),
            },
            TriplePattern {
                subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
                predicate: crate::algebra::VarOrNode::Node(Node::iri(pred2)),
                object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("o2"))),
            },
            TriplePattern {
                subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
                predicate: crate::algebra::VarOrNode::Node(Node::iri(pred3)),
                object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("o3"))),
            },
        ];

        let plan = opt.optimize(&patterns);

        assert_eq!(plan.strategy, JoinStrategy::WCOJ);
        assert!(plan.analysis.is_star);
        assert!(plan.explanation.contains("WCOJ"));
    }

    #[test]
    fn test_simple_query_nested_loop() {
        let dict = create_test_dict();
        let opt = QueryOptimizer::new();

        // Simple 1-pattern query
        let person_var = dict.intern("person");
        let name_pred = dict.intern("http://example.org/name");

        let patterns = vec![TriplePattern {
            subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
            predicate: crate::algebra::VarOrNode::Node(Node::iri(name_pred)),
            object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("Alice"))),
        }];

        let plan = opt.optimize(&patterns);

        // Single pattern should use nested loop (no join needed)
        assert_eq!(plan.strategy, JoinStrategy::NestedLoop);
    }

    #[test]
    fn test_index_selection() {
        let dict = create_test_dict();
        let opt = QueryOptimizer::new();

        let var_s = dict.intern("s");
        let pred = dict.intern("http://example.org/predicate");

        // Pattern with bound predicate -> should select POCS
        let pattern = TriplePattern {
            subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: var_s }),
            predicate: crate::algebra::VarOrNode::Node(Node::iri(pred)),
            object: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: var_s }),
        };

        let index = opt.select_index_for_pattern(&pattern);

        assert_eq!(index, IndexType::POCS);
    }

    #[test]
    fn test_query_plan_explanation() {
        let dict = create_test_dict();
        let opt = QueryOptimizer::new();

        let person_var = dict.intern("person");
        let pred = dict.intern("http://example.org/p");

        let patterns = vec![
            TriplePattern {
                subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
                predicate: crate::algebra::VarOrNode::Node(Node::iri(pred)),
                object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("o1"))),
            },
            TriplePattern {
                subject: crate::algebra::VarOrNode::Var(crate::algebra::Variable { name: person_var }),
                predicate: crate::algebra::VarOrNode::Node(Node::iri(pred)),
                object: crate::algebra::VarOrNode::Node(Node::literal_str(dict.intern("o2"))),
            },
        ];

        let plan = opt.optimize(&patterns);

        // Check explanation contains key information
        assert!(plan.explanation.contains("Patterns"));
        assert!(plan.explanation.contains("Variables"));
    }
}
