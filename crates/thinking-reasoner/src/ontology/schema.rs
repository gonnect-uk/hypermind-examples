//! Generic ontology schema loader
//!
//! Loads and parses ontology files (Turtle, RDF/XML) to extract
//! class hierarchies, property definitions, and rule patterns.

use std::collections::{HashMap, HashSet};
use crate::{Result, ThinkingReasonerError};
use super::vocabulary::*;

/// Parsed ontology schema
#[derive(Debug, Clone)]
pub struct OntologySchema {
    /// All classes defined in the ontology
    pub classes: HashSet<String>,

    /// Class hierarchy: class -> set of superclasses
    pub class_hierarchy: HashMap<String, HashSet<String>>,

    /// All properties defined in the ontology
    pub properties: HashSet<String>,

    /// Property hierarchy: property -> set of superproperties
    pub property_hierarchy: HashMap<String, HashSet<String>>,

    /// Domain constraints: property -> set of domain classes
    pub domains: HashMap<String, HashSet<String>>,

    /// Range constraints: property -> set of range classes
    pub ranges: HashMap<String, HashSet<String>>,

    /// Transitive properties
    pub transitive_properties: HashSet<String>,

    /// Symmetric properties
    pub symmetric_properties: HashSet<String>,

    /// Functional properties
    pub functional_properties: HashSet<String>,

    /// Inverse property pairs
    pub inverse_properties: HashMap<String, String>,

    /// Custom rule definitions from ontology annotations
    pub custom_rules: Vec<CustomRule>,
}

/// Custom rule defined in ontology via annotations
#[derive(Debug, Clone)]
pub struct CustomRule {
    /// Rule identifier URI
    pub id: String,
    /// Rule name
    pub name: String,
    /// Head pattern (conclusion)
    pub head: String,
    /// Body patterns (premises)
    pub body: String,
    /// Priority (higher = earlier)
    pub priority: i32,
}

impl OntologySchema {
    /// Create an empty schema
    pub fn new() -> Self {
        Self {
            classes: HashSet::new(),
            class_hierarchy: HashMap::new(),
            properties: HashSet::new(),
            property_hierarchy: HashMap::new(),
            domains: HashMap::new(),
            ranges: HashMap::new(),
            transitive_properties: HashSet::new(),
            symmetric_properties: HashSet::new(),
            functional_properties: HashSet::new(),
            inverse_properties: HashMap::new(),
            custom_rules: Vec::new(),
        }
    }

    /// Load schema from Turtle content
    ///
    /// This parses the ontology and extracts:
    /// - Class definitions and hierarchy (rdfs:subClassOf)
    /// - Property definitions and hierarchy (rdfs:subPropertyOf)
    /// - Domain and range constraints
    /// - OWL property characteristics (transitive, symmetric, functional)
    /// - Custom rule annotations
    pub fn from_turtle(ttl_content: &str) -> Result<Self> {
        let mut schema = Self::new();

        // First pass: collect prefix declarations
        let mut prefixes: HashMap<String, String> = HashMap::new();
        for line in ttl_content.lines() {
            let line = line.trim();
            if line.starts_with("@prefix") || line.starts_with("PREFIX") {
                if let Some((prefix, uri)) = parse_prefix_declaration(line) {
                    prefixes.insert(prefix, uri);
                }
            }
        }

        // Parse triples from the Turtle content
        // This is a simplified parser - in production, use rdf-io crate
        for line in ttl_content.lines() {
            let line = line.trim();

            // Skip comments and prefixes
            if line.is_empty() || line.starts_with('#') || line.starts_with('@') {
                continue;
            }

            // Parse class definitions (both prefixed and full URI)
            if is_class_definition(line) {
                if let Some(class_uri) = extract_subject_with_prefixes(line, &prefixes) {
                    schema.classes.insert(class_uri);
                }
            }

            // Parse subClassOf (both prefixed and full URI)
            if contains_predicate(line, "subClassOf") {
                if let (Some(subclass), Some(superclass)) = extract_subject_object_with_prefixes(line, &prefixes) {
                    schema.class_hierarchy
                        .entry(subclass)
                        .or_default()
                        .insert(superclass);
                }
            }

            // Parse property definitions (both prefixed and full URI)
            if is_property_definition(line) {
                if let Some(prop_uri) = extract_subject_with_prefixes(line, &prefixes) {
                    schema.properties.insert(prop_uri);
                }
            }

            // Parse subPropertyOf (both prefixed and full URI)
            if contains_predicate(line, "subPropertyOf") {
                if let (Some(subprop), Some(superprop)) = extract_subject_object_with_prefixes(line, &prefixes) {
                    schema.property_hierarchy
                        .entry(subprop)
                        .or_default()
                        .insert(superprop);
                }
            }

            // Parse domain (both prefixed and full URI)
            if contains_predicate(line, "domain") {
                if let (Some(prop), Some(domain)) = extract_subject_object_with_prefixes(line, &prefixes) {
                    schema.domains.entry(prop).or_default().insert(domain);
                }
            }

            // Parse range (both prefixed and full URI)
            if contains_predicate(line, "range") {
                if let (Some(prop), Some(range)) = extract_subject_object_with_prefixes(line, &prefixes) {
                    schema.ranges.entry(prop).or_default().insert(range);
                }
            }

            // Parse owl:TransitiveProperty (both prefixed and full URI)
            if is_transitive_property(line) {
                if let Some(prop_uri) = extract_subject_with_prefixes(line, &prefixes) {
                    schema.transitive_properties.insert(prop_uri);
                }
            }

            // Parse owl:SymmetricProperty (both prefixed and full URI)
            if is_symmetric_property(line) {
                if let Some(prop_uri) = extract_subject_with_prefixes(line, &prefixes) {
                    schema.symmetric_properties.insert(prop_uri);
                }
            }

            // Parse owl:FunctionalProperty (both prefixed and full URI)
            if is_functional_property(line) {
                if let Some(prop_uri) = extract_subject_with_prefixes(line, &prefixes) {
                    schema.functional_properties.insert(prop_uri);
                }
            }

            // Parse owl:inverseOf (both prefixed and full URI)
            if contains_predicate(line, "inverseOf") {
                if let (Some(prop1), Some(prop2)) = extract_subject_object_with_prefixes(line, &prefixes) {
                    schema.inverse_properties.insert(prop1.clone(), prop2.clone());
                    schema.inverse_properties.insert(prop2, prop1);
                }
            }

            // Parse custom rule annotations (th:DeductionRule)
            if line.contains("a th:DeductionRule") || line.contains("DeductionRule") {
                if let Some(rule_uri) = extract_subject_with_prefixes(line, &prefixes) {
                    // We'll parse the full rule in a second pass
                    schema.custom_rules.push(CustomRule {
                        id: rule_uri,
                        name: String::new(),
                        head: String::new(),
                        body: String::new(),
                        priority: 0,
                    });
                }
            }
        }

        Ok(schema)
    }

    /// Merge another schema into this one
    pub fn merge(&mut self, other: &OntologySchema) {
        self.classes.extend(other.classes.iter().cloned());

        for (class, supers) in &other.class_hierarchy {
            self.class_hierarchy
                .entry(class.clone())
                .or_default()
                .extend(supers.iter().cloned());
        }

        self.properties.extend(other.properties.iter().cloned());

        for (prop, supers) in &other.property_hierarchy {
            self.property_hierarchy
                .entry(prop.clone())
                .or_default()
                .extend(supers.iter().cloned());
        }

        for (prop, domains) in &other.domains {
            self.domains
                .entry(prop.clone())
                .or_default()
                .extend(domains.iter().cloned());
        }

        for (prop, ranges) in &other.ranges {
            self.ranges
                .entry(prop.clone())
                .or_default()
                .extend(ranges.iter().cloned());
        }

        self.transitive_properties.extend(other.transitive_properties.iter().cloned());
        self.symmetric_properties.extend(other.symmetric_properties.iter().cloned());
        self.functional_properties.extend(other.functional_properties.iter().cloned());
        self.inverse_properties.extend(other.inverse_properties.iter().map(|(k, v)| (k.clone(), v.clone())));
        self.custom_rules.extend(other.custom_rules.iter().cloned());
    }

    /// Get all superclasses of a class (transitive closure)
    pub fn get_all_superclasses(&self, class: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut to_visit = vec![class.to_string()];

        while let Some(current) = to_visit.pop() {
            if let Some(supers) = self.class_hierarchy.get(&current) {
                for sup in supers {
                    if result.insert(sup.clone()) {
                        to_visit.push(sup.clone());
                    }
                }
            }
        }

        result
    }

    /// Get all superproperties of a property (transitive closure)
    pub fn get_all_superproperties(&self, property: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut to_visit = vec![property.to_string()];

        while let Some(current) = to_visit.pop() {
            if let Some(supers) = self.property_hierarchy.get(&current) {
                for sup in supers {
                    if result.insert(sup.clone()) {
                        to_visit.push(sup.clone());
                    }
                }
            }
        }

        result
    }
}

impl Default for OntologySchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract subject URI from a triple line
fn extract_subject(line: &str) -> Option<String> {
    // Handle <uri> format
    if let Some(start) = line.find('<') {
        if let Some(end) = line.find('>') {
            return Some(line[start + 1..end].to_string());
        }
    }

    // Handle prefixed name format (e.g., th:Event)
    let parts: Vec<&str> = line.split_whitespace().collect();
    if !parts.is_empty() {
        let subject = parts[0];
        if subject.contains(':') && !subject.starts_with('<') {
            return Some(subject.to_string());
        }
    }

    None
}

/// Extract object URI after a predicate
fn extract_object_after(line: &str, predicate: &str) -> Option<String> {
    if let Some(pos) = line.find(predicate) {
        let after = &line[pos + predicate.len()..];

        // Handle <uri> format
        if let Some(start) = after.find('<') {
            if let Some(end) = after.find('>') {
                return Some(after[start + 1..end].to_string());
            }
        }

        // Handle prefixed name format
        let parts: Vec<&str> = after.split_whitespace().collect();
        if !parts.is_empty() {
            let obj = parts[0].trim_end_matches(|c| c == '.' || c == ';' || c == ',');
            if obj.contains(':') && !obj.starts_with('<') {
                return Some(obj.to_string());
            }
        }
    }

    None
}

/// Check if line contains a class definition (handles both prefixed and full URI)
fn is_class_definition(line: &str) -> bool {
    // Prefixed form: a owl:Class
    if line.contains("a owl:Class") || line.contains("rdf:type owl:Class") {
        return true;
    }
    // Full URI form: a <http://www.w3.org/2002/07/owl#Class>
    if (line.contains(" a ") || line.contains(" rdf:type "))
        && line.contains("http://www.w3.org/2002/07/owl#Class")
    {
        return true;
    }
    false
}

/// Check if line contains a property definition
fn is_property_definition(line: &str) -> bool {
    // Prefixed forms
    if line.contains("a owl:ObjectProperty")
        || line.contains("a owl:DatatypeProperty")
        || line.contains("a rdf:Property")
    {
        return true;
    }
    // Full URI forms
    if line.contains(" a ") || line.contains(" rdf:type ") {
        if line.contains("http://www.w3.org/2002/07/owl#ObjectProperty")
            || line.contains("http://www.w3.org/2002/07/owl#DatatypeProperty")
            || line.contains("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property")
        {
            return true;
        }
    }
    false
}

/// Check if line contains owl:TransitiveProperty
fn is_transitive_property(line: &str) -> bool {
    // Prefixed form
    if line.contains("a owl:TransitiveProperty") {
        return true;
    }
    // Full URI form
    if (line.contains(" a ") || line.contains(" rdf:type "))
        && line.contains("http://www.w3.org/2002/07/owl#TransitiveProperty")
    {
        return true;
    }
    false
}

/// Check if line contains owl:SymmetricProperty
fn is_symmetric_property(line: &str) -> bool {
    // Prefixed form
    if line.contains("a owl:SymmetricProperty") {
        return true;
    }
    // Full URI form
    if (line.contains(" a ") || line.contains(" rdf:type "))
        && line.contains("http://www.w3.org/2002/07/owl#SymmetricProperty")
    {
        return true;
    }
    false
}

/// Check if line contains owl:FunctionalProperty
fn is_functional_property(line: &str) -> bool {
    // Prefixed form
    if line.contains("a owl:FunctionalProperty") {
        return true;
    }
    // Full URI form
    if (line.contains(" a ") || line.contains(" rdf:type "))
        && line.contains("http://www.w3.org/2002/07/owl#FunctionalProperty")
    {
        return true;
    }
    false
}

/// Check if line contains a specific predicate (handles both prefixed and full URI)
fn contains_predicate(line: &str, predicate_local: &str) -> bool {
    // Prefixed form: rdfs:subClassOf
    if line.contains(&format!("rdfs:{}", predicate_local))
        || line.contains(&format!("owl:{}", predicate_local))
    {
        return true;
    }
    // Full URI forms
    if line.contains(&format!("http://www.w3.org/2000/01/rdf-schema#{}", predicate_local))
        || line.contains(&format!("http://www.w3.org/2002/07/owl#{}", predicate_local))
    {
        return true;
    }
    false
}

/// Extract subject and object from a triple line
fn extract_subject_object(line: &str) -> (Option<String>, Option<String>) {
    let subject = extract_subject(line);

    // Extract object (last URI or prefixed name in the line)
    let object = extract_last_uri_or_prefixed(line);

    (subject, object)
}

/// Extract the last URI or prefixed name from a line (typically the object)
fn extract_last_uri_or_prefixed(line: &str) -> Option<String> {
    // Find all URIs in angle brackets
    let mut last_uri = None;
    let mut search_from = 0;
    while let Some(start) = line[search_from..].find('<') {
        let absolute_start = search_from + start;
        if let Some(end) = line[absolute_start..].find('>') {
            let uri = &line[absolute_start + 1..absolute_start + end];
            // Skip the subject (first URI) - only keep later ones
            last_uri = Some(uri.to_string());
            search_from = absolute_start + end + 1;
        } else {
            break;
        }
    }

    if last_uri.is_some() {
        return last_uri;
    }

    // Try prefixed names
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        let obj = parts.last()?.trim_end_matches(|c| c == '.' || c == ';' || c == ',');
        if obj.contains(':') && !obj.starts_with('<') {
            return Some(obj.to_string());
        }
    }

    None
}

/// Parse a prefix declaration line like "@prefix ins: <http://insurance.gonnect.ai/> ."
/// Returns (prefix_name, base_uri) e.g. ("ins:", "http://insurance.gonnect.ai/")
fn parse_prefix_declaration(line: &str) -> Option<(String, String)> {
    let line = line.trim();

    // Handle "@prefix" or "PREFIX" forms
    let without_directive = if line.starts_with("@prefix") {
        line.strip_prefix("@prefix")?.trim()
    } else if line.to_uppercase().starts_with("PREFIX") {
        let pos = line.chars().position(|c| c.is_whitespace())?;
        line[pos..].trim()
    } else {
        return None;
    };

    // Split at first space/colon to get prefix name
    let parts: Vec<&str> = without_directive.splitn(2, char::is_whitespace).collect();
    if parts.len() < 2 {
        return None;
    }

    let prefix = parts[0].trim_end_matches(':');
    let rest = parts[1].trim();

    // Extract URI from angle brackets
    if let Some(start) = rest.find('<') {
        if let Some(end) = rest.find('>') {
            let uri = rest[start + 1..end].to_string();
            // Return with colon suffix for matching (e.g., "ins:")
            return Some((format!("{}:", prefix), uri));
        }
    }

    None
}

/// Expand a prefixed name to full URI using the prefix map
/// e.g., "ins:transfers" with prefix map {"ins:": "http://insurance.gonnect.ai/"}
/// becomes "http://insurance.gonnect.ai/transfers"
fn expand_prefixed_name(name: &str, prefixes: &HashMap<String, String>) -> String {
    // If already a full URI (contains :// but no prefix match), return as-is
    if name.contains("://") {
        return name.to_string();
    }

    // Try to find a matching prefix
    if let Some(colon_pos) = name.find(':') {
        let prefix_with_colon = &name[..=colon_pos];
        let local_name = &name[colon_pos + 1..];

        if let Some(base_uri) = prefixes.get(prefix_with_colon) {
            return format!("{}{}", base_uri, local_name);
        }
    }

    // No prefix found, return as-is
    name.to_string()
}

/// Extract subject URI from a triple line, expanding prefixes to full URIs
fn extract_subject_with_prefixes(line: &str, prefixes: &HashMap<String, String>) -> Option<String> {
    // First try the original extraction
    let raw = extract_subject(line)?;

    // Then expand any prefix
    Some(expand_prefixed_name(&raw, prefixes))
}

/// Extract subject and object from a triple line, expanding prefixes to full URIs
fn extract_subject_object_with_prefixes(line: &str, prefixes: &HashMap<String, String>) -> (Option<String>, Option<String>) {
    let (subject, object) = extract_subject_object(line);

    let expanded_subject = subject.map(|s| expand_prefixed_name(&s, prefixes));
    let expanded_object = object.map(|o| expand_prefixed_name(&o, prefixes));

    (expanded_subject, expanded_object)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_schema() {
        let schema = OntologySchema::new();
        assert!(schema.classes.is_empty());
        assert!(schema.properties.is_empty());
    }

    #[test]
    fn test_parse_class_prefixed() {
        let ttl = r#"
            <http://example.org/Person> a owl:Class .
        "#;

        let schema = OntologySchema::from_turtle(ttl).unwrap();
        assert!(schema.classes.contains("http://example.org/Person"));
    }

    #[test]
    fn test_parse_class_full_uri() {
        let ttl = r#"
            <http://example.org/Person> a <http://www.w3.org/2002/07/owl#Class> .
        "#;

        let schema = OntologySchema::from_turtle(ttl).unwrap();
        assert!(schema.classes.contains("http://example.org/Person"));
    }

    #[test]
    fn test_parse_transitive_property_prefixed() {
        let ttl = r#"
            <http://example.org/knows> a owl:TransitiveProperty .
        "#;

        let schema = OntologySchema::from_turtle(ttl).unwrap();
        assert!(schema.transitive_properties.contains("http://example.org/knows"));
    }

    #[test]
    fn test_parse_transitive_property_full_uri() {
        let ttl = r#"
            <http://example.org/knows> a <http://www.w3.org/2002/07/owl#TransitiveProperty> .
        "#;

        let schema = OntologySchema::from_turtle(ttl).unwrap();
        assert!(
            schema.transitive_properties.contains("http://example.org/knows"),
            "Failed to parse transitive property with full URI format. Found: {:?}",
            schema.transitive_properties
        );
    }

    #[test]
    fn test_parse_subclass_full_uri() {
        let ttl = r#"
            <http://example.org/Student> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://example.org/Person> .
        "#;

        let schema = OntologySchema::from_turtle(ttl).unwrap();
        assert!(
            schema.class_hierarchy.contains_key("http://example.org/Student"),
            "Student not found in class_hierarchy. Found: {:?}",
            schema.class_hierarchy
        );
        let supers = schema.class_hierarchy.get("http://example.org/Student").unwrap();
        assert!(
            supers.contains("http://example.org/Person"),
            "Person not in superclasses of Student. Found: {:?}",
            supers
        );
    }

    #[test]
    fn test_parse_domain_range_full_uri() {
        let ttl = r#"
            <http://example.org/hasParent> <http://www.w3.org/2000/01/rdf-schema#domain> <http://example.org/Person> .
            <http://example.org/hasParent> <http://www.w3.org/2000/01/rdf-schema#range> <http://example.org/Person> .
        "#;

        let schema = OntologySchema::from_turtle(ttl).unwrap();

        assert!(
            schema.domains.contains_key("http://example.org/hasParent"),
            "hasParent not in domains. Found: {:?}",
            schema.domains
        );
        assert!(
            schema.ranges.contains_key("http://example.org/hasParent"),
            "hasParent not in ranges. Found: {:?}",
            schema.ranges
        );
    }

    #[test]
    fn test_helper_is_transitive_property() {
        let line_prefixed = "<http://example.org/knows> a owl:TransitiveProperty .";
        let line_full = "<http://example.org/knows> a <http://www.w3.org/2002/07/owl#TransitiveProperty> .";

        assert!(is_transitive_property(line_prefixed), "Prefixed form not detected");
        assert!(is_transitive_property(line_full), "Full URI form not detected");
    }

    #[test]
    fn test_helper_contains_predicate() {
        let line = "<http://example.org/Student> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://example.org/Person> .";

        assert!(
            contains_predicate(line, "subClassOf"),
            "subClassOf not detected in line: {}",
            line
        );
    }

    #[test]
    fn test_helper_extract_subject_object() {
        let line = "<http://example.org/Student> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://example.org/Person> .";

        let (subject, object) = extract_subject_object(line);

        assert_eq!(
            subject,
            Some("http://example.org/Student".to_string()),
            "Subject extraction failed"
        );
        assert_eq!(
            object,
            Some("http://example.org/Person".to_string()),
            "Object extraction failed"
        );
    }

    #[test]
    fn test_superclass_transitive_closure() {
        let mut schema = OntologySchema::new();

        // Setup: Student -> Person -> LivingThing
        let mut student_supers = HashSet::new();
        student_supers.insert("http://example.org/Person".to_string());
        schema.class_hierarchy.insert("http://example.org/Student".to_string(), student_supers);

        let mut person_supers = HashSet::new();
        person_supers.insert("http://example.org/LivingThing".to_string());
        schema.class_hierarchy.insert("http://example.org/Person".to_string(), person_supers);

        let all_supers = schema.get_all_superclasses("http://example.org/Student");

        assert!(all_supers.contains("http://example.org/Person"));
        assert!(all_supers.contains("http://example.org/LivingThing"));
    }

    #[test]
    fn test_prefix_declaration_parsing() {
        let line = "@prefix ins: <http://insurance.gonnect.ai/> .";
        let result = parse_prefix_declaration(line);
        assert!(result.is_some());
        let (prefix, uri) = result.unwrap();
        assert_eq!(prefix, "ins:");
        assert_eq!(uri, "http://insurance.gonnect.ai/");
    }

    #[test]
    fn test_prefix_expansion() {
        let mut prefixes = HashMap::new();
        prefixes.insert("ins:".to_string(), "http://insurance.gonnect.ai/".to_string());
        prefixes.insert("owl:".to_string(), "http://www.w3.org/2002/07/owl#".to_string());

        // Test prefixed name expansion
        assert_eq!(
            expand_prefixed_name("ins:transfers", &prefixes),
            "http://insurance.gonnect.ai/transfers"
        );

        // Test full URI passthrough
        assert_eq!(
            expand_prefixed_name("http://example.org/test", &prefixes),
            "http://example.org/test"
        );

        // Test unknown prefix passthrough
        assert_eq!(
            expand_prefixed_name("unknown:test", &prefixes),
            "unknown:test"
        );
    }

    #[test]
    fn test_parse_symmetric_property_with_prefix() {
        let ttl = r#"
            @prefix ins: <http://insurance.gonnect.ai/> .
            @prefix owl: <http://www.w3.org/2002/07/owl#> .

            ins:relatedTo a owl:SymmetricProperty .
        "#;

        let schema = OntologySchema::from_turtle(ttl).unwrap();

        // Should contain expanded URI, not prefixed form
        assert!(
            schema.symmetric_properties.contains("http://insurance.gonnect.ai/relatedTo"),
            "Expected 'http://insurance.gonnect.ai/relatedTo' in symmetric_properties. Found: {:?}",
            schema.symmetric_properties
        );
    }

    #[test]
    fn test_parse_transitive_property_with_prefix() {
        let ttl = r#"
            @prefix ins: <http://insurance.gonnect.ai/> .
            @prefix owl: <http://www.w3.org/2002/07/owl#> .

            ins:transfers a owl:TransitiveProperty .
        "#;

        let schema = OntologySchema::from_turtle(ttl).unwrap();

        // Should contain expanded URI, not prefixed form
        assert!(
            schema.transitive_properties.contains("http://insurance.gonnect.ai/transfers"),
            "Expected 'http://insurance.gonnect.ai/transfers' in transitive_properties. Found: {:?}",
            schema.transitive_properties
        );
    }
}
