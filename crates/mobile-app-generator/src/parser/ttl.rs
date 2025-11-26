//! Turtle/TTL Parser
//!
//! Low-level RDF parsing using rdf-io crate.

use crate::error::{Result, GeneratorError};
use rdf_model::{Node, Triple, Dictionary};
use std::path::Path;
use std::fs;
use std::sync::Arc;

/// Turtle/TTL file parser
pub struct TurtleParser {
    dictionary: Arc<Dictionary>,
}

impl TurtleParser {
    pub fn new() -> Self {
        Self {
            dictionary: Arc::new(Dictionary::new()),
        }
    }

    /// Parse TTL file into RDF triples
    pub fn parse_file(&self, path: &Path) -> Result<Vec<Triple>> {
        let content = fs::read_to_string(path)
            .map_err(|e| GeneratorError::Io(e))?;

        self.parse_str(&content)
    }

    /// Parse TTL string into RDF triples
    pub fn parse_str(&self, content: &str) -> Result<Vec<Triple>> {
        let mut triples = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_subject: Option<Node> = None;
        let mut current_predicate: Option<Node> = None;

        for line in lines {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Skip @prefix declarations (handled separately)
            if trimmed.starts_with("@prefix") || trimmed.starts_with("@base") {
                continue;
            }

            // Parse triple components
            if let Some(triple) = self.parse_triple_line(trimmed, &mut current_subject, &mut current_predicate)? {
                triples.push(triple);
            }
        }

        Ok(triples)
    }

    /// Parse a single triple line
    fn parse_triple_line<'a>(
        &'a self,
        line: &str,
        current_subject: &mut Option<Node<'a>>,
        current_predicate: &mut Option<Node<'a>>,
    ) -> Result<Option<Triple<'a>>> {
        // Remove trailing punctuation
        let line = line.trim_end_matches('.').trim_end_matches(';').trim_end_matches(',');

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        // Determine if this is a new subject or continuation
        if parts[0].starts_with('<') || parts[0].starts_with("_:") {
            // New subject
            let subject = self.parse_node(parts[0])?;
            *current_subject = Some(subject.clone());

            if parts.len() >= 3 {
                let predicate = self.parse_node(parts[1])?;
                *current_predicate = Some(predicate.clone());

                let object_parts: Vec<&str> = parts[2..].iter().map(|s| *s).collect();
                let object_str = object_parts.join(" ");
                let object = self.parse_node(&object_str)?;

                return Ok(Some(Triple {
                    subject,
                    predicate,
                    object,
                }));
            }
        } else if parts[0].starts_with('a') || parts[0].starts_with('<') {
            // Predicate continuation
            let predicate = if parts[0] == "a" {
                self.parse_node("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>")?
            } else {
                self.parse_node(parts[0])?
            };
            *current_predicate = Some(predicate.clone());

            if parts.len() >= 2 && current_subject.is_some() {
                let object_parts: Vec<&str> = parts[1..].iter().map(|s| *s).collect();
                let object_str = object_parts.join(" ");
                let object = self.parse_node(&object_str)?;

                return Ok(Some(Triple {
                    subject: current_subject.clone().unwrap(),
                    predicate,
                    object,
                }));
            }
        }

        Ok(None)
    }

    /// Parse individual node (IRI, literal, blank node)
    fn parse_node(&self, s: &str) -> Result<Node> {
        let s = s.trim();

        if s.starts_with('<') && s.ends_with('>') {
            // IRI
            let iri = &s[1..s.len()-1];
            Ok(Node::iri(self.dictionary.intern(iri)))
        } else if s.starts_with('"') {
            // Literal
            self.parse_literal(s)
        } else if s.starts_with("_:") {
            // Blank node
            let id = &s[2..];
            let id_num = id.parse::<u64>()
                .unwrap_or_else(|_| id.chars().map(|c| c as u64).sum());
            Ok(Node::BlankNode(rdf_model::BlankNodeId(id_num)))
        } else {
            // Unquoted literal (number, boolean, etc.)
            Ok(Node::literal_str(self.dictionary.intern(s)))
        }
    }

    /// Parse literal with optional language tag or datatype
    fn parse_literal(&self, s: &str) -> Result<Node> {
        let s = s.trim();

        if !s.starts_with('"') {
            return Err(GeneratorError::RdfParse(
                format!("Invalid literal: {}", s)
            ));
        }

        // Find closing quote
        let mut end_quote = 1;
        while end_quote < s.len() {
            if s.chars().nth(end_quote) == Some('"') && s.chars().nth(end_quote - 1) != Some('\\') {
                break;
            }
            end_quote += 1;
        }

        let value = &s[1..end_quote];
        let rest = &s[end_quote + 1..].trim();

        if rest.starts_with("^^") {
            // Datatype
            let datatype = rest[2..].trim();
            let datatype = if datatype.starts_with('<') && datatype.ends_with('>') {
                &datatype[1..datatype.len()-1]
            } else {
                datatype
            };
            Ok(Node::Literal(rdf_model::Literal {
                lexical_form: self.dictionary.intern(value),
                language: None,
                datatype: Some(self.dictionary.intern(datatype)),
            }))
        } else if rest.starts_with('@') {
            // Language tag
            let lang = &rest[1..];
            Ok(Node::Literal(rdf_model::Literal {
                lexical_form: self.dictionary.intern(value),
                language: Some(self.dictionary.intern(lang)),
                datatype: None,
            }))
        } else {
            // Plain literal - default to xsd:string
            Ok(Node::literal_str(self.dictionary.intern(value)))
        }
    }
}

impl Default for TurtleParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_triple() {
        let parser = TurtleParser::new();
        let ttl = r#"
            <http://example.org/subject> <http://example.org/predicate> "object" .
        "#;

        let triples = parser.parse_str(ttl).unwrap();
        assert_eq!(triples.len(), 1);
    }

    #[test]
    fn test_parse_with_prefix() {
        let parser = TurtleParser::new();
        let ttl = r#"
            @prefix ex: <http://example.org/> .
            <http://example.org/s> <http://example.org/p> "o" .
        "#;

        let triples = parser.parse_str(ttl).unwrap();
        assert!(triples.len() >= 1);
    }

    #[test]
    fn test_parse_typed_literal() {
        let parser = TurtleParser::new();
        let ttl = r#"
            <http://example.org/s> <http://example.org/age> "25"^^<http://www.w3.org/2001/XMLSchema#integer> .
        "#;

        let triples = parser.parse_str(ttl).unwrap();
        assert_eq!(triples.len(), 1);

        match &triples[0].object {
            Node::Literal(lit) => {
                assert!(lit.datatype.map_or(false, |dt| dt.contains("integer")));
            },
            _ => panic!("Expected literal"),
        }
    }
}
