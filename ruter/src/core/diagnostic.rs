use super::{ErrorCode, SpanInfo};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

/// Error code wrapper from compiler JSON output.
///
/// The compiler outputs code as: `{"code": "E0433", "explanation": null}`
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CompilerCode {
    pub code: ErrorCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_code: Option<String>,
    pub explanation: Option<String>,
}

impl<'de> Deserialize<'de> for CompilerCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawCompilerCode {
            code: Option<String>,
            explanation: Option<String>,
        }

        let raw = RawCompilerCode::deserialize(deserializer)?;
        let normalized = raw.code.as_deref().unwrap_or_default();
        let parsed = ErrorCode::from_str(normalized).unwrap_or(ErrorCode::Unknown);
        Ok(Self {
            code: parsed,
            raw_code: raw.code,
            explanation: raw.explanation,
        })
    }
}

/// Diagnostic messages produced by the compiler.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    #[serde(rename = "$message_type")]
    pub message_type: Option<String>,
    pub code: Option<CompilerCode>,
    pub message: String,
    #[serde(rename = "spans")]
    pub span: Vec<SpanInfo>,
    #[serde(rename = "level")]
    pub severity: Severity,
    pub children: Vec<Diagnostic>,
    pub rendered: Option<String>,
}

/// Severity levels for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "note")]
    Note,
    #[serde(rename = "help")]
    Help,
    #[serde(rename = "failure-note")]
    FailureNote,
    #[serde(rename = "error: internal compiler error")]
    InternalCompilerError,
}
