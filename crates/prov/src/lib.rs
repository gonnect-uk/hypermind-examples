//! W3C PROV-O Provenance Ontology Support
//!
//! Complete implementation of W3C PROV-O (Provenance Ontology) for tracking data lineage
//! and provenance information.
//!
//! # Features
//!
//! - **Entity**: Physical, digital, or conceptual things
//! - **Activity**: Actions/processes that occur over time
//! - **Agent**: Actors responsible for activities or entities
//! - **Relationships**: wasGeneratedBy, used, wasAttributedTo, etc.
//! - **Provenance Bundles**: Collections of provenance statements
//!
//! # Example
//!
//! ```rust,ignore
//! use prov::{Entity, Activity, Agent, AgentType, ProvenanceBundle};
//! use rdf_model::{Node, Dictionary};
//!
//! let dict = Dictionary::new();
//! let doc = dict.intern("http://example.org/doc1");
//! let edit = dict.intern("http://example.org/edit1");
//! let alice = dict.intern("http://example.org/alice");
//!
//! // Create provenance record
//! let entity = Entity::new(Node::iri(doc))
//!     .generated_by(Node::iri(edit))
//!     .attributed_to(Node::iri(alice));
//!
//! let activity = Activity::new(Node::iri(edit))
//!     .started_at("2025-11-27T10:00:00Z".to_string())
//!     .associated_with(Node::iri(alice));
//!
//! let agent = Agent::new(Node::iri(alice), AgentType::Person)
//!     .with_name("Alice".to_string());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod types;

pub use types::{Entity, Activity, Agent, AgentType, ProvenanceBundle};

/// PROV namespace
pub const PROV_NS: &str = "http://www.w3.org/ns/prov#";

/// PROV-O Entity class
pub const ENTITY: &str = "http://www.w3.org/ns/prov#Entity";

/// PROV-O Activity class
pub const ACTIVITY: &str = "http://www.w3.org/ns/prov#Activity";

/// PROV-O Agent class
pub const AGENT: &str = "http://www.w3.org/ns/prov#Agent";

/// wasGeneratedBy property
pub const WAS_GENERATED_BY: &str = "http://www.w3.org/ns/prov#wasGeneratedBy";

/// used property
pub const USED: &str = "http://www.w3.org/ns/prov#used";

/// wasAttributedTo property
pub const WAS_ATTRIBUTED_TO: &str = "http://www.w3.org/ns/prov#wasAttributedTo";

/// wasAssociatedWith property
pub const WAS_ASSOCIATED_WITH: &str = "http://www.w3.org/ns/prov#wasAssociatedWith";

/// wasDerivedFrom property
pub const WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";

/// actedOnBehalfOf property
pub const ACTED_ON_BEHALF_OF: &str = "http://www.w3.org/ns/prov#actedOnBehalfOf";

/// Provenance record
#[derive(Debug, Clone)]
pub struct ProvenanceRecord {
    /// Entity URI
    pub entity: String,
    /// Activity URI
    pub activity: Option<String>,
    /// Agent URI
    pub agent: Option<String>,
    /// Timestamp
    pub timestamp: Option<String>,
}

impl ProvenanceRecord {
    /// Creates a new provenance record
    pub fn new(entity: String) -> Self {
        Self {
            entity,
            activity: None,
            agent: None,
            timestamp: None,
        }
    }

    /// Sets the activity
    pub fn with_activity(mut self, activity: String) -> Self {
        self.activity = Some(activity);
        self
    }

    /// Sets the agent
    pub fn with_agent(mut self, agent: String) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Sets the timestamp
    pub fn with_timestamp(mut self, timestamp: String) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_record() {
        let record = ProvenanceRecord::new("http://example.org/doc1".to_string())
            .with_activity("http://example.org/edit1".to_string())
            .with_agent("http://example.org/alice".to_string());

        assert_eq!(record.entity, "http://example.org/doc1");
        assert_eq!(record.activity, Some("http://example.org/edit1".to_string()));
        assert_eq!(record.agent, Some("http://example.org/alice".to_string()));
    }
}
