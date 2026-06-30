use super::*;
use ruter::core::{CompilerCode, Severity};
use std::fs;
use tempfile::tempdir;

use crate::runtime::function::index::FunctionIndex;

fn make_diag(
    index: usize,
    start: usize,
    end: usize,
    code: ErrorCode,
    is_primary: bool,
) -> DiagnosticRef {
    DiagnosticRef {
        diagnostic_index: index,
        code: code.to_string(),
        diagnostic: Diagnostic {
            message_type: None,
            code: Some(CompilerCode {
                code,
                raw_code: Some(code.to_string()),
                explanation: None,
            }),
            message: "x".to_string(),
            span: vec![SpanInfo {
                file_path: PathBuf::from("src/lib.rs"),
                byte_start: start,
                byte_end: end,
                line_start: 1,
                line_end: 1,
                col_start: start + 1,
                col_end: end + 1,
                is_primary,
                text: vec![],
                label: None,
                suggested_replacement: None,
                suggestion_applicability: None,
                expansion: None,
            }],
            severity: Severity::Error,
            children: vec![],
            rendered: None,
        },
    }
}

#[test]
fn dispatch_rule_first_when_mixed_codes() {
    let task = FunctionPatchTask {
        function_id: "f".to_string(),
        file_path: PathBuf::from("src/lib.rs"),
        function_line_span: (1, 10),
        diagnostics_with_index: vec![],
        error_code_counts: BTreeMap::new(),
        implemented_rule_codes_present: BTreeSet::from([ErrorCode::E0433]),
        unimplemented_codes_present: BTreeSet::from(["E0425".to_string()]),
        low_value_status: LowValueStatus::HasTestSemantics,
        low_value_reason: "markers".to_string(),
        low_value_markers: vec!["macro:assert!".to_string()],
    };

    let decision = dispatch_task(&task);
    match decision {
        FunctionDispatchDecision::RulePatcher { .. } => {}
        _ => panic!("expected rule patcher"),
    }
}

#[test]
fn dispatch_rule_first_when_e0308_is_registered() {
    let task = FunctionPatchTask {
        function_id: "f".to_string(),
        file_path: PathBuf::from("src/lib.rs"),
        function_line_span: (1, 10),
        diagnostics_with_index: vec![],
        error_code_counts: BTreeMap::from([("E0308".to_string(), 1)]),
        implemented_rule_codes_present: BTreeSet::from([ErrorCode::E0308]),
        unimplemented_codes_present: BTreeSet::new(),
        low_value_status: LowValueStatus::HasTestSemantics,
        low_value_reason: "markers".to_string(),
        low_value_markers: vec!["macro:assert!".to_string()],
    };

    let decision = dispatch_task(&task);
    match decision {
        FunctionDispatchDecision::RulePatcher {
            selected_rule_codes,
            ..
        } => {
            assert_eq!(selected_rule_codes, vec!["E0308".to_string()]);
        }
        _ => panic!("expected rule patcher"),
    }
}

#[test]
fn dispatch_to_llm_when_no_implemented_rule_code() {
    let task = FunctionPatchTask {
        function_id: "f".to_string(),
        file_path: PathBuf::from("src/lib.rs"),
        function_line_span: (1, 10),
        diagnostics_with_index: vec![],
        error_code_counts: BTreeMap::new(),
        implemented_rule_codes_present: BTreeSet::new(),
        unimplemented_codes_present: BTreeSet::from(["E0425".to_string()]),
        low_value_status: LowValueStatus::HasTestSemantics,
        low_value_reason: "markers".to_string(),
        low_value_markers: vec!["macro:assert!".to_string()],
    };

    let decision = dispatch_task(&task);
    match decision {
        FunctionDispatchDecision::LlmPatcher { .. } => {}
        _ => panic!("expected llm patcher"),
    }
}

#[test]
fn aggregate_primary_wins_when_spans_overlap() {
    let task = FunctionPatchTask {
        function_id: "f".to_string(),
        file_path: PathBuf::from("src/lib.rs"),
        function_line_span: (1, 10),
        diagnostics_with_index: vec![
            make_diag(0, 10, 20, ErrorCode::E0433, true),
            make_diag(1, 12, 18, ErrorCode::E0433, true),
        ],
        error_code_counts: BTreeMap::new(),
        implemented_rule_codes_present: BTreeSet::from([ErrorCode::E0433]),
        unimplemented_codes_present: BTreeSet::new(),
        low_value_status: LowValueStatus::HasTestSemantics,
        low_value_reason: "markers".to_string(),
        low_value_markers: vec!["macro:assert!".to_string()],
    };
    let decision = dispatch_task(&task);
    let set = arbitrate_rule_diagnostics(&task, &decision);
    assert_eq!(set.selected_diagnostics.len(), 1);
    assert_eq!(set.selected_diagnostics[0].diagnostic_index, 0);
    assert_eq!(set.suppressed_diagnostics.len(), 1);
}

#[test]
fn aggregate_append_when_spans_non_overlapping() {
    let task = FunctionPatchTask {
        function_id: "f".to_string(),
        file_path: PathBuf::from("src/lib.rs"),
        function_line_span: (1, 10),
        diagnostics_with_index: vec![
            make_diag(0, 10, 20, ErrorCode::E0433, true),
            make_diag(1, 30, 40, ErrorCode::E0433, true),
        ],
        error_code_counts: BTreeMap::new(),
        implemented_rule_codes_present: BTreeSet::from([ErrorCode::E0433]),
        unimplemented_codes_present: BTreeSet::new(),
        low_value_status: LowValueStatus::HasTestSemantics,
        low_value_reason: "markers".to_string(),
        low_value_markers: vec!["macro:assert!".to_string()],
    };
    let decision = dispatch_task(&task);
    let set = arbitrate_rule_diagnostics(&task, &decision);
    assert_eq!(set.selected_diagnostics.len(), 2);
    assert!(set.suppressed_diagnostics.is_empty());
}

#[test]
fn build_tasks_maps_test_module_level_diagnostic_into_rule_task() {
    let dir = tempdir().unwrap();
    let crate_dir = dir.path();
    fs::create_dir_all(crate_dir.join("src")).unwrap();
    fs::write(
        crate_dir.join("src/lib.rs"),
        r#"
mod wrapper {
    pub struct Duration;
}

#[cfg(test)]
mod tests {
    use crate::humantime_gpt_4_1_nano_20251109_132117::wrapper::Duration;

    #[test]
    fn case_a() {
        let _ = Duration;
    }
}
"#,
    )
    .unwrap();

    let source = fs::read_to_string(crate_dir.join("src/lib.rs")).unwrap();
    let start = source
        .find("humantime_gpt_4_1_nano_20251109_132117")
        .expect("needle should exist");
    let line_start = source[..start].bytes().filter(|b| *b == b'\n').count() + 1;
    let diagnostic = Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message:
            "failed to resolve: could not find `humantime_gpt_4_1_nano_20251109_132117` in the crate root"
                .to_string(),
        span: vec![SpanInfo {
            file_path: crate_dir.join("src/lib.rs"),
            byte_start: start,
            byte_end: start + "humantime_gpt_4_1_nano_20251109_132117".len(),
            line_start,
            line_end: line_start,
            col_start: 1,
            col_end: 2,
            is_primary: true,
            text: vec![],
            label: None,
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        }],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };

    let index = FunctionIndex::build(crate_dir).unwrap();
    let implemented = BTreeSet::from([ErrorCode::E0433]);
    let tasks = build_function_patch_tasks(&[diagnostic], crate_dir, &index, &implemented);
    assert_eq!(tasks.len(), 1, "module-level test diagnostic should become task");
    assert!(
        tasks[0]
            .implemented_rule_codes_present
            .contains(&ErrorCode::E0433)
    );
}
