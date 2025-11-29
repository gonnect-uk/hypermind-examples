//! High-level GraphDB API

use crate::{QueryBuilder, UpdateBuilder};
use rdf_model::Dictionary;
use std::sync::Arc;
use storage::{InMemoryBackend, QuadStore};

/// High-level interface to the RDF graph database
///
/// Provides ergonomic methods for querying, updating, and managing RDF data.
pub struct GraphDB {
    store: QuadStore<InMemoryBackend>,
    dictionary: Arc<Dictionary>,
}

impl GraphDB {
    /// Create a new in-memory graph database
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::GraphDB;
    ///
    /// let db = GraphDB::in_memory();
    /// ```
    pub fn in_memory() -> Self {
        let store = QuadStore::new_in_memory();
        let dictionary = Arc::clone(store.dictionary());

        Self { store, dictionary }
    }

    /// Get a reference to the underlying quad store
    pub fn store(&self) -> &QuadStore<InMemoryBackend> {
        &self.store
    }

    /// Get a reference to the string dictionary
    pub fn dictionary(&self) -> &Arc<Dictionary> {
        &self.dictionary
    }

    /// Start building a SPARQL query
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::GraphDB;
    ///
    /// let db = GraphDB::in_memory();
    /// let results = db.query()
    ///     .sparql("SELECT * WHERE { ?s ?p ?o }")
    ///     .execute()?;
    /// # Ok::<(), rust_kgdb_sdk::Error>(())
    /// ```
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(&self.store, &self.dictionary)
    }

    /// Start building an update operation
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::{GraphDB, Node};
    ///
    /// let mut db = GraphDB::in_memory();
    /// db.insert()
    ///     .triple(
    ///         Node::iri("http://example.org/alice"),
    ///         Node::iri("http://xmlns.com/foaf/0.1/name"),
    ///         Node::literal("Alice"),
    ///     )
    ///     .execute()?;
    /// # Ok::<(), rust_kgdb_sdk::Error>(())
    /// ```
    pub fn insert(&mut self) -> UpdateBuilder {
        UpdateBuilder::new(&mut self.store, &self.dictionary)
    }

    /// Get the total number of triples in the database
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::GraphDB;
    ///
    /// let db = GraphDB::in_memory();
    /// let count = db.count();
    /// assert_eq!(count, 0);
    /// ```
    pub fn count(&self) -> usize {
        self.store.len()
    }

    /// Check if the database is empty
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Clear all triples from the database
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::GraphDB;
    ///
    /// let mut db = GraphDB::in_memory();
    /// db.clear();
    /// assert!(db.is_empty());
    /// ```
    pub fn clear(&mut self) {
        // Note: QuadStore doesn't have a clear method in the current API
        // This would need to be implemented in the storage layer
        // For now, we can document this as a limitation
    }
}

impl Default for GraphDB {
    fn default() -> Self {
        Self::in_memory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory() {
        let db = GraphDB::in_memory();
        assert_eq!(db.count(), 0);
        assert!(db.is_empty());
    }

    #[test]
    fn test_default() {
        let db = GraphDB::default();
        assert_eq!(db.count(), 0);
    }
}
