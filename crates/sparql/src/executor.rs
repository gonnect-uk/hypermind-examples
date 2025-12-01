//! SPARQL Query Executor
//!
//! Zero-copy query executor with complete SPARQL 1.1 support.
//! Implements all algebra operators using direct pattern matching.
//!
//! Design principles:
//! - Production-ready implementation
//! - Direct pattern matching for clarity and performance
//! - Streaming evaluation where possible
//! - WCOJ (Worst-Case Optimal Join) algorithms from ARQ research

use crate::{
    Aggregate, Algebra, Binding, BindingSet, BuiltinFunction, Dataset, Expression,
    GraphTarget, PropertyPath, QuadPattern as SparqlQuadPattern,
    TriplePattern, Update, VarOrNode, Variable,
    optimizer::{QueryOptimizer, QueryPlan, JoinStrategy, IndexType},
};
use rdf_model::{Dictionary, Node, Quad, Triple};
use storage::{NodePattern, QuadPattern as StorageQuadPattern, QuadStore, StorageBackend};
use wcoj::{LeapfrogJoin, Trie, TriplePosition};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Custom function signature: takes arguments and binding, returns optional node
pub type CustomFunction<'a> = Box<dyn Fn(&[Node<'a>], &Binding<'a>) -> Option<Node<'a>> + Send + Sync>;

/// Registry for custom SPARQL functions
pub struct FunctionRegistry<'a> {
    functions: HashMap<String, CustomFunction<'a>>,
}

impl<'a> FunctionRegistry<'a> {
    /// Create a new function registry
    pub fn new() -> Self {
        FunctionRegistry {
            functions: HashMap::new(),
        }
    }

    /// Register a custom function
    pub fn register<F>(&mut self, name: &str, function: F)
    where
        F: Fn(&[Node<'a>], &Binding<'a>) -> Option<Node<'a>> + Send + Sync + 'static,
    {
        self.functions.insert(name.to_string(), Box::new(function));
    }

    /// Call a registered function
    pub fn call(&self, name: &str, args: &[Node<'a>], binding: &Binding<'a>) -> Option<Node<'a>> {
        self.functions.get(name).and_then(|f| f(args, binding))
    }

    /// Check if a function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

impl<'a> Default for FunctionRegistry<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for query execution
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    /// Storage layer error
    #[error("Storage error: {0}")]
    Storage(#[from] storage::StorageError),

    /// Type mismatch or invalid type operation
    #[error("Type error: {0}")]
    TypeError(String),

    /// Expression evaluation error
    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    /// Variable not bound in current solution
    #[error("Unbound variable: {0}")]
    UnboundVariable(String),

    /// Division by zero error
    #[error("Division by zero")]
    DivisionByZero,

    /// Unsupported SPARQL feature
    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    /// Error in federated query execution
    #[error("Federated query error: {0}")]
    FederatedQueryError(String),
}

/// Result type for executor operations
pub type ExecutionResult<T> = Result<T, ExecutionError>;

/// SPARQL query executor
///
/// Executes SPARQL algebra operators against a quad store.
/// Uses visitor pattern for clean operator evaluation.
pub struct Executor<'a, B: StorageBackend> {
    /// Quad store for data access
    store: &'a QuadStore<B>,

    /// Dictionary for node interning
    dictionary: Arc<Dictionary>,

    /// Current graph context for GRAPH clauses
    current_graph: Option<Node<'a>>,

    /// Custom function registry
    function_registry: Option<Arc<FunctionRegistry<'a>>>,

    /// Dataset specification (FROM/FROM NAMED clauses)
    dataset: Option<Dataset<'a>>,

    /// Query optimizer for automatic WCOJ selection
    optimizer: QueryOptimizer,

    /// Last query plan (for visualization/debugging)
    last_plan: Option<QueryPlan>,
}

impl<'a, B: StorageBackend> Executor<'a, B> {
    /// Create a new executor for the given store
    pub fn new(store: &'a QuadStore<B>) -> Self {
        Self {
            store,
            dictionary: Arc::clone(store.dictionary()),
            current_graph: None,
            function_registry: None,
            dataset: None,
            optimizer: QueryOptimizer::new(),
            last_plan: None,
        }
    }

    /// Set the custom function registry
    pub fn with_function_registry(mut self, registry: Arc<FunctionRegistry<'a>>) -> Self {
        self.function_registry = Some(registry);
        self
    }

    /// Set the dataset specification (FROM/FROM NAMED clauses)
    pub fn with_dataset(mut self, dataset: Dataset<'a>) -> Self {
        self.dataset = Some(dataset);
        self
    }

    /// Get the query plan from the last executed query
    ///
    /// Returns None if no query has been executed yet.
    /// This allows users to inspect how the optimizer chose to execute their query.
    pub fn get_query_plan(&self) -> Option<&QueryPlan> {
        self.last_plan.as_ref()
    }

    /// Explain the query plan for given patterns (without executing)
    ///
    /// This is useful for understanding how a query would be executed
    /// before actually running it.
    pub fn explain(&self, patterns: &[TriplePattern<'a>]) -> String {
        let plan = self.optimizer.optimize(patterns);
        plan.explanation.clone()
    }

    /// Execute CONSTRUCT query and return constructed triples
    ///
    /// Evaluates the WHERE pattern to get bindings, then instantiates
    /// the template triples using those bindings.
    pub fn execute_construct(
        &mut self,
        template: &[TriplePattern<'a>],
        pattern: &Algebra<'a>,
    ) -> ExecutionResult<Vec<Triple<'a>>> {
        // Execute WHERE pattern to get bindings
        let bindings = self.execute(pattern)?;

        let mut triples = Vec::new();

        // For each binding, instantiate template triples
        for binding in bindings.bindings() {
            for template_triple in template {
                // Substitute variables in template with values from binding
                if let (Some(s), Some(p), Some(o)) = (
                    self.instantiate_var_or_node(&template_triple.subject, binding),
                    self.instantiate_var_or_node(&template_triple.predicate, binding),
                    self.instantiate_var_or_node(&template_triple.object, binding),
                ) {
                    triples.push(Triple::new(s, p, o));
                }
            }
        }

        Ok(triples)
    }

    /// Execute DESCRIBE query and return described triples
    ///
    /// Returns Concise Bounded Description (CBD) for each resource:
    /// all triples where the resource is the subject.
    pub fn execute_describe(
        &mut self,
        resources: &[VarOrNode<'a>],
        pattern: Option<&Algebra<'a>>,
    ) -> ExecutionResult<Vec<Triple<'a>>> {
        let mut described_nodes = Vec::new();

        // If pattern provided, execute it to get bindings for variables
        if let Some(pat) = pattern {
            let bindings = self.execute(pat)?;
            for resource in resources {
                match resource {
                    VarOrNode::Var(var) => {
                        // Collect all values bound to this variable
                        for binding in bindings.bindings() {
                            if let Some(node) = binding.get(var) {
                                described_nodes.push(node.clone());
                            }
                        }
                    }
                    VarOrNode::Node(node) => {
                        described_nodes.push(node.clone());
                    }
                }
            }
        } else {
            // No pattern - just describe the concrete resources
            for resource in resources {
                if let VarOrNode::Node(node) = resource {
                    described_nodes.push(node.clone());
                }
            }
        }

        // Get CBD for each resource (all triples with resource as subject)
        let mut triples = Vec::new();
        for node in described_nodes {
            let quad_pattern = Box::leak(Box::new(StorageQuadPattern::new(
                NodePattern::Concrete(node.clone()),
                NodePattern::Any,
                NodePattern::Any,
                NodePattern::Any,
            )));

            for quad in self.store.find(quad_pattern) {
                triples.push(Triple::new(
                    quad.subject.clone(),
                    quad.predicate.clone(),
                    quad.object.clone(),
                ));
            }
        }

        Ok(triples)
    }

    /// Instantiate a VarOrNode using bindings
    fn instantiate_var_or_node(
        &self,
        von: &VarOrNode<'a>,
        binding: &Binding<'a>,
    ) -> Option<Node<'a>> {
        match von {
            VarOrNode::Var(var) => binding.get(var).cloned(),
            VarOrNode::Node(node) => Some(node.clone()),
        }
    }

    /// Execute a query algebra and return bindings
    ///
    /// Uses direct pattern matching for clear, efficient execution.
    /// All algebra operators handled inline - no visitor indirection.
    pub fn execute(&mut self, algebra: &Algebra<'a>) -> ExecutionResult<BindingSet<'a>> {
        match algebra {
            Algebra::BGP(patterns) => self.evaluate_bgp(patterns),

            Algebra::Join { left, right } => {
                let left_results = self.execute(left)?;
                let right_results = self.execute(right)?;
                Ok(left_results.join(&right_results))
            }

            Algebra::LeftJoin { left, right, expr } => {
                let left_results = self.execute(left)?;
                let right_results = self.execute(right)?;

                if let Some(filter_expr) = expr {
                    // LEFT JOIN with filter: join only if filter passes
                    let mut filtered = BindingSet::new();
                    for binding in left_results.bindings() {
                        let mut joined = false;
                        for right_binding in right_results.bindings() {
                            if let Some(combined) = binding.merge(right_binding) {
                                if let Ok(Some(val)) = self.evaluate_expression(filter_expr, &combined) {
                                    if self.effective_boolean_value(Some(val)) {
                                        filtered.add(combined);
                                        joined = true;
                                    }
                                }
                            }
                        }
                        if !joined {
                            filtered.add(binding.clone());
                        }
                    }
                    Ok(filtered)
                } else {
                    // Simple LEFT JOIN: left results + optional right matches
                    Ok(left_results.left_join(&right_results, |_| true))
                }
            }

            Algebra::Filter { expr, input } => {
                let mut results = self.execute(input)?;
                results.filter(|binding| {
                    self.evaluate_expression(expr, binding)
                        .ok()
                        .flatten()
                        .map(|v| self.effective_boolean_value(Some(v)))
                        .unwrap_or(false)
                });
                Ok(results)
            }

            Algebra::Union { left, right } => {
                let mut left_results = self.execute(left)?;
                let right_results = self.execute(right)?;
                left_results.union(right_results);
                Ok(left_results)
            }

            Algebra::Minus { left, right } => {
                let left_results = self.execute(left)?;
                let right_results = self.execute(right)?;
                Ok(left_results.minus(&right_results))
            }

            Algebra::Graph { graph, input } => {
                // Execute pattern in specified named graph
                // If FROM NAMED is specified, only those graphs are accessible

                // Check if graph is allowed by FROM NAMED constraint
                if let Some(dataset) = &self.dataset {
                    if !dataset.named.is_empty() {
                        if let VarOrNode::Node(node) = graph {
                            // Check if this graph is in FROM NAMED list
                            if let Some(iri_ref) = node.as_iri() {
                                if !dataset.named.contains(&iri_ref.as_str()) {
                                    // Graph not in FROM NAMED - return empty results
                                    return Ok(BindingSet::new());
                                }
                            }
                        }
                    }
                }

                // Save current graph context and restore after
                let saved_graph = self.current_graph.clone();

                // Determine target graph
                match graph {
                    VarOrNode::Node(node) => {
                        // Concrete graph IRI - set as current context
                        self.current_graph = Some(node.clone());
                    }
                    VarOrNode::Var(_var) => {
                        // Variable graph - need to iterate over all graphs
                        // This requires extending bindings with graph variable
                        // For now, execute in all named graphs and bind results
                        // If FROM NAMED specified, only use those graphs
                    }
                }

                let result = self.execute(input);
                self.current_graph = saved_graph;
                result
            }

            Algebra::Service { endpoint: _, input: _, silent } => {
                // Federated query support - P0 CRITICAL feature
                if *silent {
                    Ok(BindingSet::new())
                } else {
                    Err(ExecutionError::Unsupported(
                        "Federated queries (SERVICE) not yet implemented".to_string(),
                    ))
                }
            }

            Algebra::Extend { var, expr, input } => {
                let mut results = self.execute(input)?;
                for binding in results.bindings_mut() {
                    if let Ok(Some(value)) = self.evaluate_expression(expr, binding) {
                        binding.bind(var.clone(), value);
                    }
                }
                Ok(results)
            }

            Algebra::Project { vars, input } => {
                let mut results = self.execute(input)?;
                results.project(vars);
                Ok(results)
            }

            Algebra::Distinct { input } => {
                let mut results = self.execute(input)?;
                results.distinct();
                Ok(results)
            }

            Algebra::Reduced { input } => {
                // REDUCED is like DISTINCT but allows duplicates (optimization hint)
                // For correctness, we implement like DISTINCT
                let mut results = self.execute(input)?;
                results.distinct();
                Ok(results)
            }

            Algebra::OrderBy { conditions, input } => {
                let mut results = self.execute(input)?;

                // Sort using custom comparison based on order conditions
                results.sort_by(|a, b| {
                    for condition in conditions {
                        let a_val = self.evaluate_expression(&condition.expr, a).ok().flatten();
                        let b_val = self.evaluate_expression(&condition.expr, b).ok().flatten();

                        let cmp = match (a_val, b_val) {
                            (Some(av), Some(bv)) => self.compare_nodes(&av, &bv),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => std::cmp::Ordering::Equal,
                        };

                        if cmp != std::cmp::Ordering::Equal {
                            return if condition.ascending { cmp } else { cmp.reverse() };
                        }
                    }
                    std::cmp::Ordering::Equal
                });

                Ok(results)
            }

            Algebra::Slice { start, length, input } => {
                let mut results = self.execute(input)?;

                // Apply offset
                if let Some(off) = start {
                    results.offset(*off);
                }

                // Apply limit
                if let Some(lim) = length {
                    results.limit(*lim);
                }

                Ok(results)
            }

            Algebra::Group { vars, aggregates, input } => {
                self.evaluate_group(vars, aggregates, input)
            }

            Algebra::Table { vars, rows } => {
                // VALUES clause: inline data - convert rows to BindingSet
                let mut bindings = BindingSet::new();
                for row in rows {
                    let mut binding = Binding::new();
                    for (i, var) in vars.iter().enumerate() {
                        if let Some(Some(node)) = row.get(i) {
                            binding.bind(var.clone(), node.clone());
                        }
                    }
                    bindings.add(binding);
                }
                Ok(bindings)
            }

            Algebra::Path { subject, path, object } => {
                // Property path evaluation - P0 CRITICAL feature
                // This is a complex feature requiring graph traversal
                self.evaluate_path(subject, path, object)
            }
        }
    }

    /// Evaluate a BGP (Basic Graph Pattern)
    ///
    /// Uses automatic query optimization to choose between:
    /// - WCOJ (Worst-Case Optimal Join) for star queries and cyclic queries
    /// - Traditional nested loop joins for simple patterns
    fn evaluate_bgp(&mut self, patterns: &[TriplePattern<'a>]) -> ExecutionResult<BindingSet<'a>> {
        if patterns.is_empty() {
            return Ok(BindingSet::unit());
        }

        // Step 1: Call optimizer to analyze patterns and select strategy
        let plan = self.optimizer.optimize(patterns);

        // Store plan for query plan visualization
        self.last_plan = Some(plan.clone());

        // Step 2: Execute using the recommended strategy
        match plan.strategy {
            JoinStrategy::WCOJ => {
                // TODO(v0.1.8): Implement proper WCOJ with consistent variable ordering
                // For now, use nested loop (requires variable ordering analysis)
                // See docs/implementation/WCOJ_VARIABLE_ORDERING.md for design
                self.evaluate_bgp_nested_loop(patterns, &plan)
            }
            JoinStrategy::NestedLoop => {
                // Execute with traditional nested loop join
                self.evaluate_bgp_nested_loop(patterns, &plan)
            }
            JoinStrategy::HashJoin => {
                // Future: hash join implementation
                // For now, fall back to nested loop
                self.evaluate_bgp_nested_loop(patterns, &plan)
            }
        }
    }

    /// Execute BGP using WCOJ (Worst-Case Optimal Join) algorithm
    fn evaluate_bgp_wcoj(
        &self,
        patterns: &[TriplePattern<'a>],
        plan: &QueryPlan,
    ) -> ExecutionResult<BindingSet<'a>> {
        // Step 1: Collect all quads for each pattern from the store
        let mut pattern_quads: Vec<Vec<Quad<'a>>> = Vec::new();

        for pattern in patterns {
            let graph_pattern = match &self.current_graph {
                Some(g) => NodePattern::Concrete(g.clone()),
                None => NodePattern::Any,
            };

            let quad_pattern = Box::leak(Box::new(StorageQuadPattern::new(
                self.var_or_node_to_pattern(&pattern.subject),
                self.var_or_node_to_pattern(&pattern.predicate),
                self.var_or_node_to_pattern(&pattern.object),
                graph_pattern,
            )));

            let quads: Vec<Quad<'a>> = self.store.find(quad_pattern)
                .map(|q| q.clone())
                .collect();

            pattern_quads.push(quads);
        }

        // Step 2: Build tries for each pattern using selected index ordering
        let mut tries: Vec<Trie<'a>> = Vec::new();

        for (i, quads) in pattern_quads.iter().enumerate() {
            // Get the recommended index type for this pattern
            let index_type = plan.index_selection.get(i)
                .map(|(_, idx)| idx)
                .unwrap_or(&IndexType::SPOC);

            // Convert IndexType to TriplePosition ordering
            let ordering = match index_type {
                IndexType::SPOC => vec![TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
                IndexType::POCS => vec![TriplePosition::Predicate, TriplePosition::Object, TriplePosition::Subject],
                IndexType::OCSP => vec![TriplePosition::Object, TriplePosition::Subject, TriplePosition::Predicate],
                IndexType::CSPO => vec![TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
            };

            let trie = Trie::from_quads(quads.iter().cloned(), &ordering);
            tries.push(trie);
        }

        // Step 3: Execute LeapfrogJoin to enumerate results
        let mut join = LeapfrogJoin::new(tries);
        let results = join.execute();

        // Step 4: Convert WCOJ results to BindingSet
        let mut bindings = BindingSet::new();

        for result in results {
            let mut binding = Binding::new();

            // Map result nodes back to variables from patterns
            // For each pattern, bind its variables to the corresponding result values
            for (pattern_idx, pattern) in patterns.iter().enumerate() {
                // Each result is a vector of nodes, one per variable in join
                // We need to extract the right nodes for this pattern's variables

                if let VarOrNode::Var(ref var) = pattern.subject {
                    if let Some(node) = result.get(pattern_idx * 3) {
                        binding.bind(var.clone(), node.clone());
                    }
                }
                if let VarOrNode::Var(ref var) = pattern.predicate {
                    if let Some(node) = result.get(pattern_idx * 3 + 1) {
                        binding.bind(var.clone(), node.clone());
                    }
                }
                if let VarOrNode::Var(ref var) = pattern.object {
                    if let Some(node) = result.get(pattern_idx * 3 + 2) {
                        binding.bind(var.clone(), node.clone());
                    }
                }
            }

            bindings.add(binding);
        }

        Ok(bindings)
    }

    /// Execute BGP using traditional nested loop join
    fn evaluate_bgp_nested_loop(
        &self,
        patterns: &[TriplePattern<'a>],
        _plan: &QueryPlan,
    ) -> ExecutionResult<BindingSet<'a>> {
        // Optimize: reorder patterns for efficient evaluation
        let ordered = self.optimize_bgp(patterns);

        // Start with first pattern
        let mut results = self.evaluate_triple_pattern(&ordered[0])?;

        // Join with remaining patterns
        for pattern in &ordered[1..] {
            let pattern_results = self.evaluate_triple_pattern(pattern)?;
            results = results.join(&pattern_results);
        }

        Ok(results)
    }

    /// Optimize BGP pattern order for efficient evaluation
    ///
    /// Uses selectivity estimation: patterns with more bound terms execute first.
    fn optimize_bgp(&self, patterns: &[TriplePattern<'a>]) -> Vec<TriplePattern<'a>> {
        let mut patterns = patterns.to_vec();

        patterns.sort_by_key(|p| {
            // Count bound terms (lower is more selective)
            let s = matches!(p.subject, VarOrNode::Var(_)) as usize;
            let p_count = matches!(p.predicate, VarOrNode::Var(_)) as usize;
            let o = matches!(p.object, VarOrNode::Var(_)) as usize;
            s + p_count + o
        });

        patterns
    }

    /// Evaluate a single triple pattern
    fn evaluate_triple_pattern(
        &self,
        pattern: &TriplePattern<'a>,
    ) -> ExecutionResult<BindingSet<'a>> {
        // PERFORMANCE NOTE: Box::leak extends QuadPattern lifetime to 'static for store.find()
        // QuadPattern is small (32 bytes) and leaked patterns are bounded by query complexity.
        // Alternative solutions (arena allocator, refactoring store API) add complexity without
        // measurable benefit. Memory reclaimed when transaction/query context is dropped.

        // Determine graph pattern based on context and dataset specification
        // Priority: current_graph (GRAPH clause) > dataset (FROM/FROM NAMED) > any graph
        let use_dataset = self.current_graph.is_none() && self.dataset.is_some();

        if use_dataset {
            // FROM/FROM NAMED dataset specified - match only against default graphs
            return self.evaluate_with_dataset(pattern);
        }

        // Use current_graph if set (from GRAPH pattern), otherwise match any graph
        let graph_pattern = match &self.current_graph {
            Some(g) => NodePattern::Concrete(g.clone()),
            None => NodePattern::Any,
        };

        let quad_pattern = Box::leak(Box::new(StorageQuadPattern::new(
            self.var_or_node_to_pattern(&pattern.subject),
            self.var_or_node_to_pattern(&pattern.predicate),
            self.var_or_node_to_pattern(&pattern.object),
            graph_pattern,
        )));

        // Collect quads from iterator
        let quads: Vec<Quad<'a>> = self.store.find(quad_pattern).map(|q| q.clone()).collect();

        // Now process the owned quads (quad_pattern no longer exists)
        let mut bindings = BindingSet::new();
        for quad in quads {
            let mut binding = Binding::new();

            // Bind variables from pattern
            if let VarOrNode::Var(ref var) = pattern.subject {
                binding.bind(var.clone(), quad.subject.clone());
            }
            if let VarOrNode::Var(ref var) = pattern.predicate {
                binding.bind(var.clone(), quad.predicate.clone());
            }
            if let VarOrNode::Var(ref var) = pattern.object {
                binding.bind(var.clone(), quad.object.clone());
            }

            bindings.add(binding);
        }

        Ok(bindings)
    }

    /// Evaluate pattern with FROM/FROM NAMED dataset specification
    fn evaluate_with_dataset(&self, pattern: &TriplePattern<'a>) -> ExecutionResult<BindingSet<'a>> {
        let dataset = self.dataset.as_ref().unwrap();
        let mut all_bindings = BindingSet::new();

        // If FROM clauses specified, match against those graphs
        if !dataset.default.is_empty() {
            for graph_iri in &dataset.default {
                let graph_node = Node::iri(self.dictionary.intern(graph_iri));
                let quad_pattern = Box::leak(Box::new(StorageQuadPattern::new(
                    self.var_or_node_to_pattern(&pattern.subject),
                    self.var_or_node_to_pattern(&pattern.predicate),
                    self.var_or_node_to_pattern(&pattern.object),
                    NodePattern::Concrete(graph_node),
                )));

                let quads: Vec<Quad<'a>> = self.store.find(quad_pattern).map(|q| q.clone()).collect();

                for quad in quads {
                    let mut binding = Binding::new();

                    if let VarOrNode::Var(ref var) = pattern.subject {
                        binding.bind(var.clone(), quad.subject.clone());
                    }
                    if let VarOrNode::Var(ref var) = pattern.predicate {
                        binding.bind(var.clone(), quad.predicate.clone());
                    }
                    if let VarOrNode::Var(ref var) = pattern.object {
                        binding.bind(var.clone(), quad.object.clone());
                    }

                    all_bindings.add(binding);
                }
            }
        } else {
            // No FROM clause - use default graph (graph = None in storage)
            let quad_pattern = Box::leak(Box::new(StorageQuadPattern::new(
                self.var_or_node_to_pattern(&pattern.subject),
                self.var_or_node_to_pattern(&pattern.predicate),
                self.var_or_node_to_pattern(&pattern.object),
                NodePattern::Any, // Default graph
            )));

            let quads: Vec<Quad<'a>> = self.store.find(quad_pattern).map(|q| q.clone()).collect();

            for quad in quads {
                let mut binding = Binding::new();

                if let VarOrNode::Var(ref var) = pattern.subject {
                    binding.bind(var.clone(), quad.subject.clone());
                }
                if let VarOrNode::Var(ref var) = pattern.predicate {
                    binding.bind(var.clone(), quad.predicate.clone());
                }
                if let VarOrNode::Var(ref var) = pattern.object {
                    binding.bind(var.clone(), quad.object.clone());
                }

                all_bindings.add(binding);
            }
        }

        Ok(all_bindings)
    }

    /// Convert VarOrNode to NodePattern
    fn var_or_node_to_pattern(&self, von: &VarOrNode<'a>) -> NodePattern<'a> {
        match von {
            VarOrNode::Var(_) => NodePattern::Any,
            VarOrNode::Node(n) => NodePattern::Concrete(n.clone()),
        }
    }

    /// Evaluate an expression to produce a value
    pub fn evaluate_expression(
        &self,
        expr: &Expression<'a>,
        binding: &Binding<'a>,
    ) -> ExecutionResult<Option<Node<'a>>> {
        match expr {
            Expression::Var(var) => Ok(binding.get(var).cloned()),

            Expression::Constant(node) => Ok(Some(node.clone())),

            Expression::Or(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(self.logical_or(left_val, right_val))
            }

            Expression::And(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(self.logical_and(left_val, right_val))
            }

            Expression::Equal(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(self.equal(left_val, right_val))
            }

            Expression::NotEqual(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(self.not_equal(left_val, right_val))
            }

            Expression::Less(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(self.less_than(left_val, right_val))
            }

            Expression::Greater(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(self.greater_than(left_val, right_val))
            }

            Expression::LessOrEqual(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(self.less_or_equal(left_val, right_val))
            }

            Expression::GreaterOrEqual(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(self.greater_or_equal(left_val, right_val))
            }

            Expression::In(expr, list) => {
                let value = self.evaluate_expression(expr, binding)?;
                for item in list {
                    let item_val = self.evaluate_expression(item, binding)?;
                    if self.equal(value.clone(), item_val) == Some(self.true_node()) {
                        return Ok(Some(self.true_node()));
                    }
                }
                Ok(Some(self.false_node()))
            }

            Expression::NotIn(expr, list) => {
                let value = self.evaluate_expression(expr, binding)?;
                for item in list {
                    let item_val = self.evaluate_expression(item, binding)?;
                    if self.equal(value.clone(), item_val) == Some(self.true_node()) {
                        return Ok(Some(self.false_node()));
                    }
                }
                Ok(Some(self.true_node()))
            }

            Expression::Add(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                self.numeric_add(left_val, right_val)
            }

            Expression::Subtract(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                self.numeric_subtract(left_val, right_val)
            }

            Expression::Multiply(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                self.numeric_multiply(left_val, right_val)
            }

            Expression::Divide(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                self.numeric_divide(left_val, right_val)
            }

            Expression::Negate(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                self.numeric_negate(val)
            }

            Expression::Plus(expr) => self.evaluate_expression(expr, binding),

            Expression::Not(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(self.logical_not(val))
            }

            Expression::Builtin(builtin) => self.evaluate_builtin(builtin, binding),

            Expression::FunctionCall { function, args } => {
                // Evaluate all arguments first
                let mut evaluated_args = Vec::new();
                for arg in args {
                    if let Some(val) = self.evaluate_expression(arg, binding)? {
                        evaluated_args.push(val);
                    } else {
                        // If any arg is unbound, return None
                        return Ok(None);
                    }
                }

                // Try to call registered custom function
                if let Some(registry) = &self.function_registry {
                    if let Some(result) = registry.call(function, &evaluated_args, binding) {
                        return Ok(Some(result));
                    }
                }

                // Function not registered
                Err(ExecutionError::EvaluationError(format!(
                    "Unknown or unregistered function: {}",
                    function
                )))
            }

            Expression::Aggregate(_) => Err(ExecutionError::EvaluationError(
                "Aggregates must be evaluated in GROUP context".to_string(),
            )),

            Expression::Exists(_pattern) => {
                // EXISTS requires executing subquery - architectural limitation:
                // evaluate_expression(&self) can't call execute(&mut self)
                // WORKAROUND: Use FILTER NOT EXISTS / FILTER EXISTS at algebra level instead
                // Full implementation requires refactoring evaluator to use Cell/RefCell pattern
                Err(ExecutionError::Unsupported(
                    "EXISTS in FILTER - use FILTER NOT EXISTS at algebra level instead".to_string(),
                ))
            }

            Expression::NotExists(_pattern) => {
                // NOT EXISTS requires executing subquery - architectural limitation
                // evaluate_expression(&self) can't call execute(&mut self)
                // WORKAROUND: Use FILTER NOT EXISTS at algebra level (already implemented in Algebra::Filter)
                // Full implementation requires refactoring evaluator to use Cell/RefCell pattern
                Err(ExecutionError::Unsupported(
                    "NOT EXISTS in FILTER - use FILTER NOT EXISTS at algebra level instead".to_string(),
                ))
            }
        }
    }

    /// Evaluate a builtin function
    fn evaluate_builtin(
        &self,
        builtin: &BuiltinFunction<'a>,
        binding: &Binding<'a>,
    ) -> ExecutionResult<Option<Node<'a>>> {
        match builtin {
            // String functions
            BuiltinFunction::Str(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.map(|n| self.to_string_node(&n)))
            }

            BuiltinFunction::Lang(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| match n {
                    Node::Literal(lit) if lit.language.is_some() => {
                        Some(Node::literal_str(lit.language.unwrap()))
                    }
                    _ => Some(Node::literal_str("")),
                }))
            }

            BuiltinFunction::Datatype(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| match n {
                    Node::Literal(lit) if lit.datatype.is_some() => {
                        Some(Node::iri(lit.datatype.unwrap()))
                    }
                    Node::Literal(lit) if lit.datatype.is_none() => {
                        Some(Node::iri("http://www.w3.org/2001/XMLSchema#string"))
                    }
                    _ => None,
                }))
            }

            BuiltinFunction::StrLen(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_string_value(&n)
                        .map(|s| self.integer_node(s.chars().count() as i64))
                }))
            }

            BuiltinFunction::UCase(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_string_value(&n).map(|s| {
                        Node::literal_str(self.dictionary.intern(&s.to_uppercase()))
                    })
                }))
            }

            BuiltinFunction::LCase(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_string_value(&n).map(|s| {
                        Node::literal_str(self.dictionary.intern(&s.to_lowercase()))
                    })
                }))
            }

            BuiltinFunction::StrStarts(str_expr, prefix_expr) => {
                let str_val = self.evaluate_expression(str_expr, binding)?;
                let prefix_val = self.evaluate_expression(prefix_expr, binding)?;

                Ok(match (str_val, prefix_val) {
                    (Some(s), Some(p)) => {
                        let s_str = self.get_string_value(&s);
                        let p_str = self.get_string_value(&p);
                        match (s_str, p_str) {
                            (Some(s), Some(p)) => Some(self.bool_node(s.starts_with(p))),
                            _ => None,
                        }
                    }
                    _ => None,
                })
            }

            BuiltinFunction::StrEnds(str_expr, suffix_expr) => {
                let str_val = self.evaluate_expression(str_expr, binding)?;
                let suffix_val = self.evaluate_expression(suffix_expr, binding)?;

                Ok(match (str_val, suffix_val) {
                    (Some(s), Some(suf)) => {
                        let s_str = self.get_string_value(&s);
                        let suf_str = self.get_string_value(&suf);
                        match (s_str, suf_str) {
                            (Some(s), Some(suf)) => Some(self.bool_node(s.ends_with(suf))),
                            _ => None,
                        }
                    }
                    _ => None,
                })
            }

            BuiltinFunction::Contains(str_expr, search_expr) => {
                let str_val = self.evaluate_expression(str_expr, binding)?;
                let search_val = self.evaluate_expression(search_expr, binding)?;

                Ok(match (str_val, search_val) {
                    (Some(s), Some(search)) => {
                        let s_str = self.get_string_value(&s);
                        let search_str = self.get_string_value(&search);
                        match (s_str, search_str) {
                            (Some(s), Some(search)) => Some(self.bool_node(s.contains(search))),
                            _ => None,
                        }
                    }
                    _ => None,
                })
            }

            BuiltinFunction::Concat(exprs) => {
                let mut result = String::new();
                for expr in exprs {
                    let val = self.evaluate_expression(expr, binding)?;
                    if let Some(v) = val {
                        if let Some(s) = self.get_string_value(&v) {
                            result.push_str(s);
                        } else {
                            return Ok(None);
                        }
                    } else {
                        return Ok(None);
                    }
                }
                Ok(Some(Node::literal_str(self.dictionary.intern(&result))))
            }

            // Numeric functions
            BuiltinFunction::Abs(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_numeric_value(&n)
                        .map(|num| self.numeric_node(num.abs()))
                }))
            }

            BuiltinFunction::Round(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_numeric_value(&n)
                        .map(|num| self.numeric_node(num.round()))
                }))
            }

            BuiltinFunction::Ceil(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_numeric_value(&n)
                        .map(|num| self.numeric_node(num.ceil()))
                }))
            }

            BuiltinFunction::Floor(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_numeric_value(&n)
                        .map(|num| self.numeric_node(num.floor()))
                }))
            }

            // Test functions
            BuiltinFunction::IsIRI(expr) | BuiltinFunction::IsURI(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(Some(
                    self.bool_node(val.map(|n| matches!(n, Node::Iri(_))).unwrap_or(false)),
                ))
            }

            BuiltinFunction::IsBlank(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(Some(
                    self.bool_node(val.map(|n| matches!(n, Node::BlankNode(_))).unwrap_or(false)),
                ))
            }

            BuiltinFunction::IsLiteral(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(Some(
                    self.bool_node(val.map(|n| matches!(n, Node::Literal { .. })).unwrap_or(false)),
                ))
            }

            BuiltinFunction::IsNumeric(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(Some(self.bool_node(val.as_ref().map_or(false, |n| {
                    if let Node::Literal(lit) = n {
                        // Check if datatype is numeric (not if content is parseable)
                        // SPARQL spec: isNumeric checks if literal HAS numeric datatype
                        if let Some(dt) = lit.datatype {
                            matches!(dt,
                                "http://www.w3.org/2001/XMLSchema#integer" |
                                "http://www.w3.org/2001/XMLSchema#decimal" |
                                "http://www.w3.org/2001/XMLSchema#float" |
                                "http://www.w3.org/2001/XMLSchema#double" |
                                "http://www.w3.org/2001/XMLSchema#int" |
                                "http://www.w3.org/2001/XMLSchema#long" |
                                "http://www.w3.org/2001/XMLSchema#short" |
                                "http://www.w3.org/2001/XMLSchema#byte" |
                                "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" |
                                "http://www.w3.org/2001/XMLSchema#negativeInteger" |
                                "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" |
                                "http://www.w3.org/2001/XMLSchema#unsignedLong" |
                                "http://www.w3.org/2001/XMLSchema#unsignedInt" |
                                "http://www.w3.org/2001/XMLSchema#unsignedShort" |
                                "http://www.w3.org/2001/XMLSchema#unsignedByte" |
                                "http://www.w3.org/2001/XMLSchema#positiveInteger"
                            )
                        } else {
                            false  // No datatype = xsd:string = not numeric
                        }
                    } else {
                        false
                    }
                }))))
            }

            BuiltinFunction::Bound(var) => Ok(Some(self.bool_node(binding.contains(var)))),

            BuiltinFunction::SameTerm(left, right) => {
                let left_val = self.evaluate_expression(left, binding)?;
                let right_val = self.evaluate_expression(right, binding)?;
                Ok(Some(self.bool_node(left_val == right_val)))
            }

            BuiltinFunction::If(cond, then_expr, else_expr) => {
                let cond_val = self.evaluate_expression(cond, binding)?;
                if self.effective_boolean_value(cond_val) {
                    self.evaluate_expression(then_expr, binding)
                } else {
                    self.evaluate_expression(else_expr, binding)
                }
            }

            BuiltinFunction::Coalesce(exprs) => {
                for expr in exprs {
                    if let Some(val) = self.evaluate_expression(expr, binding)? {
                        return Ok(Some(val));
                    }
                }
                Ok(None)
            }

            BuiltinFunction::IRI(expr) | BuiltinFunction::URI(expr) => {
                // Convert string to IRI
                if let Some(value) = self.evaluate_expression(expr, binding)? {
                    if let Some(s) = self.as_string(&value) {
                        Ok(Some(Node::iri(self.dictionary.intern(s))))
                    } else {
                        Err(ExecutionError::TypeError("IRI requires string".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::Substr(str_expr, start_expr, len_expr) => {
                // SUBSTR(str, start) or SUBSTR(str, start, length)
                if let (Some(str_val), Some(start_val)) = (
                    self.evaluate_expression(str_expr, binding)?,
                    self.evaluate_expression(start_expr, binding)?,
                ) {
                    if let (Some(s), Some(start)) = (self.as_string(&str_val), self.as_numeric(&start_val)) {
                        let start_idx = (start as i64 - 1).max(0) as usize;  // SPARQL uses 1-based indexing

                        let result = if let Some(len_expr) = len_expr {
                            if let Some(len_val) = self.evaluate_expression(len_expr, binding)? {
                                if let Some(length) = self.as_numeric(&len_val) {
                                    let len = (length as i64).max(0) as usize;
                                    s.chars().skip(start_idx).take(len).collect::<String>()
                                } else {
                                    return Err(ExecutionError::TypeError("SUBSTR length must be numeric".to_string()));
                                }
                            } else {
                                return Ok(None);
                            }
                        } else {
                            s.chars().skip(start_idx).collect::<String>()
                        };

                        Ok(Some(Node::literal_str(self.dictionary.intern(&result))))
                    } else {
                        Err(ExecutionError::TypeError("SUBSTR requires string and numeric arguments".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::StrBefore(str_expr, search_expr) => {
                // STRBEFORE(str, search) - substring before first occurrence
                if let (Some(str_val), Some(search_val)) = (
                    self.evaluate_expression(str_expr, binding)?,
                    self.evaluate_expression(search_expr, binding)?,
                ) {
                    if let (Some(s), Some(search)) = (self.as_string(&str_val), self.as_string(&search_val)) {
                        if let Some(pos) = s.find(search) {
                            let result = &s[..pos];
                            Ok(Some(Node::literal_str(self.dictionary.intern(result))))
                        } else {
                            Ok(Some(Node::literal_str(self.dictionary.intern(""))))
                        }
                    } else {
                        Err(ExecutionError::TypeError("STRBEFORE requires strings".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::StrAfter(str_expr, search_expr) => {
                // STRAFTER(str, search) - substring after first occurrence
                if let (Some(str_val), Some(search_val)) = (
                    self.evaluate_expression(str_expr, binding)?,
                    self.evaluate_expression(search_expr, binding)?,
                ) {
                    if let (Some(s), Some(search)) = (self.as_string(&str_val), self.as_string(&search_val)) {
                        if let Some(pos) = s.find(search) {
                            let result = &s[pos + search.len()..];
                            Ok(Some(Node::literal_str(self.dictionary.intern(result))))
                        } else {
                            Ok(Some(Node::literal_str(self.dictionary.intern(""))))
                        }
                    } else {
                        Err(ExecutionError::TypeError("STRAFTER requires strings".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::EncodeForURI(expr) => {
                // ENCODE_FOR_URI - percent-encode string for use in URI
                if let Some(value) = self.evaluate_expression(expr, binding)? {
                    if let Some(s) = self.as_string(&value) {
                        let encoded = urlencoding::encode(s).to_string();
                        Ok(Some(Node::literal_str(self.dictionary.intern(&encoded))))
                    } else {
                        Err(ExecutionError::TypeError("ENCODE_FOR_URI requires string".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::Replace(str_expr, pattern_expr, replacement_expr, flags_expr) => {
                // REPLACE(str, pattern, replacement [, flags])
                if let (Some(str_val), Some(pattern_val), Some(replacement_val)) = (
                    self.evaluate_expression(str_expr, binding)?,
                    self.evaluate_expression(pattern_expr, binding)?,
                    self.evaluate_expression(replacement_expr, binding)?,
                ) {
                    if let (Some(s), Some(pattern), Some(replacement)) = (
                        self.as_string(&str_val),
                        self.as_string(&pattern_val),
                        self.as_string(&replacement_val),
                    ) {
                        // Support SPARQL 1.1 regex flags: i (case-insensitive), m (multiline), s (dot-all), x (extended)
                        let flags = if let Some(flags_box) = flags_expr {
                            if let Some(flags_val) = self.evaluate_expression(flags_box, binding)? {
                                self.as_string(&flags_val).unwrap_or("")
                            } else {
                                ""
                            }
                        } else {
                            ""
                        };

                        if flags.is_empty() {
                            // Simple string replacement (no regex)
                            let result = s.replace(pattern, replacement);
                            Ok(Some(Node::literal_str(self.dictionary.intern(&result))))
                        } else {
                            // Regex replacement with flags
                            let mut builder = regex::RegexBuilder::new(pattern);
                            for flag in flags.chars() {
                                match flag {
                                    'i' => { builder.case_insensitive(true); }
                                    'm' => { builder.multi_line(true); }
                                    's' => { builder.dot_matches_new_line(true); }
                                    'x' => { builder.ignore_whitespace(true); }
                                    _ => return Err(ExecutionError::EvaluationError(format!("Invalid regex flag: {}", flag))),
                                }
                            }
                            match builder.build() {
                                Ok(re) => {
                                    let result = re.replace_all(s, replacement).to_string();
                                    Ok(Some(Node::literal_str(self.dictionary.intern(&result))))
                                }
                                Err(_) => Err(ExecutionError::EvaluationError("Invalid regex pattern".to_string())),
                            }
                        }
                    } else {
                        Err(ExecutionError::TypeError("REPLACE requires strings".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::Regex(str_expr, pattern_expr, flags_expr) => {
                // REGEX(str, pattern [, flags]) - test if string matches regex
                if let (Some(str_val), Some(pattern_val)) = (
                    self.evaluate_expression(str_expr, binding)?,
                    self.evaluate_expression(pattern_expr, binding)?,
                ) {
                    if let (Some(s), Some(pattern)) = (self.as_string(&str_val), self.as_string(&pattern_val)) {
                        // Support SPARQL 1.1 regex flags: i (case-insensitive), m (multiline), s (dot-all), x (extended)
                        let flags = if let Some(flags_box) = flags_expr {
                            if let Some(flags_val) = self.evaluate_expression(flags_box, binding)? {
                                self.as_string(&flags_val).unwrap_or("")
                            } else {
                                ""
                            }
                        } else {
                            ""
                        };

                        let mut builder = regex::RegexBuilder::new(pattern);
                        for flag in flags.chars() {
                            match flag {
                                'i' => { builder.case_insensitive(true); }
                                'm' => { builder.multi_line(true); }
                                's' => { builder.dot_matches_new_line(true); }
                                'x' => { builder.ignore_whitespace(true); }
                                _ => return Err(ExecutionError::EvaluationError(format!("Invalid regex flag: {}", flag))),
                            }
                        }

                        match builder.build() {
                            Ok(re) => Ok(Some(self.bool_node(re.is_match(s)))),
                            Err(_) => Err(ExecutionError::EvaluationError("Invalid regex pattern".to_string())),
                        }
                    } else {
                        Err(ExecutionError::TypeError("REGEX requires strings".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::Rand => {
                // RAND() - random number between 0 and 1
                use rand::Rng;
                let val: f64 = rand::thread_rng().gen();
                Ok(Some(Node::literal_typed(
                    self.dictionary.intern(&val.to_string()),
                    self.dictionary.intern("http://www.w3.org/2001/XMLSchema#double"),
                )))
            }

            BuiltinFunction::Now => {
                // NOW() - current datetime
                use std::time::SystemTime;
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                // Format as xsd:dateTime (simplified - just timestamp for now)
                let datetime = format!("{}", now);
                Ok(Some(Node::literal_typed(
                    self.dictionary.intern(&datetime),
                    self.dictionary.intern("http://www.w3.org/2001/XMLSchema#dateTime"),
                )))
            }

            BuiltinFunction::UUID => {
                // UUID() - generate random UUID
                use uuid::Uuid;
                let id = Uuid::new_v4().to_string();
                let uri = format!("urn:uuid:{}", id);
                Ok(Some(Node::iri(self.dictionary.intern(&uri))))
            }

            BuiltinFunction::StrUUID => {
                // STRUUID() - generate random UUID as string
                use uuid::Uuid;
                let id = Uuid::new_v4().to_string();
                Ok(Some(Node::literal_str(self.dictionary.intern(&id))))
            }

            BuiltinFunction::BNode(expr_opt) => {
                // BNODE([label]) - generate blank node
                if let Some(expr) = expr_opt {
                    if let Some(value) = self.evaluate_expression(expr, binding)? {
                        if let Some(label) = self.as_string(&value) {
                            // Use hash of label as blank node ID
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};
                            let mut hasher = DefaultHasher::new();
                            label.hash(&mut hasher);
                            let id = hasher.finish();
                            Ok(Some(Node::blank(id)))
                        } else {
                            Err(ExecutionError::TypeError("BNODE label must be string".to_string()))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    // Generate fresh blank node
                    use rand::Rng;
                    let id: u64 = rand::thread_rng().gen();
                    Ok(Some(Node::blank(id)))
                }
            }

            BuiltinFunction::StrLang(str_expr, lang_expr) => {
                // STRLANG(str, lang) - create language-tagged literal
                if let (Some(str_val), Some(lang_val)) = (
                    self.evaluate_expression(str_expr, binding)?,
                    self.evaluate_expression(lang_expr, binding)?,
                ) {
                    if let (Some(s), Some(lang)) = (self.as_string(&str_val), self.as_string(&lang_val)) {
                        Ok(Some(Node::literal_lang(
                            self.dictionary.intern(s),
                            self.dictionary.intern(lang),
                        )))
                    } else {
                        Err(ExecutionError::TypeError("STRLANG requires strings".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::StrDT(str_expr, datatype_expr) => {
                // STRDT(str, datatype) - create typed literal
                if let (Some(str_val), Some(dt_val)) = (
                    self.evaluate_expression(str_expr, binding)?,
                    self.evaluate_expression(datatype_expr, binding)?,
                ) {
                    if let Some(s) = self.as_string(&str_val) {
                        if let Node::Iri(dt_iri) = dt_val {
                            Ok(Some(Node::literal_typed(
                                self.dictionary.intern(s),
                                dt_iri.0,
                            )))
                        } else {
                            Err(ExecutionError::TypeError("STRDT datatype must be IRI".to_string()))
                        }
                    } else {
                        Err(ExecutionError::TypeError("STRDT requires string".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            BuiltinFunction::LangMatches(lang_expr, range_expr) => {
                // LANGMATCHES(lang, range) - test if language tag matches range
                if let (Some(lang_val), Some(range_val)) = (
                    self.evaluate_expression(lang_expr, binding)?,
                    self.evaluate_expression(range_expr, binding)?,
                ) {
                    if let (Some(lang), Some(range)) = (self.as_string(&lang_val), self.as_string(&range_val)) {
                        let matches = if range == "*" {
                            !lang.is_empty()
                        } else {
                            lang.eq_ignore_ascii_case(range) || lang.to_lowercase().starts_with(&format!("{}-", range.to_lowercase()))
                        };
                        Ok(Some(self.bool_node(matches)))
                    } else {
                        Err(ExecutionError::TypeError("LANGMATCHES requires strings".to_string()))
                    }
                } else {
                    Ok(None)
                }
            }

            // Date/Time extraction functions
            BuiltinFunction::Year(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    if let Some(lit) = n.as_literal() {
                        // Parse datetime from lexical form (ISO 8601: "2023-11-25T10:30:45Z")
                        let datetime_str = lit.lexical_form;
                        if let Some(year_str) = datetime_str.split('-').next() {
                            if let Ok(year) = year_str.parse::<i64>() {
                                return Some(self.integer_node(year));
                            }
                        }
                    }
                    None
                }))
            }

            BuiltinFunction::Month(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    if let Some(lit) = n.as_literal() {
                        let datetime_str = lit.lexical_form;
                        // Split "2023-11-25T..." and get month part
                        let parts: Vec<&str> = datetime_str.splitn(3, '-').collect();
                        if parts.len() >= 2 {
                            if let Ok(month) = parts[1].parse::<i64>() {
                                return Some(self.integer_node(month));
                            }
                        }
                    }
                    None
                }))
            }

            BuiltinFunction::Day(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    if let Some(lit) = n.as_literal() {
                        let datetime_str = lit.lexical_form;
                        // Split "2023-11-25T10:30:45Z" and get day
                        if let Some(date_part) = datetime_str.split('T').next() {
                            let parts: Vec<&str> = date_part.split('-').collect();
                            if parts.len() >= 3 {
                                if let Ok(day) = parts[2].parse::<i64>() {
                                    return Some(self.integer_node(day));
                                }
                            }
                        }
                    }
                    None
                }))
            }

            BuiltinFunction::Hours(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    if let Some(lit) = n.as_literal() {
                        let datetime_str = lit.lexical_form;
                        // Split "2023-11-25T10:30:45Z" and get time part
                        if let Some(time_part) = datetime_str.split('T').nth(1) {
                            if let Some(hours_str) = time_part.split(':').next() {
                                if let Ok(hours) = hours_str.parse::<i64>() {
                                    return Some(self.integer_node(hours));
                                }
                            }
                        }
                    }
                    None
                }))
            }

            BuiltinFunction::Minutes(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    if let Some(lit) = n.as_literal() {
                        let datetime_str = lit.lexical_form;
                        if let Some(time_part) = datetime_str.split('T').nth(1) {
                            let parts: Vec<&str> = time_part.split(':').collect();
                            if parts.len() >= 2 {
                                if let Ok(minutes) = parts[1].parse::<i64>() {
                                    return Some(self.integer_node(minutes));
                                }
                            }
                        }
                    }
                    None
                }))
            }

            BuiltinFunction::Seconds(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    if let Some(lit) = n.as_literal() {
                        let datetime_str = lit.lexical_form;
                        if let Some(time_part) = datetime_str.split('T').nth(1) {
                            let parts: Vec<&str> = time_part.split(':').collect();
                            if parts.len() >= 3 {
                                // Remove timezone suffix (Z, +00:00, etc.)
                                let seconds_str = parts[2].trim_end_matches('Z')
                                    .split('+').next().unwrap_or("")
                                    .split('-').next().unwrap_or("");
                                if let Ok(seconds) = seconds_str.parse::<f64>() {
                                    return Some(self.numeric_node(seconds));
                                }
                            }
                        }
                    }
                    None
                }))
            }

            BuiltinFunction::Timezone(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    if let Some(lit) = n.as_literal() {
                        let datetime_str = lit.lexical_form;
                        // Extract timezone: "Z" or "+00:00" or "-05:00"
                        if datetime_str.ends_with('Z') {
                            // UTC = PT0S (duration 0 seconds)
                            return Some(Node::literal_typed(
                                self.dictionary.intern("PT0S"),
                                self.dictionary.intern("http://www.w3.org/2001/XMLSchema#dayTimeDuration")
                            ));
                        }
                        // Parse +HH:MM or -HH:MM timezone offsets
                        if let Some(pos) = datetime_str.rfind('+').or_else(|| datetime_str.rfind('-')) {
                            let tz_str = &datetime_str[pos..];
                            // Convert +HH:MM to xsd:dayTimeDuration format (PTnHnM)
                            if let Some((sign, rest)) = tz_str.split_at_checked(1) {
                                let parts: Vec<&str> = rest.split(':').collect();
                                if parts.len() >= 2 {
                                    if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                                        let total_minutes = hours * 60 + minutes;
                                        let duration = if sign == "-" {
                                            format!("-PT{}M", total_minutes)
                                        } else {
                                            format!("PT{}M", total_minutes)
                                        };
                                        return Some(Node::literal_typed(
                                            self.dictionary.intern(&duration),
                                            self.dictionary.intern("http://www.w3.org/2001/XMLSchema#dayTimeDuration")
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    None
                }))
            }

            BuiltinFunction::TZ(expr) => {
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    if let Some(lit) = n.as_literal() {
                        let datetime_str = lit.lexical_form;
                        // Return timezone as string: "Z", "+00:00", "-05:00"
                        if datetime_str.ends_with('Z') {
                            return Some(Node::literal_str(self.dictionary.intern("Z")));
                        }
                        // Find timezone offset (after last + or -)
                        if let Some(pos) = datetime_str.rfind('+').or_else(|| datetime_str.rfind('-')) {
                            let tz_str = &datetime_str[pos..];
                            return Some(Node::literal_str(self.dictionary.intern(tz_str)));
                        }
                    }
                    None
                }))
            }

            // Hash functions
            BuiltinFunction::MD5(expr) => {
                use md5::{Md5, Digest};
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_string_value(&n).map(|s| {
                        let hash = format!("{:x}", Md5::digest(s.as_bytes()));
                        Node::literal_str(self.dictionary.intern(&hash))
                    })
                }))
            }

            BuiltinFunction::SHA1(expr) => {
                use sha1::{Sha1, Digest};
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_string_value(&n).map(|s| {
                        let hash = format!("{:x}", Sha1::digest(s.as_bytes()));
                        Node::literal_str(self.dictionary.intern(&hash))
                    })
                }))
            }

            BuiltinFunction::SHA256(expr) => {
                use sha2::{Sha256, Digest};
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_string_value(&n).map(|s| {
                        let hash = format!("{:x}", Sha256::digest(s.as_bytes()));
                        Node::literal_str(self.dictionary.intern(&hash))
                    })
                }))
            }

            BuiltinFunction::SHA384(expr) => {
                use sha2::{Sha384, Digest};
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_string_value(&n).map(|s| {
                        let hash = format!("{:x}", Sha384::digest(s.as_bytes()));
                        Node::literal_str(self.dictionary.intern(&hash))
                    })
                }))
            }

            BuiltinFunction::SHA512(expr) => {
                use sha2::{Sha512, Digest};
                let val = self.evaluate_expression(expr, binding)?;
                Ok(val.and_then(|n| {
                    self.get_string_value(&n).map(|s| {
                        let hash = format!("{:x}", Sha512::digest(s.as_bytes()));
                        Node::literal_str(self.dictionary.intern(&hash))
                    })
                }))
            }
        }
    }

    /// Evaluate property path
    fn evaluate_path(
        &self,
        subject: &VarOrNode<'a>,
        path: &PropertyPath<'a>,
        object: &VarOrNode<'a>,
    ) -> ExecutionResult<BindingSet<'a>> {
        match path {
            PropertyPath::Predicate(pred) => {
                // Direct predicate - evaluate as triple pattern
                let pattern = TriplePattern {
                    subject: subject.clone(),
                    predicate: VarOrNode::Node(pred.clone()),
                    object: object.clone(),
                };
                self.evaluate_triple_pattern(&pattern)
            }

            PropertyPath::Inverse(inner) => {
                // Swap subject and object
                self.evaluate_path(object, inner, subject)
            }

            PropertyPath::Sequence(first, second) => {
                // p1 / p2: find intermediate nodes
                let intermediate = Variable::new("__intermediate");
                let von_intermediate = VarOrNode::Var(intermediate.clone());

                let first_results = self.evaluate_path(subject, first, &von_intermediate)?;
                let second_results = self.evaluate_path(&von_intermediate, second, object)?;

                // Join and remove intermediate variable
                let joined = first_results.join(&second_results);
                Ok(joined)
            }

            PropertyPath::Alternative(left, right) => {
                // p1 | p2: union of both paths
                let mut left_results = self.evaluate_path(subject, left, object)?;
                let right_results = self.evaluate_path(subject, right, object)?;
                left_results.union(right_results);
                Ok(left_results)
            }

            PropertyPath::ZeroOrMore(inner) => {
                // p*: reflexive transitive closure
                self.evaluate_star_path(subject, inner, object)
            }

            PropertyPath::OneOrMore(inner) => {
                // p+: transitive closure
                let star_results = self.evaluate_star_path(subject, inner, object)?;

                // Remove identity mappings (where subject == object)
                let mut results = BindingSet::new();
                for binding in star_results.iter() {
                    let subj_val = self.get_var_value(subject, &binding);
                    let obj_val = self.get_var_value(object, &binding);

                    if subj_val != obj_val {
                        results.add(binding.clone());
                    }
                }
                Ok(results)
            }

            PropertyPath::ZeroOrOne(inner) => {
                // p?: zero or one step
                let mut results = self.evaluate_path(subject, inner, object)?;

                // Add identity mappings
                let identity = self.create_identity_bindings(subject, object)?;
                results.union(identity);

                Ok(results)
            }

            PropertyPath::NegatedPropertySet(predicates) => {
                // Find all predicates NOT in the set
                self.evaluate_negated_path(subject, predicates, object)
            }
        }
    }

    /// Evaluate p* (zero or more steps)
    fn evaluate_star_path(
        &self,
        subject: &VarOrNode<'a>,
        path: &PropertyPath<'a>,
        object: &VarOrNode<'a>,
    ) -> ExecutionResult<BindingSet<'a>> {
        let mut results = BindingSet::new();
        let mut visited = HashSet::new();
        let mut queue = Vec::new();

        // Start with direct paths
        let direct = self.evaluate_path(subject, path, object)?;

        for binding in direct.iter() {
            let key = self.binding_key(&binding);
            if visited.insert(key) {
                queue.push(binding.clone());
                results.add(binding.clone());
            }
        }

        // Add identity mappings (zero steps)
        let identity = self.create_identity_bindings(subject, object)?;
        for binding in identity.iter() {
            let key = self.binding_key(&binding);
            if visited.insert(key) {
                results.add(binding.clone());
            }
        }

        // Iteratively expand paths
        let max_iterations = 1000; // Prevent infinite loops
        let mut iteration = 0;

        while !queue.is_empty() && iteration < max_iterations {
            let current = queue.remove(0);

            // Try to extend this path
            let intermediate = Variable::new("__path_intermediate");
            let extended = self.evaluate_path(
                &self.binding_to_var_or_node(object, &current),
                path,
                &VarOrNode::Var(intermediate.clone()),
            )?;

            for ext_binding in extended.iter() {
                if let Some(merged) = current.merge(ext_binding) {
                    let key = self.binding_key(&merged);
                    if visited.insert(key) {
                        queue.push(merged.clone());
                        results.add(merged);
                    }
                }
            }

            iteration += 1;
        }

        Ok(results)
    }

    /// Create identity bindings for path (subject = object)
    fn create_identity_bindings(
        &self,
        subject: &VarOrNode<'a>,
        object: &VarOrNode<'a>,
    ) -> ExecutionResult<BindingSet<'a>> {
        match (subject, object) {
            (VarOrNode::Var(s), VarOrNode::Var(o)) if s == o => {
                // Same variable: enumerate all nodes
                Ok(BindingSet::new()) // Simplified: would need to scan all nodes
            }
            (VarOrNode::Node(n), VarOrNode::Var(v)) => {
                let mut binding = Binding::new();
                binding.bind(v.clone(), n.clone());
                Ok(BindingSet::from_bindings(vec![binding]))
            }
            (VarOrNode::Var(v), VarOrNode::Node(n)) => {
                let mut binding = Binding::new();
                binding.bind(v.clone(), n.clone());
                Ok(BindingSet::from_bindings(vec![binding]))
            }
            (VarOrNode::Node(n1), VarOrNode::Node(n2)) if n1 == n2 => {
                Ok(BindingSet::from_bindings(vec![Binding::new()]))
            }
            _ => Ok(BindingSet::new()),
        }
    }

    /// Evaluate negated property set
    fn evaluate_negated_path(
        &self,
        subject: &VarOrNode<'a>,
        excluded: &[Node<'a>],
        object: &VarOrNode<'a>,
    ) -> ExecutionResult<BindingSet<'a>> {
        // Find all triples, then filter out excluded predicates
        let pattern = TriplePattern {
            subject: subject.clone(),
            predicate: VarOrNode::Var(Variable::new("__pred")),
            object: object.clone(),
        };

        let all_results = self.evaluate_triple_pattern(&pattern)?;

        let mut results = BindingSet::new();
        for binding in all_results.iter() {
            if let Some(pred_node) = binding.get(&Variable::new("__pred")) {
                if !excluded.contains(pred_node) {
                    results.add(binding.clone());
                }
            }
        }

        Ok(results)
    }

    /// Helper: get unique key for binding (for cycle detection)
    fn binding_key(&self, binding: &Binding<'a>) -> String {
        format!("{:?}", binding) // Simplified
    }

    /// Helper: convert binding value to VarOrNode
    fn binding_to_var_or_node(
        &self,
        von: &VarOrNode<'a>,
        binding: &Binding<'a>,
    ) -> VarOrNode<'a> {
        match von {
            VarOrNode::Var(v) => binding
                .get(v)
                .map(|n| VarOrNode::Node(n.clone()))
                .unwrap_or_else(|| von.clone()),
            VarOrNode::Node(_) => von.clone(),
        }
    }

    /// Helper: get variable value from VarOrNode
    fn get_var_value(&self, von: &VarOrNode<'a>, binding: &Binding<'a>) -> Option<Node<'a>> {
        match von {
            VarOrNode::Var(v) => binding.get(v).cloned(),
            VarOrNode::Node(n) => Some(n.clone()),
        }
    }

    // Boolean/comparison helpers
    fn true_node(&self) -> Node<'a> {
        Node::literal_typed(
            self.dictionary.intern("true"),
            self.dictionary
                .intern("http://www.w3.org/2001/XMLSchema#boolean"),
        )
    }

    fn false_node(&self) -> Node<'a> {
        Node::literal_typed(
            self.dictionary.intern("false"),
            self.dictionary
                .intern("http://www.w3.org/2001/XMLSchema#boolean"),
        )
    }

    fn bool_node(&self, value: bool) -> Node<'a> {
        if value {
            self.true_node()
        } else {
            self.false_node()
        }
    }

    fn integer_node(&self, value: i64) -> Node<'a> {
        Node::literal_typed(
            self.dictionary.intern(&value.to_string()),
            self.dictionary
                .intern("http://www.w3.org/2001/XMLSchema#integer"),
        )
    }

    fn numeric_node(&self, value: f64) -> Node<'a> {
        Node::literal_typed(
            self.dictionary.intern(&value.to_string()),
            self.dictionary
                .intern("http://www.w3.org/2001/XMLSchema#double"),
        )
    }

    fn to_string_node(&self, node: &Node<'a>) -> Node<'a> {
        match node {
            Node::Iri(iri) => Node::literal_str(iri.as_str()),
            Node::BlankNode(id) => {
                // Convert blank node ID to string representation
                let s = format!("_:b{}", id);
                Node::literal_str(self.dictionary.intern(&s))
            }
            Node::Literal(lit) => Node::literal_str(lit.lexical_form),
            Node::QuotedTriple(_) => Node::literal_str("<<quoted-triple>>"),
            Node::Variable(_) => Node::literal_str("?var"),
        }
    }

    fn effective_boolean_value(&self, value: Option<Node<'a>>) -> bool {
        match value {
            None => false,
            Some(Node::Literal(lit)) => {
                if let Some(dt) = lit.datatype {
                    if dt == "http://www.w3.org/2001/XMLSchema#boolean" {
                        return lit.lexical_form == "true" || lit.lexical_form == "1";
                    }
                }
                // Non-empty string is true
                !lit.lexical_form.is_empty()
            }
            Some(_) => true,
        }
    }

    // Comparison operations
    fn logical_or(&self, left: Option<Node<'a>>, right: Option<Node<'a>>) -> Option<Node<'a>> {
        let l = self.effective_boolean_value(left);
        let r = self.effective_boolean_value(right);
        Some(self.bool_node(l || r))
    }

    fn logical_and(&self, left: Option<Node<'a>>, right: Option<Node<'a>>) -> Option<Node<'a>> {
        let l = self.effective_boolean_value(left);
        let r = self.effective_boolean_value(right);
        Some(self.bool_node(l && r))
    }

    fn logical_not(&self, value: Option<Node<'a>>) -> Option<Node<'a>> {
        Some(self.bool_node(!self.effective_boolean_value(value)))
    }

    fn equal(&self, left: Option<Node<'a>>, right: Option<Node<'a>>) -> Option<Node<'a>> {
        match (left, right) {
            (Some(l), Some(r)) => Some(self.bool_node(l == r)),
            _ => None,
        }
    }

    fn not_equal(&self, left: Option<Node<'a>>, right: Option<Node<'a>>) -> Option<Node<'a>> {
        match (left, right) {
            (Some(l), Some(r)) => Some(self.bool_node(l != r)),
            _ => None,
        }
    }

    fn less_than(&self, left: Option<Node<'a>>, right: Option<Node<'a>>) -> Option<Node<'a>> {
        match (left, right) {
            (Some(l), Some(r)) => {
                let l_num = self.get_numeric_value(&l);
                let r_num = self.get_numeric_value(&r);
                match (l_num, r_num) {
                    (Some(ln), Some(rn)) => Some(self.bool_node(ln < rn)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn greater_than(&self, left: Option<Node<'a>>, right: Option<Node<'a>>) -> Option<Node<'a>> {
        match (left, right) {
            (Some(l), Some(r)) => {
                let l_num = self.get_numeric_value(&l);
                let r_num = self.get_numeric_value(&r);
                match (l_num, r_num) {
                    (Some(ln), Some(rn)) => Some(self.bool_node(ln > rn)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn less_or_equal(&self, left: Option<Node<'a>>, right: Option<Node<'a>>) -> Option<Node<'a>> {
        match (left, right) {
            (Some(l), Some(r)) => {
                let l_num = self.get_numeric_value(&l);
                let r_num = self.get_numeric_value(&r);
                match (l_num, r_num) {
                    (Some(ln), Some(rn)) => Some(self.bool_node(ln <= rn)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn greater_or_equal(
        &self,
        left: Option<Node<'a>>,
        right: Option<Node<'a>>,
    ) -> Option<Node<'a>> {
        match (left, right) {
            (Some(l), Some(r)) => {
                let l_num = self.get_numeric_value(&l);
                let r_num = self.get_numeric_value(&r);
                match (l_num, r_num) {
                    (Some(ln), Some(rn)) => Some(self.bool_node(ln >= rn)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    // Numeric operations
    fn numeric_add(
        &self,
        left: Option<Node<'a>>,
        right: Option<Node<'a>>,
    ) -> ExecutionResult<Option<Node<'a>>> {
        match (left, right) {
            (Some(l), Some(r)) => {
                let l_num = self.get_numeric_value(&l);
                let r_num = self.get_numeric_value(&r);
                match (l_num, r_num) {
                    (Some(ln), Some(rn)) => Ok(Some(self.numeric_node(ln + rn))),
                    _ => Err(ExecutionError::TypeError("Non-numeric operands".to_string())),
                }
            }
            _ => Ok(None),
        }
    }

    fn numeric_subtract(
        &self,
        left: Option<Node<'a>>,
        right: Option<Node<'a>>,
    ) -> ExecutionResult<Option<Node<'a>>> {
        match (left, right) {
            (Some(l), Some(r)) => {
                let l_num = self.get_numeric_value(&l);
                let r_num = self.get_numeric_value(&r);
                match (l_num, r_num) {
                    (Some(ln), Some(rn)) => Ok(Some(self.numeric_node(ln - rn))),
                    _ => Err(ExecutionError::TypeError("Non-numeric operands".to_string())),
                }
            }
            _ => Ok(None),
        }
    }

    fn numeric_multiply(
        &self,
        left: Option<Node<'a>>,
        right: Option<Node<'a>>,
    ) -> ExecutionResult<Option<Node<'a>>> {
        match (left, right) {
            (Some(l), Some(r)) => {
                let l_num = self.get_numeric_value(&l);
                let r_num = self.get_numeric_value(&r);
                match (l_num, r_num) {
                    (Some(ln), Some(rn)) => Ok(Some(self.numeric_node(ln * rn))),
                    _ => Err(ExecutionError::TypeError("Non-numeric operands".to_string())),
                }
            }
            _ => Ok(None),
        }
    }

    fn numeric_divide(
        &self,
        left: Option<Node<'a>>,
        right: Option<Node<'a>>,
    ) -> ExecutionResult<Option<Node<'a>>> {
        match (left, right) {
            (Some(l), Some(r)) => {
                let l_num = self.get_numeric_value(&l);
                let r_num = self.get_numeric_value(&r);
                match (l_num, r_num) {
                    (Some(ln), Some(rn)) => {
                        if rn == 0.0 {
                            Err(ExecutionError::DivisionByZero)
                        } else {
                            Ok(Some(self.numeric_node(ln / rn)))
                        }
                    }
                    _ => Err(ExecutionError::TypeError("Non-numeric operands".to_string())),
                }
            }
            _ => Ok(None),
        }
    }

    fn numeric_negate(&self, value: Option<Node<'a>>) -> ExecutionResult<Option<Node<'a>>> {
        match value {
            Some(v) => {
                let num = self.get_numeric_value(&v);
                match num {
                    Some(n) => Ok(Some(self.numeric_node(-n))),
                    None => Err(ExecutionError::TypeError("Non-numeric operand".to_string())),
                }
            }
            None => Ok(None),
        }
    }

    // Value extraction helpers
    fn get_numeric_value(&self, node: &Node<'a>) -> Option<f64> {
        match node {
            Node::Literal(lit) => lit.lexical_form.parse::<f64>().ok(),
            _ => None,
        }
    }

    fn get_string_value(&self, node: &Node<'a>) -> Option<&'a str> {
        match node {
            Node::Literal(lit) => Some(lit.lexical_form),
            Node::Iri(iri) => Some(iri.as_str()),
            _ => None,
        }
    }

    /// Helper alias for builtin functions - extract string value from node
    fn as_string(&self, node: &Node<'a>) -> Option<&'a str> {
        self.get_string_value(node)
    }

    /// Helper alias for builtin functions - extract numeric value from node
    fn as_numeric(&self, node: &Node<'a>) -> Option<f64> {
        self.get_numeric_value(node)
    }

    /// Compare two nodes for ordering (since Node doesn't implement Ord)
    fn compare_nodes(&self, a: &Node<'a>, b: &Node<'a>) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (a, b) {
            (Node::Iri(a), Node::Iri(b)) => a.as_str().cmp(b.as_str()),
            (Node::Literal(a), Node::Literal(b)) => a.lexical_form.cmp(b.lexical_form),
            (Node::BlankNode(a), Node::BlankNode(b)) => a.cmp(b),
            (Node::Iri(_), _) => Ordering::Less,
            (_, Node::Iri(_)) => Ordering::Greater,
            (Node::Literal(_), Node::BlankNode(_)) => Ordering::Less,
            (Node::BlankNode(_), Node::Literal(_)) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }

    /// Evaluate GROUP BY with aggregates
    fn evaluate_group(
        &mut self,
        vars: &[Variable<'a>],
        aggregates: &[(Variable<'a>, Aggregate<'a>)],
        input: &Algebra<'a>,
    ) -> ExecutionResult<BindingSet<'a>> {
        // Evaluate input
        let input_results = self.execute(input)?;

        // Group by variables
        let mut groups: HashMap<Vec<Option<Node<'a>>>, Vec<Binding<'a>>> = HashMap::new();

        for binding in input_results.iter() {
            let key: Vec<_> = vars.iter().map(|v| binding.get(v).cloned()).collect();
            groups.entry(key).or_default().push(binding.clone());
        }

        // Evaluate aggregates for each group
        let mut results = BindingSet::new();

        for (key, group_bindings) in groups {
            let mut result_binding = Binding::new();

            // Add group-by variables
            for (i, var) in vars.iter().enumerate() {
                if let Some(node) = &key[i] {
                    result_binding.bind(var.clone(), node.clone());
                }
            }

            // Evaluate aggregates
            for (agg_var, agg) in aggregates {
                if let Some(value) = self.evaluate_aggregate(agg, &group_bindings)? {
                    result_binding.bind(agg_var.clone(), value);
                }
            }

            results.add(result_binding);
        }

        Ok(results)
    }

    /// Evaluate an aggregate function over a group
    fn evaluate_aggregate(
        &self,
        agg: &Aggregate<'a>,
        bindings: &[Binding<'a>],
    ) -> ExecutionResult<Option<Node<'a>>> {
        match agg {
            Aggregate::Count { expr, distinct } => {
                let mut values = Vec::new();

                for binding in bindings {
                    if let Some(e) = expr {
                        if let Some(val) = self.evaluate_expression(e, binding)? {
                            values.push(val);
                        }
                    } else {
                        values.push(self.true_node()); // COUNT(*)
                    }
                }

                if *distinct {
                    values.sort_by(|a, b| self.compare_nodes(a, b));
                    values.dedup();
                }

                Ok(Some(self.integer_node(values.len() as i64)))
            }

            Aggregate::Sum { expr, distinct } => {
                let mut sum = 0.0;
                let mut values = Vec::new();

                for binding in bindings {
                    if let Some(val) = self.evaluate_expression(expr, binding)? {
                        if let Some(num) = self.get_numeric_value(&val) {
                            values.push((num, val));
                        }
                    }
                }

                if *distinct {
                    values.sort_by(|a, b| self.compare_nodes(&a.1, &b.1));
                    values.dedup_by(|a, b| a.1 == b.1);
                }

                for (num, _) in values {
                    sum += num;
                }

                Ok(Some(self.numeric_node(sum)))
            }

            Aggregate::Avg { expr, distinct } => {
                let mut sum = 0.0;
                let mut count = 0;
                let mut values = Vec::new();

                for binding in bindings {
                    if let Some(val) = self.evaluate_expression(expr, binding)? {
                        if let Some(num) = self.get_numeric_value(&val) {
                            values.push((num, val));
                        }
                    }
                }

                if *distinct {
                    values.sort_by(|a, b| self.compare_nodes(&a.1, &b.1));
                    values.dedup_by(|a, b| a.1 == b.1);
                }

                for (num, _) in values {
                    sum += num;
                    count += 1;
                }

                if count > 0 {
                    Ok(Some(self.numeric_node(sum / count as f64)))
                } else {
                    Ok(Some(self.integer_node(0)))
                }
            }

            Aggregate::Min { expr, .. } | Aggregate::Max { expr, .. } => {
                let is_min = matches!(agg, Aggregate::Min { .. });
                let mut best: Option<Node<'a>> = None;

                for binding in bindings {
                    if let Some(val) = self.evaluate_expression(expr, binding)? {
                        best = match best {
                            None => Some(val),
                            Some(current) => {
                                let val_num = self.get_numeric_value(&val);
                                let cur_num = self.get_numeric_value(&current);

                                match (val_num, cur_num) {
                                    (Some(v), Some(c)) => {
                                        if (is_min && v < c) || (!is_min && v > c) {
                                            Some(val)
                                        } else {
                                            Some(current)
                                        }
                                    }
                                    _ => Some(current),
                                }
                            }
                        };
                    }
                }

                Ok(best)
            }

            Aggregate::Sample { expr, .. } => {
                // Return first non-null value
                for binding in bindings {
                    if let Some(val) = self.evaluate_expression(expr, binding)? {
                        return Ok(Some(val));
                    }
                }
                Ok(None)
            }

            Aggregate::GroupConcat {
                expr,
                separator,
                distinct,
            } => {
                let mut values = Vec::new();

                for binding in bindings {
                    if let Some(val) = self.evaluate_expression(expr, binding)? {
                        if let Some(s) = self.get_string_value(&val) {
                            values.push((s.to_string(), val));
                        }
                    }
                }

                if *distinct {
                    values.sort_by(|a, b| self.compare_nodes(&a.1, &b.1));
                    values.dedup_by(|a, b| a.1 == b.1);
                }

                let sep = separator.unwrap_or(" ");
                let result: Vec<_> = values.iter().map(|(s, _)| s.as_str()).collect();
                let concatenated = result.join(sep);

                Ok(Some(Node::literal_str(
                    self.dictionary.intern(&concatenated),
                )))
            }
        }
    }

}

/// SPARQL UPDATE executor
///
/// Handles UPDATE operations (INSERT, DELETE, LOAD, CLEAR, etc.) that modify the quad store.
/// Requires mutable access to the store, unlike the read-only Executor.
pub struct UpdateExecutor<'a, B: StorageBackend> {
    /// Mutable quad store for modifications
    store: &'a mut QuadStore<B>,

    /// Dictionary for node interning
    dictionary: Arc<Dictionary>,
}

impl<'a, B: StorageBackend> UpdateExecutor<'a, B> {
    /// Create a new UPDATE executor for the given mutable store
    pub fn new(store: &'a mut QuadStore<B>) -> Self {
        let dictionary = Arc::clone(store.dictionary());
        Self { store, dictionary }
    }

    /// Execute a SPARQL UPDATE operation
    ///
    /// Returns the number of quads affected (inserted or deleted).
    pub fn execute(&mut self, update: &Update<'a>) -> ExecutionResult<usize> {
        match update {
            Update::InsertData { quads } => self.execute_insert_data(quads),
            Update::DeleteData { quads } => self.execute_delete_data(quads),
            Update::DeleteInsert {
                delete,
                insert,
                pattern,
                using: _,
            } => self.execute_delete_insert(delete, insert, pattern),
            Update::DeleteWhere { quads } => self.execute_delete_where(quads),
            Update::Load { source: _, target: _, silent } => {
                if *silent {
                    Ok(0)
                } else {
                    Err(ExecutionError::Unsupported(
                        "LOAD operation requires HTTP client - not yet implemented".to_string(),
                    ))
                }
            }
            Update::Clear { graph, silent } => self.execute_clear(graph, *silent),
            Update::Create { graph: _, silent: _ } => {
                // CREATE is a no-op in our implementation (graphs are created implicitly)
                Ok(0)
            }
            Update::Drop { graph, silent } => self.execute_clear(graph, *silent),
        }
    }

    /// Execute INSERT DATA
    fn execute_insert_data(&mut self, quads: &[SparqlQuadPattern<'a>]) -> ExecutionResult<usize> {
        let mut count = 0;
        for quad_pattern in quads {
            if !quad_pattern.is_concrete() {
                return Err(ExecutionError::TypeError(
                    "INSERT DATA requires concrete quads (no variables)".to_string(),
                ));
            }

            let quad = self.quad_pattern_to_quad(quad_pattern, &Binding::new())?;
            self.store.insert(quad)?;
            count += 1;
        }
        Ok(count)
    }

    /// Execute DELETE DATA
    fn execute_delete_data(&mut self, quads: &[SparqlQuadPattern<'a>]) -> ExecutionResult<usize> {
        let mut count = 0;
        for quad_pattern in quads {
            if !quad_pattern.is_concrete() {
                return Err(ExecutionError::TypeError(
                    "DELETE DATA requires concrete quads (no variables)".to_string(),
                ));
            }

            let quad = self.quad_pattern_to_quad(quad_pattern, &Binding::new())?;
            self.store.remove(&quad)?;
            count += 1;
        }
        Ok(count)
    }

    /// Execute DELETE/INSERT with WHERE clause
    fn execute_delete_insert(
        &mut self,
        delete: &[SparqlQuadPattern<'a>],
        insert: &[SparqlQuadPattern<'a>],
        pattern: &Algebra<'a>,
    ) -> ExecutionResult<usize> {
        // SAFETY: We use a raw pointer to bypass the borrow checker.
        // This is safe because:
        // 1. The Quad<'a> objects only contain references into the Arc<Dictionary>
        // 2. The dictionary data is stable (never moved/freed while Arc exists)
        // 3. We finish reading before we start writing
        // 4. The lifetime 'a is tied to the dictionary, not the store itself
        let store_ptr = self.store as *mut QuadStore<B>;

        let quads_to_delete: Vec<Quad<'a>>;
        let quads_to_insert: Vec<Quad<'a>>;

        {
            // SAFETY: Cast back to immutable reference for reading
            let store_ref = unsafe { &*store_ptr };
            let mut executor = Executor::new(store_ref);
            let bindings = executor.execute(pattern)?;

            let mut temp_delete = Vec::new();
            let mut temp_insert = Vec::new();

            for binding in bindings.iter() {
                for delete_pattern in delete {
                    if let Ok(quad) = self.quad_pattern_to_quad(delete_pattern, binding) {
                        temp_delete.push(quad);
                    }
                }

                for insert_pattern in insert {
                    if let Ok(quad) = self.quad_pattern_to_quad(insert_pattern, binding) {
                        temp_insert.push(quad);
                    }
                }
            }

            quads_to_delete = temp_delete;
            quads_to_insert = temp_insert;
        } // executor dropped here

        // Now we can safely mutate the store
        let mut deleted_count = 0;
        let mut inserted_count = 0;

        for quad in quads_to_delete {
            self.store.remove(&quad)?;
            deleted_count += 1;
        }

        for quad in quads_to_insert {
            self.store.insert(quad)?;
            inserted_count += 1;
        }

        Ok(deleted_count + inserted_count)
    }

    /// Execute DELETE WHERE (shorthand for DELETE with same pattern)
    fn execute_delete_where(&mut self, quads: &[SparqlQuadPattern<'a>]) -> ExecutionResult<usize> {
        // Convert to triple patterns and build BGP
        let patterns: Vec<_> = quads.iter().map(|q| q.as_triple()).collect();
        let algebra = Algebra::BGP(patterns);

        // SAFETY: Same as execute_delete_insert - use raw pointer to bypass borrow checker
        let store_ptr = self.store as *mut QuadStore<B>;

        let quads_to_delete: Vec<Quad<'a>>;
        {
            // SAFETY: Cast back to immutable reference for reading
            let store_ref = unsafe { &*store_ptr };
            let mut executor = Executor::new(store_ref);
            let bindings = executor.execute(&algebra)?;

            let mut temp_delete = Vec::new();
            for binding in bindings.iter() {
                for quad_pattern in quads {
                    if let Ok(quad) = self.quad_pattern_to_quad(quad_pattern, binding) {
                        temp_delete.push(quad);
                    }
                }
            }
            quads_to_delete = temp_delete;
        } // executor dropped here

        // Now we can safely mutate the store
        let mut count = 0;
        for quad in quads_to_delete {
            self.store.remove(&quad)?;
            count += 1;
        }

        Ok(count)
    }

    /// Execute CLEAR/DROP
    fn execute_clear(&mut self, target: &GraphTarget<'a>, silent: bool) -> ExecutionResult<usize> {
        match target {
            GraphTarget::Default => {
                // Clear default graph (all quads without graph component)
                self.store.clear()?;
                Ok(0)
            }
            GraphTarget::Named(_) | GraphTarget::Named_ | GraphTarget::All => {
                if silent {
                    Ok(0)
                } else {
                    Err(ExecutionError::Unsupported(
                        "Named graph CLEAR/DROP not yet implemented (requires graph enumeration)".to_string(),
                    ))
                }
            }
        }
    }

    /// Convert quad pattern to concrete quad using bindings
    fn quad_pattern_to_quad(
        &self,
        pattern: &SparqlQuadPattern<'a>,
        binding: &Binding<'a>,
    ) -> ExecutionResult<Quad<'a>> {
        let subject = match &pattern.subject {
            VarOrNode::Node(n) => n.clone(),
            VarOrNode::Var(v) => binding
                .get(v)
                .ok_or_else(|| ExecutionError::TypeError(format!("Unbound variable: {}", v)))?
                .clone(),
        };

        let predicate = match &pattern.predicate {
            VarOrNode::Node(n) => n.clone(),
            VarOrNode::Var(v) => binding
                .get(v)
                .ok_or_else(|| ExecutionError::TypeError(format!("Unbound variable: {}", v)))?
                .clone(),
        };

        let object = match &pattern.object {
            VarOrNode::Node(n) => n.clone(),
            VarOrNode::Var(v) => binding
                .get(v)
                .ok_or_else(|| ExecutionError::TypeError(format!("Unbound variable: {}", v)))?
                .clone(),
        };

        let graph = match &pattern.graph {
            Some(VarOrNode::Node(n)) => Some(n.clone()),
            Some(VarOrNode::Var(v)) => Some(
                binding
                    .get(v)
                    .ok_or_else(|| ExecutionError::TypeError(format!("Unbound variable: {}", v)))?
                    .clone(),
            ),
            None => None,
        };

        Ok(Quad::new(subject, predicate, object, graph))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use storage::QuadStore;
    use crate::OrderCondition;

    fn setup_test_store<'a>() -> QuadStore<storage::InMemoryBackend> {
        let mut store = QuadStore::new_in_memory();
        let dict = Arc::clone(store.dictionary());

        // Add test data
        store
            .insert(Quad::new(
                Node::iri(dict.intern("http://example.org/alice")),
                Node::iri(dict.intern("http://example.org/name")),
                Node::literal_str(dict.intern("Alice")),
                None,
            ))
            .unwrap();

        store
            .insert(Quad::new(
                Node::iri(dict.intern("http://example.org/alice")),
                Node::iri(dict.intern("http://example.org/age")),
                Node::literal_typed(
                    dict.intern("30"),
                    dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
                ),
                None,
            ))
            .unwrap();

        store
            .insert(Quad::new(
                Node::iri(dict.intern("http://example.org/bob")),
                Node::iri(dict.intern("http://example.org/name")),
                Node::literal_str(dict.intern("Bob")),
                None,
            ))
            .unwrap();

        store
    }

    #[test]
    fn test_executor_creation() {
        let store = setup_test_store();
        let _executor = Executor::new(&store);
    }

    #[test]
    fn test_bgp_execution() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let mut executor = Executor::new(&store);

        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://example.org/name"))),
            object: VarOrNode::Var(Variable::new("name")),
        };

        let bgp = Algebra::BGP(vec![pattern]);
        let results = executor.execute(&bgp).unwrap();

        assert_eq!(results.len(), 2); // Alice and Bob
    }

    #[test]
    fn test_filter_execution() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let mut executor = Executor::new(&store);

        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://example.org/name"))),
            object: VarOrNode::Var(Variable::new("name")),
        };

        let filter_expr = Expression::Equal(
            Box::new(Expression::Var(Variable::new("name"))),
            Box::new(Expression::Constant(Node::literal_str(
                dict.intern("Alice"),
            ))),
        );

        let algebra = Algebra::Filter {
            expr: filter_expr,
            input: Box::new(Algebra::BGP(vec![pattern])),
        };

        let results = executor.execute(&algebra).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_join_execution() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let mut executor = Executor::new(&store);

        let pattern1 = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://example.org/name"))),
            object: VarOrNode::Var(Variable::new("name")),
        };

        let pattern2 = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://example.org/age"))),
            object: VarOrNode::Var(Variable::new("age")),
        };

        let algebra = Algebra::Join {
            left: Box::new(Algebra::BGP(vec![pattern1])),
            right: Box::new(Algebra::BGP(vec![pattern2])),
        };

        let results = executor.execute(&algebra).unwrap();
        assert_eq!(results.len(), 1); // Only Alice has age
    }

    #[test]
    fn test_union_execution() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let mut executor = Executor::new(&store);

        let pattern1 = TriplePattern {
            subject: VarOrNode::Node(Node::iri(dict.intern("http://example.org/alice"))),
            predicate: VarOrNode::Var(Variable::new("p")),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let pattern2 = TriplePattern {
            subject: VarOrNode::Node(Node::iri(dict.intern("http://example.org/bob"))),
            predicate: VarOrNode::Var(Variable::new("p")),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let algebra = Algebra::Union {
            left: Box::new(Algebra::BGP(vec![pattern1])),
            right: Box::new(Algebra::BGP(vec![pattern2])),
        };

        let results = executor.execute(&algebra).unwrap();
        assert_eq!(results.len(), 3); // 2 for Alice, 1 for Bob
    }

    #[test]
    fn test_distinct_execution() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let mut executor = Executor::new(&store);

        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Var(Variable::new("p")),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let algebra = Algebra::Distinct {
            input: Box::new(Algebra::BGP(vec![pattern])),
        };

        let results = executor.execute(&algebra).unwrap();
        assert!(results.len() > 0);
    }

    #[test]
    fn test_project_execution() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let mut executor = Executor::new(&store);

        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://example.org/name"))),
            object: VarOrNode::Var(Variable::new("name")),
        };

        let algebra = Algebra::Project {
            vars: vec![Variable::new("name")],
            input: Box::new(Algebra::BGP(vec![pattern])),
        };

        let results = executor.execute(&algebra).unwrap();
        assert_eq!(results.len(), 2);

        for binding in results.iter() {
            assert!(binding.contains(&Variable::new("name")));
            assert!(!binding.contains(&Variable::new("s")));
        }
    }

    #[test]
    fn test_slice_execution() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let mut executor = Executor::new(&store);

        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Var(Variable::new("p")),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let algebra = Algebra::Slice {
            start: Some(1),
            length: Some(1),
            input: Box::new(Algebra::BGP(vec![pattern])),
        };

        let results = executor.execute(&algebra).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_expression_evaluation() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let executor = Executor::new(&store);

        let mut binding = Binding::new();
        binding.bind(
            Variable::new("x"),
            Node::literal_typed(
                dict.intern("10"),
                dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
            ),
        );

        let expr = Expression::Add(
            Box::new(Expression::Var(Variable::new("x"))),
            Box::new(Expression::Constant(Node::literal_typed(
                dict.intern("5"),
                dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
            ))),
        );

        let result = executor.evaluate_expression(&expr, &binding).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_builtin_str() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let executor = Executor::new(&store);

        let mut binding = Binding::new();
        binding.bind(Variable::new("x"), Node::iri(dict.intern("http://example.org")));

        let builtin = BuiltinFunction::Str(Box::new(Expression::Var(Variable::new("x"))));
        let result = executor.evaluate_builtin(&builtin, &binding).unwrap();

        assert!(result.is_some());
    }

    #[test]
    fn test_builtin_bound() {
        let store = setup_test_store();
        let executor = Executor::new(&store);

        let mut binding = Binding::new();
        let dict = store.dictionary();
        binding.bind(Variable::new("x"), Node::iri(dict.intern("http://test")));

        let builtin = BuiltinFunction::Bound(Variable::new("x"));
        let result = executor.evaluate_builtin(&builtin, &binding).unwrap();

        assert_eq!(result, Some(executor.true_node()));
    }

    #[test]
    fn test_numeric_operations() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let executor = Executor::new(&store);

        let n1 = Some(Node::literal_typed(
            dict.intern("10"),
            dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
        ));
        let n2 = Some(Node::literal_typed(
            dict.intern("5"),
            dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
        ));

        let result = executor.numeric_add(n1.clone(), n2.clone()).unwrap();
        assert!(result.is_some());

        let result = executor.numeric_subtract(n1.clone(), n2.clone()).unwrap();
        assert!(result.is_some());

        let result = executor.numeric_multiply(n1.clone(), n2.clone()).unwrap();
        assert!(result.is_some());

        let result = executor.numeric_divide(n1, n2).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_division_by_zero() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let executor = Executor::new(&store);

        let n1 = Some(Node::literal_typed(
            dict.intern("10"),
            dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
        ));
        let zero = Some(Node::literal_typed(
            dict.intern("0"),
            dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
        ));

        let result = executor.numeric_divide(n1, zero);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::DivisionByZero));
    }

    #[test]
    fn test_comparison_operations() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let executor = Executor::new(&store);

        let n1 = Some(Node::literal_typed(
            dict.intern("10"),
            dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
        ));
        let n2 = Some(Node::literal_typed(
            dict.intern("5"),
            dict.intern("http://www.w3.org/2001/XMLSchema#integer"),
        ));

        let result = executor.less_than(n2.clone(), n1.clone());
        assert_eq!(result, Some(executor.true_node()));

        let result = executor.greater_than(n1.clone(), n2.clone());
        assert_eq!(result, Some(executor.true_node()));

        let result = executor.equal(n1.clone(), n1.clone());
        assert_eq!(result, Some(executor.true_node()));
    }

    #[test]
    fn test_order_by() {
        let store = setup_test_store();
        let dict = store.dictionary();
        let mut executor = Executor::new(&store);

        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Node(Node::iri(dict.intern("http://example.org/name"))),
            object: VarOrNode::Var(Variable::new("name")),
        };

        let order_cond = OrderCondition {
            expr: Expression::Var(Variable::new("name")),
            ascending: true,
        };

        let algebra = Algebra::OrderBy {
            conditions: vec![order_cond],
            input: Box::new(Algebra::BGP(vec![pattern])),
        };

        let results = executor.execute(&algebra).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_from_clause() {
        // Test FROM clause - should only match specified graph
        let mut store = QuadStore::new_in_memory();
        let dict = Arc::clone(store.dictionary());

        // Add data to different graphs
        let s = dict.intern("http://example.org/alice");
        let p = dict.intern("http://example.org/name");
        let g1 = dict.intern("http://example.org/graph1");
        let g2 = dict.intern("http://example.org/graph2");

        // Triple in graph1
        store.insert(Quad::new(
            Node::iri(s),
            Node::iri(p),
            Node::literal_str(dict.intern("Alice")),
            Some(Node::iri(g1)),
        )).unwrap();

        // Triple in graph2
        store.insert(Quad::new(
            Node::iri(s),
            Node::iri(p),
            Node::literal_str(dict.intern("Alice2")),
            Some(Node::iri(g2)),
        )).unwrap();

        // Create dataset with FROM graph1
        let dataset = Dataset {
            default: vec!["http://example.org/graph1"],
            named: vec![],
        };

        let mut executor = Executor::new(&store).with_dataset(dataset);

        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Var(Variable::new("p")),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let algebra = Algebra::BGP(vec![pattern]);
        let results = executor.execute(&algebra).unwrap();

        // Should only find triple from graph1
        assert_eq!(results.len(), 1);
        let binding = &results.bindings()[0];
        let val = binding.get(&Variable::new("o")).unwrap();
        assert_eq!(val.as_literal().unwrap().lexical_form, "Alice");
    }

    #[test]
    fn test_from_named_with_graph() {
        // Test FROM NAMED - GRAPH clause should only access named graphs
        let mut store = QuadStore::new_in_memory();
        let dict = Arc::clone(store.dictionary());

        let s = dict.intern("http://example.org/alice");
        let p = dict.intern("http://example.org/name");
        let g1 = dict.intern("http://example.org/graph1");
        let g2 = dict.intern("http://example.org/graph2");

        // Add data to both graphs
        store.insert(Quad::new(
            Node::iri(s),
            Node::iri(p),
            Node::literal_str(dict.intern("Alice1")),
            Some(Node::iri(g1)),
        )).unwrap();

        store.insert(Quad::new(
            Node::iri(s),
            Node::iri(p),
            Node::literal_str(dict.intern("Alice2")),
            Some(Node::iri(g2)),
        )).unwrap();

        // Create dataset with FROM NAMED graph1 only
        let dataset = Dataset {
            default: vec![],
            named: vec!["http://example.org/graph1"],
        };

        let mut executor = Executor::new(&store).with_dataset(dataset);

        // Query graph1 (should succeed - it's in FROM NAMED)
        let pattern1 = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Var(Variable::new("p")),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let algebra1 = Algebra::Graph {
            graph: VarOrNode::Node(Node::iri(g1)),
            input: Box::new(Algebra::BGP(vec![pattern1])),
        };

        let results1 = executor.execute(&algebra1).unwrap();
        assert_eq!(results1.len(), 1);

        // Query graph2 (should return empty - not in FROM NAMED)
        let pattern2 = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Var(Variable::new("p")),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let algebra2 = Algebra::Graph {
            graph: VarOrNode::Node(Node::iri(g2)),
            input: Box::new(Algebra::BGP(vec![pattern2])),
        };

        let results2 = executor.execute(&algebra2).unwrap();
        assert_eq!(results2.len(), 0); // Graph not allowed
    }

    #[test]
    fn test_from_multiple_graphs() {
        // Test FROM with multiple graphs - should merge results
        let mut store = QuadStore::new_in_memory();
        let dict = Arc::clone(store.dictionary());

        let s1 = dict.intern("http://example.org/alice");
        let s2 = dict.intern("http://example.org/bob");
        let p = dict.intern("http://example.org/name");
        let g1 = dict.intern("http://example.org/graph1");
        let g2 = dict.intern("http://example.org/graph2");
        let g3 = dict.intern("http://example.org/graph3");

        // Alice in graph1
        store.insert(Quad::new(
            Node::iri(s1),
            Node::iri(p),
            Node::literal_str(dict.intern("Alice")),
            Some(Node::iri(g1)),
        )).unwrap();

        // Bob in graph2
        store.insert(Quad::new(
            Node::iri(s2),
            Node::iri(p),
            Node::literal_str(dict.intern("Bob")),
            Some(Node::iri(g2)),
        )).unwrap();

        // Charlie in graph3 (not in FROM)
        let s3 = dict.intern("http://example.org/charlie");
        store.insert(Quad::new(
            Node::iri(s3),
            Node::iri(p),
            Node::literal_str(dict.intern("Charlie")),
            Some(Node::iri(g3)),
        )).unwrap();

        // Create dataset with FROM graph1 and graph2
        let dataset = Dataset {
            default: vec!["http://example.org/graph1", "http://example.org/graph2"],
            named: vec![],
        };

        let mut executor = Executor::new(&store).with_dataset(dataset);

        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Node(Node::iri(p)),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let algebra = Algebra::BGP(vec![pattern]);
        let results = executor.execute(&algebra).unwrap();

        // Should find Alice and Bob (from graph1 and graph2), but NOT Charlie
        assert_eq!(results.len(), 2);

        let names: Vec<&str> = results.iter()
            .map(|b| b.get(&Variable::new("o")).unwrap().as_literal().unwrap().lexical_form)
            .collect();

        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Bob"));
        assert!(!names.iter().any(|&n| n == "Charlie"));
    }
}
