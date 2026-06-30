use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ruter::core::{FixAction, FunctionDiagnostic, TestFunction};

use crate::config::{LlmMode, ResolvedLlmConfig};
use crate::llm::client::OnlineLlmClient;
use crate::llm::context_builder::{
    ContextBuildConfig, ContextBuildOutcome, RuleErrorDriftDigestV1, RuleErrorDriftPairV1,
    build_context_bundle_v1,
};
use crate::llm::io_debug::LlmIoDebugRecord;
use crate::llm::preflight::{
    PreflightAnalyses, PreflightDecision, analyze_preflight_interceptors,
    build_preflight_interceptor_digest, decide_preflight, should_try_budget_e0308_fallback,
};
use crate::llm::prompt_builder::build_prompt_artifact;
use crate::llm::schema::{
    LlmAttemptRecord, LlmAttemptsArtifact, LlmFailureKind, LlmReplayCandidate, LlmReplayFile,
    sanitize_path_component, truncate_for_artifact,
};
use crate::runtime::artifacts::{ArtifactPaths, write_json};
use crate::runtime::function::index::FunctionIndex;
use crate::runtime::function::verify::FunctionVerifyRoundRecord;
use crate::runtime::reporter::Reporter;
use crate::runtime::stages::{LlmReplayStageResult, PartialUnionVerifyResult};

use super::attempt_history::{
    build_local_rule_failure_digest_fallback, build_previous_round_failure_digest,
    load_local_rule_failure_digest_by_function, push_attempt, render_error_counts,
};
use super::candidate_resolution::{
    current_source_for_function, merge_actions_without_conflict, plan_without_function_actions,
    resolve_candidate_actions,
};
use super::preflight_flow::run_preflight_comment_out_attempt;
use super::verify_port::LlmVerifyPort;
use super::{HISTORY_FAILURE_DETAIL_MAX_CHARS, LlmContextRecord, OnlinePromptState};

#[derive(Debug, Clone, Default)]
struct RuleDriftSnapshot {
    original_error_by_code: BTreeMap<String, usize>,
    selected_rule_fix_summaries: Vec<String>,
    attempted_rule_rounds: usize,
}

fn current_error_by_code(diagnostics: &[FunctionDiagnostic]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for diag in diagnostics {
        *counts.entry(diag.code.clone()).or_insert(0) += 1;
    }
    counts
}

fn detect_rule_error_drift_pairs(
    original: &BTreeMap<String, usize>,
    current: &BTreeMap<String, usize>,
) -> Vec<RuleErrorDriftPairV1> {
    let disappeared = original
        .iter()
        .filter(|(code, from_count)| {
            **from_count > 0 && current.get(*code).copied().unwrap_or(0) == 0
        })
        .map(|(code, count)| (code.clone(), *count))
        .collect::<Vec<_>>();
    let introduced = current
        .iter()
        .filter(|(code, to_count)| **to_count > 0 && original.get(*code).copied().unwrap_or(0) == 0)
        .map(|(code, count)| (code.clone(), *count))
        .collect::<Vec<_>>();

    if disappeared.is_empty() || introduced.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    for (from_code, from_count) in &disappeared {
        for (to_code, to_count) in &introduced {
            out.push(RuleErrorDriftPairV1 {
                from_code: from_code.clone(),
                to_code: to_code.clone(),
                from_count: *from_count,
                to_count: *to_count,
            });
        }
    }
    out
}

fn summarize_fix_action(action: &FixAction) -> String {
    match action {
        FixAction::Insert { span, .. } => {
            format!(
                "insert@{}:{}-{}",
                span.file_path.display(),
                span.line_start,
                span.line_end
            )
        }
        FixAction::Replace { span, .. } => {
            format!(
                "replace@{}:{}-{}",
                span.file_path.display(),
                span.line_start,
                span.line_end
            )
        }
        FixAction::Delete { span } => {
            format!(
                "delete@{}:{}-{}",
                span.file_path.display(),
                span.line_start,
                span.line_end
            )
        }
    }
}

fn summarize_rule_candidate(
    candidate: &crate::runtime::function::rule_plan::FunctionRuleCandidate,
) -> String {
    let trace = if candidate.diagnostic_trace.is_empty() {
        "none".to_string()
    } else {
        candidate
            .diagnostic_trace
            .iter()
            .map(|item| format!("{}/{}/{}", item.code, item.patcher, item.candidate_score))
            .collect::<Vec<_>>()
            .join(";")
    };
    let actions = if candidate.actions.is_empty() {
        "none".to_string()
    } else {
        candidate
            .actions
            .iter()
            .take(3)
            .map(summarize_fix_action)
            .collect::<Vec<_>>()
            .join(" | ")
    };
    format!("trace={trace} actions={actions}")
}

fn load_rule_drift_snapshot_by_function(
    artifacts: &ArtifactPaths,
) -> BTreeMap<String, RuleDriftSnapshot> {
    let mut snapshots = BTreeMap::<String, RuleDriftSnapshot>::new();

    let dispatch_items = std::fs::read_to_string(&artifacts.function_dispatch_report_json)
        .ok()
        .and_then(|raw| {
            serde_json::from_str::<
                Vec<crate::runtime::function::dispatch::FunctionDispatchReportItem>,
            >(&raw)
            .ok()
        })
        .unwrap_or_default();
    for item in dispatch_items {
        if item.decision != "RulePatcher" {
            continue;
        }
        snapshots
            .entry(item.function_id)
            .or_default()
            .original_error_by_code = item.error_code_counts;
    }

    let candidates_by_function = std::fs::read_to_string(&artifacts.function_rule_candidates_json)
        .ok()
        .and_then(|raw| {
            serde_json::from_str::<
                Vec<crate::runtime::function::rule_plan::FunctionRuleCandidatesArtifact>,
            >(&raw)
            .ok()
        })
        .unwrap_or_default()
        .into_iter()
        .map(|item| (item.function_id, item.candidates))
        .collect::<BTreeMap<_, _>>();
    let rounds = std::fs::read_to_string(&artifacts.function_verify_rounds_json)
        .ok()
        .and_then(|raw| serde_json::from_str::<Vec<FunctionVerifyRoundRecord>>(&raw).ok())
        .unwrap_or_default();

    for round in rounds {
        for (function_id, rank) in round.selected_rank_by_function {
            let summary = candidates_by_function
                .get(&function_id)
                .and_then(|candidates| candidates.iter().find(|candidate| candidate.rank == rank))
                .map(summarize_rule_candidate)
                .unwrap_or_else(|| format!("selected_rank={rank}"));
            let snapshot = snapshots.entry(function_id).or_default();
            snapshot.attempted_rule_rounds += 1;
            snapshot
                .selected_rule_fix_summaries
                .push(format!("r{} rank={} {}", round.round, rank, summary));
        }
    }

    snapshots
}

fn build_rule_error_drift_digest(
    snapshot: Option<&RuleDriftSnapshot>,
    current_error_by_code: &BTreeMap<String, usize>,
    original_target_function_text: Option<String>,
) -> Option<RuleErrorDriftDigestV1> {
    let snapshot = snapshot?;
    if snapshot.attempted_rule_rounds == 0 || snapshot.original_error_by_code.is_empty() {
        return None;
    }

    let drift_pairs =
        detect_rule_error_drift_pairs(&snapshot.original_error_by_code, current_error_by_code);
    if drift_pairs.is_empty() {
        return None;
    }

    Some(RuleErrorDriftDigestV1 {
        original_error_by_code: snapshot.original_error_by_code.clone(),
        current_error_by_code: current_error_by_code.clone(),
        drift_pairs,
        failed_rule_fix_summaries: snapshot.selected_rule_fix_summaries.clone(),
        original_target_function_text,
    })
}

fn read_original_target_function_text(function: &TestFunction) -> Option<String> {
    let source = std::fs::read_to_string(&function.file_path).ok()?;
    let start = function.byte_start.min(source.len());
    let end = function.byte_end.min(source.len());
    if start >= end {
        return None;
    }
    Some(source[start..end].to_string())
}

fn unresolved_snapshot(partial_verify: &PartialUnionVerifyResult) -> Vec<String> {
    partial_verify
        .unresolved_function_ids
        .iter()
        .cloned()
        .collect()
}

fn push_mapping_failure_attempt(
    attempts_artifact: &mut LlmAttemptsArtifact,
    partial_verify: &PartialUnionVerifyResult,
    function_id: &str,
    failure_detail: &str,
) {
    let unresolved = unresolved_snapshot(partial_verify);
    push_attempt(
        attempts_artifact,
        LlmAttemptRecord {
            function_id: function_id.to_string(),
            round: 1,
            phase: "mapping".to_string(),
            candidate_id: None,
            accepted: false,
            failure_kind: Some(LlmFailureKind::FunctionMappingFailed),
            failure_detail: Some(failure_detail.to_string()),
            previous_unresolved_function_ids: unresolved.clone(),
            unresolved_function_ids: unresolved,
            check_error_total: Some(partial_verify.check_error_total),
            check_error_by_code: partial_verify.check_error_by_code.clone(),
            check_stdout_log: Some(partial_verify.check_stdout_log.clone()),
            check_stderr_log: Some(partial_verify.check_stderr_log.clone()),
            raw_response_excerpt: None,
            prompt_excerpt: None,
            normalized_candidate: None,
        },
    );
}

fn push_preflight_attempt(
    attempts_artifact: &mut LlmAttemptsArtifact,
    partial_verify: &PartialUnionVerifyResult,
    function_id: &str,
    phase: &str,
    detail: String,
) {
    let unresolved = unresolved_snapshot(partial_verify);
    push_attempt(
        attempts_artifact,
        LlmAttemptRecord {
            function_id: function_id.to_string(),
            round: 1,
            phase: phase.to_string(),
            candidate_id: None,
            accepted: true,
            failure_kind: None,
            failure_detail: Some(detail),
            previous_unresolved_function_ids: unresolved.clone(),
            unresolved_function_ids: unresolved,
            check_error_total: Some(partial_verify.check_error_total),
            check_error_by_code: partial_verify.check_error_by_code.clone(),
            check_stdout_log: Some(partial_verify.check_stdout_log.clone()),
            check_stderr_log: Some(partial_verify.check_stderr_log.clone()),
            raw_response_excerpt: None,
            prompt_excerpt: None,
            normalized_candidate: None,
        },
    );
}

fn render_preflight_decision_detail(decision: &PreflightDecision) -> String {
    match decision {
        PreflightDecision::ContinueToLlm { reason } => reason.clone(),
        PreflightDecision::ContinueToLlmWithHints { reason, hints } => format!(
            "{}; hints={}",
            reason,
            hints
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        ),
        PreflightDecision::SkipLlmCommentOut { reason, risk_flags } => {
            format!("{}; risk_flags={}", reason, risk_flags.join(","))
        }
    }
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

fn non_target_unresolved_stable_id_set(
    unresolved: &BTreeSet<String>,
    ignored_identities: &BTreeSet<String>,
) -> BTreeSet<String> {
    unresolved
        .iter()
        .map(|value| stable_function_identity(value))
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

fn is_primitive_type_name(type_name: &str) -> bool {
    matches!(
        type_name,
        "bool"
            | "char"
            | "str"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
    )
}

fn no_std_primitive_to_string_guard_enabled(analyses: &PreflightAnalyses) -> bool {
    if !analyses.crate_env_no_std {
        return false;
    }
    let Some(analysis) = analyses.e0599.as_ref() else {
        return false;
    };
    let Some(target) = analysis.target.as_ref() else {
        return false;
    };
    target.method_name == "to_string"
        && target.raw_kind == "type"
        && is_primitive_type_name(target.normalized_type_name.as_str())
}

fn merge_e0599_related_free_function_defs(
    related_fn_defs: &mut Vec<String>,
    analysis: Option<&ruter::patchers::e0599::E0599Analysis>,
) {
    let Some(e0599) = analysis else {
        return;
    };
    if e0599.related_free_function_signatures.is_empty() {
        return;
    }
    related_fn_defs.extend(e0599.related_free_function_signatures.iter().cloned());
    related_fn_defs.sort();
    related_fn_defs.dedup();
}

fn candidate_uses_to_string(candidate: &LlmReplayCandidate) -> bool {
    candidate
        .patched_function_text
        .as_deref()
        .map(|text| text.contains(".to_string(") || text.contains("to_string()"))
        .unwrap_or(false)
}

pub(super) fn run_llm_stage_with_port(
    crate_path: &Path,
    artifacts: &ArtifactPaths,
    target_function_ids: &BTreeSet<String>,
    initial_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    initial_partial_verify: PartialUnionVerifyResult,
    cfg: &ResolvedLlmConfig,
    verify_port: &dyn LlmVerifyPort,
    reporter: &mut Reporter,
) -> Result<LlmReplayStageResult> {
    let replay = if matches!(cfg.mode, LlmMode::Replay) {
        let replay_path = cfg
            .replay_file
            .as_ref()
            .context("llm replay mode requires replay file")?;
        Some(LlmReplayFile::read_from_path(replay_path)?)
    } else {
        None
    };

    let online_client = if matches!(cfg.mode, LlmMode::Online) {
        let client = OnlineLlmClient::new(crate::llm::client::OnlineLlmClientConfig {
            api_url: cfg
                .api_url
                .clone()
                .context("llm online mode requires api_url")?,
            model: cfg
                .model
                .clone()
                .context("llm online mode requires model")?,
            api_key: cfg
                .api_key
                .clone()
                .context("llm online mode requires api_key")?,
            timeout_secs: cfg.timeout_secs,
            output_token_ratio: cfg.output_token_ratio,
        })?;
        Some(client)
    } else {
        None
    };

    let mode_name = match cfg.mode {
        LlmMode::Replay => "replay",
        LlmMode::Online => "online",
    };

    let mut attempts_artifact = LlmAttemptsArtifact::bootstrap(
        mode_name,
        cfg.replay_file.clone(),
        cfg.api_url.clone(),
        cfg.model.clone(),
        cfg.max_rounds,
        cfg.max_candidates_per_round,
    );

    let root_index = FunctionIndex::build(crate_path)?;
    let mut current_plan = initial_plan.clone();
    let mut current_partial_verify = initial_partial_verify;

    let mut unresolved_sorted = current_partial_verify
        .unresolved_function_ids
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    unresolved_sorted.sort();

    reporter.section("llm stage");
    reporter.kv(0, "mode", mode_name);
    reporter.kv(0, "max_rounds", cfg.max_rounds.to_string());
    reporter.kv(
        0,
        "max_candidates_per_round",
        cfg.max_candidates_per_round.to_string(),
    );
    reporter.kv(0, "output_token_ratio", cfg.output_token_ratio.to_string());
    reporter.kv(0, "unresolved_count", unresolved_sorted.len().to_string());
    reporter.kv(0, "debug_dump_full_io", cfg.debug_dump_full_io.to_string());

    let mut context_records = Vec::new();
    let mut io_debug_records = Vec::new();
    let local_rule_failure_digest_by_function =
        load_local_rule_failure_digest_by_function(&artifacts.function_verify_rounds_json);
    let rule_drift_snapshot_by_function = load_rule_drift_snapshot_by_function(artifacts);
    let mut preflight_skipped_llm_count = 0usize;
    let mut preflight_skipped_llm_by_code = BTreeMap::<String, usize>::new();

    for function_id in unresolved_sorted {
        if function_id == "__UNMAPPED_ERRORS__" {
            push_mapping_failure_attempt(
                &mut attempts_artifact,
                &current_partial_verify,
                &function_id,
                "unmapped diagnostics cannot be routed to one test function",
            );
            continue;
        }

        let Some(root_function) = root_index.get(&function_id).cloned() else {
            push_mapping_failure_attempt(
                &mut attempts_artifact,
                &current_partial_verify,
                &function_id,
                "function_id not found in root index",
            );
            continue;
        };

        let function_diags = current_partial_verify
            .error_diagnostics_by_function
            .get(&function_id)
            .cloned()
            .unwrap_or_default();
        let preflight = analyze_preflight_interceptors(crate_path, &root_function, &function_diags);
        let preflight_interceptor_digest = build_preflight_interceptor_digest(&preflight);

        if let Some(analysis) = preflight.e0599.as_ref() {
            push_preflight_attempt(
                &mut attempts_artifact,
                &current_partial_verify,
                &function_id,
                "preflight_e0599",
                truncate_for_artifact(&analysis.summary, HISTORY_FAILURE_DETAIL_MAX_CHARS),
            );
        }

        if let Some(analysis) = preflight.e0308.as_ref() {
            push_preflight_attempt(
                &mut attempts_artifact,
                &current_partial_verify,
                &function_id,
                "preflight_e0308",
                truncate_for_artifact(&analysis.summary, HISTORY_FAILURE_DETAIL_MAX_CHARS),
            );
        }

        let decision_result = decide_preflight(
            &preflight.low_value,
            preflight.e0599.as_ref(),
            preflight.e0308.as_ref(),
        );
        let decision_detail = render_preflight_decision_detail(&decision_result.decision);
        push_preflight_attempt(
            &mut attempts_artifact,
            &current_partial_verify,
            &function_id,
            "preflight_decision",
            truncate_for_artifact(&decision_detail, HISTORY_FAILURE_DETAIL_MAX_CHARS),
        );

        if let PreflightDecision::SkipLlmCommentOut { reason, risk_flags } =
            &decision_result.decision
        {
            let candidate_id = format!(
                "{}_comment_out",
                decision_result
                    .skip_error_code
                    .unwrap_or("PREFLIGHT")
                    .to_ascii_lowercase()
            );
            if let Some((accepted_plan, accepted_verify)) = run_preflight_comment_out_attempt(
                "preflight_decision",
                1,
                &candidate_id,
                reason,
                risk_flags,
                crate_path,
                &root_function,
                &function_id,
                &current_plan,
                &current_partial_verify,
                &root_index,
                target_function_ids,
                artifacts,
                cfg.keep_updated_sources,
                verify_port,
                reporter,
                &mut attempts_artifact,
            )? {
                current_plan = accepted_plan;
                current_partial_verify = accepted_verify;
                preflight_skipped_llm_count += 1;
                if let Some(code) = decision_result.skip_error_code {
                    *preflight_skipped_llm_by_code
                        .entry(code.to_string())
                        .or_insert(0) += 1;
                }
                continue;
            }
        }

        let mut accepted = false;
        let mut blocked_by_context = false;
        let local_rule_failure_digest = local_rule_failure_digest_by_function
            .get(&function_id)
            .cloned()
            .or_else(|| {
                build_local_rule_failure_digest_fallback(
                    &function_id,
                    &current_partial_verify.error_diagnostics_by_function,
                )
            });
        let original_target_function_text = read_original_target_function_text(&root_function);

        for round in 1..=cfg.max_rounds {
            let previous_unresolved = current_partial_verify
                .unresolved_function_ids
                .iter()
                .cloned()
                .collect::<Vec<_>>();

            let (round_payload, prompt_state) = match cfg.mode {
                LlmMode::Replay => {
                    let Some(ref replay) = replay else {
                        unreachable!("replay must exist in replay mode")
                    };
                    let payload = replay.find_round(&function_id, round).cloned();
                    if payload.is_none() {
                        push_attempt(
                            &mut attempts_artifact,
                            LlmAttemptRecord {
                                function_id: function_id.clone(),
                                round,
                                phase: "round_input".to_string(),
                                candidate_id: None,
                                accepted: false,
                                failure_kind: Some(LlmFailureKind::LlmOutputInvalidSchema),
                                failure_detail: Some(
                                    "missing function round payload in replay file".to_string(),
                                ),
                                previous_unresolved_function_ids: previous_unresolved.clone(),
                                unresolved_function_ids: current_partial_verify
                                    .unresolved_function_ids
                                    .iter()
                                    .cloned()
                                    .collect(),
                                check_error_total: Some(current_partial_verify.check_error_total),
                                check_error_by_code: current_partial_verify
                                    .check_error_by_code
                                    .clone(),
                                check_stdout_log: Some(
                                    current_partial_verify.check_stdout_log.clone(),
                                ),
                                check_stderr_log: Some(
                                    current_partial_verify.check_stderr_log.clone(),
                                ),
                                raw_response_excerpt: None,
                                prompt_excerpt: None,
                                normalized_candidate: None,
                            },
                        );
                        continue;
                    }
                    (payload.unwrap(), None)
                }
                LlmMode::Online => {
                    let context_cfg = ContextBuildConfig {
                        max_chars: cfg.context_max_chars,
                        target_fn_hard_limit_chars: cfg.target_fn_hard_limit_chars,
                        primary_items_limit: 4,
                    };

                    let current_source =
                        current_source_for_function(crate_path, &current_plan, &root_function)
                            .context("failed to materialize function source for context builder")?;
                    let neighbor_fix_digest = vec![format!(
                        "applied_actions_in_same_file={}",
                        current_plan
                            .get(&root_function.file_path)
                            .map(Vec::len)
                            .unwrap_or_default()
                    )];
                    let previous_round_failure_digest = build_previous_round_failure_digest(
                        &function_id,
                        round,
                        &attempts_artifact.attempts,
                    );
                    let current_error_by_code = current_error_by_code(&function_diags);
                    let rule_error_drift_digest = build_rule_error_drift_digest(
                        rule_drift_snapshot_by_function.get(&function_id),
                        &current_error_by_code,
                        original_target_function_text.clone(),
                    );

                    let context_outcome = build_context_bundle_v1(
                        crate_path,
                        &current_source.function_for_source,
                        &current_source.source,
                        &function_diags,
                        neighbor_fix_digest,
                        local_rule_failure_digest.clone(),
                        rule_error_drift_digest,
                        preflight_interceptor_digest.clone(),
                        previous_round_failure_digest,
                        &context_cfg,
                    );
                    context_records.push(LlmContextRecord {
                        function_id: function_id.clone(),
                        round,
                        outcome: context_outcome.clone(),
                    });

                    let mut context_bundle = match context_outcome {
                        ContextBuildOutcome::Ready { bundle, .. } => bundle,
                        ContextBuildOutcome::TooLarge { reason, .. } => {
                            push_attempt(
                                &mut attempts_artifact,
                                LlmAttemptRecord {
                                    function_id: function_id.clone(),
                                    round,
                                    phase: "context".to_string(),
                                    candidate_id: None,
                                    accepted: false,
                                    failure_kind: Some(LlmFailureKind::ContextTooLarge),
                                    failure_detail: Some(reason),
                                    previous_unresolved_function_ids: previous_unresolved.clone(),
                                    unresolved_function_ids: current_partial_verify
                                        .unresolved_function_ids
                                        .iter()
                                        .cloned()
                                        .collect(),
                                    check_error_total: Some(
                                        current_partial_verify.check_error_total,
                                    ),
                                    check_error_by_code: current_partial_verify
                                        .check_error_by_code
                                        .clone(),
                                    check_stdout_log: Some(
                                        current_partial_verify.check_stdout_log.clone(),
                                    ),
                                    check_stderr_log: Some(
                                        current_partial_verify.check_stderr_log.clone(),
                                    ),
                                    raw_response_excerpt: None,
                                    prompt_excerpt: None,
                                    normalized_candidate: None,
                                },
                            );
                            blocked_by_context = true;
                            break;
                        }
                    };
                    merge_e0599_related_free_function_defs(
                        &mut context_bundle.related_fn_defs,
                        preflight.e0599.as_ref(),
                    );

                    let prompt = build_prompt_artifact(&context_bundle, round);
                    let prompt_state = OnlinePromptState {
                        user_prompt: prompt.user_prompt.clone(),
                    };

                    let Some(client) = online_client.as_ref() else {
                        unreachable!("online client must exist in online mode")
                    };
                    let payload = match client.request_round(
                        &function_id,
                        round,
                        &prompt.system_prompt,
                        &prompt.user_prompt,
                        cfg.max_candidates_per_round,
                    ) {
                        Ok(payload) => payload,
                        Err(err) => {
                            if cfg.debug_dump_full_io {
                                io_debug_records.push(LlmIoDebugRecord::request_failed(
                                    &function_id,
                                    round,
                                    &prompt.system_prompt,
                                    &prompt.user_prompt,
                                    format!("{err:#}"),
                                ));
                            }
                            push_attempt(
                                &mut attempts_artifact,
                                LlmAttemptRecord {
                                    function_id: function_id.clone(),
                                    round,
                                    phase: "request".to_string(),
                                    candidate_id: None,
                                    accepted: false,
                                    failure_kind: Some(LlmFailureKind::LlmRequestFailed),
                                    failure_detail: Some(format!("{err:#}")),
                                    previous_unresolved_function_ids: previous_unresolved.clone(),
                                    unresolved_function_ids: current_partial_verify
                                        .unresolved_function_ids
                                        .iter()
                                        .cloned()
                                        .collect(),
                                    check_error_total: Some(
                                        current_partial_verify.check_error_total,
                                    ),
                                    check_error_by_code: current_partial_verify
                                        .check_error_by_code
                                        .clone(),
                                    check_stdout_log: Some(
                                        current_partial_verify.check_stdout_log.clone(),
                                    ),
                                    check_stderr_log: Some(
                                        current_partial_verify.check_stderr_log.clone(),
                                    ),
                                    raw_response_excerpt: None,
                                    prompt_excerpt: Some(truncate_for_artifact(
                                        &prompt.user_prompt,
                                        cfg.raw_response_max_chars,
                                    )),
                                    normalized_candidate: None,
                                },
                            );
                            continue;
                        }
                    };
                    if cfg.debug_dump_full_io {
                        io_debug_records.push(LlmIoDebugRecord::request_succeeded(
                            &function_id,
                            round,
                            &prompt.system_prompt,
                            &prompt.user_prompt,
                            payload.raw_response.clone(),
                            payload.raw_transport_response.clone(),
                            payload.candidates.len(),
                        ));
                    }

                    (payload, Some(prompt_state))
                }
            };

            if round_payload.candidates.is_empty() {
                push_attempt(
                    &mut attempts_artifact,
                    LlmAttemptRecord {
                        function_id: function_id.clone(),
                        round,
                        phase: "round_candidates".to_string(),
                        candidate_id: None,
                        accepted: false,
                        failure_kind: Some(LlmFailureKind::LlmBudgetExhausted),
                        failure_detail: Some("round candidate list is empty".to_string()),
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
                        raw_response_excerpt: round_payload
                            .raw_response
                            .as_ref()
                            .map(|raw| truncate_for_artifact(raw, cfg.raw_response_max_chars)),
                        prompt_excerpt: prompt_state.as_ref().map(|state| {
                            truncate_for_artifact(&state.user_prompt, cfg.raw_response_max_chars)
                        }),
                        normalized_candidate: None,
                    },
                );
                continue;
            }

            let mut round_accepted = false;
            for candidate in round_payload
                .candidates
                .iter()
                .take(cfg.max_candidates_per_round)
            {
                let raw_excerpt = candidate
                    .raw_response
                    .as_ref()
                    .or(round_payload.raw_response.as_ref())
                    .map(|raw| truncate_for_artifact(raw, cfg.raw_response_max_chars));

                if no_std_primitive_to_string_guard_enabled(&preflight)
                    && candidate_uses_to_string(candidate)
                {
                    push_attempt(
                        &mut attempts_artifact,
                        LlmAttemptRecord {
                            function_id: function_id.clone(),
                            round,
                            phase: "normalize".to_string(),
                            candidate_id: Some(candidate.candidate_id.clone()),
                            accepted: false,
                            failure_kind: Some(LlmFailureKind::LlmActionOutOfScope),
                            failure_detail: Some(
                                "candidate uses `.to_string()` under #![no_std] primitive E0599 guard"
                                    .to_string(),
                            ),
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
                            raw_response_excerpt: raw_excerpt,
                            prompt_excerpt: prompt_state.as_ref().map(|state| {
                                truncate_for_artifact(&state.user_prompt, cfg.raw_response_max_chars)
                            }),
                            normalized_candidate: None,
                        },
                    );
                    continue;
                }

                let resolved_candidate =
                    resolve_candidate_actions(crate_path, &current_plan, &root_function, candidate);

                let (actions, normalized_candidate) = match resolved_candidate {
                    Ok(value) => value,
                    Err((failure_kind, failure_detail)) => {
                        push_attempt(
                            &mut attempts_artifact,
                            LlmAttemptRecord {
                                function_id: function_id.clone(),
                                round,
                                phase: "normalize".to_string(),
                                candidate_id: Some(candidate.candidate_id.clone()),
                                accepted: false,
                                failure_kind: Some(failure_kind),
                                failure_detail: Some(failure_detail),
                                previous_unresolved_function_ids: previous_unresolved.clone(),
                                unresolved_function_ids: current_partial_verify
                                    .unresolved_function_ids
                                    .iter()
                                    .cloned()
                                    .collect(),
                                check_error_total: Some(current_partial_verify.check_error_total),
                                check_error_by_code: current_partial_verify
                                    .check_error_by_code
                                    .clone(),
                                check_stdout_log: Some(
                                    current_partial_verify.check_stdout_log.clone(),
                                ),
                                check_stderr_log: Some(
                                    current_partial_verify.check_stderr_log.clone(),
                                ),
                                raw_response_excerpt: raw_excerpt,
                                prompt_excerpt: prompt_state.as_ref().map(|state| {
                                    truncate_for_artifact(
                                        &state.user_prompt,
                                        cfg.raw_response_max_chars,
                                    )
                                }),
                                normalized_candidate: None,
                            },
                        );
                        continue;
                    }
                };

                let mut trial_plan = plan_without_function_actions(
                    &current_plan,
                    crate_path,
                    &root_index,
                    &function_id,
                );
                if !merge_actions_without_conflict(&mut trial_plan, &actions) {
                    push_attempt(
                        &mut attempts_artifact,
                        LlmAttemptRecord {
                            function_id: function_id.clone(),
                            round,
                            phase: "merge".to_string(),
                            candidate_id: Some(candidate.candidate_id.clone()),
                            accepted: false,
                            failure_kind: Some(LlmFailureKind::LlmActionConflict),
                            failure_detail: Some(
                                "candidate actions conflict with current partial plan".to_string(),
                            ),
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
                            raw_response_excerpt: raw_excerpt,
                            prompt_excerpt: prompt_state.as_ref().map(|state| {
                                truncate_for_artifact(
                                    &state.user_prompt,
                                    cfg.raw_response_max_chars,
                                )
                            }),
                            normalized_candidate: Some(normalized_candidate),
                        },
                    );
                    continue;
                }

                let attempt_tag = format!(
                    "llm/{}/r{}/{}",
                    sanitize_path_component(&function_id),
                    round,
                    sanitize_path_component(&candidate.candidate_id)
                );
                let trial_verify = verify_port.verify_partial_union_plan_with_tag(
                    crate_path,
                    &trial_plan,
                    target_function_ids,
                    artifacts,
                    cfg.keep_updated_sources,
                    &attempt_tag,
                    reporter,
                )?;

                let target_identity = stable_function_identity(&function_id);
                let target_cleared = !trial_verify
                    .unresolved_function_ids
                    .iter()
                    .map(|item| stable_function_identity(item))
                    .any(|identity| identity == target_identity);
                let ignored_identities: BTreeSet<String> = root_index
                    .function_ids_in_same_test_module(&function_id)
                    .into_iter()
                    .map(|item| stable_function_identity(&item))
                    .collect();
                let introduced_error_by_code = introduced_error_by_code(
                    &current_partial_verify.check_error_by_code,
                    &trial_verify.check_error_by_code,
                );
                let prev_non_target = non_target_unresolved_stable_id_set(
                    &current_partial_verify.unresolved_function_ids,
                    &ignored_identities,
                );
                let now_non_target = non_target_unresolved_stable_id_set(
                    &trial_verify.unresolved_function_ids,
                    &ignored_identities,
                );
                let no_new_non_target_regression = non_target_state_acceptable(
                    &prev_non_target,
                    &now_non_target,
                    &introduced_error_by_code,
                    current_partial_verify.check_error_total,
                    trial_verify.check_error_total,
                );
                let introduced_error_codes = render_error_counts(&introduced_error_by_code);

                if target_cleared && no_new_non_target_regression {
                    push_attempt(
                        &mut attempts_artifact,
                        LlmAttemptRecord {
                            function_id: function_id.clone(),
                            round,
                            phase: "verify".to_string(),
                            candidate_id: Some(candidate.candidate_id.clone()),
                            accepted: true,
                            failure_kind: None,
                            failure_detail: None,
                            previous_unresolved_function_ids: previous_unresolved.clone(),
                            unresolved_function_ids: trial_verify
                                .unresolved_function_ids
                                .iter()
                                .cloned()
                                .collect(),
                            check_error_total: Some(trial_verify.check_error_total),
                            check_error_by_code: trial_verify.check_error_by_code.clone(),
                            check_stdout_log: Some(trial_verify.check_stdout_log.clone()),
                            check_stderr_log: Some(trial_verify.check_stderr_log.clone()),
                            raw_response_excerpt: raw_excerpt,
                            prompt_excerpt: prompt_state.as_ref().map(|state| {
                                truncate_for_artifact(
                                    &state.user_prompt,
                                    cfg.raw_response_max_chars,
                                )
                            }),
                            normalized_candidate: Some(normalized_candidate),
                        },
                    );
                    current_plan = trial_plan;
                    current_partial_verify = trial_verify;
                    round_accepted = true;
                    accepted = true;
                    break;
                }

                let reason = if !target_cleared {
                    format!(
                        "target function still unresolved after verify; introduced_error_codes={introduced_error_codes}"
                    )
                } else {
                    format!(
                        "non-target function regression introduced; introduced_error_codes={introduced_error_codes}"
                    )
                };
                push_attempt(
                    &mut attempts_artifact,
                    LlmAttemptRecord {
                        function_id: function_id.clone(),
                        round,
                        phase: "verify".to_string(),
                        candidate_id: Some(candidate.candidate_id.clone()),
                        accepted: false,
                        failure_kind: Some(LlmFailureKind::LlmVerifyFailed),
                        failure_detail: Some(reason),
                        previous_unresolved_function_ids: previous_unresolved.clone(),
                        unresolved_function_ids: trial_verify
                            .unresolved_function_ids
                            .iter()
                            .cloned()
                            .collect(),
                        check_error_total: Some(trial_verify.check_error_total),
                        check_error_by_code: trial_verify.check_error_by_code.clone(),
                        check_stdout_log: Some(trial_verify.check_stdout_log.clone()),
                        check_stderr_log: Some(trial_verify.check_stderr_log.clone()),
                        raw_response_excerpt: raw_excerpt,
                        prompt_excerpt: prompt_state.as_ref().map(|state| {
                            truncate_for_artifact(&state.user_prompt, cfg.raw_response_max_chars)
                        }),
                        normalized_candidate: Some(normalized_candidate),
                    },
                );
            }

            if round_accepted {
                break;
            }
        }

        if !accepted && !blocked_by_context {
            if !matches!(
                &decision_result.decision,
                PreflightDecision::SkipLlmCommentOut { .. }
            ) && should_try_budget_e0308_fallback(&preflight.low_value, preflight.e0308.as_ref())
            {
                let reason = preflight
                    .e0308
                    .as_ref()
                    .map(|analysis| analysis.summary.clone())
                    .unwrap_or_else(|| "e0308 high-risk fallback".to_string());
                let risk_flags = vec!["budget_fallback".to_string(), "e0308_high_risk".to_string()];
                if let Some((accepted_plan, accepted_verify)) = run_preflight_comment_out_attempt(
                    "preflight_budget_e0308",
                    cfg.max_rounds,
                    "e0308_budget_comment_out",
                    &reason,
                    &risk_flags,
                    crate_path,
                    &root_function,
                    &function_id,
                    &current_plan,
                    &current_partial_verify,
                    &root_index,
                    target_function_ids,
                    artifacts,
                    cfg.keep_updated_sources,
                    verify_port,
                    reporter,
                    &mut attempts_artifact,
                )? {
                    current_plan = accepted_plan;
                    current_partial_verify = accepted_verify;
                    continue;
                }
            }

            push_attempt(
                &mut attempts_artifact,
                LlmAttemptRecord {
                    function_id: function_id.clone(),
                    round: cfg.max_rounds,
                    phase: "budget".to_string(),
                    candidate_id: None,
                    accepted: false,
                    failure_kind: Some(LlmFailureKind::LlmBudgetExhausted),
                    failure_detail: Some(format!(
                        "all candidates failed within {} rounds",
                        cfg.max_rounds
                    )),
                    previous_unresolved_function_ids: current_partial_verify
                        .unresolved_function_ids
                        .iter()
                        .cloned()
                        .collect(),
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
        }
    }

    reporter.kv(
        0,
        "llm_attempt_count",
        attempts_artifact.attempts.len().to_string(),
    );
    reporter.kv(
        0,
        "llm_remaining_unresolved_count",
        current_partial_verify
            .unresolved_function_ids
            .len()
            .to_string(),
    );
    reporter.kv(
        0,
        "preflight_skipped_llm_count",
        preflight_skipped_llm_count.to_string(),
    );
    reporter.kv(
        0,
        "preflight_skipped_llm_by_code",
        render_error_counts(&preflight_skipped_llm_by_code),
    );

    write_json(&artifacts.llm_contexts_json, &context_records)?;
    if cfg.debug_dump_full_io {
        write_json(&artifacts.llm_io_debug_json, &io_debug_records)?;
        reporter.kv(
            0,
            "llm_io_debug_artifact",
            artifacts.llm_io_debug_json.display().to_string(),
        );
    }

    Ok(LlmReplayStageResult {
        plan: current_plan,
        partial_verify: current_partial_verify,
        attempts_artifact,
        preflight_skipped_llm_count,
        preflight_skipped_llm_by_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_function_identity_drops_line_suffix() {
        let id = "src/udiv128.rs::tests_rug_16::test_max_usize:128:135";
        assert_eq!(
            stable_function_identity(id),
            "src/udiv128.rs::tests_rug_16::test_max_usize"
        );
    }

    #[test]
    fn non_target_state_accepts_equivalent_churn_without_error_growth() {
        let previous_non_target = BTreeSet::from([
            "src/a.rs::tests::case_a".to_string(),
            "src/a.rs::tests::case_b".to_string(),
        ]);
        let current_non_target = BTreeSet::from([
            "src/a.rs::tests::case_a".to_string(),
            "src/a.rs::tests::case_c".to_string(),
        ]);
        let introduced = BTreeMap::new();
        assert!(non_target_state_acceptable(
            &previous_non_target,
            &current_non_target,
            &introduced,
            10,
            10
        ));
    }

    #[test]
    fn non_target_id_set_excludes_same_module_identities() {
        let unresolved = BTreeSet::from([
            "src/a.rs::tests::case_a:10:20".to_string(),
            "src/a.rs::tests::case_b:21:30".to_string(),
            "src/a.rs::other::case_x:31:40".to_string(),
        ]);
        let ignored = BTreeSet::from([
            "src/a.rs::tests::case_a".to_string(),
            "src/a.rs::tests::case_b".to_string(),
        ]);
        let non_target = non_target_unresolved_stable_id_set(&unresolved, &ignored);
        assert_eq!(
            non_target,
            BTreeSet::from(["src/a.rs::other::case_x".to_string()])
        );
    }

    #[test]
    fn non_target_state_rejects_when_new_errors_are_introduced() {
        let previous_non_target = BTreeSet::from(["src/a.rs::tests::case_a".to_string()]);
        let current_non_target = BTreeSet::from(["src/a.rs::tests::case_b".to_string()]);
        let introduced = BTreeMap::from([("E0433".to_string(), 1)]);
        assert!(!non_target_state_acceptable(
            &previous_non_target,
            &current_non_target,
            &introduced,
            10,
            10
        ));
    }

    #[test]
    fn non_target_state_accepts_count_growth_when_error_total_not_worse() {
        let previous_non_target = BTreeSet::from(["src/a.rs::tests::case_a".to_string()]);
        let current_non_target = BTreeSet::from([
            "src/a.rs::tests::case_b".to_string(),
            "src/a.rs::tests::case_c".to_string(),
        ]);
        let introduced = BTreeMap::new();
        assert!(non_target_state_acceptable(
            &previous_non_target,
            &current_non_target,
            &introduced,
            10,
            10
        ));
    }

    #[test]
    fn no_std_to_string_guard_only_matches_primitive_type_e0599() {
        let analyses = PreflightAnalyses {
            crate_env_no_std: true,
            low_value: crate::runtime::function::low_value::LowValueAnalysis {
                status: crate::runtime::function::low_value::LowValueStatus::HasTestSemantics,
                reason: "x".to_string(),
                markers: vec![],
            },
            e0599: Some(ruter::patchers::e0599::E0599Analysis {
                classification: ruter::patchers::e0599::E0599Classification::MinimalPass,
                target: Some(ruter::patchers::e0599::E0599Target {
                    method_name: "to_string".to_string(),
                    raw_type_name: "usize".to_string(),
                    normalized_type_name: "usize".to_string(),
                    raw_kind: "type".to_string(),
                }),
                matched_impl_count: 0,
                available_method_signatures: vec![],
                related_free_function_signatures: vec![],
                recommended_call_forms: vec![],
                scope: "none".to_string(),
                summary: "x".to_string(),
            }),
            e0308: None,
            e0432: None,
            e0560: None,
        };
        assert!(no_std_primitive_to_string_guard_enabled(&analyses));
    }

    #[test]
    fn detect_rule_error_drift_pairs_hits_e0433_to_e0599() {
        let original = BTreeMap::from([("E0433".to_string(), 1)]);
        let current = BTreeMap::from([("E0599".to_string(), 1)]);
        let pairs = detect_rule_error_drift_pairs(&original, &current);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].from_code, "E0433");
        assert_eq!(pairs[0].to_code, "E0599");
    }

    #[test]
    fn detect_rule_error_drift_pairs_empty_when_code_set_stable() {
        let original = BTreeMap::from([("E0433".to_string(), 1)]);
        let current = BTreeMap::from([("E0433".to_string(), 2)]);
        let pairs = detect_rule_error_drift_pairs(&original, &current);
        assert!(pairs.is_empty());
    }

    #[test]
    fn build_rule_error_drift_digest_requires_rule_attempts() {
        let snapshot = RuleDriftSnapshot {
            original_error_by_code: BTreeMap::from([("E0433".to_string(), 1)]),
            selected_rule_fix_summaries: vec!["r1 rank=1 trace=x actions=y".to_string()],
            attempted_rule_rounds: 0,
        };
        let current = BTreeMap::from([("E0599".to_string(), 1)]);
        let digest =
            build_rule_error_drift_digest(Some(&snapshot), &current, Some("fn t() {}".to_string()));
        assert!(digest.is_none());
    }

    #[test]
    fn merge_e0599_related_free_fn_defs_dedups_existing_entries() {
        let mut related = vec!["fn helper ()".to_string()];
        let analysis = ruter::patchers::e0599::E0599Analysis {
            classification: ruter::patchers::e0599::E0599Classification::MisplacedFreeFunction,
            target: None,
            matched_impl_count: 1,
            available_method_signatures: vec![],
            related_free_function_signatures: vec![
                "fn format_rfc3339 (system_time : SystemTime) -> Rfc3339Timestamp".to_string(),
                "fn helper ()".to_string(),
            ],
            recommended_call_forms: vec![],
            scope: "crate_fallback".to_string(),
            summary: "s".to_string(),
        };
        merge_e0599_related_free_function_defs(&mut related, Some(&analysis));
        assert!(related.iter().any(|entry| entry.contains("format_rfc3339")));
        assert_eq!(
            related.iter().filter(|entry| entry.contains("fn helper ()")).count(),
            1
        );
    }
}
