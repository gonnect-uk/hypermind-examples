//! Integration tests for Swift code generation

use mobile_app_generator::{SwiftGenerator, MobileApplication, BusinessPersona, BusinessValue, ViewDefinition, FormView, FieldDefinition, TextField, QueryTemplate, QueryType};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_generate_basic_swift_app() {
    // Create test app
    let app = MobileApplication {
        title: "Test App".into(),
        persona: BusinessPersona {
            name: "Test User".into(),
            description: "Test user for testing".into(),
        },
        business_value: BusinessValue {
            problem: "Need to test".into(),
            solution: "Test solution".into(),
            metric: "100% tested".into(),
        },
        icon: "star.fill".into(),
        primary_color: "#007AFF".into(),
        accent_color: None,
        home_view: ViewDefinition::Form(FormView {
            label: "Search Form".into(),
            background_color: None,
            fields: vec![
                FieldDefinition::Text(TextField {
                    label: "Query".into(),
                    placeholder: Some("Enter search term".into()),
                    order: 0,
                    required: true,
                    data_type: "xsd:string".into(),
                    binds_to_property: "test:query".into(),
                    validation: None,
                    min_length: 1,
                    max_length: 100,
                    multiline: false,
                })
            ],
            execute_query: QueryTemplate {
                query_type: QueryType::Select,
                template: "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10".into(),
                parameters: vec![],
                result_bindings: vec!["?s".into(), "?p".into(), "?o".into()],
                result_view: None,
                requires_internet: false,
                expected_query_time: Some("2.78 microseconds".into()),
            },
            result_view: None,
        }),
        offline_capable: true,
    };

    // Generate
    let temp_dir = TempDir::new().unwrap();
    let generator = SwiftGenerator::new();
    let output_path = generator.generate(&app, temp_dir.path()).unwrap();

    // Verify files exist
    assert!(output_path.exists());
    assert!(output_path.join("TestApp").exists());
    assert!(output_path.join("TestApp/TestAppApp.swift").exists());
    assert!(output_path.join("TestApp/ContentView.swift").exists());
    assert!(output_path.join("TestApp/SPARQLService.swift").exists());
    assert!(output_path.join("TestApp/Info.plist").exists());

    // Read and verify content
    let app_swift = std::fs::read_to_string(output_path.join("TestApp/TestAppApp.swift")).unwrap();
    assert!(app_swift.contains("@main"));
    assert!(app_swift.contains("struct TestAppApp: App"));

    let content_view = std::fs::read_to_string(output_path.join("TestApp/ContentView.swift")).unwrap();
    assert!(content_view.contains("struct ContentView: View"));
    assert!(content_view.contains("TextField"));

    let sparql_service = std::fs::read_to_string(output_path.join("TestApp/SPARQLService.swift")).unwrap();
    assert!(sparql_service.contains("class SPARQLService"));
    assert!(sparql_service.contains("executeSPARQL"));

    println!("✅ Swift generation test passed!");
    println!("📁 Generated files in: {:?}", output_path);
}
