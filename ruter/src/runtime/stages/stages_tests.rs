use super::*;
use ruter::core::{CompilerCode, ErrorCode, Severity, SpanInfo};

#[test]
fn collect_error_stats_only_counts_error_level() {
    let error_diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: "e0433".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };
    let warning_diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0308,
            raw_code: None,
            explanation: None,
        }),
        message: "warning".to_string(),
        span: vec![],
        severity: Severity::Warning,
        children: vec![],
        rendered: None,
    };

    let stats = collect_error_stats(&[error_diag, warning_diag]);
    assert_eq!(stats.error_total, 1);
    assert_eq!(stats.error_by_code.get("E0433"), Some(&1));
    assert_eq!(stats.error_details.len(), 1);
}

#[test]
fn collect_error_stats_groups_by_code_and_no_code() {
    let e0433 = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: "a".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };
    let e0308 = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0308,
            raw_code: None,
            explanation: None,
        }),
        message: "b".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };
    let no_code = Diagnostic {
        message_type: None,
        code: None,
        message: "c".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };

    let stats = collect_error_stats(&[e0433, e0308, no_code]);
    assert_eq!(stats.error_total, 3);
    assert_eq!(stats.error_by_code.get("E0433"), Some(&1));
    assert_eq!(stats.error_by_code.get("E0308"), Some(&1));
    assert_eq!(stats.error_by_code.get("NO_CODE"), Some(&1));
}

#[test]
fn collect_error_stats_uses_raw_codes_for_e_and_non_e() {
    let raw_e = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::Unknown,
            raw_code: Some("E0765".to_string()),
            explanation: None,
        }),
        message: "raw e".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };
    let non_e = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::Unknown,
            raw_code: Some("unused_imports".to_string()),
            explanation: None,
        }),
        message: "lint".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };
    let stats = collect_error_stats(&[raw_e, non_e]);
    assert_eq!(stats.error_by_code.get("E0765"), Some(&1));
    assert_eq!(stats.error_by_code.get("unused_imports"), Some(&1));
}

#[test]
fn format_error_code_counts_renders_map() {
    let mut counts = BTreeMap::new();
    counts.insert("E0308".to_string(), 2);
    counts.insert("E0433".to_string(), 1);

    assert_eq!(format_error_code_counts(&counts), "E0308=2,E0433=1");
    assert_eq!(format_error_code_counts(&BTreeMap::new()), "none");
}

#[test]
fn default_rule_registry_registers_e0433_e0432_e0599_and_e0308() {
    let registry = build_default_rule_registry();
    let implemented = registry.implemented_error_codes();
    assert!(implemented.contains(&ErrorCode::E0433));
    assert!(implemented.contains(&ErrorCode::E0432));
    assert!(implemented.contains(&ErrorCode::E0599));
    assert!(implemented.contains(&ErrorCode::E0308));
}

#[test]
fn changed_files_detects_updates() {
    let file = PathBuf::from("a.rs");
    let mut before = HashMap::new();
    before.insert(file.clone(), "old".to_string());

    let mut after = HashMap::new();
    after.insert(file.clone(), "new".to_string());

    let changed = changed_files(&before, &after);
    assert_eq!(changed, vec![file]);
}

#[test]
fn normalize_paths_converts_relative_spans() {
    let mut diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: "x".to_string(),
        span: vec![SpanInfo {
            file_path: PathBuf::from("src/lib.rs"),
            byte_start: 0,
            byte_end: 1,
            line_start: 1,
            line_end: 1,
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

    diagnostics::normalize_paths(&mut diag, Path::new("/tmp/proj"));
    assert_eq!(
        diag.span[0].file_path,
        PathBuf::from("/tmp/proj/src/lib.rs")
    );
}

#[test]
fn backup_path_uses_relative_layout() {
    let crate_path = Path::new("/tmp/proj");
    let backups = Path::new("/tmp/proj/.ruter/backups");
    let file = Path::new("/tmp/proj/src/main.rs");

    let backup = io::backup_path_for(crate_path, backups, file);
    assert_eq!(
        backup,
        PathBuf::from("/tmp/proj/.ruter/backups/src/main.bak")
    );
}

#[test]
fn updated_snapshot_path_uses_suffix_layout() {
    let crate_path = Path::new("/tmp/proj");
    let updated_dir = Path::new("/tmp/proj/.ruter/run_x/updated");
    let file = Path::new("/tmp/proj/src/main.rs");

    let path = io::updated_snapshot_path_for(crate_path, updated_dir, file);
    assert_eq!(
        path,
        PathBuf::from("/tmp/proj/.ruter/run_x/updated/src/main_updated.rs")
    );
}

#[test]
fn extract_diagnostic_json_lines_filters_non_diagnostic_messages() {
    let stdout = br#"{"reason":"compiler-artifact","package_id":"demo 0.1.0"}
{"reason":"compiler-message","message":{"$message_type":"diagnostic","message":"e","code":{"code":"E0433"},"level":"error","spans":[],"children":[]}}
{"reason":"compiler-message","message":{"$message_type":"artifact","message":"skip-me"}}
"#;

    let lines = extract_diagnostic_json_lines(stdout).expect("extract diagnostics");
    assert_eq!(lines.lines().count(), 1);
    assert!(lines.contains("\"E0433\""));
}

#[test]
fn extract_diagnostic_json_lines_accepts_missing_message_type() {
    let stdout =
        br#"{"reason":"compiler-message","message":{"message":"e","code":{"code":"E0599"},"level":"error","spans":[],"children":[]}}
"#;
    let lines = extract_diagnostic_json_lines(stdout).expect("extract diagnostics");
    assert_eq!(lines.lines().count(), 1);
    assert!(lines.contains("\"E0599\""));
}
