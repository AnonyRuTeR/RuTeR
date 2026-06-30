use std::fs;
use std::path::{Path, PathBuf};

use super::*;
use crate::core::{CompilerCode, ErrorCode, Severity};
use tempfile::tempdir;

fn write_source(source: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("lib.rs");
    fs::write(&file, source).expect("write source");
    (dir, file)
}

fn span_for(
    file: &Path,
    source: &str,
    needle: &str,
    is_primary: bool,
    replacement: Option<&str>,
    applicability: Option<Applicability>,
) -> SpanInfo {
    let start = source.find(needle).expect("needle must exist");
    let end = start + needle.len();
    SpanInfo {
        file_path: file.to_path_buf(),
        byte_start: start,
        byte_end: end,
        line_start: 1,
        line_end: 1,
        col_start: start + 1,
        col_end: end + 1,
        is_primary,
        text: vec![],
        label: Some("unknown field".to_string()),
        suggested_replacement: replacement.map(ToString::to_string),
        suggestion_applicability: applicability,
        expansion: None,
    }
}

fn diag_with_children(
    message: &str,
    spans: Vec<SpanInfo>,
    children: Vec<Diagnostic>,
) -> Diagnostic {
    Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0560,
            raw_code: None,
            explanation: None,
        }),
        message: message.to_string(),
        span: spans,
        severity: Severity::Error,
        children,
        rendered: None,
    }
}

#[test]
fn p1_accepts_maybeincorrect_span_replacement() {
    let source = "let _ = Parser { ofset: 1, pos: 2 };";
    let (_guard, file) = write_source(source);
    let primary = span_for(
        &file,
        source,
        "ofset",
        true,
        None,
        Some(Applicability::MaybeIncorrect),
    );
    let child = Diagnostic {
        message_type: None,
        code: None,
        message: "help: a field with a similar name exists".to_string(),
        span: vec![span_for(
            &file,
            source,
            "ofset",
            false,
            Some("offset"),
            Some(Applicability::MaybeIncorrect),
        )],
        severity: Severity::Help,
        children: vec![],
        rendered: None,
    };
    let diagnostic = diag_with_children(
        "struct `Parser` has no field named `ofset`",
        vec![primary],
        vec![child],
    );

    let actions = analyze_e0560_diagnostic(&diagnostic).expect("analyze");
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        FixAction::Replace { new_content, .. } => assert_eq!(new_content, "offset"),
        _ => panic!("expected replace"),
    }
}

#[test]
fn r1_uses_available_fields_when_p1_absent() {
    let source = "let _ = Parser { ofset: 1, pos: 2 };";
    let (_guard, file) = write_source(source);
    let primary = span_for(&file, source, "ofset", true, None, None);
    let child = Diagnostic {
        message_type: None,
        code: None,
        message: "note: available fields are: `offset`, `pos`".to_string(),
        span: vec![],
        severity: Severity::Note,
        children: vec![],
        rendered: None,
    };
    let diagnostic = diag_with_children(
        "struct `Parser` has no field named `ofset`",
        vec![primary],
        vec![child],
    );

    let actions = analyze_e0560_diagnostic(&diagnostic).expect("analyze");
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        FixAction::Replace { new_content, .. } => assert_eq!(new_content, "offset"),
        _ => panic!("expected replace"),
    }
}

#[test]
fn r1_rejects_ambiguous_edit_distance_candidates() {
    let source = "let _ = Parser { boo: 1, pos: 2 };";
    let (_guard, file) = write_source(source);
    let primary = span_for(&file, source, "boo", true, None, None);
    let child = Diagnostic {
        message_type: None,
        code: None,
        message: "note: available fields are: `foo`, `coo`".to_string(),
        span: vec![],
        severity: Severity::Note,
        children: vec![],
        rendered: None,
    };
    let diagnostic = diag_with_children(
        "struct `Parser` has no field named `boo`",
        vec![primary],
        vec![child],
    );

    let actions = analyze_e0560_diagnostic(&diagnostic).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn r1_rejects_candidates_when_distance_is_greater_than_one() {
    let source = "let _ = Parser { ofs: 1, pos: 2 };";
    let (_guard, file) = write_source(source);
    let primary = span_for(&file, source, "ofs", true, None, None);
    let child = Diagnostic {
        message_type: None,
        code: None,
        message: "note: available fields are: `offset`, `pos`".to_string(),
        span: vec![],
        severity: Severity::Note,
        children: vec![],
        rendered: None,
    };
    let diagnostic = diag_with_children(
        "struct `Parser` has no field named `ofs`",
        vec![primary],
        vec![child],
    );

    let actions = analyze_e0560_diagnostic(&diagnostic).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn r1_rejects_non_explicit_named_field_span() {
    let source = "let ofset = 1; let _ = Parser { pos: 2 };";
    let (_guard, file) = write_source(source);
    let primary = span_for(&file, source, "ofset", true, None, None);
    let child = Diagnostic {
        message_type: None,
        code: None,
        message: "note: available fields are: `offset`, `pos`".to_string(),
        span: vec![],
        severity: Severity::Note,
        children: vec![],
        rendered: None,
    };
    let diagnostic = diag_with_children(
        "struct `Parser` has no field named `ofset`",
        vec![primary],
        vec![child],
    );

    let actions = analyze_e0560_diagnostic(&diagnostic).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn r1_rejects_when_multiple_unknown_field_mentions_exist() {
    let source = "let _ = Parser { foo: 1, bar: 2 };";
    let (_guard, file) = write_source(source);
    let primary = span_for(&file, source, "foo", true, None, None);
    let child = Diagnostic {
        message_type: None,
        code: None,
        message: "note: available fields are: `fooz`".to_string(),
        span: vec![],
        severity: Severity::Note,
        children: vec![],
        rendered: None,
    };
    let diagnostic = diag_with_children(
        "struct `Parser` has no field named `foo`; struct `Parser` has no field named `bar`",
        vec![primary],
        vec![child],
    );

    let actions = analyze_e0560_diagnostic(&diagnostic).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn hint_extraction_collects_unknown_available_and_compiler_hints() {
    let source = "let _ = Parser { ofset: 1, pos: 2 };";
    let (_guard, file) = write_source(source);
    let diagnostic = diag_with_children(
        "struct `Parser` has no field named `ofset`",
        vec![span_for(&file, source, "ofset", true, None, None)],
        vec![Diagnostic {
            message_type: None,
            code: None,
            message: "note: available fields are: `offset`, `pos`".to_string(),
            span: vec![span_for(
                &file,
                source,
                "ofset",
                false,
                Some("offset"),
                Some(Applicability::MaybeIncorrect),
            )],
            severity: Severity::Help,
            children: vec![],
            rendered: None,
        }],
    );

    let hints = analyze_e0560_hints(&diagnostic);
    assert_eq!(hints.unknown_fields, vec!["ofset".to_string()]);
    assert!(hints.available_fields.iter().any(|field| field == "offset"));
    assert!(hints.compiler_hints.iter().any(|field| field == "offset"));
}

#[test]
fn edit_distance_exactly_one_is_case_insensitive() {
    assert!(edit_distance_is_one_ascii_ci("OFset", "offset"));
    assert!(!edit_distance_is_one_ascii_ci("offset", "offset"));
    assert!(!edit_distance_is_one_ascii_ci("abc", "xyz"));
}
