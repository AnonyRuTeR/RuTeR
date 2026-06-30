use super::budget::context_char_count;
use super::*;
use ruter::core::TestFunction;
use std::path::PathBuf;

fn sample_function() -> TestFunction {
    TestFunction {
        id: "tests/basic.rs::tests::case_a:10:20".to_string(),
        relative_file: PathBuf::from("tests/basic.rs"),
        file_path: PathBuf::from("/tmp/crate/tests/basic.rs"),
        module_path: vec!["tests".to_string()],
        fn_name: "case_a".to_string(),
        byte_start: 0,
        byte_end: 44,
        line_start: 1,
        line_end: 3,
    }
}

#[test]
fn llm_context_builds_and_prunes_with_stable_order() {
    let source = "#[test]\nfn case_a() { let _ = State::new(); }\nuse crate::foo::State;\n";
    let function = sample_function();
    let diagnostics = vec![FunctionDiagnostic {
        code: "E0433".to_string(),
        message: "failed to resolve".to_string(),
        primary_span: Some("tests/basic.rs:1:1-1:5".to_string()),
        label: Some("unresolved".to_string()),
        suggested_replacement: Some("crate::foo::State".to_string()),
        children_note_messages: vec![],
        children_help_messages: vec![],
        children_suggested_replacements: vec![],
    }];
    let outcome = build_context_bundle_v1(
        Path::new("/tmp/crate"),
        &function,
        source,
        &diagnostics,
        vec!["applied_actions_in_same_file=1".to_string()],
        None,
        None,
        None,
        None,
        &ContextBuildConfig {
            max_chars: 2_000,
            target_fn_hard_limit_chars: 200,
            primary_items_limit: 4,
        },
    );

    match outcome {
        ContextBuildOutcome::Ready { bundle, .. } => {
            assert_eq!(bundle.context_schema_version, "1");
            assert_eq!(bundle.function_id, function.id);
            assert_eq!(
                bundle.location.file_path_redacted,
                "<CRATE_ROOT>/tests/basic.rs"
            );
            assert_eq!(bundle.diagnostics_digest.error_code_counts["E0433"], 1);
            assert!(
                !bundle.diagnostics_digest.primary_items.is_empty(),
                "primary diagnostics should be kept under normal context budget"
            );
            assert_eq!(
                bundle.diagnostics_digest.primary_items[0]
                    .primary_span
                    .as_deref()
                    .unwrap_or_default(),
                "<CRATE_ROOT>/tests/basic.rs:1:1-1:5"
            );
        }
        ContextBuildOutcome::TooLarge { reason, .. } => {
            panic!("unexpected too large: {reason}");
        }
    }
}

#[test]
fn llm_context_collects_related_free_fn_defs_from_diagnostic_hints() {
    let target =
        "#[test]\nfn case_a() { let _ = crate::date::two_digits::two_digits_inner('1', '2'); }\n";
    let helper = "#[inline]\nfn two_digits(b1: u8, b2: u8) -> Result<u64, Error> { let _ = (b1, b2); Ok(1) }\n";
    let source = format!("{target}{helper}");
    let mut function = sample_function();
    function.byte_end = target.len();

    let diagnostics = vec![FunctionDiagnostic {
        code: "E0433".to_string(),
        message: "failed to resolve: expected type, found function `two_digits` in `date`"
            .to_string(),
        primary_span: Some("tests/basic.rs:1:1-1:5".to_string()),
        label: Some("expected type, found function `two_digits` in `date`".to_string()),
        suggested_replacement: None,
        children_note_messages: vec![],
        children_help_messages: vec![],
        children_suggested_replacements: vec![],
    }];

    let outcome = build_context_bundle_v1(
        Path::new("/tmp/crate"),
        &function,
        &source,
        &diagnostics,
        vec![],
        None,
        None,
        None,
        None,
        &ContextBuildConfig {
            max_chars: 2000,
            target_fn_hard_limit_chars: 400,
            primary_items_limit: 4,
        },
    );

    match outcome {
        ContextBuildOutcome::Ready { bundle, .. } => {
            assert!(
                !bundle.related_fn_defs.is_empty(),
                "should include hinted helper function definitions"
            );
            assert!(
                bundle.related_fn_defs[0].contains("fn two_digits"),
                "expected two_digits helper in related_fn_defs"
            );
        }
        ContextBuildOutcome::TooLarge { reason, .. } => {
            panic!("unexpected too large: {reason}")
        }
    }
}

#[test]
fn llm_context_collects_related_type_defs_for_namespaced_expr_struct_symbol() {
    let target = "#[test]\nfn case_a() { let _ = foo::Parser { offset: 1, pos: 2 }; }\n";
    let type_def = "mod foo { pub struct Parser { pub offset: usize, pub pos: usize } }\n";
    let source = format!("{target}\n{type_def}");
    let mut function = sample_function();
    function.byte_end = target.len();

    let diagnostics = vec![FunctionDiagnostic {
        code: "E0560".to_string(),
        message: "struct `Parser` has no field named `off`".to_string(),
        primary_span: Some("tests/basic.rs:1:1-1:5".to_string()),
        label: Some("unknown field".to_string()),
        suggested_replacement: Some("offset".to_string()),
        children_note_messages: vec!["available fields are: `offset`, `pos`".to_string()],
        children_help_messages: vec![],
        children_suggested_replacements: vec![],
    }];

    let outcome = build_context_bundle_v1(
        Path::new("/tmp/crate"),
        &function,
        &source,
        &diagnostics,
        vec![],
        None,
        None,
        None,
        None,
        &ContextBuildConfig {
            max_chars: 4_000,
            target_fn_hard_limit_chars: 800,
            primary_items_limit: 4,
        },
    );

    match outcome {
        ContextBuildOutcome::Ready { bundle, .. } => {
            assert!(
                bundle
                    .related_type_defs
                    .iter()
                    .any(|item| item.contains("struct Parser")),
                "expected Parser struct definition to be collected from foo::Parser literal"
            );
        }
        ContextBuildOutcome::TooLarge { reason, .. } => {
            panic!("unexpected too large: {reason}");
        }
    }
}

#[test]
fn llm_context_reports_context_too_large_for_huge_target_fn() {
    let source = format!("#[test]\nfn case_a() {{ {} }}\n", "a".repeat(500));
    let mut function = sample_function();
    function.byte_end = source.len();

    let outcome = build_context_bundle_v1(
        Path::new("/tmp/crate"),
        &function,
        &source,
        &[],
        vec![],
        None,
        None,
        None,
        None,
        &ContextBuildConfig {
            max_chars: 1200,
            target_fn_hard_limit_chars: 64,
            primary_items_limit: 4,
        },
    );

    assert!(matches!(outcome, ContextBuildOutcome::TooLarge { .. }));
}

#[test]
fn llm_context_history_digest_is_backward_compatible_with_missing_field() {
    let raw = r##"{
          "context_schema_version": "1",
          "function_id": "f1",
          "location": {
            "file_path_redacted": "<CRATE_ROOT>/tests/basic.rs",
            "module_path": "tests",
            "fn_name": "case_a",
            "line_span": "1:3"
          },
          "target_function_text": "#[test]\nfn case_a() {}\n",
          "diagnostics_digest": {
            "error_code_counts": {},
            "primary_items": []
          },
          "constraints": {
            "test_only": true,
            "same_function_only": true,
            "signature_attrs_immutable": true,
            "no_non_target_regression": true
          },
          "related_imports": [],
          "related_type_defs": [],
          "related_impl_blocks": [],
          "related_fn_defs": [],
          "neighbor_fix_digest": [],
          "truncated_sections": []
        }"##;
    let bundle: FunctionContextBundleV1 =
        serde_json::from_str(raw).expect("bundle should parse without history field");
    assert!(bundle.previous_round_failure_digest.is_none());
    assert!(bundle.rule_error_drift_digest.is_none());
}

#[test]
fn llm_context_prunes_history_after_other_sections() {
    let mut bundle = FunctionContextBundleV1 {
        context_schema_version: "1".to_string(),
        function_id: "f1".to_string(),
        location: ContextLocation {
            file_path_redacted: "<CRATE_ROOT>/tests/basic.rs".to_string(),
            module_path: "tests".to_string(),
            fn_name: "case_a".to_string(),
            line_span: "1:3".to_string(),
        },
        target_function_text: "#[test]\nfn case_a() {}\n".to_string(),
        diagnostics_digest: DiagnosticsDigest {
            error_code_counts: BTreeMap::new(),
            primary_items: vec![],
        },
        constraints: ContextConstraints {
            test_only: true,
            same_function_only: true,
            signature_attrs_immutable: true,
            no_non_target_regression: true,
        },
        related_imports: vec![],
        related_type_defs: vec![],
        related_impl_blocks: vec![],
        related_fn_defs: vec![],
        neighbor_fix_digest: vec![],
        local_rule_failure_digest: None,
        rule_error_drift_digest: None,
        preflight_interceptor_digest: None,
        previous_round_failure_digest: Some(PreviousRoundFailureDigestV1 {
            round: 1,
            dominant_failure_kinds: vec!["LLM_VERIFY_FAILED".to_string()],
            candidate_failures: vec![
                PreviousCandidateFailureDigestV1 {
                    candidate_id: "c1".to_string(),
                    patch_summary: format!("replace unresolved path {}", "A".repeat(180)),
                    failure_kind: "LLM_VERIFY_FAILED".to_string(),
                    failure_detail: Some("target function still unresolved".to_string()),
                    unresolved_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
                    introduced_error_by_code: BTreeMap::new(),
                },
                PreviousCandidateFailureDigestV1 {
                    candidate_id: "c2".to_string(),
                    patch_summary: format!("replace unresolved path {}", "B".repeat(180)),
                    failure_kind: "LLM_VERIFY_FAILED".to_string(),
                    failure_detail: Some("target function still unresolved".to_string()),
                    unresolved_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
                    introduced_error_by_code: BTreeMap::new(),
                },
            ],
        }),
        truncated_sections: vec![],
    };
    let full_chars = context_char_count(&bundle);
    let mut one_failure_bundle = bundle.clone();
    let _ = one_failure_bundle
        .previous_round_failure_digest
        .as_mut()
        .and_then(|history| history.candidate_failures.pop());
    let one_failure_chars = context_char_count(&one_failure_bundle);
    let max_chars = one_failure_chars + 16;
    assert!(full_chars > max_chars);

    let outcome = prune_bundle_to_budget(&mut bundle, max_chars);

    match outcome {
        ContextBuildOutcome::Ready { bundle, .. } => {
            assert!(
                bundle
                    .truncated_sections
                    .iter()
                    .any(|item| item == "history"),
                "history should be truncated when budget is tight"
            );
            let history = bundle
                .previous_round_failure_digest
                .expect("history digest should remain present");
            assert_eq!(history.candidate_failures.len(), 1);
        }
        ContextBuildOutcome::TooLarge { reason, .. } => {
            panic!("unexpected too large: {reason}");
        }
    }
}

#[test]
fn llm_context_prunes_local_rule_digest_when_budget_is_tight() {
    let mut bundle = FunctionContextBundleV1 {
        context_schema_version: "1".to_string(),
        function_id: "f1".to_string(),
        location: ContextLocation {
            file_path_redacted: "<CRATE_ROOT>/tests/basic.rs".to_string(),
            module_path: "tests".to_string(),
            fn_name: "case_a".to_string(),
            line_span: "1:3".to_string(),
        },
        target_function_text: "#[test]\nfn case_a() {}\n".to_string(),
        diagnostics_digest: DiagnosticsDigest {
            error_code_counts: BTreeMap::new(),
            primary_items: vec![],
        },
        constraints: ContextConstraints {
            test_only: true,
            same_function_only: true,
            signature_attrs_immutable: true,
            no_non_target_regression: true,
        },
        related_imports: vec![],
        related_type_defs: vec![],
        related_impl_blocks: vec![],
        related_fn_defs: vec![],
        neighbor_fix_digest: vec![],
        local_rule_failure_digest: Some(LocalRuleFailureDigestV1 {
            summary_lines: vec![
                format!("selected_rank_history={}", "r1:1,r2:2,".repeat(20)),
                format!("remaining_error_by_code={}", "E0433=1,".repeat(20)),
            ],
        }),
        rule_error_drift_digest: None,
        preflight_interceptor_digest: None,
        previous_round_failure_digest: None,
        truncated_sections: vec![],
    };

    let full_chars = context_char_count(&bundle);
    let mut one_line_bundle = bundle.clone();
    let _ = one_line_bundle
        .local_rule_failure_digest
        .as_mut()
        .and_then(|digest| digest.summary_lines.pop());
    let one_line_chars = context_char_count(&one_line_bundle);
    let max_chars = one_line_chars + 16;
    assert!(full_chars > max_chars);

    let outcome = prune_bundle_to_budget(&mut bundle, max_chars);
    match outcome {
        ContextBuildOutcome::Ready { bundle, .. } => {
            assert!(
                bundle
                    .truncated_sections
                    .iter()
                    .any(|item| item == "local_rule"),
                "local_rule should be truncated when budget is tight"
            );
            let local = bundle
                .local_rule_failure_digest
                .expect("local rule digest should remain present");
            assert_eq!(local.summary_lines.len(), 1);
        }
        ContextBuildOutcome::TooLarge { reason, .. } => {
            panic!("unexpected too large: {reason}");
        }
    }
}

#[test]
fn llm_context_prunes_rule_drift_digest_when_budget_is_tight() {
    let mut bundle = FunctionContextBundleV1 {
        context_schema_version: "1".to_string(),
        function_id: "f1".to_string(),
        location: ContextLocation {
            file_path_redacted: "<CRATE_ROOT>/tests/basic.rs".to_string(),
            module_path: "tests".to_string(),
            fn_name: "case_a".to_string(),
            line_span: "1:3".to_string(),
        },
        target_function_text: "#[test]\nfn case_a() {}\n".to_string(),
        diagnostics_digest: DiagnosticsDigest {
            error_code_counts: BTreeMap::new(),
            primary_items: vec![],
        },
        constraints: ContextConstraints {
            test_only: true,
            same_function_only: true,
            signature_attrs_immutable: true,
            no_non_target_regression: true,
        },
        related_imports: vec![],
        related_type_defs: vec![],
        related_impl_blocks: vec![],
        related_fn_defs: vec![],
        neighbor_fix_digest: vec![],
        local_rule_failure_digest: None,
        rule_error_drift_digest: Some(RuleErrorDriftDigestV1 {
            original_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
            current_error_by_code: BTreeMap::from([("E0599".to_string(), 1)]),
            drift_pairs: vec![RuleErrorDriftPairV1 {
                from_code: "E0433".to_string(),
                to_code: "E0599".to_string(),
                from_count: 1,
                to_count: 1,
            }],
            failed_rule_fix_summaries: vec![
                format!(
                    "r1 rank=1 trace={} actions={}",
                    "E0433/E0433Patcher/100",
                    "x".repeat(160)
                ),
                format!(
                    "r2 rank=2 trace={} actions={}",
                    "E0433/E0433Patcher/90",
                    "y".repeat(160)
                ),
            ],
            original_target_function_text: Some(format!(
                "#[test]\nfn case_a() {{\n{}\n}}\n",
                "let _ = crate::foo::State::new();".repeat(20)
            )),
        }),
        preflight_interceptor_digest: None,
        previous_round_failure_digest: None,
        truncated_sections: vec![],
    };

    let full_chars = context_char_count(&bundle);
    let mut reduced_bundle = bundle.clone();
    if let Some(digest) = reduced_bundle.rule_error_drift_digest.as_mut() {
        let _ = digest.failed_rule_fix_summaries.pop();
    }
    let reduced_chars = context_char_count(&reduced_bundle);
    let max_chars = reduced_chars + 16;
    assert!(full_chars > max_chars);

    let outcome = prune_bundle_to_budget(&mut bundle, max_chars);
    match outcome {
        ContextBuildOutcome::Ready { bundle, .. } => {
            assert!(
                bundle
                    .truncated_sections
                    .iter()
                    .any(|item| item == "rule_drift"),
                "rule_drift should be truncated when budget is tight"
            );
            let digest = bundle
                .rule_error_drift_digest
                .expect("rule drift digest should remain present");
            assert!(digest.failed_rule_fix_summaries.len() <= 1);
        }
        ContextBuildOutcome::TooLarge { reason, .. } => {
            panic!("unexpected too large: {reason}");
        }
    }
}
