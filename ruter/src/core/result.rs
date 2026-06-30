use thiserror::Error;

/// Define a custom error type for result handling
#[derive(Debug, Error)]
pub enum RuTeRError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Regex Error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Parsing Error: {0}")]
    ParseError(String),

    #[error("No available patches for Error code {0}")]
    NoFixAvailable(String),

    #[error("Source file not found: {0}")]
    SourceFileNotFound(String),

    #[error("Unsupported fix action: {0}")]
    UnsupportedFixAction(String),

    #[error("Conflicting fix actions: {0}")]
    ConflictingFixActions(String),

    #[error("Invalid byte range: {0}")]
    InvalidByteRange(String),
}

/// Define a type alias for Result using the custom error type
pub type Result<T> = std::result::Result<T, RuTeRError>;
