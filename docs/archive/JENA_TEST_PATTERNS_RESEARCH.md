# Apache Jena Testing Patterns - Comprehensive Research for Rust RDF/SPARQL Implementation

**Date:** 2025-11-17
**Research Scope:** Apache Jena test patterns from jena-core, jena-arq, jena-shacl, and W3C test suites

---

## Table of Contents

1. [RDF Model Edge Cases](#1-rdf-model-edge-cases)
2. [Turtle Parser Tests](#2-turtle-parser-tests)
3. [SPARQL Parser & Query Tests](#3-sparql-parser--query-tests)
4. [SPARQL Update Tests](#4-sparql-update-tests)
5. [Reasoning Tests](#5-reasoning-tests)
6. [SHACL Validation Tests](#6-shacl-validation-tests)
7. [Integration & End-to-End Tests](#7-integration--end-to-end-tests)
8. [RDF-star Tests](#8-rdf-star-tests)

---

## 1. RDF Model Edge Cases

### 1.1 Blank Node Handling

**What it tests:** Blank node identity, skolemization, and serialization edge cases

**Why it's critical:**
- Blank nodes have complex identity semantics across serialization boundaries
- Skolemization must be reversible for proper round-tripping
- Cross-graph blank node references need special handling

**Test Patterns:**

#### Test 1.1.1: Blank Node Label Extraction
```turtle
# Input
_:b1 <http://example.org/prop> "value" .
<_:cba> <http://example.org/prop2> "value2" .

# Expected behavior
- Parser should distinguish between blank node labels (_:b1) and skolemized IRIs (<_:cba>)
- Both should be recognized as blank nodes internally
- getBlankNodeLabel() should return consistent identifiers
```

**Rust considerations:**
- Use enum for Node types: `IRI | BlankNode | Literal`
- Store blank node labels separately from IRIs
- Implement `Eq` and `Hash` carefully for blank nodes

#### Test 1.1.2: Blank Node Skolemization Round-Trip
```turtle
# Input graph 1
_:x <http://example.org/knows> _:y .

# Skolemize to
<urn:uuid:12345> <http://example.org/knows> <urn:uuid:67890> .

# Deserialize and verify
- Original graph isomorphism should be preserved
- Blank node IDs may differ but structure must match
```

**Rust considerations:**
- Implement graph isomorphism algorithm using canonical labeling
- Use UUIDs for skolemization: `use uuid::Uuid;`
- Consider using petgraph for graph algorithms

#### Test 1.1.3: Blank Node Identity Across Graphs
```sparql
# Test behavior in dataset with multiple named graphs
GRAPH <http://g1> { _:b1 <p> "o1" }
GRAPH <http://g2> { _:b1 <p> "o2" }

# Expected: _:b1 in g1 and g2 are DIFFERENT blank nodes
```

**Rust considerations:**
- Store graph context with each blank node: `(BlankNodeId, GraphName)`
- Use scoped blank node allocators per graph

#### Test 1.1.4: Blank Nodes as Subjects/Objects (Not Predicates)
```turtle
# Valid
_:b1 <http://ex.org/p> _:b2 .

# Invalid - must reject
_:b1 _:p <http://ex.org/o> .

# Expected: Parser error "Blank nodes cannot be predicates"
```

**Rust considerations:**
- Validate in parser using type system:
  ```rust
  struct Triple {
      subject: SubjectNode,    // IRI | BlankNode
      predicate: IRI,          // Only IRI allowed
      object: Node,            // IRI | BlankNode | Literal
  }
  ```

---

### 1.2 Literal Datatype Validation

**What it tests:** Datatype validation for numeric, boolean, date/time literals

**Why it's critical:**
- RDF requires lexical validation for typed literals
- Invalid literals must be accepted but flagged
- Type conversions (integer widening) affect storage and comparison

**Test Patterns:**

#### Test 1.2.1: Integer Type Widening
```turtle
# Input
<http://ex.org/s> <http://ex.org/age> "42"^^xsd:integer .
<http://ex.org/s> <http://ex.org/big> "9223372036854775807"^^xsd:integer .
<http://ex.org/s> <http://ex.org/huge> "99999999999999999999"^^xsd:integer .

# Expected behavior
- "42" -> i32 (or i64 depending on platform)
- "9223372036854775807" -> i64 (Long)
- "99999999999999999999" -> BigInt
- All should remain type xsd:integer semantically
```

**Rust considerations:**
```rust
enum IntegerValue {
    I32(i32),
    I64(i64),
    BigInt(num_bigint::BigInt),
}

impl IntegerValue {
    fn from_str(s: &str) -> Result<Self, ParseError> {
        // Try i32, then i64, then BigInt
    }
}
```

#### Test 1.2.2: Boolean Edge Cases
```turtle
# Valid
<s> <p> "true"^^xsd:boolean .
<s> <p> "false"^^xsd:boolean .
<s> <p> "1"^^xsd:boolean .
<s> <p> "0"^^xsd:boolean .

# Invalid but must accept with warning
<s> <p> "kind of?"^^xsd:boolean .

# Expected: Store as ill-typed literal, emit warning
```

**Rust considerations:**
```rust
struct TypedLiteral {
    lexical: String,
    datatype: IRI,
    is_well_formed: bool,
}

fn validate_boolean(lex: &str) -> bool {
    matches!(lex, "true" | "false" | "1" | "0")
}
```

#### Test 1.2.3: DateTime Validation
```turtle
# Valid
<s> <p> "2000-01-01T00:00:00"^^xsd:dateTime .
<s> <p> "2000-01-01T00:00:00Z"^^xsd:dateTime .
<s> <p> "2000-01-01T00:00:00-05:00"^^xsd:dateTime .

# Invalid lexical forms (must warn)
<s> <p> "2000-01-01"^^xsd:dateTime .              # Missing time
<s> <p> "2000-01-01-06:00"^^xsd:dateTime .        # Wrong format

# Expected: Accept but mark as ill-typed
```

**Rust considerations:**
```rust
use chrono::{DateTime, NaiveDate, NaiveDateTime};

fn validate_datetime(lex: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(lex)
        .map_err(|_| ParseError::InvalidDateTime)
}
```

#### Test 1.2.4: Numeric Subtype Constraints
```turtle
# Invalid - negative value for nonNegativeInteger
<s> <p> "-1"^^xsd:nonNegativeInteger .

# Invalid - zero for positiveInteger
<s> <p> "0"^^xsd:positiveInteger .

# Expected: Accept but mark as constraint violation
```

**Rust considerations:**
- Validate lexical form matches datatype constraints
- Store validation errors separately: `Vec<ValidationError>`

---

### 1.3 IRI Validation and Relative IRI Resolution

**What it tests:** IRI syntax validation, relative IRI resolution per RFC 3986

**Why it's critical:**
- IRIs must be absolute in RDF graphs
- Relative IRI resolution depends on base URI and context
- Invalid IRI characters must be detected

**Test Patterns:**

#### Test 1.3.1: Relative IRI Resolution with Base
```turtle
# Input with base
@base <http://example.org/data/> .
<resource1> <p> <resource2> .

# Expected output (absolute IRIs)
<http://example.org/data/resource1> <p> <http://example.org/data/resource2> .

# With trailing slash edge case
@base <http://example.org/data> .
<resource1> <p> <resource2> .

# Expected
<http://example.org/resource1> <p> <http://example.org/resource2> .
```

**Rust considerations:**
```rust
use url::{Url, ParseError};

fn resolve_iri(base: &Url, relative: &str) -> Result<Url, ParseError> {
    base.join(relative)
}

// Test trailing slash behavior
let base1 = Url::parse("http://ex.org/data/")?;  // With slash
let base2 = Url::parse("http://ex.org/data")?;   // Without slash
assert_eq!(base1.join("res")?, "http://ex.org/data/res");
assert_eq!(base2.join("res")?, "http://ex.org/res");
```

#### Test 1.3.2: Base URI Changes
```turtle
@base <http://example.org/base1/> .
<r1> <p> <o1> .

@base <http://example.org/base2/> .
<r2> <p> <o2> .

# Expected
<http://example.org/base1/r1> <p> <http://example.org/base1/o1> .
<http://example.org/base2/r2> <p> <http://example.org/base2/o2> .
```

**Rust considerations:**
- Maintain base URI stack or current state in parser
- Update base on `@base` directive

#### Test 1.3.3: Invalid IRI Characters (Negative Test)
```turtle
# Invalid - space in IRI
<http://example.org/invalid resource> <p> "o" .

# Invalid - unescaped special chars
<http://example.org/bad{character}> <p> "o" .

# Expected: Parse error with specific message
```

**Rust considerations:**
```rust
fn validate_iri(iri: &str) -> Result<(), IriError> {
    // Check for invalid characters
    if iri.contains(' ') {
        return Err(IriError::InvalidCharacter(' '));
    }
    // Use url crate for validation
    Url::parse(iri).map_err(|e| IriError::ParseError(e))?;
    Ok(())
}
```

---

### 1.4 Language Tags

**What it tests:** Language tag validation and case normalization per BCP 47

**Why it's critical:**
- Language tags are case-insensitive but should be normalized
- Comparison and matching require proper normalization

**Test Patterns:**

#### Test 1.4.1: Language Tag Normalization
```turtle
# Input
<s> <p> "hello"@en .
<s> <p> "bonjour"@FR .
<s> <p> "guten tag"@de-DE .
<s> <p> "hello"@EN-us .

# Expected normalization (lowercase primary, uppercase region)
"hello"@en
"bonjour"@fr
"guten tag"@de-DE
"hello"@en-US
```

**Rust considerations:**
```rust
fn normalize_langtag(tag: &str) -> String {
    let parts: Vec<&str> = tag.split('-').collect();
    if parts.len() == 1 {
        parts[0].to_lowercase()
    } else {
        format!("{}-{}", parts[0].to_lowercase(), parts[1].to_uppercase())
    }
}
```

#### Test 1.4.2: Language Tag Matching
```sparql
# Query with language filter
SELECT ?label WHERE {
    ?s rdfs:label ?label .
    FILTER(langMatches(lang(?label), "en"))
}

# Should match: @en, @en-US, @en-GB
# Should NOT match: @fr, @de
```

**Rust considerations:**
- Implement `lang()` and `langMatches()` SPARQL functions
- Use BCP 47 matching algorithm

---

## 2. Turtle Parser Tests

### 2.1 Prefix Handling

**What it tests:** PREFIX directive parsing, reuse, and scoping

**Why it's critical:**
- Prefixes enable compact notation
- Scope and redefinition edge cases
- Interaction with BASE directive

**Test Patterns:**

#### Test 2.1.1: Basic Prefix Usage
```turtle
@prefix ex: <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:Alice foaf:knows ex:Bob .

# Expected expansion
<http://example.org/Alice> <http://xmlns.com/foaf/0.1/knows> <http://example.org/Bob> .
```

#### Test 2.1.2: Prefix Redefinition
```turtle
@prefix ex: <http://example.org/v1/> .
ex:resource1 <p> "o1" .

@prefix ex: <http://example.org/v2/> .
ex:resource2 <p> "o2" .

# Expected (some parsers warn on redefinition)
<http://example.org/v1/resource1> <p> "o1" .
<http://example.org/v2/resource2> <p> "o2" .
```

**Rust considerations:**
```rust
struct PrefixMap {
    map: HashMap<String, String>,
}

impl PrefixMap {
    fn insert(&mut self, prefix: String, iri: String) -> Option<String> {
        // Optionally warn if prefix already exists
        let old = self.map.insert(prefix.clone(), iri.clone());
        if old.is_some() {
            eprintln!("Warning: Prefix '{}' redefined", prefix);
        }
        old
    }
}
```

#### Test 2.1.3: Empty Prefix
```turtle
@prefix : <http://example.org/default/> .

:Alice :knows :Bob .

# Expected
<http://example.org/default/Alice> <http://example.org/default/knows> <http://example.org/default/Bob> .
```

#### Test 2.1.4: Case-Sensitive Prefixes
```turtle
@prefix ex: <http://example.org/lower/> .
@prefix EX: <http://example.org/upper/> .

ex:resource1 <p> EX:resource2 .

# Expected - different prefixes
<http://example.org/lower/resource1> <p> <http://example.org/upper/resource2> .
```

---

### 2.2 Literal Syntax Edge Cases

**What it tests:** String escaping, multi-line literals, numeric literals

**Why it's critical:**
- Complex escape sequences in strings
- Multi-line string handling
- Numeric literal formats (decimals, doubles, integers)

**Test Patterns:**

#### Test 2.2.1: String Escape Sequences
```turtle
<s> <p> "Line 1\nLine 2" .
<s> <p> "Tab\there" .
<s> <p> "Quote: \"hello\"" .
<s> <p> "Backslash: \\" .
<s> <p> "Unicode: \u0041\u0042" .  # AB
<s> <p> "Unicode long: \U00000041" .  # A

# Expected unescaped values
"Line 1
Line 2"
"Tab	here"
"Quote: "hello""
"Backslash: \"
"Unicode: AB"
"Unicode long: A"
```

**Rust considerations:**
```rust
fn unescape_string(s: &str) -> Result<String, EscapeError> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('u') => {
                    // Parse \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    let code = u32::from_str_radix(&hex, 16)?;
                    result.push(char::from_u32(code).ok_or(EscapeError::InvalidCodePoint)?);
                }
                _ => return Err(EscapeError::InvalidEscape),
            }
        } else {
            result.push(c);
        }
    }
    Ok(result)
}
```

#### Test 2.2.2: Multi-Line Literals (Long Strings)
```turtle
<s> <p> """This is a
multi-line
string""" .

<s> <p> '''Also
multi-line
with single quotes''' .

# Expected: Preserve line breaks
"This is a
multi-line
string"
```

#### Test 2.2.3: Numeric Literal Formats
```turtle
<s> <p> 42 .                    # xsd:integer
<s> <p> 3.14 .                  # xsd:decimal
<s> <p> 1.23e-4 .               # xsd:double
<s> <p> +42 .                   # xsd:integer (positive)
<s> <p> -42 .                   # xsd:integer (negative)
<s> <p> .5 .                    # xsd:decimal (leading dot)

# Expected datatypes
"42"^^xsd:integer
"3.14"^^xsd:decimal
"1.23e-4"^^xsd:double
"42"^^xsd:integer
"-42"^^xsd:integer
"0.5"^^xsd:decimal
```

**Rust considerations:**
```rust
fn parse_numeric_literal(s: &str) -> TypedLiteral {
    if s.contains('e') || s.contains('E') {
        TypedLiteral::new(s, XSD_DOUBLE)
    } else if s.contains('.') {
        TypedLiteral::new(s, XSD_DECIMAL)
    } else {
        TypedLiteral::new(s, XSD_INTEGER)
    }
}
```

---

### 2.3 Collection Syntax

**What it tests:** RDF collections (lists) using parentheses

**Why it's critical:**
- Collections are syntactic sugar for rdf:List structures
- Empty collections, nested collections
- Proper blank node generation

**Test Patterns:**

#### Test 2.3.1: Simple Collection
```turtle
<s> <p> (1 2 3) .

# Expected expansion
<s> <p> _:b1 .
_:b1 rdf:first "1"^^xsd:integer .
_:b1 rdf:rest _:b2 .
_:b2 rdf:first "2"^^xsd:integer .
_:b2 rdf:rest _:b3 .
_:b3 rdf:first "3"^^xsd:integer .
_:b3 rdf:rest rdf:nil .
```

#### Test 2.3.2: Empty Collection
```turtle
<s> <p> () .

# Expected
<s> <p> rdf:nil .
```

#### Test 2.3.3: Nested Collections
```turtle
<s> <p> (1 (2 3) 4) .

# Expected expansion (nested blank nodes)
<s> <p> _:b1 .
_:b1 rdf:first "1"^^xsd:integer .
_:b1 rdf:rest _:b2 .
_:b2 rdf:first _:b3 .  # Nested collection
_:b3 rdf:first "2"^^xsd:integer .
_:b3 rdf:rest _:b4 .
_:b4 rdf:first "3"^^xsd:integer .
_:b4 rdf:rest rdf:nil .
_:b2 rdf:rest _:b5 .
_:b5 rdf:first "4"^^xsd:integer .
_:b5 rdf:rest rdf:nil .
```

**Rust considerations:**
```rust
fn parse_collection(items: Vec<Node>) -> BlankNode {
    if items.is_empty() {
        return RDF_NIL;
    }

    let mut current = allocate_blank_node();
    let head = current.clone();

    for (i, item) in items.iter().enumerate() {
        emit_triple(current, RDF_FIRST, item.clone());
        if i < items.len() - 1 {
            let next = allocate_blank_node();
            emit_triple(current, RDF_REST, next.clone());
            current = next;
        } else {
            emit_triple(current, RDF_REST, RDF_NIL);
        }
    }

    head
}
```

---

### 2.4 Blank Node Property Lists

**What it tests:** Square bracket syntax for blank nodes with properties

**Why it's critical:**
- Compact syntax for blank nodes
- Nesting and complex structures
- Blank node allocation

**Test Patterns:**

#### Test 2.4.1: Simple Blank Node Property List
```turtle
<Alice> foaf:knows [ foaf:name "Bob" ; foaf:age 30 ] .

# Expected expansion
<Alice> foaf:knows _:b1 .
_:b1 foaf:name "Bob" .
_:b1 foaf:age "30"^^xsd:integer .
```

#### Test 2.4.2: Nested Blank Node Property Lists
```turtle
<Alice> foaf:knows [
    foaf:name "Bob" ;
    foaf:knows [
        foaf:name "Charlie"
    ]
] .

# Expected expansion
<Alice> foaf:knows _:b1 .
_:b1 foaf:name "Bob" .
_:b1 foaf:knows _:b2 .
_:b2 foaf:name "Charlie" .
```

#### Test 2.4.3: Blank Node as Subject
```turtle
[ foaf:name "Anonymous" ] foaf:knows <Bob> .

# Expected
_:b1 foaf:name "Anonymous" .
_:b1 foaf:knows <Bob> .
```

---

### 2.5 Negative Syntax Tests

**What it tests:** Parser error handling for invalid Turtle syntax

**Why it's critical:**
- Robust error reporting
- Security (prevent parser exploits)
- Spec compliance

**Test Patterns:**

#### Test 2.5.1: Missing Dot Terminator
```turtle
<s> <p> <o>
<s2> <p2> <o2> .

# Expected: Parse error "Expected '.' after triple"
```

#### Test 2.5.2: Invalid Prefix Name
```turtle
@prefix 123: <http://example.org/> .

# Expected: Parse error "Prefix name must start with letter"
```

#### Test 2.5.3: Unterminated String Literal
```turtle
<s> <p> "unterminated .

# Expected: Parse error "Unterminated string literal"
```

#### Test 2.5.4: Invalid Escape Sequence
```turtle
<s> <p> "invalid\xescape" .

# Expected: Parse error "Invalid escape sequence \x"
```

**Rust considerations:**
```rust
#[derive(Debug)]
enum ParseError {
    UnexpectedEof,
    InvalidEscape(char),
    UnterminatedString,
    MissingDot { line: usize, col: usize },
    InvalidPrefixName(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::UnterminatedString =>
                write!(f, "Unterminated string literal"),
            ParseError::MissingDot { line, col } =>
                write!(f, "Expected '.' at line {}, column {}", line, col),
            // ...
        }
    }
}
```

---

## 3. SPARQL Parser & Query Tests

### 3.1 Basic Query Patterns

**What it tests:** Triple patterns, variable binding, result sets

**Why it's critical:**
- Foundation for all SPARQL queries
- Variable scoping and binding
- Result set correctness

**Test Patterns:**

#### Test 3.1.1: Simple SELECT Query
```sparql
SELECT ?s ?p ?o
WHERE {
    ?s ?p ?o .
}

# Test data
<Alice> <knows> <Bob> .
<Bob> <knows> <Charlie> .

# Expected results
| s       | p      | o         |
|---------|--------|-----------|
| <Alice> | <knows>| <Bob>     |
| <Bob>   | <knows>| <Charlie> |
```

#### Test 3.1.2: Variable Reuse and Constraints
```sparql
SELECT ?person1 ?person2
WHERE {
    ?person1 <knows> ?person2 .
    ?person2 <knows> ?person1 .
}

# Test data
<Alice> <knows> <Bob> .
<Bob> <knows> <Alice> .
<Charlie> <knows> <Alice> .

# Expected (mutual knows only)
| person1 | person2 |
|---------|---------|
| <Alice> | <Bob>   |
| <Bob>   | <Alice> |
```

#### Test 3.1.3: ASK Query
```sparql
ASK {
    <Alice> <knows> <Bob> .
}

# Expected: true (boolean)

ASK {
    <Alice> <knows> <Unknown> .
}

# Expected: false
```

#### Test 3.1.4: CONSTRUCT Query
```sparql
CONSTRUCT {
    ?p1 <isFriendOf> ?p2 .
}
WHERE {
    ?p1 <knows> ?p2 .
}

# Test data
<Alice> <knows> <Bob> .

# Expected output RDF
<Alice> <isFriendOf> <Bob> .
```

---

### 3.2 FILTER Clauses

**What it tests:** Filtering results with expressions

**Why it's critical:**
- Complex expression evaluation
- Type coercion and error handling
- Performance optimization opportunities

**Test Patterns:**

#### Test 3.2.1: Numeric Filters
```sparql
SELECT ?person ?age
WHERE {
    ?person <hasAge> ?age .
    FILTER(?age > 25)
}

# Test data
<Alice> <hasAge> "30"^^xsd:integer .
<Bob> <hasAge> "20"^^xsd:integer .
<Charlie> <hasAge> "25"^^xsd:integer .

# Expected (> not >=)
| person    | age |
|-----------|-----|
| <Alice>   | 30  |
```

#### Test 3.2.2: String Filters with Regex
```sparql
SELECT ?name
WHERE {
    ?person <name> ?name .
    FILTER(regex(?name, "^A"))
}

# Test data
<p1> <name> "Alice" .
<p2> <name> "Bob" .
<p3> <name> "Andrew" .

# Expected (names starting with A)
| name    |
|---------|
| "Alice" |
| "Andrew"|
```

#### Test 3.2.3: Language Tag Filters
```sparql
SELECT ?label
WHERE {
    ?s rdfs:label ?label .
    FILTER(langMatches(lang(?label), "en"))
}

# Test data
<s1> rdfs:label "Hello"@en .
<s2> rdfs:label "Bonjour"@fr .
<s3> rdfs:label "Hi"@en-US .

# Expected (en and en-US)
| label       |
|-------------|
| "Hello"@en  |
| "Hi"@en-US  |
```

#### Test 3.2.4: Bound/Unbound Filters
```sparql
SELECT ?s ?o
WHERE {
    ?s <p> ?o .
    FILTER(!bound(?o))
}

# This is unusual but tests the filter - typically used with OPTIONAL
```

---

### 3.3 OPTIONAL Patterns

**What it tests:** Optional pattern matching

**Why it's critical:**
- Left-join semantics
- Interaction with FILTER
- Nested OPTIONAL behavior

**Test Patterns:**

#### Test 3.3.1: Basic OPTIONAL
```sparql
SELECT ?person ?email
WHERE {
    ?person <name> ?name .
    OPTIONAL { ?person <email> ?email }
}

# Test data
<Alice> <name> "Alice" .
<Alice> <email> "alice@example.org" .
<Bob> <name> "Bob" .

# Expected (Bob has no email, so unbound)
| person  | email                |
|---------|----------------------|
| <Alice> | "alice@example.org"  |
| <Bob>   | (unbound)            |
```

#### Test 3.3.2: OPTIONAL with FILTER
```sparql
SELECT ?person ?age
WHERE {
    ?person <name> ?name .
    OPTIONAL {
        ?person <age> ?age .
        FILTER(?age > 25)
    }
}

# Test data
<Alice> <name> "Alice" .
<Alice> <age> "30"^^xsd:integer .
<Bob> <name> "Bob" .
<Bob> <age> "20"^^xsd:integer .
<Charlie> <name> "Charlie" .

# Expected
| person    | age |
|-----------|-----|
| <Alice>   | 30  |
| <Bob>     | (unbound) |
| <Charlie> | (unbound) |
```

**Critical:** FILTER inside OPTIONAL only filters when pattern matches!

#### Test 3.3.3: FILTER After OPTIONAL (Different Semantics)
```sparql
SELECT ?person ?age
WHERE {
    ?person <name> ?name .
    OPTIONAL { ?person <age> ?age }
    FILTER(!bound(?age) || ?age > 25)
}

# Same test data as above

# Expected (Bob filtered out, Charlie included)
| person    | age |
|-----------|-----|
| <Alice>   | 30  |
| <Charlie> | (unbound) |
```

#### Test 3.3.4: Nested OPTIONAL
```sparql
SELECT ?person ?email ?phone
WHERE {
    ?person <name> ?name .
    OPTIONAL {
        ?person <email> ?email .
        OPTIONAL { ?person <phone> ?phone }
    }
}

# Test all combinations: neither, email only, both, phone only (impossible)
```

---

### 3.4 UNION Patterns

**What it tests:** Alternative patterns (logical OR)

**Why it's critical:**
- Disjunctive queries
- Variable scoping across UNION branches
- Result deduplication

**Test Patterns:**

#### Test 3.4.1: Basic UNION
```sparql
SELECT ?person
WHERE {
    { ?person <knows> <Alice> }
    UNION
    { ?person <worksFor> <CompanyX> }
}

# Test data
<Bob> <knows> <Alice> .
<Charlie> <worksFor> <CompanyX> .
<Dave> <knows> <Alice> .
<Dave> <worksFor> <CompanyX> .

# Expected (no duplicates if same binding)
| person    |
|-----------|
| <Bob>     |
| <Charlie> |
| <Dave>    |
```

#### Test 3.4.2: UNION with Different Variables
```sparql
SELECT ?name
WHERE {
    { ?person <firstName> ?name }
    UNION
    { ?company <companyName> ?name }
}

# Both person names and company names in ?name
```

---

### 3.5 Property Paths

**What it tests:** Complex path expressions for graph navigation

**Why it's critical:**
- Transitive queries without recursion
- Alternative paths
- Negated property sets

**Test Patterns:**

#### Test 3.5.1: Transitive Closure (Zero or More)
```sparql
SELECT ?ancestor ?descendant
WHERE {
    ?ancestor <hasChild>* ?descendant .
}

# Test data (family tree)
<Alice> <hasChild> <Bob> .
<Bob> <hasChild> <Charlie> .

# Expected (includes reflexive)
| ancestor | descendant |
|----------|------------|
| <Alice>  | <Alice>    |  # Zero steps
| <Alice>  | <Bob>      |  # One step
| <Alice>  | <Charlie>  |  # Two steps
| <Bob>    | <Bob>      |
| <Bob>    | <Charlie>  |
| <Charlie>| <Charlie>  |
```

#### Test 3.5.2: Transitive (One or More)
```sparql
SELECT ?ancestor ?descendant
WHERE {
    ?ancestor <hasChild>+ ?descendant .
}

# Same data as above

# Expected (excludes reflexive)
| ancestor | descendant |
|----------|------------|
| <Alice>  | <Bob>      |
| <Alice>  | <Charlie>  |
| <Bob>    | <Charlie>  |
```

#### Test 3.5.3: Alternative Paths
```sparql
SELECT ?s ?o
WHERE {
    ?s <knows>|<worksFor> ?o .
}

# Matches either knows or worksFor predicates
```

#### Test 3.5.4: Inverse Path
```sparql
SELECT ?child ?parent
WHERE {
    ?child ^<hasChild> ?parent .
}

# Equivalent to: ?parent <hasChild> ?child .
```

#### Test 3.5.5: Sequence Path
```sparql
SELECT ?person ?grandchild
WHERE {
    ?person <hasChild>/<hasChild> ?grandchild .
}

# Two hasChild steps in sequence
```

#### Test 3.5.6: Negated Property Set
```sparql
SELECT ?s ?o
WHERE {
    ?s !<knows> ?o .
}

# Any property except knows
```

**Rust considerations:**
- Implement path evaluation using graph traversal algorithms
- Use BFS for shortest paths, DFS for exhaustive search
- Cache intermediate results for `*` and `+` operators

---

### 3.6 Aggregation and GROUP BY

**What it tests:** Aggregate functions, grouping, HAVING filters

**Why it's critical:**
- Statistical queries
- Edge cases with empty groups
- DISTINCT within aggregates

**Test Patterns:**

#### Test 3.6.1: COUNT Aggregation
```sparql
SELECT (COUNT(?friend) AS ?friendCount)
WHERE {
    <Alice> <knows> ?friend .
}

# Test data
<Alice> <knows> <Bob> .
<Alice> <knows> <Charlie> .

# Expected
| friendCount |
|-------------|
| 2           |
```

#### Test 3.6.2: COUNT with No Matches (Critical Edge Case)
```sparql
SELECT (COUNT(*) AS ?count)
WHERE {
    ?s ?p ?o .
    FILTER(false)
}

# Empty result set

# Expected per SPARQL spec: ONE row with count = 0
| count |
|-------|
| 0     |
```

**Critical:** This differs from SQL! Empty group still produces one row.

#### Test 3.6.3: GROUP BY with COUNT
```sparql
SELECT ?person (COUNT(?friend) AS ?friendCount)
WHERE {
    ?person <knows> ?friend .
}
GROUP BY ?person

# Test data
<Alice> <knows> <Bob> .
<Alice> <knows> <Charlie> .
<Bob> <knows> <Alice> .

# Expected
| person  | friendCount |
|---------|-------------|
| <Alice> | 2           |
| <Bob>   | 1           |
```

#### Test 3.6.4: GROUP BY with HAVING
```sparql
SELECT ?person (COUNT(?friend) AS ?friendCount)
WHERE {
    ?person <knows> ?friend .
}
GROUP BY ?person
HAVING (COUNT(?friend) > 1)

# Same data as above

# Expected (only Alice)
| person  | friendCount |
|---------|-------------|
| <Alice> | 2           |
```

#### Test 3.6.5: Multiple Aggregates
```sparql
SELECT ?dept (AVG(?salary) AS ?avgSalary) (MAX(?salary) AS ?maxSalary)
WHERE {
    ?person <department> ?dept .
    ?person <salary> ?salary .
}
GROUP BY ?dept
```

#### Test 3.6.6: COUNT DISTINCT
```sparql
SELECT (COUNT(DISTINCT ?type) AS ?typeCount)
WHERE {
    ?s rdf:type ?type .
}

# Test data
<a> rdf:type <Person> .
<b> rdf:type <Person> .
<c> rdf:type <Organization> .

# Expected (2 distinct types)
| typeCount |
|-----------|
| 2         |
```

**Rust considerations:**
```rust
enum Aggregate {
    Count { expr: Option<Expression>, distinct: bool },
    Sum { expr: Expression, distinct: bool },
    Avg { expr: Expression, distinct: bool },
    Min { expr: Expression },
    Max { expr: Expression },
}

struct GroupKey(Vec<Value>);

fn evaluate_aggregates(
    solutions: Vec<Solution>,
    group_by: Vec<Variable>,
    aggregates: Vec<(Variable, Aggregate)>,
) -> Vec<Solution> {
    // Group solutions by group_by variables
    // Evaluate each aggregate per group
}
```

---

### 3.7 Solution Modifiers

**What it tests:** ORDER BY, LIMIT, OFFSET, DISTINCT

**Why it's critical:**
- Query result ordering and pagination
- Deduplication semantics
- Interaction with aggregation

**Test Patterns:**

#### Test 3.7.1: ORDER BY
```sparql
SELECT ?person ?age
WHERE {
    ?person <age> ?age .
}
ORDER BY DESC(?age)

# Test data
<Alice> <age> "30"^^xsd:integer .
<Bob> <age> "25"^^xsd:integer .
<Charlie> <age> "35"^^xsd:integer .

# Expected (descending order)
| person    | age |
|-----------|-----|
| <Charlie> | 35  |
| <Alice>   | 30  |
| <Bob>     | 25  |
```

#### Test 3.7.2: LIMIT and OFFSET
```sparql
SELECT ?person ?age
WHERE {
    ?person <age> ?age .
}
ORDER BY ?age
LIMIT 2
OFFSET 1

# Same data as above

# Expected (skip first, take next 2)
| person  | age |
|---------|-----|
| <Alice> | 30  |
| <Charlie>| 35  |
```

#### Test 3.7.3: DISTINCT
```sparql
SELECT DISTINCT ?type
WHERE {
    ?s rdf:type ?type .
}

# Test data
<a> rdf:type <Person> .
<b> rdf:type <Person> .
<c> rdf:type <Organization> .

# Expected (2 distinct types)
| type            |
|-----------------|
| <Person>        |
| <Organization>  |
```

---

### 3.8 BIND and VALUES

**What it tests:** Variable binding and inline data

**Why it's critical:**
- Dynamic value assignment
- Inline data injection
- Interaction with other patterns

**Test Patterns:**

#### Test 3.8.1: BIND Expression
```sparql
SELECT ?person ?firstName ?lastName ?fullName
WHERE {
    ?person <firstName> ?firstName .
    ?person <lastName> ?lastName .
    BIND(CONCAT(?firstName, " ", ?lastName) AS ?fullName)
}

# Test data
<Alice> <firstName> "Alice" .
<Alice> <lastName> "Smith" .

# Expected
| person  | firstName | lastName | fullName      |
|---------|-----------|----------|---------------|
| <Alice> | "Alice"   | "Smith"  | "Alice Smith" |
```

#### Test 3.8.2: VALUES Clause
```sparql
SELECT ?person ?age
WHERE {
    VALUES ?person { <Alice> <Bob> }
    ?person <age> ?age .
}

# Test data
<Alice> <age> "30"^^xsd:integer .
<Bob> <age> "25"^^xsd:integer .
<Charlie> <age> "35"^^xsd:integer .

# Expected (only Alice and Bob)
| person  | age |
|---------|-----|
| <Alice> | 30  |
| <Bob>   | 25  |
```

#### Test 3.8.3: VALUES with Multiple Variables
```sparql
SELECT ?name ?category
WHERE {
    VALUES (?name ?category) {
        ("Alice" "Student")
        ("Bob" "Professor")
    }
}

# Expected
| name    | category     |
|---------|--------------|
| "Alice" | "Student"    |
| "Bob"   | "Professor"  |
```

---

### 3.9 Subqueries

**What it tests:** Nested SELECT queries

**Why it's critical:**
- Complex query composition
- Variable scoping
- Performance optimization

**Test Patterns:**

#### Test 3.9.1: Basic Subquery
```sparql
SELECT ?person
WHERE {
    ?person <age> ?age .
    {
        SELECT (AVG(?age) AS ?avgAge)
        WHERE {
            ?p <age> ?age .
        }
    }
    FILTER(?age > ?avgAge)
}

# Find people older than average
```

#### Test 3.9.2: Subquery with LIMIT
```sparql
SELECT ?person ?friend
WHERE {
    ?person <name> ?name .
    {
        SELECT ?friend
        WHERE {
            ?friend <score> ?score .
        }
        ORDER BY DESC(?score)
        LIMIT 10
    }
    ?person <knows> ?friend .
}

# Find connections to top 10 high-scorers
```

---

### 3.10 Negation (MINUS, NOT EXISTS, FILTER NOT EXISTS)

**What it tests:** Negative patterns

**Why it's critical:**
- Set difference semantics
- Subtle differences between negation methods
- Performance implications

**Test Patterns:**

#### Test 3.10.1: NOT EXISTS
```sparql
SELECT ?person
WHERE {
    ?person <name> ?name .
    FILTER NOT EXISTS { ?person <email> ?email }
}

# Find people without email addresses
```

#### Test 3.10.2: MINUS
```sparql
SELECT ?person
WHERE {
    ?person <knows> ?friend .
    MINUS { ?friend <knows> ?person }
}

# Find one-way friendships (knows someone who doesn't know them back)
```

**Critical difference:**
- `NOT EXISTS` tests for pattern existence
- `MINUS` performs set difference on solution mappings

#### Test 3.10.3: Comparison of Negation Methods
```sparql
# NOT EXISTS
SELECT ?s WHERE {
    ?s ?p ?o .
    FILTER NOT EXISTS { ?s <type> ?t }
}

# MINUS
SELECT ?s WHERE {
    ?s ?p ?o .
    MINUS { ?s <type> ?t }
}

# These can give DIFFERENT results when ?s is projected!
```

---

## 4. SPARQL Update Tests

### 4.1 INSERT DATA

**What it tests:** Adding triples without pattern matching

**Why it's critical:**
- Basic graph modification
- Named graph handling
- Blank node insertion

**Test Patterns:**

#### Test 4.1.1: Insert into Default Graph
```sparql
INSERT DATA {
    <Alice> <knows> <Bob> .
    <Bob> <knows> <Charlie> .
}

# Expected: 2 triples added to default graph
```

#### Test 4.1.2: Insert into Named Graph
```sparql
INSERT DATA {
    GRAPH <http://example.org/g1> {
        <Alice> <knows> <Bob> .
    }
}

# Expected: Triple added to named graph g1
```

#### Test 4.1.3: Insert with Blank Nodes (Edge Case)
```sparql
INSERT DATA {
    _:b1 <name> "Anonymous" .
}

# Expected: New blank node created
# NOTE: Cannot reference _:b1 later!
```

---

### 4.2 DELETE DATA

**What it tests:** Removing specific triples

**Why it's critical:**
- Exact triple matching required
- Named graph handling
- No variable substitution

**Test Patterns:**

#### Test 4.2.1: Delete from Default Graph
```sparql
DELETE DATA {
    <Alice> <knows> <Bob> .
}

# Expected: Removes that exact triple if it exists
```

#### Test 4.2.2: Delete from Named Graph (Critical)
```sparql
# WRONG - won't delete from named graph!
DELETE DATA {
    <Alice> <knows> <Bob> .
}

# CORRECT - must specify graph
DELETE DATA {
    GRAPH <http://example.org/g1> {
        <Alice> <knows> <Bob> .
    }
}
```

---

### 4.3 INSERT/DELETE WHERE

**What it tests:** Pattern-based updates

**Why it's critical:**
- Dynamic updates based on query results
- Variable substitution
- Interaction with OPTIONAL, FILTER

**Test Patterns:**

#### Test 4.3.1: DELETE WHERE
```sparql
DELETE WHERE {
    ?person <age> ?age .
    FILTER(?age < 18)
}

# Deletes all age triples where age < 18
```

#### Test 4.3.2: INSERT WHERE
```sparql
INSERT {
    ?person <category> "Adult" .
}
WHERE {
    ?person <age> ?age .
    FILTER(?age >= 18)
}

# Adds category "Adult" for all adults
```

#### Test 4.3.3: DELETE/INSERT (Modify)
```sparql
DELETE {
    ?person <age> ?oldAge .
}
INSERT {
    ?person <age> ?newAge .
}
WHERE {
    ?person <age> ?oldAge .
    BIND(?oldAge + 1 AS ?newAge)
}

# Increment everyone's age by 1
```

#### Test 4.3.4: WITH Clause for Named Graphs
```sparql
WITH <http://example.org/g1>
DELETE { ?s <deprecated> ?o }
INSERT { ?s <status> "active" }
WHERE {
    ?s <deprecated> ?o .
}

# Operate on named graph g1
```

---

### 4.4 Graph Management

**What it tests:** CREATE, DROP, CLEAR, COPY, MOVE, ADD

**Why it's critical:**
- Graph lifecycle management
- Atomic operations
- Silent vs. non-silent variants

**Test Patterns:**

#### Test 4.4.1: CREATE GRAPH
```sparql
CREATE GRAPH <http://example.org/new>

# Expected: New empty named graph created
# If graph exists: ERROR (unless SILENT)

CREATE SILENT GRAPH <http://example.org/new>

# Expected: No error if graph exists
```

#### Test 4.4.2: DROP GRAPH
```sparql
DROP GRAPH <http://example.org/g1>

# Expected: Graph deleted with all triples
# If graph doesn't exist: ERROR (unless SILENT)
```

#### Test 4.4.3: CLEAR GRAPH
```sparql
CLEAR GRAPH <http://example.org/g1>

# Expected: All triples removed, graph still exists
# Different from DROP!
```

#### Test 4.4.4: COPY, MOVE, ADD
```sparql
# COPY: Replaces destination with source content
COPY <http://example.org/source> TO <http://example.org/dest>

# MOVE: Copy then delete source
MOVE <http://example.org/source> TO <http://example.org/dest>

# ADD: Merge source into destination (no deletion)
ADD <http://example.org/source> TO <http://example.org/dest>
```

---

## 5. Reasoning Tests

### 5.1 RDFS Entailment

**What it tests:** RDFS inference rules

**Why it's critical:**
- Subclass and subproperty reasoning
- Domain and range inference
- Type propagation

**Test Patterns:**

#### Test 5.1.1: rdfs:subClassOf Transitivity
```turtle
# Input data
:Dog rdfs:subClassOf :Mammal .
:Mammal rdfs:subClassOf :Animal .
:Fido rdf:type :Dog .

# Expected inferred triples
:Fido rdf:type :Mammal .
:Fido rdf:type :Animal .
:Dog rdfs:subClassOf :Animal .  # Transitive closure
```

#### Test 5.1.2: rdfs:subPropertyOf Transitivity
```turtle
# Input
:owns rdfs:subPropertyOf :controls .
:controls rdfs:subPropertyOf :hasRelationWith .
:Alice :owns :Car .

# Expected inferred
:Alice :controls :Car .
:Alice :hasRelationWith :Car .
```

#### Test 5.1.3: rdfs:domain Inference
```turtle
# Input
:knows rdfs:domain :Person .
:Alice :knows :Bob .

# Expected inferred
:Alice rdf:type :Person .
```

#### Test 5.1.4: rdfs:range Inference
```turtle
# Input
:hasFather rdfs:range :Male .
:Alice :hasFather :Bob .

# Expected inferred
:Bob rdf:type :Male .
```

**Rust considerations:**
```rust
struct RDFSReasoner {
    rules: Vec<InferenceRule>,
}

impl RDFSReasoner {
    fn apply_rules(&self, graph: &mut Graph) -> usize {
        let mut inferred_count = 0;
        loop {
            let new_triples = self.apply_one_iteration(graph);
            if new_triples == 0 {
                break;
            }
            inferred_count += new_triples;
        }
        inferred_count
    }

    fn apply_subclass_transitivity(&self, graph: &mut Graph) -> Vec<Triple> {
        // Find all (A subClassOf B) and (B subClassOf C)
        // Infer (A subClassOf C)
    }
}
```

---

### 5.2 OWL Reasoning (Basic)

**What it tests:** OWL-Lite/DL inference

**Why it's critical:**
- Transitive and symmetric properties
- Inverse properties
- Functional properties

**Test Patterns:**

#### Test 5.2.1: owl:TransitiveProperty
```turtle
# Input
:ancestorOf rdf:type owl:TransitiveProperty .
:Alice :ancestorOf :Bob .
:Bob :ancestorOf :Charlie .

# Expected inferred
:Alice :ancestorOf :Charlie .
```

#### Test 5.2.2: owl:SymmetricProperty
```turtle
# Input
:married rdf:type owl:SymmetricProperty .
:Alice :married :Bob .

# Expected inferred
:Bob :married :Alice .
```

#### Test 5.2.3: owl:inverseOf
```turtle
# Input
:hasChild owl:inverseOf :hasParent .
:Alice :hasChild :Bob .

# Expected inferred
:Bob :hasParent :Alice .
```

#### Test 5.2.4: owl:FunctionalProperty (Consistency Check)
```turtle
# Input
:hasBirthdate rdf:type owl:FunctionalProperty .
:Alice :hasBirthdate "1990-01-01"^^xsd:date .
:Alice :hasBirthdate "1991-01-01"^^xsd:date .

# Expected: Inconsistency detected!
# (Functional property can have only one value)
```

**Rust considerations:**
```rust
enum OwlProperty {
    Transitive,
    Symmetric,
    Functional,
    InverseFunctional,
}

struct ConsistencyViolation {
    property: IRI,
    subject: Node,
    values: Vec<Node>,
    constraint: OwlProperty,
}

fn check_functional_property(
    graph: &Graph,
    prop: &IRI,
) -> Vec<ConsistencyViolation> {
    // Find subjects with multiple values for functional property
}
```

---

## 6. SHACL Validation Tests

### 6.1 Node Shapes

**What it tests:** SHACL shape validation against RDF data

**Why it's critical:**
- Data quality validation
- Constraint checking
- Violation reporting

**Test Patterns:**

#### Test 6.1.1: sh:targetClass with Property Constraints
```turtle
# Shape definition
:PersonShape a sh:NodeShape ;
    sh:targetClass :Person ;
    sh:property [
        sh:path :name ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
    ] .

# Valid data
:Alice rdf:type :Person ;
    :name "Alice" .

# Invalid data (violates minCount)
:Bob rdf:type :Person .

# Expected: Validation report with violation for :Bob
```

#### Test 6.1.2: sh:minLength and sh:maxLength
```turtle
# Shape
:UsernameShape a sh:NodeShape ;
    sh:property [
        sh:path :username ;
        sh:minLength 3 ;
        sh:maxLength 20 ;
    ] .

# Valid
:User1 :username "alice" .

# Invalid (too short)
:User2 :username "ab" .

# Expected: Violation for :User2
```

#### Test 6.1.3: sh:pattern (Regex Validation)
```turtle
# Shape
:EmailShape a sh:NodeShape ;
    sh:property [
        sh:path :email ;
        sh:pattern "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$" ;
    ] .

# Valid
:User1 :email "alice@example.org" .

# Invalid
:User2 :email "invalid-email" .

# Expected: Violation for :User2
```

---

### 6.2 Property Shapes

**What it tests:** SHACL property constraints

**Test Patterns:**

#### Test 6.2.1: sh:class Constraint
```turtle
# Shape
:PersonShape a sh:NodeShape ;
    sh:property [
        sh:path :knows ;
        sh:class :Person ;
    ] .

# Valid
:Alice :knows :Bob .
:Bob rdf:type :Person .

# Invalid (Charlie not a Person)
:Alice :knows :Charlie .

# Expected: Violation if :Charlie not typed as :Person
```

#### Test 6.2.2: sh:minInclusive and sh:maxInclusive
```turtle
# Shape
:AgeShape a sh:NodeShape ;
    sh:property [
        sh:path :age ;
        sh:minInclusive 0 ;
        sh:maxInclusive 150 ;
    ] .

# Valid
:Alice :age "30"^^xsd:integer .

# Invalid
:Bob :age "-5"^^xsd:integer .
:Charlie :age "200"^^xsd:integer .

# Expected: Violations for :Bob and :Charlie
```

---

### 6.3 SPARQL-based Constraints

**What it tests:** Custom SHACL validation using SPARQL

**Test Patterns:**

#### Test 6.3.1: sh:sparql Constraint
```turtle
# Shape with SPARQL constraint
:PersonShape a sh:NodeShape ;
    sh:targetClass :Person ;
    sh:sparql [
        sh:message "Person must have at least one friend" ;
        sh:select """
            SELECT $this
            WHERE {
                $this a :Person .
                FILTER NOT EXISTS { $this :knows ?friend }
            }
        """ ;
    ] .

# Valid (has friend)
:Alice :knows :Bob .

# Invalid (no friends)
:Charlie a :Person .

# Expected: Violation for :Charlie
```

**Rust considerations:**
```rust
struct ShaclShape {
    id: IRI,
    target_class: Option<IRI>,
    properties: Vec<PropertyShape>,
    sparql_constraints: Vec<SparqlConstraint>,
}

struct ValidationReport {
    conforms: bool,
    violations: Vec<ValidationViolation>,
}

struct ValidationViolation {
    focus_node: Node,
    result_path: Option<IRI>,
    message: String,
    severity: Severity,
}

enum Severity {
    Info,
    Warning,
    Violation,
}
```

---

## 7. Integration & End-to-End Tests

### 7.1 Model Lifecycle Tests

**What it tests:** Complete RDF model operations from creation to query

**Test Patterns:**

#### Test 7.1.1: Create, Populate, Query, Update, Query Again
```rust
#[test]
fn test_model_lifecycle() {
    // Create empty model
    let mut model = Model::new();

    // Add triples
    model.add_triple(
        IRI::new("http://ex.org/Alice"),
        IRI::new("http://ex.org/age"),
        Literal::integer(30),
    );

    // Query
    let results = model.query("SELECT ?s ?age WHERE { ?s <http://ex.org/age> ?age }");
    assert_eq!(results.len(), 1);

    // Update
    model.delete_triple(...);
    model.add_triple(...);

    // Query again
    let results2 = model.query(...);
    assert_eq!(results2.len(), 2);
}
```

---

### 7.2 Serialization Round-Trip Tests

**What it tests:** RDF format reading and writing preserves semantics

**Test Patterns:**

#### Test 7.2.1: Turtle -> Model -> Turtle
```rust
#[test]
fn test_turtle_roundtrip() {
    let input_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:Alice ex:knows ex:Bob .
        ex:Bob ex:age "25"^^xsd:integer .
    "#;

    // Parse
    let model = parse_turtle(input_ttl).unwrap();

    // Serialize
    let output_ttl = model.to_turtle();

    // Parse again
    let model2 = parse_turtle(&output_ttl).unwrap();

    // Check isomorphism
    assert!(model.is_isomorphic_to(&model2));
}
```

#### Test 7.2.2: Cross-Format Round-Trip
```rust
#[test]
fn test_cross_format_roundtrip() {
    let ttl = "...";
    let model1 = parse_turtle(ttl).unwrap();

    // To N-Triples
    let nt = model1.to_ntriples();
    let model2 = parse_ntriples(&nt).unwrap();

    // To RDF/XML
    let rdfxml = model2.to_rdfxml();
    let model3 = parse_rdfxml(&rdfxml).unwrap();

    assert!(model1.is_isomorphic_to(&model3));
}
```

---

### 7.3 Graph Isomorphism Tests

**What it tests:** Blank node graph comparison

**Test Patterns:**

#### Test 7.3.1: Simple Isomorphism
```rust
#[test]
fn test_graph_isomorphism() {
    let graph1 = parse_turtle(r#"
        _:b1 <p> "value" .
        _:b1 <q> _:b2 .
        _:b2 <r> "other" .
    "#).unwrap();

    let graph2 = parse_turtle(r#"
        _:x <p> "value" .
        _:x <q> _:y .
        _:y <r> "other" .
    "#).unwrap();

    // Blank node labels differ, but graphs are isomorphic
    assert!(graph1.is_isomorphic_to(&graph2));
}
```

**Rust considerations:**
```rust
fn is_isomorphic_to(&self, other: &Graph) -> bool {
    // Use VF2 algorithm or canonical labeling
    // Libraries: petgraph, graphlib

    // 1. Check triple counts match
    // 2. Build blank node mapping using backtracking
    // 3. Verify all triples match under mapping
}
```

---

### 7.4 Performance Benchmarks

**What it tests:** Query performance, memory usage, scalability

**Test Patterns:**

#### Test 7.4.1: Large Dataset Loading
```rust
#[bench]
fn bench_load_million_triples(b: &mut Bencher) {
    b.iter(|| {
        let model = Model::new();
        // Load 1M triples from file
        model.load_from_file("large_dataset.nt").unwrap();
    });
}
```

#### Test 7.4.2: Complex Query Performance
```rust
#[bench]
fn bench_complex_sparql_query(b: &mut Bencher) {
    let model = setup_large_model();
    let query = "SELECT ?s ?o WHERE { ?s ?p ?o . ?o ?p2 ?o2 }";

    b.iter(|| {
        model.query(query)
    });
}
```

---

## 8. RDF-star Tests

### 8.1 Quoted Triples

**What it tests:** RDF-star syntax for assertions about statements

**Why it's critical:**
- Metadata about triples
- Provenance tracking
- Temporal annotations

**Test Patterns:**

#### Test 8.1.1: Simple Quoted Triple
```turtle
# RDF-star syntax
<< <Alice> <knows> <Bob> >> <validFrom> "2020-01-01"^^xsd:date .

# Expected: Statement about a statement
# The triple <Alice> <knows> <Bob> is annotated with validFrom
```

#### Test 8.1.2: Nested Quoted Triples
```turtle
# Input
<< << <s> <p> "o" >> <validFrom> "2023-02-06T12:00:00"^^xsd:dateTime >>
    <validUntil> "9999-12-31T12:00:00"^^xsd:dateTime .

# Expected: Two levels of quotation
# Statement about a statement about a statement
```

#### Test 8.1.3: RDF-star Annotation Syntax
```turtle
# Compact annotation syntax
<Alice> <knows> <Bob> {|
    <since> "2020-01-01"^^xsd:date ;
    <certainty> "high"
|} .

# Equivalent to
<< <Alice> <knows> <Bob> >> <since> "2020-01-01"^^xsd:date .
<< <Alice> <knows> <Bob> >> <certainty> "high" .
```

---

### 8.2 RDF-star SPARQL Queries

**What it tests:** Querying quoted triples

**Test Patterns:**

#### Test 8.2.1: Query Quoted Triples
```sparql
SELECT ?when
WHERE {
    << <Alice> <knows> ?person >> <validFrom> ?when .
}

# Find when Alice's knowledge relationships became valid
```

#### Test 8.2.2: CONSTRUCT with Quoted Triples
```sparql
CONSTRUCT {
    << ?s ?p ?o >> <source> <http://example.org/db1> .
}
WHERE {
    ?s ?p ?o .
}

# Add source provenance to all triples
```

**Rust considerations:**
```rust
enum Node {
    IRI(IRI),
    BlankNode(BlankNodeId),
    Literal(Literal),
    QuotedTriple(Box<Triple>),  // RDF-star support
}

struct Triple {
    subject: SubjectNode,     // IRI | BlankNode | QuotedTriple
    predicate: IRI,
    object: Node,             // IRI | BlankNode | Literal | QuotedTriple
}
```

---

## Summary of Critical Test Categories

### Priority 1 (Must-Have for Basic Compliance)
1. **RDF Model**: Blank nodes, literals, IRI validation
2. **Turtle Parser**: Prefix handling, collections, literals
3. **SPARQL Basic**: SELECT, triple patterns, FILTER
4. **SPARQL Update**: INSERT/DELETE DATA
5. **Graph Isomorphism**: Blank node comparison

### Priority 2 (Standard SPARQL 1.1 Features)
6. **SPARQL Advanced**: OPTIONAL, UNION, property paths
7. **SPARQL Aggregation**: GROUP BY, COUNT, SUM, AVG
8. **SPARQL Solution Modifiers**: ORDER BY, LIMIT, DISTINCT
9. **SPARQL Update**: INSERT/DELETE WHERE, graph management
10. **Reasoning (RDFS)**: Basic subclass/subproperty inference

### Priority 3 (Advanced Features)
11. **SPARQL Federated**: SERVICE clause
12. **RDF-star**: Quoted triples
13. **SHACL**: Shape validation
14. **OWL**: Advanced reasoning

---

## Rust-Specific Implementation Considerations

### 1. Error Handling
```rust
#[derive(Debug, thiserror::Error)]
enum RdfError {
    #[error("Parse error at line {line}, column {col}: {message}")]
    ParseError { line: usize, col: usize, message: String },

    #[error("Invalid IRI: {0}")]
    InvalidIri(String),

    #[error("Type error: expected {expected}, got {actual}")]
    TypeError { expected: String, actual: String },
}
```

### 2. Memory Efficiency
- Use `Arc<String>` for shared IRI strings
- Intern common URIs (rdf:type, rdfs:subClassOf, etc.)
- Use custom allocators for blank nodes

### 3. Parsing Strategy
- Use `nom` or `pest` for parser combinators
- Streaming parser for large files
- Error recovery for partial parsing

### 4. Query Optimization
- Build query plan optimizer
- Use indexes: SPO, POS, OSP
- Statistics collection for cardinality estimation

### 5. Testing Framework
```rust
// Use property-based testing for RDF graph operations
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_graph_isomorphism_reflexive(triples: Vec<Triple>) {
        let graph = Graph::from_triples(triples);
        assert!(graph.is_isomorphic_to(&graph));
    }
}
```

---

## W3C Test Suite Integration

All implementations should pass the official W3C test suites:

1. **RDF 1.1 Test Suite**: https://w3c.github.io/rdf-tests/
   - Turtle: https://w3c.github.io/rdf-tests/rdf/rdf11/rdf-turtle/
   - N-Triples: https://w3c.github.io/rdf-tests/rdf/rdf11/rdf-n-triples/
   - TriG: https://w3c.github.io/rdf-tests/rdf/rdf11/rdf-trig/

2. **SPARQL 1.1 Test Suite**: https://w3c.github.io/rdf-tests/sparql/sparql11/
   - Manifest: https://w3c.github.io/rdf-tests/sparql/sparql11/manifest-all.ttl

3. **RDF-star Test Suite**: https://w3c.github.io/rdf-star/tests/

### Running W3C Tests in Rust
```rust
// Example test runner for W3C manifests
#[test]
fn run_w3c_turtle_tests() {
    let manifest = load_manifest("https://w3c.github.io/rdf-tests/rdf/rdf11/rdf-turtle/manifest.ttl");

    for test in manifest.tests() {
        match test.test_type() {
            TestType::PositiveSyntax => {
                let result = parse_turtle_file(test.action());
                assert!(result.is_ok(), "Failed to parse valid Turtle: {}", test.name());
            }
            TestType::NegativeSyntax => {
                let result = parse_turtle_file(test.action());
                assert!(result.is_err(), "Should reject invalid Turtle: {}", test.name());
            }
            TestType::Evaluation => {
                let parsed = parse_turtle_file(test.action()).unwrap();
                let expected = parse_ntriples_file(test.result()).unwrap();
                assert!(parsed.is_isomorphic_to(&expected), "Evaluation failed: {}", test.name());
            }
        }
    }
}
```

---

## References

- Apache Jena ARQ Documentation: https://jena.apache.org/documentation/query/
- Apache Jena SHACL: https://jena.apache.org/documentation/shacl/
- W3C RDF 1.1 Specification: https://www.w3.org/TR/rdf11-concepts/
- W3C SPARQL 1.1 Specification: https://www.w3.org/TR/sparql11-query/
- W3C Turtle Specification: https://www.w3.org/TR/turtle/
- W3C SHACL Specification: https://www.w3.org/TR/shacl/
- RDF-star Working Draft: https://w3c.github.io/rdf-star/

---

**End of Research Document**
