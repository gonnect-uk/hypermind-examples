//! Strongly-typed Rust models matching universal-meta-mobile-ontology
//!
//! 100% coverage of the Zenya Universal Meta-Ontology for professional
//! ontology-driven mobile app generation. No hardcoding, pure magic.
//!
//! Architecture: Core generation is UNTOUCHABLE. Users extend via ExtensionConfig.

use serde::{Serialize, Deserialize};

pub mod architecture;
pub use architecture::*;

/// Complete mobile application specification
///
/// Core fields are generated from ontology - NEVER modify generated code.
/// Use `extensions` to customize behavior without touching core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileApplication {
    // === CORE (Generated from Ontology) ===
    pub title: String,
    pub persona: BusinessPersona,
    pub business_value: BusinessValue,
    pub icon: String,
    pub primary_color: String,
    pub accent_color: Option<String>,
    pub home_view: ViewDefinition,
    pub additional_views: Vec<ViewDefinition>,
    pub offline_capable: bool,
    pub datasets: Vec<DatasetReference>,
    pub navigation: NavigationStructure,
    pub theme: AppTheme,

    // === EXTENSIONS (User Customizable) ===
    /// User-defined extensions - safe to modify
    #[serde(default)]
    pub extensions: ExtensionConfig,

    /// Hooks for user-defined behavior injection
    #[serde(default)]
    pub hooks: Vec<Hook>,

    /// Business rules from ontology
    #[serde(default)]
    pub business_rules: Vec<BusinessRule>,
}

/// Reference to bundled RDF dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetReference {
    pub name: String,
    pub filename: String,
    pub format: DataFormat,
    pub graph_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    Turtle,
    NTriples,
    RdfXml,
    JsonLd,
}

/// App navigation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationStructure {
    pub style: NavigationStyle,
    pub tabs: Vec<TabDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationStyle {
    TabBar,
    Sidebar,
    Stack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabDefinition {
    pub label: String,
    pub icon: String,
    pub view: ViewDefinition,
}

/// App theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTheme {
    pub primary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub card_background: String,
    pub success_color: String,
    pub warning_color: String,
    pub error_color: String,
    pub font_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessPersona {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessValue {
    pub problem: String,
    pub solution: String,
    pub metric: String,
}

/// All view types supported by the generator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "view_type")]
pub enum ViewDefinition {
    Form(FormView),
    List(ListView),
    Detail(DetailView),
    Dashboard(DashboardView),
    HowItWorks(HowItWorksView),
}

/// Input form with fields bound to ontology properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormView {
    pub label: String,
    pub background_color: Option<String>,
    pub fields: Vec<FieldDefinition>,
    pub execute_query: QueryTemplate,
    pub result_view: Option<Box<ViewDefinition>>,
    pub how_it_works_panel: Option<HowItWorksPanel>,
}

/// Scrollable list displaying SPARQL query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListView {
    pub label: String,
    pub background_color: Option<String>,
    pub binds_to: Vec<String>,
    pub item_template: ListItemTemplate,
    pub detail_view: Option<Box<DetailView>>,
    pub empty_state: EmptyState,
    pub query: QueryTemplate,
    pub sort_options: Vec<SortOption>,
    pub filter_options: Vec<FilterOption>,
}

/// Entity detail screen with "How It Works" reasoning panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailView {
    pub label: String,
    pub background_color: Option<String>,
    pub sections: Vec<DetailSection>,
    pub how_it_works_panel: HowItWorksPanel,
    pub actions: Vec<ActionButton>,
    pub binds_to_class: String,
}

/// Summary metrics and charts showing business insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardView {
    pub label: String,
    pub background_color: Option<String>,
    pub metrics: Vec<MetricCard>,
    pub charts: Vec<ChartDefinition>,
    pub quick_actions: Vec<ActionButton>,
    pub refresh_interval: Option<u32>,
}

/// View showing how the tech works (SPARQL, reasoning, graph)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowItWorksView {
    pub label: String,
    pub panels: Vec<HowItWorksPanel>,
}

/// List item template for customizing list display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItemTemplate {
    pub title_binding: String,
    pub subtitle_binding: Option<String>,
    pub image_binding: Option<String>,
    pub badge_binding: Option<String>,
    pub badge_color_binding: Option<String>,
    pub trailing_text_binding: Option<String>,
}

/// Empty state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyState {
    pub title: String,
    pub message: String,
    pub icon: String,
    pub action: Option<ActionButton>,
}

/// Detail view section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailSection {
    pub title: String,
    pub fields: Vec<DetailField>,
    pub is_collapsible: bool,
}

/// Field in detail view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailField {
    pub label: String,
    pub value_binding: String,
    pub format: ValueFormat,
    pub icon: Option<String>,
}

/// Value formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueFormat {
    Text,
    Number { decimal_places: u8 },
    Currency { code: String },
    Percentage,
    Date { format: String },
    DateTime { format: String },
    Boolean { true_text: String, false_text: String },
    Badge { color_binding: Option<String> },
    Link,
}

/// Metric card for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCard {
    pub title: String,
    pub value_query: QueryTemplate,
    pub icon: String,
    pub color: String,
    pub trend: Option<TrendIndicator>,
    pub unit: Option<String>,
}

/// Trend indicator for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendIndicator {
    pub direction: TrendDirection,
    pub percentage: f64,
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Neutral,
}

/// Chart definition for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDefinition {
    pub title: String,
    pub chart_type: ChartType,
    pub data_query: QueryTemplate,
    pub x_axis_binding: String,
    pub y_axis_binding: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Donut,
    Area,
}

/// Action button
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionButton {
    pub label: String,
    pub icon: String,
    pub style: ButtonStyle,
    pub action: ActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Destructive,
    Link,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Navigate { view: Box<ViewDefinition> },
    ExecuteQuery { query: QueryTemplate },
    Share,
    Copy { binding: String },
    OpenUrl { url_binding: String },
}

/// Sort option for lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOption {
    pub label: String,
    pub binding: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Filter option for lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOption {
    pub label: String,
    pub binding: String,
    pub filter_type: FilterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
    Text,
    Select { options: Vec<String> },
    Range { min: f64, max: f64 },
    Boolean,
}

/// All field types supported by the generator (9 types from ontology)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "field_type")]
pub enum FieldDefinition {
    Text(TextField),
    Number(NumberField),
    Date(DateField),
    DateTime(DateTimeField),
    Currency(CurrencyField),
    Percentage(PercentageField),
    Picker(PickerField),
    Toggle(ToggleField),
    Search(SearchField),
    Boolean(BooleanField),
}

/// DateTime picker field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTimeField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub format: String,
    pub min_date_time: Option<String>,
    pub max_date_time: Option<String>,
}

/// Percentage field (0-100)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentageField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub decimal_places: i32,
    pub show_slider: bool,
}

/// Toggle/switch field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleField {
    pub label: String,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub default_value: bool,
    pub on_label: Option<String>,
    pub off_label: Option<String>,
}

/// Search field with autocomplete from SPARQL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub autocomplete_query: Option<QueryTemplate>,
    pub min_characters: i32,
    pub debounce_ms: i32,
}

// Common field trait for all field types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub min_length: i32,
    pub max_length: i32,
    pub multiline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub decimal_places: i32,
    pub unit: Option<String>,  // Added missing field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub format: String,  // Renamed from date_format
    pub min_date: Option<String>,
    pub max_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub currency_code: String,
    pub min_value: Option<f64>,  // Added field
    pub max_value: Option<f64>,  // Added field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickerField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub options: Vec<PickerOption>,
    pub sparql_query: Option<String>,  // Added field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickerOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanField {
    pub label: String,
    pub placeholder: Option<String>,
    pub order: i32,
    pub required: bool,
    pub data_type: String,
    pub binds_to_property: String,
    pub validation: Option<ValidationRule>,
    pub default_value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: String,
    pub pattern: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTemplate {
    pub query_type: QueryType,
    pub template: String,
    pub parameters: Vec<FieldDefinition>,
    pub result_bindings: Vec<String>,
    pub result_view: Option<Box<ViewDefinition>>,
    pub requires_internet: bool,
    pub expected_query_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Construct,
    Ask,
    Describe,
}

/// How It Works panel - the crown jewel showing tech power
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowItWorksPanel {
    pub label: String,
    pub is_expandable: bool,
    pub expanded_by_default: bool,
    pub graph_visualization: Option<GraphVisualizationConfig>,
    pub sparql_query_display: Option<SPARQLQueryDisplayConfig>,
    pub triple_output: Option<TripleOutputConfig>,
    pub performance_metrics: Option<PerformanceMetricsConfig>,
    pub reasoning_explanation: Option<ReasoningExplanationConfig>,
}

/// Interactive RDF graph with level-by-level expansion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphVisualizationConfig {
    pub root_entity_binding: String,
    pub max_depth: i32,
    pub layout: GraphLayout,
    pub node_colors: NodeColorScheme,
    pub show_labels: bool,
    pub enable_zoom: bool,
    pub enable_pan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphLayout {
    ForceDirected,
    Hierarchical,
    Radial,
    Grid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeColorScheme {
    pub class_color: String,
    pub property_color: String,
    pub instance_color: String,
    pub literal_color: String,
}

/// Syntax-highlighted SPARQL query display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SPARQLQueryDisplayConfig {
    pub show_prefixes: bool,
    pub syntax_highlighting: bool,
    pub show_parameter_values: bool,
    pub show_execution_plan: bool,
    pub copyable: bool,
}

/// Subject-Predicate-Object table showing RDF triples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripleOutputConfig {
    pub max_triples: i32,
    pub show_graph: bool,
    pub show_types: bool,
    pub sortable: bool,
    pub searchable: bool,
    pub export_formats: Vec<ExportFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    NTriples,
    Turtle,
    JsonLd,
    Csv,
}

/// Performance metrics showing speed advantage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsConfig {
    pub show_query_time: bool,
    pub show_triple_count: bool,
    pub show_index_used: bool,
    pub show_offline_indicator: bool,
    pub show_comparison_to_api: bool,
    pub comparison_baseline_ms: Option<f64>,
}

/// Plain-English explanation of reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningExplanationConfig {
    pub show_inference_chain: bool,
    pub show_rules_applied: bool,
    pub show_ontology_classes: bool,
    pub language_style: ExplanationStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExplanationStyle {
    Technical,
    Business,
    Simple,
}

/// Business rule evaluated via SPARQL reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule {
    pub name: String,
    pub description: String,
    pub sparql_construct: String,
    pub severity: RuleSeverity,
    pub category: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Validation rule types from ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    Required,
    Pattern { regex: String },
    Range { min: f64, max: f64 },
    Length { min: i32, max: i32 },
    Email,
    Phone,
    Url,
    Custom { validator: String },
}
