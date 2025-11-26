use thiserror::Error;

pub type Result<T> = std::result::Result<T, GeneratorError>;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("RDF parsing error: {0}")]
    RdfParse(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid ontology: {0}")]
    InvalidOntology(String),
}
