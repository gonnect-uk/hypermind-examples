//! W3C PROV-O Provenance Ontology Support (Stub)
//!
//! This crate provides types and structures for W3C PROV-O provenance tracking.
//! Full implementation to be completed.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

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

/// Type of provenance agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Generic agent
    Agent,
    /// Person (human)
    Person,
    /// Organization
    Organization,
    /// Software agent
    SoftwareAgent,
}

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
