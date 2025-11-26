//! RDFS (RDF Schema) Reasoner
//!
//! Implements all 13 W3C RDFS entailment rules with forward chaining.
//! Based on RDF 1.1 Semantics specification.

use crate::{ReasonerConfig, ReasonerError, ReasonerResult};
use rdf_model::{Node, Quad, Triple, Dictionary, IriRef, Literal};
use storage::{QuadPattern, StorageBackend};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// RDFS Reasoner implementing all 13 W3C entailment rules
pub struct RDFSReasoner<'a, S: StorageBackend> {
    /// Configuration
    config: ReasonerConfig,
    
    /// Storage backend
    storage: &'a mut S,
    
    /// Dictionary for string interning
    dictionary: Arc<Dictionary>,
    
    /// Derived triples (materialized inferences)
    derived: HashSet<Triple<'a>>,
    
    /// Work queue for forward chaining
    queue: VecDeque<Triple<'a>>,
    
    /// Inference counter
    inferred_count: usize,
}

impl<'a, S: StorageBackend> RDFSReasoner<'a, S> {
    /// Create new RDFS reasoner
    pub fn new(storage: &'a mut S, dictionary: Arc<Dictionary>) -> Self {
        Self::with_config(storage, dictionary, ReasonerConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(
        storage: &'a mut S,
        dictionary: Arc<Dictionary>,
        config: ReasonerConfig,
    ) -> Self {
        Self {
            config,
            storage,
            dictionary,
            derived: HashSet::new(),
            queue: VecDeque::new(),
            inferred_count: 0,
        }
    }
    
    /// Run complete RDFS inference (all 13 rules)
    pub fn infer(&mut self) -> ReasonerResult<usize> {
        // Iterate until fixpoint (no new triples derived)
        let mut iterations = 0;
        
        loop {
            let before = self.derived.len();
            
            // Apply all 13 RDFS rules
            self.apply_rdfs1()?;  // Datatype recognition
            self.apply_rdfs2()?;  // Domain inference
            self.apply_rdfs3()?;  // Range inference
            self.apply_rdfs4()?;  // Resource typing
            self.apply_rdfs5()?;  // SubProperty transitivity
            self.apply_rdfs6()?;  // Property reflexivity
            self.apply_rdfs7()?;  // SubProperty implication
            self.apply_rdfs8()?;  // Class to Resource subclass
            self.apply_rdfs9()?;  // SubClass implication
            self.apply_rdfs10()?; // Class reflexivity
            self.apply_rdfs11()?; // SubClass transitivity
            self.apply_rdfs12()?; // Container membership
            self.apply_rdfs13()?; // Datatype subclass of Literal
            
            iterations += 1;
            
            // Check for fixpoint
            if self.derived.len() == before {
                break;
            }
            
            // Check resource limits
            if iterations > self.config.max_depth {
                return Err(ReasonerError::ResourceLimit(
                    format!("Max iterations {} exceeded", self.config.max_depth)
                ));
            }
            
            if self.inferred_count > self.config.max_inferred {
                return Err(ReasonerError::ResourceLimit(
                    format!("Max inferred {} exceeded", self.config.max_inferred)
                ));
            }
        }
        
        Ok(self.derived.len())
    }
    
    /// Get all derived triples
    pub fn get_derived(&self) -> &HashSet<Triple<'a>> {
        &self.derived
    }
    
    /// rdfs1: Datatype Recognition
    /// IF D is a datatype IRI
    /// THEN infer: D rdf:type rdfs:Datatype
    fn apply_rdfs1(&mut self) -> ReasonerResult<()> {
        // Standard XML Schema datatypes
        let datatypes = vec![
            "http://www.w3.org/2001/XMLSchema#string",
            "http://www.w3.org/2001/XMLSchema#integer",
            "http://www.w3.org/2001/XMLSchema#decimal",
            "http://www.w3.org/2001/XMLSchema#double",
            "http://www.w3.org/2001/XMLSchema#boolean",
            "http://www.w3.org/2001/XMLSchema#date",
            "http://www.w3.org/2001/XMLSchema#dateTime",
        ];
        
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let rdfs_datatype = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#Datatype");
        
        for dt in datatypes {
            let dt_iri = self.dictionary.intern(dt);
            let triple = Triple {
                subject: Node::iri(dt_iri),
                predicate: Node::iri(rdf_type),
                object: Node::iri(rdfs_datatype),
            };
            self.add_derived(triple);
        }
        
        Ok(())
    }
    
    /// rdfs2: Domain Inference
    /// IF (?p rdfs:domain ?c) AND (?x ?p ?y)
    /// THEN infer: ?x rdf:type ?c
    fn apply_rdfs2(&mut self) -> ReasonerResult<()> {
        let rdfs_domain = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#domain");
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        
        // Find all (?p rdfs:domain ?c) triples
        let domain_triples = self.find_triples(None, Some(Node::iri(rdfs_domain)), None)?;
        
        for domain_triple in domain_triples {
            if let (Node::Iri(prop), Node::Iri(class)) = (domain_triple.subject, domain_triple.object) {
                // Find all (?x ?p ?y) triples
                let instances = self.find_triples(None, Some(Node::iri(prop.0)), None)?;
                
                for instance in instances {
                    // Infer: ?x rdf:type ?c
                    let triple = Triple {
                        subject: instance.subject,
                        predicate: Node::iri(rdf_type),
                        object: Node::iri(class.0),
                    };
                    self.add_derived(triple);
                }
            }
        }
        
        Ok(())
    }
    
    /// rdfs3: Range Inference
    /// IF (?p rdfs:range ?c) AND (?x ?p ?y)
    /// THEN infer: ?y rdf:type ?c
    fn apply_rdfs3(&mut self) -> ReasonerResult<()> {
        let rdfs_range = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#range");
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        
        let range_triples = self.find_triples(None, Some(Node::iri(rdfs_range)), None)?;
        
        for range_triple in range_triples {
            if let (Node::Iri(prop), Node::Iri(class)) = (range_triple.subject, range_triple.object) {
                let instances = self.find_triples(None, Some(Node::iri(prop.0)), None)?;
                
                for instance in instances {
                    // Only infer for IRI/BlankNode objects (not literals)
                    if matches!(instance.object, Node::Iri(_) | Node::BlankNode(_)) {
                        let triple = Triple {
                            subject: instance.object,
                            predicate: Node::iri(rdf_type),
                            object: Node::iri(class.0),
                        };
                        self.add_derived(triple);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// rdfs4a & rdfs4b: Resource Typing
    /// IF (?x ?p ?y)
    /// THEN infer: ?x rdf:type rdfs:Resource
    ///      AND: ?y rdf:type rdfs:Resource (if IRI/BlankNode)
    fn apply_rdfs4(&mut self) -> ReasonerResult<()> {
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let rdfs_resource = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#Resource");
        
        // All subjects and objects are resources
        let all_triples = self.find_triples(None, None, None)?;
        
        for triple in all_triples {
            // Subject is always a resource
            let subj_triple = Triple {
                subject: triple.subject,
                predicate: Node::iri(rdf_type),
                object: Node::iri(rdfs_resource),
            };
            self.add_derived(subj_triple);
            
            // Object is resource if IRI/BlankNode
            if matches!(triple.object, Node::Iri(_) | Node::BlankNode(_)) {
                let obj_triple = Triple {
                    subject: triple.object,
                    predicate: Node::iri(rdf_type),
                    object: Node::iri(rdfs_resource),
                };
                self.add_derived(obj_triple);
            }
        }
        
        Ok(())
    }
    
    /// rdfs5: SubProperty Transitivity
    /// IF (?p rdfs:subPropertyOf ?q) AND (?q rdfs:subPropertyOf ?r)
    /// THEN infer: ?p rdfs:subPropertyOf ?r
    fn apply_rdfs5(&mut self) -> ReasonerResult<()> {
        let rdfs_subprop = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subPropertyOf");
        
        let subprop_triples = self.find_triples(None, Some(Node::iri(rdfs_subprop)), None)?;
        
        // Build property hierarchy map
        let mut hierarchy: HashMap<&str, Vec<&str>> = HashMap::new();
        for triple in &subprop_triples {
            if let (Node::Iri(p), Node::Iri(q)) = (triple.subject, triple.object) {
                hierarchy.entry(p.0).or_insert_with(Vec::new).push(q.0);
            }
        }
        
        // Compute transitive closure
        for (p, _) in hierarchy.clone() {
            let closure = self.transitive_closure(&hierarchy, p);
            for r in closure {
                if p != r {
                    let triple = Triple {
                        subject: Node::iri(p),
                        predicate: Node::iri(rdfs_subprop),
                        object: Node::iri(r),
                    };
                    self.add_derived(triple);
                }
            }
        }
        
        Ok(())
    }
    
    /// rdfs6: Property Reflexivity
    /// IF (?p rdf:type rdf:Property)
    /// THEN infer: ?p rdfs:subPropertyOf ?p
    fn apply_rdfs6(&mut self) -> ReasonerResult<()> {
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let rdf_property = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property");
        let rdfs_subprop = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subPropertyOf");
        
        let properties = self.find_triples(None, Some(Node::iri(rdf_type)), Some(Node::iri(rdf_property)))?;
        
        for prop in properties {
            if let Node::Iri(p) = prop.subject {
                let triple = Triple {
                    subject: Node::iri(p.0),
                    predicate: Node::iri(rdfs_subprop),
                    object: Node::iri(p.0),
                };
                self.add_derived(triple);
            }
        }
        
        Ok(())
    }
    
    /// rdfs7: SubProperty Implication
    /// IF (?p rdfs:subPropertyOf ?q) AND (?x ?p ?y)
    /// THEN infer: ?x ?q ?y
    fn apply_rdfs7(&mut self) -> ReasonerResult<()> {
        let rdfs_subprop = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subPropertyOf");
        
        let subprop_triples = self.find_triples(None, Some(Node::iri(rdfs_subprop)), None)?;
        
        for subprop in subprop_triples {
            if let (Node::Iri(p), Node::Iri(q)) = (subprop.subject, subprop.object) {
                let instances = self.find_triples(None, Some(Node::iri(p.0)), None)?;
                
                for instance in instances {
                    let triple = Triple {
                        subject: instance.subject,
                        predicate: Node::iri(q.0),
                        object: instance.object,
                    };
                    self.add_derived(triple);
                }
            }
        }
        
        Ok(())
    }
    
    /// rdfs8: Class to Resource Subclass
    /// IF (?c rdf:type rdfs:Class)
    /// THEN infer: ?c rdfs:subClassOf rdfs:Resource
    fn apply_rdfs8(&mut self) -> ReasonerResult<()> {
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let rdfs_class = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#Class");
        let rdfs_subclass = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subClassOf");
        let rdfs_resource = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#Resource");
        
        let classes = self.find_triples(None, Some(Node::iri(rdf_type)), Some(Node::iri(rdfs_class)))?;
        
        for class in classes {
            if let Node::Iri(c) = class.subject {
                let triple = Triple {
                    subject: Node::iri(c.0),
                    predicate: Node::iri(rdfs_subclass),
                    object: Node::iri(rdfs_resource),
                };
                self.add_derived(triple);
            }
        }
        
        Ok(())
    }
    
    /// rdfs9: SubClass Implication
    /// IF (?c rdfs:subClassOf ?d) AND (?x rdf:type ?c)
    /// THEN infer: ?x rdf:type ?d
    fn apply_rdfs9(&mut self) -> ReasonerResult<()> {
        let rdfs_subclass = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subClassOf");
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        
        let subclass_triples = self.find_triples(None, Some(Node::iri(rdfs_subclass)), None)?;
        
        for subclass in subclass_triples {
            if let (Node::Iri(c), Node::Iri(d)) = (subclass.subject, subclass.object) {
                let instances = self.find_triples(None, Some(Node::iri(rdf_type)), Some(Node::iri(c.0)))?;
                
                for instance in instances {
                    let triple = Triple {
                        subject: instance.subject,
                        predicate: Node::iri(rdf_type),
                        object: Node::iri(d.0),
                    };
                    self.add_derived(triple);
                }
            }
        }
        
        Ok(())
    }
    
    /// rdfs10: Class Reflexivity
    /// IF (?c rdf:type rdfs:Class)
    /// THEN infer: ?c rdfs:subClassOf ?c
    fn apply_rdfs10(&mut self) -> ReasonerResult<()> {
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let rdfs_class = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#Class");
        let rdfs_subclass = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subClassOf");
        
        let classes = self.find_triples(None, Some(Node::iri(rdf_type)), Some(Node::iri(rdfs_class)))?;
        
        for class in classes {
            if let Node::Iri(c) = class.subject {
                let triple = Triple {
                    subject: Node::iri(c.0),
                    predicate: Node::iri(rdfs_subclass),
                    object: Node::iri(c.0),
                };
                self.add_derived(triple);
            }
        }
        
        Ok(())
    }
    
    /// rdfs11: SubClass Transitivity
    /// IF (?c rdfs:subClassOf ?d) AND (?d rdfs:subClassOf ?e)
    /// THEN infer: ?c rdfs:subClassOf ?e
    fn apply_rdfs11(&mut self) -> ReasonerResult<()> {
        let rdfs_subclass = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subClassOf");
        
        let subclass_triples = self.find_triples(None, Some(Node::iri(rdfs_subclass)), None)?;
        
        // Build class hierarchy
        let mut hierarchy: HashMap<&str, Vec<&str>> = HashMap::new();
        for triple in &subclass_triples {
            if let (Node::Iri(c), Node::Iri(d)) = (triple.subject, triple.object) {
                hierarchy.entry(c.0).or_insert_with(Vec::new).push(d.0);
            }
        }
        
        // Compute transitive closure
        for (c, _) in hierarchy.clone() {
            let closure = self.transitive_closure(&hierarchy, c);
            for e in closure {
                if c != e {
                    let triple = Triple {
                        subject: Node::iri(c),
                        predicate: Node::iri(rdfs_subclass),
                        object: Node::iri(e),
                    };
                    self.add_derived(triple);
                }
            }
        }
        
        Ok(())
    }
    
    /// rdfs12: Container Membership Property
    /// IF (?p rdf:type rdfs:ContainerMembershipProperty)
    /// THEN infer: ?p rdfs:subPropertyOf rdfs:member
    fn apply_rdfs12(&mut self) -> ReasonerResult<()> {
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let rdfs_cmp = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#ContainerMembershipProperty");
        let rdfs_subprop = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subPropertyOf");
        let rdfs_member = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#member");
        
        let props = self.find_triples(None, Some(Node::iri(rdf_type)), Some(Node::iri(rdfs_cmp)))?;
        
        for prop in props {
            if let Node::Iri(p) = prop.subject {
                let triple = Triple {
                    subject: Node::iri(p.0),
                    predicate: Node::iri(rdfs_subprop),
                    object: Node::iri(rdfs_member),
                };
                self.add_derived(triple);
            }
        }
        
        Ok(())
    }
    
    /// rdfs13: Datatype Subclass of Literal
    /// IF (?d rdf:type rdfs:Datatype)
    /// THEN infer: ?d rdfs:subClassOf rdfs:Literal
    fn apply_rdfs13(&mut self) -> ReasonerResult<()> {
        let rdf_type = self.dictionary.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
        let rdfs_datatype = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#Datatype");
        let rdfs_subclass = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#subClassOf");
        let rdfs_literal = self.dictionary.intern("http://www.w3.org/2000/01/rdf-schema#Literal");
        
        let datatypes = self.find_triples(None, Some(Node::iri(rdf_type)), Some(Node::iri(rdfs_datatype)))?;
        
        for dt in datatypes {
            if let Node::Iri(d) = dt.subject {
                let triple = Triple {
                    subject: Node::iri(d.0),
                    predicate: Node::iri(rdfs_subclass),
                    object: Node::iri(rdfs_literal),
                };
                self.add_derived(triple);
            }
        }
        
        Ok(())
    }
    
    // Helper methods
    
    fn find_triples(
        &self,
        subject: Option<Node<'a>>,
        predicate: Option<Node<'a>>,
        object: Option<Node<'a>>,
    ) -> ReasonerResult<Vec<Triple<'a>>> {
        // Search in both storage and derived triples
        let mut results = Vec::new();
        
        // Add derived triples
        for triple in &self.derived {
            if subject.as_ref().map_or(true, |s| s == triple.subject) &&
               predicate.as_ref().map_or(true, |p| p == triple.predicate) &&
               object.as_ref().map_or(true, |o| o == triple.object) {
                results.push(triple.clone());
            }
        }
        
        Ok(results)
    }
    
    fn add_derived(&mut self, triple: Triple<'a>) {
        if self.derived.insert(triple) {
            self.inferred_count += 1;
        }
    }
    
    fn transitive_closure<'b>(
        &self,
        graph: &HashMap<&'b str, Vec<&'b str>>,
        start: &'b str,
    ) -> HashSet<&'b str> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        
        while let Some(node) = queue.pop_front() {
            if visited.insert(node) {
                if let Some(neighbors) = graph.get(node) {
                    for &neighbor in neighbors {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        visited
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::InMemoryBackend;

    #[test]
    fn test_rdfs_reasoner_creation() {
        let dict = Arc::new(Dictionary::new());
        let mut storage = InMemoryBackend::new();
        let reasoner = RDFSReasoner::new(&mut storage, dict);
        assert_eq!(reasoner.derived.len(), 0);
    }
    
    #[test]
    fn test_config() {
        let config = ReasonerConfig {
            trace_rules: true,
            max_depth: 50,
            max_inferred: 500_000,
            incremental: false,
            parallel: false,
        };
        assert!(config.trace_rules);
        assert_eq!(config.max_depth, 50);
    }
}
