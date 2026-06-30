use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Location information given by the compiler.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SpanInfo {
    #[serde(rename = "file_name")]
    pub file_path: PathBuf,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line_start: usize,
    pub line_end: usize,
    #[serde(rename = "column_start")]
    pub col_start: usize,
    #[serde(rename = "column_end")]
    pub col_end: usize,
    pub is_primary: bool,
    #[serde(default)]
    pub text: Vec<SpanText>,
    pub label: Option<String>,
    pub suggested_replacement: Option<String>,
    pub suggestion_applicability: Option<Applicability>,
    pub expansion: Option<Box<SpanExpansion>>,
}

/// Source line with highlight information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SpanText {
    pub text: String,
    pub highlight_start: usize,
    pub highlight_end: usize,
}

/// Macro expansion information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SpanExpansion {
    pub span: SpanInfo,
    pub macro_decl_name: String,
    pub def_site_span: Option<Box<SpanInfo>>,
}

/// Applicability of a suggestion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Applicability {
    MachineApplicable,
    MaybeIncorrect,
    HasPlaceholders,
    Unspecified,
}
