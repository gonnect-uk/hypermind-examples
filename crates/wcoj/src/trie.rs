//! Trie data structure for WCOJ (Worst-Case Optimal Joins)
//!
//! A trie provides sorted access to triples/quads organized hierarchically.
//! Used by LeapFrog iterator to efficiently find intersections.
//!
//! # Structure
//!
//! For SPOC index with triples: `(s1, p1, o1), (s1, p2, o2), (s2, p1, o3)`
//!
//! ```text
//! Root
//!  ├─ s1
//!  │   ├─ p1 → o1
//!  │   └─ p2 → o2
//!  └─ s2
//!      └─ p1 → o3
//! ```

use rdf_model::{Node, Quad};
use std::collections::BTreeMap;

/// Position in a triple pattern: Subject, Predicate, or Object
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriplePosition {
    Subject,
    Predicate,
    Object,
}

/// A trie level containing sorted values and children
#[derive(Debug, Clone)]
pub struct TrieLevel<'a> {
    /// Sorted values at this level (using BTreeMap for sorted iteration)
    values: BTreeMap<Node<'a>, Box<TrieLevel<'a>>>,
    /// Leaf values (if this is the last level)
    leaves: Vec<Node<'a>>,
    /// Is this a leaf level?
    is_leaf: bool,
}

impl<'a> TrieLevel<'a> {
    /// Create a new trie level
    pub fn new(is_leaf: bool) -> Self {
        Self {
            values: BTreeMap::new(),
            leaves: Vec::new(),
            is_leaf,
        }
    }

    /// Insert a path into the trie
    pub fn insert(&mut self, path: &[Node<'a>]) {
        if path.is_empty() {
            return;
        }

        if path.len() == 1 {
            // Leaf node
            if !self.leaves.contains(&path[0]) {
                self.leaves.push(path[0].clone());
                self.leaves.sort();
            }
        } else {
            // Internal node
            let key = path[0].clone();
            let rest = &path[1..];

            let child = self.values.entry(key).or_insert_with(|| {
                Box::new(TrieLevel::new(rest.len() == 1))
            });

            child.insert(rest);
        }
    }

    /// Get sorted values at this level
    pub fn sorted_values(&self) -> Vec<Node<'a>> {
        if self.is_leaf {
            self.leaves.clone()
        } else {
            self.values.keys().cloned().collect()
        }
    }

    /// Get child for a given value
    pub fn get_child(&self, value: &Node<'a>) -> Option<&TrieLevel<'a>> {
        self.values.get(value).map(|b| b.as_ref())
    }

    /// Check if this level is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty() && self.leaves.is_empty()
    }

    /// Get the number of values at this level
    pub fn len(&self) -> usize {
        if self.is_leaf {
            self.leaves.len()
        } else {
            self.values.len()
        }
    }
}

/// Trie for efficient sorted access to quads
///
/// Organizes quads hierarchically for LeapFrog iteration.
/// Different index orders (SPOC, POCS, etc.) create different trie structures.
#[derive(Debug, Clone)]
pub struct Trie<'a> {
    /// Root level of the trie
    root: TrieLevel<'a>,
    /// Depth of the trie (typically 3 for S-P-O)
    depth: usize,
    /// Current position in the trie (for iteration)
    current_path: Vec<Node<'a>>,
    /// Current level in traversal
    current_level: usize,
}

impl<'a> Trie<'a> {
    /// Create a new empty trie
    pub fn new(depth: usize) -> Self {
        Self {
            root: TrieLevel::new(false),
            depth,
            current_path: Vec::new(),
            current_level: 0,
        }
    }

    /// Build trie from quads with specific ordering
    ///
    /// # Arguments
    ///
    /// * `quads` - Iterator of quads to insert
    /// * `ordering` - Order of positions [S, P, O] in trie
    ///
    /// # Example
    ///
    /// ```ignore
    /// // SPOC index: organize by Subject -> Predicate -> Object
    /// let trie = Trie::from_quads(quads, &[Subject, Predicate, Object]);
    /// ```
    pub fn from_quads<I>(quads: I, ordering: &[TriplePosition]) -> Self
    where
        I: IntoIterator<Item = Quad<'a>>,
    {
        let mut trie = Self::new(ordering.len());

        for quad in quads {
            let path = Self::extract_path(&quad, ordering);
            trie.root.insert(&path);
        }

        trie
    }

    /// Extract path from quad based on ordering
    fn extract_path(quad: &Quad<'a>, ordering: &[TriplePosition]) -> Vec<Node<'a>> {
        ordering
            .iter()
            .map(|pos| match pos {
                TriplePosition::Subject => quad.subject.clone(),
                TriplePosition::Predicate => quad.predicate.clone(),
                TriplePosition::Object => quad.object.clone(),
            })
            .collect()
    }

    /// Get sorted values at the root level
    pub fn root_values(&self) -> Vec<Node<'a>> {
        self.root.sorted_values()
    }

    /// Seek to a specific value at current level
    ///
    /// Returns true if value exists, false otherwise.
    /// Advances iterator to the value or next greater value.
    pub fn seek(&mut self, value: &Node<'a>) -> bool {
        let level = self.get_current_level();
        let values = level.sorted_values();

        // Binary search for value or next greater
        match values.binary_search_by(|v| v.cmp(value)) {
            Ok(idx) => {
                // Exact match found
                self.current_path.truncate(self.current_level);
                self.current_path.push(values[idx].clone());
                true
            }
            Err(idx) => {
                // Not found - seek to next greater value
                if idx < values.len() {
                    self.current_path.truncate(self.current_level);
                    self.current_path.push(values[idx].clone());
                    false
                } else {
                    // Value is greater than all values - move to end
                    self.current_level = self.depth; // Mark as at end
                    false
                }
            }
        }
    }

    /// Move to next value at current level
    pub fn next(&mut self) -> Option<Node<'a>> {
        let level = self.get_current_level();
        let values = level.sorted_values();

        if values.is_empty() {
            return None;
        }

        // If at start, return first value
        if self.current_path.len() <= self.current_level {
            let first = values[0].clone();
            self.current_path.truncate(self.current_level);
            self.current_path.push(first.clone());
            return Some(first);
        }

        // Find current position and move to next
        let current_value = &self.current_path[self.current_level];
        if let Ok(idx) = values.binary_search_by(|v| v.cmp(current_value)) {
            if idx + 1 < values.len() {
                let next = values[idx + 1].clone();
                self.current_path.truncate(self.current_level);
                self.current_path.push(next.clone());
                return Some(next);
            }
        }

        // No next value - at end
        self.current_level = self.depth;
        None
    }

    /// Check if iterator is at end
    pub fn at_end(&self) -> bool {
        self.current_level >= self.depth
    }

    /// Move to child level (go deeper in trie)
    pub fn open(&mut self) -> bool {
        if self.current_level >= self.depth - 1 {
            return false; // Already at leaf level
        }

        if self.current_path.len() <= self.current_level {
            return false; // No current value
        }

        self.current_level += 1;
        true
    }

    /// Move to parent level (go up in trie)
    pub fn up(&mut self) -> bool {
        if self.current_level == 0 {
            return false; // Already at root
        }

        self.current_level -= 1;
        self.current_path.truncate(self.current_level);
        true
    }

    /// Get current level in trie
    fn get_current_level(&self) -> &TrieLevel<'a> {
        let mut level = &self.root;

        for i in 0..self.current_level {
            if i < self.current_path.len() {
                if let Some(child) = level.get_child(&self.current_path[i]) {
                    level = child;
                } else {
                    break;
                }
            }
        }

        level
    }

    /// Get current value at current level
    pub fn current(&self) -> Option<Node<'a>> {
        if self.current_path.len() > self.current_level {
            Some(self.current_path[self.current_level].clone())
        } else {
            None
        }
    }

    /// Reset iterator to beginning
    pub fn reset(&mut self) {
        self.current_path.clear();
        self.current_level = 0;
    }

    /// Get the depth of the trie
    pub fn depth(&self) -> usize {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdf_model::Dictionary;
    use std::sync::Arc;

    fn create_test_quads(dict: &Arc<Dictionary>) -> Vec<Quad<'static>> {
        let s1 = dict.intern("http://example.org/s1");
        let s2 = dict.intern("http://example.org/s2");
        let p1 = dict.intern("http://example.org/p1");
        let p2 = dict.intern("http://example.org/p2");
        let o1 = dict.intern("http://example.org/o1");
        let o2 = dict.intern("http://example.org/o2");
        let o3 = dict.intern("http://example.org/o3");

        vec![
            Quad::new(Node::iri(s1), Node::iri(p1), Node::iri(o1), None),
            Quad::new(Node::iri(s1), Node::iri(p2), Node::iri(o2), None),
            Quad::new(Node::iri(s2), Node::iri(p1), Node::iri(o3), None),
        ]
    }

    #[test]
    fn test_trie_creation() {
        let dict = Arc::new(Dictionary::new());
        let quads = create_test_quads(&dict);

        let trie = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        // Should have 2 root values (s1, s2)
        let root_values = trie.root_values();
        assert_eq!(root_values.len(), 2);
    }

    #[test]
    fn test_trie_seek() {
        let dict = Arc::new(Dictionary::new());
        let quads = create_test_quads(&dict);

        let mut trie = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let s1 = dict.intern("http://example.org/s1");
        let found = trie.seek(&Node::iri(s1));

        assert!(found);
        assert_eq!(trie.current(), Some(Node::iri(s1)));
    }

    #[test]
    fn test_trie_next() {
        let dict = Arc::new(Dictionary::new());
        let quads = create_test_quads(&dict);

        let mut trie = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        // First value
        let first = trie.next();
        assert!(first.is_some());

        // Second value
        let second = trie.next();
        assert!(second.is_some());
        assert_ne!(first, second);

        // No third value at root level
        let third = trie.next();
        assert!(third.is_none());
    }

    #[test]
    fn test_trie_open() {
        let dict = Arc::new(Dictionary::new());
        let quads = create_test_quads(&dict);

        let mut trie = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        // Move to first subject
        trie.next();

        // Open to predicate level
        assert!(trie.open());

        // Should have predicates now
        let pred = trie.next();
        assert!(pred.is_some());
    }

    #[test]
    fn test_trie_iteration() {
        let dict = Arc::new(Dictionary::new());
        let quads = create_test_quads(&dict);

        let mut trie = Trie::from_quads(
            quads.clone(),
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let mut count = 0;

        // Iterate through all subjects
        while let Some(_subj) = trie.next() {
            count += 1;
            if trie.open() {
                // Iterate through predicates
                while let Some(_pred) = trie.next() {
                    if trie.open() {
                        // Iterate through objects
                        while let Some(_obj) = trie.next() {
                            count += 1;
                        }
                        trie.up();
                    }
                }
                trie.up();
            }
        }

        assert!(count > 0);
    }
}
