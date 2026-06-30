use super::*;
use ruter::core::{Applicability, CompilerCode, Severity, SpanInfo};
use ruter::patchers::E0308Patcher;
use ruter::patchers::E0432Patcher;
use ruter::patchers::E0560Patcher;
use ruter::patchers::E0599Patcher;
use std::fs;
use tempfile::tempdir;

fn make_action(start: usize, end: usize, content: &str) -> FixAction {
    FixAction::Replace {
        span: SpanInfo {
            file_path: PathBuf::from("src/lib.rs"),
            byte_start: start,
            byte_end: end,
            line_start: 1,
            line_end: 1,
            col_start: 1,
            col_end: 1,
            is_primary: true,
            text: vec![],
            label: None,
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        },
        new_content: content.to_string(),
    }
}

fn ranked_list(diag_idx: usize, code: &str, sets: Vec<Vec<FixAction>>) -> DiagnosticRankedList {
    DiagnosticRankedList {
        diagnostic_index: diag_idx,
        code: code.to_string(),
        candidates: sets
            .into_iter()
            .enumerate()
            .map(|(i, actions)| RankedDiagnosticCandidate {
                rank: i + 1,
                score: 10 - i as i32,
                actions,
                patcher: "mock".to_string(),
            })
            .collect(),
    }
}

#[test]
fn zip_generation_no_cartesian() {
    let l1 = ranked_list(
        0,
        "E0433",
        vec![
            vec![make_action(0, 1, "a1")],
            vec![make_action(0, 1, "a2")],
            vec![make_action(0, 1, "a3")],
        ],
    );
    let l2 = ranked_list(
        1,
        "E0433",
        vec![
            vec![make_action(10, 11, "b1")],
            vec![make_action(10, 11, "b2")],
            vec![make_action(10, 11, "b3")],
        ],
    );

    let candidates = build_zip_candidates("f", &[l1, l2], 3);
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0].rank, 1);
    assert_eq!(candidates[1].rank, 2);
    assert_eq!(candidates[2].rank, 3);
}

#[test]
fn zip_padding_repeat_last_candidate() {
    let l1 = ranked_list(
        0,
        "E0433",
        vec![
            vec![make_action(0, 1, "a1")],
            vec![make_action(0, 1, "a2")],
            vec![make_action(0, 1, "a3")],
        ],
    );
    let l2 = ranked_list(1, "E0433", vec![vec![make_action(10, 11, "b1")]]);

    let candidates = build_zip_candidates("f", &[l1, l2], 3);
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[1].zip_alignment_meta.repeated_last_count, 1);
    assert_eq!(candidates[2].zip_alignment_meta.repeated_last_count, 1);
    assert!(candidates[2].diagnostic_trace[1].reused_last);
}

#[test]
fn zip_consumption_strict_rank_order() {
    let l1 = ranked_list(
        0,
        "E0433",
        vec![
            vec![make_action(0, 1, "first")],
            vec![make_action(0, 1, "second")],
        ],
    );

    let candidates = build_zip_candidates("f", &[l1], 2);
    assert_eq!(candidates[0].rank, 1);
    assert_eq!(candidates[1].rank, 2);
}

#[test]
fn ranked_candidates_for_non_e0433_without_patcher_is_empty() {
    let diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0425,
            raw_code: None,
            explanation: None,
        }),
        message: "x".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };
    let registry = PatcherRegistry::new();
    let ranked = ranked_candidates_for_diagnostic(&diag, &registry, 3).unwrap();
    assert!(ranked.is_empty());
}

#[test]
fn ranked_candidates_for_e0599_with_registered_placeholder_is_empty() {
    let diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0599,
            raw_code: None,
            explanation: None,
        }),
        message: "no function or associated item named `new` found for struct `Parser`".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };

    let mut registry = PatcherRegistry::new();
    registry.register(Box::new(E0599Patcher::new()));

    let ranked = ranked_candidates_for_diagnostic(&diag, &registry, 3).unwrap();
    assert!(ranked.is_empty());
}

#[test]
fn ranked_candidates_for_e0308_with_registered_placeholder_is_empty() {
    let diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0308,
            raw_code: None,
            explanation: None,
        }),
        message: "mismatched types".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };

    let mut registry = PatcherRegistry::new();
    registry.register(Box::new(E0308Patcher::new()));

    let ranked = ranked_candidates_for_diagnostic(&diag, &registry, 3).unwrap();
    assert!(ranked.is_empty());
}

#[test]
fn ranked_candidates_for_e0432_with_registered_placeholder_is_empty() {
    let diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0432,
            raw_code: None,
            explanation: None,
        }),
        message: "unresolved import `foo::bar`".to_string(),
        span: vec![],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };

    let mut registry = PatcherRegistry::new();
    registry.register(Box::new(E0432Patcher::new()));

    let ranked = ranked_candidates_for_diagnostic(&diag, &registry, 3).unwrap();
    assert!(ranked.is_empty());
}

#[test]
fn ranked_candidates_for_e0432_with_machine_applicable_returns_non_empty() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("Cargo.toml"),
        r#"[package]
name = "demo_pkg"
version = "0.1.0"
edition = "2021"
"#,
    )
    .expect("write manifest");

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("mkdir src");
    let file = src_dir.join("lib.rs");
    let source = "fn case_a() { use demo_pkg::duration; }\n";
    fs::write(&file, source).expect("write source");

    let start = source.find("demo_pkg::duration").expect("needle");
    let end = start + "demo_pkg::duration".len();
    let diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0432,
            raw_code: None,
            explanation: None,
        }),
        message: "unresolved import `demo_pkg::duration`".to_string(),
        span: vec![SpanInfo {
            file_path: file.clone(),
            byte_start: start,
            byte_end: end,
            line_start: 1,
            line_end: 1,
            col_start: start + 1,
            col_end: end + 1,
            is_primary: true,
            text: vec![],
            label: None,
            suggested_replacement: Some("crate::duration".to_string()),
            suggestion_applicability: Some(Applicability::MachineApplicable),
            expansion: None,
        }],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };

    let mut registry = PatcherRegistry::new();
    registry.register(Box::new(E0432Patcher::new()));

    let ranked = ranked_candidates_for_diagnostic(&diag, &registry, 3).unwrap();
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].rank, 1);
    assert!(!ranked[0].actions.is_empty());
}

#[test]
fn ranked_candidates_for_e0560_with_machine_or_maybe_returns_non_empty() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("lib.rs");
    let source = "fn case_a() { let _ = Parser { ofset: 1, pos: 2 }; }\n";
    fs::write(&file, source).expect("write source");

    let start = source.find("ofset").expect("needle");
    let end = start + "ofset".len();
    let diag = Diagnostic {
        message_type: None,
        code: Some(CompilerCode {
            code: ErrorCode::E0560,
            raw_code: None,
            explanation: None,
        }),
        message: "struct `Parser` has no field named `ofset`".to_string(),
        span: vec![SpanInfo {
            file_path: file.clone(),
            byte_start: start,
            byte_end: end,
            line_start: 1,
            line_end: 1,
            col_start: start + 1,
            col_end: end + 1,
            is_primary: true,
            text: vec![],
            label: Some("unknown field".to_string()),
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        }],
        severity: Severity::Error,
        children: vec![Diagnostic {
            message_type: None,
            code: None,
            message: "help: a field with a similar name exists".to_string(),
            span: vec![SpanInfo {
                file_path: file,
                byte_start: start,
                byte_end: end,
                line_start: 1,
                line_end: 1,
                col_start: start + 1,
                col_end: end + 1,
                is_primary: false,
                text: vec![],
                label: None,
                suggested_replacement: Some("offset".to_string()),
                suggestion_applicability: Some(Applicability::MaybeIncorrect),
                expansion: None,
            }],
            severity: Severity::Help,
            children: vec![],
            rendered: None,
        }],
        rendered: None,
    };

    let mut registry = PatcherRegistry::new();
    registry.register(Box::new(E0560Patcher::new()));

    let ranked = ranked_candidates_for_diagnostic(&diag, &registry, 3).unwrap();
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].rank, 1);
    assert!(!ranked[0].actions.is_empty());
}
