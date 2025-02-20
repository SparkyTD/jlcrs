use strum::Display;
use thiserror::Error;

pub mod symbol;
pub mod footprint;
mod json_reader;
mod geometry;
pub mod tests;

#[derive(Debug, Display)]
pub enum ParserType {
    Footprint,
    Symbol,
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid {0} property type: {1}")]
    InvalidPropertyType(ParserType, String),
    #[error("Invalid {0} array length: {1}")]
    InvalidArrayLength(ParserType, String),
    #[error("Format error in {0}: {1}")]
    FormatError(ParserType, String),
}