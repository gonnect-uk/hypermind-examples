//! Web Dashboard Server for Self-Driving Car Visualization

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use av_reasoning::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::info;

/// Dashboard state (shared across requests)
struct DashboardState {
    current_decision: Arc<Mutex<Option<Decision>>>,
    sensor_data: Arc<Mutex<SensorData>>,
    scenario_index: Arc<Mutex<usize>>,
}

/// API response for current state
#[derive(Serialize)]
struct StateResponse {
    decision: Option<Decision>,
    sensor_data: SensorData,
    scenario_name: String,
}

/// Get current state
async fn get_state(state: web::Data<DashboardState>) -> Result<HttpResponse> {
    let decision = state.current_decision.lock().unwrap().clone();
    let sensor_data = state.sensor_data.lock().unwrap().clone();
    let scenario_idx = *state.scenario_index.lock().unwrap();

    let scenario_name = match scenario_idx {
        0 => "Idle",
        1 => "Red Traffic Light",
        2 => "Pedestrian Crossing",
        3 => "Speed Limit Violation",
        _ => "Unknown",
    };

    Ok(HttpResponse::Ok().json(StateResponse {
        decision,
        sensor_data,
        scenario_name: scenario_name.to_string(),
    }))
}

/// Trigger next scenario
async fn next_scenario(state: web::Data<DashboardState>) -> Result<HttpResponse> {
    let mut scenario_idx = state.scenario_index.lock().unwrap();
    *scenario_idx = (*scenario_idx + 1) % 4; // Cycle through 4 scenarios

    let new_idx = *scenario_idx;
    drop(scenario_idx);

    // Generate scenario data
    let (sensor_data, decision) = generate_scenario(new_idx);

    // Update state
    {
        let mut data = state.sensor_data.lock().unwrap();
        *data = sensor_data;
    }
    {
        let mut dec = state.current_decision.lock().unwrap();
        *dec = Some(decision);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"scenario": new_idx})))
}

/// Generate scenario data
fn generate_scenario(scenario: usize) -> (SensorData, Decision) {
    match scenario {
        1 => {
            // Scenario 1: Red Traffic Light
            let sensor_data = SensorData {
                vehicle_state: Some(VehicleState {
                    velocity_mps: 13.4, // 30 mph
                    position_x: 100.0,
                    position_y: 50.0,
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
                road_segment: Some(RoadSegment {
                    id: 1,
                    speed_limit_kmh: 50.0,
                    zone_type: ZoneType::Urban,
                    surface_condition: SurfaceCondition::Dry,
                }),
                ..Default::default()
            };

            let decision = ReasoningExecutor::make_decision(&sensor_data).unwrap();
            (sensor_data, decision)
        }
        2 => {
            // Scenario 2: Pedestrian Crossing
            let sensor_data = SensorData {
                vehicle_state: Some(VehicleState {
                    velocity_mps: 10.0,
                    position_x: 100.0,
                    position_y: 50.0,
                    heading_deg: 0.0,
                    timestamp: Utc::now(),
                }),
                pedestrians: vec![PedestrianDetection {
                    id: 1,
                    position_x: 115.0,
                    position_y: 50.0,
                    velocity_x: 1.0,
                    velocity_y: 0.0,
                    in_crosswalk: true,
                    confidence: 0.95,
                    timestamp: Utc::now(),
                }],
                road_segment: Some(RoadSegment {
                    id: 1,
                    speed_limit_kmh: 50.0,
                    zone_type: ZoneType::Urban,
                    surface_condition: SurfaceCondition::Dry,
                }),
                ..Default::default()
            };

            let decision = ReasoningExecutor::make_decision(&sensor_data).unwrap();
            (sensor_data, decision)
        }
        3 => {
            // Scenario 3: Speed Limit
            let sensor_data = SensorData {
                vehicle_state: Some(VehicleState {
                    velocity_mps: 20.0, // 72 km/h
                    position_x: 100.0,
                    position_y: 50.0,
                    heading_deg: 0.0,
                    timestamp: Utc::now(),
                }),
                road_segment: Some(RoadSegment {
                    id: 2,
                    speed_limit_kmh: 30.0, // School zone
                    zone_type: ZoneType::SchoolZone,
                    surface_condition: SurfaceCondition::Dry,
                }),
                ..Default::default()
            };

            let decision = ReasoningExecutor::make_decision(&sensor_data).unwrap();
            (sensor_data, decision)
        }
        _ => {
            // Idle
            let sensor_data = SensorData::default();
            let decision = Decision {
                action: ControlAction::Maintain,
                query_name: "idle".to_string(),
                reason: "Waiting for scenario".to_string(),
                priority: Priority::Low,
                confidence: 1.0,
                timestamp: Utc::now(),
            };
            (sensor_data, decision)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("🚗 Self-Driving Car Web Dashboard");
    info!("🌐 Starting server on http://localhost:3000");

    // Initialize state
    let state = web::Data::new(DashboardState {
        current_decision: Arc::new(Mutex::new(None)),
        sensor_data: Arc::new(Mutex::new(SensorData::default())),
        scenario_index: Arc::new(Mutex::new(0)),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .route("/api/state", web::get().to(get_state))
            .route("/api/next", web::post().to(next_scenario))
            .service(fs::Files::new("/", "./crates/web-dashboard/static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
