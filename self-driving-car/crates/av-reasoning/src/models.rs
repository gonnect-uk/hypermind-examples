//! Data models for autonomous vehicle reasoning

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Vehicle state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleState {
    /// Velocity in meters per second
    pub velocity_mps: f32,

    /// X position (meters)
    pub position_x: f32,

    /// Y position (meters)
    pub position_y: f32,

    /// Heading in degrees (0-360, 0 = North)
    pub heading_deg: f32,

    /// Timestamp of state measurement
    pub timestamp: DateTime<Utc>,
}

/// Traffic light states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrafficLightState {
    Red,
    Yellow,
    Green,
}

/// Traffic light detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficLightDetection {
    pub id: u32,
    pub state: TrafficLightState,
    pub distance_m: f32,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

/// Pedestrian detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedestrianDetection {
    pub id: u32,
    pub position_x: f32,
    pub position_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub in_crosswalk: bool,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

/// Vehicle detection (other vehicles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleDetection {
    pub id: u32,
    pub relative_position_m: f32, // Relative to ego vehicle (-ve = behind, +ve = ahead)
    pub relative_velocity_mps: f32,
    pub lane_id: u32,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

/// Road segment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadSegment {
    pub id: u32,
    pub speed_limit_kmh: f32,
    pub zone_type: ZoneType,
    pub surface_condition: SurfaceCondition,
}

/// Zone types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    Highway,
    Urban,
    Residential,
    SchoolZone,
}

/// Surface conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceCondition {
    Dry,
    Wet,
    Icy,
    Snow,
}

impl SurfaceCondition {
    /// Get friction coefficient for surface condition
    pub fn friction_coefficient(&self) -> f32 {
        match self {
            SurfaceCondition::Dry => 1.0,
            SurfaceCondition::Wet => 0.7,
            SurfaceCondition::Icy => 0.3,
            SurfaceCondition::Snow => 0.5,
        }
    }
}

/// Control action types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlAction {
    /// Brake with given intensity (0.0 - 1.0)
    Brake { intensity: f32 },

    /// Accelerate with given intensity (0.0 - 1.0)
    Accelerate { intensity: f32 },

    /// Steer with angle in degrees (-45 to +45)
    Steer { angle_deg: f32 },

    /// Maintain current speed and direction
    Maintain,

    /// Emergency stop
    EmergencyBrake,
}

/// Decision with provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Control action to execute
    pub action: ControlAction,

    /// SPARQL query that generated this decision
    pub query_name: String,

    /// Human-readable reason
    pub reason: String,

    /// Priority level
    pub priority: Priority,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,

    /// Timestamp of decision
    pub timestamp: DateTime<Utc>,
}

/// Decision priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "LOW"),
            Priority::Medium => write!(f, "MEDIUM"),
            Priority::High => write!(f, "HIGH"),
            Priority::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Sensor data bundle (all sensors at once)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensorData {
    pub vehicle_state: Option<VehicleState>,
    pub traffic_lights: Vec<TrafficLightDetection>,
    pub pedestrians: Vec<PedestrianDetection>,
    pub vehicles: Vec<VehicleDetection>,
    pub road_segment: Option<RoadSegment>,
}

impl ControlAction {
    /// Get JSON representation for Unity simulator
    pub fn to_unity_command(&self) -> serde_json::Value {
        match self {
            ControlAction::Brake { intensity } => {
                serde_json::json!({
                    "steering_angle": 0.0,
                    "throttle": 0.0,
                    "brake": intensity,
                })
            }
            ControlAction::Accelerate { intensity } => {
                serde_json::json!({
                    "steering_angle": 0.0,
                    "throttle": intensity,
                    "brake": 0.0,
                })
            }
            ControlAction::Steer { angle_deg } => {
                // Normalize to -1.0 to +1.0 range
                let normalized_angle = (angle_deg / 45.0).clamp(-1.0, 1.0);
                serde_json::json!({
                    "steering_angle": normalized_angle,
                    "throttle": 0.3,  // Maintain some speed while steering
                    "brake": 0.0,
                })
            }
            ControlAction::Maintain => {
                serde_json::json!({
                    "steering_angle": 0.0,
                    "throttle": 0.3,
                    "brake": 0.0,
                })
            }
            ControlAction::EmergencyBrake => {
                serde_json::json!({
                    "steering_angle": 0.0,
                    "throttle": 0.0,
                    "brake": 1.0,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_friction() {
        assert_eq!(SurfaceCondition::Dry.friction_coefficient(), 1.0);
        assert_eq!(SurfaceCondition::Wet.friction_coefficient(), 0.7);
        assert_eq!(SurfaceCondition::Icy.friction_coefficient(), 0.3);
    }

    #[test]
    fn test_control_action_to_unity() {
        let brake_action = ControlAction::Brake { intensity: 0.8 };
        let cmd = brake_action.to_unity_command();
        assert_eq!(cmd["brake"], 0.8);
        assert_eq!(cmd["throttle"], 0.0);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }
}
