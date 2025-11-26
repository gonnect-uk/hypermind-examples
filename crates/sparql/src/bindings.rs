//! Solution bindings and result sets
//!
//! Represents the bindings produced during SPARQL query execution.
//! A binding maps variables to RDF terms.

use rdf_model::Node;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::Variable;

/// A single solution mapping from variables to nodes
///
/// Represents one row in a SPARQL result set. Variables are stored
/// in sorted order (BTreeMap) for deterministic iteration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding<'a> {
    /// Variable to node mappings (sorted by variable name)
    bindings: BTreeMap<Variable<'a>, Node<'a>>,
}

impl<'a> Binding<'a> {
    /// Create a new empty binding
    pub fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
        }
    }

    /// Create a binding from an iterator of (variable, node) pairs
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Variable<'a>, Node<'a>)>,
    {
        Self {
            bindings: iter.into_iter().collect(),
        }
    }

    /// Bind a variable to a node
    ///
    /// Returns `true` if the binding is compatible (variable was unbound or bound to same value),
    /// `false` if there's a conflict (variable bound to different value).
    pub fn bind(&mut self, var: Variable<'a>, node: Node<'a>) -> bool {
        match self.bindings.get(&var) {
            Some(existing) => existing == &node, // Compatible if same value
            None => {
                self.bindings.insert(var, node);
                true
            }
        }
    }

    /// Get the binding for a variable
    pub fn get(&self, var: &Variable<'a>) -> Option<&Node<'a>> {
        self.bindings.get(var)
    }

    /// Check if a variable is bound
    pub fn contains(&self, var: &Variable<'a>) -> bool {
        self.bindings.contains_key(var)
    }

    /// Get all bound variables
    pub fn variables(&self) -> impl Iterator<Item = &Variable<'a>> {
        self.bindings.keys()
    }

    /// Get all bindings as (variable, node) pairs
    pub fn iter(&self) -> impl Iterator<Item = (&Variable<'a>, &Node<'a>)> {
        self.bindings.iter()
    }

    /// Get the number of bound variables
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if binding is empty (no variables bound)
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Merge two bindings
    ///
    /// Returns `Some(merged)` if bindings are compatible, `None` if there's a conflict.
    pub fn merge(&self, other: &Binding<'a>) -> Option<Binding<'a>> {
        let mut result = self.clone();

        for (var, node) in other.iter() {
            if !result.bind(var.clone(), node.clone()) {
                return None; // Conflict
            }
        }

        Some(result)
    }

    /// Check if this binding is compatible with another
    ///
    /// Two bindings are compatible if they agree on all shared variables.
    pub fn compatible_with(&self, other: &Binding<'a>) -> bool {
        for (var, node) in &self.bindings {
            if let Some(other_node) = other.get(var) {
                if node != other_node {
                    return false;
                }
            }
        }
        true
    }

    /// Project binding to only include specified variables
    pub fn project(&self, vars: &[Variable<'a>]) -> Binding<'a> {
        let bindings: BTreeMap<_, _> = vars
            .iter()
            .filter_map(|var| self.get(var).map(|node| (var.clone(), node.clone())))
            .collect();

        Binding { bindings }
    }

    /// Extend binding with a new variable (used for EXTEND operator)
    pub fn extend(&mut self, var: Variable<'a>, node: Node<'a>) {
        self.bindings.insert(var, node);
    }
}

impl<'a> Default for Binding<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> fmt::Display for Binding<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ ")?;
        for (i, (var, node)) in self.bindings.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} = {}", var, node)?;
        }
        write!(f, " }}")
    }
}

/// A set of solution bindings (result set)
///
/// Represents the complete result of a SPARQL query execution.
/// Bindings are stored in a Vec to preserve order.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSet<'a> {
    /// The bindings in this set
    bindings: Vec<Binding<'a>>,
}

impl<'a> BindingSet<'a> {
    /// Create a new empty binding set
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Create a binding set with a single empty binding
    ///
    /// This represents the "identity" for join operations.
    pub fn unit() -> Self {
        Self {
            bindings: vec![Binding::new()],
        }
    }

    /// Create a binding set from a vector of bindings
    pub fn from_bindings(bindings: Vec<Binding<'a>>) -> Self {
        Self { bindings }
    }

    /// Add a binding to the set
    pub fn add(&mut self, binding: Binding<'a>) {
        self.bindings.push(binding);
    }

    /// Get all bindings
    pub fn bindings(&self) -> &[Binding<'a>] {
        &self.bindings
    }

    /// Get mutable bindings
    pub fn bindings_mut(&mut self) -> &mut Vec<Binding<'a>> {
        &mut self.bindings
    }

    /// Iterate over bindings
    pub fn iter(&self) -> impl Iterator<Item = &Binding<'a>> {
        self.bindings.iter()
    }

    /// Get the number of bindings
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if binding set is empty
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Get all variables mentioned in any binding
    pub fn variables(&self) -> BTreeSet<Variable<'a>> {
        let mut vars = BTreeSet::new();
        for binding in &self.bindings {
            for var in binding.variables() {
                vars.insert(var.clone());
            }
        }
        vars
    }

    /// Remove duplicate bindings (DISTINCT)
    pub fn distinct(&mut self) {
        // Deduplication without requiring Ord - use seen set
        let mut seen = std::collections::HashSet::new();
        self.bindings.retain(|binding| {
            // Create a comparable key from the binding
            let key: Vec<_> = binding
                .variables()
                .map(|v| (v.name, binding.get(v).map(|n| format!("{:?}", n))))
                .collect();
            seen.insert(key)
        });
    }

    /// Take only the first n bindings (LIMIT)
    pub fn limit(&mut self, n: usize) {
        self.bindings.truncate(n);
    }

    /// Skip the first n bindings (OFFSET)
    pub fn offset(&mut self, n: usize) {
        if n < self.bindings.len() {
            self.bindings.drain(0..n);
        } else {
            self.bindings.clear();
        }
    }

    /// Project all bindings to only include specified variables
    pub fn project(&mut self, vars: &[Variable<'a>]) {
        for binding in &mut self.bindings {
            *binding = binding.project(vars);
        }
    }

    /// Sort bindings using a comparison function
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Binding<'a>, &Binding<'a>) -> std::cmp::Ordering,
    {
        self.bindings.sort_by(compare);
    }

    /// Extend this binding set with another (UNION)
    pub fn union(&mut self, other: BindingSet<'a>) {
        self.bindings.extend(other.bindings);
    }

    /// Filter bindings using a predicate
    pub fn filter<F>(&mut self, predicate: F)
    where
        F: Fn(&Binding<'a>) -> bool,
    {
        self.bindings.retain(predicate);
    }

    /// Perform inner join with another binding set
    pub fn join(&self, other: &BindingSet<'a>) -> BindingSet<'a> {
        let mut result = BindingSet::new();

        for left in &self.bindings {
            for right in &other.bindings {
                if let Some(merged) = left.merge(right) {
                    result.add(merged);
                }
            }
        }

        result
    }

    /// Perform left join (OPTIONAL) with another binding set
    ///
    /// For each binding in self, include it merged with compatible bindings from other.
    /// If no compatible bindings exist in other, include the original binding.
    pub fn left_join<F>(&self, other: &BindingSet<'a>, filter: F) -> BindingSet<'a>
    where
        F: Fn(&Binding<'a>) -> bool,
    {
        let mut result = BindingSet::new();

        for left in &self.bindings {
            let mut found = false;

            for right in &other.bindings {
                if let Some(merged) = left.merge(right) {
                    if filter(&merged) {
                        result.add(merged);
                        found = true;
                    }
                }
            }

            // If no compatible binding found, keep original
            if !found {
                result.add(left.clone());
            }
        }

        result
    }

    /// Perform minus operation (remove matching bindings)
    ///
    /// Remove bindings from self that are compatible with any binding in other.
    pub fn minus(&self, other: &BindingSet<'a>) -> BindingSet<'a> {
        let mut result = BindingSet::new();

        for left in &self.bindings {
            let mut compatible = false;

            for right in &other.bindings {
                if left.compatible_with(right) {
                    compatible = true;
                    break;
                }
            }

            if !compatible {
                result.add(left.clone());
            }
        }

        result
    }
}

impl<'a> Default for BindingSet<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for BindingSet<'a> {
    type Item = Binding<'a>;
    type IntoIter = std::vec::IntoIter<Binding<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.bindings.into_iter()
    }
}

impl<'a> FromIterator<Binding<'a>> for BindingSet<'a> {
    fn from_iter<I: IntoIterator<Item = Binding<'a>>>(iter: I) -> Self {
        Self {
            bindings: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdf_model::Dictionary;

    #[test]
    fn test_binding_creation() {
        let binding = Binding::new();
        assert!(binding.is_empty());
        assert_eq!(binding.len(), 0);
    }

    #[test]
    fn test_binding_bind() {
        let mut binding = Binding::new();
        let dict = Dictionary::new();
        let node = Node::iri(dict.intern("http://example.org/test"));
        let var = Variable::new("x");

        assert!(binding.bind(var.clone(), node.clone()));
        assert_eq!(binding.get(&var), Some(&node));
        assert!(binding.contains(&var));

        // Rebind to same value should succeed
        assert!(binding.bind(var.clone(), node.clone()));

        // Bind to different value should fail
        let other_node = Node::iri(dict.intern("http://example.org/other"));
        assert!(!binding.bind(var.clone(), other_node));
    }

    #[test]
    fn test_binding_merge() {
        let dict = Dictionary::new();
        let node1 = Node::iri(dict.intern("http://example.org/1"));
        let node2 = Node::iri(dict.intern("http://example.org/2"));

        let mut b1 = Binding::new();
        b1.bind(Variable::new("x"), node1.clone());

        let mut b2 = Binding::new();
        b2.bind(Variable::new("y"), node2.clone());

        // Compatible merge
        let merged = b1.merge(&b2).unwrap();
        assert_eq!(merged.get(&Variable::new("x")), Some(&node1));
        assert_eq!(merged.get(&Variable::new("y")), Some(&node2));

        // Incompatible merge
        let mut b3 = Binding::new();
        b3.bind(Variable::new("x"), node2.clone());

        assert!(b1.merge(&b3).is_none());
    }

    #[test]
    fn test_binding_project() {
        let dict = Dictionary::new();
        let node1 = Node::iri(dict.intern("http://example.org/1"));
        let node2 = Node::iri(dict.intern("http://example.org/2"));

        let mut binding = Binding::new();
        binding.bind(Variable::new("x"), node1.clone());
        binding.bind(Variable::new("y"), node2.clone());

        let projected = binding.project(&[Variable::new("x")]);
        assert_eq!(projected.get(&Variable::new("x")), Some(&node1));
        assert_eq!(projected.get(&Variable::new("y")), None);
        assert_eq!(projected.len(), 1);
    }

    #[test]
    fn test_binding_set_unit() {
        let unit = BindingSet::unit();
        assert_eq!(unit.len(), 1);
        assert!(unit.bindings()[0].is_empty());
    }

    #[test]
    fn test_binding_set_distinct() {
        let dict = Dictionary::new();
        let node = Node::iri(dict.intern("http://example.org/test"));

        let mut b1 = Binding::new();
        b1.bind(Variable::new("x"), node.clone());

        let mut set = BindingSet::new();
        set.add(b1.clone());
        set.add(b1.clone());
        set.add(b1.clone());

        assert_eq!(set.len(), 3);
        set.distinct();
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_binding_set_join() {
        let dict = Dictionary::new();
        let node1 = Node::iri(dict.intern("http://example.org/1"));
        let node2 = Node::iri(dict.intern("http://example.org/2"));

        let mut b1 = Binding::new();
        b1.bind(Variable::new("x"), node1.clone());

        let mut b2 = Binding::new();
        b2.bind(Variable::new("y"), node2.clone());

        let set1 = BindingSet::from_bindings(vec![b1]);
        let set2 = BindingSet::from_bindings(vec![b2]);

        let joined = set1.join(&set2);
        assert_eq!(joined.len(), 1);
        assert_eq!(joined.bindings()[0].len(), 2);
    }

    #[test]
    fn test_binding_set_union() {
        let dict = Dictionary::new();
        let node1 = Node::iri(dict.intern("http://example.org/1"));
        let node2 = Node::iri(dict.intern("http://example.org/2"));

        let mut b1 = Binding::new();
        b1.bind(Variable::new("x"), node1);

        let mut b2 = Binding::new();
        b2.bind(Variable::new("x"), node2);

        let mut set1 = BindingSet::from_bindings(vec![b1]);
        let set2 = BindingSet::from_bindings(vec![b2]);

        set1.union(set2);
        assert_eq!(set1.len(), 2);
    }

    #[test]
    fn test_binding_set_filter() {
        let dict = Dictionary::new();
        let node1 = Node::iri(dict.intern("http://example.org/1"));
        let node2 = Node::iri(dict.intern("http://example.org/2"));

        let mut b1 = Binding::new();
        b1.bind(Variable::new("x"), node1.clone());

        let mut b2 = Binding::new();
        b2.bind(Variable::new("x"), node2);

        let mut set = BindingSet::from_bindings(vec![b1, b2]);

        set.filter(|b| {
            b.get(&Variable::new("x"))
                .map(|n| n == &node1)
                .unwrap_or(false)
        });

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_binding_set_minus() {
        let dict = Dictionary::new();
        let node = Node::iri(dict.intern("http://example.org/test"));

        let mut b1 = Binding::new();
        b1.bind(Variable::new("x"), node.clone());

        let mut b2 = Binding::new();
        b2.bind(Variable::new("x"), node);

        let set1 = BindingSet::from_bindings(vec![b1]);
        let set2 = BindingSet::from_bindings(vec![b2]);

        let result = set1.minus(&set2);
        assert_eq!(result.len(), 0);
    }
}
