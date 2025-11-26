use mobile_app_generator::{generate_app, GeneratorConfig, Platform};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "generate-mobile-app")]
#[command(about = "Generate iOS/Android apps from ontologies", long_about = None)]
struct Cli {
    /// Path to ontology TTL file
    #[arg(short, long)]
    ontology: PathBuf,
    
    /// Output directory
    #[arg(short, long)]
    output: PathBuf,
    
    /// Target platform (ios or android)
    #[arg(short, long, default_value = "ios")]
    platform: String,
    
    /// Disable validation
    #[arg(long)]
    no_validate: bool,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let platform = match cli.platform.to_lowercase().as_str() {
        "ios" => Platform::iOS,
        "android" => Platform::Android,
        _ => {
            eprintln!("❌ Invalid platform: {}. Use 'ios' or 'android'", cli.platform);
            std::process::exit(1);
        }
    };
    
    let config = GeneratorConfig {
        ontology_path: cli.ontology,
        output_dir: cli.output,
        platform,
        validate: !cli.no_validate,
        verbose: cli.verbose,
    };
    
    match generate_app(config) {
        Ok(path) => {
            println!("✅ Generated app at: {:?}", path);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }
}
