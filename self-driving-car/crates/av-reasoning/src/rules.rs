//! Traffic rules and safety constraints

use crate::models::*;

/// Traffic rules engine
pub struct TrafficRules;

impl TrafficRules {
    /// Calculate stopping distance for given speed and surface condition
    /// Formula: d = v² / (2 * a * f)
    /// where:
    ///   d = stopping distance (m)
    ///   v = velocity (m/s)
    ///   a = max deceleration (5 m/s² assumed)
    ///   f = friction coefficient (surface dependent)
    pub fn calculate_stopping_distance(
        velocity_mps: f32,
        surface_condition: SurfaceCondition,
    ) -> f32 {
        let max_decel = 5.0; // m/s²
        let friction = surface_condition.friction_coefficient();

        (velocity_mps * velocity_mps) / (2.0 * max_decel * friction)
    }

    /// Calculate safe stopping distance with safety margin
    pub fn safe_stopping_distance(
        velocity_mps: f32,
        surface_condition: SurfaceCondition,
    ) -> f32 {
        let min_stopping = Self::calculate_stopping_distance(velocity_mps, surface_condition);
        min_stopping + 10.0 // 10m safety margin
    }

    /// Calculate brake intensity based on distance to obstacle
    /// Returns 0.0 (no brake) to 1.0 (full brake)
    pub fn calculate_brake_intensity(
        velocity_mps: f32,
        distance_m: f32,
        surface_condition: SurfaceCondition,
    ) -> f32 {
        let min_stopping = Self::calculate_stopping_distance(velocity_mps, surface_condition);
        let safe_stopping = Self::safe_stopping_distance(velocity_mps, surface_condition);

        if distance_m < min_stopping {
            // CRITICAL: Full emergency brake
            1.0
        } else if distance_m < safe_stopping {
            // Gradual brake: interpolate between 0.6 and 1.0
            let ratio = 1.0 - (distance_m - min_stopping) / (safe_stopping - min_stopping);
            0.6 + (0.4 * ratio)
        } else {
            // Safe: No braking needed
            0.0
        }
    }

    /// Check if red traffic light requires braking
    pub fn should_brake_for_red_light(
        velocity_mps: f32,
        distance_m: f32,
        surface_condition: SurfaceCondition,
        confidence: f32,
    ) -> bool {
        // Only act on high-confidence detections
        if confidence < 0.85 {
            return false;
        }

        let safe_distance = Self::safe_stopping_distance(velocity_mps, surface_condition);
        distance_m <= safe_distance
    }

    /// Priority for pedestrian crossing (always CRITICAL)
    pub fn pedestrian_priority() -> Priority {
        Priority::Critical
    }

    /// Check if vehicle is in blind spot
    /// Blind spot defined as: -5m (behind) to +10m (ahead)
    pub fn is_in_blind_spot(relative_position_m: f32) -> bool {
        relative_position_m > -5.0 && relative_position_m < 10.0
    }

    /// Calculate target speed for speed limit compliance
    /// Returns 90% of limit as safety margin
    pub fn calculate_target_speed(limit_kmh: f32) -> f32 {
        (limit_kmh / 3.6) * 0.9 // Convert to m/s and apply 90% margin
    }

    /// Get priority for speed limit zone type
    pub fn speed_limit_priority(zone_type: ZoneType) -> Priority {
        match zone_type {
            ZoneType::SchoolZone => Priority::High,
            ZoneType::Residential => Priority::Medium,
            ZoneType::Urban => Priority::Medium,
            ZoneType::Highway => Priority::Low,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stopping_distance_dry() {
        // At 50 km/h (13.89 m/s) on dry road
        let velocity = 13.89;
        let dist = TrafficRules::calculate_stopping_distance(velocity, SurfaceCondition::Dry);

        // Expected: v² / (2*a*f) = 13.89² / (2*5*1.0) ≈ 19.3m
        assert!((dist - 19.3).abs() < 1.0);
    }

    #[test]
    fn test_stopping_distance_wet() {
        // At 50 km/h (13.89 m/s) on wet road
        let velocity = 13.89;
        let dist = TrafficRules::calculate_stopping_distance(velocity, SurfaceCondition::Wet);

        // Expected: 19.3 / 0.7 ≈ 27.6m (40% longer)
        assert!((dist - 27.6).abs() < 1.0);
    }

    #[test]
    fn test_brake_intensity_critical() {
        // Vehicle at 15 m/s, obstacle at 20m, min stopping = 22.5m
        let intensity = TrafficRules::calculate_brake_intensity(15.0, 20.0, SurfaceCondition::Dry);
        assert_eq!(intensity, 1.0); // Full brake
    }

    #[test]
    fn test_brake_intensity_safe() {
        // Vehicle at 10 m/s, obstacle at 50m, safe stopping = 20m
        let intensity = TrafficRules::calculate_brake_intensity(10.0, 50.0, SurfaceCondition::Dry);
        assert_eq!(intensity, 0.0); // No brake
    }

    #[test]
    fn test_blind_spot_detection() {
        assert!(TrafficRules::is_in_blind_spot(0.0)); // Directly beside
        assert!(TrafficRules::is_in_blind_spot(-2.0)); // 2m behind
        assert!(TrafficRules::is_in_blind_spot(5.0)); // 5m ahead
        assert!(!TrafficRules::is_in_blind_spot(-10.0)); // Far behind
        assert!(!TrafficRules::is_in_blind_spot(15.0)); // Far ahead
    }

    #[test]
    fn test_target_speed() {
        // 30 km/h school zone → target = 30 * 0.9 / 3.6 = 7.5 m/s
        let target = TrafficRules::calculate_target_speed(30.0);
        assert!((target - 7.5).abs() < 0.1);
    }

    #[test]
    fn test_speed_limit_priority() {
        assert_eq!(
            TrafficRules::speed_limit_priority(ZoneType::SchoolZone),
            Priority::High
        );
        assert_eq!(
            TrafficRules::speed_limit_priority(ZoneType::Highway),
            Priority::Low
        );
    }
}
