//! NAPI-RS TypeScript/JavaScript bindings for rust-kgdb
//!
//! Zero-config InMemory RDF/SPARQL database for Node.js

use napi::bindgen_prelude::*;
use napi_derive::napi;
use parking_lot::Mutex;
use rdf_model::{Dictionary, Node as RdfNode, Quad};
use sparql::{Executor, Query, Variable};
use sparql::parser::SPARQLParser;
use std::sync::Arc;
use storage::{InMemoryBackend, QuadStore};

/// GraphDB - High-performance RDF/SPARQL database
#[napi]
pub struct GraphDB {
    store: Arc<Mutex<QuadStore<InMemoryBackend>>>,
    dictionary: Arc<Dictionary>,
    app_graph: String,
}

#[napi]
impl GraphDB {
    /// Create new in-memory GraphDB instance
    #[napi(constructor)]
    pub fn new(app_graph_uri: String) -> Result<Self> {
        let dictionary = Arc::new(Dictionary::new());
        let backend = InMemoryBackend::new();
        let store = QuadStore::new(backend);

        Ok(GraphDB {
            store: Arc::new(Mutex::new(store)),
            dictionary,
            app_graph: app_graph_uri,
        })
    }

    /// Load Turtle (TTL) data
    #[napi]
    pub fn load_ttl(&self, ttl_content: String, graph_name: Option<String>) -> Result<()> {
        let graph_uri = graph_name.unwrap_or_else(|| self.app_graph.clone());
        let graph_str = self.dictionary.intern(&graph_uri);
        let graph_node = RdfNode::iri(graph_str);

        // Simple TTL parsing (production would use full turtle parser)
        let lines: Vec<&str> = ttl_content.lines().collect();
        let mut store = self.store.lock();

        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('@') {
                continue;
            }

            // Simple triple parsing (subject predicate object .)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let subj_str = self.dictionary.intern(parts[0].trim_matches('<').trim_matches('>'));
                let pred_str = self.dictionary.intern(parts[1].trim_matches('<').trim_matches('>'));
                let object_str = parts[2..parts.len()-1].join(" ");
                let obj_str = if object_str.starts_with('<') {
                    self.dictionary.intern(object_str.trim_matches('<').trim_matches('>'))
                } else {
                    self.dictionary.intern(object_str.trim_matches('"'))
                };

                let subject = RdfNode::iri(subj_str);
                let predicate = RdfNode::iri(pred_str);
                let object = if object_str.starts_with('<') {
                    RdfNode::iri(obj_str)
                } else {
                    RdfNode::literal_str(obj_str)
                };

                let quad = Quad::new(subject, predicate, object, Some(graph_node.clone()));

                store.insert(quad).map_err(|e| {
                    Error::from_reason(format!("Failed to insert triple: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Execute SPARQL SELECT query
    #[napi]
    pub fn query_select(&self, sparql: String) -> Result<Vec<QueryResult>> {
        let store = self.store.lock();
        let mut parser = SPARQLParser::new();

        let query = parser.parse_query(&sparql).map_err(|e| {
            Error::from_reason(format!("SPARQL parse error: {:?}", e))
        })?;

        // Extract pattern from SELECT query
        match query {
            Query::Select { pattern, .. } => {
                let mut executor = Executor::new(&*store);
                let bindings = executor.execute(&pattern).map_err(|e| {
                    Error::from_reason(format!("Query execution error: {:?}", e))
                })?;

                let mut output = Vec::new();
                for binding in bindings.iter() {
                    let mut bindings_map = std::collections::HashMap::new();
                    for (var, node) in binding.iter() {
                        bindings_map.insert(var.name.to_string(), node_to_string(node));
                    }
                    output.push(QueryResult { bindings: bindings_map });
                }

                Ok(output)
            }
            _ => Err(Error::from_reason("Only SELECT queries supported via query_select".to_string()))
        }
    }

    /// Execute SPARQL query (returns triples from SELECT with ?s ?p ?o variables)
    #[napi]
    pub fn query(&self, sparql: String) -> Result<Vec<TripleResult>> {
        let store = self.store.lock();
        let mut parser = SPARQLParser::new();

        let query = parser.parse_query(&sparql).map_err(|e| {
            Error::from_reason(format!("SPARQL parse error: {:?}", e))
        })?;

        // Extract pattern from SELECT query
        match query {
            Query::Select { pattern, .. } => {
                let mut executor = Executor::new(&*store);
                let bindings = executor.execute(&pattern).map_err(|e| {
                    Error::from_reason(format!("Query execution error: {:?}", e))
                })?;

                // Convert bindings to triples (expects ?s/?subject, ?p/?predicate, ?o/?object)
                let mut output = Vec::new();
                let var_s = Variable::new("s");
                let var_subject = Variable::new("subject");
                let var_p = Variable::new("p");
                let var_predicate = Variable::new("predicate");
                let var_o = Variable::new("o");
                let var_object = Variable::new("object");

                for binding in bindings.iter() {
                    if let (Some(s), Some(p), Some(o)) = (
                        binding.get(&var_s).or(binding.get(&var_subject)),
                        binding.get(&var_p).or(binding.get(&var_predicate)),
                        binding.get(&var_o).or(binding.get(&var_object))
                    ) {
                        output.push(TripleResult {
                            subject: node_to_string(s),
                            predicate: node_to_string(p),
                            object: node_to_string(o),
                        });
                    }
                }

                Ok(output)
            }
            _ => Err(Error::from_reason("Only SELECT queries supported via query".to_string()))
        }
    }

    /// Count total triples
    #[napi]
    pub fn count_triples(&self) -> Result<i64> {
        let store = self.store.lock();
        Ok(store.len() as i64)
    }

    /// Clear all data
    #[napi]
    pub fn clear(&self) -> Result<()> {
        let mut store = self.store.lock();
        // Recreate store
        let backend = InMemoryBackend::new();
        *store = QuadStore::new(backend);
        Ok(())
    }

    /// Get app graph URI
    #[napi]
    pub fn get_graph_uri(&self) -> String {
        self.app_graph.clone()
    }
}

/// SPARQL SELECT query result
#[napi(object)]
pub struct QueryResult {
    pub bindings: std::collections::HashMap<String, String>,
}

/// Triple query result
#[napi(object)]
pub struct TripleResult {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

/// Get library version
#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Helper function to convert Node to String
fn node_to_string(node: &RdfNode) -> String {
    match node {
        RdfNode::Iri(iri) => iri.0.to_string(),
        RdfNode::Literal(lit) => lit.lexical_form.to_string(),
        RdfNode::BlankNode(id) => format!("_:b{}", id.0),
        RdfNode::Variable(var) => format!("?{}", var.0),
        RdfNode::QuotedTriple(_) => "<< quoted triple >>".to_string(),
    }
}
