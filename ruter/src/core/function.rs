use std::path::PathBuf;

/// Stable identity and source range for one test function.
#[derive(Debug, Clone)]
pub struct TestFunction {
    pub id: String,
    pub relative_file: PathBuf,
    pub file_path: PathBuf,
    pub module_path: Vec<String>,
    pub fn_name: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line_start: usize,
    pub line_end: usize,
}

/// Function-scoped diagnostic evidence used by dispatch, preflight and LLM context.
#[derive(Debug, Clone)]
pub struct FunctionDiagnostic {
    pub code: String,
    pub message: String,
    pub primary_span: Option<String>,
    pub label: Option<String>,
    pub suggested_replacement: Option<String>,
    pub children_note_messages: Vec<String>,
    pub children_help_messages: Vec<String>,
    pub children_suggested_replacements: Vec<String>,
}
