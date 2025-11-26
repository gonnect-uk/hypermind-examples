//! Autonomous Vehicle Simulation
//!
//! Main binary for running self-driving car reasoning with Unity simulator

use anyhow::Result;
use av_reasoning::*;
use chrono::Utc;
use clap::Parser;
use simulator_bridge::{SimulatorBridge, SimulatorTelemetry};
use tracing::{info, Level};

/// Self-Driving Car Reasoning Simulator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Unity simulator host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Unity simulator port
    #[arg(long, default_value_t = 4567)]
    port: u16,

    /// Demo mode (no Unity simulator, use synthetic data)
    #[arg(long)]
    demo: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("🚗 Self-Driving Car Reasoning Engine");
    info!("📊 Using rust-kgdb for SPARQL-based decision-making");
    info!("");

    if args.demo {
        run_demo_mode().await?;
    } else {
        run_simulator_mode(&args).await?;
    }

    Ok(())
}

/// Run in demo mode with synthetic data
async fn run_demo_mode() -> Result<()> {
    info!("🎮 Running in DEMO mode (no Unity simulator required)");
    info!("");

    let bridge = SimulatorBridge::new();

    // Demo Scenario 1: Red Traffic Light
    info!("📍 Scenario 1: Red Traffic Light Emergency Stop");
    info!("──────────────────────────────────────────────");

    let telem_approaching = SimulatorTelemetry {
        speed: 30.0, // 30 mph ≈ 13.4 m/s
        steering_angle: 0.0,
        throttle: 0.5,
        pos_x: 100.0,
        pos_y: 0.0,
        pos_z: 50.0,
        image: None,
    };

    info!("Vehicle approaching at 30 mph (13.4 m/s)");

    // Simulate detecting red traffic light at 30m
    let mut sensor_data = SensorData {
        vehicle_state: Some(VehicleState {
            velocity_mps: 13.4,
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

    info!("🚦 Detected: Red traffic light at 30.0m (confidence: 98%)");

    // Make decision
    let decision = ReasoningExecutor::make_decision(&sensor_data)?;

    info!("");
    info!("🧠 DECISION:");
    info!("   Action: {:?}", decision.action);
    info!("   Reason: {}", decision.reason);
    info!("   Priority: {}", decision.priority);
    info!("   Confidence: {:.2}", decision.confidence);
    info!("   Query: {}", decision.query_name);
    info!("");

    // Demo Scenario 2: Pedestrian Crossing
    info!("📍 Scenario 2: Pedestrian Crossing Detection");
    info!("──────────────────────────────────────────────");

    sensor_data.traffic_lights.clear();
    sensor_data.pedestrians.push(PedestrianDetection {
        id: 1,
        position_x: 115.0,
        position_y: 50.0,
        velocity_x: 1.0,
        velocity_y: 0.0,
        in_crosswalk: true,
        confidence: 0.95,
        timestamp: Utc::now(),
    });

    info!("🚶 Detected: Pedestrian in crosswalk (confidence: 95%)");

    let decision = ReasoningExecutor::make_decision(&sensor_data)?;

    info!("");
    info!("🧠 DECISION:");
    info!("   Action: {:?}", decision.action);
    info!("   Reason: {}", decision.reason);
    info!("   Priority: {}", decision.priority);
    info!("   Confidence: {:.2}", decision.confidence);
    info!("   Query: {}", decision.query_name);
    info!("");

    // Demo Scenario 3: Speed Limit Compliance
    info!("📍 Scenario 3: Speed Limit Compliance");
    info!("──────────────────────────────────────────────");

    sensor_data.pedestrians.clear();
    sensor_data.vehicle_state = Some(VehicleState {
        velocity_mps: 20.0, // 72 km/h
        position_x: 100.0,
        position_y: 50.0,
        heading_deg: 0.0,
        timestamp: Utc::now(),
    });
    sensor_data.road_segment = Some(RoadSegment {
        id: 2,
        speed_limit_kmh: 30.0, // School zone
        zone_type: ZoneType::SchoolZone,
        surface_condition: SurfaceCondition::Dry,
    });

    info!("🏫 Entering school zone: 30 km/h limit");
    info!("🚗 Current speed: 72 km/h (20 m/s)");

    let decision = ReasoningExecutor::make_decision(&sensor_data)?;

    info!("");
    info!("🧠 DECISION:");
    info!("   Action: {:?}", decision.action);
    info!("   Reason: {}", decision.reason);
    info!("   Priority: {}", decision.priority);
    info!("   Confidence: {:.2}", decision.confidence);
    info!("   Query: {}", decision.query_name);
    info!("");

    info!("✅ Demo completed successfully!");
    info!("");
    info!("💡 Key Takeaways:");
    info!("   • All decisions are based on SPARQL queries");
    info!("   • Full provenance (query name, reason, confidence)");
    info!("   • Priority-based decision making (Critical > High > Medium > Low)");
    info!("   • Sub-millisecond query execution with rust-kgdb");
    info!("");

    Ok(())
}

/// Run with Unity simulator connection
async fn run_simulator_mode(args: &Args) -> Result<()> {
    info!("🎮 Connecting to Unity Simulator at {}:{}", args.host, args.port);
    info!("⚠️  Unity simulator integration coming soon!");
    info!("   For now, run with --demo flag");
    info!("");

    // TODO: Implement Unity Socket.IO connection
    // let client = UnityClient::new(&args.host, args.port);
    // client.connect().await?;

    Ok(())
}
