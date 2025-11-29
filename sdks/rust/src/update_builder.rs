//! Fluent update builder API

use crate::{Error, Node, Result};
use rdf_model::{Dictionary, Quad};
use std::sync::Arc;
use storage::{InMemoryBackend, QuadStore};

/// Builder for update operations (INSERT, DELETE)
pub struct UpdateBuilder<'a> {
    store: &'a mut QuadStore<InMemoryBackend>,
    dictionary: &'a Arc<Dictionary>,
    triples: Vec<(Node, Node, Node)>,
    graph: Option<Node>,
}

impl<'a> UpdateBuilder<'a> {
    pub(crate) fn new(
        store: &'a mut QuadStore<InMemoryBackend>,
        dictionary: &'a Arc<Dictionary>,
    ) -> Self {
        Self {
            store,
            dictionary,
            triples: Vec::new(),
            graph: None,
        }
    }

    /// Add a triple to insert
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
    pub fn triple(mut self, subject: Node, predicate: Node, object: Node) -> Self {
        self.triples.push((subject, predicate, object));
        self
    }

    /// Set the named graph for all triples
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::{GraphDB, Node};
    ///
    /// let mut db = GraphDB::in_memory();
    /// db.insert()
    ///     .graph(Node::iri("http://example.org/graph1"))
    ///     .triple(
    ///         Node::iri("http://example.org/alice"),
    ///         Node::iri("http://xmlns.com/foaf/0.1/name"),
    ///         Node::literal("Alice"),
    ///     )
    ///     .execute()?;
    /// # Ok::<(), rust_kgdb_sdk::Error>(())
    /// ```
    pub fn graph(mut self, graph: Node) -> Self {
        self.graph = Some(graph);
        self
    }

    /// Execute the update operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No triples were specified
    /// - Storage operation fails
    pub fn execute(mut self) -> Result<()> {
        if self.triples.is_empty() {
            return Err(Error::InvalidOperation(
                "No triples specified for insert".to_string(),
            ));
        }

        // Clone triples and graph to avoid borrow checker issues
        let triples_to_insert = std::mem::take(&mut self.triples);
        let graph_node = self.graph.take();

        for (subj, pred, obj) in &triples_to_insert {
            let s = Self::node_to_rdf_node(self.dictionary, subj)?;
            let p = Self::node_to_rdf_node(self.dictionary, pred)?;
            let o = Self::node_to_rdf_node(self.dictionary, obj)?;

            let graph = if let Some(ref g) = graph_node {
                Some(Self::node_to_rdf_node(self.dictionary, g)?)
            } else {
                None
            };

            let quad = Quad::new(s, p, o, graph);
            self.store
                .insert(quad)
                .map_err(|e| Error::Storage(e.to_string()))?;
        }

        Ok(())
    }

    fn node_to_rdf_node<'d>(dictionary: &'d Arc<Dictionary>, node: &Node) -> Result<rdf_model::Node<'d>> {
        match node {
            Node::IRI(iri) => {
                let interned = dictionary.intern(iri);
                Ok(rdf_model::Node::iri(interned))
            }
            Node::Literal {
                value,
                datatype,
                lang,
            } => {
                let val = dictionary.intern(value);
                if let Some(dt) = datatype {
                    let dt_interned = dictionary.intern(dt);
                    Ok(rdf_model::Node::literal_typed(val, dt_interned))
                } else if let Some(l) = lang {
                    // Intern the language tag as well to ensure proper lifetime
                    let lang_interned = dictionary.intern(l);
                    Ok(rdf_model::Node::literal_lang(val, lang_interned))
                } else {
                    Ok(rdf_model::Node::literal_str(val))
                }
            }
            Node::BlankNode(id) => {
                // Convert string ID to numeric ID
                // For now, use a simple hash
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                id.hash(&mut hasher);
                let numeric_id = hasher.finish();

                Ok(rdf_model::Node::blank(numeric_id))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphDB;

    #[test]
    fn test_update_builder_no_triples() {
        let mut db = GraphDB::in_memory();
        let result = db.insert().execute();
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::InvalidOperation(_))));
    }

    #[test]
    fn test_insert_single_triple() {
        let mut db = GraphDB::in_memory();
        let result = db
            .insert()
            .triple(
                Node::iri("http://example.org/alice"),
                Node::iri("http://xmlns.com/foaf/0.1/name"),
                Node::literal("Alice"),
            )
            .execute();
        assert!(result.is_ok());
        assert_eq!(db.count(), 1);
    }

    #[test]
    fn test_insert_multiple_triples() {
        let mut db = GraphDB::in_memory();
        let result = db
            .insert()
            .triple(
                Node::iri("http://example.org/alice"),
                Node::iri("http://xmlns.com/foaf/0.1/name"),
                Node::literal("Alice"),
            )
            .triple(
                Node::iri("http://example.org/bob"),
                Node::iri("http://xmlns.com/foaf/0.1/name"),
                Node::literal("Bob"),
            )
            .execute();
        assert!(result.is_ok());
        assert_eq!(db.count(), 2);
    }
}
