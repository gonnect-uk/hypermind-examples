//! Ontology Parser Module
//!
//! Parses universal-meta-mobile-ontology TTL files into strongly-typed Rust structs.
//!
//! # Architecture
//!
//! - `ttl.rs`: Low-level TTL/RDF parsing using rdf-io
//! - `ontology.rs`: High-level ontology extraction (MobileApplication, FieldDefinition, etc.)
//! - `validation.rs`: SHACL validation and constraint checking
//! - `mapper.rs`: RDF triples → Rust struct mapping

pub mod ttl;
pub mod ontology;
pub mod validation;
pub mod mapper;

use crate::model::MobileApplication;
use crate::error::{Result, GeneratorError};
use std::path::Path;

/// Main parser orchestrator - coordinates all parsing stages
pub struct OntologyParser {
    ttl_parser: ttl::TurtleParser,
    ontology_extractor: ontology::OntologyExtractor,
    validator: validation::ShaclValidator,
    mapper: mapper::TripleMapper,
}

impl OntologyParser {
    /// Create new parser with default configuration
    pub fn new() -> Self {
        Self {
            ttl_parser: ttl::TurtleParser::new(),
            ontology_extractor: ontology::OntologyExtractor::new(),
            validator: validation::ShaclValidator::new(),
            mapper: mapper::TripleMapper::new(),
        }
    }

    /// Parse TTL ontology file into MobileApplication
    ///
    /// # Process
    /// 1. Parse TTL → RDF triples
    /// 2. Extract ontology entities (classes, properties)
    /// 3. Validate against SHACL shapes
    /// 4. Map triples → strongly-typed structs
    pub fn parse_file(&self, path: &Path) -> Result<MobileApplication> {
        if !path.exists() {
            return Err(GeneratorError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Ontology file not found: {:?}", path)
                )
            ));
        }

        // Step 1: Parse TTL file → RDF triples
        let triples = self.ttl_parser.parse_file(path)?;

        // Step 2: Extract ontology structure
        let ontology = self.ontology_extractor.extract(&triples)?;

        // Step 3: Validate against SHACL shapes
        self.validator.validate(&ontology)?;

        // Step 4: Map to strongly-typed MobileApplication
        let app = self.mapper.map_to_application(&ontology)?;

        Ok(app)
    }

    /// Parse from string (useful for testing)
    pub fn parse_str(&self, ttl_content: &str) -> Result<MobileApplication> {
        let triples = self.ttl_parser.parse_str(ttl_content)?;
        let ontology = self.ontology_extractor.extract(&triples)?;
        self.validator.validate(&ontology)?;
        let app = self.mapper.map_to_application(&ontology)?;
        Ok(app)
    }
}

impl Default for OntologyParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = OntologyParser::new();
        assert!(true); // Basic smoke test
    }

    #[test]
    fn test_file_not_found() {
        let parser = OntologyParser::new();
        let result = parser.parse_file(Path::new("/nonexistent/file.ttl"));
        assert!(result.is_err());
    }
}
