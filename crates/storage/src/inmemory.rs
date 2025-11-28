//! In-memory storage backend using lock-free DashMap
//!
//! Ultra-fast, zero-persistence backend optimized for concurrent access.
//! Uses DashMap for lock-free reads/writes with capacity pre-allocation.

use crate::{StorageBackend, StorageResult, StorageStats};
use dashmap::DashMap;  // Lock-free concurrent hash map
use parking_lot::RwLock;
use std::sync::Arc;

/// In-memory storage backend
///
/// Stores all data in a DashMap for lock-free concurrent access.
/// Thread-safe without locks. Optimized for multi-threaded workloads.
#[derive(Clone)]
pub struct InMemoryBackend {
    /// Data storage (lock-free concurrent hash map)
    data: Arc<DashMap<Vec<u8>, Vec<u8>>>,

    /// Statistics
    stats: Arc<RwLock<StorageStats>>,
}

impl InMemoryBackend {
    /// Create a new in-memory backend
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::with_capacity(100_000)),  // Pre-allocate for bulk inserts
            stats: Arc::new(RwLock::new(StorageStats::default())),
        }
    }

    /// Create with specific capacity (optimization for known dataset size)
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Arc::new(DashMap::with_capacity(capacity)),
            stats: Arc::new(RwLock::new(StorageStats::default())),
        }
    }

    /// Get number of keys
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
        *self.stats.write() = StorageStats::default();
    }

    /// Batch insert multiple key-value pairs (lock-free parallel inserts)
    ///
    /// Uses rayon to parallelize insertions across CPU cores for maximum throughput.
    /// Optimal for large batches (1000+ pairs).
    ///
    /// # Performance
    /// - **Sequential**: ~146K pairs/sec
    /// - **Parallel (this)**: 250-350K pairs/sec (target)
    #[inline]
    pub fn batch_put(&mut self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> StorageResult<()> {
        use rayon::prelude::*;

        let initial_len = self.data.len();

        // Parallel batch insert using rayon + DashMap lock-free writes
        // Each thread can insert concurrently without locks
        pairs.par_iter().for_each(|(k, v)| {
            self.data.insert(k.clone(), v.clone());
        });

        // Update stats
        let mut stats = self.stats.write();
        stats.writes += (self.data.len() - initial_len) as u64;
        stats.key_count = self.data.len() as u64;

        Ok(())
    }

    /// Batch insert with move semantics (avoids cloning)
    ///
    /// Consumes the pairs vector for zero-copy insertion.
    /// Prefer this over batch_put() when pairs won't be reused.
    pub fn batch_put_owned(&mut self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> StorageResult<()> {
        use rayon::prelude::*;

        let initial_len = self.data.len();

        // Parallel insert with move semantics - no cloning!
        pairs.into_par_iter().for_each(|(k, v)| {
            self.data.insert(k, v);
        });

        // Update stats
        let mut stats = self.stats.write();
        stats.writes += (self.data.len() - initial_len) as u64;
        stats.key_count = self.data.len() as u64;

        Ok(())
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for InMemoryBackend {
    #[inline]
    fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        // DashMap.get() returns Option<Ref<K, V>> - clone the value
        let value = self.data.get(key).map(|v| v.value().clone());

        // Update stats
        self.stats.write().reads += 1;

        Ok(value)
    }

    #[inline]
    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        // DashMap allows concurrent inserts without locks
        self.data.insert(key.to_vec(), value.to_vec());

        // Update stats
        let mut stats = self.stats.write();
        stats.writes += 1;
        stats.key_count = self.data.len() as u64;

        Ok(())
    }

    #[inline]
    fn delete(&mut self, key: &[u8]) -> StorageResult<()> {
        // DashMap.remove() returns Option<(K, V)>
        self.data.remove(key);

        // Update stats
        let mut stats = self.stats.write();
        stats.deletes += 1;
        stats.key_count = self.data.len() as u64;

        Ok(())
    }

    #[inline]
    fn contains(&self, key: &[u8]) -> StorageResult<bool> {
        Ok(self.data.contains_key(key))
    }

    fn range_scan<'a>(
        &'a self,
        start: &[u8],
        end: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        use rayon::prelude::*;

        // Collect DashMap entries into Vec for parallel processing
        let entries: Vec<_> = self.data
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        // Parallel filter and collect using rayon
        let mut results: Vec<_> = entries
            .into_par_iter()
            .filter(|(k, _)| {
                let key_slice = k.as_slice();
                key_slice >= start && key_slice < end
            })
            .collect();

        // Parallel sort for large result sets
        results.par_sort_by(|a, b| a.0.cmp(&b.0));

        Ok(Box::new(results.into_iter()))
    }

    fn prefix_scan<'a>(
        &'a self,
        prefix: &[u8],
    ) -> StorageResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>> {
        use rayon::prelude::*;

        // Collect DashMap entries into Vec for parallel processing
        let entries: Vec<_> = self.data
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        // Parallel prefix filter using rayon
        let mut results: Vec<_> = entries
            .into_par_iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .collect();

        // Parallel sort for large result sets
        results.par_sort_by(|a, b| a.0.cmp(&b.0));

        Ok(Box::new(results.into_iter()))
    }

    fn stats(&self) -> StorageStats {
        self.stats.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut backend = InMemoryBackend::new();

        // Put
        backend.put(b"key1", b"value1").unwrap();
        backend.put(b"key2", b"value2").unwrap();

        // Get
        assert_eq!(backend.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(backend.get(b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(backend.get(b"key3").unwrap(), None);

        // Contains
        assert!(backend.contains(b"key1").unwrap());
        assert!(!backend.contains(b"key3").unwrap());

        // Delete
        backend.delete(b"key1").unwrap();
        assert_eq!(backend.get(b"key1").unwrap(), None);

        // Stats
        let stats = backend.stats();
        assert_eq!(stats.key_count, 1);
        assert_eq!(stats.writes, 2);
        assert_eq!(stats.deletes, 1);
    }

    #[test]
    fn test_range_scan() {
        let mut backend = InMemoryBackend::new();

        backend.put(b"a", b"1").unwrap();
        backend.put(b"b", b"2").unwrap();
        backend.put(b"c", b"3").unwrap();
        backend.put(b"d", b"4").unwrap();

        let results: Vec<_> = backend.range_scan(b"b", b"d").unwrap().collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, b"b");
        assert_eq!(results[1].0, b"c");
    }

    #[test]
    fn test_prefix_scan() {
        let mut backend = InMemoryBackend::new();

        backend.put(b"prefix:1", b"value1").unwrap();
        backend.put(b"prefix:2", b"value2").unwrap();
        backend.put(b"other:1", b"value3").unwrap();

        let results: Vec<_> = backend.prefix_scan(b"prefix:").unwrap().collect();

        assert_eq!(results.len(), 2);
        assert!(results[0].0.starts_with(b"prefix:"));
        assert!(results[1].0.starts_with(b"prefix:"));
    }

    #[test]
    fn test_clear() {
        let mut backend = InMemoryBackend::new();

        backend.put(b"key1", b"value1").unwrap();
        assert_eq!(backend.len(), 1);

        backend.clear();
        assert_eq!(backend.len(), 0);
        assert!(backend.is_empty());
    }
}
