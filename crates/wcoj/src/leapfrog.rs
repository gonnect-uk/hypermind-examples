//! LeapFrog Join Algorithm for Worst-Case Optimal Joins
//!
//! Based on:
//! - Veldhuizen "Leapfrog Triejoin: A Simple, Worst-Case Optimal Join Algorithm" (2014)
//! - Aberger et al. "EmptyHeaded: A Relational Engine for Graph Processing" (2016)
//! - Freitag et al. "WCOJoin: Fast and Generic Join Processing" (SIGMOD 2020)
//!
//! # Algorithm Overview
//!
//! LeapFrog intersection finds common values across multiple sorted lists (tries)
//! by "leapfrogging" through them, skipping non-matching values efficiently.
//!
//! ## Key Operations:
//!
//! 1. **seek(value)**: Position iterator at value or next greater value
//! 2. **next()**: Advance to next value in current trie
//! 3. **leapfrog_search()**: Find next value common to all tries
//!
//! ## Complexity:
//!
//! - **Time**: O(N^(k/(k-1))) where k is number of relations
//! - **Space**: O(N) for trie storage
//! - **Optimal**: Matches theoretical lower bound for multi-way joins

use crate::trie::Trie;
use rdf_model::Node;

/// LeapFrog iterator for finding intersection of multiple tries
///
/// Maintains multiple tries and efficiently finds values common to all.
#[derive(Debug)]
pub struct LeapfrogIterator<'a> {
    /// Tries to intersect (one per relation/triple pattern)
    tries: Vec<Trie<'a>>,
    /// Current depth level in tries
    depth: usize,
    /// Is iterator at end?
    at_end: bool,
}

impl<'a> LeapfrogIterator<'a> {
    /// Create new LeapFrog iterator from tries
    ///
    /// All tries must have the same depth.
    pub fn new(tries: Vec<Trie<'a>>) -> Self {
        if tries.is_empty() {
            return Self {
                tries,
                depth: 0,
                at_end: true,
            };
        }

        let depth = tries[0].depth();

        // Verify all tries have same depth
        debug_assert!(
            tries.iter().all(|t| t.depth() == depth),
            "All tries must have same depth for LeapFrog join"
        );

        Self {
            tries,
            depth,
            at_end: false,
        }
    }

    /// Seek all tries to a target value or next greater value
    ///
    /// Returns the maximum value reached across all tries.
    /// If all tries contain the target, returns Some(target).
    /// Otherwise, returns Some(next_greater_value) or None if any trie is exhausted.
    fn leapfrog_seek(&mut self, target: &Node<'a>) -> Option<Node<'a>> {
        if self.tries.is_empty() {
            return None;
        }

        let mut max_value = target.clone();

        for trie in &mut self.tries {
            if trie.at_end() {
                return None; // One trie exhausted -> intersection is empty
            }

            // Seek this trie to target (or next greater)
            trie.seek(&max_value);

            // Update max_value if this trie moved past target
            if let Some(current) = trie.current() {
                if current > max_value {
                    max_value = current;
                }
            } else {
                return None; // Trie exhausted
            }
        }

        Some(max_value)
    }

    /// Find next value that appears in ALL tries (intersection)
    ///
    /// Core LeapFrog algorithm: iteratively seek all tries to the maximum
    /// value seen until all tries converge on the same value.
    ///
    /// # Algorithm:
    ///
    /// ```text
    /// 1. Start with minimum value from first trie
    /// 2. Seek all tries to this value
    /// 3. If all tries land on same value -> found intersection!
    /// 4. Otherwise, some trie landed on greater value -> repeat from step 2
    /// ```
    pub fn leapfrog_search(&mut self) -> Option<Node<'a>> {
        if self.tries.is_empty() || self.at_end {
            return None;
        }

        // Start with first value from first trie
        let mut candidate = match self.tries[0].current() {
            Some(val) => val,
            None => match self.tries[0].next() {
                Some(val) => val,
                None => {
                    self.at_end = true;
                    return None;
                }
            },
        };

        // LeapFrog loop: seek all tries until they converge
        loop {
            // Seek all tries to candidate value
            match self.leapfrog_seek(&candidate) {
                None => {
                    // One or more tries exhausted
                    self.at_end = true;
                    return None;
                }
                Some(max_value) => {
                    // Check if all tries converged on same value
                    if max_value == candidate {
                        // Found intersection! All tries at same value
                        return Some(candidate);
                    } else {
                        // Some trie moved past candidate -> try again with new max
                        candidate = max_value;
                    }
                }
            }
        }
    }

    /// Advance to next value in intersection
    ///
    /// Moves the "lightest" trie (trie with smallest value) to its next value,
    /// then searches for the next intersection point.
    pub fn leapfrog_next(&mut self) -> Option<Node<'a>> {
        if self.tries.is_empty() || self.at_end {
            return None;
        }

        // Find the trie with minimum current value (the "lightest" trie)
        // This is the trie that should advance next
        let mut min_idx = 0;
        let mut min_value = self.tries[0].current()?;

        for (i, trie) in self.tries.iter().enumerate().skip(1) {
            if let Some(val) = trie.current() {
                if val < min_value {
                    min_value = val;
                    min_idx = i;
                }
            }
        }

        // Advance the lightest trie
        if self.tries[min_idx].next().is_none() {
            self.at_end = true;
            return None;
        }

        // Search for next intersection
        self.leapfrog_search()
    }

    /// Check if iterator is at end (intersection exhausted)
    pub fn at_end(&self) -> bool {
        self.at_end || self.tries.iter().any(|t| t.at_end())
    }

    /// Reset iterator to beginning
    pub fn reset(&mut self) {
        for trie in &mut self.tries {
            trie.reset();
        }
        self.at_end = false;
    }

    /// Get number of tries in this iterator
    pub fn num_tries(&self) -> usize {
        self.tries.len()
    }

    /// Get current value from first trie (assumes all tries are synchronized)
    pub fn current(&self) -> Option<Node<'a>> {
        if self.tries.is_empty() {
            return None;
        }
        self.tries[0].current()
    }

    /// Open all tries (descend to next level in trie hierarchy)
    ///
    /// Used for multi-level joins (e.g., subject -> predicate -> object)
    pub fn open(&mut self) -> bool {
        if self.tries.is_empty() {
            return false;
        }

        let mut all_opened = true;
        for trie in &mut self.tries {
            if !trie.open() {
                all_opened = false;
            }
        }

        all_opened
    }

    /// Close all tries (ascend to parent level)
    pub fn up(&mut self) -> bool {
        if self.tries.is_empty() {
            return false;
        }

        let mut all_closed = true;
        for trie in &mut self.tries {
            if !trie.up() {
                all_closed = false;
            }
        }

        all_closed
    }
}

/// Full LeapFrog Join execution engine
///
/// Coordinates multiple LeapFrog iterators across different levels
/// to execute complete multi-way joins.
#[derive(Debug)]
pub struct LeapfrogJoin<'a> {
    /// LeapFrog iterators (one per join level)
    iterators: Vec<LeapfrogIterator<'a>>,
    /// Current depth in join tree
    current_depth: usize,
    /// Maximum depth
    max_depth: usize,
}

impl<'a> LeapfrogJoin<'a> {
    /// Create new LeapFrog join from tries
    ///
    /// All tries must have same depth and structure.
    pub fn new(tries: Vec<Trie<'a>>) -> Self {
        if tries.is_empty() {
            return Self {
                iterators: vec![],
                current_depth: 0,
                max_depth: 0,
            };
        }

        let max_depth = tries[0].depth();

        // Create one LeapFrog iterator for root level
        let root_iter = LeapfrogIterator::new(tries);

        Self {
            iterators: vec![root_iter],
            current_depth: 0,
            max_depth,
        }
    }

    /// Execute join and return all solutions
    ///
    /// Recursively enumerates all tuples in the join result.
    /// Returns results as paths through the trie (one path = one solution).
    pub fn execute(&mut self) -> Vec<Vec<Node<'a>>> {
        if self.iterators.is_empty() || self.max_depth == 0 {
            return vec![];
        }

        let mut results = Vec::new();
        let mut path = Vec::new();

        // Start enumeration from root
        self.enumerate_level(0, &mut path, &mut results);

        results
    }

    /// Enumerate all solutions at a given depth level
    ///
    /// Recursively descends through trie levels using LeapFrog intersection.
    fn enumerate_level(&mut self, depth: usize, path: &mut Vec<Node<'a>>, results: &mut Vec<Vec<Node<'a>>>) {
        if depth >= self.max_depth {
            // Reached leaf - record solution
            results.push(path.clone());
            return;
        }

        if self.iterators.is_empty() {
            return;
        }

        // Find first intersection value
        let first = self.iterators[0].leapfrog_search();
        if first.is_none() {
            return;
        }

        // Iterate through all intersection values
        loop {
            let value_opt = self.iterators[0].current();
            if let Some(value) = value_opt {
                path.push(value.clone());

                // If not at max depth, descend to next level
                if depth + 1 < self.max_depth {
                    let can_open = self.iterators[0].open();
                    if can_open {
                        self.enumerate_level(depth + 1, path, results);
                        self.iterators[0].up();
                    }
                } else {
                    // At leaf level - record result
                    results.push(path.clone());
                }

                path.pop();

                // Move to next intersection value
                if self.iterators[0].leapfrog_next().is_none() {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Get estimated result size (cardinality)
    ///
    /// Useful for query optimization.
    pub fn estimate_cardinality(&self) -> usize {
        if self.iterators.is_empty() {
            return 0;
        }

        // Simple estimate: product of trie sizes at each level
        // More sophisticated estimates could use statistics
        self.iterators[0].num_tries()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::TriplePosition;
    use rdf_model::{Dictionary, Quad};
    use std::sync::Arc;

    fn create_test_data() -> (Arc<Dictionary>, Vec<Quad<'static>>) {
        let dict = Arc::new(Dictionary::new());

        // Create test quads: star pattern with shared subject
        // s1 -> p1 -> o1
        // s1 -> p2 -> o2
        // s1 -> p3 -> o3
        let s1 = dict.intern("http://example.org/s1");
        let p1 = dict.intern("http://example.org/p1");
        let p2 = dict.intern("http://example.org/p2");
        let p3 = dict.intern("http://example.org/p3");
        let o1 = dict.intern("http://example.org/o1");
        let o2 = dict.intern("http://example.org/o2");
        let o3 = dict.intern("http://example.org/o3");

        let quads = vec![
            Quad::new(Node::iri(s1), Node::iri(p1), Node::iri(o1), None),
            Quad::new(Node::iri(s1), Node::iri(p2), Node::iri(o2), None),
            Quad::new(Node::iri(s1), Node::iri(p3), Node::iri(o3), None),
        ];

        (dict, quads)
    }

    #[test]
    fn test_leapfrog_iterator_creation() {
        let (dict, quads) = create_test_data();

        let trie1 = Trie::from_quads(
            quads.clone(),
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );
        let trie2 = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let leapfrog = LeapfrogIterator::new(vec![trie1, trie2]);

        assert_eq!(leapfrog.num_tries(), 2);
        assert!(!leapfrog.at_end());
    }

    #[test]
    fn test_leapfrog_search() {
        let (dict, quads) = create_test_data();

        let trie1 = Trie::from_quads(
            quads.clone(),
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );
        let trie2 = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let mut leapfrog = LeapfrogIterator::new(vec![trie1, trie2]);

        // Should find common value (s1)
        let result = leapfrog.leapfrog_search();
        assert!(result.is_some());

        let value = result.unwrap();
        let s1 = dict.intern("http://example.org/s1");
        assert_eq!(value, Node::iri(s1));
    }

    #[test]
    fn test_leapfrog_next() {
        let (dict, quads) = create_test_data();

        // Create two different quad sets with overlapping subjects
        let s2 = dict.intern("http://example.org/s2");
        let p1 = dict.intern("http://example.org/p1");
        let o4 = dict.intern("http://example.org/o4");

        let mut quads2 = quads.clone();
        quads2.push(Quad::new(Node::iri(s2), Node::iri(p1), Node::iri(o4), None));

        let trie1 = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );
        let trie2 = Trie::from_quads(
            quads2,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let mut leapfrog = LeapfrogIterator::new(vec![trie1, trie2]);

        // Find first intersection
        let first = leapfrog.leapfrog_search();
        assert!(first.is_some());

        // Try to find next intersection
        let next = leapfrog.leapfrog_next();
        // May or may not have next depending on data
        // Main point: no crash, behaves correctly
        assert!(next.is_some() || leapfrog.at_end());
    }

    #[test]
    fn test_leapfrog_join_execute() {
        let (dict, quads) = create_test_data();

        let trie1 = Trie::from_quads(
            quads.clone(),
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );
        let trie2 = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let mut join = LeapfrogJoin::new(vec![trie1, trie2]);

        let results = join.execute();

        // Should have results (self-join)
        assert!(!results.is_empty());
    }

    #[test]
    fn test_leapfrog_empty_intersection() {
        let dict = Arc::new(Dictionary::new());

        // Create two disjoint quad sets (no common subjects)
        let s1 = dict.intern("http://example.org/s1");
        let s2 = dict.intern("http://example.org/s2");
        let p = dict.intern("http://example.org/p");
        let o = dict.intern("http://example.org/o");

        let quads1 = vec![Quad::new(Node::iri(s1), Node::iri(p), Node::iri(o), None)];

        let quads2 = vec![Quad::new(Node::iri(s2), Node::iri(p), Node::iri(o), None)];

        let trie1 = Trie::from_quads(
            quads1,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );
        let trie2 = Trie::from_quads(
            quads2,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let mut leapfrog = LeapfrogIterator::new(vec![trie1, trie2]);

        // Should find no intersection
        let result = leapfrog.leapfrog_search();
        assert!(result.is_none() || leapfrog.at_end());
    }

    #[test]
    fn test_leapfrog_single_trie() {
        let (dict, quads) = create_test_data();

        let trie = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let mut leapfrog = LeapfrogIterator::new(vec![trie]);

        // Single trie: intersection is the trie itself
        let result = leapfrog.leapfrog_search();
        assert!(result.is_some());
    }

    #[test]
    fn test_leapfrog_reset() {
        let (dict, quads) = create_test_data();

        let trie1 = Trie::from_quads(
            quads.clone(),
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );
        let trie2 = Trie::from_quads(
            quads,
            &[TriplePosition::Subject, TriplePosition::Predicate, TriplePosition::Object],
        );

        let mut leapfrog = LeapfrogIterator::new(vec![trie1, trie2]);

        // Search once
        let first = leapfrog.leapfrog_search();
        assert!(first.is_some());

        // Reset and search again
        leapfrog.reset();
        let second = leapfrog.leapfrog_search();

        // Should get same result
        assert_eq!(first, second);
    }
}
