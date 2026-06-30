use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use ruter::core::{FixAction, FunctionDiagnostic};
use serde::{Deserialize, Serialize};

use crate::runtime::function::index::FunctionIndex;
use crate::runtime::stages::PreparedPatch;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmHandoffItem {
    pub function_id: String,
    pub file_path: Option<PathBuf>,
    pub module_path: Option<String>,
    pub fn_name: Option<String>,
    pub line_span: Option<String>,
    pub remaining_error_codes: BTreeMap<String, usize>,
    pub remaining_error_messages: Vec<String>,
    pub source_snippet: Option<String>,
    pub already_applied_neighbor_fixes: Vec<String>,
}

pub fn build_llm_handoff_items(
    function_index: &FunctionIndex,
    unresolved_function_ids: &BTreeSet<String>,
    error_diagnostics_by_function: &BTreeMap<String, Vec<FunctionDiagnostic>>,
    partial_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    prepared: &PreparedPatch,
) -> Vec<LlmHandoffItem> {
    let mut out = Vec::new();

    for function_id in unresolved_function_ids {
        if function_id == "__UNMAPPED_ERRORS__" {
            out.push(LlmHandoffItem {
                function_id: function_id.clone(),
                file_path: None,
                module_path: None,
                fn_name: None,
                line_span: None,
                remaining_error_codes: collapse_code_counts(
                    error_diagnostics_by_function
                        .get(function_id)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                ),
                remaining_error_messages: error_diagnostics_by_function
                    .get(function_id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|diag| diag.message)
                    .collect(),
                source_snippet: None,
                already_applied_neighbor_fixes: Vec::new(),
            });
            continue;
        }

        let function = function_index.get(function_id);
        let source_snippet = function.and_then(|f| {
            let source = prepared
                .source_for_file(&f.file_path)
                .or_else(|| fs::read_to_string(&f.file_path).ok())?;
            let start = f.byte_start.min(source.len());
            let end = f.byte_end.min(source.len());
            if start >= end {
                return None;
            }
            Some(source[start..end].to_string())
        });

        let neighbor_fix_count = function
            .map(|f| {
                partial_plan
                    .get(&f.file_path)
                    .map(Vec::len)
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        out.push(LlmHandoffItem {
            function_id: function_id.clone(),
            file_path: function.map(|f| f.file_path.clone()),
            module_path: function.map(|f| f.module_path.join("::")),
            fn_name: function.map(|f| f.fn_name.clone()),
            line_span: function.map(|f| format!("{}:{}", f.line_start, f.line_end)),
            remaining_error_codes: collapse_code_counts(
                error_diagnostics_by_function
                    .get(function_id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
            ),
            remaining_error_messages: error_diagnostics_by_function
                .get(function_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|diag| diag.message)
                .collect(),
            source_snippet,
            already_applied_neighbor_fixes: vec![format!(
                "applied_actions_in_same_file={neighbor_fix_count}"
            )],
        });
    }

    out
}

fn collapse_code_counts(diags: &[FunctionDiagnostic]) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for diag in diags {
        *out.entry(diag.code.clone()).or_insert(0) += 1;
    }
    out
}
