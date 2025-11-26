//! Ontology Extraction
//!
//! Extracts high-level ontology entities from RDF triples.

use crate::error::{Result, GeneratorError};
use rdf_model::{Node, Triple};
use std::collections::HashMap;

/// Ontology structure extracted from RDF triples
#[derive(Debug, Clone)]
pub struct Ontology {
    pub mobile_application: Option<OntologyEntity>,
    pub personas: Vec<OntologyEntity>,
    pub business_values: Vec<OntologyEntity>,
    pub views: Vec<OntologyEntity>,
    pub fields: Vec<OntologyEntity>,
    pub queries: Vec<OntologyEntity>,
    pub how_it_works: Vec<OntologyEntity>,
}

/// Individual ontology entity with properties
#[derive(Debug, Clone)]
pub struct OntologyEntity {
    pub subject: String,
    pub rdf_type: Option<String>,
    pub properties: HashMap<String, Vec<String>>,
}

impl OntologyEntity {
    fn new(subject: String) -> Self {
        Self {
            subject,
            rdf_type: None,
            properties: HashMap::new(),
        }
    }

    /// Get single property value
    pub fn get_property(&self, predicate: &str) -> Option<&String> {
        self.properties.get(predicate).and_then(|v| v.first())
    }

    /// Get all property values
    pub fn get_all_properties(&self, predicate: &str) -> Vec<&String> {
        self.properties.get(predicate).map(|v| v.iter().collect()).unwrap_or_default()
    }
}

/// Ontology extractor
pub struct OntologyExtractor;

impl OntologyExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract ontology structure from RDF triples
    pub fn extract(&self, triples: &[Triple]) -> Result<Ontology> {
        // Group triples by subject
        let mut entities_map: HashMap<String, OntologyEntity> = HashMap::new();

        for triple in triples {
            let subject_str = self.node_to_string(&triple.subject);
            let predicate_str = self.node_to_string(&triple.predicate);
            let object_str = self.node_to_string(&triple.object);

            let entity = entities_map.entry(subject_str.clone())
                .or_insert_with(|| OntologyEntity::new(subject_str.clone()));

            // Handle rdf:type specially
            if predicate_str.contains("rdf-syntax-ns#type") || predicate_str.ends_with("/type") {
                entity.rdf_type = Some(object_str.clone());
            }

            // Add property
            entity.properties.entry(predicate_str)
                .or_insert_with(Vec::new)
                .push(object_str);
        }

        // Categorize entities by type
        let mut ontology = Ontology {
            mobile_application: None,
            personas: Vec::new(),
            business_values: Vec::new(),
            views: Vec::new(),
            fields: Vec::new(),
            queries: Vec::new(),
            how_it_works: Vec::new(),
        };

        for entity in entities_map.values() {
            if let Some(rdf_type) = &entity.rdf_type {
                if rdf_type.contains("MobileApplication") {
                    ontology.mobile_application = Some(entity.clone());
                } else if rdf_type.contains("BusinessPersona") {
                    ontology.personas.push(entity.clone());
                } else if rdf_type.contains("BusinessValue") {
                    ontology.business_values.push(entity.clone());
                } else if rdf_type.contains("FormView") || rdf_type.contains("ListView") {
                    ontology.views.push(entity.clone());
                } else if rdf_type.contains("TextField") || rdf_type.contains("NumberField") ||
                          rdf_type.contains("DateField") || rdf_type.contains("CurrencyField") ||
                          rdf_type.contains("PickerField") || rdf_type.contains("BooleanField") {
                    ontology.fields.push(entity.clone());
                } else if rdf_type.contains("QueryTemplate") {
                    ontology.queries.push(entity.clone());
                } else if rdf_type.contains("HowItWorksPanel") {
                    ontology.how_it_works.push(entity.clone());
                }
            }
        }

        // Validate that we found at least a MobileApplication
        if ontology.mobile_application.is_none() {
            return Err(GeneratorError::Validation(
                "No MobileApplication entity found in ontology".to_string()
            ));
        }

        Ok(ontology)
    }

    /// Convert Node to string representation
    fn node_to_string<'a>(&self, node: &Node<'a>) -> String {
        match node {
            Node::Iri(iri_ref) => iri_ref.0.to_string(),
            Node::Literal(lit) => lit.lexical_form.to_string(),
            Node::BlankNode(id) => format!("_:b{}", id.0),
            Node::Variable(var) => format!("?{}", var.0),
            Node::QuotedTriple(_) => "<quoted-triple>".to_string(),
        }
    }

    /// Resolve property URI (handles prefixes like meta:, rdf:, etc.)
    pub fn resolve_property_uri(&self, prop: &str, prefixes: &HashMap<String, String>) -> String {
        if prop.starts_with("http://") || prop.starts_with("https://") {
            return prop.to_string();
        }

        // Check if it has a prefix
        if let Some(colon_pos) = prop.find(':') {
            let prefix = &prop[..colon_pos];
            let local_name = &prop[colon_pos + 1..];

            if let Some(base_uri) = prefixes.get(prefix) {
                return format!("{}{}", base_uri, local_name);
            }
        }

        // Default: assume it's a local name in meta namespace
        format!("http://universal-kg.com/meta/{}", prop)
    }
}

impl Default for OntologyExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ontology_entity_creation() {
        let mut entity = OntologyEntity::new("http://example.org/app".to_string());
        entity.rdf_type = Some("MobileApplication".to_string());
        entity.properties.insert("title".to_string(), vec!["My App".to_string()]);

        assert_eq!(entity.get_property("title"), Some(&"My App".to_string()));
        assert!(entity.get_property("nonexistent").is_none());
    }

    #[test]
    fn test_resolve_property_uri() {
        let extractor = OntologyExtractor::new();
        let mut prefixes = HashMap::new();
        prefixes.insert("meta".to_string(), "http://universal-kg.com/meta/".to_string());

        let resolved = extractor.resolve_property_uri("meta:hasTitle", &prefixes);
        assert_eq!(resolved, "http://universal-kg.com/meta/hasTitle");
    }
}
