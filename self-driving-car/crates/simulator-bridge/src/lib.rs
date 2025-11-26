//! Bridge between Udacity Unity Simulator and rust-kgdb reasoning engine

use anyhow::{Context, Result};
use av_reasoning::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod unity_client;

/// Simulator telemetry data from Unity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorTelemetry {
    /// Speed in mph (Unity format)
    pub speed: f32,

    /// Steering angle (-1.0 to 1.0)
    pub steering_angle: f32,

    /// Throttle (0.0 to 1.0)
    pub throttle: f32,

    /// Position X
    pub pos_x: f32,

    /// Position Y (Unity uses Y as vertical, but we'll use it as Z)
    pub pos_y: f32,

    /// Position Z (becomes our Y)
    pub pos_z: f32,

    /// Base64 encoded camera image (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Convert Unity telemetry to sensor data
pub struct TelemetryConverter;

impl TelemetryConverter {
    /// Convert mph to m/s
    fn mph_to_mps(mph: f32) -> f32 {
        mph * 0.44704
    }

    /// Convert Unity telemetry to VehicleState
    pub fn to_vehicle_state(telem: &SimulatorTelemetry) -> VehicleState {
        VehicleState {
            velocity_mps: Self::mph_to_mps(telem.speed),
            position_x: telem.pos_x,
            position_y: telem.pos_z, // Unity's Z → our Y
            heading_deg: telem.steering_angle * 45.0, // Approximate
            timestamp: Utc::now(),
        }
    }

    /// Convert Unity telemetry to SensorData
    /// Note: Real implementation would parse camera image for object detection
    pub fn to_sensor_data(telem: &SimulatorTelemetry) -> SensorData {
        SensorData {
            vehicle_state: Some(Self::to_vehicle_state(telem)),
            traffic_lights: vec![], // TODO: Parse from camera
            pedestrians: vec![],    // TODO: Parse from camera
            vehicles: vec![],       // TODO: Parse from camera
            road_segment: None,     // TODO: Get from map data
        }
    }
}

/// Bridge between simulator and reasoning engine
pub struct SimulatorBridge {
    /// Reasoning executor
    executor: ReasoningExecutor,

    /// Latest sensor data
    sensor_data: Arc<RwLock<SensorData>>,

    /// Latest decision
    last_decision: Arc<RwLock<Option<Decision>>>,
}

impl SimulatorBridge {
    pub fn new() -> Self {
        Self {
            executor: ReasoningExecutor,
            sensor_data: Arc::new(RwLock::new(SensorData::default())),
            last_decision: Arc::new(RwLock::new(None)),
        }
    }

    /// Process telemetry from Unity simulator
    pub async fn process_telemetry(&self, telem: SimulatorTelemetry) -> Result<serde_json::Value> {
        // Convert to sensor data
        let sensor_data = TelemetryConverter::to_sensor_data(&telem);

        // Store sensor data
        {
            let mut data = self.sensor_data.write().await;
            *data = sensor_data.clone();
        }

        // Make decision
        let decision = ReasoningExecutor::make_decision(&sensor_data)
            .context("Failed to make decision")?;

        // Store decision
        {
            let mut last = self.last_decision.write().await;
            *last = Some(decision.clone());
        }

        // Convert to Unity control command
        let command = decision.action.to_unity_command();

        tracing::info!(
            "Decision: {:?}, Priority: {}, Reason: {}",
            decision.action,
            decision.priority,
            decision.reason
        );

        Ok(command)
    }

    /// Get latest sensor data (for dashboard)
    pub async fn get_sensor_data(&self) -> SensorData {
        self.sensor_data.read().await.clone()
    }

    /// Get latest decision (for dashboard)
    pub async fn get_last_decision(&self) -> Option<Decision> {
        self.last_decision.read().await.clone()
    }
}

impl Default for SimulatorBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mph_to_mps() {
        // 50 mph ≈ 22.35 m/s
        let mps = TelemetryConverter::mph_to_mps(50.0);
        assert!((mps - 22.35).abs() < 0.1);
    }

    #[test]
    fn test_telemetry_conversion() {
        let telem = SimulatorTelemetry {
            speed: 30.0, // mph
            steering_angle: 0.0,
            throttle: 0.5,
            pos_x: 100.0,
            pos_y: 0.0,
            pos_z: 50.0,
            image: None,
        };

        let state = TelemetryConverter::to_vehicle_state(&telem);
        assert!((state.velocity_mps - 13.41).abs() < 0.1); // 30 mph ≈ 13.41 m/s
        assert_eq!(state.position_x, 100.0);
        assert_eq!(state.position_y, 50.0);
    }

    #[tokio::test]
    async fn test_simulator_bridge() {
        let bridge = SimulatorBridge::new();

        let telem = SimulatorTelemetry {
            speed: 30.0,
            steering_angle: 0.0,
            throttle: 0.5,
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            image: None,
        };

        let command = bridge.process_telemetry(telem).await.unwrap();
        assert!(command["throttle"].is_number() || command["brake"].is_number());
    }
}
