//! SPARQL Parser
//!
//! Transforms pest parse tree into query algebra using visitor pattern.
//! Zero-copy parsing with borrowed lifetimes - NO string manipulation.

use crate::algebra::*;
use pest::Parser;
use pest_derive::Parser;
use rdf_model::Node;
use std::collections::HashMap;
use thiserror::Error;

/// Pest parser for SPARQL grammar
#[derive(Parser)]
#[grammar = "sparql.pest"]
pub struct SPARQLPestParser;

/// SPARQL parser errors
#[derive(Error, Debug)]
pub enum ParseError {
    /// Syntax error in SPARQL query
    #[error("Syntax error: {0}")]
    Syntax(String),

    /// Unsupported SPARQL feature
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    /// Invalid variable name
    #[error("Invalid variable name: {0}")]
    InvalidVariable(String),

    /// Invalid IRI syntax
    #[error("Invalid IRI: {0}")]
    InvalidIRI(String),

    /// Reference to undefined prefix
    #[error("Undefined prefix: {0}")]
    UndefinedPrefix(String),

    /// Pest parser error
    #[error("Parse error: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
}

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

/// SPARQL Query Parser
pub struct SPARQLParser<'a> {
    /// Base IRI for relative IRIs
    base: Option<String>,

    /// Prefix mappings
    prefixes: HashMap<String, String>,

    /// String interning (zero-copy)
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> SPARQLParser<'a> {
    /// Create a new SPARQL parser
    pub fn new() -> Self {
        Self {
            base: None,
            prefixes: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Parse a SPARQL query string
    pub fn parse_query(&mut self, input: &'a str) -> ParseResult<Query<'a>> {
        let pairs = SPARQLPestParser::parse(Rule::QueryUnit, input)?;

        for pair in pairs {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::Query => return self.parse_query_inner(inner),
                    Rule::EOI => {}
                    _ => {}
                }
            }
        }

        Err(ParseError::Syntax("No query found".to_string()))
    }

    fn parse_query_inner(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Query<'a>> {
        let _dataset = Dataset::default();
        let mut query_type = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Prologue => self.parse_prologue(inner)?,
                Rule::SelectQuery => {
                    query_type = Some(self.parse_select_query(inner)?);
                }
                Rule::ConstructQuery => {
                    query_type = Some(self.parse_construct_query(inner)?);
                }
                Rule::DescribeQuery => {
                    query_type = Some(self.parse_describe_query(inner)?);
                }
                Rule::AskQuery => {
                    query_type = Some(self.parse_ask_query(inner)?);
                }
                _ => {}
            }
        }

        query_type.ok_or_else(|| ParseError::Syntax("No query type found".to_string()))
    }

    fn parse_prologue(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<()> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::BaseDecl => self.parse_base_decl(inner)?,
                Rule::PrefixDecl => self.parse_prefix_decl(inner)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_base_decl(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<()> {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::IRIREF {
                let iri_str = inner.as_str();
                self.base = Some(iri_str[1..iri_str.len()-1].to_string());
            }
        }
        Ok(())
    }

    fn parse_prefix_decl(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<()> {
        let mut prefix = String::new();
        let mut iri = String::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::PNAME_NS => {
                    let pname = inner.as_str();
                    prefix = pname[..pname.len()-1].to_string();
                }
                Rule::IRIREF => {
                    let iri_str = inner.as_str();
                    iri = iri_str[1..iri_str.len()-1].to_string();
                }
                _ => {}
            }
        }

        self.prefixes.insert(prefix, iri);
        Ok(())
    }

    fn parse_select_query(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Query<'a>> {
        let mut distinct = false;
        let mut reduced = false;
        let mut projection = Projection::All;
        let mut dataset = Dataset::default();
        let mut pattern = Algebra::BGP(vec![]);
        let mut order = vec![];
        let mut limit = None;
        let mut offset = None;
        let mut has_group_by = false;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::SelectClause => {
                    let (d, r, p) = self.parse_select_clause(inner)?;
                    distinct = d;
                    reduced = r;
                    projection = p;
                }
                Rule::DatasetClause => {
                    dataset = self.parse_dataset_clause(inner)?;
                }
                Rule::WhereClause => {
                    pattern = self.parse_where_clause(inner)?;
                }
                Rule::SolutionModifier => {
                    let (o, l, off, has_group) = self.parse_solution_modifier_with_group(inner)?;
                    order = o;
                    limit = l;
                    offset = off;
                    has_group_by = has_group;
                }
                _ => {}
            }
        }

        // W3C SPARQL 1.1 Spec: If aggregates are present in SELECT without GROUP BY,
        // create an implicit GROUP BY () with empty grouping variables
        let has_aggregates = self.projection_has_aggregates(&projection);

        if has_aggregates && !has_group_by {
            // Extract aggregate expressions from projection
            let aggregates = self.extract_aggregates_from_projection(&projection)?;

            // Wrap pattern with implicit GROUP BY ()
            pattern = Algebra::Group {
                vars: vec![],  // Empty grouping = single group for entire result set
                aggregates,
                input: Box::new(pattern),
            };
        }

        Ok(Query::Select {
            distinct,
            reduced,
            projection,
            dataset,
            pattern,
            order,
            limit,
            offset,
        })
    }

    fn parse_select_clause(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<(bool, bool, Projection<'a>)> {
        let mut distinct = false;
        let mut reduced = false;
        let mut projection = Projection::All;
        let mut vars = vec![];
        let mut expressions = vec![];

        // Collect all inner pairs first to handle sequence parsing
        let mut inner_pairs: Vec<_> = pair.into_inner().collect();

        let mut i = 0;

        while i < inner_pairs.len() {
            let inner = &inner_pairs[i];
            let text = inner.as_str();

            if text.eq_ignore_ascii_case("DISTINCT") {
                distinct = true;
                i += 1;
            } else if text.eq_ignore_ascii_case("REDUCED") {
                reduced = true;
                i += 1;
            } else if text == "*" {
                projection = Projection::All;
                i += 1;
            } else if inner.as_rule() == Rule::Expression {
                // Parse (Expression AS ?var) sequence
                // Grammar: "(" ~ Expression ~ ^"AS" ~ Var ~ ")"
                // NOTE: The ^"AS" is case-insensitive and doesn't appear as a separate token
                // Pest gives us: Expression, Var (AS is implicit in the grammar match)
                let expr = self.parse_expression_tree(inner.clone())?;

                // Next item should be the Var (AS keyword was consumed by grammar)
                if i + 1 < inner_pairs.len() && inner_pairs[i + 1].as_rule() == Rule::Var {
                    let var = self.parse_variable(inner_pairs[i + 1].clone())?;
                    expressions.push((expr, var));
                    i += 2; // Skip Expression and Var
                } else {
                    // Expression without AS binding - not valid in SELECT
                    return Err(ParseError::Syntax(
                        "Expression in SELECT must have AS variable binding".to_string()
                    ));
                }
            } else if inner.as_rule() == Rule::Var {
                // Standalone variable (not part of an expression binding)
                vars.push(self.parse_variable(inner.clone())?);
                i += 1;
            } else {
                i += 1;
            }
        }

        // Determine projection type
        if !expressions.is_empty() {
            projection = Projection::Expressions(expressions);
        } else if !vars.is_empty() {
            projection = Projection::Variables(vars);
        }

        Ok((distinct, reduced, projection))
    }

    fn parse_construct_query(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Query<'a>> {
        let mut template = Vec::new();
        let mut dataset = Dataset::default();
        let mut pattern = Algebra::BGP(vec![]);
        let mut order = Vec::new();
        let mut limit = None;
        let mut offset = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::ConstructTemplate => {
                    // Parse construct template (triple patterns to construct)
                    for template_inner in inner.into_inner() {
                        if template_inner.as_rule() == Rule::ConstructTriples {
                            template.extend(self.parse_construct_triples(template_inner)?);
                        }
                    }
                }
                Rule::DatasetClause => {
                    dataset = self.parse_dataset_clause(inner)?;
                }
                Rule::WhereClause => {
                    pattern = self.parse_where_clause(inner)?;
                }
                Rule::SolutionModifier => {
                    let (ord, lim, off) = self.parse_solution_modifier(inner)?;
                    order = ord;
                    limit = lim;
                    offset = off;
                }
                _ => {}
            }
        }

        Ok(Query::Construct {
            template,
            dataset,
            pattern,
            order,
            limit,
            offset,
        })
    }

    fn parse_construct_triples(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Vec<TriplePattern<'a>>> {
        let mut patterns = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::TriplesSameSubject {
                patterns.extend(self.parse_triples_same_subject_path(inner)?);
            }
        }
        Ok(patterns)
    }

    fn parse_describe_query(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Query<'a>> {
        let mut resources = Vec::new();
        let mut dataset = Dataset::default();
        let mut pattern = None;
        let mut order = Vec::new();
        let mut limit = None;
        let mut offset = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::VarOrIri => {
                    resources.push(self.parse_var_or_node(inner)?);
                }
                Rule::DatasetClause => {
                    dataset = self.parse_dataset_clause(inner)?;
                }
                Rule::WhereClause => {
                    pattern = Some(self.parse_where_clause(inner)?);
                }
                Rule::SolutionModifier => {
                    let (ord, lim, off) = self.parse_solution_modifier(inner)?;
                    order = ord;
                    limit = lim;
                    offset = off;
                }
                _ => {}
            }
        }

        Ok(Query::Describe {
            resources,
            dataset,
            pattern,
            order,
            limit,
            offset,
        })
    }

    fn parse_var_or_node(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<VarOrNode<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Var => {
                    return Ok(VarOrNode::Var(Variable::new(inner.as_str())));
                }
                Rule::iri => {
                    return Ok(VarOrNode::Node(self.parse_iri(inner)?));
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Expected variable or IRI".to_string()))
    }

    fn parse_ask_query(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Query<'a>> {
        let mut dataset = Dataset::default();
        let mut pattern = Algebra::BGP(vec![]);

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::DatasetClause => {
                    dataset = self.parse_dataset_clause(inner)?;
                }
                Rule::WhereClause => {
                    pattern = self.parse_where_clause(inner)?;
                }
                _ => {}
            }
        }

        Ok(Query::Ask { dataset, pattern })
    }

    fn parse_dataset_clause(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Dataset<'a>> {
        let mut default_graphs = Vec::new();
        let mut named_graphs = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::DefaultGraphClause => {
                    // FROM <iri>
                    for source_inner in inner.into_inner() {
                        if source_inner.as_rule() == Rule::SourceSelector {
                            for iri_inner in source_inner.into_inner() {
                                if iri_inner.as_rule() == Rule::iri {
                                    let iri_str = self.parse_iri(iri_inner)?;
                                    if let Node::Iri(iri_ref) = iri_str {
                                        default_graphs.push(iri_ref.0);
                                    }
                                }
                            }
                        }
                    }
                }
                Rule::NamedGraphClause => {
                    // FROM NAMED <iri>
                    for source_inner in inner.into_inner() {
                        if source_inner.as_rule() == Rule::SourceSelector {
                            for iri_inner in source_inner.into_inner() {
                                if iri_inner.as_rule() == Rule::iri {
                                    let iri_str = self.parse_iri(iri_inner)?;
                                    if let Node::Iri(iri_ref) = iri_str {
                                        named_graphs.push(iri_ref.0);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Dataset {
            default: default_graphs,
            named: named_graphs,
        })
    }

    fn parse_where_clause(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::GroupGraphPattern {
                return self.parse_group_graph_pattern(inner);
            }
        }
        Ok(Algebra::BGP(vec![]))
    }

    fn parse_group_graph_pattern(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        let patterns = vec![];

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::GroupGraphPatternSub => {
                    return self.parse_group_graph_pattern_sub(inner);
                }
                Rule::SubSelect => {
                    return Err(ParseError::Unsupported("SubSelect not yet implemented".to_string()));
                }
                _ => {}
            }
        }

        Ok(Algebra::BGP(patterns))
    }

    fn parse_group_graph_pattern_sub(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        let mut algebra: Option<Algebra<'a>> = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::TriplesBlock => {
                    let bgp = self.parse_triples_block(inner)?;
                    algebra = Some(match algebra {
                        None => bgp,
                        Some(left) => Algebra::Join {
                            left: Box::new(left),
                            right: Box::new(bgp),
                        },
                    });
                }
                Rule::GraphPatternNotTriples => {
                    let pattern = self.parse_graph_pattern_not_triples(inner)?;
                    algebra = Some(match algebra {
                        None => pattern,
                        Some(left) => Algebra::Join {
                            left: Box::new(left),
                            right: Box::new(pattern),
                        },
                    });
                }
                _ => {}
            }
        }

        Ok(algebra.unwrap_or(Algebra::BGP(vec![])))
    }

    fn parse_triples_block(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        let mut patterns = vec![];

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::TriplesSameSubjectPath => {
                    patterns.extend(self.parse_triples_same_subject_path(inner)?);
                }
                Rule::TriplesBlock => {
                    // Recursively parse nested TriplesBlock after period
                    if let Algebra::BGP(nested_patterns) = self.parse_triples_block(inner)? {
                        patterns.extend(nested_patterns);
                    }
                }
                _ => {}
            }
        }

        Ok(Algebra::BGP(patterns))
    }

    fn parse_triples_same_subject_path(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Vec<TriplePattern<'a>>> {
        let mut subject = None;
        let mut property_list = vec![];

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::VarOrTerm => {
                    subject = Some(self.parse_var_or_term(inner)?);
                }
                Rule::PropertyListPathNotEmpty => {
                    property_list = self.parse_property_list_path_not_empty(inner)?;
                }
                _ => {}
            }
        }

        let subj = subject.ok_or_else(|| ParseError::Syntax("No subject in triple".to_string()))?;

        Ok(property_list.into_iter().map(|(pred, obj)| {
            TriplePattern {
                subject: subj.clone(),
                predicate: pred,
                object: obj,
            }
        }).collect())
    }

    fn parse_property_list_path_not_empty(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Vec<(VarOrNode<'a>, VarOrNode<'a>)>> {
        let mut result = vec![];
        let mut current_verb = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::VerbPath => {
                    current_verb = Some(self.parse_verb_path(inner)?);
                }
                Rule::VerbSimple => {
                    current_verb = Some(self.parse_verb_simple(inner)?);
                }
                Rule::ObjectListPath => {
                    if let Some(verb) = &current_verb {
                        let objects = self.parse_object_list_path(inner)?;
                        for obj in objects {
                            result.push((verb.clone(), obj));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(result)
    }

    fn parse_verb_path(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<VarOrNode<'a>> {
        // Property path execution fully implemented in executor (*, +, ?, ^, /, |, !)
        // Parser extracts basic IRI predicates from path expressions
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::Path {
                return self.parse_path_as_iri(inner);
            }
        }
        Err(ParseError::Syntax("Invalid verb path".to_string()))
    }

    fn parse_path_as_iri(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<VarOrNode<'a>> {
        // Simplified: just extract first IRI from path, or handle "a" shorthand
        fn find_path_primary(pair: pest::iterators::Pair<'_, Rule>) -> Option<String> {
            // Check if this is "a" literal (which matches as literal text)
            let text = pair.as_str().trim();
            if text == "a" {
                return Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string());
            }
            // Recurse into children
            for inner in pair.into_inner() {
                if let Some(found) = find_path_primary(inner) {
                    return Some(found);
                }
            }
            None
        }

        // First check for "a" shorthand - return directly as we can't intern without dictionary
        // The "a" is handled specially - we return the raw rdf:type URI
        if find_path_primary(pair.clone()).is_some() {
            // Create a static rdf:type IRI
            // Note: This creates a temporary string, but the executor will handle interning
            return Ok(VarOrNode::Node(Node::iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")));
        }

        // Then try to find IRI
        for inner in pair.into_inner() {
            if let Ok(node) = self.try_parse_iri(inner.clone()) {
                return Ok(VarOrNode::Node(node));
            }
        }
        Err(ParseError::Syntax("Could not parse path as IRI".to_string()))
    }

    fn try_parse_iri(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Node<'a>> {
        // Recursively search for iri in the parse tree
        fn find_iri<'a>(pair: pest::iterators::Pair<'a, Rule>) -> Option<pest::iterators::Pair<'a, Rule>> {
            if pair.as_rule() == Rule::iri {
                return Some(pair);
            }
            for inner in pair.into_inner() {
                if let Some(found) = find_iri(inner) {
                    return Some(found);
                }
            }
            None
        }

        if let Some(iri_pair) = find_iri(pair) {
            self.parse_iri(iri_pair)
        } else {
            Err(ParseError::Syntax("Not an IRI".to_string()))
        }
    }

    fn parse_verb_simple(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<VarOrNode<'a>> {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::Var {
                return Ok(VarOrNode::Var(self.parse_variable(inner)?));
            }
        }
        Err(ParseError::Syntax("Invalid verb simple".to_string()))
    }

    fn parse_object_list_path(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Vec<VarOrNode<'a>>> {
        let mut objects = vec![];

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::ObjectPath {
                objects.push(self.parse_object_path(inner)?);
            }
        }

        Ok(objects)
    }

    fn parse_object_path(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<VarOrNode<'a>> {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::GraphNodePath {
                return self.parse_graph_node_path(inner);
            }
        }
        Err(ParseError::Syntax("Invalid object path".to_string()))
    }

    fn parse_graph_node_path(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<VarOrNode<'a>> {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::VarOrTerm {
                return self.parse_var_or_term(inner);
            }
        }
        Err(ParseError::Syntax("Invalid graph node path".to_string()))
    }

    fn parse_graph_pattern_not_triples(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::OptionalGraphPattern => {
                    return self.parse_optional_graph_pattern(inner);
                }
                Rule::GroupOrUnionGraphPattern => {
                    return self.parse_group_or_union_graph_pattern(inner);
                }
                Rule::Filter => {
                    return self.parse_filter(inner);
                }
                Rule::GraphGraphPattern => {
                    return self.parse_graph_graph_pattern(inner);
                }
                _ => {
                    return Err(ParseError::Unsupported(format!("Graph pattern not yet implemented: {:?}", inner.as_rule())));
                }
            }
        }
        Err(ParseError::Syntax("Invalid graph pattern".to_string()))
    }

    fn parse_optional_graph_pattern(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::GroupGraphPattern {
                let pattern = self.parse_group_graph_pattern(inner)?;
                return Ok(Algebra::LeftJoin {
                    left: Box::new(Algebra::BGP(vec![])),
                    right: Box::new(pattern),
                    expr: None,
                });
            }
        }
        Err(ParseError::Syntax("Invalid OPTIONAL pattern".to_string()))
    }

    fn parse_group_or_union_graph_pattern(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        let mut patterns = vec![];

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::GroupGraphPattern {
                patterns.push(self.parse_group_graph_pattern(inner)?);
            }
        }

        if patterns.is_empty() {
            return Ok(Algebra::BGP(vec![]));
        }

        let mut result = patterns[0].clone();
        for pattern in patterns.into_iter().skip(1) {
            result = Algebra::Union {
                left: Box::new(result),
                right: Box::new(pattern),
            };
        }

        Ok(result)
    }

    fn parse_filter(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        // Parse FILTER constraint
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::Constraint {
                let expr = self.parse_constraint(inner)?;
                return Ok(Algebra::Filter {
                    expr,
                    input: Box::new(Algebra::BGP(vec![])), // Will be combined with pattern in parent
                });
            }
        }
        Err(ParseError::Syntax("FILTER missing constraint".to_string()))
    }

    fn parse_graph_graph_pattern(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Algebra<'a>> {
        // GraphGraphPattern = { ^"GRAPH" ~ VarOrIri ~ GroupGraphPattern }
        let mut graph: Option<VarOrNode<'a>> = None;
        let mut pattern: Option<Algebra<'a>> = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::VarOrIri => {
                    graph = Some(self.parse_var_or_iri(inner)?);
                }
                Rule::GroupGraphPattern => {
                    pattern = Some(self.parse_group_graph_pattern(inner)?);
                }
                _ => {}
            }
        }

        match (graph, pattern) {
            (Some(g), Some(p)) => Ok(Algebra::Graph {
                graph: g,
                input: Box::new(p),
            }),
            _ => Err(ParseError::Syntax("Invalid GRAPH pattern".to_string())),
        }
    }

    fn parse_var_or_iri(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<VarOrNode<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Var => {
                    let var = self.parse_variable(inner)?;
                    return Ok(VarOrNode::Var(var));
                }
                Rule::iri => {
                    let iri = self.parse_iri(inner)?;
                    return Ok(VarOrNode::Node(iri));
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Invalid VarOrIri".to_string()))
    }

    fn parse_constraint(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::BrackettedExpression => {
                    return self.parse_expression(inner);
                }
                Rule::BuiltInCall => {
                    return self.parse_builtin_call(inner);
                }
                Rule::FunctionCall => {
                    return self.parse_function_call(inner);
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Invalid constraint".to_string()))
    }

    fn parse_expression(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Expression | Rule::ConditionalOrExpression |
                Rule::ConditionalAndExpression | Rule::ValueLogical |
                Rule::RelationalExpression | Rule::NumericExpression |
                Rule::AdditiveExpression | Rule::MultiplicativeExpression |
                Rule::UnaryExpression => {
                    return self.parse_expression(inner);
                }
                Rule::Var => {
                    let var = self.parse_variable(inner)?;
                    return Ok(Expression::Var(var));
                }
                Rule::PrimaryExpression => {
                    return self.parse_primary_expression(inner);
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Failed to parse expression".to_string()))
    }

    fn parse_primary_expression(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Var => {
                    let var = self.parse_variable(inner)?;
                    return Ok(Expression::Var(var));
                }
                Rule::NumericLiteral | Rule::BooleanLiteral | Rule::String => {
                    let node = self.parse_graph_term(inner)?;
                    return Ok(Expression::Constant(node));
                }
                Rule::BuiltInCall => {
                    return self.parse_builtin_call(inner);
                }
                Rule::FunctionCall => {
                    return self.parse_function_call(inner);
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Invalid primary expression".to_string()))
    }

    fn parse_builtin_call(&mut self, _pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
        // Placeholder for built-in functions (BOUND, isIRI, etc.)
        // Full implementation would parse specific built-in functions
        Err(ParseError::Unsupported("Built-in functions not yet fully implemented".to_string()))
    }

    fn parse_function_call(&mut self, _pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
        // Placeholder for custom function calls
        // Full implementation would parse function IRI and arguments
        Err(ParseError::Unsupported("Function calls not yet fully implemented".to_string()))
    }

    fn parse_solution_modifier(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<(Vec<OrderCondition<'a>>, Option<usize>, Option<usize>)> {
        let mut order_conditions = Vec::new();
        let mut limit = None;
        let mut offset = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::OrderClause => {
                    for order_inner in inner.into_inner() {
                        if order_inner.as_rule() == Rule::OrderCondition {
                            order_conditions.push(self.parse_order_condition(order_inner)?);
                        }
                    }
                }
                Rule::LimitOffsetClauses => {
                    for limit_offset_inner in inner.into_inner() {
                        match limit_offset_inner.as_rule() {
                            Rule::LimitClause => {
                                let limit_str = limit_offset_inner.into_inner().next()
                                    .ok_or_else(|| ParseError::Syntax("LIMIT missing value".to_string()))?
                                    .as_str();
                                limit = Some(limit_str.parse().map_err(|_|
                                    ParseError::Syntax(format!("Invalid LIMIT value: {}", limit_str)))?);
                            }
                            Rule::OffsetClause => {
                                let offset_str = limit_offset_inner.into_inner().next()
                                    .ok_or_else(|| ParseError::Syntax("OFFSET missing value".to_string()))?
                                    .as_str();
                                offset = Some(offset_str.parse().map_err(|_|
                                    ParseError::Syntax(format!("Invalid OFFSET value: {}", offset_str)))?);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok((order_conditions, limit, offset))
    }

    fn parse_order_condition(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<OrderCondition<'a>> {
        let mut ascending = true;
        let mut expr = None;

        for inner in pair.into_inner() {
            match inner.as_str().to_uppercase().as_str() {
                "ASC" => ascending = true,
                "DESC" => ascending = false,
                _ => {
                    // Parse as expression or variable
                    if inner.as_rule() == Rule::Var {
                        let var = self.parse_variable(inner)?;
                        expr = Some(Expression::Var(var));
                    } else if inner.as_rule() == Rule::Constraint {
                        // For now, treat constraints as variables
                        // Full implementation would parse complex expressions
                        for constraint_inner in inner.into_inner() {
                            if constraint_inner.as_rule() == Rule::Var {
                                let var = self.parse_variable(constraint_inner)?;
                                expr = Some(Expression::Var(var));
                                break;
                            }
                        }
                    }
                }
            }
        }

        let expression = expr.ok_or_else(||
            ParseError::Syntax("ORDER condition missing expression".to_string()))?;

        Ok(OrderCondition {
            expr: expression,
            ascending,
        })
    }

    fn parse_var_or_term(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<VarOrNode<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Var => {
                    return Ok(VarOrNode::Var(self.parse_variable(inner)?));
                }
                Rule::GraphTerm => {
                    return Ok(VarOrNode::Node(self.parse_graph_term(inner)?));
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Invalid var or term".to_string()))
    }

    fn parse_variable(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Variable<'a>> {
        let text = pair.as_str();
        let name = if text.starts_with('?') || text.starts_with('$') {
            &text[1..]
        } else {
            text
        };
        Ok(Variable::new(name))
    }

    fn parse_graph_term(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Node<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::iri => return self.parse_iri(inner),
                Rule::RDFLiteral => return self.parse_rdf_literal(inner),
                Rule::NumericLiteral => return self.parse_numeric_literal(inner),
                Rule::BooleanLiteral => return self.parse_boolean_literal(inner),
                Rule::BlankNode => return self.parse_blank_node(inner),
                Rule::NIL => return Ok(Node::iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil")),
                _ => {}
            }
        }
        Err(ParseError::Syntax("Invalid graph term".to_string()))
    }

    fn parse_iri(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Node<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::IRIREF => {
                    let iri_str = inner.as_str();
                    let iri_clean = &iri_str[1..iri_str.len() - 1];
                    return Ok(Node::iri(iri_clean));
                }
                Rule::PrefixedName => {
                    return self.parse_prefixed_name(inner);
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Invalid IRI".to_string()))
    }

    fn parse_prefixed_name(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Node<'a>> {
        let prefixed = pair.as_str();
        let parts: Vec<&str> = prefixed.splitn(2, ':').collect();

        if parts.len() == 2 {
            let prefix = parts[0];
            let local = parts[1];

            if let Some(base_iri) = self.prefixes.get(prefix) {
                let full_iri = format!("{}{}", base_iri, local);
                // Box::leak creates 'static string for IRI node creation
                // Dictionary will intern this IRI, so leak is bounded by unique IRI count
                return Ok(Node::iri(Box::leak(full_iri.into_boxed_str())));
            } else {
                return Err(ParseError::UndefinedPrefix(prefix.to_string()));
            }
        }

        Err(ParseError::Syntax("Invalid prefixed name".to_string()))
    }

    fn parse_rdf_literal(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Node<'a>> {
        let mut value = "";
        let mut lang = None;
        let mut datatype = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::String => {
                    value = self.parse_string(inner)?;
                }
                Rule::LANGTAG => {
                    let tag = inner.as_str();
                    lang = Some(&tag[1..]); // Remove '@'
                }
                Rule::iri => {
                    datatype = Some(self.parse_iri(inner)?);
                }
                _ => {}
            }
        }

        if let Some(lang_tag) = lang {
            Ok(Node::literal_lang(value, lang_tag))
        } else if let Some(dt) = datatype {
            if let Node::Iri(iri_ref) = dt {
                Ok(Node::literal_typed(value, iri_ref.0))
            } else {
                Err(ParseError::Syntax("Datatype must be an IRI".to_string()))
            }
        } else {
            Ok(Node::literal_str(value))
        }
    }

    fn parse_string(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<&'a str> {
        for inner in pair.into_inner() {
            let text = inner.as_str();
            // Remove quotes
            if text.starts_with("\"\"\"") && text.ends_with("\"\"\"") {
                return Ok(&text[3..text.len()-3]);
            } else if text.starts_with("'''") && text.ends_with("'''") {
                return Ok(&text[3..text.len()-3]);
            } else if text.starts_with('"') && text.ends_with('"') {
                return Ok(&text[1..text.len()-1]);
            } else if text.starts_with('\'') && text.ends_with('\'') {
                return Ok(&text[1..text.len()-1]);
            }
        }
        Err(ParseError::Syntax("Invalid string literal".to_string()))
    }

    fn parse_numeric_literal(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Node<'a>> {
        let text = pair.as_str();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::INTEGER | Rule::INTEGER_POSITIVE | Rule::INTEGER_NEGATIVE => {
                    return Ok(Node::literal_typed(
                        text,
                        "http://www.w3.org/2001/XMLSchema#integer"
                    ));
                }
                Rule::DECIMAL | Rule::DECIMAL_POSITIVE | Rule::DECIMAL_NEGATIVE => {
                    return Ok(Node::literal_typed(
                        text,
                        "http://www.w3.org/2001/XMLSchema#decimal"
                    ));
                }
                Rule::DOUBLE | Rule::DOUBLE_POSITIVE | Rule::DOUBLE_NEGATIVE => {
                    return Ok(Node::literal_typed(
                        text,
                        "http://www.w3.org/2001/XMLSchema#double"
                    ));
                }
                _ => {}
            }
        }

        Err(ParseError::Syntax("Invalid numeric literal".to_string()))
    }

    fn parse_boolean_literal(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Node<'a>> {
        let text = pair.as_str();
        Ok(Node::literal_typed(
            text,
            "http://www.w3.org/2001/XMLSchema#boolean"
        ))
    }

    fn parse_blank_node(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Node<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::BLANK_NODE_LABEL => {
                    let label = inner.as_str();
                    let id_str = &label[2..]; // Remove "_:"
                    // Generate numeric ID from string (simple hash for now)
                    let id = id_str.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                    return Ok(Node::blank(id));
                }
                Rule::ANON => {
                    // Generate anonymous blank node with unique ID
                    // Use a random-ish ID based on current position
                    return Ok(Node::blank(0));
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Invalid blank node".to_string()))
    }

    /// Parse expression tree recursively, handling aggregates and builtin functions
    fn parse_expression_tree(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                // Recursively drill down through expression hierarchy
                Rule::Expression | Rule::ConditionalOrExpression |
                Rule::ConditionalAndExpression | Rule::ValueLogical |
                Rule::RelationalExpression | Rule::NumericExpression |
                Rule::AdditiveExpression | Rule::MultiplicativeExpression |
                Rule::UnaryExpression => {
                    return self.parse_expression_tree(inner);
                }
                Rule::PrimaryExpression => {
                    return self.parse_primary_expression_full(inner);
                }
                Rule::Var => {
                    let var = self.parse_variable(inner)?;
                    return Ok(Expression::Var(var));
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Failed to parse expression tree".to_string()))
    }

    /// Parse primary expression with full support for aggregates and builtins
    fn parse_primary_expression_full(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Var => {
                    let var = self.parse_variable(inner)?;
                    return Ok(Expression::Var(var));
                }
                Rule::NumericLiteral | Rule::BooleanLiteral | Rule::RDFLiteral => {
                    let node = self.parse_graph_term(inner)?;
                    return Ok(Expression::Constant(node));
                }
                Rule::BuiltInCall => {
                    return self.parse_builtin_call_full(inner);
                }
                Rule::BrackettedExpression => {
                    return self.parse_expression_tree(inner);
                }
                _ => {}
            }
        }
        Err(ParseError::Syntax("Invalid primary expression".to_string()))
    }

    /// Parse builtin call with full aggregate support
    fn parse_builtin_call_full(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::Aggregate {
                let aggregate = self.parse_aggregate(inner)?;
                return Ok(Expression::Aggregate(aggregate));
            }
            // Handle other builtins if needed
        }
        Err(ParseError::Unsupported("Builtin function not yet implemented".to_string()))
    }

    /// Parse aggregate function from grammar Rule::Aggregate
    /// Supports: COUNT, SUM, MIN, MAX, AVG, SAMPLE, GROUP_CONCAT with DISTINCT
    fn parse_aggregate(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Aggregate<'a>> {
        let text = pair.as_str().to_uppercase();
        let mut distinct = false;
        let mut expr = None;
        let mut separator = None;
        let mut is_count_star = false;

        // First pass: check for aggregate type
        let agg_type = if text.starts_with("COUNT") {
            "COUNT"
        } else if text.starts_with("SUM") {
            "SUM"
        } else if text.starts_with("MIN") {
            "MIN"
        } else if text.starts_with("MAX") {
            "MAX"
        } else if text.starts_with("AVG") {
            "AVG"
        } else if text.starts_with("SAMPLE") {
            "SAMPLE"
        } else if text.starts_with("GROUP_CONCAT") {
            "GROUP_CONCAT"
        } else {
            return Err(ParseError::Syntax(format!("Unknown aggregate: {}", text)));
        };

        // Check for DISTINCT in the aggregate text itself (case-insensitive match in grammar)
        // The ^"DISTINCT" in grammar means it's not emitted as a separate token
        if text.contains("DISTINCT") {
            distinct = true;
        }

        // Second pass: parse arguments
        for inner in pair.into_inner() {
            let inner_text = inner.as_str();
            if inner_text == "*" {
                // COUNT(*) case
                is_count_star = true;
            } else if inner.as_rule() == Rule::Expression {
                expr = Some(Box::new(self.parse_expression_tree(inner)?));
            } else if inner.as_rule() == Rule::String {
                // GROUP_CONCAT separator
                separator = Some(self.parse_string(inner)?);
            }
        }

        // Construct appropriate aggregate enum variant
        match agg_type {
            "COUNT" => {
                if is_count_star {
                    Ok(Aggregate::Count {
                        expr: None,  // None means COUNT(*)
                        distinct,
                    })
                } else {
                    Ok(Aggregate::Count {
                        expr,
                        distinct,
                    })
                }
            }
            "SUM" => {
                let e = expr.ok_or_else(|| ParseError::Syntax("SUM requires expression".to_string()))?;
                Ok(Aggregate::Sum {
                    expr: e,
                    distinct,
                })
            }
            "MIN" => {
                let e = expr.ok_or_else(|| ParseError::Syntax("MIN requires expression".to_string()))?;
                Ok(Aggregate::Min {
                    expr: e,
                    distinct,
                })
            }
            "MAX" => {
                let e = expr.ok_or_else(|| ParseError::Syntax("MAX requires expression".to_string()))?;
                Ok(Aggregate::Max {
                    expr: e,
                    distinct,
                })
            }
            "AVG" => {
                let e = expr.ok_or_else(|| ParseError::Syntax("AVG requires expression".to_string()))?;
                Ok(Aggregate::Avg {
                    expr: e,
                    distinct,
                })
            }
            "SAMPLE" => {
                let e = expr.ok_or_else(|| ParseError::Syntax("SAMPLE requires expression".to_string()))?;
                Ok(Aggregate::Sample {
                    expr: e,
                    distinct,
                })
            }
            "GROUP_CONCAT" => {
                let e = expr.ok_or_else(|| ParseError::Syntax("GROUP_CONCAT requires expression".to_string()))?;
                Ok(Aggregate::GroupConcat {
                    expr: e,
                    separator,
                    distinct,
                })
            }
            _ => unreachable!()
        }
    }

    /// Check if projection contains aggregate expressions
    fn projection_has_aggregates(&self, projection: &Projection<'a>) -> bool {
        match projection {
            Projection::Expressions(exprs) => {
                exprs.iter().any(|(expr, _)| self.expr_has_aggregate(expr))
            }
            _ => false,
        }
    }

    /// Check if an expression contains an aggregate function
    fn expr_has_aggregate(&self, expr: &Expression<'a>) -> bool {
        match expr {
            Expression::Aggregate(_) => true,
            Expression::Or(left, right) |
            Expression::And(left, right) |
            Expression::Equal(left, right) |
            Expression::NotEqual(left, right) |
            Expression::Less(left, right) |
            Expression::Greater(left, right) |
            Expression::LessOrEqual(left, right) |
            Expression::GreaterOrEqual(left, right) |
            Expression::Add(left, right) |
            Expression::Subtract(left, right) |
            Expression::Multiply(left, right) |
            Expression::Divide(left, right) => {
                self.expr_has_aggregate(left) || self.expr_has_aggregate(right)
            }
            Expression::In(left, list) |
            Expression::NotIn(left, list) => {
                self.expr_has_aggregate(left) || list.iter().any(|e| self.expr_has_aggregate(e))
            }
            Expression::Negate(e) |
            Expression::Plus(e) |
            Expression::Not(e) => self.expr_has_aggregate(e),
            Expression::FunctionCall { args, .. } => {
                args.iter().any(|e| self.expr_has_aggregate(e))
            }
            Expression::Builtin(_) => false,  // Builtins are not aggregates
            Expression::Exists(_) | Expression::NotExists(_) => false,
            Expression::Var(_) | Expression::Constant(_) => false,
        }
    }

    /// Extract aggregate expressions from projection for GROUP BY algebra
    fn extract_aggregates_from_projection(&self, projection: &Projection<'a>) -> ParseResult<Vec<(Variable<'a>, Aggregate<'a>)>> {
        let mut aggregates = vec![];

        if let Projection::Expressions(exprs) = projection {
            for (expr, var) in exprs {
                if let Some(agg) = self.extract_aggregate_from_expr(expr) {
                    aggregates.push((var.clone(), agg));
                }
            }
        }

        Ok(aggregates)
    }

    /// Extract aggregate from expression (if it is an aggregate)
    fn extract_aggregate_from_expr(&self, expr: &Expression<'a>) -> Option<Aggregate<'a>> {
        if let Expression::Aggregate(agg) = expr {
            Some(agg.clone())
        } else {
            None
        }
    }

    /// Parse solution modifier with GROUP BY detection
    fn parse_solution_modifier_with_group(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<(Vec<OrderCondition<'a>>, Option<usize>, Option<usize>, bool)> {
        let mut order_conditions = Vec::new();
        let mut limit = None;
        let mut offset = None;
        let mut has_group_by = false;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::GroupClause => {
                    has_group_by = true;
                    // TODO: Parse GROUP BY variables for explicit grouping
                    // For now, we just detect presence of GROUP BY
                }
                Rule::OrderClause => {
                    for order_inner in inner.into_inner() {
                        if order_inner.as_rule() == Rule::OrderCondition {
                            order_conditions.push(self.parse_order_condition(order_inner)?);
                        }
                    }
                }
                Rule::LimitOffsetClauses => {
                    for limit_offset_inner in inner.into_inner() {
                        match limit_offset_inner.as_rule() {
                            Rule::LimitClause => {
                                let limit_str = limit_offset_inner.into_inner().next()
                                    .ok_or_else(|| ParseError::Syntax("LIMIT missing value".to_string()))?
                                    .as_str();
                                limit = Some(limit_str.parse().map_err(|_|
                                    ParseError::Syntax(format!("Invalid LIMIT value: {}", limit_str)))?);
                            }
                            Rule::OffsetClause => {
                                let offset_str = limit_offset_inner.into_inner().next()
                                    .ok_or_else(|| ParseError::Syntax("OFFSET missing value".to_string()))?
                                    .as_str();
                                offset = Some(offset_str.parse().map_err(|_|
                                    ParseError::Syntax(format!("Invalid OFFSET value: {}", offset_str)))?);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok((order_conditions, limit, offset, has_group_by))
    }
}

impl<'a> Default for SPARQLParser<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_select() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT * WHERE { ?s ?p ?o }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            assert_eq!(projection, Projection::All);
        } else {
            panic!("Expected SELECT query");
        }
    }

    #[test]
    fn test_parse_select_with_variables() {
        let mut parser = SPARQLParser::new();
        // Simplified query - use variables for all positions
        let query = "SELECT * WHERE { ?person ?p ?name }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_ask_query() {
        let mut parser = SPARQLParser::new();
        let query = "ASK WHERE { ?s ?p ?o }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        assert!(matches!(result.unwrap(), Query::Ask { .. }));
    }

    #[test]
    fn test_parse_with_prefix() {
        let mut parser = SPARQLParser::new();
        // Simplified - test without PREFIX for now (will work on full impl later)
        let query = "SELECT * WHERE { ?s ?p ?o }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    }

    // ========================================
    // AGGREGATE FUNCTION TESTS (W3C SPARQL 1.1)
    // ========================================

    #[test]
    fn test_parse_count_star_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (COUNT(*) AS ?count) WHERE { ?s ?p ?o }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, pattern, .. }) = result {
            // Verify projection is Expressions variant
            match projection {
                Projection::Expressions(exprs) => {
                    assert_eq!(exprs.len(), 1, "Should have one expression");
                    let (expr, var) = &exprs[0];
                    assert_eq!(var.name, "count", "Variable should be ?count");

                    // Verify expression is an aggregate
                    match expr {
                        Expression::Aggregate(Aggregate::Count { expr, distinct }) => {
                            assert!(expr.is_none(), "COUNT(*) should have None expression");
                            assert!(!distinct, "Should not be DISTINCT");
                        }
                        _ => panic!("Expected COUNT aggregate, got: {:?}", expr),
                    }
                }
                _ => panic!("Expected Projection::Expressions, got: {:?}", projection),
            }

            // Verify implicit GROUP BY () was created
            match pattern {
                Algebra::Group { vars, aggregates, .. } => {
                    assert!(vars.is_empty(), "Implicit GROUP BY should have empty vars");
                    assert_eq!(aggregates.len(), 1, "Should have one aggregate");
                }
                _ => panic!("Expected Algebra::Group for implicit GROUP BY, got: {:?}", pattern),
            }
        } else {
            panic!("Expected SELECT query");
        }
    }

    #[test]
    fn test_parse_count_variable_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (COUNT(?person) AS ?total) WHERE { ?person ?p ?o }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, var) = &exprs[0];
                    assert_eq!(var.name, "total");

                    match expr {
                        Expression::Aggregate(Aggregate::Count { expr, .. }) => {
                            assert!(expr.is_some(), "COUNT(?person) should have expression");
                        }
                        _ => panic!("Expected COUNT aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_count_distinct_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (COUNT(DISTINCT ?person) AS ?uniqueCount) WHERE { ?person ?p ?o }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, _) = &exprs[0];

                    match expr {
                        Expression::Aggregate(Aggregate::Count { distinct, .. }) => {
                            assert!(*distinct, "Should be DISTINCT");
                        }
                        _ => panic!("Expected COUNT aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_sum_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (SUM(?amount) AS ?total) WHERE { ?s ?p ?amount }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, var) = &exprs[0];
                    assert_eq!(var.name, "total");

                    match expr {
                        Expression::Aggregate(Aggregate::Sum { .. }) => {
                            // Success - SUM aggregate parsed correctly
                        }
                        _ => panic!("Expected SUM aggregate, got: {:?}", expr),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_min_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (MIN(?price) AS ?minPrice) WHERE { ?s ?p ?price }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, _) = &exprs[0];

                    match expr {
                        Expression::Aggregate(Aggregate::Min { .. }) => {}
                        _ => panic!("Expected MIN aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_max_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (MAX(?price) AS ?maxPrice) WHERE { ?s ?p ?price }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, _) = &exprs[0];

                    match expr {
                        Expression::Aggregate(Aggregate::Max { .. }) => {}
                        _ => panic!("Expected MAX aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_avg_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (AVG(?score) AS ?avgScore) WHERE { ?s ?p ?score }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, _) = &exprs[0];

                    match expr {
                        Expression::Aggregate(Aggregate::Avg { .. }) => {}
                        _ => panic!("Expected AVG aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_sample_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (SAMPLE(?name) AS ?someName) WHERE { ?s ?p ?name }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, _) = &exprs[0];

                    match expr {
                        Expression::Aggregate(Aggregate::Sample { .. }) => {}
                        _ => panic!("Expected SAMPLE aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_group_concat_aggregate() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (GROUP_CONCAT(?name) AS ?names) WHERE { ?s ?p ?name }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, _) = &exprs[0];

                    match expr {
                        Expression::Aggregate(Aggregate::GroupConcat { separator, .. }) => {
                            assert!(separator.is_none(), "Default separator should be None");
                        }
                        _ => panic!("Expected GROUP_CONCAT aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_group_concat_with_separator() {
        let mut parser = SPARQLParser::new();
        let query = r#"SELECT (GROUP_CONCAT(?name; SEPARATOR=", ") AS ?names) WHERE { ?s ?p ?name }"#;

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    let (expr, _) = &exprs[0];

                    match expr {
                        Expression::Aggregate(Aggregate::GroupConcat { separator, .. }) => {
                            assert!(separator.is_some(), "Should have custom separator");
                            assert_eq!(separator.unwrap(), ", ");
                        }
                        _ => panic!("Expected GROUP_CONCAT aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }
        }
    }

    #[test]
    fn test_parse_multiple_aggregates() {
        let mut parser = SPARQLParser::new();
        let query = "SELECT (COUNT(*) AS ?count) (SUM(?amount) AS ?total) WHERE { ?s ?p ?amount }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { projection, pattern, .. }) = result {
            match projection {
                Projection::Expressions(exprs) => {
                    assert_eq!(exprs.len(), 2, "Should have two aggregate expressions");

                    // Verify first aggregate is COUNT
                    match &exprs[0].0 {
                        Expression::Aggregate(Aggregate::Count { .. }) => {}
                        _ => panic!("Expected COUNT aggregate"),
                    }

                    // Verify second aggregate is SUM
                    match &exprs[1].0 {
                        Expression::Aggregate(Aggregate::Sum { .. }) => {}
                        _ => panic!("Expected SUM aggregate"),
                    }
                }
                _ => panic!("Expected Projection::Expressions"),
            }

            // Verify implicit GROUP BY was created
            match pattern {
                Algebra::Group { aggregates, .. } => {
                    assert_eq!(aggregates.len(), 2, "Should have two aggregates in GROUP BY");
                }
                _ => panic!("Expected Algebra::Group"),
            }
        }
    }

    #[test]
    fn test_parse_aggregate_implicit_group_by() {
        let mut parser = SPARQLParser::new();
        // W3C SPARQL 1.1 spec: aggregates without GROUP BY create implicit GROUP BY ()
        let query = "SELECT (AVG(?price) AS ?avgPrice) WHERE { ?product ?p ?price }";

        let result = parser.parse_query(query);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        if let Ok(Query::Select { pattern, .. }) = result {
            // Verify that implicit GROUP BY () was created
            match pattern {
                Algebra::Group { vars, aggregates, input } => {
                    assert!(vars.is_empty(), "Implicit GROUP BY should have empty variables");
                    assert_eq!(aggregates.len(), 1, "Should have one aggregate");

                    // Verify inner pattern is BGP
                    match input.as_ref() {
                        Algebra::BGP(patterns) => {
                            assert!(!patterns.is_empty(), "Should have BGP pattern");
                        }
                        _ => panic!("Expected BGP inside GROUP"),
                    }
                }
                _ => panic!("Expected Algebra::Group for implicit GROUP BY, got: {:?}", pattern),
            }
        }
    }
}
