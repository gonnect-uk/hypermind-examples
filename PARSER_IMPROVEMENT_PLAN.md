# Parser Improvement Plan

**Date**: 2025-11-26
**Target**: Production-Grade W3C Compliance
**Estimated Timeline**: 2-3 weeks to complete all improvements

---

## Executive Summary

This document provides actionable implementation plans for the 12 identified gaps in our RDF/SPARQL parsers. Each improvement includes specific code changes, test requirements, and complexity estimates.

**Priority Classification**:
- **P0 (Must Fix)**: 4 critical correctness issues - 4-5 days
- **P1 (Should Fix)**: 4 completeness issues - 3 days
- **P2 (Nice to Have)**: 4 edge cases and optimizations - 1.5 weeks

---

## Part 1: RDF Turtle Parser Improvements

### GAP-1: Blank Node Property List Expansion [P0]

**File**: `crates/rdf-io/src/turtle.rs:587`
**Priority**: P0 - CRITICAL
**Effort**: 2-3 hours
**Complexity**: Medium

#### Current Code
```rust
fn blank_node_property_list(input: &str) -> IResult<&str, NodePattern> {
    let (input, _) = char('[')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _pred_obj_list) = predicate_object_list(input)?;  // Parsed but discarded!
    let (input, _) = multispace0(input)?;
    let (input, _) = char(']')(input)?;

    // TODO: Expand blank node property list into separate triples
    // For now, return a generated blank node ID
    Ok((input, NodePattern::BlankNode("_anon_with_props".to_string())))
}
```

#### Problem
Input: `[ :name "Alice" ; :age 30 ] :knows :Bob .`

**Current Behavior**: Creates single blank node, loses properties
**Expected Behavior**: Expands to:
```turtle
_:b1 :name "Alice" .
_:b1 :age 30 .
_:b1 :knows :Bob .
```

#### Solution Design

**Step 1**: Capture predicate-object list during parsing
```rust
#[derive(Debug, Clone)]
struct BlankNodeWithProperties {
    id: String,
    properties: Vec<(NodePattern, Vec<NodePattern>)>,  // pred -> objects
}

fn blank_node_property_list(input: &str) -> IResult<&str, NodePattern> {
    let (input, _) = char('[')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, pred_obj_list) = predicate_object_list(input)?;  // CAPTURE this!
    let (input, _) = multispace0(input)?;
    let (input, _) = char(']')(input)?;

    Ok((input, NodePattern::BlankNodeWithProps {
        id: "_anon".to_string(),
        properties: pred_obj_list,
    }))
}
```

**Step 2**: Expand during triple resolution
```rust
fn resolve_triple<'a>(&mut self, triple: TriplePattern) -> ParseResult<Vec<Triple<'a>>> {
    let mut result_triples = Vec::new();

    // Expand subject if it's a blank node with properties
    let subject = match triple.subject {
        NodePattern::BlankNodeWithProps { id, properties } => {
            let bn_id = self.blank_node_counter;
            self.blank_node_counter += 1;
            let bn_node = Node::blank(bn_id);

            // Generate triples for properties: _:b :p :o
            for (pred, objects) in properties {
                let pred_node = self.resolve_node(pred)?;
                for obj in objects {
                    let obj_node = self.resolve_node(obj)?;
                    result_triples.push(Triple::new(bn_node, pred_node, obj_node));
                }
            }

            bn_node
        }
        other => self.resolve_node(other)?,
    };

    // Similarly for object position
    let predicate = self.resolve_node(triple.predicate)?;
    let object = self.resolve_node(triple.object)?;

    result_triples.push(Triple::new(subject, predicate, object));
    Ok(result_triples)
}
```

**Step 3**: Update parse() to handle Vec<Triple>
```rust
pub fn parse<'a>(&mut self, content: &'a str) -> ParseResult<Vec<Quad<'a>>> {
    // ... existing code ...

    for statement in statements {
        match statement {
            Statement::Triples(triples) => {
                for triple in triples {
                    // Expansion now returns Vec<Triple> instead of single Triple
                    let resolved_triples = self.resolve_triple(triple)?;
                    for resolved_triple in resolved_triples {
                        quads.push(Quad::from_triple(resolved_triple));
                    }
                }
            }
            // ...
        }
    }

    Ok(quads)
}
```

#### Testing Requirements

**Test 1: Basic Property List**
```rust
#[test]
fn test_blank_node_property_list_expansion() {
    let dict = Arc::new(Dictionary::new());
    let mut parser = TurtleParser::new(Arc::clone(&dict));

    let turtle = r#"
        PREFIX : <http://example/>
        [ :name "Alice" ; :age 30 ] :knows :Bob .
    "#;

    let quads = parser.parse(turtle).unwrap();

    // Should produce 3 triples:
    // _:b1 :name "Alice" .
    // _:b1 :age 30 .
    // _:b1 :knows :Bob .
    assert_eq!(quads.len(), 3);

    // Verify first two triples have same blank node subject
    let bn = match &quads[0].subject {
        Node::BlankNode(id) => *id,
        _ => panic!("Expected blank node"),
    };

    match &quads[1].subject {
        Node::BlankNode(id) => assert_eq!(*id, bn),
        _ => panic!("Expected same blank node"),
    }
}
```

**Test 2: Nested Property Lists**
```rust
#[test]
fn test_nested_blank_node_property_lists() {
    let turtle = r#"
        PREFIX : <http://example/>
        :Alice :friend [ :name "Bob" ; :friend [ :name "Carol" ] ] .
    "#;

    let quads = parser.parse(turtle).unwrap();

    // Should produce 5 triples:
    // :Alice :friend _:b1 .
    // _:b1 :name "Bob" .
    // _:b1 :friend _:b2 .
    // _:b2 :name "Carol" .
    assert_eq!(quads.len(), 4);
}
```

**Test 3: W3C Conformance**
```rust
#[test]
fn test_w3c_blank_node_property_list() {
    // Run W3C test: blankNodePropertyList-001.ttl
    let test_file = "test-data/rdf-tests/rdf/rdf12/rdf-turtle/blankNodePropertyList-001.ttl";
    let expected = "test-data/rdf-tests/rdf/rdf12/rdf-turtle/blankNodePropertyList-001.nt";

    let quads = parser.parse(fs::read_to_string(test_file)?)?;
    let expected_quads = load_ntriples(expected)?;

    assert_graph_isomorphic(&quads, &expected_quads);
}
```

#### Implementation Checklist

- [ ] Add `BlankNodeWithProps` to `NodePattern` enum
- [ ] Modify `blank_node_property_list()` to capture properties
- [ ] Change `resolve_triple()` signature to return `Vec<Triple>`
- [ ] Implement expansion logic in `resolve_triple()`
- [ ] Update `parse()` to handle Vec<Triple>
- [ ] Add 5+ unit tests
- [ ] Run W3C blankNodePropertyList tests
- [ ] Update documentation

---

### GAP-2: RDF Collection Expansion [P0]

**File**: `crates/rdf-io/src/turtle.rs:175`
**Priority**: P0 - CRITICAL
**Effort**: 4-5 hours
**Complexity**: High (recursive structure)

#### Current Code
```rust
NodePattern::Collection(items) => {
    // For now, return blank node (proper RDF list requires expansion)
    let id = self.blank_node_counter;
    self.blank_node_counter += 1;
    Ok(Node::blank(id))
}
```

#### Problem
Input: `( :a :b :c )`

**Current**: Returns single blank node
**Expected**: Expands to RDF list structure:
```turtle
_:b1 rdf:first :a ; rdf:rest _:b2 .
_:b2 rdf:first :b ; rdf:rest _:b3 .
_:b3 rdf:first :c ; rdf:rest rdf:nil .
```

#### Solution Design

**Step 1**: Expand collection recursively
```rust
/// Expand RDF collection into rdf:first/rdf:rest chain
fn expand_collection<'a>(
    &mut self,
    items: Vec<NodePattern>,
) -> ParseResult<(Node<'a>, Vec<Triple<'a>>)> {
    if items.is_empty() {
        // Empty collection () = rdf:nil
        let nil_uri = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil");
        return Ok((Node::iri(nil_uri), vec![]));
    }

    let mut triples = Vec::new();
    let first_bn_id = self.blank_node_counter;
    self.blank_node_counter += 1;

    let rdf_first = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#first");
    let rdf_rest = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#rest");

    let mut current_bn = Node::blank(first_bn_id);

    for (i, item) in items.into_iter().enumerate() {
        let item_node = self.resolve_node(item)?;

        // Current blank node rdf:first item
        triples.push(Triple::new(
            current_bn,
            Node::iri(rdf_first),
            item_node,
        ));

        // Determine rdf:rest value
        let rest_value = if i == items.len() - 1 {
            // Last item: rest is rdf:nil
            let nil = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil");
            Node::iri(nil)
        } else {
            // Not last: rest is next blank node
            let next_bn_id = self.blank_node_counter;
            self.blank_node_counter += 1;
            Node::blank(next_bn_id)
        };

        // Current blank node rdf:rest next_bn
        triples.push(Triple::new(
            current_bn,
            Node::iri(rdf_rest),
            rest_value,
        ));

        current_bn = rest_value;  // Move to next node
    }

    Ok((Node::blank(first_bn_id), triples))
}
```

**Step 2**: Integrate into resolve_node()
```rust
fn resolve_node<'a>(&mut self, node: NodePattern) -> ParseResult<(Node<'a>, Vec<Triple<'a>>)> {
    match node {
        NodePattern::Collection(items) => {
            let (head_node, expansion_triples) = self.expand_collection(items)?;
            Ok((head_node, expansion_triples))
        }
        NodePattern::IriRef(iri) => {
            let interned = self.dictionary.intern(&iri);
            Ok((Node::iri(interned), vec![]))  // No expansion
        }
        // ... other patterns
    }
}
```

**Step 3**: Update callers to accumulate triples
```rust
fn resolve_triple<'a>(&mut self, triple: TriplePattern) -> ParseResult<Vec<Triple<'a>>> {
    let mut all_triples = Vec::new();

    let (subject, subj_triples) = self.resolve_node(triple.subject)?;
    all_triples.extend(subj_triples);

    let (predicate, pred_triples) = self.resolve_node(triple.predicate)?;
    all_triples.extend(pred_triples);

    let (object, obj_triples) = self.resolve_node(triple.object)?;
    all_triples.extend(obj_triples);

    // Main triple
    all_triples.push(Triple::new(subject, predicate, object));

    Ok(all_triples)
}
```

#### Testing Requirements

**Test 1: Simple Collection**
```rust
#[test]
fn test_rdf_collection_expansion() {
    let turtle = r#"
        PREFIX : <http://example/>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        :list rdf:value ( :a :b :c ) .
    "#;

    let quads = parser.parse(turtle).unwrap();

    // Should produce 7 triples:
    // :list rdf:value _:b1 .
    // _:b1 rdf:first :a ; rdf:rest _:b2 .
    // _:b2 rdf:first :b ; rdf:rest _:b3 .
    // _:b3 rdf:first :c ; rdf:rest rdf:nil .
    assert_eq!(quads.len(), 7);

    // Verify rdf:nil termination
    let last_triple = &quads[quads.len() - 1];
    assert!(matches!(last_triple.object, Node::Iri(uri) if uri.0.ends_with("#nil")));
}
```

**Test 2: Empty Collection**
```rust
#[test]
fn test_empty_collection() {
    let turtle = r#"
        PREFIX : <http://example/>
        :emptyList rdf:value ( ) .
    "#;

    let quads = parser.parse(turtle).unwrap();

    // Should produce 1 triple: :emptyList rdf:value rdf:nil
    assert_eq!(quads.len(), 1);
    assert!(matches!(quads[0].object, Node::Iri(uri) if uri.0.ends_with("#nil")));
}
```

**Test 3: Nested Collections**
```rust
#[test]
fn test_nested_collections() {
    let turtle = r#"
        PREFIX : <http://example/>
        :nested rdf:value ( ( :a :b ) ( :c :d ) ) .
    "#;

    let quads = parser.parse(turtle).unwrap();

    // Nested collections expand recursively
    assert!(quads.len() > 10);
}
```

#### Implementation Checklist

- [ ] Implement `expand_collection()` function
- [ ] Change `resolve_node()` to return `(Node, Vec<Triple>)`
- [ ] Update all callers to accumulate expansion triples
- [ ] Handle empty collection `()` → `rdf:nil`
- [ ] Handle nested collections recursively
- [ ] Add 10+ unit tests (simple, empty, nested, edge cases)
- [ ] Run W3C collection tests
- [ ] Performance test with large collections (1000+ items)

---

### GAP-3: IRI Resolution (Base + Relative) [P1]

**File**: `crates/rdf-io/src/turtle.rs:183`
**Priority**: P1 - MEDIUM
**Effort**: 1 hour
**Complexity**: Low

#### Current Code
```rust
fn resolve_prefixed_name(&self, prefix: &str, local: &str) -> ParseResult<String> {
    if let Some(namespace) = self.prefixes.get(prefix) {
        Ok(format!("{}{}", namespace, local))
    } else {
        Err(ParseError::InvalidIri(format!("Unknown prefix: '{}'", prefix)))
    }
}

// Base directive is parsed but never used!
Directive::Base { iri } => {
    self.base = Some(iri);  // Stored but not applied
}
```

#### Solution Design

**Step 1**: Add IRI resolution helper
```rust
/// Resolve IRI against base URI
/// Implements RFC 3986 URI resolution
fn resolve_iri(&self, iri: &str) -> String {
    // Check if already absolute
    if iri.contains("://") {
        return iri.to_string();
    }

    // Resolve against base
    match &self.base {
        Some(base) => {
            // Simple resolution (full RFC 3986 requires more logic)
            if iri.starts_with('/') {
                // Absolute path: use base scheme + authority
                format!("{}{}", base.trim_end_matches('/'), iri)
            } else {
                // Relative path: append to base
                format!("{}/{}", base.trim_end_matches('/'), iri)
            }
        }
        None => {
            // No base - return as-is (may be invalid)
            iri.to_string()
        }
    }
}
```

**Step 2**: Use in IriRef parsing
```rust
fn resolve_node<'a>(&mut self, node: NodePattern) -> ParseResult<Node<'a>> {
    match node {
        NodePattern::IriRef(iri) => {
            let resolved = self.resolve_iri(&iri);  // Apply base resolution
            let interned = self.dictionary.intern(&resolved);
            Ok(Node::iri(interned))
        }
        // ... other cases
    }
}
```

**Step 3**: For production, use `url` crate
```rust
// Cargo.toml
[dependencies]
url = "2.5"

// Use proper RFC 3986 resolution
use url::Url;

fn resolve_iri(&self, iri: &str) -> ParseResult<String> {
    match &self.base {
        Some(base_str) => {
            let base = Url::parse(base_str)
                .map_err(|e| ParseError::InvalidIri(format!("Invalid base: {}", e)))?;
            let resolved = base.join(iri)
                .map_err(|e| ParseError::InvalidIri(format!("IRI resolution failed: {}", e)))?;
            Ok(resolved.to_string())
        }
        None => Ok(iri.to_string()),
    }
}
```

#### Testing Requirements

**Test 1: Base Resolution**
```rust
#[test]
fn test_base_iri_resolution() {
    let turtle = r#"
        @base <http://example.org/> .
        <relative> :p :o .
    "#;

    let quads = parser.parse(turtle).unwrap();

    // Subject should be <http://example.org/relative>
    match &quads[0].subject {
        Node::Iri(uri) => {
            assert_eq!(uri.0, "http://example.org/relative");
        }
        _ => panic!("Expected IRI"),
    }
}
```

**Test 2: Absolute IRI Not Affected**
```rust
#[test]
fn test_absolute_iri_not_resolved() {
    let turtle = r#"
        @base <http://example.org/> .
        <http://other.com/absolute> :p :o .
    "#;

    let quads = parser.parse(turtle).unwrap();

    // Should remain http://other.com/absolute
    match &quads[0].subject {
        Node::Iri(uri) => {
            assert_eq!(uri.0, "http://other.com/absolute");
        }
        _ => panic!("Expected IRI"),
    }
}
```

#### Implementation Checklist

- [ ] Add `url` crate dependency
- [ ] Implement `resolve_iri()` with RFC 3986 semantics
- [ ] Apply resolution in `resolve_node()` for IriRef
- [ ] Add 5+ tests (relative, absolute, paths, fragments)
- [ ] Run W3C IRI resolution tests
- [ ] Handle edge cases (no base, invalid base, etc.)

---

### GAP-4: Unicode Escape Decoding [P2]

**File**: `crates/rdf-io/src/turtle.rs` (string parsing)
**Priority**: P2 - LOW
**Effort**: 2 hours
**Complexity**: Low

#### Current Code
```rust
fn string_literal(input: &str) -> IResult<&str, String> {
    alt((
        map(string_literal_long_quote, |s| s[3..s.len()-3].to_string()),
        map(string_literal_quote, |s| s[1..s.len()-1].to_string()),
        // ...
    ))(input)
}
// Issue: Passes through \uXXXX literally, doesn't decode
```

#### Solution Design

**Step 1**: Add escape decoding
```rust
/// Decode Unicode and character escapes in strings
/// Handles: \uXXXX, \UXXXXXXXX, \t, \n, \r, etc.
fn decode_escapes(s: &str) -> ParseResult<String> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('u') => {
                    // \uXXXX - 4 hex digits
                    let hex: String = chars.by_ref().take(4).collect();
                    if hex.len() != 4 {
                        return Err(ParseError::Syntax {
                            line: 0, col: 0,
                            message: format!("Invalid \\u escape: \\u{}", hex),
                        });
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| ParseError::Syntax {
                            line: 0, col: 0,
                            message: format!("Invalid hex in \\u{}", hex),
                        })?;
                    let unicode_char = char::from_u32(code)
                        .ok_or_else(|| ParseError::Syntax {
                            line: 0, col: 0,
                            message: format!("Invalid Unicode codepoint: U+{:04X}", code),
                        })?;
                    result.push(unicode_char);
                }
                Some('U') => {
                    // \UXXXXXXXX - 8 hex digits
                    let hex: String = chars.by_ref().take(8).collect();
                    if hex.len() != 8 {
                        return Err(ParseError::Syntax {
                            line: 0, col: 0,
                            message: format!("Invalid \\U escape: \\U{}", hex),
                        });
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| ParseError::Syntax {
                            line: 0, col: 0,
                            message: format!("Invalid hex in \\U{}", hex),
                        })?;
                    let unicode_char = char::from_u32(code)
                        .ok_or_else(|| ParseError::Syntax {
                            line: 0, col: 0,
                            message: format!("Invalid Unicode codepoint: U+{:08X}", code),
                        })?;
                    result.push(unicode_char);
                }
                // ECHAR - standard escapes
                Some('t') => result.push('\t'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('b') => result.push('\x08'),  // backspace
                Some('f') => result.push('\x0C'),  // form feed
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    return Err(ParseError::Syntax {
                        line: 0, col: 0,
                        message: format!("Invalid escape sequence: \\{}", other),
                    });
                }
                None => {
                    return Err(ParseError::Syntax {
                        line: 0, col: 0,
                        message: "Incomplete escape sequence at end of string".to_string(),
                    });
                }
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}
```

**Step 2**: Apply in string parsing
```rust
fn string_literal(input: &str) -> IResult<&str, String> {
    let (input, raw_str) = alt((
        string_literal_long_quote,
        string_literal_quote,
        // ...
    ))(input)?;

    // Decode escapes
    let decoded = decode_escapes(raw_str)
        .map_err(|e| nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?;

    Ok((input, decoded))
}
```

#### Testing Requirements

```rust
#[test]
fn test_unicode_escape_decoding() {
    let tests = vec![
        (r#""Hello\u0020World""#, "Hello World"),  // Space
        (r#""\u00E9""#, "é"),                       // Latin small letter e with acute
        (r#""\U0001F600""#, "😀"),                  // Emoji
        (r#""Tab\tNewline\n""#, "Tab\tNewline\n"), // Standard escapes
    ];

    for (input, expected) in tests {
        let turtle = format!("PREFIX : <http://example/> :s :p {} .", input);
        let quads = parser.parse(&turtle).unwrap();
        match &quads[0].object {
            Node::Literal(lit) => assert_eq!(lit.value, expected),
            _ => panic!("Expected literal"),
        }
    }
}
```

#### Implementation Checklist

- [ ] Implement `decode_escapes()` function
- [ ] Handle `\uXXXX` (4-digit Unicode)
- [ ] Handle `\UXXXXXXXX` (8-digit Unicode)
- [ ] Handle ECHAR (`\t`, `\n`, `\r`, `\b`, `\f`, `\"`, `\'`, `\\`)
- [ ] Validate Unicode codepoints
- [ ] Add 10+ tests (all escape types)
- [ ] Run W3C escape tests

---

## Part 2: SPARQL Parser Improvements

### GAP-7: GROUP BY Variable Parsing [P0]

**File**: `crates/sparql/src/parser.rs:1406`
**Priority**: P0 - CRITICAL
**Effort**: 3-4 hours
**Complexity**: Medium

#### Current Code
```rust
Rule::GroupClause => {
    has_group_by = true;
    // TODO: Parse GROUP BY variables for explicit grouping
    // For now, we just detect presence of GROUP BY
}
```

#### Solution Design

**Step 1**: Parse GROUP BY variables
```rust
/// Parse GROUP BY clause and extract grouping variables
fn parse_group_clause(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Vec<Variable<'a>>> {
    let mut group_vars = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::GroupCondition {
            // GroupCondition can be:
            // - Var (most common)
            // - BuiltInCall
            // - FunctionCall
            // - "(" Expression ( AS Var )? ")"

            for cond_inner in inner.into_inner() {
                match cond_inner.as_rule() {
                    Rule::Var => {
                        let var = self.parse_variable(cond_inner)?;
                        group_vars.push(var);
                    }
                    Rule::Expression => {
                        // GROUP BY expression - extract variable if AS binding exists
                        // For now, skip complex expressions
                        // TODO: Support GROUP BY (expr AS ?var)
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(group_vars)
}
```

**Step 2**: Update solution modifier parsing
```rust
fn parse_solution_modifier_with_group(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<(Vec<OrderCondition<'a>>, Option<usize>, Option<usize>, Option<Vec<Variable<'a>>>)>
{
    let mut order_conditions = Vec::new();
    let mut limit = None;
    let mut offset = None;
    let mut group_vars = None;  // Change from bool to Option<Vec<Variable>>

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::GroupClause => {
                let vars = self.parse_group_clause(inner)?;
                group_vars = Some(vars);
            }
            // ... other rules
        }
    }

    Ok((order_conditions, limit, offset, group_vars))
}
```

**Step 3**: Create Algebra::Group with variables
```rust
let (order, limit, offset, group_vars_opt) = self.parse_solution_modifier_with_group(inner)?;

// Check if aggregates present
let has_aggregates = self.projection_has_aggregates(&projection);

if has_aggregates {
    let aggregates = self.extract_aggregates_from_projection(&projection)?;

    let group_vars = match group_vars_opt {
        Some(vars) => vars,  // Explicit GROUP BY
        None => vec![],      // Implicit GROUP BY () for aggregates without GROUP BY
    };

    pattern = Algebra::Group {
        vars: group_vars,
        aggregates,
        input: Box::new(pattern),
    };
}
```

#### Testing Requirements

**Test 1: Explicit GROUP BY**
```rust
#[test]
fn test_group_by_variable() {
    let query = r#"
        SELECT ?dept (AVG(?salary) AS ?avgSalary)
        WHERE { ?emp :dept ?dept ; :salary ?salary }
        GROUP BY ?dept
    "#;

    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, .. } => {
            match pattern {
                Algebra::Group { vars, aggregates, .. } => {
                    assert_eq!(vars.len(), 1);
                    assert_eq!(vars[0].name, "dept");
                    assert_eq!(aggregates.len(), 1);
                }
                _ => panic!("Expected GROUP algebra"),
            }
        }
        _ => panic!("Expected SELECT query"),
    }
}
```

**Test 2: Multiple GROUP BY Variables**
```rust
#[test]
fn test_group_by_multiple_variables() {
    let query = r#"
        SELECT ?dept ?location (COUNT(?emp) AS ?count)
        WHERE { ?emp :dept ?dept ; :location ?location }
        GROUP BY ?dept ?location
    "#;

    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, .. } => {
            match pattern {
                Algebra::Group { vars, .. } => {
                    assert_eq!(vars.len(), 2);
                    assert_eq!(vars[0].name, "dept");
                    assert_eq!(vars[1].name, "location");
                }
                _ => panic!("Expected GROUP algebra"),
            }
        }
        _ => panic!("Expected SELECT query"),
    }
}
```

#### Implementation Checklist

- [ ] Implement `parse_group_clause()` function
- [ ] Extract variables from GroupCondition
- [ ] Change `has_group_by: bool` to `group_vars: Option<Vec<Variable>>`
- [ ] Update all callers to use new signature
- [ ] Support GROUP BY expressions with AS
- [ ] Add 5+ tests (simple, multiple vars, expressions)
- [ ] Verify executor handles grouped vars correctly

---

### GAP-9: Builtin Function Implementation [P0]

**File**: `crates/sparql/src/parser.rs:877`
**Priority**: P0 - CRITICAL
**Effort**: 2-3 days
**Complexity**: High (64 functions)

#### Current Code
```rust
fn parse_builtin_call(&mut self, _pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
    // Placeholder for built-in functions (BOUND, isIRI, etc.)
    Err(ParseError::Unsupported("Built-in functions not yet fully implemented".to_string()))
}
```

#### Solution Design (Systematic Approach)

**Step 1**: Create builtin function enum (may already exist in algebra.rs)
```rust
// In algebra.rs
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction<'a> {
    // String functions
    Str(Box<Expression<'a>>),
    Lang(Box<Expression<'a>>),
    LangMatches(Box<Expression<'a>>, Box<Expression<'a>>),
    Datatype(Box<Expression<'a>>),
    Strlen(Box<Expression<'a>>),
    Substr(Box<Expression<'a>>, Box<Expression<'a>>, Option<Box<Expression<'a>>>),
    Ucase(Box<Expression<'a>>),
    Lcase(Box<Expression<'a>>),
    StrStarts(Box<Expression<'a>>, Box<Expression<'a>>),
    StrEnds(Box<Expression<'a>>, Box<Expression<'a>>),
    Contains(Box<Expression<'a>>, Box<Expression<'a>>),
    StrBefore(Box<Expression<'a>>, Box<Expression<'a>>),
    StrAfter(Box<Expression<'a>>, Box<Expression<'a>>),
    Concat(Vec<Expression<'a>>),
    Replace(Box<Expression<'a>>, Box<Expression<'a>>, Box<Expression<'a>>, Option<Box<Expression<'a>>>),
    EncodeForUri(Box<Expression<'a>>),

    // Test functions
    Bound(Variable<'a>),
    IsIri(Box<Expression<'a>>),
    IsBlank(Box<Expression<'a>>),
    IsLiteral(Box<Expression<'a>>),
    IsNumeric(Box<Expression<'a>>),
    SameTerm(Box<Expression<'a>>, Box<Expression<'a>>),

    // Numeric functions
    Abs(Box<Expression<'a>>),
    Round(Box<Expression<'a>>),
    Ceil(Box<Expression<'a>>),
    Floor(Box<Expression<'a>>),
    Rand,

    // Date/Time functions
    Now,
    Year(Box<Expression<'a>>),
    Month(Box<Expression<'a>>),
    Day(Box<Expression<'a>>),
    Hours(Box<Expression<'a>>),
    Minutes(Box<Expression<'a>>),
    Seconds(Box<Expression<'a>>),
    Timezone(Box<Expression<'a>>),
    Tz(Box<Expression<'a>>),

    // Hash functions
    Md5(Box<Expression<'a>>),
    Sha1(Box<Expression<'a>>),
    Sha256(Box<Expression<'a>>),
    Sha384(Box<Expression<'a>>),
    Sha512(Box<Expression<'a>>),

    // Constructor functions
    Iri(Box<Expression<'a>>),
    Bnode(Option<Box<Expression<'a>>>),
    Strdt(Box<Expression<'a>>, Box<Expression<'a>>),
    Strlang(Box<Expression<'a>>, Box<Expression<'a>>),
    Uuid,
    Struuid,

    // Conditional
    If(Box<Expression<'a>>, Box<Expression<'a>>, Box<Expression<'a>>),
    Coalesce(Vec<Expression<'a>>),

    // Regex
    Regex(Box<Expression<'a>>, Box<Expression<'a>>, Option<Box<Expression<'a>>>),
}
```

**Step 2**: Implement comprehensive parser
```rust
fn parse_builtin_call_full(&mut self, pair: pest::iterators::Pair<'a, Rule>) -> ParseResult<Expression<'a>> {
    for inner in pair.into_inner() {
        let builtin_text = inner.as_str();

        match builtin_text.to_uppercase().as_str() {
            // String functions (21 functions)
            s if s.starts_with("STR(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Str(Box::new(expr))));
            }
            s if s.starts_with("LANG(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Lang(Box::new(expr))));
            }
            s if s.starts_with("LANGMATCHES(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::LangMatches(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }
            s if s.starts_with("DATATYPE(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Datatype(Box::new(expr))));
            }
            s if s.starts_with("STRLEN(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Strlen(Box::new(expr))));
            }
            s if s.starts_with("SUBSTR(") => {
                let exprs = self.parse_substr_expression(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Substr(
                    Box::new(exprs.0), Box::new(exprs.1), exprs.2.map(Box::new)
                )));
            }
            s if s.starts_with("UCASE(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Ucase(Box::new(expr))));
            }
            s if s.starts_with("LCASE(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Lcase(Box::new(expr))));
            }
            s if s.starts_with("STRSTARTS(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::StrStarts(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }
            s if s.starts_with("STRENDS(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::StrEnds(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }
            s if s.starts_with("CONTAINS(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Contains(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }
            s if s.starts_with("STRBEFORE(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::StrBefore(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }
            s if s.starts_with("STRAFTER(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::StrAfter(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }
            s if s.starts_with("CONCAT(") => {
                let exprs = self.parse_expression_list(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Concat(exprs)));
            }
            s if s.starts_with("REPLACE(") => {
                let exprs = self.parse_replace_expression(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Replace(
                    Box::new(exprs.0), Box::new(exprs.1), Box::new(exprs.2), exprs.3.map(Box::new)
                )));
            }
            s if s.starts_with("ENCODE_FOR_URI(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::EncodeForUri(Box::new(expr))));
            }

            // Test functions (6 functions)
            s if s.starts_with("BOUND(") => {
                let var = self.parse_variable_from_rule(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Bound(var)));
            }
            s if s.starts_with("ISIRI(") || s.starts_with("ISURI(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::IsIri(Box::new(expr))));
            }
            s if s.starts_with("ISBLANK(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::IsBlank(Box::new(expr))));
            }
            s if s.starts_with("ISLITERAL(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::IsLiteral(Box::new(expr))));
            }
            s if s.starts_with("ISNUMERIC(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::IsNumeric(Box::new(expr))));
            }
            s if s.starts_with("SAMETERM(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::SameTerm(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }

            // Numeric functions (5 functions)
            s if s.starts_with("ABS(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Abs(Box::new(expr))));
            }
            s if s.starts_with("ROUND(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Round(Box::new(expr))));
            }
            s if s.starts_with("CEIL(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Ceil(Box::new(expr))));
            }
            s if s.starts_with("FLOOR(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Floor(Box::new(expr))));
            }
            "RAND()" => {
                return Ok(Expression::Builtin(BuiltinFunction::Rand));
            }

            // Date/Time functions (9 functions)
            "NOW()" => {
                return Ok(Expression::Builtin(BuiltinFunction::Now));
            }
            s if s.starts_with("YEAR(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Year(Box::new(expr))));
            }
            s if s.starts_with("MONTH(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Month(Box::new(expr))));
            }
            s if s.starts_with("DAY(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Day(Box::new(expr))));
            }
            s if s.starts_with("HOURS(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Hours(Box::new(expr))));
            }
            s if s.starts_with("MINUTES(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Minutes(Box::new(expr))));
            }
            s if s.starts_with("SECONDS(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Seconds(Box::new(expr))));
            }
            s if s.starts_with("TIMEZONE(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Timezone(Box::new(expr))));
            }
            s if s.starts_with("TZ(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Tz(Box::new(expr))));
            }

            // Hash functions (5 functions)
            s if s.starts_with("MD5(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Md5(Box::new(expr))));
            }
            s if s.starts_with("SHA1(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Sha1(Box::new(expr))));
            }
            s if s.starts_with("SHA256(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Sha256(Box::new(expr))));
            }
            s if s.starts_with("SHA384(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Sha384(Box::new(expr))));
            }
            s if s.starts_with("SHA512(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Sha512(Box::new(expr))));
            }

            // Constructor functions (6 functions)
            s if s.starts_with("IRI(") || s.starts_with("URI(") => {
                let expr = self.parse_expression_from_rule(inner, Rule::Expression)?;
                return Ok(Expression::Builtin(BuiltinFunction::Iri(Box::new(expr))));
            }
            s if s.starts_with("BNODE(") => {
                let expr_opt = self.parse_optional_expression(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Bnode(expr_opt.map(Box::new))));
            }
            s if s.starts_with("STRDT(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Strdt(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }
            s if s.starts_with("STRLANG(") => {
                let exprs = self.parse_two_expressions(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Strlang(
                    Box::new(exprs.0), Box::new(exprs.1)
                )));
            }
            "UUID()" => {
                return Ok(Expression::Builtin(BuiltinFunction::Uuid));
            }
            "STRUUID()" => {
                return Ok(Expression::Builtin(BuiltinFunction::Struuid));
            }

            // Conditional (2 functions)
            s if s.starts_with("IF(") => {
                let exprs = self.parse_if_expression(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::If(
                    Box::new(exprs.0), Box::new(exprs.1), Box::new(exprs.2)
                )));
            }
            s if s.starts_with("COALESCE(") => {
                let exprs = self.parse_expression_list(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Coalesce(exprs)));
            }

            // Regex (1 function)
            s if s.starts_with("REGEX(") => {
                let exprs = self.parse_regex_expression(inner)?;
                return Ok(Expression::Builtin(BuiltinFunction::Regex(
                    Box::new(exprs.0), Box::new(exprs.1), exprs.2.map(Box::new)
                )));
            }

            _ => {
                // Unknown builtin - fall through to error
            }
        }
    }

    Err(ParseError::Unsupported(format!("Unknown builtin function: {}", pair.as_str())))
}
```

**Step 3**: Helper functions
```rust
/// Parse two expressions from BuiltInCall rule
fn parse_two_expressions(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<(Expression<'a>, Expression<'a>)>
{
    let mut exprs = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::Expression {
            exprs.push(self.parse_expression_tree(inner)?);
        }
    }

    if exprs.len() != 2 {
        return Err(ParseError::Syntax(format!(
            "Expected 2 expressions, got {}", exprs.len()
        )));
    }

    Ok((exprs.remove(0), exprs.remove(0)))
}

/// Parse expression list (for CONCAT, COALESCE, etc.)
fn parse_expression_list(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<Vec<Expression<'a>>>
{
    let mut exprs = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::Expression {
            exprs.push(self.parse_expression_tree(inner)?);
        }
    }
    Ok(exprs)
}

// Similar helpers for SUBSTR, REPLACE, REGEX, IF
```

#### Testing Requirements

**Systematic Test Coverage** (64 functions × 2 tests each = 128 tests)

```rust
// String functions (21 × 2 = 42 tests)
#[test]
fn test_str_function() {
    let query = r#"SELECT (STR(?x) AS ?str) WHERE { BIND("test" AS ?x) }"#;
    // ... verify builtin parsed
}

#[test]
fn test_str_function_execution() {
    // Verify executor implements STR correctly
}

// Repeat for all 64 builtins ...

#[test]
fn test_all_builtins_parsed() {
    // Comprehensive test ensuring all 64 builtins parse
    let builtins = vec![
        "STR(?x)", "LANG(?x)", "LANGMATCHES(?x, \"en\")", /* ... all 64 ... */
    ];

    for builtin_str in builtins {
        let query = format!("SELECT ({} AS ?result) WHERE {{ }}", builtin_str);
        let result = parser.parse_query(&query);
        assert!(result.is_ok(), "Failed to parse: {}", builtin_str);
    }
}
```

#### Implementation Checklist

- [ ] Add all 64 builtin variants to BuiltinFunction enum
- [ ] Implement parse logic for each category (string, test, numeric, date, hash, constructor, conditional, regex)
- [ ] Add helper functions (parse_two_expressions, parse_expression_list, etc.)
- [ ] Create 128 tests (2 per builtin: parsing + execution)
- [ ] Verify executor implements all 64 builtins
- [ ] Run W3C builtin function tests
- [ ] Document each function with examples

**Estimated Breakdown**:
- Day 1: String functions (21) + helpers
- Day 2: Test (6) + Numeric (5) + Date (9) + Hash (5)
- Day 3: Constructor (6) + Conditional (2) + Regex (1) + testing

---

### GAP-8: HAVING Clause Implementation [P1]

**File**: Grammar defined, parser not implemented
**Priority**: P1 - MEDIUM
**Effort**: 2 hours
**Complexity**: Low (reuse constraint parsing)

#### Solution Design

**Step 1**: Parse HAVING clause
```rust
fn parse_having_clause(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<Vec<Expression<'a>>>
{
    let mut conditions = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::HavingCondition {
            let constraint = self.parse_constraint(inner)?;
            conditions.push(constraint);
        }
    }

    Ok(conditions)
}
```

**Step 2**: Update solution modifier
```rust
fn parse_solution_modifier_with_group(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<(Vec<OrderCondition<'a>>, Option<usize>, Option<usize>, Option<Vec<Variable<'a>>>, Option<Vec<Expression<'a>>>)>
{
    // ... existing code ...
    let mut having = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::HavingClause => {
                having = Some(self.parse_having_clause(inner)?);
            }
            // ... other rules
        }
    }

    Ok((order_conditions, limit, offset, group_vars, having))
}
```

**Step 3**: Add HAVING to Algebra::Group
```rust
// In algebra.rs
pub enum Algebra<'a> {
    Group {
        vars: Vec<Variable<'a>>,
        aggregates: Vec<(Variable<'a>, Aggregate<'a>)>,
        input: Box<Algebra<'a>>,
        having: Option<Vec<Expression<'a>>>,  // NEW FIELD
    },
    // ... other variants
}
```

#### Testing Requirements

```rust
#[test]
fn test_having_clause() {
    let query = r#"
        SELECT ?dept (AVG(?salary) AS ?avgSalary)
        WHERE { ?emp :dept ?dept ; :salary ?salary }
        GROUP BY ?dept
        HAVING (AVG(?salary) > 50000)
    "#;

    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, .. } => {
            match pattern {
                Algebra::Group { having, .. } => {
                    assert!(having.is_some());
                    let conditions = having.unwrap();
                    assert_eq!(conditions.len(), 1);
                    // Verify condition is AVG(?salary) > 50000
                }
                _ => panic!("Expected GROUP algebra"),
            }
        }
        _ => panic!("Expected SELECT query"),
    }
}
```

#### Implementation Checklist

- [ ] Add `having: Option<Vec<Expression>>` to Algebra::Group
- [ ] Implement `parse_having_clause()`
- [ ] Update `parse_solution_modifier_with_group()` signature
- [ ] Add 5+ tests (simple, multiple conditions, aggregate refs)
- [ ] Verify executor filters results correctly

---

### GAP-12: VALUES Clause (Inline Data) [P1]

**File**: Grammar complete, parser incomplete
**Priority**: P1 - MEDIUM
**Effort**: 4 hours
**Complexity**: Medium

#### Solution Design

**Step 1**: Parse VALUES clause
```rust
fn parse_values_clause(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<Option<InlineData<'a>>>
{
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::DataBlock {
            return Ok(Some(self.parse_data_block(inner)?));
        }
    }
    Ok(None)
}

fn parse_data_block(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<InlineData<'a>>
{
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::InlineDataOneVar => {
                return self.parse_inline_data_one_var(inner);
            }
            Rule::InlineDataFull => {
                return self.parse_inline_data_full(inner);
            }
            _ => {}
        }
    }
    Err(ParseError::Syntax("Invalid data block".to_string()))
}

fn parse_inline_data_one_var(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<InlineData<'a>>
{
    let mut var = None;
    let mut values = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::Var => {
                var = Some(self.parse_variable(inner)?);
            }
            Rule::DataBlockValue => {
                values.push(self.parse_data_block_value(inner)?);
            }
            _ => {}
        }
    }

    let var = var.ok_or_else(|| ParseError::Syntax("VALUES missing variable".to_string()))?;

    Ok(InlineData::OneVar {
        var,
        values,
    })
}

fn parse_inline_data_full(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<InlineData<'a>>
{
    let mut vars = Vec::new();
    let mut rows = Vec::new();

    // First pass: collect variables
    let mut in_var_list = false;
    let mut current_row = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::Var => {
                if !in_var_list {
                    vars.push(self.parse_variable(inner)?);
                }
            }
            Rule::DataBlockValue => {
                current_row.push(self.parse_data_block_value(inner)?);
                if current_row.len() == vars.len() {
                    rows.push(current_row.clone());
                    current_row.clear();
                }
            }
            _ => {}
        }
    }

    Ok(InlineData::Full {
        vars,
        rows,
    })
}

fn parse_data_block_value(&mut self, pair: pest::iterators::Pair<'a, Rule>)
    -> ParseResult<Option<Node<'a>>>
{
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::iri => {
                return Ok(Some(self.parse_iri(inner)?));
            }
            Rule::RDFLiteral => {
                return Ok(Some(self.parse_rdf_literal(inner)?));
            }
            Rule::NumericLiteral => {
                return Ok(Some(self.parse_numeric_literal(inner)?));
            }
            Rule::BooleanLiteral => {
                return Ok(Some(self.parse_boolean_literal(inner)?));
            }
            _ if inner.as_str().eq_ignore_ascii_case("UNDEF") => {
                return Ok(None);  // UNDEF = unbound
            }
            _ => {}
        }
    }

    Err(ParseError::Syntax("Invalid data block value".to_string()))
}
```

**Step 2**: Add to algebra
```rust
// In algebra.rs
#[derive(Debug, Clone, PartialEq)]
pub enum InlineData<'a> {
    OneVar {
        var: Variable<'a>,
        values: Vec<Option<Node<'a>>>,
    },
    Full {
        vars: Vec<Variable<'a>>,
        rows: Vec<Vec<Option<Node<'a>>>>,
    },
}

pub enum Algebra<'a> {
    // ... existing variants ...
    Values {
        data: InlineData<'a>,
    },
}
```

**Step 3**: Integrate into query parsing
```rust
// Parse VALUES at end of query
let (input, values_clause) = self.parse_values_clause(pair)?;

if let Some(inline_data) = values_clause {
    // Combine with existing pattern via Join
    pattern = Algebra::Join {
        left: Box::new(pattern),
        right: Box::new(Algebra::Values { data: inline_data }),
    };
}
```

#### Testing Requirements

```rust
#[test]
fn test_values_one_var() {
    let query = r#"
        SELECT ?person ?name
        WHERE {
            ?person :name ?name .
            VALUES ?person { :Alice :Bob :Carol }
        }
    "#;

    let parsed = parser.parse_query(query).unwrap();

    match parsed {
        Query::Select { pattern, .. } => {
            // Verify pattern contains VALUES clause
            // Should be Join(BGP, Values)
        }
        _ => panic!("Expected SELECT query"),
    }
}

#[test]
fn test_values_multiple_vars() {
    let query = r#"
        SELECT ?x ?y
        WHERE {
            VALUES (?x ?y) {
                (:a :b)
                (:c :d)
                (UNDEF :e)
            }
        }
    "#;

    let parsed = parser.parse_query(query).unwrap();

    // Verify 3 rows with UNDEF handling
}
```

#### Implementation Checklist

- [ ] Add InlineData enum to algebra
- [ ] Implement parse_values_clause()
- [ ] Implement parse_data_block()
- [ ] Implement parse_inline_data_one_var()
- [ ] Implement parse_inline_data_full()
- [ ] Handle UNDEF (unbound variable)
- [ ] Add 8+ tests (one var, multi var, UNDEF, empty)
- [ ] Verify executor binds values correctly

---

## Part 3: Error Message Improvements [P1]

**Priority**: P1 - CRITICAL for developer adoption
**Effort**: 1-2 days
**Complexity**: Medium-High

### Current State

**Turtle Parser**:
```rust
.map_err(|e| ParseError::Syntax {
    line: 0,
    col: 0,  // NO LINE/COL TRACKING!
    message: format!("Parse error: {:?}", e),  // GENERIC MESSAGE
})
```

**Example Error**:
```
Error: Syntax { line: 0, col: 0, message: "Parse error: Error { input: \"PREFIX : <\", code: Tag }" }
```

**Issues**:
1. No line/column numbers
2. Shows internal nom error codes
3. No context about what was expected
4. No suggestions

### Solution Design

**Step 1**: Add position tracking
```rust
/// Calculate line and column from input position
fn calculate_position(full_input: &str, error_pos: &str) -> (usize, usize) {
    let offset = full_input.len() - error_pos.len();
    let before_error = &full_input[..offset];

    let line = before_error.chars().filter(|&c| c == '\n').count() + 1;
    let col = before_error.chars().rev()
        .take_while(|&c| c != '\n')
        .count() + 1;

    (line, col)
}
```

**Step 2**: Context-aware errors
```rust
fn create_parse_error(input: &str, error_input: &str, expected: &str) -> ParseError {
    let (line, col) = calculate_position(input, error_input);
    let context = &error_input[..error_input.len().min(50)];

    ParseError::Syntax {
        line,
        col,
        message: format!(
            "Expected {} at line {}:{}\nGot: '{}'",
            expected, line, col, context
        ),
    }
}
```

**Step 3**: Specific error messages
```rust
fn prefix_directive(input: &str) -> IResult<&str, Directive> {
    let full_input = input;  // Save for error reporting

    let (input, _) = tag("@prefix")(input)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(
            full_input,
            nom::error::ErrorKind::Tag
        )))?;

    let (input, _) = multispace1(input)
        .map_err(|_| {
            let (line, col) = calculate_position(full_input, input);
            nom::Err::Failure(nom::error::Error::new(
                full_input,
                nom::error::ErrorKind::Tag
            ))
            // Better: throw custom error with message "PREFIX directive requires space after @prefix"
        })?;

    let (input, prefix_ns) = pname_ns(input)
        .map_err(|_| {
            // "Expected prefix name (e.g., 'prefix:') after @prefix"
        })?;

    // ... etc
}
```

**Step 4**: Error recovery (advanced)
```rust
/// Try to suggest fix for common errors
fn suggest_fix(input: &str, context: &str) -> Option<String> {
    if context.starts_with("PREFIX") && !context.contains(':') {
        return Some("Did you mean 'PREFIX prefix: <uri>'? Missing ':' after prefix name.".to_string());
    }

    if context.starts_with('<') && !context.contains('>') {
        return Some("Unclosed IRI. Expected '>' to close IRI reference.".to_string());
    }

    if context.contains("<<") && !context.contains(">>") {
        return Some("Unclosed quoted triple. Expected '>>' to close quoted triple.".to_string());
    }

    None
}
```

#### Implementation Checklist

- [ ] Add position_tracker module
- [ ] Implement calculate_position()
- [ ] Create error_builder with context
- [ ] Add specific errors for each grammar rule
- [ ] Implement suggestion system
- [ ] Add error recovery (partial parsing)
- [ ] Create 20+ error test cases
- [ ] Document common errors in user guide

---

## Part 4: Performance Optimizations [P2]

### OPT-1: String Interning During Parse [P2]

**Current**: Parse → NodePattern → resolve → intern
**Optimized**: Parse → intern → Node

**Benefit**: 15-20% faster, single pass
**Trade-off**: Requires threading Arc<Dictionary> through combinators

**Complexity**: Medium (requires refactoring)

### OPT-2: Predicate-Object List Pre-allocation [P2]

**Current**:
```rust
let (input, objects) = separated_list0(...)(input)?;
// separated_list0 allocates Vec with grow-as-needed
```

**Optimized**:
```rust
// Heuristic: most property lists have 1-3 objects
let (input, objects) = {
    let mut vec = Vec::with_capacity(3);
    separated_list0_into(&mut vec, ...)(input)?
};
```

**Benefit**: 5-10% reduction in allocations
**Complexity**: Low

### OPT-3: Comment Pre-stripping [P2]

**Current**: Comments parsed by multispace0/multispace1
**Optimized**: Strip comments in preprocessing pass

```rust
fn strip_comments(input: &str) -> String {
    input.lines()
        .map(|line| {
            if let Some(pos) = line.find('#') {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

**Benefit**: 10-15% faster for heavily commented files
**Trade-off**: Breaks error position reporting
**Recommendation**: Optional flag for production mode

---

## Timeline Summary

### P0 Critical Fixes (Week 1)
- Day 1-2: GAP-1 Blank Node Expansion (2-3 hours) + GAP-2 Collection Expansion (4-5 hours)
- Day 3: GAP-7 GROUP BY Variables (3-4 hours) + Testing
- Day 4-5: GAP-9 Builtin Functions (2-3 days)

**Result**: Core RDF/SPARQL correctness achieved

### P1 Completeness (Week 2)
- Day 1: GAP-3 IRI Resolution (1h) + GAP-8 HAVING (2h) + GAP-12 VALUES (4h)
- Day 2-3: Error Message Quality (1-2 days)

**Result**: Production-grade usability

### P2 Nice-to-Have (Week 3)
- Day 1: GAP-4 Unicode Escapes + GAP-6 ECHAR Decoding
- Day 2-3: Performance Optimizations (OPT-1, OPT-2, OPT-3)
- Day 4-5: W3C Conformance Suite Integration

**Result**: Certification-ready

---

## Success Metrics

### Correctness
- [ ] All W3C RDF 1.2 Turtle tests passing (target: 100%)
- [ ] All W3C SPARQL 1.1 tests passing (target: 95%+)
- [ ] No known semantic bugs in triple expansion

### Performance
- [ ] Turtle parsing: 50-100 MB/s
- [ ] SPARQL parsing: 20-50 MB/s
- [ ] Memory usage: <100 MB for 1M triples

### Usability
- [ ] Error messages include line/column
- [ ] Suggestions for common mistakes
- [ ] Comprehensive documentation with examples

---

**End of Improvement Plan**
