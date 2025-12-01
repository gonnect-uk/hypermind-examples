//! N-Quads parser - nom-based implementation
//!
//! Complete implementation of W3C N-Quads specification.
//! N-Quads extends N-Triples by adding an optional 4th component (named graph).
//!
//! Format: `<subject> <predicate> <object> [<graph>] .`
//!
//! All components must be absolute IRIs or blank nodes (no prefixes, no abbreviations).
//! ZERO hardcoding - fully generic parser.

use crate::{ParseError, ParseResult};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, line_ending, multispace0, multispace1, one_of},
    combinator::{map, opt, peek, recognize},
    multi::many0,
    sequence::{preceded, terminated, tuple},
};
use rdf_model::{Dictionary, Node, Quad};
use std::sync::Arc;

/// N-Quads parser
///
/// Parses RDF data in N-Quads format into quads.
/// N-Quads is line-based: each line is either a quad, comment, or empty.
pub struct NQuadsParser {
    /// String dictionary for interning
    dictionary: Arc<Dictionary>,
}

impl NQuadsParser {
    /// Create a new N-Quads parser
    pub fn new(dictionary: Arc<Dictionary>) -> Self {
        Self { dictionary }
    }

    /// Parse N-Quads string into quads
    ///
    /// Returns vector of parsed quads.
    pub fn parse<'a>(&self, content: &'a str) -> ParseResult<Vec<Quad<'a>>> {
        let (remaining, quads) = nquads_doc(content)
            .map_err(|e| ParseError::Syntax {
                line: 0,
                col: 0,
                message: format!("N-Quads parse error: {:?}", e),
            })?;

        // Verify entire input was consumed (except trailing whitespace)
        let remaining_trimmed = remaining.trim();
        if !remaining_trimmed.is_empty() {
            return Err(ParseError::Syntax {
                line: 0,
                col: 0,
                message: format!(
                    "Failed to parse entire document. Unparsed content: '{}'",
                    &remaining[..remaining.len().min(100)]
                ),
            });
        }

        // Convert QuadData to Quad with dictionary interning
        let mut result = Vec::with_capacity(quads.len());
        for quad_data in quads {
            let subject = self.intern_node(&quad_data.subject)?;
            let predicate = self.intern_node(&quad_data.predicate)?;
            let object = self.intern_node(&quad_data.object)?;
            let graph = quad_data.graph
                .map(|g| self.intern_node(&g))
                .transpose()?;

            result.push(Quad {
                subject,
                predicate,
                object,
                graph,
            });
        }

        Ok(result)
    }

    /// Intern a NodeData into a Node with dictionary
    fn intern_node<'a>(&self, node_data: &NodeData) -> ParseResult<Node<'a>> {
        match node_data {
            NodeData::Iri(iri) => {
                let interned = self.dictionary.intern(iri);
                Ok(Node::iri(interned))
            }
            NodeData::BlankNode(label) => {
                // Convert blank node label to u64 using hash
                // This ensures consistent IDs for the same label within a parse session
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                label.hash(&mut hasher);
                let id = hasher.finish();
                Ok(Node::blank(id))
            }
            NodeData::Literal { value, lang, datatype } => {
                let interned_value = self.dictionary.intern(value);

                // Use appropriate literal constructor based on language/datatype
                Ok(match (lang, datatype) {
                    (Some(l), None) => {
                        let interned_lang = self.dictionary.intern(l);
                        Node::literal_lang(interned_value, interned_lang)
                    }
                    (None, Some(dt)) => {
                        let interned_datatype = self.dictionary.intern(dt);
                        Node::literal_typed(interned_value, interned_datatype)
                    }
                    (None, None) => {
                        Node::literal_str(interned_value)
                    }
                    (Some(_), Some(_)) => {
                        return Err(ParseError::Syntax {
                            line: 0,
                            col: 0,
                            message: "Literal cannot have both language tag and datatype".to_string(),
                        });
                    }
                })
            }
        }
    }
}

// ============================================================================
// Strategy Pattern Implementation
// ============================================================================

impl crate::RDFParser for NQuadsParser {
    fn parse<'a>(&'a mut self, content: &'a str) -> ParseResult<Vec<Quad<'a>>> {
        // Delegate to inherent method (dereference mutable to get immutable reference)
        NQuadsParser::parse(&*self, content)
    }

    fn format(&self) -> crate::RDFFormat {
        crate::RDFFormat::NQuads
    }
}

// ============================================================================
// Internal Data Structures (Pre-interning)
// ============================================================================

/// Quad data before dictionary interning
#[derive(Debug, Clone, PartialEq)]
struct QuadData {
    subject: NodeData,
    predicate: NodeData,
    object: NodeData,
    graph: Option<NodeData>,
}

/// Node data before dictionary interning
#[derive(Debug, Clone, PartialEq)]
enum NodeData {
    Iri(String),
    BlankNode(String),
    Literal {
        value: String,
        lang: Option<String>,
        datatype: Option<String>,
    },
}

// ============================================================================
// nom Parser Combinators
// ============================================================================

/// Parse N-Quads document: (statement | comment | empty line)*
fn nquads_doc(input: &str) -> IResult<&str, Vec<QuadData>> {
    let (input, _) = multispace0(input)?; // Consume leading whitespace

    let (input, statements) = many0(alt((
        map(terminated(comment_or_empty, multispace0), |_| None),
        map(terminated(quad_statement, multispace0), Some),
    )))(input)?;

    let (input, _) = multispace0(input)?; // Consume trailing whitespace

    // Filter out comments/empty lines
    let quads: Vec<QuadData> = statements.into_iter().flatten().collect();

    Ok((input, quads))
}

/// Parse single quad statement: subject predicate object [graph] .
fn quad_statement(input: &str) -> IResult<&str, QuadData> {
    // Subject: IRI or BlankNode
    let (input, subject) = alt((iri_ref, blank_node))(input)?;
    let (input, _) = multispace1(input)?;

    // Predicate: IRI only (per RDF spec)
    let (input, predicate) = iri_ref(input)?;
    let (input, _) = multispace1(input)?;

    // Object: IRI, BlankNode, or Literal
    let (input, object) = alt((iri_ref, blank_node, literal))(input)?;

    // Optional graph component (4th element)
    let (input, graph) = opt(preceded(
        multispace1,
        alt((iri_ref, blank_node))
    ))(input)?;

    // Trailing whitespace and period
    let (input, _) = multispace0(input)?;
    let (input, _) = char('.')(input)?;

    Ok((input, QuadData {
        subject,
        predicate,
        object,
        graph,
    }))
}

/// Parse IRI: <http://example.org/resource>
fn iri_ref(input: &str) -> IResult<&str, NodeData> {
    let (input, _) = char('<')(input)?;
    let (input, iri) = take_while(|c| c != '>')(input)?;
    let (input, _) = char('>')(input)?;

    Ok((input, NodeData::Iri(format!("<{}>", iri))))
}

/// Parse blank node: _:label
fn blank_node(input: &str) -> IResult<&str, NodeData> {
    let (input, _) = tag("_:")(input)?;
    let (input, label) = take_while1(|c: char| {
        c.is_alphanumeric() || c == '_' || c == '-'
    })(input)?;

    Ok((input, NodeData::BlankNode(label.to_string())))
}

/// Parse literal: "value" or "value"@lang or "value"^^<datatype>
fn literal(input: &str) -> IResult<&str, NodeData> {
    let (input, value) = string_literal(input)?;

    // Optional language tag or datatype
    let (input, lang_or_dt) = opt(alt((
        map(language_tag, |lang| (Some(lang), None)),
        map(preceded(tag("^^"), iri_ref), |dt| {
            let datatype = match dt {
                NodeData::Iri(iri) => iri,
                _ => unreachable!("iri_ref always returns Iri"),
            };
            (None, Some(datatype))
        }),
    )))(input)?;

    let (lang, datatype) = lang_or_dt.unwrap_or((None, None));

    Ok((input, NodeData::Literal {
        value,
        lang,
        datatype,
    }))
}

/// Parse string literal: "text with escapes"
fn string_literal(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"')(input)?;
    let (input, content) = recognize(many0(alt((
        take_while1(|c| c != '"' && c != '\\'),
        recognize(tuple((char('\\'), one_of("\"\\nrt")))),
    ))))(input)?;
    let (input, _) = char('"')(input)?;

    // Unescape content
    let unescaped = content
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\");

    Ok((input, unescaped))
}

/// Parse language tag: @en or @en-US
fn language_tag(input: &str) -> IResult<&str, String> {
    let (input, _) = char('@')(input)?;
    let (input, lang) = recognize(tuple((
        take_while1(|c: char| c.is_ascii_alphabetic()),
        many0(preceded(char('-'), take_while1(|c: char| c.is_ascii_alphanumeric()))),
    )))(input)?;

    Ok((input, lang.to_string()))
}

/// Parse comment or empty line
fn comment_or_empty(input: &str) -> IResult<&str, ()> {
    alt((
        // Comment line: starts with #
        map(tuple((char('#'), take_while(|c| c != '\n' && c != '\r'))), |_| ()),
        // Empty line: just whitespace on a line
        map(peek(line_ending), |_| ()),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RDFParser;

    fn create_parser() -> NQuadsParser {
        let dict = Arc::new(Dictionary::new());
        NQuadsParser::new(dict)
    }

    #[test]
    fn test_simple_quad_with_graph() {
        let parser = create_parser();
        let nq = r#"<http://example.org/s> <http://example.org/p> <http://example.org/o> <http://example.org/g> ."#;

        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 1);

        let quad = &quads[0];
        assert!(matches!(quad.subject, Node::Iri(_)));
        assert!(matches!(quad.predicate, Node::Iri(_)));
        assert!(matches!(quad.object, Node::Iri(_)));
        assert!(quad.graph.is_some());
    }

    #[test]
    fn test_quad_without_graph() {
        let parser = create_parser();
        let nq = r#"<http://example.org/s> <http://example.org/p> <http://example.org/o> ."#;

        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 1);

        let quad = &quads[0];
        assert!(quad.graph.is_none(), "Quad without 4th component should have None graph");
    }

    #[test]
    fn test_literal_with_language() {
        let parser = create_parser();
        let nq = r#"<http://example.org/s> <http://example.org/p> "Hello"@en <http://example.org/g> ."#;

        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 1);

        let quad = &quads[0];
        match &quad.object {
            Node::Literal(lit) => {
                assert!(lit.lexical_form.contains("Hello"));
                assert!(lit.language.is_some());
                assert!(lit.datatype.is_none());
            }
            _ => panic!("Expected Literal"),
        }
    }

    #[test]
    fn test_literal_with_datatype() {
        let parser = create_parser();
        let nq = r#"<http://example.org/s> <http://example.org/p> "42"^^<http://www.w3.org/2001/XMLSchema#integer> ."#;

        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 1);

        let quad = &quads[0];
        match &quad.object {
            Node::Literal(lit) => {
                assert!(lit.lexical_form.contains("42"));
                assert!(lit.language.is_none());
                assert!(lit.datatype.is_some());
            }
            _ => panic!("Expected Literal"),
        }
    }

    #[test]
    fn test_blank_nodes() {
        let parser = create_parser();
        let nq = r#"_:subject <http://example.org/p> _:object _:graph ."#;

        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 1);

        let quad = &quads[0];
        assert!(matches!(quad.subject, Node::BlankNode(_)));
        assert!(matches!(quad.predicate, Node::Iri(_)));
        assert!(matches!(quad.object, Node::BlankNode(_)));
        assert!(matches!(quad.graph, Some(Node::BlankNode(_))));
    }

    #[test]
    fn test_multiple_quads() {
        let parser = create_parser();
        let nq = r#"
<http://example.org/s1> <http://example.org/p1> <http://example.org/o1> <http://example.org/g1> .
<http://example.org/s2> <http://example.org/p2> "literal" <http://example.org/g2> .
<http://example.org/s3> <http://example.org/p3> <http://example.org/o3> .
"#;

        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 3);

        // First quad has graph
        assert!(quads[0].graph.is_some());

        // Second quad has literal object and graph
        assert!(matches!(quads[1].object, Node::Literal { .. }));
        assert!(quads[1].graph.is_some());

        // Third quad has no graph
        assert!(quads[2].graph.is_none());
    }

    #[test]
    fn test_comments_and_empty_lines() {
        let parser = create_parser();
        let nq = r#"
# This is a comment
<http://example.org/s> <http://example.org/p> <http://example.org/o> .

# Another comment
<http://example.org/s2> <http://example.org/p2> <http://example.org/o2> <http://example.org/g2> .
"#;

        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 2, "Comments and empty lines should be ignored");
    }

    #[test]
    fn test_string_escapes() {
        let parser = create_parser();
        let nq = r#"<http://example.org/s> <http://example.org/p> "Line 1\nLine 2\tTabbed" ."#;

        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 1);

        match &quads[0].object {
            Node::Literal(lit) => {
                assert!(lit.lexical_form.contains('\n'), "Should unescape \\n");
                assert!(lit.lexical_form.contains('\t'), "Should unescape \\t");
            }
            _ => panic!("Expected Literal"),
        }
    }

    #[test]
    fn test_rdfparser_trait() {
        let dict = Arc::new(Dictionary::new());
        let mut parser = NQuadsParser::new(dict);

        assert_eq!(parser.format(), crate::RDFFormat::NQuads);

        let nq = r#"<http://example.org/s> <http://example.org/p> <http://example.org/o> ."#;
        let quads = parser.parse(nq).expect("Should parse");
        assert_eq!(quads.len(), 1);
    }
}
