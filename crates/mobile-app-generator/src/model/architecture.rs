//! Architecture for Extensible Ontology-Driven App Generation
//!
//! Design Pattern: Template Method + Strategy + Dependency Injection
//!
//! Core Principle: Generated code is NEVER modified by users.
//! Users extend through well-defined extension points.

use serde::{Serialize, Deserialize};

/// Extension point configuration
/// Users define their customizations HERE, not in generated code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// Custom theme overrides (colors, fonts, spacing)
    pub theme_overrides: Option<ThemeOverrides>,

    /// Custom view templates (user-provided SwiftUI views)
    pub custom_views: Vec<CustomViewDefinition>,

    /// Custom field renderers (for domain-specific inputs)
    pub custom_field_renderers: Vec<CustomFieldRenderer>,

    /// Custom validators (beyond SHACL)
    pub custom_validators: Vec<CustomValidator>,

    /// Custom actions (app-specific behaviors)
    pub custom_actions: Vec<CustomAction>,

    /// Analytics/logging hooks
    pub analytics_config: Option<AnalyticsConfig>,

    /// Localization overrides
    pub localization: Option<LocalizationConfig>,
}

/// Theme overrides - users can customize without touching core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeOverrides {
    /// Override any color from AppTheme
    pub colors: std::collections::HashMap<String, String>,

    /// Override fonts
    pub fonts: std::collections::HashMap<String, FontConfig>,

    /// Override spacing/padding
    pub spacing: std::collections::HashMap<String, f64>,

    /// Override corner radius
    pub corner_radius: std::collections::HashMap<String, f64>,

    /// Override shadows
    pub shadows: std::collections::HashMap<String, ShadowConfig>,

    /// Dark mode specific overrides
    pub dark_mode_overrides: Option<Box<ThemeOverrides>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: f64,
    pub weight: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowConfig {
    pub color: String,
    pub radius: f64,
    pub x: f64,
    pub y: f64,
}

/// Custom view that user provides (injected into generated app)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomViewDefinition {
    /// Unique identifier
    pub id: String,

    /// When to show this view (replaces or extends)
    pub injection_point: InjectionPoint,

    /// Path to user's SwiftUI file
    pub swift_file_path: String,

    /// View name in Swift
    pub view_name: String,

    /// Props passed from generated code
    pub props: Vec<PropDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InjectionPoint {
    /// Replace a generated view entirely
    Replace { view_id: String },

    /// Add before a generated view
    Before { view_id: String },

    /// Add after a generated view
    After { view_id: String },

    /// Wrap a generated view
    Wrap { view_id: String },

    /// Add to navigation tabs
    Tab { order: i32 },

    /// Add to detail view sections
    DetailSection { entity_type: String, order: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropDefinition {
    pub name: String,
    pub swift_type: String,
    pub binding: String,
}

/// Custom field renderer for domain-specific inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldRenderer {
    /// Which field type to customize
    pub field_type: String,

    /// Path to custom SwiftUI view
    pub swift_file_path: String,

    /// View name
    pub view_name: String,

    /// Condition when to use (optional)
    pub condition: Option<String>,
}

/// Custom validator beyond SHACL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomValidator {
    pub id: String,
    pub name: String,
    pub swift_function: String,
    pub error_message: String,
}

/// Custom action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAction {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub swift_function: String,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub provider: AnalyticsProvider,
    pub track_screens: bool,
    pub track_queries: bool,
    pub track_errors: bool,
    pub custom_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsProvider {
    None,
    Firebase,
    Mixpanel,
    Amplitude,
    Custom { endpoint: String },
}

/// Localization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizationConfig {
    pub default_language: String,
    pub supported_languages: Vec<String>,
    pub strings_file_path: Option<String>,
}

//=============================================================================
// GENERATED CODE STRUCTURE
//=============================================================================

/// Structure of generated iOS project
/// This is FIXED - users cannot modify this structure
#[derive(Debug, Clone)]
pub struct GeneratedProjectStructure {
    // CORE (Never modify)
    pub core_dir: &'static str,

    // EXTENSIONS (User modifiable)
    pub extensions_dir: &'static str,

    // RESOURCES (User can add)
    pub resources_dir: &'static str,
}

impl Default for GeneratedProjectStructure {
    fn default() -> Self {
        Self {
            // Core generated code - NEVER TOUCH
            core_dir: "Core/Generated",

            // User extensions - SAFE TO MODIFY
            extensions_dir: "Extensions",

            // Resources - User can add
            resources_dir: "Resources",
        }
    }
}

/// File categories in generated project
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileCategory {
    /// Core generated - NEVER modify
    CoreGenerated,

    /// Extension point - User CAN modify
    ExtensionPoint,

    /// User custom - Fully user controlled
    UserCustom,

    /// Configuration - User CAN modify
    Configuration,
}

/// Metadata for each generated file
#[derive(Debug, Clone)]
pub struct GeneratedFileMetadata {
    pub path: String,
    pub category: FileCategory,
    pub regenerate_safe: bool,
    pub description: String,
}

//=============================================================================
// GENERATION PHASES
//=============================================================================

/// Generation phases - clear separation of concerns
#[derive(Debug, Clone, Copy)]
pub enum GenerationPhase {
    /// Phase 1: Parse ontology → Model
    Parse,

    /// Phase 2: Validate with SHACL
    Validate,

    /// Phase 3: Generate core code
    GenerateCore,

    /// Phase 4: Apply extensions
    ApplyExtensions,

    /// Phase 5: Generate configuration
    GenerateConfig,

    /// Phase 6: Copy resources
    CopyResources,

    /// Phase 7: Generate project files
    GenerateProject,
}

//=============================================================================
// HOOKS FOR EXTENSION
//=============================================================================

/// Hook points where users can inject behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookPoint {
    /// Before app initialization
    AppWillInitialize,

    /// After app initialization
    AppDidInitialize,

    /// Before SPARQL query execution
    BeforeQuery { query_id: String },

    /// After SPARQL query execution
    AfterQuery { query_id: String },

    /// Before view appears
    ViewWillAppear { view_id: String },

    /// After view appears
    ViewDidAppear { view_id: String },

    /// On field value change
    FieldDidChange { field_id: String },

    /// On validation
    WillValidate { field_id: String },

    /// On error
    OnError { error_type: String },
}

/// Hook implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub point: HookPoint,
    pub swift_function: String,
    pub async_: bool,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self {
            theme_overrides: None,
            custom_views: Vec::new(),
            custom_field_renderers: Vec::new(),
            custom_validators: Vec::new(),
            custom_actions: Vec::new(),
            analytics_config: None,
            localization: None,
        }
    }
}
