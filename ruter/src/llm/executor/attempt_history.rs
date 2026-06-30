use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::llm::context_builder::{
    LocalRuleFailureDigestV1, PreviousCandidateFailureDigestV1, PreviousRoundFailureDigestV1,
};
use crate::llm::schema::{
    LlmAttemptRecord, LlmAttemptsArtifact, LlmFailureKind, truncate_for_artifact,
};
use crate::runtime::function::verify::FunctionVerifyRoundRecord;
use ruter::core::FunctionDiagnostic;

use super::{
    HISTORY_FAILURE_DETAIL_MAX_CHARS, HISTORY_PATCH_SUMMARY_MAX_CHARS,
    LOCAL_RULE_SUMMARY_LINE_MAX_CHARS,
};

pub(super) fn push_attempt(artifact: &mut LlmAttemptsArtifact, record: LlmAttemptRecord) {
    artifact.attempts.push(record);
}

pub(super) fn load_local_rule_failure_digest_by_function(
    rounds_path: &Path,
) -> BTreeMap<String, LocalRuleFailureDigestV1> {
    if !rounds_path.exists() {
        return BTreeMap::new();
    }

    let Ok(raw) = fs::read_to_string(rounds_path) else {
        return BTreeMap::new();
    };
    let Ok(rounds) = serde_json::from_str::<Vec<FunctionVerifyRoundRecord>>(&raw) else {
        return BTreeMap::new();
    };
    summarize_local_rule_failure_rounds(&rounds)
}

pub(super) fn summarize_local_rule_failure_rounds(
    rounds: &[FunctionVerifyRoundRecord],
) -> BTreeMap<String, LocalRuleFailureDigestV1> {
    let Some(last_round) = rounds.last() else {
        return BTreeMap::new();
    };
    let round_count = rounds.len();

    let mut unresolved = last_round.unresolved_function_ids.clone();
    unresolved.sort();

    let mut by_function: BTreeMap<String, LocalRuleFailureDigestV1> = BTreeMap::new();

    for function_id in unresolved {
        let mut selected_rank_history = Vec::new();
        let mut unresolved_rounds = Vec::new();
        let mut independence_broken_rounds = Vec::new();

        for round in rounds {
            if let Some(rank) = round.selected_rank_by_function.get(&function_id) {
                selected_rank_history.push(format!("r{}:{}", round.round, rank));
            }
            if round
                .unresolved_function_ids
                .iter()
                .any(|id| id == &function_id)
            {
                unresolved_rounds.push(round.round);
            }
            if round
                .independence_broken_function_ids
                .iter()
                .any(|id| id == &function_id)
            {
                independence_broken_rounds.push(round.round);
            }
        }

        let mut summary_lines = Vec::new();
        summary_lines.push(format!("rule_verify_rounds={round_count}"));
        if !selected_rank_history.is_empty() {
            summary_lines.push(format!(
                "selected_rank_history={}",
                selected_rank_history.join(",")
            ));
        }
        if !unresolved_rounds.is_empty() {
            summary_lines.push(format!(
                "unresolved_rounds={}",
                join_usize_csv(&unresolved_rounds)
            ));
        }
        if !independence_broken_rounds.is_empty() {
            summary_lines.push(format!(
                "independence_broken_rounds={}",
                join_usize_csv(&independence_broken_rounds)
            ));
        }

        summary_lines = summary_lines
            .into_iter()
            .map(|line| truncate_for_artifact(&line, LOCAL_RULE_SUMMARY_LINE_MAX_CHARS))
            .collect();

        by_function.insert(function_id, LocalRuleFailureDigestV1 { summary_lines });
    }

    by_function
}

pub(super) fn build_local_rule_failure_digest_fallback(
    function_id: &str,
    error_diagnostics_by_function: &BTreeMap<String, Vec<FunctionDiagnostic>>,
) -> Option<LocalRuleFailureDigestV1> {
    let diags = error_diagnostics_by_function.get(function_id)?;
    if diags.is_empty() {
        return None;
    }

    let mut error_by_code = BTreeMap::<String, usize>::new();
    for diag in diags {
        *error_by_code.entry(diag.code.clone()).or_insert(0) += 1;
    }

    let mut summary_lines = vec![
        "local_rule_handoff_reason=unresolved_after_rule_verify".to_string(),
        format!(
            "remaining_error_by_code={}",
            render_error_counts(&error_by_code)
        ),
    ];
    summary_lines = summary_lines
        .into_iter()
        .map(|line| truncate_for_artifact(&line, LOCAL_RULE_SUMMARY_LINE_MAX_CHARS))
        .collect();

    Some(LocalRuleFailureDigestV1 { summary_lines })
}

fn join_usize_csv(values: &[usize]) -> String {
    values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn render_error_counts(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(code, count)| format!("{code}={count}"))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn build_previous_round_failure_digest(
    function_id: &str,
    round: u8,
    attempts: &[LlmAttemptRecord],
) -> Option<PreviousRoundFailureDigestV1> {
    if round <= 1 {
        return None;
    }
    let previous_round = round - 1;

    let mut selected_by_candidate = BTreeMap::<String, &LlmAttemptRecord>::new();
    for attempt in attempts.iter().filter(|attempt| {
        attempt.function_id == function_id
            && attempt.round == previous_round
            && !attempt.accepted
            && attempt.failure_kind.is_some()
    }) {
        let key = if let Some(candidate_id) = attempt.candidate_id.as_ref() {
            candidate_id.clone()
        } else {
            format!("round_event_{}", sanitize_history_key(&attempt.phase))
        };

        if let Some(existing) = selected_by_candidate.get(&key) {
            let existing_priority = failure_phase_priority(&existing.phase);
            let current_priority = failure_phase_priority(&attempt.phase);
            if current_priority < existing_priority {
                continue;
            }
        }
        selected_by_candidate.insert(key, attempt);
    }

    if selected_by_candidate.is_empty() {
        return None;
    }

    let mut kind_counts = BTreeMap::<String, usize>::new();
    let mut candidate_failures = Vec::new();
    for (candidate_id, attempt) in selected_by_candidate {
        let Some(failure_kind) = attempt.failure_kind.as_ref() else {
            continue;
        };
        let failure_kind_name = failure_kind_name(failure_kind).to_string();
        *kind_counts.entry(failure_kind_name.clone()).or_insert(0) += 1;

        candidate_failures.push(PreviousCandidateFailureDigestV1 {
            candidate_id,
            patch_summary: patch_summary_from_attempt(attempt),
            failure_kind: failure_kind_name,
            failure_detail: attempt
                .failure_detail
                .as_deref()
                .map(|detail| truncate_for_artifact(detail, HISTORY_FAILURE_DETAIL_MAX_CHARS)),
            unresolved_error_by_code: attempt.check_error_by_code.clone(),
            introduced_error_by_code: parse_introduced_error_by_code(
                attempt.failure_detail.as_deref(),
            ),
        });
    }

    let mut dominant_failure_kinds = kind_counts.into_iter().collect::<Vec<_>>();
    dominant_failure_kinds.sort_by(|(kind_a, count_a), (kind_b, count_b)| {
        count_b.cmp(count_a).then_with(|| kind_a.cmp(kind_b))
    });

    Some(PreviousRoundFailureDigestV1 {
        round: previous_round,
        dominant_failure_kinds: dominant_failure_kinds
            .into_iter()
            .map(|(kind, _)| kind)
            .collect(),
        candidate_failures,
    })
}

fn sanitize_history_key(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

fn failure_phase_priority(phase: &str) -> u8 {
    match phase {
        "verify" => 60,
        "merge" => 50,
        "normalize" => 40,
        "request" => 30,
        "context" => 20,
        "round_candidates" => 10,
        "round_input" => 5,
        _ => 0,
    }
}

fn failure_kind_name(kind: &LlmFailureKind) -> &'static str {
    match kind {
        LlmFailureKind::LlmRequestFailed => "LLM_REQUEST_FAILED",
        LlmFailureKind::LlmOutputInvalidSchema => "LLM_OUTPUT_INVALID_SCHEMA",
        LlmFailureKind::LlmActionOutOfScope => "LLM_ACTION_OUT_OF_SCOPE",
        LlmFailureKind::LlmActionConflict => "LLM_ACTION_CONFLICT",
        LlmFailureKind::LlmVerifyFailed => "LLM_VERIFY_FAILED",
        LlmFailureKind::LlmBudgetExhausted => "LLM_BUDGET_EXHAUSTED",
        LlmFailureKind::FunctionMappingFailed => "FUNCTION_MAPPING_FAILED",
        LlmFailureKind::ContextTooLarge => "CONTEXT_TOO_LARGE",
    }
}

fn patch_summary_from_attempt(attempt: &LlmAttemptRecord) -> String {
    if let Some(candidate) = attempt.normalized_candidate.as_ref() {
        if !candidate.action_summaries.is_empty() {
            return truncate_for_artifact(
                &candidate.action_summaries.join(" | "),
                HISTORY_PATCH_SUMMARY_MAX_CHARS,
            );
        }
        if let Some(text) = candidate.normalized_text_excerpt.as_deref() {
            return truncate_for_artifact(text, HISTORY_PATCH_SUMMARY_MAX_CHARS);
        }
    }
    if let Some(raw) = attempt.raw_response_excerpt.as_deref() {
        return truncate_for_artifact(raw, HISTORY_PATCH_SUMMARY_MAX_CHARS);
    }
    if attempt.phase.is_empty() {
        "phase=unknown".to_string()
    } else {
        format!("phase={}", attempt.phase)
    }
}

fn parse_introduced_error_by_code(detail: Option<&str>) -> BTreeMap<String, usize> {
    let Some(detail) = detail else {
        return BTreeMap::new();
    };
    let Some((_, raw_counts)) = detail.split_once("introduced_error_codes=") else {
        return BTreeMap::new();
    };
    let payload = raw_counts.trim();
    if payload.is_empty() || payload == "none" {
        return BTreeMap::new();
    }

    let mut out = BTreeMap::new();
    for token in payload.split(',') {
        let Some((code, raw_count)) = token.split_once('=') else {
            continue;
        };
        let code = code.trim();
        let count = raw_count.trim().parse::<usize>().ok();
        if code.is_empty() {
            continue;
        }
        if let Some(count) = count {
            out.insert(code.to_string(), count);
        }
    }
    out
}
