//! W3C SHACL (Shapes Constraint Language) Validation (Stub)
//!
//! This crate provides types and structures for W3C SHACL validation.
//! Full implementation to be completed.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

/// SHACL namespace
pub const SH_NS: &str = "http://www.w3.org/ns/shacl#";

/// sh:NodeShape
pub const NODE_SHAPE: &str = "http://www.w3.org/ns/shacl#NodeShape";

/// sh:PropertyShape
pub const PROPERTY_SHAPE: &str = "http://www.w3.org/ns/shacl#PropertyShape";

/// sh:Violation
pub const VIOLATION: &str = "http://www.w3.org/ns/shacl#Violation";

/// sh:Warning
pub const WARNING: &str = "http://www.w3.org/ns/shacl#Warning";

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Info
    Info,
    /// Warning
    Warning,
    /// Violation
    Violation,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Severity
    pub severity: Severity,
    /// Focus node
    pub focus_node: String,
    /// Message
    pub message: String,
}

impl ValidationResult {
    /// Creates a new validation result
    pub fn new(severity: Severity, focus_node: String, message: String) -> Self {
        Self {
            severity,
            focus_node,
            message,
        }
    }
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    conforms: bool,
    results: Vec<ValidationResult>,
}

impl ValidationReport {
    /// Creates a new report
    pub fn new(conforms: bool) -> Self {
        Self {
            conforms,
            results: Vec::new(),
        }
    }

    /// Returns whether data conforms
    pub fn conforms(&self) -> bool {
        self.conforms
    }

    /// Returns results
    pub fn results(&self) -> &[ValidationResult] {
        &self.results
    }

    /// Adds a result
    pub fn add_result(&mut self, result: ValidationResult) {
        if result.severity == Severity::Violation {
            self.conforms = false;
        }
        self.results.push(result);
    }
}

/// Validates minCount constraint
pub fn validate_min_count(count: usize, min: usize) -> bool {
    count >= min
}

/// Validates maxCount constraint
pub fn validate_max_count(count: usize, max: usize) -> bool {
    count <= max
}

/// Validates minLength constraint
pub fn validate_min_length(value: &str, min: usize) -> bool {
    value.len() >= min
}

/// Validates maxLength constraint
pub fn validate_max_length(value: &str, max: usize) -> bool {
    value.len() <= max
}

/// Validates pattern constraint
pub fn validate_pattern(value: &str, pattern: &str) -> bool {
    regex::Regex::new(pattern).ok().map_or(false, |re| re.is_match(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new(true);
        assert!(report.conforms());

        report.add_result(ValidationResult::new(
            Severity::Violation,
            "node1".to_string(),
            "Error".to_string(),
        ));

        assert!(!report.conforms());
        assert_eq!(report.results().len(), 1);
    }

    #[test]
    fn test_constraints() {
        assert!(validate_min_count(5, 3));
        assert!(!validate_min_count(2, 3));
        assert!(validate_max_count(3, 5));
        assert!(!validate_max_count(6, 5));
        assert!(validate_min_length("hello", 3));
        assert!(!validate_min_length("hi", 5));
        assert!(validate_max_length("hi", 5));
        assert!(!validate_max_length("toolong", 5));
    }
}
