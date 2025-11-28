//! W3C PROV-O Core Types
//!
//! Implements the core PROV-O (Provenance Ontology) data model with:
//! - **Entity**: Things (documents, datasets, etc.)
//! - **Activity**: Actions/processes (creation, modification, etc.)
//! - **Agent**: Actors (people, organizations, software)
//!
//! # W3C PROV-O Spec
//!
//! Based on [W3C PROV-O Recommendation](https://www.w3.org/TR/prov-o/)

use rdf_model::Node;
use std::collections::HashMap;

/// PROV-O Entity
///
/// An entity is a physical, digital, conceptual, or other kind of thing with some fixed aspects.
///
/// # Examples
///
/// - A document
/// - A dataset
/// - A database record
/// - A file
#[derive(Debug, Clone)]
pub struct Entity<'a> {
    /// Entity IRI
    pub id: Node<'a>,
    /// Entity type (optional)
    pub entity_type: Option<Node<'a>>,
    /// Generation activity (prov:wasGeneratedBy)
    pub was_generated_by: Option<Node<'a>>,
    /// Attribution (prov:wasAttributedTo)
    pub was_attributed_to: Vec<Node<'a>>,
    /// Derivation (prov:wasDerivedFrom)
    pub was_derived_from: Vec<Node<'a>>,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

impl<'a> Entity<'a> {
    /// Create a new entity
    pub fn new(id: Node<'a>) -> Self {
        Self {
            id,
            entity_type: None,
            was_generated_by: None,
            was_attributed_to: Vec::new(),
            was_derived_from: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Set entity type
    pub fn with_type(mut self, entity_type: Node<'a>) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    /// Set generation activity
    pub fn generated_by(mut self, activity: Node<'a>) -> Self {
        self.was_generated_by = Some(activity);
        self
    }

    /// Add attribution
    pub fn attributed_to(mut self, agent: Node<'a>) -> Self {
        self.was_attributed_to.push(agent);
        self
    }

    /// Add derivation
    pub fn derived_from(mut self, entity: Node<'a>) -> Self {
        self.was_derived_from.push(entity);
        self
    }

    /// Add custom attribute
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

/// PROV-O Activity
///
/// An activity is something that occurs over a period of time and acts upon or with entities.
///
/// # Examples
///
/// - Document editing
/// - Data transformation
/// - File upload
/// - Query execution
#[derive(Debug, Clone)]
pub struct Activity<'a> {
    /// Activity IRI
    pub id: Node<'a>,
    /// Activity type (optional)
    pub activity_type: Option<Node<'a>>,
    /// Start time (xsd:dateTime)
    pub start_time: Option<String>,
    /// End time (xsd:dateTime)
    pub end_time: Option<String>,
    /// Associated agent (prov:wasAssociatedWith)
    pub was_associated_with: Vec<Node<'a>>,
    /// Used entities (prov:used)
    pub used: Vec<Node<'a>>,
    /// Generated entities (inverse of wasGeneratedBy)
    pub generated: Vec<Node<'a>>,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

impl<'a> Activity<'a> {
    /// Create a new activity
    pub fn new(id: Node<'a>) -> Self {
        Self {
            id,
            activity_type: None,
            start_time: None,
            end_time: None,
            was_associated_with: Vec::new(),
            used: Vec::new(),
            generated: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Set activity type
    pub fn with_type(mut self, activity_type: Node<'a>) -> Self {
        self.activity_type = Some(activity_type);
        self
    }

    /// Set start time
    pub fn started_at(mut self, time: String) -> Self {
        self.start_time = Some(time);
        self
    }

    /// Set end time
    pub fn ended_at(mut self, time: String) -> Self {
        self.end_time = Some(time);
        self
    }

    /// Add associated agent
    pub fn associated_with(mut self, agent: Node<'a>) -> Self {
        self.was_associated_with.push(agent);
        self
    }

    /// Add used entity
    pub fn used_entity(mut self, entity: Node<'a>) -> Self {
        self.used.push(entity);
        self
    }

    /// Add generated entity
    pub fn generated_entity(mut self, entity: Node<'a>) -> Self {
        self.generated.push(entity);
        self
    }

    /// Add custom attribute
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

/// PROV-O Agent
///
/// An agent is something that bears some form of responsibility for an activity taking place,
/// for the existence of an entity, or for another agent's activity.
///
/// # Examples
///
/// - Person
/// - Organization
/// - Software agent/bot
#[derive(Debug, Clone)]
pub struct Agent<'a> {
    /// Agent IRI
    pub id: Node<'a>,
    /// Agent type
    pub agent_type: AgentType,
    /// Agent name (foaf:name)
    pub name: Option<String>,
    /// Agent email (foaf:mbox)
    pub email: Option<String>,
    /// Acted on behalf of (prov:actedOnBehalfOf)
    pub acted_on_behalf_of: Option<Node<'a>>,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

impl<'a> Agent<'a> {
    /// Create a new agent
    pub fn new(id: Node<'a>, agent_type: AgentType) -> Self {
        Self {
            id,
            agent_type,
            name: None,
            email: None,
            acted_on_behalf_of: None,
            attributes: HashMap::new(),
        }
    }

    /// Set agent name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set agent email
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Set delegation
    pub fn on_behalf_of(mut self, agent: Node<'a>) -> Self {
        self.acted_on_behalf_of = Some(agent);
        self
    }

    /// Add custom attribute
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

/// Type of PROV-O Agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Generic agent
    Agent,
    /// Person (prov:Person)
    Person,
    /// Organization (prov:Organization)
    Organization,
    /// Software agent (prov:SoftwareAgent)
    SoftwareAgent,
}

impl AgentType {
    /// Get the PROV-O IRI for this agent type
    pub fn as_iri(&self) -> &'static str {
        match self {
            AgentType::Agent => "http://www.w3.org/ns/prov#Agent",
            AgentType::Person => "http://www.w3.org/ns/prov#Person",
            AgentType::Organization => "http://www.w3.org/ns/prov#Organization",
            AgentType::SoftwareAgent => "http://www.w3.org/ns/prov#SoftwareAgent",
        }
    }
}

/// Provenance Bundle
///
/// A collection of provenance statements (entities, activities, agents) that form a cohesive
/// provenance record.
#[derive(Debug, Clone, Default)]
pub struct ProvenanceBundle<'a> {
    /// Entities in this bundle
    pub entities: Vec<Entity<'a>>,
    /// Activities in this bundle
    pub activities: Vec<Activity<'a>>,
    /// Agents in this bundle
    pub agents: Vec<Agent<'a>>,
}

impl<'a> ProvenanceBundle<'a> {
    /// Create a new empty bundle
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entity to the bundle
    pub fn add_entity(&mut self, entity: Entity<'a>) {
        self.entities.push(entity);
    }

    /// Add an activity to the bundle
    pub fn add_activity(&mut self, activity: Activity<'a>) {
        self.activities.push(activity);
    }

    /// Add an agent to the bundle
    pub fn add_agent(&mut self, agent: Agent<'a>) {
        self.agents.push(agent);
    }

    /// Get total number of provenance statements
    pub fn size(&self) -> usize {
        self.entities.len() + self.activities.len() + self.agents.len()
    }

    /// Check if bundle is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty() && self.activities.is_empty() && self.agents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdf_model::Dictionary;
    use std::sync::Arc;

    #[test]
    fn test_entity_creation() {
        let dict = Arc::new(Dictionary::new());
        let doc_iri = dict.intern("http://example.org/doc1");
        let activity_iri = dict.intern("http://example.org/edit1");
        let agent_iri = dict.intern("http://example.org/alice");

        let entity = Entity::new(Node::iri(doc_iri))
            .generated_by(Node::iri(activity_iri))
            .attributed_to(Node::iri(agent_iri));

        assert!(entity.was_generated_by.is_some());
        assert_eq!(entity.was_attributed_to.len(), 1);
    }

    #[test]
    fn test_activity_creation() {
        let dict = Arc::new(Dictionary::new());
        let activity_iri = dict.intern("http://example.org/edit1");
        let doc_iri = dict.intern("http://example.org/doc1");
        let agent_iri = dict.intern("http://example.org/alice");

        let activity = Activity::new(Node::iri(activity_iri))
            .started_at("2025-11-27T10:00:00Z".to_string())
            .ended_at("2025-11-27T10:05:00Z".to_string())
            .associated_with(Node::iri(agent_iri))
            .generated_entity(Node::iri(doc_iri));

        assert!(activity.start_time.is_some());
        assert!(activity.end_time.is_some());
        assert_eq!(activity.was_associated_with.len(), 1);
        assert_eq!(activity.generated.len(), 1);
    }

    #[test]
    fn test_agent_creation() {
        let dict = Arc::new(Dictionary::new());
        let agent_iri = dict.intern("http://example.org/alice");

        let agent = Agent::new(Node::iri(agent_iri), AgentType::Person)
            .with_name("Alice".to_string())
            .with_email("alice@example.org".to_string());

        assert_eq!(agent.agent_type, AgentType::Person);
        assert_eq!(agent.name, Some("Alice".to_string()));
        assert_eq!(agent.email, Some("alice@example.org".to_string()));
    }

    #[test]
    fn test_provenance_bundle() {
        let dict = Arc::new(Dictionary::new());
        let doc_iri = dict.intern("http://example.org/doc1");
        let activity_iri = dict.intern("http://example.org/edit1");
        let agent_iri = dict.intern("http://example.org/alice");

        let mut bundle = ProvenanceBundle::new();

        bundle.add_entity(Entity::new(Node::iri(doc_iri)));
        bundle.add_activity(Activity::new(Node::iri(activity_iri)));
        bundle.add_agent(Agent::new(Node::iri(agent_iri), AgentType::Person));

        assert_eq!(bundle.size(), 3);
        assert!(!bundle.is_empty());
        assert_eq!(bundle.entities.len(), 1);
        assert_eq!(bundle.activities.len(), 1);
        assert_eq!(bundle.agents.len(), 1);
    }

    #[test]
    fn test_agent_type_iris() {
        assert_eq!(AgentType::Person.as_iri(), "http://www.w3.org/ns/prov#Person");
        assert_eq!(AgentType::Organization.as_iri(), "http://www.w3.org/ns/prov#Organization");
        assert_eq!(AgentType::SoftwareAgent.as_iri(), "http://www.w3.org/ns/prov#SoftwareAgent");
    }

    #[test]
    fn test_entity_derivation() {
        let dict = Arc::new(Dictionary::new());
        let doc1 = dict.intern("http://example.org/doc1");
        let doc2 = dict.intern("http://example.org/doc2");

        let entity = Entity::new(Node::iri(doc1))
            .derived_from(Node::iri(doc2));

        assert_eq!(entity.was_derived_from.len(), 1);
    }
}
