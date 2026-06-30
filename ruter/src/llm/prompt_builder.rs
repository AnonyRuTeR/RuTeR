use serde::{Deserialize, Serialize};

use crate::llm::context_builder::{
    FunctionContextBundleV1, LocalRuleFailureDigestV1, PreflightInterceptorDigestV1,
    PreviousCandidateFailureDigestV1, PreviousRoundFailureDigestV1, PrimaryDiagnosticItem,
    RuleErrorDriftDigestV1, RuleErrorDriftPairV1,
};

pub const PROMPT_CONTRACT_VERSION: &str = "2";

/// Prompt artifact for one function-round attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionPromptArtifactV1 {
    pub prompt_contract_version: String,
    pub function_id: String,
    pub round: u8,
    pub system_prompt: String,
    pub user_prompt: String,
}

/// Build system+user prompt from context bundle.
pub fn build_prompt_artifact(
    bundle: &FunctionContextBundleV1,
    round: u8,
) -> FunctionPromptArtifactV1 {
    let system_prompt = [
        "You are a Rust unit-test repair assistant.",
        "Return exactly one fenced Rust code block that contains the full patched target function item.",
        "Do not return JSON and do not add explanations.",
        "Keep function signature and attributes unchanged.",
        "Patch only within the same test module; function body edits are preferred.",
        "Do not edit non-target test modules and do not change production code.",
        "Avoid repeating equivalent fixes that already failed in the previous round.",
    ]
    .join("\n");

    let diagnostics = render_primary_items(&bundle.diagnostics_digest.primary_items);
    let imports = render_list_or_none(&bundle.related_imports);
    let type_defs = render_list_or_none(&bundle.related_type_defs);
    let impl_blocks = render_list_or_none(&bundle.related_impl_blocks);
    let fn_defs = render_multiline_code_items_or_none(&bundle.related_fn_defs);
    let numbered_source =
        render_numbered_source(&bundle.target_function_text, &bundle.location.line_span);
    let previous_round_failures = render_previous_round_failures_to_avoid(
        bundle.previous_round_failure_digest.as_ref(),
        round,
    );
    let local_rule_failures = render_local_rule_failures(bundle.local_rule_failure_digest.as_ref());
    let rule_drift_hints = render_rule_drift_hints(
        bundle.rule_error_drift_digest.as_ref(),
        &bundle.location.line_span,
    );
    let preflight_notes = render_preflight_notes(bundle.preflight_interceptor_digest.as_ref());
    let environment_constraints =
        render_environment_constraints(bundle.preflight_interceptor_digest.as_ref());

    let user_prompt = format!(
        concat!(
            "Task: Fix unresolved compiler errors in this Rust test function.\n",
            "Location: {file}, module={module}, fn={fn_name}\n",
            "Target function source:\n{source}\n",
            "Compiler diagnostics:\n",
            "- error_code_counts: {error_counts}\n",
            "- primary diagnostics:\n{diagnostics}\n",
            "Local rule patcher failure summary:\n{local_rule_failures}\n",
            "Rule patch drift hints:\n{rule_drift_hints}\n",
            "Pre-flight interceptor notes:\n{preflight_notes}\n",
            "Environment constraints:\n{environment_constraints}\n",
            "Related context:\n",
            "- imports: {imports}\n",
            "- related free function defs:\n{fn_defs}\n",
            "- related type defs: {type_defs}\n",
            "- related impl blocks: {impl_blocks}\n",
            "Previous round failures to avoid repeating:\n{previous_round_failures}\n",
            "Constraints:\n",
            "- patch test code only\n",
            "- modify same test module scope only\n",
            "- keep signature and attributes unchanged\n",
            "- no unresolved regression on non-target functions\n"
        ),
        file = bundle.location.file_path_redacted,
        module = if bundle.location.module_path.is_empty() {
            "<root>"
        } else {
            &bundle.location.module_path
        },
        fn_name = bundle.location.fn_name,
        source = numbered_source,
        error_counts = render_error_counts(&bundle.diagnostics_digest.error_code_counts),
        diagnostics = diagnostics,
        local_rule_failures = local_rule_failures,
        rule_drift_hints = rule_drift_hints,
        preflight_notes = preflight_notes,
        environment_constraints = environment_constraints,
        imports = imports,
        fn_defs = fn_defs,
        type_defs = type_defs,
        impl_blocks = impl_blocks,
        previous_round_failures = previous_round_failures,
    );

    FunctionPromptArtifactV1 {
        prompt_contract_version: PROMPT_CONTRACT_VERSION.to_string(),
        function_id: bundle.function_id.clone(),
        round,
        system_prompt,
        user_prompt,
    }
}

fn render_error_counts(counts: &std::collections::BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(code, cnt)| format!("{code}={cnt}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_primary_items(items: &[PrimaryDiagnosticItem]) -> String {
    if items.is_empty() {
        return "  (none)".to_string();
    }

    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            format!(
                "  {}) {} @ {} | label={} | suggestion={}",
                idx + 1,
                item.message,
                item.primary_span
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                item.label.clone().unwrap_or_else(|| "none".to_string()),
                item.suggested_replacement
                    .clone()
                    .unwrap_or_else(|| "none".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    items.join(" | ")
}

fn render_multiline_code_items_or_none(items: &[String]) -> String {
    if items.is_empty() {
        return "  (none)".to_string();
    }
    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let block = item
                .lines()
                .map(|line| format!("    {line}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("  [{}]\n{block}", idx + 1)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_previous_round_failures_to_avoid(
    digest: Option<&PreviousRoundFailureDigestV1>,
    round: u8,
) -> String {
    if round <= 1 {
        return "  (none)".to_string();
    }
    let Some(digest) = digest else {
        return "  (none)".to_string();
    };
    if digest.candidate_failures.is_empty() {
        return "  (none)".to_string();
    }

    let kinds = if digest.dominant_failure_kinds.is_empty() {
        "none".to_string()
    } else {
        digest.dominant_failure_kinds.join(", ")
    };

    let mut lines = vec![format!(
        "  round={} dominant_failure_kinds={kinds}",
        digest.round
    )];
    for (idx, item) in digest.candidate_failures.iter().enumerate() {
        lines.push(render_previous_candidate_failure(idx + 1, item));
    }
    let introduced_e0433 = digest.candidate_failures.iter().any(|item| {
        item.introduced_error_by_code
            .get("E0433")
            .copied()
            .unwrap_or(0)
            > 0
    });
    if introduced_e0433 {
        lines.push(
            "  priority_guard=previous candidate introduced E0433; do not add new unresolved symbols or paths".to_string(),
        );
    }
    lines.join("\n")
}

fn render_local_rule_failures(digest: Option<&LocalRuleFailureDigestV1>) -> String {
    let Some(digest) = digest else {
        return "  (none)".to_string();
    };
    if digest.summary_lines.is_empty() {
        return "  (none)".to_string();
    }

    digest
        .summary_lines
        .iter()
        .enumerate()
        .map(|(idx, line)| format!("  [{}] {line}", idx + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_preflight_notes(digest: Option<&PreflightInterceptorDigestV1>) -> String {
    let Some(digest) = digest else {
        return "  (none)".to_string();
    };
    if digest.notes.is_empty() {
        return "  (none)".to_string();
    }
    digest
        .notes
        .iter()
        .enumerate()
        .map(|(idx, line)| format!("  [{}] {line}", idx + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_rule_drift_hints(digest: Option<&RuleErrorDriftDigestV1>, line_span: &str) -> String {
    let Some(digest) = digest else {
        return "  (none)".to_string();
    };

    let original = render_error_counts(&digest.original_error_by_code);
    let current = render_error_counts(&digest.current_error_by_code);
    let drift_pairs = render_drift_pairs(&digest.drift_pairs);
    let mut lines = vec![
        format!("  original_error_by_code={original}"),
        format!("  current_error_by_code={current}"),
        format!("  drift_pairs={drift_pairs}"),
        "  avoid_repeating_failed_rule_fixes=true; prioritize resolving newly introduced code roots".to_string(),
    ];

    if digest.failed_rule_fix_summaries.is_empty() {
        lines.push("  failed_rule_fix_summaries=(none)".to_string());
    } else {
        lines.push("  failed_rule_fix_summaries:".to_string());
        for (idx, line) in digest.failed_rule_fix_summaries.iter().enumerate() {
            lines.push(format!("    [{}] {line}", idx + 1));
        }
    }

    if let Some(source) = digest.original_target_function_text.as_deref() {
        lines.push("  original target function source (before rule patch):".to_string());
        lines.push(render_numbered_source(source, line_span));
    } else {
        lines.push("  original target function source (before rule patch): (none)".to_string());
    }

    lines.join("\n")
}

fn render_drift_pairs(pairs: &[RuleErrorDriftPairV1]) -> String {
    if pairs.is_empty() {
        return "none".to_string();
    }
    pairs
        .iter()
        .map(|item| {
            format!(
                "{}->{}({}->{})",
                item.from_code, item.to_code, item.from_count, item.to_count
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_environment_constraints(digest: Option<&PreflightInterceptorDigestV1>) -> String {
    let Some(digest) = digest else {
        return "  (none)".to_string();
    };
    let is_no_std = digest
        .notes
        .iter()
        .any(|line| line.trim().eq_ignore_ascii_case("CRATE_ENV_NO_STD=true"));
    let has_primitive_to_string_guard = digest
        .notes
        .iter()
        .any(|line| line.starts_with("E0599_NO_STD_PRIMITIVE_TOSTRING=true"));
    if is_no_std {
        let mut lines = vec![
            "  [1] This crate is #![no_std]; avoid std-only assumptions (for example primitive `.to_string()` in tests).".to_string(),
            "  [2] Prefer existing imports and crate-local paths already in scope; do not introduce new unresolved symbols.".to_string(),
        ];
        if has_primitive_to_string_guard {
            lines.push(
                "  [3] This function currently fails on primitive `.to_string()`; your patch must not call `.to_string()`.".to_string(),
            );
        }
        return lines.join("\n");
    }
    "  (none)".to_string()
}

fn render_previous_candidate_failure(
    idx: usize,
    item: &PreviousCandidateFailureDigestV1,
) -> String {
    let detail = item.failure_detail.as_deref().unwrap_or("none");
    let unresolved = if item.unresolved_error_by_code.is_empty() {
        "none".to_string()
    } else {
        item.unresolved_error_by_code
            .iter()
            .map(|(code, count)| format!("{code}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let introduced = if item.introduced_error_by_code.is_empty() {
        "none".to_string()
    } else {
        item.introduced_error_by_code
            .iter()
            .map(|(code, count)| format!("{code}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        "  [{}] candidate={} | failure_kind={} | patch_summary={}\n      failure_detail={}\n      unresolved_error_by_code={}\n      introduced_error_by_code={}",
        idx,
        item.candidate_id,
        item.failure_kind,
        item.patch_summary,
        detail,
        unresolved,
        introduced
    )
}

fn render_numbered_source(source: &str, line_span: &str) -> String {
    let line_start = line_span
        .split(':')
        .next()
        .and_then(|raw| raw.parse::<usize>().ok())
        .unwrap_or(1);
    source
        .lines()
        .enumerate()
        .map(|(idx, line)| format!("{:>4} | {line}", line_start + idx))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::context_builder::{
        ContextConstraints, ContextLocation, DiagnosticsDigest, FunctionContextBundleV1,
        LocalRuleFailureDigestV1, PreflightInterceptorDigestV1, PreviousCandidateFailureDigestV1,
        PreviousRoundFailureDigestV1, PrimaryDiagnosticItem, RuleErrorDriftDigestV1,
        RuleErrorDriftPairV1,
    };
    use std::collections::BTreeMap;

    #[test]
    fn llm_prompt_uses_trimmed_user_prompt_and_rust_codeblock_contract() {
        let mut code_counts = BTreeMap::new();
        code_counts.insert("E0433".to_string(), 2);
        let bundle = FunctionContextBundleV1 {
            context_schema_version: "1".to_string(),
            function_id: "tests/basic.rs::mod_tests::case_a:12:30".to_string(),
            location: ContextLocation {
                file_path_redacted: "<CRATE_ROOT>/tests/basic.rs".to_string(),
                module_path: "mod_tests".to_string(),
                fn_name: "case_a".to_string(),
                line_span: "12:30".to_string(),
            },
            target_function_text: "#[test]\nfn case_a() { let _ = State::new(); }\n".to_string(),
            diagnostics_digest: DiagnosticsDigest {
                error_code_counts: code_counts,
                primary_items: vec![PrimaryDiagnosticItem {
                    code: "E0433".to_string(),
                    message: "failed to resolve".to_string(),
                    primary_span: Some("tests/basic.rs:16:13-16:30".to_string()),
                    label: Some("unresolved module".to_string()),
                    suggested_replacement: Some("crate::foo::State".to_string()),
                }],
            },
            constraints: ContextConstraints {
                test_only: true,
                same_function_only: true,
                signature_attrs_immutable: true,
                no_non_target_regression: true,
            },
            related_imports: vec!["use crate::foo::State;".to_string()],
            related_type_defs: vec![],
            related_impl_blocks: vec![],
            related_fn_defs: vec!["fn helper() {}".to_string()],
            neighbor_fix_digest: vec!["applied_actions_in_same_file=2".to_string()],
            local_rule_failure_digest: Some(LocalRuleFailureDigestV1 {
                summary_lines: vec![
                    "rule_verify_rounds=2".to_string(),
                    "selected_rank_history=r1:1,r2:2".to_string(),
                ],
            }),
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
                    "r1 rank=1 trace=E0433/E0433Patcher/10 actions=replace@10-10".to_string(),
                ],
                original_target_function_text: Some(
                    "#[test]\nfn case_a() { let _ = State::new(); }\n".to_string(),
                ),
            }),
            preflight_interceptor_digest: Some(PreflightInterceptorDigestV1 {
                notes: vec![
                    "CRATE_ENV_NO_STD=true".to_string(),
                    "LOW_VALUE_STATUS=HAS_TEST_SEMANTICS".to_string(),
                    "E0599_CLASSIFICATION=PARTIAL_MISSING_METHOD".to_string(),
                    "E0308_EXPECTED=Option<String>".to_string(),
                    "E0308_FOUND=String".to_string(),
                ],
            }),
            previous_round_failure_digest: Some(PreviousRoundFailureDigestV1 {
                round: 1,
                dominant_failure_kinds: vec!["LLM_VERIFY_FAILED".to_string()],
                candidate_failures: vec![PreviousCandidateFailureDigestV1 {
                    candidate_id: "c1".to_string(),
                    patch_summary: "replace path with crate::State".to_string(),
                    failure_kind: "LLM_VERIFY_FAILED".to_string(),
                    failure_detail: Some("target function still unresolved".to_string()),
                    unresolved_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
                    introduced_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
                }],
            }),
            truncated_sections: vec![],
        };

        let prompt = build_prompt_artifact(&bundle, 2);
        assert_eq!(prompt.prompt_contract_version, "2");
        assert!(prompt.system_prompt.contains("fenced Rust code block"));
        assert!(
            prompt
                .system_prompt
                .contains("Avoid repeating equivalent fixes")
        );
        assert!(prompt.user_prompt.contains("Target function source"));
        assert!(prompt.user_prompt.contains("related free function defs"));
        assert!(
            prompt
                .user_prompt
                .contains("Previous round failures to avoid repeating")
        );
        assert!(
            prompt
                .user_prompt
                .contains("Local rule patcher failure summary")
        );
        assert!(prompt.user_prompt.contains("Rule patch drift hints:"));
        assert!(
            prompt
                .user_prompt
                .contains("drift_pairs=E0433->E0599(1->1)")
        );
        assert!(
            prompt
                .user_prompt
                .contains("original target function source (before rule patch)")
        );
        assert!(
            prompt
                .user_prompt
                .contains("r1 rank=1 trace=E0433/E0433Patcher/10")
        );
        assert!(prompt.user_prompt.contains("Pre-flight interceptor notes"));
        assert!(
            prompt
                .user_prompt
                .contains("E0599_CLASSIFICATION=PARTIAL_MISSING_METHOD")
        );
        assert!(prompt.user_prompt.contains("E0308_EXPECTED=Option<String>"));
        assert!(prompt.user_prompt.contains("E0308_FOUND=String"));
        assert!(prompt.user_prompt.contains("rule_verify_rounds=2"));
        assert!(prompt.user_prompt.contains("candidate=c1"));
        assert!(
            prompt
                .user_prompt
                .contains("introduced_error_by_code=E0433=1")
        );
        assert!(prompt.user_prompt.contains("Environment constraints:"));
        assert!(prompt.user_prompt.contains("#![no_std]"));
        assert!(prompt.user_prompt.contains("  12 | #[test]"));
        assert!(prompt.user_prompt.contains("  [1]"));
        assert!(prompt.user_prompt.contains("    fn helper() {}"));
        assert!(!prompt.user_prompt.contains("Round:"));
        assert!(!prompt.user_prompt.contains("Target: function_id"));
        assert!(!prompt.user_prompt.contains("Output contract"));
        assert!(!prompt.user_prompt.contains("Acceptance:"));
    }

    #[test]
    fn llm_prompt_round1_uses_none_for_previous_failures() {
        let bundle = FunctionContextBundleV1 {
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
                round: 0,
                dominant_failure_kinds: vec!["LLM_REQUEST_FAILED".to_string()],
                candidate_failures: vec![PreviousCandidateFailureDigestV1 {
                    candidate_id: "round_event_request".to_string(),
                    patch_summary: "phase=request".to_string(),
                    failure_kind: "LLM_REQUEST_FAILED".to_string(),
                    failure_detail: Some("timeout".to_string()),
                    unresolved_error_by_code: BTreeMap::new(),
                    introduced_error_by_code: BTreeMap::new(),
                }],
            }),
            truncated_sections: vec![],
        };

        let prompt = build_prompt_artifact(&bundle, 1);
        assert!(
            prompt
                .user_prompt
                .contains("Local rule patcher failure summary:\n  (none)")
        );
        assert!(
            prompt
                .user_prompt
                .contains("Rule patch drift hints:\n  (none)")
        );
        assert!(
            prompt
                .user_prompt
                .contains("Previous round failures to avoid repeating:\n  (none)")
        );
    }
}
