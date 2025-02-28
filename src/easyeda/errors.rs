use strum::Display;
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum SymbolConverterError {
    #[error("Elements of type {0} are not supported by KiCAD")]
    UnsupportedElement(String),

    #[error("One or more symbol sub-unit names are incorrectly formatted: {0}")]
    IncorrectUnitFormat(String),

    #[error("One or more symbol sub-unit names have an incorrect numeric identifier: {0}")]
    IncorrectUnitNumIdentifier(String),

    #[error("One or more symbol sub-unit names have an incorrect name: {0}")]
    IncorrectUnitName(String),
}

#[derive(Error, Debug)]
pub enum FootprintConverterError {
    #[error("This type of pad shape is not supported: {0}")]
    UnsupportedPadShape(String),

    #[error("Elements are not supported on this layer: {0}")]
    UnsupportedLayer(String),

    #[error("Unsupported pad drill rotation: {0}")]
    UnsupportedDrillRotation(u32),

    #[error("Unsupported inner layer: {0}")]
    UnsupportedInnerLayer(String),
}