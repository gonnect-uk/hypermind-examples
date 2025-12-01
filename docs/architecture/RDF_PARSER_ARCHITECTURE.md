# RDF Parser Architecture - Professional Design

**Status**: v0.1.3+ Implementation Plan
**Date**: 2025-11-29
**Goal**: Add 4 RDF formats (N-Quads, TriG, JSON-LD, RDF/XML) with Strategy pattern

---

## 🎯 Architecture Overview

### Design Principles

1. **Strategy Pattern** - Unified `RDFParser` trait for all formats
2. **nom Parser Combinator** - Use nom for ALL parsers (consistency with Turtle)
3. **Unified Storage** - All formats → `Quad` structs → Single storage layer
4. **Factory Pattern** - Auto-detect format from file extension
5. **100% W3C Compliance** - Each parser follows official W3C specifications

---

## 📐 System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                 RDF Format Detection (Factory)                   │
│  .ttl → Turtle | .nt → N-Triples | .nq → N-Quads | .trig → TriG │
│  .jsonld → JSON-LD | .rdf/.owl → RDF/XML                        │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│            RDFParser Trait (Strategy Interface)                  │
│  - parse(&mut self, content: &str) -> ParseResult<Vec<Quad>>   │
│  - format(&self) -> RDFFormat                                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌──────────┬──────────┬──────────┬──────────┬──────────┬─────────┐
│  Turtle  │N-Triples │ N-Quads  │   TriG   │ JSON-LD  │ RDF/XML │
│  Parser  │  Parser  │  Parser  │  Parser  │  Parser  │  Parser │
│  (nom)   │  (nom)   │  (nom)   │  (nom)   │  (nom)   │  (nom)  │
└──────────┴──────────┴──────────┴──────────┴──────────┴─────────┘
                              ↓
         ┌────────────────────────────────────────────────┐
         │       Unified Quad Structure (4 components)     │
         │  Quad { subject, predicate, object, graph }    │
         └────────────────────────────────────────────────┘
                              ↓
         ┌────────────────────────────────────────────────┐
         │         QuadStore (Storage Backend)             │
         │  SPOC/POCS/OCSP/CSPO Indexes                   │
         │  InMemory / RocksDB / LMDB                     │
         └────────────────────────────────────────────────┘
```

---

## 🔧 Implementation Plan

### Phase 1: Core Trait & Factory

**File**: `crates/rdf-io/src/lib.rs`

```rust
/// Strategy trait for RDF parsers
pub trait RDFParser {
    /// Parse RDF content into Quads with dictionary interning
    fn parse(&mut self, content: &str) -> ParseResult<Vec<Quad>>;

    /// Get the format this parser handles
    fn format(&self) -> RDFFormat;
}

/// Factory for creating parsers based on format
pub struct ParserFactory;

impl ParserFactory {
    /// Create parser for specified format
    pub fn create(format: RDFFormat, dictionary: Arc<Dictionary>) -> Box<dyn RDFParser> {
        match format {
            RDFFormat::Turtle => Box::new(TurtleParser::new(dictionary)),
            RDFFormat::NTriples => Box::new(NTriplesParser::new(dictionary)),
            RDFFormat::NQuads => Box::new(NQuadsParser::new(dictionary)),
            RDFFormat::TriG => Box::new(TriGParser::new(dictionary)),
            RDFFormat::JSONLD => Box::new(JSONLDParser::new(dictionary)),
            RDFFormat::RDFXML => Box::new(RDFXMLParser::new(dictionary)),
        }
    }

    /// Auto-detect format from filename
    pub fn detect_format(filename: &str) -> RDFFormat {
        match filename.rsplit('.').next() {
            Some("ttl") => RDFFormat::Turtle,
            Some("nt") => RDFFormat::NTriples,
            Some("nq") => RDFFormat::NQuads,
            Some("trig") => RDFFormat::TriG,
            Some("jsonld") => RDFFormat::JSONLD,
            Some("rdf") | Some("owl") | Some("xml") => RDFFormat::RDFXML,
            _ => RDFFormat::Turtle // default
        }
    }

    /// Parse file with auto-detected format
    pub fn parse_file(filename: &str, dictionary: Arc<Dictionary>) -> ParseResult<Vec<Quad>> {
        let format = Self::detect_format(filename);
        let content = std::fs::read_to_string(filename)?;
        let mut parser = Self::create(format, dictionary);
        parser.parse(&content)
    }
}
```

### Phase 2: Implement RDFParser for Existing Parsers

**File**: `crates/rdf-io/src/turtle.rs` (add trait impl)

```rust
impl RDFParser for TurtleParser {
    fn parse(&mut self, content: &str) -> ParseResult<Vec<Quad>> {
        // Existing parse logic
        self.parse(content)
    }

    fn format(&self) -> RDFFormat {
        RDFFormat::Turtle
    }
}
```

**File**: `crates/rdf-io/src/ntriples.rs` (add trait impl)

```rust
impl RDFParser for NTriplesParser {
    fn parse(&mut self, content: &str) -> ParseResult<Vec<Quad>> {
        // Existing parse logic
        self.parse(content)
    }

    fn format(&self) -> RDFFormat {
        RDFFormat::NTriples
    }
}
```

### Phase 3: N-Quads Parser (nom-based)

**File**: `crates/rdf-io/src/nquads.rs`

**Based on W3C N-Quads spec**: https://www.w3.org/TR/n-quads/

```rust
use nom::{
    IResult,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0, multispace1},
    combinator::{opt, map},
    sequence::{preceded, tuple, terminated},
    branch::alt,
};

/// N-Quads parser (N-Triples + named graph)
pub struct NQuadsParser {
    dictionary: Arc<Dictionary>,
}

impl NQuadsParser {
    pub fn new(dictionary: Arc<Dictionary>) -> Self {
        Self { dictionary }
    }

    /// Parse N-Quads: subject predicate object [graph] .
    fn parse_quad(&mut self, input: &str) -> IResult<&str, Quad> {
        let (input, _) = multispace0(input)?;
        let (input, subject) = self.parse_subject(input)?;
        let (input, _) = multispace1(input)?;
        let (input, predicate) = self.parse_predicate(input)?;
        let (input, _) = multispace1(input)?;
        let (input, object) = self.parse_object(input)?;
        let (input, _) = multispace0(input)?;
        let (input, graph) = opt(preceded(multispace1, self.parse_graph))(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('.')(input)?;

        Ok((input, Quad::new(subject, predicate, object, graph)))
    }
}

impl RDFParser for NQuadsParser {
    fn parse(&mut self, content: &str) -> ParseResult<Vec<Quad>> {
        // Parse line by line (N-Quads is line-based)
        content.lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|line| {
                self.parse_quad(line)
                    .map(|(_, quad)| quad)
                    .map_err(|e| ParseError::Syntax {
                        line: 0, col: 0,
                        message: format!("N-Quads parse error: {}", e)
                    })
            })
            .collect()
    }

    fn format(&self) -> RDFFormat {
        RDFFormat::NQuads
    }
}
```

### Phase 4: TriG Parser (nom-based)

**File**: `crates/rdf-io/src/trig.rs`

**Based on W3C TriG spec**: https://www.w3.org/TR/trig/

```rust
/// TriG parser (Turtle + named graphs)
/// Syntax: { GRAPH <uri> { turtle_content } }
pub struct TriGParser {
    dictionary: Arc<Dictionary>,
    current_graph: Option<Node<'static>>,
}

impl TriGParser {
    pub fn new(dictionary: Arc<Dictionary>) -> Self {
        Self {
            dictionary,
            current_graph: None,
        }
    }

    /// Parse TriG: combines Turtle with graph blocks
    /// GRAPH <http://example.org/g1> { :s :p :o . }
    fn parse_graph_block(&mut self, input: &str) -> IResult<&str, Vec<Quad>> {
        let (input, _) = tag("GRAPH")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, graph_iri) = self.parse_iri(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('{')(input)?;

        // Set current graph context
        self.current_graph = Some(graph_iri);

        // Parse Turtle triples within this graph
        let (input, triples) = self.parse_turtle_block(input)?;

        let (input, _) = multispace0(input)?;
        let (input, _) = char('}')(input)?;

        // Convert triples to quads with graph
        let quads = triples.into_iter()
            .map(|triple| Quad::new(
                triple.subject,
                triple.predicate,
                triple.object,
                self.current_graph.clone()
            ))
            .collect();

        Ok((input, quads))
    }
}

impl RDFParser for TriGParser {
    fn parse(&mut self, content: &str) -> ParseResult<Vec<Quad>> {
        // TriG can have:
        // 1. Default graph triples (no GRAPH keyword)
        // 2. Named graph blocks (GRAPH <uri> { ... })
        // Parse both and combine
        todo!("Full TriG implementation")
    }

    fn format(&self) -> RDFFormat {
        RDFFormat::TriG
    }
}
```

### Phase 5: JSON-LD Parser (nom + serde_json)

**File**: `crates/rdf-io/src/jsonld.rs`

**Based on W3C JSON-LD spec**: https://www.w3.org/TR/json-ld11/

```rust
use serde_json::Value;

/// JSON-LD parser
pub struct JSONLDParser {
    dictionary: Arc<Dictionary>,
}

impl JSONLDParser {
    pub fn new(dictionary: Arc<Dictionary>) -> Self {
        Self { dictionary }
    }

    /// Expand JSON-LD @context and convert to quads
    fn expand_context(&self, obj: &Value) -> ParseResult<Vec<Quad>> {
        // Parse @context, @id, @type, @value, @language, @graph
        todo!("JSON-LD expansion")
    }
}

impl RDFParser for JSONLDParser {
    fn parse(&mut self, content: &str) -> ParseResult<Vec<Quad>> {
        let json: Value = serde_json::from_str(content)?;
        self.expand_context(&json)
    }

    fn format(&self) -> RDFFormat {
        RDFFormat::JSONLD
    }
}
```

### Phase 6: RDF/XML Parser (nom + quick-xml)

**File**: `crates/rdf-io/src/rdfxml.rs`

**Based on W3C RDF/XML spec**: https://www.w3.org/TR/rdf-syntax-grammar/

```rust
use quick_xml::events::Event;
use quick_xml::Reader;

/// RDF/XML parser
pub struct RDFXMLParser {
    dictionary: Arc<Dictionary>,
}

impl RDFXMLParser {
    pub fn new(dictionary: Arc<Dictionary>) -> Self {
        Self { dictionary }
    }

    /// Parse RDF/XML streaming
    fn parse_xml_events(&mut self, reader: &mut Reader<&[u8]>) -> ParseResult<Vec<Quad>> {
        // Parse rdf:RDF, rdf:Description, rdf:about, rdf:resource
        todo!("RDF/XML streaming parse")
    }
}

impl RDFParser for RDFXMLParser {
    fn parse(&mut self, content: &str) -> ParseResult<Vec<Quad>> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);
        self.parse_xml_events(&mut reader)
    }

    fn format(&self) -> RDFFormat {
        RDFFormat::RDFXML
    }
}
```

---

## 🧪 Testing Strategy

### Unified Test Suite

**File**: `crates/rdf-io/tests/all_formats_test.rs`

```rust
/// Side-by-side tests for ALL 6 RDF formats
mod tests {
    use super::*;

    // Test data in all 6 formats (same semantic content)
    const TURTLE: &str = r#"
@prefix ex: <http://example.org/> .
ex:Alice ex:knows ex:Bob .
ex:Alice ex:age "25"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;

    const NTRIPLES: &str = r#"
<http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .
<http://example.org/Alice> <http://example.org/age> "25"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;

    const NQUADS: &str = r#"
<http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> <http://example.org/g1> .
<http://example.org/Alice> <http://example.org/age> "25"^^<http://www.w3.org/2001/XMLSchema#integer> <http://example.org/g1> .
"#;

    const TRIG: &str = r#"
@prefix ex: <http://example.org/> .
GRAPH ex:g1 {
    ex:Alice ex:knows ex:Bob .
    ex:Alice ex:age "25"^^<http://www.w3.org/2001/XMLSchema#integer> .
}
"#;

    const JSONLD: &str = r#"{
  "@context": {"ex": "http://example.org/"},
  "@id": "ex:Alice",
  "ex:knows": {"@id": "ex:Bob"},
  "ex:age": {"@value": "25", "@type": "http://www.w3.org/2001/XMLSchema#integer"}
}"#;

    const RDFXML: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/Alice">
    <ex:knows rdf:resource="http://example.org/Bob"/>
    <ex:age rdf:datatype="http://www.w3.org/2001/XMLSchema#integer">25</ex:age>
  </rdf:Description>
</rdf:RDF>"#;

    #[test]
    fn test_all_formats_parse() {
        let dict = Arc::new(Dictionary::new());

        // Parse all 6 formats
        let mut turtle = TurtleParser::new(Arc::clone(&dict));
        let mut ntriples = NTriplesParser::new(Arc::clone(&dict));
        let mut nquads = NQuadsParser::new(Arc::clone(&dict));
        let mut trig = TriGParser::new(Arc::clone(&dict));
        let mut jsonld = JSONLDParser::new(Arc::clone(&dict));
        let mut rdfxml = RDFXMLParser::new(Arc::clone(&dict));

        assert!(turtle.parse(TURTLE).is_ok());
        assert!(ntriples.parse(NTRIPLES).is_ok());
        assert!(nquads.parse(NQUADS).is_ok());
        assert!(trig.parse(TRIG).is_ok());
        assert!(jsonld.parse(JSONLD).is_ok());
        assert!(rdfxml.parse(RDFXML).is_ok());
    }

    #[test]
    fn test_factory_auto_detect() {
        let dict = Arc::new(Dictionary::new());

        assert_eq!(ParserFactory::detect_format("data.ttl"), RDFFormat::Turtle);
        assert_eq!(ParserFactory::detect_format("data.nt"), RDFFormat::NTriples);
        assert_eq!(ParserFactory::detect_format("data.nq"), RDFFormat::NQuads);
        assert_eq!(ParserFactory::detect_format("data.trig"), RDFFormat::TriG);
        assert_eq!(ParserFactory::detect_format("data.jsonld"), RDFFormat::JSONLD);
        assert_eq!(ParserFactory::detect_format("data.rdf"), RDFFormat::RDFXML);
    }
}
```

---

## 📦 SDK Integration

### Python SDK Example

**File**: `sdks/python/examples/multi_format_example.py`

```python
from rust_kgdb import GraphDB

# Create database
db = GraphDB("MultiFormatDB")

# Load Turtle
db.load_file("data.ttl")  # Auto-detects format

# Load N-Quads
db.load_file("data.nq")   # Auto-detects format

# Load JSON-LD
db.load_file("data.jsonld")  # Auto-detects format

# Query all data
results = db.query_select("SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10")
print(f"Found {len(results)} triples")
```

### TypeScript SDK Example

**File**: `sdks/typescript/examples/multi-format.ts`

```typescript
import { GraphDB } from 'rust-kgdb';

const db = new GraphDB('MultiFormatDB');

// Load different formats
await db.loadFile('data.ttl');      // Turtle
await db.loadFile('data.nq');       // N-Quads
await db.loadFile('data.jsonld');   // JSON-LD

// Query
const results = await db.querySelect('SELECT ?s ?p ?o WHERE { ?s ?p ?o }');
console.log(`Found ${results.length} triples`);
```

### Kotlin SDK Example

**File**: `sdks/kotlin/examples/MultiFormatExample.kt`

```kotlin
import com.zenya.rust_kgdb.GraphDB

fun main() {
    val db = GraphDB("MultiFormatDB")

    // Load different formats
    db.loadFile("data.ttl")      // Turtle
    db.loadFile("data.nq")       // N-Quads
    db.loadFile("data.jsonld")   // JSON-LD

    // Query
    val results = db.querySelect("SELECT ?s ?p ?o WHERE { ?s ?p ?o }")
    println("Found ${results.size} triples")
}
```

---

## 📊 Implementation Checklist

### Core (crates/rdf-io/)
- [ ] Add `RDFParser` trait to lib.rs
- [ ] Add `ParserFactory` to lib.rs
- [ ] Implement `RDFParser` for `TurtleParser`
- [ ] Implement `RDFParser` for `NTriplesParser`
- [ ] Create `nquads.rs` (nom-based)
- [ ] Create `trig.rs` (nom-based)
- [ ] Create `jsonld.rs` (nom + serde_json)
- [ ] Create `rdfxml.rs` (nom + quick-xml)

### Tests (crates/rdf-io/tests/)
- [ ] Create `all_formats_test.rs` (comprehensive)
- [ ] Add side-by-side parsing tests
- [ ] Add auto-detection tests
- [ ] Add error handling tests
- [ ] Add W3C conformance tests

### SDKs (sdks/)
- [ ] Python example (`python/examples/multi_format_example.py`)
- [ ] TypeScript example (`typescript/examples/multi-format.ts`)
- [ ] Kotlin example (`kotlin/examples/MultiFormatExample.kt`)

### Mobile FFI (crates/mobile-ffi/)
- [ ] Update `load_ttl_file` to use `ParserFactory::parse_file`
- [ ] Add format auto-detection
- [ ] Test with all 6 formats

---

## 🎯 Success Criteria

✅ All 6 parsers implement `RDFParser` trait
✅ `ParserFactory` auto-detects format from filename
✅ All parsers produce identical `Quad` structures
✅ 100% nom-based (except serde_json for JSON-LD, quick-xml for RDF/XML)
✅ Comprehensive test suite with side-by-side tests
✅ SDK examples for Python/TypeScript/Kotlin
✅ 100% W3C compliance for each format
✅ Production-grade error handling

**Estimated Implementation Time**: 6-8 hours for full implementation + tests + SDK examples
