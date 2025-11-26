# Rust-KGDB Reasoner Implementation Guide

**Version:** 1.0
**Date:** 2025-11-17
**Target:** Production-grade mobile hypergraph database reasoning engine
**Based on:** Apache Jena 5.x reasoner architecture

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Apache Jena Reasoner Architecture](#apache-jena-reasoner-architecture)
3. [RDFS Reasoner Implementation](#rdfs-reasoner-implementation)
4. [OWL 2 Profile Reasoners](#owl-2-profile-reasoners)
5. [Generic Rule Engine](#generic-rule-engine)
6. [RETE Algorithm Implementation](#rete-algorithm-implementation)
7. [Transitive Reasoner](#transitive-reasoner)
8. [Rust Architecture Design](#rust-architecture-design)
9. [Mobile Optimization Strategies](#mobile-optimization-strategies)
10. [Testing Requirements](#testing-requirements)
11. [Performance Benchmarks](#performance-benchmarks)
12. [Implementation Roadmap](#implementation-roadmap)

---

## Executive Summary

This guide provides complete specifications for implementing Apache Jena's production-grade reasoning capabilities in Rust for the rust-kgdb mobile hypergraph database. The implementation must achieve:

- **Zero Compromise**: All 13 RDFS rules, OWL 2 RL/EL/QL profiles, and RETE-based rule engine
- **Sub-millisecond Inference**: Mobile-optimized with zero-copy semantics
- **Production Quality**: Fortune 500 enterprise-grade reliability
- **Grammar-Based**: No string manipulation, pure structural operations
- **Memory Efficient**: Designed for mobile constraints (< 100MB reasoning engine)

### Key Differentiators from Apache Jena

| Aspect | Apache Jena (Java) | Rust-KGDB (Rust) |
|--------|-------------------|------------------|
| **Memory Model** | GC-based, allocation-heavy | Zero-copy lifetimes, arena allocation |
| **Concurrency** | Synchronized collections | Lock-free data structures |
| **Mobile Support** | Not optimized | Primary target platform |
| **Inference Speed** | ~10-50ms per query | Target < 1ms per query |
| **Memory Footprint** | 200MB+ runtime | < 100MB total |
| **Type Safety** | Runtime checks | Compile-time guarantees |

---

## Apache Jena Reasoner Architecture

### Overview

Apache Jena implements a modular reasoner framework with:

1. **Reasoner Interface**: Abstract contract for all reasoning implementations
2. **InfGraph**: Graph wrapper that exposes derived triples
3. **Rule System**: Forward/backward/hybrid chaining
4. **Property Functions**: Custom SPARQL extensions
5. **Validation Framework**: Constraint checking

### Core Components

```
Reasoner (Interface)
    ├── GenericRuleReasoner (configurable rule-based)
    ├── RDFSReasoner (RDFS entailment)
    ├── OWLReasoner (OWL Full)
    ├── OWLMicroReasoner (OWL Micro subset)
    ├── OWLMiniReasoner (OWL Mini subset)
    └── TransitiveReasoner (transitive closure only)

InfGraph (Interface)
    ├── BasicForwardRuleInfGraph (forward chaining)
    ├── LPBackwardRuleInfGraph (backward chaining)
    └── FBRuleInfGraph (hybrid forward-backward)
```

### Reasoner Lifecycle

1. **Configuration**: Rules, builtins, optimization flags
2. **Schema Binding**: Precompute TBox inferences
3. **Data Binding**: Apply reasoner to ABox data
4. **Inference**: Execute forward/backward chains
5. **Query**: Access materialized + virtual triples

---

## RDFS Reasoner Implementation

### Complete RDFS Entailment Rules

The W3C RDF 1.1 Semantics specification defines 13 RDFS entailment rules:

#### **rdfs1: Datatype Recognition**

**Rule:**
```
IF   graph contains IRI `D` in datatype map
THEN infer: D rdf:type rdfs:Datatype
```

**Rust Implementation:**
```rust
pub struct Rdfs1Rule;

impl InferenceRule for Rdfs1Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        for datatype_iri in KNOWN_DATATYPES {
            let triple = Triple::new(
                Node::iri(datatype_iri),
                Node::iri(RDF_TYPE),
                Node::iri(RDFS_DATATYPE)
            );
            ctx.infer(triple);
        }
    }
}
```

**Example:**
```turtle
# Given: xsd:integer exists in datatype map
# Infer:
xsd:integer rdf:type rdfs:Datatype .
```

---

#### **rdfs2: Domain Inference**

**Rule:**
```
IF   ?p rdfs:domain ?c .
     ?x ?p ?y .
THEN ?x rdf:type ?c .
```

**Rust Implementation:**
```rust
pub struct Rdfs2Rule;

impl InferenceRule for Rdfs2Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        // Pattern: (?p rdfs:domain ?c)
        let domain_pattern = QuadPattern::new(
            NodePattern::Variable("p"),
            NodePattern::Iri(RDFS_DOMAIN),
            NodePattern::Variable("c"),
            NodePattern::DefaultGraph,
        );

        for domain_quad in graph.find(&domain_pattern) {
            let property = domain_quad.subject();
            let class = domain_quad.object();

            // Pattern: (?x ?p ?y) where ?p matches above property
            let usage_pattern = QuadPattern::new(
                NodePattern::Variable("x"),
                NodePattern::Concrete(property.clone()),
                NodePattern::Variable("y"),
                NodePattern::DefaultGraph,
            );

            for usage_quad in graph.find(&usage_pattern) {
                let subject = usage_quad.subject();

                // Infer: ?x rdf:type ?c
                let inferred = Triple::new(
                    subject.clone(),
                    Node::iri(RDF_TYPE),
                    class.clone()
                );
                ctx.infer(inferred);
            }
        }
    }
}
```

**Example:**
```turtle
# Given:
:worksFor rdfs:domain :Person .
:alice :worksFor :CompanyX .

# Infer:
:alice rdf:type :Person .
```

---

#### **rdfs3: Range Inference**

**Rule:**
```
IF   ?p rdfs:range ?c .
     ?x ?p ?y .
THEN ?y rdf:type ?c .
```

**Rust Implementation:**
```rust
pub struct Rdfs3Rule;

impl InferenceRule for Rdfs3Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        let range_pattern = QuadPattern::new(
            NodePattern::Variable("p"),
            NodePattern::Iri(RDFS_RANGE),
            NodePattern::Variable("c"),
            NodePattern::DefaultGraph,
        );

        for range_quad in graph.find(&range_pattern) {
            let property = range_quad.subject();
            let class = range_quad.object();

            let usage_pattern = QuadPattern::new(
                NodePattern::Variable("x"),
                NodePattern::Concrete(property.clone()),
                NodePattern::Variable("y"),
                NodePattern::DefaultGraph,
            );

            for usage_quad in graph.find(&usage_pattern) {
                let object = usage_quad.object();

                // Infer: ?y rdf:type ?c
                let inferred = Triple::new(
                    object.clone(),
                    Node::iri(RDF_TYPE),
                    class.clone()
                );
                ctx.infer(inferred);
            }
        }
    }
}
```

**Example:**
```turtle
# Given:
:worksFor rdfs:range :Organization .
:alice :worksFor :CompanyX .

# Infer:
:CompanyX rdf:type :Organization .
```

---

#### **rdfs4a & rdfs4b: Resource Typing**

**Rules:**
```
rdfs4a: IF ?x ?a ?y THEN ?x rdf:type rdfs:Resource
rdfs4b: IF ?x ?a ?y THEN ?y rdf:type rdfs:Resource
```

**Rust Implementation:**
```rust
pub struct Rdfs4Rule;

impl InferenceRule for Rdfs4Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        // Pattern: (?x ?a ?y) - match all triples
        let all_pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Variable("a"),
            NodePattern::Variable("y"),
            NodePattern::DefaultGraph,
        );

        for quad in graph.find(&all_pattern) {
            let subject = quad.subject();
            let object = quad.object();

            // rdfs4a: subject is a resource
            let inferred_subj = Triple::new(
                subject.clone(),
                Node::iri(RDF_TYPE),
                Node::iri(RDFS_RESOURCE)
            );
            ctx.infer(inferred_subj);

            // rdfs4b: object is a resource (if not literal)
            if !object.is_literal() {
                let inferred_obj = Triple::new(
                    object.clone(),
                    Node::iri(RDF_TYPE),
                    Node::iri(RDFS_RESOURCE)
                );
                ctx.infer(inferred_obj);
            }
        }
    }
}
```

**Example:**
```turtle
# Given:
:alice :knows :bob .

# Infer:
:alice rdf:type rdfs:Resource .
:bob rdf:type rdfs:Resource .
```

---

#### **rdfs5: SubProperty Transitivity**

**Rule:**
```
IF   ?x rdfs:subPropertyOf ?y .
     ?y rdfs:subPropertyOf ?z .
THEN ?x rdfs:subPropertyOf ?z .
```

**Rust Implementation:**
```rust
pub struct Rdfs5Rule;

impl InferenceRule for Rdfs5Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        // Build transitive closure using Floyd-Warshall or similar
        let mut closure = TransitiveClosure::new();

        // Pattern: (?x rdfs:subPropertyOf ?y)
        let pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Iri(RDFS_SUB_PROPERTY_OF),
            NodePattern::Variable("y"),
            NodePattern::DefaultGraph,
        );

        // Collect direct subPropertyOf relations
        for quad in graph.find(&pattern) {
            closure.add_edge(quad.subject(), quad.object());
        }

        // Compute transitive closure
        closure.compute();

        // Emit inferred triples
        for (from, to) in closure.inferred_edges() {
            let inferred = Triple::new(
                from.clone(),
                Node::iri(RDFS_SUB_PROPERTY_OF),
                to.clone()
            );
            ctx.infer(inferred);
        }
    }
}
```

**Example:**
```turtle
# Given:
:parentOf rdfs:subPropertyOf :ancestorOf .
:ancestorOf rdfs:subPropertyOf :relatedTo .

# Infer:
:parentOf rdfs:subPropertyOf :relatedTo .
```

---

#### **rdfs6: Property Reflexivity**

**Rule:**
```
IF   ?x rdf:type rdf:Property .
THEN ?x rdfs:subPropertyOf ?x .
```

**Rust Implementation:**
```rust
pub struct Rdfs6Rule;

impl InferenceRule for Rdfs6Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        let pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Iri(RDF_TYPE),
            NodePattern::Iri(RDF_PROPERTY),
            NodePattern::DefaultGraph,
        );

        for quad in graph.find(&pattern) {
            let property = quad.subject();

            // Infer: ?x rdfs:subPropertyOf ?x
            let inferred = Triple::new(
                property.clone(),
                Node::iri(RDFS_SUB_PROPERTY_OF),
                property.clone()
            );
            ctx.infer(inferred);
        }
    }
}
```

**Example:**
```turtle
# Given:
:knows rdf:type rdf:Property .

# Infer:
:knows rdfs:subPropertyOf :knows .
```

---

#### **rdfs7: SubProperty Implication**

**Rule:**
```
IF   ?a rdfs:subPropertyOf ?b .
     ?x ?a ?y .
THEN ?x ?b ?y .
```

**Rust Implementation:**
```rust
pub struct Rdfs7Rule;

impl InferenceRule for Rdfs7Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        // Pattern: (?a rdfs:subPropertyOf ?b)
        let subprop_pattern = QuadPattern::new(
            NodePattern::Variable("a"),
            NodePattern::Iri(RDFS_SUB_PROPERTY_OF),
            NodePattern::Variable("b"),
            NodePattern::DefaultGraph,
        );

        for subprop_quad in graph.find(&subprop_pattern) {
            let sub_property = subprop_quad.subject();
            let super_property = subprop_quad.object();

            // Pattern: (?x ?a ?y)
            let usage_pattern = QuadPattern::new(
                NodePattern::Variable("x"),
                NodePattern::Concrete(sub_property.clone()),
                NodePattern::Variable("y"),
                NodePattern::DefaultGraph,
            );

            for usage_quad in graph.find(&usage_pattern) {
                let subject = usage_quad.subject();
                let object = usage_quad.object();

                // Infer: ?x ?b ?y
                let inferred = Triple::new(
                    subject.clone(),
                    super_property.clone(),
                    object.clone()
                );
                ctx.infer(inferred);
            }
        }
    }
}
```

**Example:**
```turtle
# Given:
:parentOf rdfs:subPropertyOf :ancestorOf .
:alice :parentOf :bob .

# Infer:
:alice :ancestorOf :bob .
```

---

#### **rdfs8: Class to Resource Subclass**

**Rule:**
```
IF   ?x rdf:type rdfs:Class .
THEN ?x rdfs:subClassOf rdfs:Resource .
```

**Rust Implementation:**
```rust
pub struct Rdfs8Rule;

impl InferenceRule for Rdfs8Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        let pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Iri(RDF_TYPE),
            NodePattern::Iri(RDFS_CLASS),
            NodePattern::DefaultGraph,
        );

        for quad in graph.find(&pattern) {
            let class = quad.subject();

            // Infer: ?x rdfs:subClassOf rdfs:Resource
            let inferred = Triple::new(
                class.clone(),
                Node::iri(RDFS_SUB_CLASS_OF),
                Node::iri(RDFS_RESOURCE)
            );
            ctx.infer(inferred);
        }
    }
}
```

**Example:**
```turtle
# Given:
:Person rdf:type rdfs:Class .

# Infer:
:Person rdfs:subClassOf rdfs:Resource .
```

---

#### **rdfs9: SubClass Implication**

**Rule:**
```
IF   ?x rdfs:subClassOf ?y .
     ?z rdf:type ?x .
THEN ?z rdf:type ?y .
```

**Rust Implementation:**
```rust
pub struct Rdfs9Rule;

impl InferenceRule for Rdfs9Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        // Pattern: (?x rdfs:subClassOf ?y)
        let subclass_pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Iri(RDFS_SUB_CLASS_OF),
            NodePattern::Variable("y"),
            NodePattern::DefaultGraph,
        );

        for subclass_quad in graph.find(&subclass_pattern) {
            let subclass = subclass_quad.subject();
            let superclass = subclass_quad.object();

            // Pattern: (?z rdf:type ?x)
            let instance_pattern = QuadPattern::new(
                NodePattern::Variable("z"),
                NodePattern::Iri(RDF_TYPE),
                NodePattern::Concrete(subclass.clone()),
                NodePattern::DefaultGraph,
            );

            for instance_quad in graph.find(&instance_pattern) {
                let instance = instance_quad.subject();

                // Infer: ?z rdf:type ?y
                let inferred = Triple::new(
                    instance.clone(),
                    Node::iri(RDF_TYPE),
                    superclass.clone()
                );
                ctx.infer(inferred);
            }
        }
    }
}
```

**Example:**
```turtle
# Given:
:Employee rdfs:subClassOf :Person .
:alice rdf:type :Employee .

# Infer:
:alice rdf:type :Person .
```

---

#### **rdfs10: Class Reflexivity**

**Rule:**
```
IF   ?x rdf:type rdfs:Class .
THEN ?x rdfs:subClassOf ?x .
```

**Rust Implementation:**
```rust
pub struct Rdfs10Rule;

impl InferenceRule for Rdfs10Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        let pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Iri(RDF_TYPE),
            NodePattern::Iri(RDFS_CLASS),
            NodePattern::DefaultGraph,
        );

        for quad in graph.find(&pattern) {
            let class = quad.subject();

            // Infer: ?x rdfs:subClassOf ?x
            let inferred = Triple::new(
                class.clone(),
                Node::iri(RDFS_SUB_CLASS_OF),
                class.clone()
            );
            ctx.infer(inferred);
        }
    }
}
```

**Example:**
```turtle
# Given:
:Person rdf:type rdfs:Class .

# Infer:
:Person rdfs:subClassOf :Person .
```

---

#### **rdfs11: SubClass Transitivity**

**Rule:**
```
IF   ?x rdfs:subClassOf ?y .
     ?y rdfs:subClassOf ?z .
THEN ?x rdfs:subClassOf ?z .
```

**Rust Implementation:**
```rust
pub struct Rdfs11Rule;

impl InferenceRule for Rdfs11Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        let mut closure = TransitiveClosure::new();

        // Pattern: (?x rdfs:subClassOf ?y)
        let pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Iri(RDFS_SUB_CLASS_OF),
            NodePattern::Variable("y"),
            NodePattern::DefaultGraph,
        );

        // Collect direct subClassOf relations
        for quad in graph.find(&pattern) {
            closure.add_edge(quad.subject(), quad.object());
        }

        // Compute transitive closure
        closure.compute();

        // Emit inferred triples
        for (from, to) in closure.inferred_edges() {
            let inferred = Triple::new(
                from.clone(),
                Node::iri(RDFS_SUB_CLASS_OF),
                to.clone()
            );
            ctx.infer(inferred);
        }
    }
}
```

**Example:**
```turtle
# Given:
:Manager rdfs:subClassOf :Employee .
:Employee rdfs:subClassOf :Person .

# Infer:
:Manager rdfs:subClassOf :Person .
```

---

#### **rdfs12: Container Membership Property**

**Rule:**
```
IF   ?x rdf:type rdfs:ContainerMembershipProperty .
THEN ?x rdfs:subPropertyOf rdfs:member .
```

**Rust Implementation:**
```rust
pub struct Rdfs12Rule;

impl InferenceRule for Rdfs12Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        let pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Iri(RDF_TYPE),
            NodePattern::Iri(RDFS_CONTAINER_MEMBERSHIP_PROPERTY),
            NodePattern::DefaultGraph,
        );

        for quad in graph.find(&pattern) {
            let property = quad.subject();

            // Infer: ?x rdfs:subPropertyOf rdfs:member
            let inferred = Triple::new(
                property.clone(),
                Node::iri(RDFS_SUB_PROPERTY_OF),
                Node::iri(RDFS_MEMBER)
            );
            ctx.infer(inferred);
        }
    }
}
```

**Example:**
```turtle
# Given:
rdf:_1 rdf:type rdfs:ContainerMembershipProperty .

# Infer:
rdf:_1 rdfs:subPropertyOf rdfs:member .
```

---

#### **rdfs13: Datatype Subclass of Literal**

**Rule:**
```
IF   ?x rdf:type rdfs:Datatype .
THEN ?x rdfs:subClassOf rdfs:Literal .
```

**Rust Implementation:**
```rust
pub struct Rdfs13Rule;

impl InferenceRule for Rdfs13Rule {
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>) {
        let pattern = QuadPattern::new(
            NodePattern::Variable("x"),
            NodePattern::Iri(RDF_TYPE),
            NodePattern::Iri(RDFS_DATATYPE),
            NodePattern::DefaultGraph,
        );

        for quad in graph.find(&pattern) {
            let datatype = quad.subject();

            // Infer: ?x rdfs:subClassOf rdfs:Literal
            let inferred = Triple::new(
                datatype.clone(),
                Node::iri(RDFS_SUB_CLASS_OF),
                Node::iri(RDFS_LITERAL)
            );
            ctx.infer(inferred);
        }
    }
}
```

**Example:**
```turtle
# Given:
xsd:string rdf:type rdfs:Datatype .

# Infer:
xsd:string rdfs:subClassOf rdfs:Literal .
```

---

### RDFS Reasoner Configuration

Apache Jena provides three compliance levels:

#### **1. RDFS Full**
- All 13 rules enabled
- Complete RDFS entailment
- Includes all RDF/RDFS axioms
- **Excludes:** bNode closure rules (non-finitary)

#### **2. RDFS Default**
- Most rules enabled
- **Omits:** rdfs1 (datatype recognition), rdfs4a/rdfs4b (resource typing)
- Skips container membership checks
- Faster than Full, suitable for most use cases

#### **3. RDFS Simple**
- Only transitive closure rules (rdfs5, rdfs7, rdfs9, rdfs11)
- Domain/range inference (rdfs2, rdfs3)
- No axiom assertions
- Fastest, minimal reasoning

---

### Incremental RDFS Reasoning

For mobile performance, implement incremental reasoning:

```rust
pub struct IncrementalRdfsReasoner {
    // Materialized inferences
    inferred_triples: HashSet<Triple<'static>>,

    // Dependency tracking for delta maintenance
    domain_index: HashMap<IriRef<'static>, Vec<IriRef<'static>>>,
    range_index: HashMap<IriRef<'static>, Vec<IriRef<'static>>>,
    subclass_closure: TransitiveClosure,
    subproperty_closure: TransitiveClosure,
}

impl IncrementalRdfsReasoner {
    /// Process a single added triple
    pub fn add_triple(&mut self, triple: &Triple) {
        match (triple.predicate().as_iri(), triple.object().as_iri()) {
            // Track domain assertions
            (Some(pred), _) if pred.as_str() == RDFS_DOMAIN => {
                self.domain_index
                    .entry(triple.subject().clone())
                    .or_default()
                    .push(triple.object().clone());
            }

            // Track range assertions
            (Some(pred), _) if pred.as_str() == RDFS_RANGE => {
                self.range_index
                    .entry(triple.subject().clone())
                    .or_default()
                    .push(triple.object().clone());
            }

            // Update subclass closure
            (Some(pred), Some(obj)) if pred.as_str() == RDFS_SUB_CLASS_OF => {
                self.subclass_closure.add_edge(triple.subject(), obj);
                self.materialize_subclass_implications(triple.subject());
            }

            // Apply domain/range rules on property usage
            _ => {
                self.apply_domain_range_rules(triple);
            }
        }
    }

    /// Remove a triple and retract dependent inferences
    pub fn remove_triple(&mut self, triple: &Triple) {
        // Truth maintenance: retract inferences that depend on this triple
        self.retract_dependent_inferences(triple);
    }
}
```

---

## OWL 2 Profile Reasoners

### OWL 2 EL Profile

**Computational Complexity:** PTIME
**Use Case:** Large ontologies with many classes/properties (biomedical, enterprise)

#### Key Features

- Existential quantification (`ObjectSomeValuesFrom`)
- Class intersection (`ObjectIntersectionOf`)
- Property chains (`SubObjectPropertyOf` with chain)
- Reflexive properties
- No universal quantification, negation, or disjunction

#### Core Rules (EL Fragment)

```rust
pub struct Owl2ElReasoner {
    subclass_closure: TransitiveClosure,
    existential_cache: HashMap<(IriRef, IriRef), Vec<IriRef>>, // (property, filler) -> classes
}

impl Owl2ElReasoner {
    /// EL Rule: Existential Elimination
    /// IF   ?x rdf:type ?C .
    ///      ?C owl:equivalentClass [ owl:someValuesFrom ?D ; owl:onProperty ?P ] .
    ///      ?x ?P ?y .
    /// THEN ?y rdf:type ?D .
    pub fn apply_existential_elimination(&mut self, graph: &QuadStore) {
        // Implementation
    }

    /// EL Rule: Intersection Subsumption
    /// IF   ?C owl:equivalentClass [ owl:intersectionOf (?D1 ?D2) ] .
    ///      ?x rdf:type ?C .
    /// THEN ?x rdf:type ?D1 .
    ///      ?x rdf:type ?D2 .
    pub fn apply_intersection_subsumption(&mut self, graph: &QuadStore) {
        // Implementation
    }

    /// EL Rule: Property Chain
    /// IF   ?P owl:propertyChainAxiom (?P1 ?P2) .
    ///      ?x ?P1 ?y .
    ///      ?y ?P2 ?z .
    /// THEN ?x ?P ?z .
    pub fn apply_property_chain(&mut self, graph: &QuadStore) {
        // Implementation
    }
}
```

#### Mobile Optimization for EL

- **Precompute TBox:** Classification done once, reused for all ABoxes
- **Lazy Evaluation:** Only compute property chains on-demand
- **Compact Storage:** Use integer IDs for classes/properties

---

### OWL 2 QL Profile

**Computational Complexity:** AC0 in data, NLogSpace for taxonomy
**Use Case:** Query answering over large databases (OBDA - Ontology-Based Data Access)

#### Key Features

- Designed for query rewriting to SQL
- Limited expressiveness to maintain first-order rewritability
- No cardinality restrictions
- No property chains
- No individual equality (`owl:sameAs`)

#### Query Rewriting Approach

```rust
pub struct Owl2QlQueryRewriter {
    subclass_index: HashMap<IriRef, Vec<IriRef>>,
    subproperty_index: HashMap<IriRef, Vec<IriRef>>,
    domain_index: HashMap<IriRef, Vec<IriRef>>,
    range_index: HashMap<IriRef, Vec<IriRef>>,
}

impl Owl2QlQueryRewriter {
    /// Rewrite SPARQL query to account for ontology
    /// Example:
    ///   Original: SELECT ?x WHERE { ?x rdf:type :Manager }
    ///   Rewritten: SELECT ?x WHERE {
    ///       { ?x rdf:type :Manager } UNION
    ///       { ?x rdf:type :SeniorManager } UNION
    ///       { ?x rdf:type :CEO }
    ///   }
    pub fn rewrite_query(&self, query: &SparqlQuery) -> Vec<SparqlQuery> {
        let mut rewritten = Vec::new();

        for bgp in query.basic_graph_patterns() {
            for triple_pattern in bgp.patterns() {
                if let Some(type_pattern) = self.is_type_pattern(triple_pattern) {
                    // Expand to subclasses
                    let expanded = self.expand_class_hierarchy(type_pattern.class);
                    rewritten.extend(expanded);
                } else if let Some(prop_pattern) = self.is_property_pattern(triple_pattern) {
                    // Expand to sub-properties
                    let expanded = self.expand_property_hierarchy(prop_pattern.property);
                    rewritten.extend(expanded);
                }
            }
        }

        rewritten
    }
}
```

---

### OWL 2 RL Profile

**Computational Complexity:** PTIME for consistency, co-NP-complete for subsumption
**Use Case:** Rule-based reasoning with good expressiveness-performance balance

#### Key Features

- Implementable via rule engines (like Jena's RETE)
- Supports property chains, functional properties
- Permits most OWL 2 constructs
- Good for forward-chaining materialization

#### Complete OWL 2 RL Rules

The W3C OWL 2 RL/RDF specification defines rules in 9 tables:

**Table 4: Equality Semantics (7 rules)**
- `eq-ref`: Reflexivity of `owl:sameAs`
- `eq-sym`: Symmetry of `owl:sameAs`
- `eq-trans`: Transitivity of `owl:sameAs`
- `eq-rep-s`: Replace subject in triples
- `eq-rep-p`: Replace predicate in triples
- `eq-rep-o`: Replace object in triples
- `eq-diff1`: Disjointness of `owl:differentFrom`

**Table 5: Property Axioms (16 rules)**
- `prp-dom`: Domain inference
- `prp-rng`: Range inference
- `prp-fp`: Functional property
- `prp-ifp`: Inverse functional property
- `prp-irp`: Irreflexive property violation
- `prp-symp`: Symmetric property
- `prp-asyp`: Asymmetric property violation
- `prp-trp`: Transitive property
- `prp-spo1`: Sub-property inference
- `prp-spo2`: Property chain (2-element)
- `prp-eqp1/eqp2`: Equivalent properties
- `prp-pdw`: Property disjointness violation
- `prp-inv1/inv2`: Inverse properties
- `prp-key`: Key properties
- `prp-npa1/npa2`: Negative property assertions

**Table 6: Class Semantics (22 rules)**
- `cls-int1/int2`: Intersection
- `cls-uni`: Union
- `cls-com`: Complement (inconsistency)
- `cls-svf1/svf2`: `someValuesFrom`
- `cls-avf`: `allValuesFrom`
- `cls-hv1/hv2`: `hasValue`
- `cls-maxc1/maxc2`: `maxCardinality`
- `cls-maxqc1/maxqc2/maxqc3/maxqc4`: `maxQualifiedCardinality`
- `cls-oo`: `oneOf` (enumeration)

**Table 7: Class Axioms (11 rules)**
- `cax-sco`: Subclass implication
- `cax-eqc1/eqc2`: Equivalent classes
- `cax-dw`: Class disjointness violation
- `cax-adc`: All disjoint classes violation

**Table 8-9: Datatypes & Schema (5 rules)**
- Datatype validation and literal range checks

#### Rust Implementation Skeleton

```rust
pub struct Owl2RlReasoner {
    rules: Vec<Box<dyn Owl2RlRule>>,
    inferred: HashSet<Triple<'static>>,
}

impl Owl2RlReasoner {
    pub fn new() -> Self {
        let mut rules: Vec<Box<dyn Owl2RlRule>> = vec![
            // Table 4: Equality
            Box::new(EqReflexivityRule),
            Box::new(EqSymmetryRule),
            Box::new(EqTransitivityRule),

            // Table 5: Properties
            Box::new(PropertyDomainRule),
            Box::new(PropertyRangeRule),
            Box::new(FunctionalPropertyRule),
            Box::new(TransitivePropertyRule),

            // Table 6: Classes
            Box::new(IntersectionRule),
            Box::new(UnionRule),
            Box::new(SomeValuesFromRule),

            // Table 7: Axioms
            Box::new(SubclassRule),
            Box::new(EquivalentClassRule),

            // ... all 61 rules
        ];

        Self {
            rules,
            inferred: HashSet::new(),
        }
    }

    /// Forward chaining until fixpoint
    pub fn materialize(&mut self, graph: &QuadStore) {
        let mut changed = true;
        let mut iteration = 0;

        while changed {
            changed = false;
            iteration += 1;

            for rule in &self.rules {
                let before_count = self.inferred.len();
                rule.apply(graph, &mut self.inferred);

                if self.inferred.len() > before_count {
                    changed = true;
                }
            }

            if iteration > MAX_ITERATIONS {
                panic!("Reasoning did not converge!");
            }
        }
    }
}

/// Base trait for all OWL 2 RL rules
pub trait Owl2RlRule {
    fn apply(&self, graph: &QuadStore, inferred: &mut HashSet<Triple<'static>>);
}

/// Example: prp-dom (Property Domain)
pub struct PropertyDomainRule;

impl Owl2RlRule for PropertyDomainRule {
    fn apply(&self, graph: &QuadStore, inferred: &mut HashSet<Triple<'static>>) {
        // IF   ?p rdfs:domain ?c .
        //      ?x ?p ?y .
        // THEN ?x rdf:type ?c .

        let domain_pattern = QuadPattern::new(
            NodePattern::Variable("p"),
            NodePattern::Iri(RDFS_DOMAIN),
            NodePattern::Variable("c"),
            NodePattern::DefaultGraph,
        );

        for domain_quad in graph.find(&domain_pattern) {
            let property = domain_quad.subject();
            let class = domain_quad.object();

            let usage_pattern = QuadPattern::new(
                NodePattern::Variable("x"),
                NodePattern::Concrete(property.clone()),
                NodePattern::Variable("y"),
                NodePattern::DefaultGraph,
            );

            for usage_quad in graph.find(&usage_pattern) {
                let triple = Triple::new(
                    usage_quad.subject().clone(),
                    Node::iri(RDF_TYPE),
                    class.clone()
                );
                inferred.insert(triple);
            }
        }
    }
}
```

---

## Generic Rule Engine

### Rule Language Syntax

Apache Jena's rule syntax (adapted for Rust):

```
Rule := [ruleName: body -> head]

body := term, ... term
head := term, ... term

term := (node, node, node)           // Triple pattern
      | builtin(node, ... node)      // Builtin function call
      | node == node                 // Equality test
      | node != node                 // Inequality test

node := <iri>                        // Concrete IRI
      | ?var                         // Variable
      | "literal"                    // String literal
      | 123                          // Integer literal
      | _:b0                         // Blank node
```

#### Example Rules

```rust
// RDFS rule: domain inference
[rdfs2:
    (?p rdfs:domain ?c), (?x ?p ?y)
    ->
    (?x rdf:type ?c)
]

// OWL rule: transitive property
[owl-transitive:
    (?p rdf:type owl:TransitiveProperty), (?x ?p ?y), (?y ?p ?z)
    ->
    (?x ?p ?z)
]

// Custom rule with builtin
[age-adult:
    (?person rdf:type :Person), (?person :age ?age), greaterThan(?age, 18)
    ->
    (?person rdf:type :Adult)
]
```

---

### Rule Grammar (Rust Implementation)

```rust
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum RuleToken {
    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("->")]
    Arrow,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[regex(r"\?[a-zA-Z_][a-zA-Z0-9_]*")]
    Variable,

    #[regex(r"<[^>]+>")]
    Iri,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*:[a-zA-Z_][a-zA-Z0-9_]*")]
    PrefixedName,

    #[regex(r#""[^"]*""#)]
    StringLiteral,

    #[regex(r"[0-9]+")]
    IntegerLiteral,

    #[regex(r"_:[a-zA-Z0-9]+")]
    BlankNode,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

pub struct RuleParser {
    tokens: Vec<RuleToken>,
    pos: usize,
}

impl RuleParser {
    pub fn parse(&mut self) -> Result<Rule, ParseError> {
        self.expect(RuleToken::LeftBracket)?;

        let name = self.parse_rule_name()?;

        self.expect(RuleToken::Colon)?;

        let body = self.parse_body()?;

        self.expect(RuleToken::Arrow)?;

        let head = self.parse_head()?;

        self.expect(RuleToken::RightBracket)?;

        Ok(Rule { name, body, head })
    }
}
```

---

### Rule Execution Engine

```rust
pub struct RuleEngine {
    rules: Vec<Rule>,
    builtins: HashMap<String, Box<dyn BuiltinFunction>>,
}

impl RuleEngine {
    /// Execute rules in forward chaining mode
    pub fn forward_chain(&self, graph: &QuadStore) -> HashSet<Triple<'static>> {
        let mut inferred = HashSet::new();
        let mut changed = true;

        while changed {
            changed = false;

            for rule in &self.rules {
                let bindings = self.match_body(&rule.body, graph, &inferred);

                for binding in bindings {
                    for head_atom in &rule.head {
                        if let Some(triple) = self.instantiate_head(head_atom, &binding) {
                            if inferred.insert(triple) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        inferred
    }

    /// Execute rules in backward chaining mode (goal-driven)
    pub fn backward_chain(&self, goal: &Triple, graph: &QuadStore) -> bool {
        // Check if goal is in base data
        if self.is_in_graph(goal, graph) {
            return true;
        }

        // Try to prove via rules
        for rule in &self.rules {
            for head_atom in &rule.head {
                if let Some(binding) = self.unify(head_atom, goal) {
                    // Try to prove all body atoms with this binding
                    if self.prove_body(&rule.body, &binding, graph) {
                        return true;
                    }
                }
            }
        }

        false
    }
}
```

---

## RETE Algorithm Implementation

### Overview

RETE (Latin for "network") is an efficient pattern matching algorithm for production systems. It optimizes rule execution by:

1. **Sharing Computation**: Common patterns evaluated once
2. **Incremental Updates**: Only affected rules re-evaluated on changes
3. **State Preservation**: Intermediate results cached in network nodes

### RETE Network Structure

```
        [Root]
           |
    [Type Nodes] (?x rdf:type ?type)
           |
    [Alpha Nodes] (filters on single patterns)
           |
    [Beta Nodes] (joins multiple patterns)
           |
    [Terminal Nodes] (rule activations)
```

### Rust Implementation

```rust
pub struct ReteNetwork {
    root: RootNode,
    alpha_memory: HashMap<PatternHash, Vec<WorkingMemoryElement>>,
    beta_memory: HashMap<JoinHash, Vec<Token>>,
    agenda: VecDeque<Activation>,
}

/// Working memory element (fact)
pub struct WorkingMemoryElement {
    triple: Triple<'static>,
    timestamp: u64,
}

/// Token (partial match)
pub struct Token {
    wmes: Vec<WorkingMemoryElement>,
    binding: HashMap<Variable, Node<'static>>,
}

/// Rule activation
pub struct Activation {
    rule: Rule,
    token: Token,
}

impl ReteNetwork {
    /// Add a triple to the network
    pub fn assert_triple(&mut self, triple: Triple<'static>) {
        let wme = WorkingMemoryElement {
            triple: triple.clone(),
            timestamp: self.next_timestamp(),
        };

        // Propagate through alpha network
        for alpha_node in self.matching_alpha_nodes(&triple) {
            alpha_node.activate(&wme);

            // Store in alpha memory
            self.alpha_memory
                .entry(alpha_node.pattern_hash())
                .or_default()
                .push(wme.clone());

            // Propagate to beta network
            for beta_node in alpha_node.successors() {
                self.propagate_to_beta(beta_node, &wme);
            }
        }
    }

    /// Remove a triple from the network
    pub fn retract_triple(&mut self, triple: &Triple) {
        // Find all WMEs matching this triple
        let wmes_to_remove: Vec<_> = self.alpha_memory
            .values()
            .flatten()
            .filter(|wme| &wme.triple == triple)
            .cloned()
            .collect();

        // Retract each WME
        for wme in wmes_to_remove {
            self.retract_wme(&wme);
        }
    }

    /// Propagate WME to beta nodes (join logic)
    fn propagate_to_beta(&mut self, beta_node: &BetaNode, wme: &WorkingMemoryElement) {
        // Retrieve tokens from left input
        let left_tokens = self.beta_memory
            .get(&beta_node.left_input_hash())
            .cloned()
            .unwrap_or_default();

        // Perform join
        for left_token in left_tokens {
            if let Some(joined_token) = self.join_token(&left_token, wme, &beta_node.join_tests) {
                // Store in beta memory
                self.beta_memory
                    .entry(beta_node.hash())
                    .or_default()
                    .push(joined_token.clone());

                // If this is a terminal node, create activation
                if let Some(rule) = beta_node.rule() {
                    let activation = Activation {
                        rule: rule.clone(),
                        token: joined_token,
                    };
                    self.agenda.push_back(activation);
                }
            }
        }
    }

    /// Join a token with a WME
    fn join_token(
        &self,
        token: &Token,
        wme: &WorkingMemoryElement,
        join_tests: &[JoinTest]
    ) -> Option<Token> {
        let mut new_binding = token.binding.clone();

        // Check all join conditions
        for test in join_tests {
            match test {
                JoinTest::VariableBinding { var, node_position } => {
                    let node = wme.triple.get_node(*node_position);

                    if let Some(existing) = new_binding.get(var) {
                        // Variable already bound, must match
                        if existing != node {
                            return None;
                        }
                    } else {
                        // Bind variable
                        new_binding.insert(var.clone(), node.clone());
                    }
                }

                JoinTest::Equality { left, right } => {
                    let left_val = self.evaluate_expression(left, token, wme);
                    let right_val = self.evaluate_expression(right, token, wme);

                    if left_val != right_val {
                        return None;
                    }
                }
            }
        }

        // All tests passed, create new token
        let mut new_wmes = token.wmes.clone();
        new_wmes.push(wme.clone());

        Some(Token {
            wmes: new_wmes,
            binding: new_binding,
        })
    }

    /// Execute the next activation in the agenda
    pub fn fire_next_rule(&mut self) -> Option<Vec<Triple<'static>>> {
        let activation = self.agenda.pop_front()?;

        let mut results = Vec::new();

        for head_atom in &activation.rule.head {
            if let Some(triple) = self.instantiate_head(head_atom, &activation.token.binding) {
                results.push(triple);
            }
        }

        Some(results)
    }
}
```

### RETE Optimization Techniques

#### 1. Node Sharing
```rust
// Alpha nodes are shared when patterns match
let pattern1 = TriplePattern::new(Var("x"), IRI(RDF_TYPE), Var("type"));
let pattern2 = TriplePattern::new(Var("y"), IRI(RDF_TYPE), IRI(PERSON));

// Both use same alpha node for rdf:type predicate
```

#### 2. Hash-based Indexing
```rust
pub struct AlphaNode {
    pattern: TriplePattern,
    hash: u64, // Precomputed hash for fast lookup
    successors: Vec<BetaNodeRef>,
}

impl AlphaNode {
    fn matches(&self, triple: &Triple) -> bool {
        // Fast hash comparison first
        if self.hash != triple.pattern_hash() {
            return false;
        }

        // Detailed pattern matching only if hash matches
        self.pattern.matches(triple)
    }
}
```

#### 3. Right-Unlinking
```rust
// Disable beta nodes with empty left input
impl BetaNode {
    fn should_evaluate(&self, network: &ReteNetwork) -> bool {
        let left_tokens = network.beta_memory.get(&self.left_input_hash());

        !left_tokens.map_or(true, |tokens| tokens.is_empty())
    }
}
```

---

## Transitive Reasoner

### Purpose

Efficiently compute transitive closures for:
- `rdfs:subClassOf` (class hierarchy)
- `rdfs:subPropertyOf` (property hierarchy)
- Custom transitive properties (e.g., `ancestorOf`)

### Implementation Strategies

#### 1. Floyd-Warshall Algorithm

Best for small, dense graphs (< 1000 nodes).

```rust
pub struct FloydWarshallClosure {
    nodes: Vec<Node<'static>>,
    adjacency: Vec<Vec<bool>>, // adjacency[i][j] = true if edge i->j
}

impl FloydWarshallClosure {
    pub fn compute(&mut self) {
        let n = self.nodes.len();

        // Floyd-Warshall: for each intermediate node k
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    if self.adjacency[i][k] && self.adjacency[k][j] {
                        self.adjacency[i][j] = true;
                    }
                }
            }
        }
    }

    pub fn inferred_edges(&self) -> impl Iterator<Item = (&Node, &Node)> {
        self.adjacency.iter().enumerate().flat_map(|(i, row)| {
            row.iter().enumerate().filter_map(move |(j, &connected)| {
                if connected && i != j {
                    Some((&self.nodes[i], &self.nodes[j]))
                } else {
                    None
                }
            })
        })
    }
}
```

**Complexity:** O(n³) time, O(n²) space

#### 2. Warshall's Algorithm (Optimized)

```rust
pub struct WarshallClosure {
    nodes: Vec<Node<'static>>,
    reachability: BitVec, // Packed bit matrix
}

impl WarshallClosure {
    pub fn compute(&mut self) {
        let n = self.nodes.len();

        for k in 0..n {
            for i in 0..n {
                if self.get(i, k) {
                    for j in 0..n {
                        if self.get(k, j) {
                            self.set(i, j);
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn get(&self, i: usize, j: usize) -> bool {
        self.reachability[i * self.nodes.len() + j]
    }

    #[inline]
    fn set(&mut self, i: usize, j: usize) {
        self.reachability.set(i * self.nodes.len() + j, true);
    }
}
```

**Optimization:** Uses bit-packing for 64x memory reduction.

#### 3. Graph Traversal (BFS/DFS)

Best for sparse graphs or on-demand queries.

```rust
pub struct LazyTransitiveClosure {
    adjacency_list: HashMap<Node<'static>, Vec<Node<'static>>>,
    cache: RwLock<HashMap<Node<'static>, HashSet<Node<'static>>>>,
}

impl LazyTransitiveClosure {
    /// Compute reachable nodes from `start` (memoized)
    pub fn reachable_from(&self, start: &Node) -> HashSet<Node<'static>> {
        // Check cache
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.get(start) {
                return cached.clone();
            }
        }

        // BFS traversal
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start.clone());
        visited.insert(start.clone());

        while let Some(node) = queue.pop_front() {
            if let Some(neighbors) = self.adjacency_list.get(&node) {
                for neighbor in neighbors {
                    if visited.insert(neighbor.clone()) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        // Cache result
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(start.clone(), visited.clone());
        }

        visited
    }
}
```

**Complexity:** O(V + E) per query, O(V²) space for full cache

#### 4. Topological Sort with Levels

Best for DAG hierarchies (class/property taxonomies).

```rust
pub struct TopologicalClosure {
    levels: Vec<Vec<Node<'static>>>, // Nodes grouped by topological level
    parent_map: HashMap<Node<'static>, Vec<Node<'static>>>,
}

impl TopologicalClosure {
    pub fn compute(&mut self) {
        // Topological sort to determine levels
        let sorted = self.topological_sort();

        // Group by level
        self.levels = self.group_by_levels(&sorted);

        // Precompute all ancestors for each node
        for level in &self.levels {
            for node in level {
                let ancestors = self.collect_ancestors(node);
                self.parent_map.insert(node.clone(), ancestors);
            }
        }
    }

    fn collect_ancestors(&self, node: &Node) -> Vec<Node<'static>> {
        let mut ancestors = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(node.clone());

        while let Some(current) = queue.pop_front() {
            if let Some(parents) = self.parent_map.get(&current) {
                for parent in parents {
                    if !ancestors.contains(parent) {
                        ancestors.push(parent.clone());
                        queue.push_back(parent.clone());
                    }
                }
            }
        }

        ancestors
    }
}
```

---

### Transitive Graph Cache (TGC)

Apache Jena's optimized structure for hierarchies:

```rust
pub struct TransitiveGraphCache {
    // Direct edges (asserted)
    direct_edges: HashMap<Node<'static>, SmallVec<[Node<'static>; 4]>>,

    // Transitive closure (inferred)
    closure: HashMap<Node<'static>, Arc<HashSet<Node<'static>>>>,

    // Reverse index for fast subsumption checks
    reverse_index: HashMap<Node<'static>, Arc<HashSet<Node<'static>>>>,
}

impl TransitiveGraphCache {
    /// Check if `a` is subsumed by `b` (i.e., a rdfs:subClassOf* b)
    pub fn is_subsumed_by(&self, a: &Node, b: &Node) -> bool {
        if a == b {
            return true; // Reflexive
        }

        if let Some(ancestors) = self.closure.get(a) {
            ancestors.contains(b)
        } else {
            false
        }
    }

    /// Get all direct subclasses of `node`
    pub fn direct_subclasses(&self, node: &Node) -> &[Node<'static>] {
        self.reverse_index
            .get(node)
            .map(|set| set.as_slice())
            .unwrap_or(&[])
    }

    /// Get all transitive subclasses of `node`
    pub fn all_subclasses(&self, node: &Node) -> HashSet<Node<'static>> {
        let mut result = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(node.clone());

        while let Some(current) = queue.pop_front() {
            if let Some(direct_subs) = self.reverse_index.get(&current) {
                for sub in direct_subs.iter() {
                    if result.insert(sub.clone()) {
                        queue.push_back(sub.clone());
                    }
                }
            }
        }

        result
    }

    /// Incremental update: add edge and update closure
    pub fn add_edge(&mut self, from: Node<'static>, to: Node<'static>) {
        // Add direct edge
        self.direct_edges
            .entry(from.clone())
            .or_default()
            .push(to.clone());

        // Update transitive closure incrementally
        self.incremental_update(from, to);
    }

    fn incremental_update(&mut self, from: Node<'static>, to: Node<'static>) {
        // All nodes that can reach `from` can now reach `to` and its ancestors
        let from_ancestors = self.closure.get(&to).cloned().unwrap_or_default();

        let to_update: Vec<_> = self.reverse_index
            .iter()
            .filter(|(_, children)| children.contains(&from))
            .map(|(parent, _)| parent.clone())
            .collect();

        for node in to_update {
            let closure = Arc::make_mut(self.closure.entry(node).or_default());
            closure.insert(to.clone());
            closure.extend(from_ancestors.iter().cloned());
        }
    }
}
```

---

## Rust Architecture Design

### Module Structure

```
crates/reasoning/
├── src/
│   ├── lib.rs                    # Public API
│   ├── reasoner.rs               # Reasoner trait
│   ├── context.rs                # Inference context
│   │
│   ├── rdfs/
│   │   ├── mod.rs
│   │   ├── rules.rs              # All 13 RDFS rules
│   │   ├── reasoner.rs           # RDFS reasoner implementation
│   │   └── incremental.rs        # Incremental RDFS reasoning
│   │
│   ├── owl/
│   │   ├── mod.rs
│   │   ├── el.rs                 # OWL 2 EL profile
│   │   ├── ql.rs                 # OWL 2 QL profile
│   │   ├── rl.rs                 # OWL 2 RL profile
│   │   └── rules.rs              # OWL 2 RL rule implementations
│   │
│   ├── rule_engine/
│   │   ├── mod.rs
│   │   ├── parser.rs             # Rule syntax parser
│   │   ├── executor.rs           # Forward/backward chaining
│   │   ├── builtins.rs           # Builtin functions
│   │   └── rete.rs               # RETE algorithm
│   │
│   ├── transitive/
│   │   ├── mod.rs
│   │   ├── closure.rs            # Transitive closure algorithms
│   │   ├── cache.rs              # Transitive graph cache
│   │   └── reasoner.rs           # Transitive reasoner
│   │
│   └── validation/
│       ├── mod.rs
│       └── validator.rs          # Constraint validation
│
└── tests/
    ├── rdfs_tests.rs             # RDFS conformance tests
    ├── owl_tests.rs              # OWL 2 profile tests
    ├── rule_engine_tests.rs      # Rule execution tests
    └── benchmarks.rs             # Performance benchmarks
```

---

### Core Traits

```rust
/// Base trait for all reasoners
pub trait Reasoner {
    /// Compute inferences for the given graph
    fn infer<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>);

    /// Check if a triple is entailed (without materializing all inferences)
    fn entails(&self, graph: &QuadStore, triple: &Triple) -> bool;

    /// Validate the graph for inconsistencies
    fn validate(&self, graph: &QuadStore) -> ValidationResult;

    /// Get reasoner configuration
    fn config(&self) -> &ReasonerConfig;
}

/// Inference context for collecting inferred triples
pub struct InferenceContext<'a> {
    inferred: HashSet<Triple<'a>>,
    variables: HashMap<Variable<'a>, Node<'a>>,
}

impl<'a> InferenceContext<'a> {
    pub fn infer(&mut self, triple: Triple<'a>) {
        self.inferred.insert(triple);
    }

    pub fn bind(&mut self, var: Variable<'a>, value: Node<'a>) {
        self.variables.insert(var, value);
    }

    pub fn get_binding(&self, var: &Variable<'a>) -> Option<&Node<'a>> {
        self.variables.get(var)
    }

    pub fn inferred_triples(&self) -> &HashSet<Triple<'a>> {
        &self.inferred
    }
}

/// Individual inference rule
pub trait InferenceRule {
    fn name(&self) -> &'static str;
    fn apply<'a>(&self, graph: &QuadStore, ctx: &mut InferenceContext<'a>);
}

/// Configuration for reasoners
#[derive(Clone, Debug)]
pub struct ReasonerConfig {
    pub profile: ReasonerProfile,
    pub forward_chain: bool,
    pub backward_chain: bool,
    pub max_iterations: usize,
    pub enable_validation: bool,
    pub cache_inferences: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReasonerProfile {
    RdfsFull,
    RdfsDefault,
    RdfsSimple,
    Owl2El,
    Owl2Ql,
    Owl2Rl,
    Custom(Vec<String>), // Custom rule names
}
```

---

### Zero-Copy Design

All reasoning operations use borrowed references:

```rust
pub struct ZeroCopyReasoner<'a> {
    graph: &'a QuadStore,
    dict: &'a Dictionary,
    rules: Vec<Box<dyn InferenceRule>>,

    // Arena for temporary allocations during reasoning
    arena: &'a Bump,
}

impl<'a> ZeroCopyReasoner<'a> {
    pub fn new(
        graph: &'a QuadStore,
        dict: &'a Dictionary,
        arena: &'a Bump,
        config: ReasonerConfig
    ) -> Self {
        let rules = Self::load_rules(&config);

        Self {
            graph,
            dict,
            rules,
            arena,
        }
    }

    pub fn materialize(&self) -> InferenceGraph<'a> {
        let mut ctx = InferenceContext::new();

        for rule in &self.rules {
            rule.apply(self.graph, &mut ctx);
        }

        InferenceGraph {
            base: self.graph,
            inferred: ctx.inferred_triples().clone(),
        }
    }
}

/// Virtual graph combining base + inferred triples
pub struct InferenceGraph<'a> {
    base: &'a QuadStore,
    inferred: HashSet<Triple<'a>>,
}

impl<'a> InferenceGraph<'a> {
    pub fn contains(&self, triple: &Triple) -> bool {
        self.base.contains(triple) || self.inferred.contains(triple)
    }

    pub fn find(&self, pattern: &QuadPattern) -> impl Iterator<Item = Quad<'a>> + '_ {
        self.base.find(pattern).chain(
            self.inferred
                .iter()
                .filter(move |t| pattern.matches_triple(t))
                .map(|t| Quad::from_triple(t.clone(), None))
        )
    }
}
```

---

### Visitor Pattern for Rules

```rust
pub trait TripleVisitor {
    fn visit_triple(&mut self, triple: &Triple);
}

pub struct RuleExecutionVisitor<'a> {
    rules: &'a [Box<dyn InferenceRule>],
    context: InferenceContext<'a>,
}

impl<'a> TripleVisitor for RuleExecutionVisitor<'a> {
    fn visit_triple(&mut self, triple: &Triple) {
        for rule in self.rules {
            // Check if rule is applicable to this triple
            if self.is_applicable(rule, triple) {
                rule.apply_to_triple(triple, &mut self.context);
            }
        }
    }
}

// Usage:
pub fn reason_incrementally(graph: &QuadStore, new_triples: &[Triple]) -> HashSet<Triple> {
    let mut visitor = RuleExecutionVisitor::new(&RDFS_RULES);

    for triple in new_triples {
        visitor.visit_triple(triple);
    }

    visitor.context.inferred_triples().clone()
}
```

---

## Mobile Optimization Strategies

### 1. Memory Budget

Target: < 100MB for reasoning engine

```rust
pub struct MobileReasonerConfig {
    /// Maximum memory for RETE network (MB)
    pub max_rete_memory: usize,

    /// Maximum cached inferences
    pub max_cached_inferences: usize,

    /// Use memory-mapped files for large graphs
    pub use_mmap: bool,

    /// Aggressive deduplication
    pub deduplicate_aggressively: bool,
}

impl MobileReasonerConfig {
    pub fn default() -> Self {
        Self {
            max_rete_memory: 50,  // 50MB for RETE
            max_cached_inferences: 100_000,
            use_mmap: true,
            deduplicate_aggressively: true,
        }
    }
}
```

### 2. Lazy Evaluation

Don't materialize all inferences upfront:

```rust
pub struct LazyReasoner {
    graph: Arc<QuadStore>,
    rules: Vec<Box<dyn InferenceRule>>,

    // LRU cache for recently computed inferences
    cache: Mutex<LruCache<Triple<'static>, bool>>,
}

impl LazyReasoner {
    /// Check entailment without full materialization
    pub fn entails(&self, triple: &Triple) -> bool {
        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(&result) = cache.get(triple) {
                return result;
            }
        }

        // Check base graph
        if self.graph.contains(triple) {
            return true;
        }

        // Try to derive via rules (backward chaining)
        let result = self.can_derive(triple);

        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(triple.clone(), result);
        }

        result
    }
}
```

### 3. Incremental Reasoning

Only recompute affected inferences:

```rust
pub struct IncrementalReasoner {
    base_graph: Arc<QuadStore>,
    inferred: Arc<RwLock<HashSet<Triple<'static>>>>,

    // Dependency graph for truth maintenance
    dependencies: Arc<RwLock<HashMap<Triple<'static>, Vec<Triple<'static>>>>>,
}

impl IncrementalReasoner {
    pub fn add_triple(&self, triple: Triple<'static>) {
        // Add to base graph
        self.base_graph.insert(Quad::from_triple(triple.clone(), None));

        // Compute only new inferences
        let new_inferences = self.compute_delta(&triple);

        // Update inferred set
        let mut inferred = self.inferred.write().unwrap();
        for inf in new_inferences {
            // Track dependency
            self.dependencies.write().unwrap()
                .entry(triple.clone())
                .or_default()
                .push(inf.clone());

            inferred.insert(inf);
        }
    }

    pub fn remove_triple(&self, triple: &Triple) {
        // Retract dependent inferences
        let deps = self.dependencies.read().unwrap()
            .get(triple)
            .cloned()
            .unwrap_or_default();

        let mut inferred = self.inferred.write().unwrap();
        for dep in deps {
            inferred.remove(&dep);
        }

        // Remove from base graph
        self.base_graph.remove(&Quad::from_triple(triple.clone(), None));
    }
}
```

### 4. Rule Pruning

Disable rules not relevant to data:

```rust
pub fn prune_rules(rules: &[Box<dyn InferenceRule>], graph: &QuadStore) -> Vec<Box<dyn InferenceRule>> {
    let mut relevant_rules = Vec::new();

    for rule in rules {
        // Check if rule's body patterns have potential matches
        if rule.has_potential_matches(graph) {
            relevant_rules.push(rule.clone());
        }
    }

    relevant_rules
}
```

### 5. Parallel Reasoning

Use Rayon for data parallelism:

```rust
use rayon::prelude::*;

pub fn parallel_forward_chain(graph: &QuadStore, rules: &[Box<dyn InferenceRule>]) -> HashSet<Triple<'static>> {
    let inferred: HashSet<_> = rules
        .par_iter()
        .flat_map(|rule| {
            let mut ctx = InferenceContext::new();
            rule.apply(graph, &mut ctx);
            ctx.inferred_triples().clone()
        })
        .collect();

    inferred
}
```

### 6. Profile-Based Optimization

```rust
pub struct AdaptiveReasoner {
    config: MobileReasonerConfig,
    profile: ReasonerProfile,

    // Runtime statistics
    stats: Arc<RwLock<ReasoningStats>>,
}

impl AdaptiveReasoner {
    pub fn optimize_for_battery(&mut self) {
        // Switch to simpler profile
        self.profile = ReasonerProfile::RdfsSimple;
        self.config.max_rete_memory = 20; // Reduce memory
    }

    pub fn optimize_for_performance(&mut self) {
        // Use full reasoning
        self.profile = ReasonerProfile::RdfsFull;
        self.config.max_rete_memory = 100;
    }

    pub fn adjust_based_on_stats(&mut self) {
        let stats = self.stats.read().unwrap();

        if stats.avg_inference_time > Duration::from_millis(10) {
            // Too slow, reduce complexity
            self.profile = ReasonerProfile::RdfsDefault;
        }

        if stats.memory_usage > self.config.max_rete_memory * 1024 * 1024 {
            // Memory pressure, clear cache
            self.clear_cache();
        }
    }
}
```

---

## Testing Requirements

### 1. RDFS Conformance Tests

All 13 RDFS rules must pass W3C test suite:

```rust
#[cfg(test)]
mod rdfs_conformance {
    use super::*;

    #[test]
    fn test_rdfs2_domain_inference() {
        let mut graph = QuadStore::new(InMemoryBackend::new());
        let dict = Dictionary::new();

        // Asserted triples
        graph.insert_triple(
            Node::iri(dict.intern("http://ex.org/worksFor")),
            Node::iri(RDFS_DOMAIN),
            Node::iri(dict.intern("http://ex.org/Person"))
        );
        graph.insert_triple(
            Node::iri(dict.intern("http://ex.org/alice")),
            Node::iri(dict.intern("http://ex.org/worksFor")),
            Node::iri(dict.intern("http://ex.org/CompanyX"))
        );

        // Apply RDFS reasoning
        let reasoner = RdfsReasoner::new(ReasonerProfile::RdfsFull);
        let mut ctx = InferenceContext::new();
        reasoner.infer(&graph, &mut ctx);

        // Check inferred triple
        let expected = Triple::new(
            Node::iri(dict.intern("http://ex.org/alice")),
            Node::iri(RDF_TYPE),
            Node::iri(dict.intern("http://ex.org/Person"))
        );

        assert!(ctx.inferred_triples().contains(&expected));
    }

    // Repeat for all 13 rules...
}
```

### 2. OWL 2 Profile Tests

Test each profile against W3C OWL 2 test cases:

```rust
#[cfg(test)]
mod owl2_conformance {
    use super::*;

    #[test]
    fn test_owl2_el_existential_elimination() {
        // Test data from OWL 2 EL test suite
        // ...
    }

    #[test]
    fn test_owl2_ql_query_rewriting() {
        // Test query rewriting correctness
        // ...
    }

    #[test]
    fn test_owl2_rl_property_chain() {
        // Test property chain inference
        // ...
    }
}
```

### 3. Rule Engine Tests

```rust
#[cfg(test)]
mod rule_engine_tests {
    use super::*;

    #[test]
    fn test_rule_parsing() {
        let rule_text = r#"
            [rdfs2:
                (?p rdfs:domain ?c), (?x ?p ?y)
                ->
                (?x rdf:type ?c)
            ]
        "#;

        let rule = RuleParser::parse(rule_text).unwrap();

        assert_eq!(rule.name, "rdfs2");
        assert_eq!(rule.body.len(), 2);
        assert_eq!(rule.head.len(), 1);
    }

    #[test]
    fn test_forward_chaining() {
        // Test forward chaining execution
        // ...
    }

    #[test]
    fn test_backward_chaining() {
        // Test backward chaining (goal-driven)
        // ...
    }
}
```

### 4. RETE Network Tests

```rust
#[cfg(test)]
mod rete_tests {
    use super::*;

    #[test]
    fn test_rete_incremental_update() {
        let mut network = ReteNetwork::new();

        // Add triples incrementally
        network.assert_triple(triple1);
        network.assert_triple(triple2);

        // Verify activations
        assert_eq!(network.agenda.len(), 1);

        // Fire rule
        let inferred = network.fire_next_rule().unwrap();
        assert_eq!(inferred.len(), 1);
    }

    #[test]
    fn test_rete_retraction() {
        let mut network = ReteNetwork::new();
        network.assert_triple(triple1);
        network.retract_triple(&triple1);

        // Verify no activations remain
        assert_eq!(network.agenda.len(), 0);
    }
}
```

### 5. Property-Based Tests

Use `proptest` for fuzzing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_transitive_closure_correctness(edges: Vec<(u32, u32)>) {
        let mut closure = TransitiveClosure::new();

        for (from, to) in edges {
            closure.add_edge(Node::blank(from), Node::blank(to));
        }

        closure.compute();

        // Property: if a->b and b->c, then a->c
        for (a, b) in closure.direct_edges() {
            for (b2, c) in closure.direct_edges() {
                if b == b2 {
                    prop_assert!(closure.is_reachable(a, c));
                }
            }
        }
    }
}
```

---

## Performance Benchmarks

### Benchmark Suite

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_rdfs_reasoning(c: &mut Criterion) {
    let mut group = c.benchmark_group("rdfs_reasoning");

    for size in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::new("full", size), size, |b, &size| {
            let graph = generate_test_graph(size);
            let reasoner = RdfsReasoner::new(ReasonerProfile::RdfsFull);

            b.iter(|| {
                let mut ctx = InferenceContext::new();
                reasoner.infer(black_box(&graph), &mut ctx);
            });
        });
    }

    group.finish();
}

fn benchmark_rete_network(c: &mut Criterion) {
    let mut group = c.benchmark_group("rete_network");

    for num_rules in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("forward_chain", num_rules), num_rules, |b, &num_rules| {
            let network = create_rete_network(num_rules);
            let triple = generate_random_triple();

            b.iter(|| {
                network.assert_triple(black_box(triple.clone()));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_rdfs_reasoning, benchmark_rete_network);
criterion_main!(benches);
```

### Target Performance

| Operation | Target | Rationale |
|-----------|--------|-----------|
| RDFS inference (1K triples) | < 1ms | Real-time UI updates |
| RDFS inference (100K triples) | < 100ms | Background processing |
| OWL 2 EL classification (10K classes) | < 500ms | Ontology loading |
| RETE rule firing (single triple) | < 100μs | Incremental updates |
| Transitive closure (1K nodes, sparse) | < 10ms | Hierarchy navigation |
| Transitive closure (1K nodes, dense) | < 50ms | Full materialization |

---

## Implementation Roadmap

### Phase 1: Core RDFS (Weeks 1-2)

- [ ] Implement all 13 RDFS rules
- [ ] Basic forward chaining reasoner
- [ ] Unit tests for each rule
- [ ] Memory-efficient data structures

**Deliverables:**
- `crates/reasoning/src/rdfs/` fully implemented
- 100% test coverage for RDFS rules
- < 100MB memory for 1M triples

### Phase 2: Transitive Closure (Week 3)

- [ ] Floyd-Warshall for small graphs
- [ ] Lazy BFS/DFS for large graphs
- [ ] Transitive graph cache (TGC)
- [ ] Incremental updates

**Deliverables:**
- `crates/reasoning/src/transitive/` complete
- Sub-10ms queries for hierarchies

### Phase 3: Generic Rule Engine (Weeks 4-5)

- [ ] Rule parser (Logos-based lexer + Pest grammar)
- [ ] Forward chaining executor
- [ ] Backward chaining executor
- [ ] Builtin functions (30+ from Jena)

**Deliverables:**
- `crates/reasoning/src/rule_engine/` operational
- Custom rules supported

### Phase 4: RETE Algorithm (Weeks 6-7)

- [ ] Alpha network (single-pattern matching)
- [ ] Beta network (join nodes)
- [ ] Agenda management
- [ ] Node sharing optimization
- [ ] Right-unlinking

**Deliverables:**
- `crates/reasoning/src/rule_engine/rete.rs`
- < 100μs rule firing

### Phase 5: OWL 2 Profiles (Weeks 8-10)

- [ ] OWL 2 EL reasoner
- [ ] OWL 2 QL query rewriter
- [ ] OWL 2 RL rule implementation (61 rules)
- [ ] Profile selection logic

**Deliverables:**
- `crates/reasoning/src/owl/` complete
- W3C OWL 2 conformance tests passing

### Phase 6: Mobile Optimization (Weeks 11-12)

- [ ] Lazy evaluation
- [ ] Incremental reasoning
- [ ] Rule pruning
- [ ] Memory budgeting
- [ ] Battery-aware profiles

**Deliverables:**
- < 1ms query latency on mobile
- < 100MB total memory footprint

### Phase 7: Testing & Benchmarking (Week 13)

- [ ] Full W3C test suite integration
- [ ] Criterion benchmarks
- [ ] Property-based tests
- [ ] Fuzzing with cargo-fuzz
- [ ] Memory profiling with valgrind

**Deliverables:**
- 100% conformance with W3C specs
- Performance metrics documented

### Phase 8: Integration & Documentation (Week 14)

- [ ] Integrate with rust-kgdb storage layer
- [ ] FFI bindings for Swift/Kotlin
- [ ] API documentation
- [ ] Usage examples
- [ ] Migration guide from Jena

**Deliverables:**
- Production-ready reasoning engine
- Complete documentation

---

## Appendix A: Builtin Functions

Complete list of builtin functions to implement (from Apache Jena):

### Type Checking
- `isLiteral(?x)` - Test if node is a literal
- `isFunctor(?x)` - Test if node is a functor
- `isBNode(?x)` - Test if node is a blank node
- `isIRI(?x)` - Test if node is an IRI
- `bound(?x)` - Test if variable is bound
- `unbound(?x)` - Test if variable is not bound

### Comparison
- `equal(?x, ?y)` - Equality test
- `notEqual(?x, ?y)` - Inequality test
- `lessThan(?x, ?y)` - Numeric less than
- `lessThanOrEqual(?x, ?y)` - Numeric ≤
- `greaterThan(?x, ?y)` - Numeric greater than
- `greaterThanOrEqual(?x, ?y)` - Numeric ≥

### Arithmetic
- `sum(?x, ?y, ?z)` - ?z = ?x + ?y
- `difference(?x, ?y, ?z)` - ?z = ?x - ?y
- `product(?x, ?y, ?z)` - ?z = ?x * ?y
- `quotient(?x, ?y, ?z)` - ?z = ?x / ?y
- `min(?x, ?y, ?z)` - ?z = min(?x, ?y)
- `max(?x, ?y, ?z)` - ?z = max(?x, ?y)
- `addOne(?x, ?y)` - ?y = ?x + 1

### String Operations
- `strConcat(?s1, ?s2, ?result)` - String concatenation
- `uriConcat(?base, ?suffix, ?uri)` - URI concatenation
- `regex(?string, ?pattern)` - Regular expression match
- `strLength(?string, ?length)` - String length

### List Operations
- `listContains(?list, ?element)` - List membership test
- `listEntry(?list, ?index, ?element)` - List indexing
- `listLength(?list, ?length)` - List length
- `listEqual(?list1, ?list2)` - List equality

### Temporal
- `now(?datetime)` - Current timestamp

### Resource Creation
- `makeTemp(?x)` - Create fresh blank node
- `makeInstance(?x, ?p, ?c)` - Create instance of class
- `makeSkolem(?x, ?y, ...)` - Create Skolem IRI

### Graph Operations
- `noValue(?s, ?p)` - Test absence of property
- `remove(?x)` - Remove triple from graph
- `drop(?i)` - Remove index entry

### Control Flow
- `table(?p)` - Mark predicate for tabling (memoization)
- `tableAll()` - Table all predicates
- `hide(?p)` - Don't expose predicate in results

### Debugging
- `print(?x, ...)` - Print values to console

---

## Appendix B: RDF/RDFS/OWL Vocabulary

Standard vocabulary constants:

```rust
// RDF namespace
pub const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
pub const RDF_PROPERTY: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property";
pub const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
pub const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
pub const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";

// RDFS namespace
pub const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";
pub const RDFS_RESOURCE: &str = "http://www.w3.org/2000/01/rdf-schema#Resource";
pub const RDFS_CLASS: &str = "http://www.w3.org/2000/01/rdf-schema#Class";
pub const RDFS_LITERAL: &str = "http://www.w3.org/2000/01/rdf-schema#Literal";
pub const RDFS_DATATYPE: &str = "http://www.w3.org/2000/01/rdf-schema#Datatype";
pub const RDFS_SUB_CLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
pub const RDFS_SUB_PROPERTY_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";
pub const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
pub const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
pub const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
pub const RDFS_COMMENT: &str = "http://www.w3.org/2000/01/rdf-schema#comment";
pub const RDFS_MEMBER: &str = "http://www.w3.org/2000/01/rdf-schema#member";
pub const RDFS_CONTAINER_MEMBERSHIP_PROPERTY: &str = "http://www.w3.org/2000/01/rdf-schema#ContainerMembershipProperty";

// OWL namespace
pub const OWL_NS: &str = "http://www.w3.org/2002/07/owl#";
pub const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
pub const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";
pub const OWL_NOTHING: &str = "http://www.w3.org/2002/07/owl#Nothing";
pub const OWL_OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
pub const OWL_DATATYPE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DatatypeProperty";
pub const OWL_TRANSITIVE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#TransitiveProperty";
pub const OWL_SYMMETRIC_PROPERTY: &str = "http://www.w3.org/2002/07/owl#SymmetricProperty";
pub const OWL_FUNCTIONAL_PROPERTY: &str = "http://www.w3.org/2002/07/owl#FunctionalProperty";
pub const OWL_INVERSE_FUNCTIONAL_PROPERTY: &str = "http://www.w3.org/2002/07/owl#InverseFunctionalProperty";
pub const OWL_EQUIVALENT_CLASS: &str = "http://www.w3.org/2002/07/owl#equivalentClass";
pub const OWL_EQUIVALENT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#equivalentProperty";
pub const OWL_SAME_AS: &str = "http://www.w3.org/2002/07/owl#sameAs";
pub const OWL_DIFFERENT_FROM: &str = "http://www.w3.org/2002/07/owl#differentFrom";
pub const OWL_INVERSE_OF: &str = "http://www.w3.org/2002/07/owl#inverseOf";

// XSD namespace
pub const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";
pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
pub const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
pub const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";
pub const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";
pub const XSD_FLOAT: &str = "http://www.w3.org/2001/XMLSchema#float";
pub const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
pub const XSD_DATE: &str = "http://www.w3.org/2001/XMLSchema#date";
pub const XSD_DATE_TIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";
```

---

## Appendix C: References

### W3C Specifications
- [RDF 1.1 Semantics](https://www.w3.org/TR/rdf11-mt/) - RDFS entailment rules
- [OWL 2 Web Ontology Language Profiles](https://www.w3.org/TR/owl2-profiles/) - EL/QL/RL specifications
- [OWL 2 RL/RDF Rules](https://www.w3.org/TR/owl2-profiles/#OWL_2_RL) - Complete rule tables

### Apache Jena Documentation
- [Jena Inference Support](https://jena.apache.org/documentation/inference/) - Architecture overview
- [Jena Rule Syntax](https://jena.apache.org/documentation/inference/#rules) - Rule language spec
- [Jena Reasoners](https://jena.apache.org/documentation/inference/#reasoners) - Built-in reasoners

### Academic Papers
- Forgy, C. L. (1982). "Rete: A Fast Algorithm for the Many Pattern/Many Object Pattern Match Problem". Artificial Intelligence.
- Motik, B. et al. (2009). "OWL 2 Web Ontology Language Profiles". W3C Recommendation.

### Rust Resources
- [Zero-Copy Deserialization](https://manishearth.github.io/blog/2022/08/03/zero-copy-1-not-a-yoking-matter/)
- [Rust Lifetime Management](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [Arena Allocation with Bumpalo](https://docs.rs/bumpalo/)

---

## Conclusion

This guide provides complete specifications for implementing Apache Jena's reasoning capabilities in Rust for rust-kgdb. Key achievements:

1. **Complete RDFS**: All 13 entailment rules with formal definitions
2. **OWL 2 Profiles**: EL/QL/RL implementations with mobile optimizations
3. **RETE Algorithm**: Production-grade pattern matching
4. **Zero Compromise**: Feature parity with Jena, optimized for mobile
5. **Sub-millisecond**: Target < 1ms inference for typical queries

Next steps:
1. Begin Phase 1 implementation (RDFS core)
2. Set up CI/CD with W3C test suites
3. Establish performance baseline benchmarks
4. Iterate with mobile profiling

**Target completion:** 14 weeks
**Expected performance:** 10-100x faster than Jena on mobile devices
**Memory footprint:** < 100MB (vs. Jena's 200MB+)

---

*Document Version: 1.0*
*Last Updated: 2025-11-17*
*Maintainer: Rust-KGDB Team*
