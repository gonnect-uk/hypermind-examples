# TypeScript SDK Implementation Guide

This document provides step-by-step instructions for implementing the TypeScript/JavaScript SDK bindings for rust-kgdb using NAPI-RS.

## Overview

The TypeScript SDK architecture uses NAPI-RS (NOT UniFFI, which doesn't support JavaScript):

```
TypeScript/JavaScript Application
    ↓
rust-kgdb (TypeScript wrapper) ← YOU IMPLEMENT THIS
    ↓
NAPI-RS Bindings              ← YOU IMPLEMENT THIS
    ↓
Core Engine (sparql + storage) ← ALREADY EXISTS
```

## Why NAPI-RS Instead of UniFFI?

- **UniFFI**: Designed for mobile platforms (Swift, Kotlin, Python, Ruby)
- **NAPI-RS**: Designed specifically for Node.js and JavaScript/TypeScript
- **Performance**: NAPI-RS has lower overhead for Node.js
- **Ecosystem**: Better npm integration

## Prerequisites

- Node.js 18+ and npm 9+
- Rust toolchain
- NAPI-RS CLI tools

## Step 1: Install NAPI-RS

```bash
# Install NAPI-RS CLI
npm install -g @napi-rs/cli

# Verify installation
napi --version
```

## Step 2: Create NAPI-RS Bindings Crate

```bash
# From repository root
cd crates
cargo new napi-bindings --lib

# Add to workspace Cargo.toml
# [workspace]
# members = [..., "crates/napi-bindings"]
```

## Step 3: Configure Cargo.toml for NAPI-RS

Edit `crates/napi-bindings/Cargo.toml`:

```toml
[package]
name = "napi-bindings"
version = "0.1.2"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Core engine
rdf-model = { path = "../rdf-model" }
storage = { path = "../storage" }
sparql = { path = "../sparql" }
rdf-io = { path = "../rdf-io" }

# NAPI-RS
napi = "2.16"
napi-derive = "2.16"

[build-dependencies]
napi-build = "2.1"
```

## Step 4: Implement NAPI-RS Bindings

Create `crates/napi-bindings/src/lib.rs`:

```rust
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{Arc, Mutex};

use rdf_model::{Dictionary, Node as RdfNode, Quad, Triple};
use storage::{InMemoryBackend, QuadStore};
use sparql::{Executor, SPARQLParser};

// ============================================================================
// GraphDB Class
// ============================================================================

#[napi]
pub struct GraphDB {
    store: Arc<Mutex<QuadStore<InMemoryBackend>>>,
    dictionary: Arc<Dictionary>,
}

#[napi]
impl GraphDB {
    /// Creates a new in-memory database
    #[napi(factory)]
    pub fn in_memory() -> Result<GraphDB> {
        let dictionary = Arc::new(Dictionary::new());
        let backend = InMemoryBackend::new();
        let store = QuadStore::new(backend, dictionary.clone());

        Ok(GraphDB {
            store: Arc::new(Mutex::new(store)),
            dictionary,
        })
    }

    /// Counts total triples in the database
    #[napi]
    pub fn count(&self) -> Result<u32> {
        let store = self.store.lock().unwrap();
        Ok(store.count() as u32)
    }

    /// Checks if database is empty
    #[napi]
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.count()? == 0)
    }

    /// Clears all triples
    #[napi]
    pub fn clear(&mut self) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store.clear();
        Ok(())
    }

    /// Inserts a triple
    #[napi]
    pub fn insert_triple(
        &mut self,
        subject: &Node,
        predicate: &Node,
        object: &Node,
        graph: Option<&Node>,
    ) -> Result<()> {
        let s = node_to_rdf(&self.dictionary, subject)?;
        let p = node_to_rdf(&self.dictionary, predicate)?;
        let o = node_to_rdf(&self.dictionary, object)?;
        let g = graph.map(|n| node_to_rdf(&self.dictionary, n)).transpose()?;

        let quad = Quad::new(s, p, o, g);
        let mut store = self.store.lock().unwrap();
        store.insert(quad).map_err(|e| {
            Error::from_reason(format!("Insert failed: {}", e))
        })?;

        Ok(())
    }

    /// Executes a SPARQL query
    #[napi]
    pub fn execute_sparql(&self, query: String) -> Result<String> {
        let mut parser = SPARQLParser::new();
        let parsed = parser.parse(&query).map_err(|e| {
            Error::from_reason(format!("Parse error: {}", e))
        })?;

        let store = self.store.lock().unwrap();
        let executor = Executor::new(&*store, self.dictionary.clone());
        let bindings = executor.execute_select(&parsed).map_err(|e| {
            Error::from_reason(format!("Execution error: {}", e))
        })?;

        // Convert to JSON
        let json = bindings_to_json(&bindings);
        Ok(json)
    }
}

// ============================================================================
// Node Class
// ============================================================================

#[napi]
pub struct Node {
    kind: NodeKind,
}

#[napi]
pub enum NodeKind {
    Iri,
    Literal,
    BlankNode,
}

#[napi]
impl Node {
    /// Creates an IRI node
    #[napi(factory)]
    pub fn iri(uri: String) -> Result<Node> {
        Ok(Node {
            kind: NodeKind::Iri,
            value: uri,
            datatype: None,
            language: None,
        })
    }

    /// Creates a plain literal
    #[napi(factory)]
    pub fn literal(value: String) -> Result<Node> {
        Ok(Node {
            kind: NodeKind::Literal,
            value,
            datatype: None,
            language: None,
        })
    }

    /// Creates a typed literal
    #[napi(factory)]
    pub fn typed_literal(value: String, datatype: String) -> Result<Node> {
        Ok(Node {
            kind: NodeKind::Literal,
            value,
            datatype: Some(datatype),
            language: None,
        })
    }

    /// Creates a language-tagged literal
    #[napi(factory)]
    pub fn lang_literal(value: String, lang: String) -> Result<Node> {
        Ok(Node {
            kind: NodeKind::Literal,
            value,
            datatype: None,
            language: Some(lang),
        })
    }

    /// Creates an integer literal
    #[napi(factory)]
    pub fn integer(value: i32) -> Result<Node> {
        Self::typed_literal(
            value.to_string(),
            "http://www.w3.org/2001/XMLSchema#integer".to_string(),
        )
    }

    /// Creates a boolean literal
    #[napi(factory)]
    pub fn boolean(value: bool) -> Result<Node> {
        Self::typed_literal(
            value.to_string(),
            "http://www.w3.org/2001/XMLSchema#boolean".to_string(),
        )
    }

    /// Creates a double literal
    #[napi(factory)]
    pub fn double(value: f64) -> Result<Node> {
        Self::typed_literal(
            value.to_string(),
            "http://www.w3.org/2001/XMLSchema#double".to_string(),
        )
    }

    /// Creates a blank node
    #[napi(factory)]
    pub fn blank(id: String) -> Result<Node> {
        Ok(Node {
            kind: NodeKind::BlankNode,
            value: id,
            datatype: None,
            language: None,
        })
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn node_to_rdf(dict: &Arc<Dictionary>, node: &Node) -> Result<RdfNode> {
    match node.kind {
        NodeKind::Iri => {
            let interned = dict.intern(&node.value);
            Ok(RdfNode::iri(interned))
        }
        NodeKind::Literal => {
            let val = dict.intern(&node.value);
            if let Some(ref dt) = node.datatype {
                let dt_interned = dict.intern(dt);
                Ok(RdfNode::literal_typed(val, dt_interned))
            } else if let Some(ref lang) = node.language {
                let lang_interned = dict.intern(lang);
                Ok(RdfNode::literal_lang(val, lang_interned))
            } else {
                Ok(RdfNode::literal_str(val))
            }
        }
        NodeKind::BlankNode => {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            node.value.hash(&mut hasher);
            Ok(RdfNode::blank(hasher.finish()))
        }
    }
}

fn bindings_to_json(bindings: &sparql::BindingSet) -> String {
    use serde_json::json;

    let results: Vec<_> = bindings
        .iter()
        .map(|binding| {
            let vars: serde_json::Map<String, serde_json::Value> = binding
                .iter()
                .map(|(var, node)| {
                    (var.name.to_string(), json!(node_to_string(node)))
                })
                .collect();
            json!(vars)
        })
        .collect();

    serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
}

fn node_to_string(node: &RdfNode) -> String {
    match node {
        RdfNode::IRI(iri) => format!("<{}>", iri),
        RdfNode::Literal(value, dtype, lang) => {
            if let Some(l) = lang {
                format!("\"{}\"@{}", value, l)
            } else if let Some(dt) = dtype {
                format!("\"{}\"^^<{}>", value, dt)
            } else {
                format!("\"{}\"", value)
            }
        }
        RdfNode::BlankNode(id) => format!("_:b{}", id),
        _ => String::new(),
    }
}
```

## Step 5: Create TypeScript Wrapper

Create `sdks/typescript/src/index.ts`:

```typescript
import { GraphDB as NativeGraphDB, Node as NativeNode } from '../native';

/**
 * RDF graph database with SPARQL support.
 *
 * @example
 * ```typescript
 * const db = GraphDB.inMemory();
 *
 * await db.insert()
 *   .triple(
 *     Node.iri("http://example.org/alice"),
 *     Node.iri("http://xmlns.com/foaf/0.1/name"),
 *     Node.literal("Alice")
 *   )
 *   .execute();
 *
 * const results = await db.query()
 *   .sparql("SELECT ?name WHERE { ?person <http://xmlns.com/foaf/0.1/name> ?name }")
 *   .execute();
 * ```
 */
export class GraphDB {
    private _native: NativeGraphDB;

    private constructor(native: NativeGraphDB) {
        this._native = native;
    }

    /**
     * Creates a new in-memory database.
     */
    static inMemory(): GraphDB {
        return new GraphDB(NativeGraphDB.inMemory());
    }

    /**
     * Starts building an insert operation.
     */
    insert(): InsertBuilder {
        return new InsertBuilder(this._native);
    }

    /**
     * Starts building a SPARQL query.
     */
    query(): QueryBuilder {
        return new QueryBuilder(this._native);
    }

    /**
     * Counts total triples.
     */
    count(): number {
        return this._native.count();
    }

    /**
     * Checks if database is empty.
     */
    isEmpty(): boolean {
        return this._native.isEmpty();
    }

    /**
     * Clears all triples.
     */
    clear(): void {
        this._native.clear();
    }
}

// ... InsertBuilder, QueryBuilder, etc. (similar structure to Python/Kotlin)
```

## Step 6: Create package.json

```json
{
  "name": "@zenya/rust-kgdb",
  "version": "0.1.2",
  "description": "Production-ready TypeScript bindings for rust-kgdb RDF/SPARQL database",
  "main": "index.js",
  "types": "index.d.ts",
  "napi": {
    "name": "rust-kgdb",
    "triples": {
      "defaults": true,
      "additional": [
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "aarch64-linux-android"
      ]
    }
  },
  "scripts": {
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "prepublishOnly": "napi prepublish -t npm",
    "test": "jest",
    "artifacts": "napi artifacts",
    "version": "napi version"
  },
  "devDependencies": {
    "@napi-rs/cli": "^2.18.0",
    "@types/node": "^20.10.0",
    "jest": "^29.7.0",
    "typescript": "^5.3.0"
  },
  "keywords": ["rdf", "sparql", "graph", "database", "semantic"],
  "license": "MIT"
}
```

## Step 7: Port Regression Tests

Create `sdks/typescript/tests/regression.test.ts`:

```typescript
import { GraphDB, Node } from '../src';

describe('Regression Tests', () => {
    let db: GraphDB;

    beforeEach(() => {
        db = GraphDB.inMemory();
    });

    afterEach(() => {
        db.clear();
    });

    test('basic CRUD', () => {
        db.insert()
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                Node.iri("http://example.org/TestClass")
            )
            .execute();

        expect(db.count()).toBe(1);

        const results = db.query()
            .sparql("SELECT ?type WHERE { <http://example.org/test> a ?type }")
            .execute();

        expect(results.length).toBe(1);
    });

    // Tests 2-20...
});
```

## Implementation Checklist

- [ ] Create NAPI-RS bindings crate
- [ ] Implement GraphDB class with NAPI-RS
- [ ] Implement Node factory with NAPI-RS
- [ ] Create TypeScript wrapper
- [ ] Add type definitions (.d.ts)
- [ ] Port all 20 regression tests to Jest
- [ ] Create package.json with NAPI-RS config
- [ ] Build for multiple platforms
- [ ] Generate TypeDoc documentation
- [ ] Test in Node.js and browser (via webpack)

## Build Commands

```bash
# Build native module
cd sdks/typescript
npm run build

# Run tests
npm test

# Build for all platforms
napi build --platform

# Generate type definitions
npm run build:types
```

## Estimated Effort

- NAPI-RS bindings crate: 8 hours
- TypeScript wrapper: 4 hours
- Test porting: 3 hours
- Documentation: 2 hours
- Multi-platform build: 2 hours

**Total: ~19 hours (2.5 days)**

## Notes

- NAPI-RS is the ONLY way to create Node.js bindings for Rust
- UniFFI does NOT support JavaScript/TypeScript
- NAPI-RS has excellent TypeScript support
- Can generate bindings for browser via wasm (future work)
