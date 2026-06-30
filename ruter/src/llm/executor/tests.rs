use std::collections::BTreeMap;

use ruter::core::FunctionDiagnostic;

use crate::llm::schema::{LlmAttemptRecord, LlmAttemptsArtifact, LlmFailureKind};
use crate::runtime::function::verify::FunctionVerifyRoundRecord;

use super::attempt_history::{
    build_local_rule_failure_digest_fallback, build_previous_round_failure_digest, push_attempt,
    summarize_local_rule_failure_rounds,
};

fn attempt(
    function_id: &str,
    round: u8,
    phase: &str,
    candidate_id: Option<&str>,
    failure_kind: Option<LlmFailureKind>,
    raw_excerpt: Option<&str>,
) -> LlmAttemptRecord {
    LlmAttemptRecord {
        function_id: function_id.to_string(),
        round,
        phase: phase.to_string(),
        candidate_id: candidate_id.map(ToString::to_string),
        accepted: false,
        failure_kind,
        failure_detail: Some("detail".to_string()),
        previous_unresolved_function_ids: vec![],
        unresolved_function_ids: vec![],
        check_error_total: Some(1),
        check_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
        check_stdout_log: None,
        check_stderr_log: None,
        raw_response_excerpt: raw_excerpt.map(ToString::to_string),
        prompt_excerpt: None,
        normalized_candidate: None,
    }
}

#[test]
fn llm_executor_budget_exhausted_record_keeps_phase() {
    let mut artifact = LlmAttemptsArtifact::bootstrap("replay", None, None, None, 3, 3);
    push_attempt(
        &mut artifact,
        LlmAttemptRecord {
            function_id: "f1".to_string(),
            round: 3,
            phase: "budget".to_string(),
            candidate_id: None,
            accepted: false,
            failure_kind: Some(LlmFailureKind::LlmBudgetExhausted),
            failure_detail: Some("x".to_string()),
            previous_unresolved_function_ids: vec![],
            unresolved_function_ids: vec!["f1".to_string()],
            check_error_total: Some(1),
            check_error_by_code: BTreeMap::new(),
            check_stdout_log: None,
            check_stderr_log: None,
            raw_response_excerpt: None,
            prompt_excerpt: None,
            normalized_candidate: None,
        },
    );
    assert_eq!(artifact.attempts[0].phase, "budget");
}

#[test]
fn llm_executor_builds_previous_round_digest_with_phase_priority() {
    let attempts = vec![
        attempt(
            "f1",
            1,
            "normalize",
            Some("c1"),
            Some(LlmFailureKind::LlmOutputInvalidSchema),
            Some("bad json"),
        ),
        attempt(
            "f1",
            1,
            "verify",
            Some("c1"),
            Some(LlmFailureKind::LlmVerifyFailed),
            Some("still unresolved"),
        ),
        attempt(
            "f1",
            1,
            "merge",
            Some("c2"),
            Some(LlmFailureKind::LlmActionConflict),
            Some("conflict"),
        ),
    ];

    let digest = build_previous_round_failure_digest("f1", 2, &attempts)
        .expect("digest should be generated for round 2");
    assert_eq!(digest.round, 1);
    assert!(
        digest
            .dominant_failure_kinds
            .contains(&"LLM_VERIFY_FAILED".to_string())
    );
    let c1 = digest
        .candidate_failures
        .iter()
        .find(|item| item.candidate_id == "c1")
        .expect("c1 should exist");
    assert_eq!(c1.failure_kind, "LLM_VERIFY_FAILED");
}

#[test]
fn llm_executor_extracts_introduced_error_codes_from_failure_detail() {
    let attempts = vec![LlmAttemptRecord {
        function_id: "f1".to_string(),
        round: 1,
        phase: "verify".to_string(),
        candidate_id: Some("c1".to_string()),
        accepted: false,
        failure_kind: Some(LlmFailureKind::LlmVerifyFailed),
        failure_detail: Some(
            "non-target function regression introduced; introduced_error_codes=E0433=2,E0599=1"
                .to_string(),
        ),
        previous_unresolved_function_ids: vec![],
        unresolved_function_ids: vec![],
        check_error_total: Some(1),
        check_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
        check_stdout_log: None,
        check_stderr_log: None,
        raw_response_excerpt: None,
        prompt_excerpt: None,
        normalized_candidate: None,
    }];

    let digest =
        build_previous_round_failure_digest("f1", 2, &attempts).expect("history should exist");
    let item = digest
        .candidate_failures
        .iter()
        .find(|it| it.candidate_id == "c1")
        .expect("candidate c1 should exist");
    assert_eq!(item.introduced_error_by_code.get("E0433"), Some(&2));
    assert_eq!(item.introduced_error_by_code.get("E0599"), Some(&1));
}

#[test]
fn llm_executor_builds_previous_round_digest_for_round_level_failures() {
    let attempts = vec![attempt(
        "f1",
        1,
        "request",
        None,
        Some(LlmFailureKind::LlmRequestFailed),
        None,
    )];
    let digest = build_previous_round_failure_digest("f1", 2, &attempts)
        .expect("digest should include request-level failure");
    assert_eq!(digest.candidate_failures.len(), 1);
    assert!(
        digest.candidate_failures[0]
            .candidate_id
            .starts_with("round_event_")
    );
    assert_eq!(
        digest.candidate_failures[0].failure_kind,
        "LLM_REQUEST_FAILED"
    );
    assert_eq!(digest.candidate_failures[0].patch_summary, "phase=request");
}

fn round_record(
    round: usize,
    unresolved_function_ids: Vec<&str>,
    selected_rank_by_function: Vec<(&str, usize)>,
    independence_broken_function_ids: Vec<&str>,
) -> FunctionVerifyRoundRecord {
    FunctionVerifyRoundRecord {
        round,
        selected_rank_by_function: selected_rank_by_function
            .into_iter()
            .map(|(fid, rank)| (fid.to_string(), rank))
            .collect(),
        plan_file_count: 1,
        plan_action_count: 1,
        check_error_total: 1,
        check_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
        resolved_function_ids: vec![],
        unresolved_function_ids: unresolved_function_ids
            .into_iter()
            .map(ToString::to_string)
            .collect(),
        independence_broken_function_ids: independence_broken_function_ids
            .into_iter()
            .map(ToString::to_string)
            .collect(),
    }
}

#[test]
fn llm_executor_summarizes_local_rule_failure_from_rounds() {
    let rounds = vec![
        round_record(1, vec!["f1"], vec![("f1", 1)], vec![]),
        round_record(2, vec!["f1"], vec![("f1", 2)], vec!["f1"]),
    ];
    let map = summarize_local_rule_failure_rounds(&rounds);
    let digest = map.get("f1").expect("f1 digest should exist");
    assert!(
        digest
            .summary_lines
            .iter()
            .any(|line| line.contains("rule_verify_rounds=2"))
    );
    assert!(
        digest
            .summary_lines
            .iter()
            .any(|line| line.contains("selected_rank_history=r1:1,r2:2"))
    );
    assert!(
        digest
            .summary_lines
            .iter()
            .any(|line| line.contains("independence_broken_rounds=2"))
    );
}

#[test]
fn llm_executor_builds_local_rule_failure_fallback_from_partial_verify() {
    let mut error_diagnostics_by_function = BTreeMap::new();
    error_diagnostics_by_function.insert(
        "f1".to_string(),
        vec![FunctionDiagnostic {
            code: "E0433".to_string(),
            message: "failed to resolve".to_string(),
            primary_span: None,
            label: None,
            suggested_replacement: None,
            children_note_messages: vec![],
            children_help_messages: vec![],
            children_suggested_replacements: vec![],
        }],
    );

    let digest = build_local_rule_failure_digest_fallback("f1", &error_diagnostics_by_function)
        .expect("fallback digest should exist");
    assert!(
        digest
            .summary_lines
            .iter()
            .any(|line| line.contains("local_rule_handoff_reason=unresolved_after_rule_verify"))
    );
    assert!(
        digest
            .summary_lines
            .iter()
            .any(|line| line.contains("remaining_error_by_code=E0433=1"))
    );
}
