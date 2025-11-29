//! Fluent query builder API

use crate::{Error, Result};
use rdf_model::Dictionary;
use sparql::{Executor, SPARQLParser};
use std::sync::Arc;
use storage::{InMemoryBackend, QuadStore};

/// Builder for SPARQL queries
pub struct QueryBuilder<'a> {
    store: &'a QuadStore<InMemoryBackend>,
    dictionary: &'a Arc<Dictionary>,
    sparql: Option<String>,
}

impl<'a> QueryBuilder<'a> {
    pub(crate) fn new(
        store: &'a QuadStore<InMemoryBackend>,
        dictionary: &'a Arc<Dictionary>,
    ) -> Self {
        Self {
            store,
            dictionary,
            sparql: None,
        }
    }

    /// Set the SPARQL query string
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kgdb_sdk::GraphDB;
    ///
    /// let db = GraphDB::in_memory();
    /// let results = db.query()
    ///     .sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }")
    ///     .execute()?;
    /// # Ok::<(), rust_kgdb_sdk::Error>(())
    /// ```
    pub fn sparql<S: Into<String>>(mut self, query: S) -> Self {
        self.sparql = Some(query.into());
        self
    }

    /// Execute the query and return results
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No query was specified
    /// - Query parsing fails
    /// - Query execution fails
    pub fn execute(self) -> Result<QueryResult> {
        let sparql = self
            .sparql
            .ok_or_else(|| Error::InvalidOperation("No SPARQL query specified".to_string()))?;

        let mut parser = SPARQLParser::new();
        let query = parser
            .parse_query(&sparql)
            .map_err(|e| Error::Query(e.to_string()))?;

        let mut executor = Executor::new(self.store);

        match query {
            sparql::Query::Select { pattern, .. } => {
                let bindings = executor
                    .execute(&pattern)
                    .map_err(|e| Error::Query(e.to_string()))?;

                // Convert borrowed bindings to owned results
                let results: Vec<OwnedBinding> = bindings
                    .iter()
                    .map(|binding| {
                        let mut vars = std::collections::HashMap::new();
                        for (var, node) in binding.iter() {
                            vars.insert(var.name.to_string(), node_to_string(node));
                        }
                        OwnedBinding { vars }
                    })
                    .collect();

                Ok(QueryResult { results })
            }
            _ => Err(Error::InvalidOperation(
                "Only SELECT queries are currently supported".to_string(),
            )),
        }
    }
}

fn node_to_string<'a>(node: &rdf_model::Node<'a>) -> String {
    format!("{}", node)
}

/// Owned query result binding
#[derive(Debug, Clone)]
pub struct OwnedBinding {
    vars: std::collections::HashMap<String, String>,
}

impl OwnedBinding {
    /// Get the value for a variable
    pub fn get(&self, var: &str) -> Option<&str> {
        self.vars.get(var).map(|s| s.as_str())
    }

    /// Get all variable names
    pub fn variables(&self) -> impl Iterator<Item = &str> {
        self.vars.keys().map(|s| s.as_str())
    }

    /// Get all (variable, value) pairs
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// Query results iterator
pub struct QueryResult {
    results: Vec<OwnedBinding>,
}

impl QueryResult {
    /// Get the number of result bindings
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Check if there are no results
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Iterate over the bindings
    pub fn iter(&self) -> impl Iterator<Item = &OwnedBinding> + '_ {
        self.results.iter()
    }
}

impl IntoIterator for QueryResult {
    type Item = OwnedBinding;
    type IntoIter = std::vec::IntoIter<OwnedBinding>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphDB;

    #[test]
    fn test_query_builder_no_sparql() {
        let db = GraphDB::in_memory();
        let result = db.query().execute();
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::InvalidOperation(_))));
    }
}
