//! Decision executor for autonomous vehicle reasoning

use crate::models::*;
use crate::rules::TrafficRules;
use anyhow::Result;
use chrono::Utc;

/// Reasoning executor that makes decisions based on sensor data
pub struct ReasoningExecutor;

impl ReasoningExecutor {
    /// Process sensor data and generate decision
    pub fn make_decision(sensor_data: &SensorData) -> Result<Decision> {
        // Priority-based decision making:
        // 1. CRITICAL: Pedestrian in crosswalk → Emergency brake
        // 2. CRITICAL: Red traffic light within stopping distance → Brake
        // 3. HIGH: Vehicle in blind spot → Deny lane change
        // 4. MEDIUM: Speed limit violation → Decelerate
        // 5. LOW: Maintain current course

        // Check pedestrian crossing (HIGHEST PRIORITY)
        if let Some(decision) = Self::check_pedestrian_crossing(sensor_data) {
            return Ok(decision);
        }

        // Check red traffic light
        if let Some(decision) = Self::check_red_traffic_light(sensor_data) {
            return Ok(decision);
        }

        // Check speed limit
        if let Some(decision) = Self::check_speed_limit(sensor_data) {
            return Ok(decision);
        }

        // Default: Maintain
        Ok(Decision {
            action: ControlAction::Maintain,
            query_name: "default".to_string(),
            reason: "No immediate hazards detected".to_string(),
            priority: Priority::Low,
            confidence: 1.0,
            timestamp: Utc::now(),
        })
    }

    /// Check if pedestrian crossing requires emergency brake
    fn check_pedestrian_crossing(sensor_data: &SensorData) -> Option<Decision> {
        for ped in &sensor_data.pedestrians {
            if ped.in_crosswalk && ped.confidence > 0.9 {
                return Some(Decision {
                    action: ControlAction::EmergencyBrake,
                    query_name: "pedestrian-crossing".to_string(),
                    reason: format!("Pedestrian {} in crosswalk", ped.id),
                    priority: Priority::Critical,
                    confidence: ped.confidence,
                    timestamp: Utc::now(),
                });
            }
        }
        None
    }

    /// Check if red traffic light requires braking
    fn check_red_traffic_light(sensor_data: &SensorData) -> Option<Decision> {
        let vehicle_state = sensor_data.vehicle_state.as_ref()?;
        let surface = sensor_data
            .road_segment
            .as_ref()
            .map(|r| r.surface_condition)
            .unwrap_or(SurfaceCondition::Dry);

        for tl in &sensor_data.traffic_lights {
            if tl.state == TrafficLightState::Red {
                if TrafficRules::should_brake_for_red_light(
                    vehicle_state.velocity_mps,
                    tl.distance_m,
                    surface,
                    tl.confidence,
                ) {
                    let intensity = TrafficRules::calculate_brake_intensity(
                        vehicle_state.velocity_mps,
                        tl.distance_m,
                        surface,
                    );

                    let stopping_dist = TrafficRules::safe_stopping_distance(
                        vehicle_state.velocity_mps,
                        surface,
                    );

                    return Some(Decision {
                        action: ControlAction::Brake { intensity },
                        query_name: "red-traffic-light".to_string(),
                        reason: format!(
                            "Red traffic light at {:.1}m (stopping distance: {:.1}m, brake: {:.2})",
                            tl.distance_m, stopping_dist, intensity
                        ),
                        priority: Priority::Critical,
                        confidence: tl.confidence,
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        None
    }

    /// Check speed limit compliance
    fn check_speed_limit(sensor_data: &SensorData) -> Option<Decision> {
        let vehicle_state = sensor_data.vehicle_state.as_ref()?;
        let road_segment = sensor_data.road_segment.as_ref()?;

        let limit_mps = road_segment.speed_limit_kmh / 3.6;

        if vehicle_state.velocity_mps > limit_mps {
            let target_speed = TrafficRules::calculate_target_speed(road_segment.speed_limit_kmh);
            let excess = vehicle_state.velocity_mps - limit_mps;
            let priority = TrafficRules::speed_limit_priority(road_segment.zone_type);

            // Deceleration intensity: 0.3 for minor, 0.6 for major violations
            let intensity = if excess > 5.0 { 0.6 } else { 0.3 };

            return Some(Decision {
                action: ControlAction::Brake { intensity },
                query_name: "speed-limit-compliance".to_string(),
                reason: format!(
                    "Speeding: {:.1} km/h in {:.0} km/h {:?} zone (target: {:.1} m/s)",
                    vehicle_state.velocity_mps * 3.6,
                    road_segment.speed_limit_kmh,
                    road_segment.zone_type,
                    target_speed
                ),
                priority,
                confidence: 1.0,
                timestamp: Utc::now(),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedestrian_crossing_decision() {
        let sensor_data = SensorData {
            vehicle_state: Some(VehicleState {
                velocity_mps: 10.0,
                position_x: 0.0,
                position_y: 0.0,
                heading_deg: 0.0,
                timestamp: Utc::now(),
            }),
            pedestrians: vec![PedestrianDetection {
                id: 1,
                position_x: 10.0,
                position_y: 0.0,
                velocity_x: 1.0,
                velocity_y: 0.0,
                in_crosswalk: true,
                confidence: 0.95,
                timestamp: Utc::now(),
            }],
            ..Default::default()
        };

        let decision = ReasoningExecutor::make_decision(&sensor_data).unwrap();
        assert_eq!(decision.action, ControlAction::EmergencyBrake);
        assert_eq!(decision.priority, Priority::Critical);
    }

    #[test]
    fn test_red_light_decision() {
        let sensor_data = SensorData {
            vehicle_state: Some(VehicleState {
                velocity_mps: 15.0, // ~54 km/h
                position_x: 0.0,
                position_y: 0.0,
                heading_deg: 0.0,
                timestamp: Utc::now(),
            }),
            traffic_lights: vec![TrafficLightDetection {
                id: 1,
                state: TrafficLightState::Red,
                distance_m: 30.0,
                confidence: 0.98,
                timestamp: Utc::now(),
            }],
            ..Default::default()
        };

        let decision = ReasoningExecutor::make_decision(&sensor_data).unwrap();
        match decision.action {
            ControlAction::Brake { intensity } => {
                assert!(intensity > 0.0);
                assert!(intensity <= 1.0);
            }
            _ => panic!("Expected Brake action"),
        }
        assert_eq!(decision.priority, Priority::Critical);
    }

    #[test]
    fn test_default_maintain() {
        let sensor_data = SensorData::default();

        let decision = ReasoningExecutor::make_decision(&sensor_data).unwrap();
        assert_eq!(decision.action, ControlAction::Maintain);
        assert_eq!(decision.priority, Priority::Low);
    }
}
