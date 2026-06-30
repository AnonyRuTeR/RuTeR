use super::*;
use ruter::core::{CompilerCode, ErrorCode};
use tempfile::tempdir;

#[test]
fn index_discovers_test_functions_and_maps_diagnostics() {
    let dir = tempdir().unwrap();
    let crate_dir = dir.path();
    fs::create_dir_all(crate_dir.join("src")).unwrap();
    fs::write(
        crate_dir.join("src/lib.rs"),
        r#"
mod foo {
    pub struct State;
}

#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        let _ = State::new();
    }

    #[tokio::test]
    async fn case_b() {
        let _ = State::new();
    }
}
"#,
    )
    .unwrap();

    let index = FunctionIndex::build(crate_dir).unwrap();
    assert_eq!(index.functions().len(), 2);

    let diagnostic = Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: "failed to resolve".to_string(),
        span: vec![SpanInfo {
            file_path: crate_dir.join("src/lib.rs"),
            byte_start: 110,
            byte_end: 115,
            line_start: 10,
            line_end: 10,
            col_start: 17,
            col_end: 22,
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

    let target = index.target_function_ids_for_errors(&[diagnostic], crate_dir);
    assert_eq!(target.len(), 1);
}

#[test]
fn function_mapping_prefers_line_when_byte_offsets_drift() {
    let dir = tempdir().unwrap();
    let crate_dir = dir.path();
    fs::create_dir_all(crate_dir.join("src")).unwrap();
    fs::write(
        crate_dir.join("src/lib.rs"),
        r#"
#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        let _a = 1;
    }

    #[test]
    fn case_b() {
        let _b = 2;
    }
}
"#,
    )
    .unwrap();

    let index = FunctionIndex::build(crate_dir).unwrap();
    let span = SpanInfo {
        file_path: crate_dir.join("src/lib.rs"),
        // Intentionally place byte range into case_a range.
        byte_start: 40,
        byte_end: 48,
        // But line points to case_b body.
        line_start: 10,
        line_end: 10,
        col_start: 9,
        col_end: 14,
        is_primary: true,
        text: vec![],
        label: None,
        suggested_replacement: None,
        suggestion_applicability: None,
        expansion: None,
    };

    let mapped = index
        .function_for_span(&span, crate_dir)
        .expect("function should be found");
    assert_eq!(mapped.fn_name, "case_b");
}

#[test]
fn error_diagnostics_capture_child_notes_help_and_suggestions() {
    let dir = tempdir().unwrap();
    let crate_dir = dir.path();
    fs::create_dir_all(crate_dir.join("src")).unwrap();
    fs::write(
        crate_dir.join("src/lib.rs"),
        r#"
#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        let _ = Some(1);
    }
}
"#,
    )
    .unwrap();

    let index = FunctionIndex::build(crate_dir).unwrap();
    let diagnostic = Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0308,
            raw_code: None,
            explanation: None,
        }),
        message: "mismatched types".to_string(),
        span: vec![SpanInfo {
            file_path: crate_dir.join("src/lib.rs"),
            byte_start: 48,
            byte_end: 57,
            line_start: 6,
            line_end: 6,
            col_start: 17,
            col_end: 26,
            is_primary: true,
            text: vec![],
            label: Some("expected `Option<String>`, found `Option<i32>`".to_string()),
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        }],
        severity: Severity::Error,
        children: vec![
            Diagnostic {
                message_type: None,
                code: None,
                message: "note: expected enum `Option<String>`, found enum `Option<i32>`"
                    .to_string(),
                span: vec![],
                severity: Severity::Note,
                children: vec![],
                rendered: None,
            },
            Diagnostic {
                message_type: None,
                code: None,
                message: "help: try converting to string".to_string(),
                span: vec![SpanInfo {
                    file_path: crate_dir.join("src/lib.rs"),
                    byte_start: 48,
                    byte_end: 57,
                    line_start: 6,
                    line_end: 6,
                    col_start: 17,
                    col_end: 26,
                    is_primary: false,
                    text: vec![],
                    label: None,
                    suggested_replacement: Some(".to_string()".to_string()),
                    suggestion_applicability: None,
                    expansion: None,
                }],
                severity: Severity::Help,
                children: vec![],
                rendered: None,
            },
        ],
        rendered: None,
    };

    let grouped = index.error_diagnostics_by_function(&[diagnostic], crate_dir);
    let only_function = grouped
        .values()
        .next()
        .expect("mapped function diagnostics must exist");
    assert_eq!(only_function.len(), 1);
    let evidence = &only_function[0];
    assert!(
        evidence
            .children_note_messages
            .iter()
            .any(|line| line.contains("expected enum"))
    );
    assert!(
        evidence
            .children_help_messages
            .iter()
            .any(|line| line.contains("try converting"))
    );
    assert!(
        evidence
            .children_suggested_replacements
            .contains(&".to_string()".to_string())
    );
}

#[test]
fn test_module_level_diagnostic_falls_back_to_nearest_test_function() {
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
        .expect("needle must exist");
    let line_start = source[..start].bytes().filter(|b| *b == b'\n').count() + 1;
    let line_len = source
        .lines()
        .nth(line_start - 1)
        .map(str::len)
        .unwrap_or(1);
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
            col_end: line_len,
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
    let target = index.target_function_ids_for_errors(&[diagnostic], crate_dir);
    assert_eq!(target.len(), 1, "module-level test diagnostic should map");
}

#[test]
fn enclosing_scope_prefers_cfg_test_module_range() {
    let dir = tempdir().unwrap();
    let crate_dir = dir.path();
    fs::create_dir_all(crate_dir.join("src")).unwrap();
    fs::write(
        crate_dir.join("src/lib.rs"),
        r#"
#[cfg(test)]
mod tests_outer {
    #[test]
    fn case_a() {}

    mod nested {
        #[test]
        fn case_b() {}
    }
}
"#,
    )
    .unwrap();

    let index = FunctionIndex::build(crate_dir).unwrap();
    let case_b = index
        .functions()
        .iter()
        .find(|item| item.fn_name == "case_b")
        .expect("case_b should exist");

    let scope = index
        .enclosing_test_module_scope(case_b)
        .expect("scope should exist");
    assert_eq!(
        scope.module_path,
        vec!["tests_outer".to_string(), "nested".to_string()]
    );
    assert!(scope.line_start <= case_b.line_start);
    assert!(scope.line_end >= case_b.line_end);
}

#[test]
fn same_test_module_ids_return_siblings_with_fallback() {
    let dir = tempdir().unwrap();
    let crate_dir = dir.path();
    fs::create_dir_all(crate_dir.join("src")).unwrap();
    fs::write(
        crate_dir.join("src/lib.rs"),
        r#"
#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {}

    #[test]
    fn case_b() {}
}

#[cfg(test)]
mod other_tests {
    #[test]
    fn case_c() {}
}
"#,
    )
    .unwrap();

    let index = FunctionIndex::build(crate_dir).unwrap();
    let case_a = index
        .functions()
        .iter()
        .find(|item| item.fn_name == "case_a")
        .expect("case_a should exist");
    let ids = index.function_ids_in_same_test_module(&case_a.id);

    assert_eq!(ids.len(), 2);
    assert!(ids.iter().any(|id| id.contains("case_a")));
    assert!(ids.iter().any(|id| id.contains("case_b")));
    assert!(!ids.iter().any(|id| id.contains("case_c")));

    let unknown = index.function_ids_in_same_test_module("missing::function:id");
    assert_eq!(
        unknown,
        std::collections::BTreeSet::from(["missing::function:id".to_string()])
    );
}
