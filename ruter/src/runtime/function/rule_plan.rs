use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use anyhow::Result;
use ruter::core::{Diagnostic, ErrorCode, FixAction};
use ruter::patchers::PatcherRegistry;
use ruter::patchers::e0433::E0433Patcher;
use ruter::transformer::CodeTransformer;
use serde::{Deserialize, Serialize};

use crate::runtime::function::dispatch::{
    FunctionDispatchDecision, FunctionDispatchOutput, FunctionPatchTask, FunctionRuleDiagnosticSet,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDiagnosticTrace {
    pub diagnostic_index: usize,
    pub code: String,
    pub patcher: String,
    pub selected_rank: usize,
    pub available_rank_count: usize,
    pub reused_last: bool,
    pub candidate_score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZipAlignmentMeta {
    pub repeated_last_count: usize,
    pub diagnostic_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionRuleCandidate {
    pub function_id: String,
    pub rank: usize,
    pub score: i32,
    pub actions: Vec<FixAction>,
    pub diagnostic_trace: Vec<FunctionDiagnosticTrace>,
    pub zip_alignment_meta: ZipAlignmentMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionRuleCandidatesArtifact {
    pub function_id: String,
    pub candidates: Vec<FunctionRuleCandidate>,
}

#[derive(Debug, Clone)]
pub struct FunctionRulePlanResult {
    pub candidates_by_function: BTreeMap<String, Vec<FunctionRuleCandidate>>,
    pub flat_candidates: Vec<FunctionRuleCandidate>,
    pub max_rank: usize,
}

#[derive(Debug, Clone)]
struct RankedDiagnosticCandidate {
    rank: usize,
    score: i32,
    actions: Vec<FixAction>,
    patcher: String,
}

#[derive(Debug, Clone)]
struct DiagnosticRankedList {
    diagnostic_index: usize,
    code: String,
    candidates: Vec<RankedDiagnosticCandidate>,
}

pub fn plan_rule_candidates(
    dispatch: &FunctionDispatchOutput,
    registry: &PatcherRegistry,
    k: usize,
) -> Result<FunctionRulePlanResult> {
    let max_rank = k.max(1);
    let mut candidates_by_function: BTreeMap<String, Vec<FunctionRuleCandidate>> = BTreeMap::new();
    let tasks_by_function: HashMap<_, _> = dispatch
        .tasks
        .iter()
        .map(|task| (task.function_id.clone(), task))
        .collect();

    for function_id in &dispatch.rule_function_ids {
        let Some(task) = tasks_by_function.get(function_id) else {
            continue;
        };
        let Some(decision) = dispatch.decisions.get(function_id) else {
            continue;
        };
        let Some(selected) = dispatch.rule_diagnostic_sets.get(function_id) else {
            continue;
        };

        let ranked_lists = build_diagnostic_ranked_lists(task, decision, selected, registry, k)?;
        let function_candidates = build_zip_candidates(function_id, &ranked_lists, max_rank);

        candidates_by_function.insert(function_id.clone(), function_candidates);
    }

    let mut flat_candidates = Vec::new();
    for candidates in candidates_by_function.values() {
        flat_candidates.extend_from_slice(candidates);
    }

    Ok(FunctionRulePlanResult {
        candidates_by_function,
        flat_candidates,
        max_rank,
    })
}

fn build_diagnostic_ranked_lists(
    _task: &FunctionPatchTask,
    decision: &FunctionDispatchDecision,
    selected: &FunctionRuleDiagnosticSet,
    registry: &PatcherRegistry,
    k: usize,
) -> Result<Vec<DiagnosticRankedList>> {
    let mut out = Vec::new();

    let FunctionDispatchDecision::RulePatcher {
        selected_rule_codes,
        ..
    } = decision
    else {
        return Ok(out);
    };
    let selected_codes: BTreeSet<String> = selected_rule_codes.iter().cloned().collect();

    for diag_ref in &selected.selected_diagnostics {
        if !selected_codes.contains(&diag_ref.code) {
            continue;
        }

        let ranked = ranked_candidates_for_diagnostic(&diag_ref.diagnostic, registry, k)?;
        if ranked.is_empty() {
            continue;
        }

        out.push(DiagnosticRankedList {
            diagnostic_index: diag_ref.diagnostic_index,
            code: diag_ref.code.clone(),
            candidates: ranked,
        });
    }

    out.sort_by_key(|item| item.diagnostic_index);
    Ok(out)
}

fn ranked_candidates_for_diagnostic(
    diagnostic: &Diagnostic,
    registry: &PatcherRegistry,
    k: usize,
) -> Result<Vec<RankedDiagnosticCandidate>> {
    let Some(code) = diagnostic.code.as_ref().map(|value| value.code) else {
        return Ok(Vec::new());
    };

    if code == ErrorCode::E0433 {
        let e0433 = E0433Patcher::new();
        let ranked = e0433.analyze_top_k(diagnostic, k)?;
        let mut out = Vec::new();
        for (idx, item) in ranked.into_iter().enumerate() {
            out.push(RankedDiagnosticCandidate {
                rank: idx + 1,
                score: item.score,
                actions: vec![item.action],
                patcher: "E0433Patcher".to_string(),
            });
        }
        return Ok(out);
    }
    // TODO: 支持更多 error code 的 patcher。它们的排序、评分逻辑可能和 E0433Patcher 不同，后续需要抽象一个统一的接口供 Patcher 实现。

    let Some(patcher) = registry.find_patcher_by_code(code) else {
        return Ok(Vec::new());
    };

    let actions = patcher.analyze(diagnostic)?;
    if actions.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![RankedDiagnosticCandidate {
        rank: 1,
        score: 0,
        actions,
        patcher: patcher.description().to_string(),
    }])
}

fn build_zip_candidates(
    function_id: &str,
    ranked_lists: &[DiagnosticRankedList],
    max_rank: usize,
) -> Vec<FunctionRuleCandidate> {
    if ranked_lists.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();

    for rank in 1..=max_rank {
        let mut actions_by_file: BTreeMap<PathBuf, Vec<FixAction>> = BTreeMap::new();
        let mut trace = Vec::new();
        let mut score = 0i32;
        let mut repeated_last_count = 0usize;

        for list in ranked_lists {
            if list.candidates.is_empty() {
                continue;
            }
            let selected_idx = (rank - 1).min(list.candidates.len() - 1);
            let selected = &list.candidates[selected_idx];
            if rank > list.candidates.len() {
                repeated_last_count += 1;
            }

            for action in &selected.actions {
                let file = action_file(action);
                actions_by_file
                    .entry(file)
                    .or_default()
                    .push(action.clone());
            }

            trace.push(FunctionDiagnosticTrace {
                diagnostic_index: list.diagnostic_index,
                code: list.code.clone(),
                patcher: selected.patcher.clone(),
                selected_rank: selected.rank,
                available_rank_count: list.candidates.len(),
                reused_last: rank > list.candidates.len(),
                candidate_score: selected.score,
            });
            score += selected.score;
        }

        // 函数候选必须是无冲突动作集合，否则该 rank 候选丢弃。
        if has_conflicts(&mut actions_by_file) {
            continue;
        }

        let actions = actions_by_file.into_values().flatten().collect::<Vec<_>>();
        out.push(FunctionRuleCandidate {
            function_id: function_id.to_string(),
            rank,
            score,
            actions,
            diagnostic_trace: trace,
            zip_alignment_meta: ZipAlignmentMeta {
                repeated_last_count,
                diagnostic_count: ranked_lists.len(),
            },
        });
    }

    out
}

fn has_conflicts(actions_by_file: &mut BTreeMap<PathBuf, Vec<FixAction>>) -> bool {
    for actions in actions_by_file.values_mut() {
        actions.sort_by(|a, b| {
            let (a_start, a_end) = action_range(a);
            let (b_start, b_end) = action_range(b);
            a_start.cmp(&b_start).then_with(|| a_end.cmp(&b_end))
        });
        if CodeTransformer::ensure_no_conflicts(actions).is_err() {
            return true;
        }
    }
    false
}

fn action_file(action: &FixAction) -> PathBuf {
    match action {
        FixAction::Insert { span, .. }
        | FixAction::Replace { span, .. }
        | FixAction::Delete { span } => span.file_path.clone(),
    }
}

fn action_range(action: &FixAction) -> (usize, usize) {
    match action {
        FixAction::Insert { span, .. }
        | FixAction::Replace { span, .. }
        | FixAction::Delete { span } => (span.byte_start, span.byte_end),
    }
}

#[cfg(test)]
mod tests;
