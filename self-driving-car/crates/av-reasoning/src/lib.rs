//! Autonomous Vehicle Reasoning Engine
//!
//! This crate implements SPARQL-based reasoning for self-driving cars using rust-kgdb.

pub mod queries;
pub mod rules;
pub mod executor;
pub mod models;

use anyhow::Result;
use rdf_model::{Dictionary, Node, Triple, Quad};
use storage::{QuadStore, InMemoryBackend};
use std::sync::Arc;

pub use executor::ReasoningExecutor;
pub use models::*;

/// Main reasoning engine for autonomous vehicle decision-making
pub struct ReasoningEngine {
    /// Knowledge graph storage
    pub quad_store: QuadStore<InMemoryBackend>,

    /// Ego vehicle identifier
    ego_vehicle_uri: String,
}

impl ReasoningEngine {
    /// Create a new reasoning engine
    pub fn new() -> Self {
        let quad_store = QuadStore::new_in_memory();

        Self {
            quad_store,
            ego_vehicle_uri: "http://zenya.com/vehicle/ego".to_string(),
        }
    }

    /// Get the dictionary from the quad store
    pub fn dictionary(&self) -> &Arc<Dictionary> {
        self.quad_store.dictionary()
    }

    /// Insert a triple into the knowledge graph
    pub fn insert_triple(&mut self, subject: Node, predicate: Node, object: Node) -> Result<()> {
        let triple = Triple::new(subject, predicate, object);
        let quad = Quad::from_triple(triple);
        self.quad_store.insert(quad)?;
        Ok(())
    }

    /// Get the ego vehicle URI as a Node
    pub fn ego_vehicle(&self) -> Node {
        let dict = self.dictionary();
        Node::iri(dict.intern(&self.ego_vehicle_uri))
    }

    /// Update vehicle state (position, velocity, etc.)
    pub fn update_vehicle_state(&mut self, state: VehicleState) -> Result<()> {
        use av_ontology::*;

        let dict = Arc::clone(self.dictionary());
        let ego_uri = dict.intern(&self.ego_vehicle_uri);
        let ego = Node::iri(ego_uri);
        let vocab = VocabHelper::new(Arc::clone(&dict));

        // Vehicle velocity
        let velocity_pred = Node::iri(vocab.av_uri(vocab.av.has_velocity));
        let velocity_obj = Node::literal_typed(
            dict.intern(&state.velocity_mps.to_string()),
            dict.intern("http://www.w3.org/2001/XMLSchema#float"),
        );
        self.insert_triple(ego.clone(), velocity_pred, velocity_obj)?;

        // Vehicle position (simplified - x,y coordinates)
        let pos_pred = Node::iri(vocab.av_uri(vocab.av.has_position));
        let pos_obj = Node::literal_str(
            dict.intern(&format!("({}, {})", state.position_x, state.position_y))
        );
        self.insert_triple(ego.clone(), pos_pred, pos_obj)?;

        // Timestamp
        let ts_pred = Node::iri(vocab.av_uri(vocab.av.timestamp));
        let ts_obj = Node::literal_typed(
            dict.intern(&state.timestamp.to_rfc3339()),
            dict.intern("http://www.w3.org/2001/XMLSchema#dateTime"),
        );
        self.insert_triple(ego, ts_pred, ts_obj)?;

        Ok(())
    }

    /// Detect traffic light and add to knowledge graph
    pub fn detect_traffic_light(
        &mut self,
        id: u32,
        state: TrafficLightState,
        distance_m: f32,
        confidence: f32,
    ) -> Result<()> {
        use av_ontology::*;

        let dict = Arc::clone(self.dictionary());
        let vocab = VocabHelper::new(Arc::clone(&dict));

        let tl_uri = format!("http://zenya.com/traffic_light/{}", id);
        let tl_node = Node::iri(dict.intern(&tl_uri));

        // Type: TrafficLight
        let type_pred = Node::iri(dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"));
        let tl_type = Node::iri(vocab.av_uri(vocab.av.traffic_light));
        self.insert_triple(tl_node.clone(), type_pred, tl_type)?;

        // State: red/yellow/green
        let state_pred = Node::iri(vocab.av_uri(vocab.av.state));
        let state_str = match state {
            TrafficLightState::Red => "red",
            TrafficLightState::Yellow => "yellow",
            TrafficLightState::Green => "green",
        };
        let state_obj = Node::literal_str(dict.intern(state_str));
        self.insert_triple(tl_node.clone(), state_pred, state_obj)?;

        // Distance to ego vehicle
        let ego_uri = dict.intern(&self.ego_vehicle_uri);
        let ego = Node::iri(ego_uri);
        let dist_pred = Node::iri(vocab.av_uri(vocab.av.distance_to));
        let dist_obj = Node::literal_typed(
            dict.intern(&distance_m.to_string()),
            dict.intern("http://www.w3.org/2001/XMLSchema#float"),
        );
        self.insert_triple(ego, dist_pred.clone(), tl_node.clone())?;

        // Also store as property of traffic light
        self.insert_triple(tl_node.clone(), dist_pred, dist_obj)?;

        // Confidence
        let conf_pred = Node::iri(vocab.sensor_uri("confidence"));
        let conf_obj = Node::literal_typed(
            dict.intern(&confidence.to_string()),
            dict.intern("http://www.w3.org/2001/XMLSchema#float"),
        );
        self.insert_triple(tl_node, conf_pred, conf_obj)?;

        Ok(())
    }

    /// Check if emergency braking is required for red traffic light
    /// Returns (should_brake, intensity)
    pub fn check_red_light_braking(&self) -> Result<(bool, f32)> {
        // This is a simplified version - full SPARQL query in queries module

        // For now, use a simple ASK query
        // In full implementation, we'll parse and execute the SPARQL from queries/red-traffic-light.rq

        // Placeholder logic:
        // TODO: Implement actual SPARQL query execution

        Ok((false, 0.0))
    }
}

impl Default for ReasoningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_create_reasoning_engine() {
        let engine = ReasoningEngine::new();
        assert_eq!(engine.ego_vehicle_uri, "http://zenya.com/vehicle/ego");
    }

    #[test]
    fn test_update_vehicle_state() {
        let mut engine = ReasoningEngine::new();

        let state = VehicleState {
            velocity_mps: 15.5,
            position_x: 100.0,
            position_y: 50.0,
            heading_deg: 90.0,
            timestamp: Utc::now(),
        };

        let result = engine.update_vehicle_state(state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_traffic_light() {
        let mut engine = ReasoningEngine::new();

        let result = engine.detect_traffic_light(
            1,
            TrafficLightState::Red,
            30.0,
            0.98,
        );
        assert!(result.is_ok());
    }
}
