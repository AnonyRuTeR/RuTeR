use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use ruter::core::FixAction;
use ruter::core::TestFunction;

use crate::llm::preflight::build_comment_out_action;
use crate::llm::schema::{
    LlmAttemptRecord, LlmAttemptsArtifact, LlmFailureKind, NormalizedCandidateEvidence,
    sanitize_path_component, summarize_actions, truncate_for_artifact,
};
use crate::runtime::artifacts::ArtifactPaths;
use crate::runtime::function::index::FunctionIndex;
use crate::runtime::reporter::Reporter;
use crate::runtime::stages::PartialUnionVerifyResult;

use super::HISTORY_PATCH_SUMMARY_MAX_CHARS;
use super::attempt_history::push_attempt;
use super::candidate_resolution::{merge_actions_without_conflict, plan_without_function_actions};
use super::verify_port::LlmVerifyPort;

fn stable_function_identity(function_id: &str) -> String {
    if function_id == "__UNMAPPED_ERRORS__" {
        return function_id.to_string();
    }
    let mut parts = function_id.rsplitn(3, ':');
    let Some(line_end) = parts.next() else {
        return function_id.to_string();
    };
    let Some(line_start) = parts.next() else {
        return function_id.to_string();
    };
    let Some(prefix) = parts.next() else {
        return function_id.to_string();
    };
    if !line_start.chars().all(|ch| ch.is_ascii_digit())
        || !line_end.chars().all(|ch| ch.is_ascii_digit())
    {
        return function_id.to_string();
    }
    prefix.to_string()
}

fn introduced_error_by_code(
    previous: &BTreeMap<String, usize>,
    current: &BTreeMap<String, usize>,
) -> BTreeMap<String, usize> {
    let mut introduced = BTreeMap::new();
    for (code, current_count) in current {
        let previous_count = previous.get(code).copied().unwrap_or(0);
        if *current_count > previous_count {
            introduced.insert(code.clone(), current_count - previous_count);
        }
    }
    introduced
}

fn non_target_identities(
    unresolved: &BTreeSet<String>,
    ignored_identities: &BTreeSet<String>,
) -> BTreeSet<String> {
    unresolved
        .iter()
        .map(|item| stable_function_identity(item))
        .filter(|identity| !ignored_identities.contains(identity))
        .collect()
}

fn non_target_state_acceptable(
    previous_non_target: &BTreeSet<String>,
    current_non_target: &BTreeSet<String>,
    introduced_error_by_code: &BTreeMap<String, usize>,
    previous_error_total: usize,
    current_error_total: usize,
) -> bool {
    if current_non_target.is_subset(previous_non_target) {
        return true;
    }
    introduced_error_by_code.is_empty() && current_error_total <= previous_error_total
}

pub(super) fn run_preflight_comment_out_attempt(
    phase: &str,
    round: u8,
    candidate_id: &str,
    reason: &str,
    risk_flags: &[String],
    crate_path: &Path,
    root_function: &TestFunction,
    function_id: &str,
    current_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    current_partial_verify: &PartialUnionVerifyResult,
    root_index: &FunctionIndex,
    target_function_ids: &BTreeSet<String>,
    artifacts: &ArtifactPaths,
    keep_updated_sources: bool,
    verify_port: &dyn LlmVerifyPort,
    reporter: &mut Reporter,
    attempts_artifact: &mut LlmAttemptsArtifact,
) -> Result<Option<(BTreeMap<PathBuf, Vec<FixAction>>, PartialUnionVerifyResult)>> {
    let previous_unresolved = current_partial_verify
        .unresolved_function_ids
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let scope = root_index.enclosing_test_module_scope_for_function_id(function_id);
    let comment_action =
        match build_comment_out_action(crate_path, root_function, scope.as_ref(), reason) {
            Ok(value) => value,
            Err(err) => {
                push_attempt(
                    attempts_artifact,
                    LlmAttemptRecord {
                        function_id: function_id.to_string(),
                        round,
                        phase: phase.to_string(),
                        candidate_id: Some(candidate_id.to_string()),
                        accepted: false,
                        failure_kind: Some(LlmFailureKind::FunctionMappingFailed),
                        failure_detail: Some(format!(
                            "failed to materialize preflight comment action: {err:#}"
                        )),
                        previous_unresolved_function_ids: previous_unresolved.clone(),
                        unresolved_function_ids: current_partial_verify
                            .unresolved_function_ids
                            .iter()
                            .cloned()
                            .collect(),
                        check_error_total: Some(current_partial_verify.check_error_total),
                        check_error_by_code: current_partial_verify.check_error_by_code.clone(),
                        check_stdout_log: Some(current_partial_verify.check_stdout_log.clone()),
                        check_stderr_log: Some(current_partial_verify.check_stderr_log.clone()),
                        raw_response_excerpt: None,
                        prompt_excerpt: None,
                        normalized_candidate: None,
                    },
                );
                return Ok(None);
            }
        };

    let mut trial_plan =
        plan_without_function_actions(current_plan, crate_path, root_index, function_id);
    if !merge_actions_without_conflict(&mut trial_plan, std::slice::from_ref(&comment_action)) {
        push_attempt(
            attempts_artifact,
            LlmAttemptRecord {
                function_id: function_id.to_string(),
                round,
                phase: phase.to_string(),
                candidate_id: Some(candidate_id.to_string()),
                accepted: false,
                failure_kind: Some(LlmFailureKind::LlmActionConflict),
                failure_detail: Some(
                    "preflight comment action conflicts with current plan".to_string(),
                ),
                previous_unresolved_function_ids: previous_unresolved,
                unresolved_function_ids: current_partial_verify
                    .unresolved_function_ids
                    .iter()
                    .cloned()
                    .collect(),
                check_error_total: Some(current_partial_verify.check_error_total),
                check_error_by_code: current_partial_verify.check_error_by_code.clone(),
                check_stdout_log: Some(current_partial_verify.check_stdout_log.clone()),
                check_stderr_log: Some(current_partial_verify.check_stderr_log.clone()),
                raw_response_excerpt: None,
                prompt_excerpt: None,
                normalized_candidate: None,
            },
        );
        return Ok(None);
    }

    let attempt_tag = format!(
        "preflight/{}/{}",
        sanitize_path_component(function_id),
        sanitize_path_component(phase)
    );
    let trial_verify = verify_port.verify_partial_union_plan_with_tag(
        crate_path,
        &trial_plan,
        target_function_ids,
        artifacts,
        keep_updated_sources,
        &attempt_tag,
        reporter,
    )?;
    let target_identity = stable_function_identity(function_id);
    let target_cleared = !trial_verify
        .unresolved_function_ids
        .iter()
        .map(|item| stable_function_identity(item))
        .any(|identity| identity == target_identity);

    let ignored_identities: BTreeSet<String> = root_index
        .function_ids_in_same_test_module(function_id)
        .into_iter()
        .map(|item| stable_function_identity(&item))
        .collect();
    let introduced = introduced_error_by_code(
        &current_partial_verify.check_error_by_code,
        &trial_verify.check_error_by_code,
    );
    let prev_non_target = non_target_identities(
        &current_partial_verify.unresolved_function_ids,
        &ignored_identities,
    );
    let now_non_target =
        non_target_identities(&trial_verify.unresolved_function_ids, &ignored_identities);
    let no_new_non_target_regression = non_target_state_acceptable(
        &prev_non_target,
        &now_non_target,
        &introduced,
        current_partial_verify.check_error_total,
        trial_verify.check_error_total,
    );

    if target_cleared && no_new_non_target_regression {
        push_attempt(
            attempts_artifact,
            LlmAttemptRecord {
                function_id: function_id.to_string(),
                round,
                phase: phase.to_string(),
                candidate_id: Some(candidate_id.to_string()),
                accepted: true,
                failure_kind: None,
                failure_detail: Some(format!("preflight comment accepted: {reason}")),
                previous_unresolved_function_ids: current_partial_verify
                    .unresolved_function_ids
                    .iter()
                    .cloned()
                    .collect(),
                unresolved_function_ids: trial_verify
                    .unresolved_function_ids
                    .iter()
                    .cloned()
                    .collect(),
                check_error_total: Some(trial_verify.check_error_total),
                check_error_by_code: trial_verify.check_error_by_code.clone(),
                check_stdout_log: Some(trial_verify.check_stdout_log.clone()),
                check_stderr_log: Some(trial_verify.check_stderr_log.clone()),
                raw_response_excerpt: None,
                prompt_excerpt: None,
                normalized_candidate: Some(NormalizedCandidateEvidence {
                    candidate_id: candidate_id.to_string(),
                    source_kind: phase.to_string(),
                    merge_strategy: None,
                    action_count: 1,
                    action_summaries: summarize_actions(std::slice::from_ref(&comment_action)),
                    normalized_text_excerpt: None,
                    rationale: Some(truncate_for_artifact(
                        reason,
                        HISTORY_PATCH_SUMMARY_MAX_CHARS,
                    )),
                    risk_flags: risk_flags.to_vec(),
                }),
            },
        );
        return Ok(Some((trial_plan, trial_verify)));
    }

    let detail = if !target_cleared {
        "preflight comment action did not clear target function".to_string()
    } else {
        "preflight comment action introduced non-target regression".to_string()
    };
    push_attempt(
        attempts_artifact,
        LlmAttemptRecord {
            function_id: function_id.to_string(),
            round,
            phase: phase.to_string(),
            candidate_id: Some(candidate_id.to_string()),
            accepted: false,
            failure_kind: Some(LlmFailureKind::LlmVerifyFailed),
            failure_detail: Some(detail),
            previous_unresolved_function_ids: current_partial_verify
                .unresolved_function_ids
                .iter()
                .cloned()
                .collect(),
            unresolved_function_ids: trial_verify
                .unresolved_function_ids
                .iter()
                .cloned()
                .collect(),
            check_error_total: Some(trial_verify.check_error_total),
            check_error_by_code: trial_verify.check_error_by_code.clone(),
            check_stdout_log: Some(trial_verify.check_stdout_log.clone()),
            check_stderr_log: Some(trial_verify.check_stderr_log.clone()),
            raw_response_excerpt: None,
            prompt_excerpt: None,
            normalized_candidate: None,
        },
    );
    Ok(None)
}
