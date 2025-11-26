//! SHACL Validation
//!
//! Validates ontology against SHACL shapes from universal-meta-mobile-ontology.

use crate::error::{Result, GeneratorError};
use crate::parser::ontology::Ontology;

/// SHACL validator
pub struct ShaclValidator;

impl ShaclValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate ontology against SHACL constraints
    pub fn validate(&self, ontology: &Ontology) -> Result<()> {
        // Validate MobileApplication
        if let Some(app) = &ontology.mobile_application {
            self.validate_mobile_application(app)?;
        } else {
            return Err(GeneratorError::Validation(
                "MobileApplication entity is required".to_string()
            ));
        }

        // Validate Personas
        for persona in &ontology.personas {
            self.validate_persona(persona)?;
        }

        // Validate BusinessValues
        for bv in &ontology.business_values {
            self.validate_business_value(bv)?;
        }

        // Validate Views
        for view in &ontology.views {
            self.validate_view(view)?;
        }

        // Validate Fields
        for field in &ontology.fields {
            self.validate_field(field)?;
        }

        // Validate Queries
        for query in &ontology.queries {
            self.validate_query(query)?;
        }

        Ok(())
    }

    /// Validate MobileApplication shape
    fn validate_mobile_application(&self, app: &crate::parser::ontology::OntologyEntity) -> Result<()> {
        // Required: meta:hasTitle (minCount 1, maxCount 1, minLength 3, maxLength 50)
        let title = app.get_property("hasTitle")
            .or_else(|| app.get_property("http://universal-kg.com/meta/hasTitle"))
            .ok_or_else(|| GeneratorError::Validation("MobileApplication must have hasTitle".to_string()))?;

        if title.len() < 3 || title.len() > 50 {
            return Err(GeneratorError::Validation(
                format!("Title length must be 3-50 characters, got {}", title.len())
            ));
        }

        // Required: meta:hasPersona (minCount 1, maxCount 1)
        if app.get_property("hasPersona").is_none() &&
           app.get_property("http://universal-kg.com/meta/hasPersona").is_none() {
            return Err(GeneratorError::Validation("MobileApplication must have hasPersona".to_string()));
        }

        // Required: meta:hasBusinessValue (minCount 1, maxCount 1)
        if app.get_property("hasBusinessValue").is_none() &&
           app.get_property("http://universal-kg.com/meta/hasBusinessValue").is_none() {
            return Err(GeneratorError::Validation("MobileApplication must have hasBusinessValue".to_string()));
        }

        // Required: meta:hasIcon (minCount 1, maxCount 1, pattern)
        let icon = app.get_property("hasIcon")
            .or_else(|| app.get_property("http://universal-kg.com/meta/hasIcon"))
            .ok_or_else(|| GeneratorError::Validation("MobileApplication must have hasIcon".to_string()))?;

        if !self.is_valid_icon_name(icon) {
            return Err(GeneratorError::Validation(
                format!("Icon name must match pattern ^[a-zA-Z0-9._-]+$, got '{}'", icon)
            ));
        }

        // Required: meta:hasPrimaryColor (minCount 1, maxCount 1, pattern)
        let color = app.get_property("hasPrimaryColor")
            .or_else(|| app.get_property("http://universal-kg.com/meta/hasPrimaryColor"))
            .ok_or_else(|| GeneratorError::Validation("MobileApplication must have hasPrimaryColor".to_string()))?;

        if !self.is_valid_hex_color(color) {
            return Err(GeneratorError::Validation(
                format!("Color must be hex format #RRGGBB, got '{}'", color)
            ));
        }

        // Required: meta:hasHomeView (minCount 1, maxCount 1)
        if app.get_property("hasHomeView").is_none() &&
           app.get_property("http://universal-kg.com/meta/hasHomeView").is_none() {
            return Err(GeneratorError::Validation("MobileApplication must have hasHomeView".to_string()));
        }

        // Required: meta:offlineCapable (minCount 1, maxCount 1, value must be true)
        let offline = app.get_property("offlineCapable")
            .or_else(|| app.get_property("http://universal-kg.com/meta/offlineCapable"))
            .ok_or_else(|| GeneratorError::Validation("MobileApplication must have offlineCapable".to_string()))?;

        if offline != "true" && offline != "\"true\"^^http://www.w3.org/2001/XMLSchema#boolean" {
            return Err(GeneratorError::Validation(
                "MobileApplication must be offline-capable (offlineCapable=true)".to_string()
            ));
        }

        Ok(())
    }

    /// Validate BusinessPersona shape
    fn validate_persona(&self, persona: &crate::parser::ontology::OntologyEntity) -> Result<()> {
        // Required: meta:personaName (minCount 1, maxCount 1, minLength 1, maxLength 50)
        let name = persona.get_property("personaName")
            .or_else(|| persona.get_property("http://universal-kg.com/meta/personaName"))
            .ok_or_else(|| GeneratorError::Validation("BusinessPersona must have personaName".to_string()))?;

        if name.is_empty() || name.len() > 50 {
            return Err(GeneratorError::Validation(
                format!("Persona name length must be 1-50 characters, got {}", name.len())
            ));
        }

        // Required: meta:personaDescription (minCount 1, maxCount 1, minLength 10, maxLength 500)
        let desc = persona.get_property("personaDescription")
            .or_else(|| persona.get_property("http://universal-kg.com/meta/personaDescription"))
            .ok_or_else(|| GeneratorError::Validation("BusinessPersona must have personaDescription".to_string()))?;

        if desc.len() < 10 || desc.len() > 500 {
            return Err(GeneratorError::Validation(
                format!("Persona description length must be 10-500 characters, got {}", desc.len())
            ));
        }

        Ok(())
    }

    /// Validate BusinessValue shape
    fn validate_business_value(&self, bv: &crate::parser::ontology::OntologyEntity) -> Result<()> {
        // Required: meta:businessProblem (minCount 1, maxCount 1, minLength 20)
        let problem = bv.get_property("businessProblem")
            .or_else(|| bv.get_property("http://universal-kg.com/meta/businessProblem"))
            .ok_or_else(|| GeneratorError::Validation("BusinessValue must have businessProblem".to_string()))?;

        if problem.len() < 20 {
            return Err(GeneratorError::Validation(
                format!("Business problem must be at least 20 characters, got {}", problem.len())
            ));
        }

        // Required: meta:businessSolution (minCount 1, maxCount 1, minLength 20)
        let solution = bv.get_property("businessSolution")
            .or_else(|| bv.get_property("http://universal-kg.com/meta/businessSolution"))
            .ok_or_else(|| GeneratorError::Validation("BusinessValue must have businessSolution".to_string()))?;

        if solution.len() < 20 {
            return Err(GeneratorError::Validation(
                format!("Business solution must be at least 20 characters, got {}", solution.len())
            ));
        }

        // Required: meta:businessMetric (minCount 1, maxCount 1, minLength 5)
        let metric = bv.get_property("businessMetric")
            .or_else(|| bv.get_property("http://universal-kg.com/meta/businessMetric"))
            .ok_or_else(|| GeneratorError::Validation("BusinessValue must have businessMetric".to_string()))?;

        if metric.len() < 5 {
            return Err(GeneratorError::Validation(
                format!("Business metric must be at least 5 characters, got {}", metric.len())
            ));
        }

        Ok(())
    }

    /// Validate View shape
    fn validate_view(&self, _view: &crate::parser::ontology::OntologyEntity) -> Result<()> {
        // View validation (label, hasField, executesQuery)
        // Simplified for now - full SHACL validation would be more complex
        Ok(())
    }

    /// Validate Field shape
    fn validate_field(&self, field: &crate::parser::ontology::OntologyEntity) -> Result<()> {
        // Required: meta:fieldLabel (minCount 1, maxCount 1, minLength 1, maxLength 50)
        let label = field.get_property("fieldLabel")
            .or_else(|| field.get_property("http://universal-kg.com/meta/fieldLabel"));

        if let Some(label) = label {
            if label.is_empty() || label.len() > 50 {
                return Err(GeneratorError::Validation(
                    format!("Field label length must be 1-50 characters, got {}", label.len())
                ));
            }
        }

        // Required: meta:fieldOrder (minCount 1, maxCount 1, datatype xsd:integer, minInclusive 0)
        // Required: meta:isRequired (minCount 1, maxCount 1, datatype xsd:boolean)
        // Required: meta:dataType (minCount 1, maxCount 1)
        // Required: meta:bindsToProperty (minCount 1, maxCount 1)

        Ok(())
    }

    /// Validate QueryTemplate shape
    fn validate_query(&self, query: &crate::parser::ontology::OntologyEntity) -> Result<()> {
        // Required: meta:queryType (minCount 1, maxCount 1)
        // Required: meta:queryTemplate (minCount 1, maxCount 1, minLength 10, maxLength 10000)
        let template = query.get_property("queryTemplate")
            .or_else(|| query.get_property("http://universal-kg.com/meta/queryTemplate"));

        if let Some(template) = template {
            if template.len() < 10 || template.len() > 10000 {
                return Err(GeneratorError::Validation(
                    format!("Query template length must be 10-10000 characters, got {}", template.len())
                ));
            }
        }

        Ok(())
    }

    /// Validate icon name pattern: ^[a-zA-Z0-9._-]+$
    fn is_valid_icon_name(&self, icon: &str) -> bool {
        !icon.is_empty() && icon.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
    }

    /// Validate hex color pattern: ^#[0-9A-Fa-f]{6}$
    fn is_valid_hex_color(&self, color: &str) -> bool {
        if !color.starts_with('#') || color.len() != 7 {
            return false;
        }
        color[1..].chars().all(|c| c.is_ascii_hexdigit())
    }
}

impl Default for ShaclValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_icon_name() {
        let validator = ShaclValidator::new();
        assert!(validator.is_valid_icon_name("star.fill"));
        assert!(validator.is_valid_icon_name("person_circle"));
        assert!(validator.is_valid_icon_name("chart-bar"));
        assert!(!validator.is_valid_icon_name("star fill")); // Space not allowed
        assert!(!validator.is_valid_icon_name("star@fill")); // @ not allowed
    }

    #[test]
    fn test_valid_hex_color() {
        let validator = ShaclValidator::new();
        assert!(validator.is_valid_hex_color("#007AFF"));
        assert!(validator.is_valid_hex_color("#FFFFFF"));
        assert!(validator.is_valid_hex_color("#000000"));
        assert!(!validator.is_valid_hex_color("007AFF")); // Missing #
        assert!(!validator.is_valid_hex_color("#007AF")); // Too short
        assert!(!validator.is_valid_hex_color("#007AFFF")); // Too long
        assert!(!validator.is_valid_hex_color("#GGGGGG")); // Invalid hex
    }
}
