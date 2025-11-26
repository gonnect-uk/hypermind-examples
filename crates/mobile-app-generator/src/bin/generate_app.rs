//! CLI tool to generate iOS apps from ontologies
//!
//! Usage:
//!   cargo run --bin generate_app -- /path/to/ontology.ttl /output/dir

use mobile_app_generator::{generate_app, GeneratorConfig, Platform};
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <ontology.ttl> <output_dir>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} insurance-risk-analyzer.ttl ./ios/GeneratedApps", args[0]);
        process::exit(1);
    }

    let ontology_path = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);

    if !ontology_path.exists() {
        eprintln!("❌ Error: Ontology file not found: {:?}", ontology_path);
        process::exit(1);
    }

    let config = GeneratorConfig {
        ontology_path: ontology_path.clone(),
        output_dir,
        platform: Platform::iOS,
        validate: true,
        verbose: true,
    };

    println!("🚀 Universal Mobile App Generator");
    println!("═══════════════════════════════════════");

    match generate_app(config) {
        Ok(output_path) => {
            println!("\n✅ SUCCESS!");
            println!("📱 Generated iOS app at: {:?}", output_path);
            println!("\nNext steps:");
            println!("1. Open {}/YourApp.xcodeproj in Xcode", output_path.display());
            println!("2. Build and run on simulator");
        }
        Err(e) => {
            eprintln!("\n❌ FAILED!");
            eprintln!("Error: {:?}", e);
            process::exit(1);
        }
    }
}
