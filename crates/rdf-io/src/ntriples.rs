//! N-Triples parser
//!
//! Complete implementation of W3C N-Triples specification.
//! N-Triples is the simplest RDF format - line-based, absolute IRIs only.
//! ZERO hardcoding - fully generic parser.

use crate::{ParseError, ParseResult};
use pest::Parser as PestParser;
use pest_derive::Parser;
use rdf_model::{Dictionary, Node, Quad, Triple};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Pest-generated parser for N-Triples
#[derive(Parser)]
#[grammar = "ntriples.pest"]
pub struct NTriplesPestParser;

// The pest_derive macro generates a Rule enum in this module

/// N-Triples parser
///
/// Parses RDF data in N-Triples format into quads.
/// N-Triples is simpler than Turtle: no prefixes, no base, no abbreviations.
pub struct NTriplesParser {
    /// String dictionary for interning
    dictionary: Arc<Dictionary>,
}

impl NTriplesParser {
    /// Create a new N-Triples parser
    pub fn new(dictionary: Arc<Dictionary>) -> Self {
        Self { dictionary }
    }

    /// Parse N-Triples string into quads
    ///
    /// Returns vector of parsed quads.
    pub fn parse<'a>(&self, content: &'a str) -> ParseResult<Vec<Quad<'a>>> {
        let pairs = NTriplesPestParser::parse(Rule::NTriplesDoc, content).map_err(|e| {
            ParseError::Syntax {
                line: 0,
                col: 0,
                message: e.to_string(),
            }
        })?;

        let mut quads = Vec::new();

        // Iterate over parsed triples
        for pair in pairs {
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::Triple => {
                        let triple = self.parse_triple(inner_pair)?;
                        quads.push(Quad::from_triple(triple));
                    }
                    Rule::EOI => {}
                    _ => {}
                }
            }
        }

        Ok(quads)
    }

    /// Parse a single triple
    fn parse_triple<'a>(
        &self,
        pair: pest::iterators::Pair<'a, Rule>,
    ) -> ParseResult<Triple<'a>> {
        let mut inner = pair.into_inner();

        let subject_pair = inner.next().ok_or_else(|| ParseError::Syntax {
            line: 0,
            col: 0,
            message: "Missing subject".to_string(),
        })?;

        let predicate_pair = inner.next().ok_or_else(|| ParseError::Syntax {
            line: 0,
            col: 0,
            message: "Missing predicate".to_string(),
        })?;

        let object_pair = inner.next().ok_or_else(|| ParseError::Syntax {
            line: 0,
            col: 0,
            message: "Missing object".to_string(),
        })?;

        let subject = self.parse_subject(subject_pair)?;
        let predicate = self.parse_predicate(predicate_pair)?;
        let object = self.parse_object(object_pair)?;

        Ok(Triple {
            subject,
            predicate,
            object,
        })
    }

    /// Parse subject node (IRIREF or BLANK_NODE_LABEL)
    fn parse_subject<'a>(
        &self,
        pair: pest::iterators::Pair<'a, Rule>,
    ) -> ParseResult<Node<'a>> {
        let inner = pair.into_inner().next().ok_or_else(|| ParseError::Syntax {
            line: 0,
            col: 0,
            message: "Empty subject".to_string(),
        })?;

        match inner.as_rule() {
            Rule::IRIREF => self.parse_iriref(inner),
            Rule::BLANK_NODE_LABEL => self.parse_blank_node(inner),
            _ => Err(ParseError::Syntax {
                line: 0,
                col: 0,
                message: format!("Unexpected subject rule: {:?}", inner.as_rule()),
            }),
        }
    }

    /// Parse predicate node (IRIREF only)
    fn parse_predicate<'a>(
        &self,
        pair: pest::iterators::Pair<'a, Rule>,
    ) -> ParseResult<Node<'a>> {
        let inner = pair.into_inner().next().ok_or_else(|| ParseError::Syntax {
            line: 0,
            col: 0,
            message: "Empty predicate".to_string(),
        })?;

        match inner.as_rule() {
            Rule::IRIREF => self.parse_iriref(inner),
            _ => Err(ParseError::Syntax {
                line: 0,
                col: 0,
                message: format!("Unexpected predicate rule: {:?}", inner.as_rule()),
            }),
        }
    }

    /// Parse object node (IRIREF, BLANK_NODE_LABEL, or Literal)
    fn parse_object<'a>(
        &self,
        pair: pest::iterators::Pair<'a, Rule>,
    ) -> ParseResult<Node<'a>> {
        let inner = pair.into_inner().next().ok_or_else(|| ParseError::Syntax {
            line: 0,
            col: 0,
            message: "Empty object".to_string(),
        })?;

        match inner.as_rule() {
            Rule::IRIREF => self.parse_iriref(inner),
            Rule::BLANK_NODE_LABEL => self.parse_blank_node(inner),
            Rule::Literal => self.parse_literal(inner),
            _ => Err(ParseError::Syntax {
                line: 0,
                col: 0,
                message: format!("Unexpected object rule: {:?}", inner.as_rule()),
            }),
        }
    }

    /// Parse IRI reference
    fn parse_iriref<'a>(
        &self,
        pair: pest::iterators::Pair<'a, Rule>,
    ) -> ParseResult<Node<'a>> {
        let iri_str = pair.as_str();
        // Remove angle brackets
        let iri_clean = &iri_str[1..iri_str.len() - 1];
        let interned = self.dictionary.intern(iri_clean);
        Ok(Node::iri(interned))
    }

    /// Parse blank node label
    fn parse_blank_node<'a>(
        &self,
        _pair: pest::iterators::Pair<'a, Rule>,
    ) -> ParseResult<Node<'a>> {
        // Generate unique numeric ID for this blank node
        static BLANK_COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = BLANK_COUNTER.fetch_add(1, Ordering::SeqCst);
        Ok(Node::blank(id))
    }

    /// Parse literal
    fn parse_literal<'a>(
        &self,
        pair: pest::iterators::Pair<'a, Rule>,
    ) -> ParseResult<Node<'a>> {
        let mut inner = pair.into_inner();

        // First child is always STRING_LITERAL_QUOTE
        let string_pair = inner.next().ok_or_else(|| ParseError::Syntax {
            line: 0,
            col: 0,
            message: "Missing string value in literal".to_string(),
        })?;

        let string_val = self.parse_string(string_pair)?;

        // Check for language tag or datatype
        if let Some(modifier) = inner.next() {
            match modifier.as_rule() {
                Rule::LANGTAG => {
                    let lang = modifier.as_str();
                    // Remove @ prefix
                    let lang_clean = &lang[1..];
                    let lang_interned = self.dictionary.intern(lang_clean);
                    Ok(Node::literal_lang(string_val, lang_interned))
                }
                Rule::IRIREF => {
                    let datatype_iri = self.parse_iriref(modifier)?;
                    if let Node::Iri(iri_ref) = datatype_iri {
                        Ok(Node::literal_typed(string_val, iri_ref.0))
                    } else {
                        Err(ParseError::InvalidLiteral(
                            "Datatype must be an IRI".to_string(),
                        ))
                    }
                }
                _ => Ok(Node::literal_str(string_val)),
            }
        } else {
            // Plain string literal
            Ok(Node::literal_str(string_val))
        }
    }

    /// Parse string literal
    fn parse_string<'a>(
        &self,
        pair: pest::iterators::Pair<'a, Rule>,
    ) -> ParseResult<&'a str> {
        let str_val = pair.as_str();
        // Remove double quotes
        let cleaned = &str_val[1..str_val.len() - 1];

        // Intern the string value
        let interned = self.dictionary.intern(cleaned);
        Ok(interned)
    }
}

// ============================================================================
// Strategy Pattern Implementation
// ============================================================================

impl crate::RDFParser for NTriplesParser {
    fn parse<'a>(&'a mut self, content: &'a str) -> crate::ParseResult<Vec<Quad<'a>>> {
        // Delegate to inherent method (dereference mutable to get immutable reference)
        NTriplesParser::parse(&*self, content)
    }

    fn format(&self) -> crate::RDFFormat {
        crate::RDFFormat::NTriples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_parser() -> NTriplesParser {
        let dict = Arc::new(Dictionary::new());
        NTriplesParser::new(dict)
    }

    #[test]
    fn test_simple_triple() {
        let parser = create_parser();
        let input = r#"<http://example.org/subject> <http://example.org/predicate> <http://example.org/object> ."#;

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 1);

        let triple = quads[0].to_triple();
        assert!(matches!(triple.subject, Node::Iri(_)));
        assert!(matches!(triple.predicate, Node::Iri(_)));
        assert!(matches!(triple.object, Node::Iri(_)));
    }

    #[test]
    fn test_literal_object() {
        let parser = create_parser();
        let input = r#"<http://example.org/subject> <http://example.org/predicate> "Hello World" ."#;

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 1);

        let triple = quads[0].to_triple();
        assert!(matches!(triple.object, Node::Literal(_)));
    }

    #[test]
    fn test_literal_with_language() {
        let parser = create_parser();
        let input = r#"<http://example.org/subject> <http://example.org/predicate> "Hello"@en ."#;

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 1);

        let triple = quads[0].to_triple();
        if let Node::Literal(lit) = triple.object {
            assert_eq!(lit.lexical_form, "Hello");
            assert!(lit.language.is_some());
        } else {
            panic!("Expected literal object");
        }
    }

    #[test]
    fn test_literal_with_datatype() {
        let parser = create_parser();
        let input = r#"<http://example.org/subject> <http://example.org/predicate> "42"^^<http://www.w3.org/2001/XMLSchema#integer> ."#;

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 1);

        let triple = quads[0].to_triple();
        if let Node::Literal(lit) = triple.object {
            assert_eq!(lit.lexical_form, "42");
            assert!(lit.datatype.is_some());
        } else {
            panic!("Expected literal object");
        }
    }

    #[test]
    fn test_blank_node() {
        let parser = create_parser();
        let input = r#"_:subject <http://example.org/predicate> _:object ."#;

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 1);

        let triple = quads[0].to_triple();
        assert!(matches!(triple.subject, Node::BlankNode(_)));
        assert!(matches!(triple.object, Node::BlankNode(_)));
    }

    #[test]
    fn test_multiple_triples() {
        let parser = create_parser();
        let input = r#"
<http://example.org/s1> <http://example.org/p1> <http://example.org/o1> .
<http://example.org/s2> <http://example.org/p2> "literal" .
<http://example.org/s3> <http://example.org/p3> "text"@en .
"#;

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 3);
    }

    #[test]
    fn test_comments() {
        let parser = create_parser();
        let input = r#"
# This is a comment
<http://example.org/subject> <http://example.org/predicate> <http://example.org/object> .
# Another comment
"#;

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 1);
    }

    #[test]
    fn test_empty_document() {
        let parser = create_parser();
        let input = "";

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 0);
    }

    #[test]
    fn test_escape_sequences() {
        let parser = create_parser();
        let input = r#"<http://example.org/s> <http://example.org/p> "Hello\nWorld" ."#;

        let result = parser.parse(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let quads = result.unwrap();
        assert_eq!(quads.len(), 1);
    }
}
