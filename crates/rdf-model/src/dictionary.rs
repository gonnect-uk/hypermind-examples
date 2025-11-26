//! String interning dictionary for zero-copy semantics
//!
//! Provides bidirectional string-to-ID mapping with:
//! - Thread-safe concurrent access
//! - Deduplication of identical strings
//! - Stable references with 'static lifetime
//! - Memory-efficient storage

#![allow(unsafe_code)]  // Required for stable 'static references from Arc

use parking_lot::RwLock;
use rustc_hash::FxHashSet;
use std::sync::Arc;

/// Thread-safe string interning dictionary
///
/// Stores unique strings and returns stable 'static references.
/// Internally uses a HashSet with Arc for reference counting.
#[derive(Clone)]
pub struct Dictionary {
    /// Set of interned strings
    /// Using Arc<str> for stable references
    strings: Arc<RwLock<FxHashSet<Arc<str>>>>,
}

impl Dictionary {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        Self {
            strings: Arc::new(RwLock::new(FxHashSet::default())),
        }
    }

    /// Intern a string and return a stable reference
    ///
    /// If the string already exists, returns the existing reference.
    /// Otherwise, allocates and stores a new Arc<str>.
    ///
    /// # Safety
    ///
    /// The returned reference is 'static because the Arc is never freed
    /// until the Dictionary is dropped.
    pub fn intern(&self, s: &str) -> &'static str {
        // Fast path: check if already interned (read lock)
        {
            let guard = self.strings.read();
            if let Some(existing) = guard.get(s) {
                // SAFETY: The Arc lives as long as the Dictionary,
                // and we never remove strings, so this reference is stable
                return unsafe { &*(Arc::as_ptr(existing) as *const str) };
            }
        }

        // Slow path: insert new string (write lock)
        let mut guard = self.strings.write();

        // Double-check after acquiring write lock
        if let Some(existing) = guard.get(s) {
            return unsafe { &*(Arc::as_ptr(existing) as *const str) };
        }

        // Allocate new Arc<str>
        let arc: Arc<str> = s.into();
        let ptr = Arc::as_ptr(&arc);
        guard.insert(arc);

        // SAFETY: Same as above
        unsafe { &*(ptr as *const str) }
    }

    /// Check if dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.strings.read().is_empty()
    }

    /// Get number of interned strings
    pub fn len(&self) -> usize {
        self.strings.read().len()
    }

    /// Get memory usage in bytes (approximate)
    pub fn memory_usage(&self) -> usize {
        let guard = self.strings.read();
        guard.iter().map(|s| s.len()).sum()
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_same_string() {
        let dict = Dictionary::new();
        let s1 = dict.intern("test");
        let s2 = dict.intern("test");

        // Should return same pointer
        assert_eq!(s1.as_ptr(), s2.as_ptr());
        assert_eq!(dict.len(), 1);
    }

    #[test]
    fn test_intern_different_strings() {
        let dict = Dictionary::new();
        let s1 = dict.intern("test1");
        let s2 = dict.intern("test2");

        assert_ne!(s1, s2);
        assert_eq!(dict.len(), 2);
    }

    #[test]
    fn test_clone_shares_storage() {
        let dict1 = Dictionary::new();
        let _s1 = dict1.intern("test");

        let dict2 = dict1.clone();
        let s2 = dict2.intern("test");

        // Should reuse same string from shared storage
        assert_eq!(dict1.len(), 1);
        assert_eq!(dict2.len(), 1);
        assert_eq!(s2, "test");
    }

    #[test]
    fn test_memory_usage() {
        let dict = Dictionary::new();
        dict.intern("hello");
        dict.intern("world");

        assert_eq!(dict.memory_usage(), 10); // "hello" + "world"
    }
}
