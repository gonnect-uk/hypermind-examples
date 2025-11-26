//! Autonomous Vehicle Ontology
//!
//! This crate defines the RDF vocabulary and namespaces for autonomous vehicle
//! reasoning using rust-kgdb.

use rdf_model::Dictionary;
use std::sync::Arc;

/// Autonomous Vehicle namespace
pub const AV_NS: &str = "http://zenya.com/ontology/av#";

/// Road infrastructure namespace
pub const ROAD_NS: &str = "http://zenya.com/ontology/road#";

/// Sensor namespace
pub const SENSOR_NS: &str = "http://zenya.com/ontology/sensor#";

/// Action namespace
pub const ACTION_NS: &str = "http://zenya.com/ontology/action#";

/// Rule namespace
pub const RULE_NS: &str = "http://zenya.com/ontology/rule#";

/// Hypergraph namespace
pub const HYPER_NS: &str = "http://zenya.com/ontology/hypergraph#";

/// Autonomous Vehicle vocabulary terms
pub struct AVVocab {
    // Classes
    pub vehicle: &'static str,
    pub traffic_light: &'static str,
    pub pedestrian: &'static str,
    pub obstacle: &'static str,
    pub lane: &'static str,
    pub intersection: &'static str,
    pub crosswalk: &'static str,

    // Properties - Spatial
    pub has_position: &'static str,
    pub has_velocity: &'static str,
    pub has_acceleration: &'static str,
    pub distance_to: &'static str,
    pub in_lane: &'static str,
    pub heading: &'static str,
    pub trajectory: &'static str,

    // Properties - State
    pub state: &'static str,
    pub confidence: &'static str,
    pub timestamp: &'static str,

    // Properties - Detection
    pub detects: &'static str,
    pub detected_by: &'static str,
}

impl Default for AVVocab {
    fn default() -> Self {
        Self {
            // Classes
            vehicle: "Vehicle",
            traffic_light: "TrafficLight",
            pedestrian: "Pedestrian",
            obstacle: "Obstacle",
            lane: "Lane",
            intersection: "Intersection",
            crosswalk: "Crosswalk",

            // Properties - Spatial
            has_position: "hasPosition",
            has_velocity: "hasVelocity",
            has_acceleration: "hasAcceleration",
            distance_to: "distanceTo",
            in_lane: "inLane",
            heading: "heading",
            trajectory: "trajectory",

            // Properties - State
            state: "state",
            confidence: "confidence",
            timestamp: "timestamp",

            // Properties - Detection
            detects: "detects",
            detected_by: "detectedBy",
        }
    }
}

/// Action vocabulary terms
pub struct ActionVocab {
    // Actions
    pub brake: &'static str,
    pub accelerate: &'static str,
    pub steer_left: &'static str,
    pub steer_right: &'static str,
    pub emergency_brake: &'static str,
    pub yield_action: &'static str,
    pub change_lane: &'static str,

    // Properties
    pub intensity: &'static str,
    pub duration: &'static str,
    pub priority: &'static str,
    pub triggered_by: &'static str,
    pub reason: &'static str,
}

impl Default for ActionVocab {
    fn default() -> Self {
        Self {
            // Actions
            brake: "Brake",
            accelerate: "Accelerate",
            steer_left: "SteerLeft",
            steer_right: "SteerRight",
            emergency_brake: "EmergencyBrake",
            yield_action: "Yield",
            change_lane: "ChangeLane",

            // Properties
            intensity: "intensity",
            duration: "duration",
            priority: "priority",
            triggered_by: "triggeredBy",
            reason: "reason",
        }
    }
}

/// Road vocabulary terms
pub struct RoadVocab {
    // Classes
    pub road_segment: &'static str,
    pub speed_limit_zone: &'static str,

    // Properties
    pub speed_limit: &'static str,
    pub zone_type: &'static str,
    pub surface_condition: &'static str,
    pub friction_coefficient: &'static str,
    pub adjacent_lane: &'static str,
}

impl Default for RoadVocab {
    fn default() -> Self {
        Self {
            // Classes
            road_segment: "RoadSegment",
            speed_limit_zone: "SpeedLimitZone",

            // Properties
            speed_limit: "speedLimit",
            zone_type: "zoneType",
            surface_condition: "surfaceCondition",
            friction_coefficient: "frictionCoefficient",
            adjacent_lane: "adjacentLane",
        }
    }
}

/// Build a complete URI from namespace and local name
#[inline]
pub fn build_uri(namespace: &str, local_name: &str) -> String {
    format!("{}{}", namespace, local_name)
}

/// Vocabulary helper for constructing URIs
pub struct VocabHelper {
    dict: Arc<Dictionary>,
    pub av: AVVocab,
    pub action: ActionVocab,
    pub road: RoadVocab,
}

impl VocabHelper {
    pub fn new(dict: Arc<Dictionary>) -> Self {
        Self {
            dict,
            av: AVVocab::default(),
            action: ActionVocab::default(),
            road: RoadVocab::default(),
        }
    }

    /// Get interned URI for AV namespace term
    pub fn av_uri(&self, term: &str) -> &str {
        let uri = build_uri(AV_NS, term);
        self.dict.intern(&uri)
    }

    /// Get interned URI for Action namespace term
    pub fn action_uri(&self, term: &str) -> &str {
        let uri = build_uri(ACTION_NS, term);
        self.dict.intern(&uri)
    }

    /// Get interned URI for Road namespace term
    pub fn road_uri(&self, term: &str) -> &str {
        let uri = build_uri(ROAD_NS, term);
        self.dict.intern(&uri)
    }

    /// Get interned URI for Sensor namespace term
    pub fn sensor_uri(&self, term: &str) -> &str {
        let uri = build_uri(SENSOR_NS, term);
        self.dict.intern(&uri)
    }

    /// Get interned URI for Hypergraph namespace term
    pub fn hyper_uri(&self, term: &str) -> &str {
        let uri = build_uri(HYPER_NS, term);
        self.dict.intern(&uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_uri() {
        assert_eq!(
            build_uri(AV_NS, "Vehicle"),
            "http://zenya.com/ontology/av#Vehicle"
        );
    }

    #[test]
    fn test_vocab_helper() {
        let dict = Arc::new(Dictionary::new());
        let vocab = VocabHelper::new(dict);

        let vehicle_uri = vocab.av_uri("Vehicle");
        assert_eq!(vehicle_uri, "http://zenya.com/ontology/av#Vehicle");
    }
}
