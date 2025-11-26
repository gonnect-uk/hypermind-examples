//! SPARQL query templates for autonomous vehicle reasoning

/// SPARQL query: Check if red traffic light requires emergency braking
pub const RED_TRAFFIC_LIGHT_QUERY: &str = r#"
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX action: <http://zenya.com/ontology/action#>
PREFIX sensor: <http://zenya.com/ontology/sensor#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX prov: <http://www.w3.org/ns/prov#>

# Check if emergency braking is required for red light
ASK {
  # Get ego vehicle state
  <http://zenya.com/vehicle/ego> av:hasVelocity ?speedMps .

  # Traffic light detection
  ?tl a av:TrafficLight ;
      av:state "red"^^xsd:string ;
      av:distanceTo ?distance ;
      sensor:confidence ?conf .

  # Only trust high-confidence detections (>85%)
  FILTER(?conf > 0.85)

  # Calculate stopping distance: d = v^2 / (2 * a)
  # Assuming max deceleration a = 5 m/s^2
  # stoppingDist = v^2 / 10
  BIND((?speedMps * ?speedMps) / 10.0 AS ?minStoppingDist)

  # Add 10m safety margin
  BIND(?minStoppingDist + 10.0 AS ?safeStoppingDist)

  # Check if within stopping distance
  FILTER(?distance <= ?safeStoppingDist)
}
"#;

/// SPARQL query: Calculate brake intensity for red light
pub const RED_LIGHT_BRAKE_INTENSITY_QUERY: &str = r#"
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX sensor: <http://zenya.com/ontology/sensor#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?intensity WHERE {
  <http://zenya.com/vehicle/ego> av:hasVelocity ?speedMps .

  ?tl a av:TrafficLight ;
      av:state "red"^^xsd:string ;
      av:distanceTo ?distance ;
      sensor:confidence ?conf .

  FILTER(?conf > 0.85)

  # Calculate minimum stopping distance
  BIND((?speedMps * ?speedMps) / 10.0 AS ?minStoppingDist)
  BIND(?minStoppingDist + 10.0 AS ?safeStoppingDist)

  # Only if we need to brake
  FILTER(?distance <= ?safeStoppingDist)

  # Calculate brake intensity (stronger if closer)
  # If distance < minStoppingDist: full brake (1.0)
  # Otherwise: gradual brake (0.6 - 1.0)
  BIND(
    IF(?distance < ?minStoppingDist,
       1.0,
       0.6 + (0.4 * (1.0 - ?distance / ?safeStoppingDist)))
    AS ?intensity
  )
}
ORDER BY DESC(?intensity)
LIMIT 1
"#;

/// SPARQL query: Pedestrian crossing detection
pub const PEDESTRIAN_CROSSING_QUERY: &str = r#"
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX sensor: <http://zenya.com/ontology/sensor#>

ASK {
  # Pedestrian detected
  ?ped a av:Pedestrian ;
       av:inCrosswalk "true"^^xsd:boolean ;
       sensor:confidence ?conf .

  # High confidence only
  FILTER(?conf > 0.9)
}
"#;

/// SPARQL query: Lane change safety check
pub const LANE_CHANGE_SAFETY_QUERY: &str = r#"
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX road: <http://zenya.com/ontology/road#>

ASK {
  # Get current and target lanes
  <http://zenya.com/vehicle/ego> av:inLane ?currentLane .
  ?currentLane road:adjacentLane ?targetLane .

  # Check for vehicles in target lane
  ?otherVehicle a av:Vehicle ;
                av:inLane ?targetLane ;
                av:relativePosition ?relPos .

  # Blind spot zone: 5m behind to 10m ahead
  FILTER(?relPos > -5.0 && ?relPos < 10.0)
}
# Returns TRUE if unsafe (vehicle detected), FALSE if safe
"#;

/// SPARQL query: Speed limit compliance check
pub const SPEED_LIMIT_COMPLIANCE_QUERY: &str = r#"
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX road: <http://zenya.com/ontology/road#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?currentSpeed ?limitMps ?excessSpeed ?targetSpeed WHERE {
  # Get current speed (m/s) and road segment
  <http://zenya.com/vehicle/ego> av:hasVelocity ?currentSpeed ;
                                  av:inRoadSegment ?segment .

  # Get speed limit (stored in km/h, convert to m/s)
  ?segment road:speedLimit ?limitKmh .

  BIND(?limitKmh / 3.6 AS ?limitMps)

  # Check if speeding
  FILTER(?currentSpeed > ?limitMps)

  # Calculate excess speed
  BIND((?currentSpeed - ?limitMps) AS ?excessSpeed)

  # Target speed: 90% of limit (safety margin)
  BIND(?limitMps * 0.9 AS ?targetSpeed)
}
"#;

/// Get SPARQL query by name
pub fn get_query(name: &str) -> Option<&'static str> {
    match name {
        "red-traffic-light" => Some(RED_TRAFFIC_LIGHT_QUERY),
        "red-light-intensity" => Some(RED_LIGHT_BRAKE_INTENSITY_QUERY),
        "pedestrian-crossing" => Some(PEDESTRIAN_CROSSING_QUERY),
        "lane-change-safety" => Some(LANE_CHANGE_SAFETY_QUERY),
        "speed-limit-compliance" => Some(SPEED_LIMIT_COMPLIANCE_QUERY),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_query() {
        assert!(get_query("red-traffic-light").is_some());
        assert!(get_query("pedestrian-crossing").is_some());
        assert!(get_query("nonexistent").is_none());
    }

    #[test]
    fn test_queries_not_empty() {
        assert!(!RED_TRAFFIC_LIGHT_QUERY.is_empty());
        assert!(!RED_LIGHT_BRAKE_INTENSITY_QUERY.is_empty());
    }
}
