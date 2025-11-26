//! Standard RDF vocabularies (RDF, RDFS, OWL, XSD, etc.)

/// Standard RDF/RDFS/OWL/XSD vocabularies
pub struct Vocabulary;

impl Vocabulary {
    // RDF namespace
    /// RDF namespace IRI
    pub const RDF_NS: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    /// rdf:type - indicates class membership
    pub const RDF_TYPE: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    /// rdf:Property - class of RDF properties
    pub const RDF_PROPERTY: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property";
    /// rdf:first - first element of RDF list
    pub const RDF_FIRST: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
    /// rdf:rest - rest of RDF list
    pub const RDF_REST: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
    /// rdf:nil - empty RDF list
    pub const RDF_NIL: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";

    // RDFS namespace
    /// RDFS namespace IRI
    pub const RDFS_NS: &'static str = "http://www.w3.org/2000/01/rdf-schema#";
    /// rdfs:Resource - class of all resources
    pub const RDFS_RESOURCE: &'static str = "http://www.w3.org/2000/01/rdf-schema#Resource";
    /// rdfs:Class - class of classes
    pub const RDFS_CLASS: &'static str = "http://www.w3.org/2000/01/rdf-schema#Class";
    /// rdfs:subClassOf - indicates class hierarchy
    pub const RDFS_SUBCLASSOF: &'static str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
    /// rdfs:subPropertyOf - indicates property hierarchy
    pub const RDFS_SUBPROPERTYOF: &'static str = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";
    /// rdfs:domain - domain of a property
    pub const RDFS_DOMAIN: &'static str = "http://www.w3.org/2000/01/rdf-schema#domain";
    /// rdfs:range - range of a property
    pub const RDFS_RANGE: &'static str = "http://www.w3.org/2000/01/rdf-schema#range";
    /// rdfs:label - human-readable label
    pub const RDFS_LABEL: &'static str = "http://www.w3.org/2000/01/rdf-schema#label";
    /// rdfs:comment - human-readable description
    pub const RDFS_COMMENT: &'static str = "http://www.w3.org/2000/01/rdf-schema#comment";

    // OWL namespace
    /// OWL namespace IRI
    pub const OWL_NS: &'static str = "http://www.w3.org/2002/07/owl#";
    /// owl:Class - class of OWL classes
    pub const OWL_CLASS: &'static str = "http://www.w3.org/2002/07/owl#Class";
    /// owl:Thing - class of all individuals
    pub const OWL_THING: &'static str = "http://www.w3.org/2002/07/owl#Thing";
    /// owl:Nothing - empty class
    pub const OWL_NOTHING: &'static str = "http://www.w3.org/2002/07/owl#Nothing";
    /// owl:ObjectProperty - property between individuals
    pub const OWL_OBJECT_PROPERTY: &'static str = "http://www.w3.org/2002/07/owl#ObjectProperty";
    /// owl:DatatypeProperty - property with literal values
    pub const OWL_DATATYPE_PROPERTY: &'static str = "http://www.w3.org/2002/07/owl#DatatypeProperty";
    /// owl:TransitiveProperty - transitive property
    pub const OWL_TRANSITIVE_PROPERTY: &'static str = "http://www.w3.org/2002/07/owl#TransitiveProperty";
    /// owl:SymmetricProperty - symmetric property
    pub const OWL_SYMMETRIC_PROPERTY: &'static str = "http://www.w3.org/2002/07/owl#SymmetricProperty";
    /// owl:FunctionalProperty - property with at most one value
    pub const OWL_FUNCTIONAL_PROPERTY: &'static str = "http://www.w3.org/2002/07/owl#FunctionalProperty";
    /// owl:InverseFunctionalProperty - property whose inverse is functional
    pub const OWL_INVERSE_FUNCTIONAL_PROPERTY: &'static str = "http://www.w3.org/2002/07/owl#InverseFunctionalProperty";
    /// owl:equivalentClass - indicates equivalent classes
    pub const OWL_EQUIVALENT_CLASS: &'static str = "http://www.w3.org/2002/07/owl#equivalentClass";
    /// owl:equivalentProperty - indicates equivalent properties
    pub const OWL_EQUIVALENT_PROPERTY: &'static str = "http://www.w3.org/2002/07/owl#equivalentProperty";
    /// owl:sameAs - indicates identical individuals
    pub const OWL_SAME_AS: &'static str = "http://www.w3.org/2002/07/owl#sameAs";
    /// owl:differentFrom - indicates distinct individuals
    pub const OWL_DIFFERENT_FROM: &'static str = "http://www.w3.org/2002/07/owl#differentFrom";
    /// owl:inverseOf - indicates inverse properties
    pub const OWL_INVERSE_OF: &'static str = "http://www.w3.org/2002/07/owl#inverseOf";

    // XSD namespace
    /// XSD namespace IRI
    pub const XSD_NS: &'static str = "http://www.w3.org/2001/XMLSchema#";
    /// xsd:string - string datatype
    pub const XSD_STRING: &'static str = "http://www.w3.org/2001/XMLSchema#string";
    /// xsd:integer - arbitrary-precision integer
    pub const XSD_INTEGER: &'static str = "http://www.w3.org/2001/XMLSchema#integer";
    /// xsd:int - 32-bit signed integer
    pub const XSD_INT: &'static str = "http://www.w3.org/2001/XMLSchema#int";
    /// xsd:long - 64-bit signed integer
    pub const XSD_LONG: &'static str = "http://www.w3.org/2001/XMLSchema#long";
    /// xsd:double - double-precision floating point
    pub const XSD_DOUBLE: &'static str = "http://www.w3.org/2001/XMLSchema#double";
    /// xsd:float - single-precision floating point
    pub const XSD_FLOAT: &'static str = "http://www.w3.org/2001/XMLSchema#float";
    /// xsd:decimal - arbitrary-precision decimal
    pub const XSD_DECIMAL: &'static str = "http://www.w3.org/2001/XMLSchema#decimal";
    /// xsd:boolean - boolean datatype (true/false)
    pub const XSD_BOOLEAN: &'static str = "http://www.w3.org/2001/XMLSchema#boolean";
    /// xsd:date - date without time zone
    pub const XSD_DATE: &'static str = "http://www.w3.org/2001/XMLSchema#date";
    /// xsd:dateTime - date and time with optional timezone
    pub const XSD_DATETIME: &'static str = "http://www.w3.org/2001/XMLSchema#dateTime";
    /// xsd:time - time of day with optional timezone
    pub const XSD_TIME: &'static str = "http://www.w3.org/2001/XMLSchema#time";

    // SHACL namespace
    /// SHACL namespace IRI
    pub const SHACL_NS: &'static str = "http://www.w3.org/ns/shacl#";
    /// sh:Shape - base class for SHACL shapes
    pub const SHACL_SHAPE: &'static str = "http://www.w3.org/ns/shacl#Shape";
    /// sh:NodeShape - shape for validating nodes
    pub const SHACL_NODE_SHAPE: &'static str = "http://www.w3.org/ns/shacl#NodeShape";
    /// sh:PropertyShape - shape for validating property values
    pub const SHACL_PROPERTY_SHAPE: &'static str = "http://www.w3.org/ns/shacl#PropertyShape";

    // PROV namespace
    /// PROV namespace IRI
    pub const PROV_NS: &'static str = "http://www.w3.org/ns/prov#";
    /// prov:Entity - physical, digital, or conceptual thing
    pub const PROV_ENTITY: &'static str = "http://www.w3.org/ns/prov#Entity";
    /// prov:Activity - something that occurs over time
    pub const PROV_ACTIVITY: &'static str = "http://www.w3.org/ns/prov#Activity";
    /// prov:Agent - something that bears responsibility
    pub const PROV_AGENT: &'static str = "http://www.w3.org/ns/prov#Agent";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocabulary_constants() {
        assert!(Vocabulary::RDF_TYPE.starts_with(Vocabulary::RDF_NS));
        assert!(Vocabulary::RDFS_CLASS.starts_with(Vocabulary::RDFS_NS));
        assert!(Vocabulary::OWL_CLASS.starts_with(Vocabulary::OWL_NS));
        assert!(Vocabulary::XSD_STRING.starts_with(Vocabulary::XSD_NS));
    }
}
