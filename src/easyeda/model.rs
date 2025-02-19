use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid property type: {0}")]
    InvalidPropertyType(String),
    #[error("Invalid array length: {0}")]
    InvalidArrayLength(String),
    #[error("Format error")]
    FormatError(String),
}