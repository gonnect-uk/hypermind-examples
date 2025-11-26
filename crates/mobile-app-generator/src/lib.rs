//! Universal Mobile App Generator
//! 
//! Generates iOS/Android apps dynamically from universal-meta-mobile-ontology.ttl files.
//! 
//! # Architecture
//! 
//! 1. Parse TTL ontology → strongly-typed Rust structs
//! 2. Validate against SHACL shapes
//! 3. Generate Swift/Kotlin code using Tera templates
//! 4. Output complete Xcode/Android Studio project
//! 
//! # Example
//! 
//! ```rust
//! use mobile_app_generator::{generate_app, GeneratorConfig};
//! 
//! let config = GeneratorConfig {
//!     ontology_path: "insurance-risk-analyzer.ttl".into(),
//!     output_dir: "build/InsuranceApp".into(),
//!     platform: Platform::iOS,
//! };
//! 
//! generate_app(config)?;
//! ```

pub mod model;
pub mod parser;
pub mod generator;
pub mod error;

pub use error::{Result, GeneratorError};
pub use model::*;
pub use parser::OntologyParser;
pub use generator::{SwiftGenerator, KotlinGenerator};

use std::path::PathBuf;

/// Platform target for app generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    iOS,
    Android,
}

/// Generator configuration
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Path to ontology TTL file
    pub ontology_path: PathBuf,
    
    /// Output directory for generated app
    pub output_dir: PathBuf,
    
    /// Target platform
    pub platform: Platform,
    
    /// Enable validation (SHACL)
    pub validate: bool,
    
    /// Verbose logging
    pub verbose: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            ontology_path: PathBuf::from("app.ttl"),
            output_dir: PathBuf::from("build/GeneratedApp"),
            platform: Platform::iOS,
            validate: true,
            verbose: false,
        }
    }
}

/// Main entry point: Generate mobile app from ontology
pub fn generate_app(config: GeneratorConfig) -> Result<PathBuf> {
    if config.verbose {
        println!("🚀 Universal Mobile App Generator");
        println!("   Ontology: {:?}", config.ontology_path);
        println!("   Output: {:?}", config.output_dir);
        println!("   Platform: {:?}", config.platform);
    }
    
    // Parse ontology
    if config.verbose {
        println!("📖 Parsing ontology...");
    }
    let parser = OntologyParser::new();
    let app = parser.parse_file(&config.ontology_path)?;
    
    if config.verbose {
        println!("   ✅ Parsed: {}", app.title);
        println!("   📝 Persona: {}", app.persona.name);
        println!("   🎯 Business Value: {}", app.business_value.metric);
    }
    
    // Validate
    if config.validate {
        if config.verbose {
            println!("✔️  Validating against SHACL...");
        }
        validate_app(&app)?;
        if config.verbose {
            println!("   ✅ Validation passed");
        }
    }
    
    // Generate code
    if config.verbose {
        println!("🎨 Generating {} code...", match config.platform {
            Platform::iOS => "Swift",
            Platform::Android => "Kotlin",
        });
    }
    
    let output_path = match config.platform {
        Platform::iOS => {
            let generator = SwiftGenerator::new();
            generator.generate(&app, &config.output_dir)?
        }
        Platform::Android => {
            let generator = KotlinGenerator::new();
            generator.generate(&app, &config.output_dir)?
        }
    };
    
    if config.verbose {
        println!("✅ Generated app at: {:?}", output_path);
    }
    
    Ok(output_path)
}

/// Validate app against SHACL shapes
fn validate_app(app: &MobileApplication) -> Result<()> {
    // Title validation
    if app.title.len() < 3 || app.title.len() > 50 {
        return Err(GeneratorError::Validation(
            format!("Title length must be 3-50 chars, got {}", app.title.len())
        ));
    }
    
    // Persona validation
    if app.persona.name.is_empty() {
        return Err(GeneratorError::Validation(
            "Persona name cannot be empty".into()
        ));
    }
    
    // Offline validation
    if !app.offline_capable {
        return Err(GeneratorError::Validation(
            "App must be offline-capable (offlineCapable=true)".into()
        ));
    }
    
    // View validation
    if let ViewDefinition::Form(form) = &app.home_view {
        if form.fields.is_empty() {
            return Err(GeneratorError::Validation(
                "Form view must have at least one field".into()
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation() {
        let app = MobileApplication {
            title: "Test App".into(),
            persona: BusinessPersona {
                name: "Test User".into(),
                description: "Test description".into(),
            },
            business_value: BusinessValue {
                problem: "Test problem statement".into(),
                solution: "Test solution with SPARQL reasoning".into(),
                metric: "35x faster than APIs".into(),
            },
            icon: "star.fill".into(),
            primary_color: "#007AFF".into(),
            accent_color: None,
            home_view: ViewDefinition::Form(FormView {
                label: "Test Form".into(),
                background_color: None,
                fields: vec![
                    FieldDefinition::Text(TextField {
                        label: "Search".into(),
                        placeholder: Some("Enter term".into()),
                        order: 0,
                        required: true,
                        data_type: "xsd:string".into(),
                        binds_to_property: "test:search".into(),
                        validation: None,
                        min_length: 1,
                        max_length: 100,
                        multiline: false,
                    })
                ],
                execute_query: QueryTemplate {
                    query_type: QueryType::Select,
                    template: "SELECT ?s WHERE { ?s ?p ?o }".into(),
                    parameters: vec![],
                    result_bindings: vec!["?s".into()],
                    result_view: None,
                    requires_internet: false,
                    expected_query_time: Some("2.78 microseconds".into()),
                },
                result_view: None,
            }),
            offline_capable: true,
        };
        
        assert!(validate_app(&app).is_ok());
    }
}
