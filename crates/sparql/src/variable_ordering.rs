//! Variable Ordering Analysis for WCOJ (Worst-Case Optimal Join)
//!
//! Analyzes triple patterns in a BGP to determine the optimal variable ordering
//! for LeapFrog TrieJoin execution. All tries must use the SAME variable ordering
//! for correct intersection.
//!
//! # Algorithm
//!
//! 1. **Collect all variables** across all patterns
//! 2. **Rank variables** by frequency and join selectivity
//! 3. **Order variables** from most selective to least selective
//! 4. **Return canonical ordering** to be used by all tries
//!
//! # Example
//!
//! Given patterns:
//! ```sparql
//! ?person foaf:name ?name .
//! ?person foaf:age ?age .
//! ?person foaf:email ?email .
//! ```
//!
//! Ordering: `[?person, ?name, ?age, ?email]`
//! - `?person` appears in 3 patterns (most frequent → first)
//! - Other variables appear once each (ordered arbitrarily)

use crate::{TriplePattern, VarOrNode, Variable};
use std::collections::{HashMap, HashSet};

/// Variable ordering analysis result
#[derive(Debug, Clone)]
pub struct VariableOrdering<'a> {
    /// Canonical ordering of variables
    pub variables: Vec<Variable<'a>>,

    /// Frequency of each variable (# of patterns it appears in)
    pub frequencies: HashMap<Variable<'a>, usize>,

    /// Position of each variable in the ordering
    pub positions: HashMap<Variable<'a>, usize>,
}

impl<'a> VariableOrdering<'a> {
    /// Analyze patterns and determine optimal variable ordering
    ///
    /// Uses frequency-based heuristic: variables that appear in more patterns
    /// should be ordered earlier (more selective joins).
    pub fn analyze(patterns: &[TriplePattern<'a>]) -> Self {
        if patterns.is_empty() {
            return Self {
                variables: vec![],
                frequencies: HashMap::new(),
                positions: HashMap::new(),
            };
        }

        // Step 1: Collect all variables and count their frequencies
        let mut frequencies = HashMap::new();
        let mut all_vars = HashSet::new();

        for pattern in patterns {
            // Count subject variables
            if let VarOrNode::Var(var) = &pattern.subject {
                *frequencies.entry(var.clone()).or_insert(0) += 1;
                all_vars.insert(var.clone());
            }

            // Count predicate variables
            if let VarOrNode::Var(var) = &pattern.predicate {
                *frequencies.entry(var.clone()).or_insert(0) += 1;
                all_vars.insert(var.clone());
            }

            // Count object variables
            if let VarOrNode::Var(var) = &pattern.object {
                *frequencies.entry(var.clone()).or_insert(0) += 1;
                all_vars.insert(var.clone());
            }
        }

        // Step 2: Sort variables by frequency (descending)
        // Variables that appear in more patterns should be ordered first
        let mut variables: Vec<Variable<'a>> = all_vars.into_iter().collect();
        variables.sort_by(|a, b| {
            let freq_a = frequencies.get(a).unwrap_or(&0);
            let freq_b = frequencies.get(b).unwrap_or(&0);

            // Primary: frequency (descending - more frequent first)
            match freq_b.cmp(freq_a) {
                std::cmp::Ordering::Equal => {
                    // Secondary: alphabetical (for deterministic ordering)
                    a.name.cmp(&b.name)
                }
                other => other,
            }
        });

        // Step 3: Build position map for fast lookup
        let positions: HashMap<Variable<'a>, usize> = variables
            .iter()
            .enumerate()
            .map(|(idx, var)| (var.clone(), idx))
            .collect();

        Self {
            variables,
            frequencies,
            positions,
        }
    }

    /// Get the position of a variable in the ordering
    ///
    /// Returns None if variable is not in the ordering (constant-only patterns).
    pub fn position(&self, var: &Variable<'a>) -> Option<usize> {
        self.positions.get(var).copied()
    }

    /// Get the number of variables in the ordering
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if ordering is empty
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Extract variables from a pattern in canonical order
    ///
    /// Returns a vector of (Variable, position_in_triple) tuples.
    /// position_in_triple: 0 = subject, 1 = predicate, 2 = object
    pub fn extract_pattern_variables(&self, pattern: &TriplePattern<'a>) -> Vec<(Variable<'a>, usize)> {
        let mut vars = Vec::new();

        if let VarOrNode::Var(var) = &pattern.subject {
            if self.position(var).is_some() {
                vars.push((var.clone(), 0)); // Subject position
            }
        }

        if let VarOrNode::Var(var) = &pattern.predicate {
            if self.position(var).is_some() {
                vars.push((var.clone(), 1)); // Predicate position
            }
        }

        if let VarOrNode::Var(var) = &pattern.object {
            if self.position(var).is_some() {
                vars.push((var.clone(), 2)); // Object position
            }
        }

        // Sort by canonical ordering position
        vars.sort_by_key(|(var, _triple_pos)| self.position(var).unwrap_or(usize::MAX));

        vars
    }

    /// Convert pattern to node sequence using canonical variable ordering
    ///
    /// For WCOJ trie construction, we need to extract nodes in canonical order.
    /// This method returns the pattern's nodes ordered by variable frequency.
    ///
    /// # Example
    ///
    /// Pattern: `?person foaf:name ?name`
    /// Ordering: `[?person, ?name, ...]`
    /// Result: `[?person_node, foaf:name, ?name_node]` in order
    pub fn pattern_to_canonical_nodes(
        &self,
        pattern: &'a TriplePattern<'a>,
    ) -> Vec<(Variable<'a>, &'a VarOrNode<'a>)> {
        // Collect (variable, node) pairs from pattern
        let mut pairs = Vec::new();

        if let VarOrNode::Var(var) = &pattern.subject {
            pairs.push((var.clone(), &pattern.subject));
        }
        if let VarOrNode::Var(var) = &pattern.predicate {
            pairs.push((var.clone(), &pattern.predicate));
        }
        if let VarOrNode::Var(var) = &pattern.object {
            pairs.push((var.clone(), &pattern.object));
        }

        // Sort by canonical ordering
        pairs.sort_by_key(|(var, _)| self.position(var).unwrap_or(usize::MAX));

        pairs
    }
}

/// Variable ordering strategy for query optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingStrategy {
    /// Frequency-based: most frequent variables first
    Frequency,

    /// Selectivity-based: most selective variables first (future)
    Selectivity,

    /// Hybrid: combine frequency and selectivity (future)
    Hybrid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdf_model::{Dictionary, Node};
    use std::sync::Arc;

    fn create_test_patterns(dict: &Arc<Dictionary>) -> Vec<TriplePattern<'static>> {
        let name = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/name"));
        let age = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/age"));
        let email = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/email"));

        vec![
            TriplePattern {
                subject: VarOrNode::Var(Variable::new("person")),
                predicate: VarOrNode::Node(name),
                object: VarOrNode::Var(Variable::new("name")),
            },
            TriplePattern {
                subject: VarOrNode::Var(Variable::new("person")),
                predicate: VarOrNode::Node(age),
                object: VarOrNode::Var(Variable::new("age")),
            },
            TriplePattern {
                subject: VarOrNode::Var(Variable::new("person")),
                predicate: VarOrNode::Node(email),
                object: VarOrNode::Var(Variable::new("email")),
            },
        ]
    }

    #[test]
    fn test_variable_ordering_star_query() {
        let dict = Arc::new(Dictionary::new());
        let patterns = create_test_patterns(&dict);

        let ordering = VariableOrdering::analyze(&patterns);

        // Should have 4 variables: person, name, age, email
        assert_eq!(ordering.len(), 4);

        // person appears 3 times, others appear once
        assert_eq!(*ordering.frequencies.get(&Variable::new("person")).unwrap(), 3);
        assert_eq!(*ordering.frequencies.get(&Variable::new("name")).unwrap(), 1);
        assert_eq!(*ordering.frequencies.get(&Variable::new("age")).unwrap(), 1);
        assert_eq!(*ordering.frequencies.get(&Variable::new("email")).unwrap(), 1);

        // person should be first (most frequent)
        assert_eq!(ordering.variables[0], Variable::new("person"));

        // Get position
        assert_eq!(ordering.position(&Variable::new("person")), Some(0));
    }

    #[test]
    fn test_empty_patterns() {
        let ordering = VariableOrdering::analyze(&[]);
        assert!(ordering.is_empty());
        assert_eq!(ordering.len(), 0);
    }

    #[test]
    fn test_extract_pattern_variables() {
        let dict = Arc::new(Dictionary::new());
        let patterns = create_test_patterns(&dict);

        let ordering = VariableOrdering::analyze(&patterns);

        // First pattern: ?person foaf:name ?name
        let vars = ordering.extract_pattern_variables(&patterns[0]);

        // Should have 2 variables (person and name)
        assert_eq!(vars.len(), 2);

        // person should be first (most frequent)
        assert_eq!(vars[0].0, Variable::new("person"));
        assert_eq!(vars[0].1, 0); // Subject position

        // name should be second
        assert_eq!(vars[1].0, Variable::new("name"));
        assert_eq!(vars[1].1, 2); // Object position
    }

    #[test]
    fn test_pattern_to_canonical_nodes() {
        let dict = Arc::new(Dictionary::new());
        let patterns = create_test_patterns(&dict);

        let ordering = VariableOrdering::analyze(&patterns);

        // First pattern: ?person foaf:name ?name
        let nodes = ordering.pattern_to_canonical_nodes(&patterns[0]);

        // Should have 2 variable nodes
        assert_eq!(nodes.len(), 2);

        // First should be person (most frequent)
        assert_eq!(nodes[0].0, Variable::new("person"));

        // Second should be name
        assert_eq!(nodes[1].0, Variable::new("name"));
    }

    #[test]
    fn test_multi_pattern_join() {
        let dict = Arc::new(Dictionary::new());
        let knows = Node::iri(dict.intern("http://xmlns.com/foaf/0.1/knows"));

        // Friend-of-friend pattern
        let patterns = vec![
            TriplePattern {
                subject: VarOrNode::Var(Variable::new("person1")),
                predicate: VarOrNode::Node(knows.clone()),
                object: VarOrNode::Var(Variable::new("person2")),
            },
            TriplePattern {
                subject: VarOrNode::Var(Variable::new("person2")),
                predicate: VarOrNode::Node(knows),
                object: VarOrNode::Var(Variable::new("person3")),
            },
        ];

        let ordering = VariableOrdering::analyze(&patterns);

        // Should have 3 variables
        assert_eq!(ordering.len(), 3);

        // person2 appears twice (most frequent)
        assert_eq!(*ordering.frequencies.get(&Variable::new("person2")).unwrap(), 2);
        assert_eq!(*ordering.frequencies.get(&Variable::new("person1")).unwrap(), 1);
        assert_eq!(*ordering.frequencies.get(&Variable::new("person3")).unwrap(), 1);

        // person2 should be first
        assert_eq!(ordering.variables[0], Variable::new("person2"));
    }
}
