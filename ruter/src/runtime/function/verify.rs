use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::Result;
use ruter::core::FixAction;
use ruter::transformer::CodeTransformer;
use serde::{Deserialize, Serialize};

use crate::runtime::function::rule_plan::FunctionRuleCandidate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionVerifyRoundRecord {
    pub round: usize,
    pub selected_rank_by_function: BTreeMap<String, usize>,
    pub plan_file_count: usize,
    pub plan_action_count: usize,
    pub check_error_total: usize,
    pub check_error_by_code: BTreeMap<String, usize>,
    pub resolved_function_ids: Vec<String>,
    pub unresolved_function_ids: Vec<String>,
    pub independence_broken_function_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RoundEvaluation {
    pub check_error_total: usize,
    pub check_error_by_code: BTreeMap<String, usize>,
    pub unresolved_function_ids: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionOptimisticVerifyResult {
    pub rounds: Vec<FunctionVerifyRoundRecord>,
    pub final_plan: BTreeMap<PathBuf, Vec<FixAction>>,
    pub independence_broken_function_ids: BTreeSet<String>,
}

/// Optimistic Greedy：每轮 unresolved 函数批量升 rank，直到收敛或预算耗尽。
pub fn run_optimistic_greedy<F>(
    candidates_by_function: &BTreeMap<String, Vec<FunctionRuleCandidate>>,
    target_function_ids: &BTreeSet<String>,
    k: usize,
    mut evaluate: F,
) -> Result<FunctionOptimisticVerifyResult>
where
    F: FnMut(&BTreeMap<PathBuf, Vec<FixAction>>) -> Result<RoundEvaluation>,
{
    let max_rank = k.max(1);
    let mut rank_by_function = BTreeMap::new();
    for (function_id, candidates) in candidates_by_function {
        if !candidates.is_empty() {
            rank_by_function.insert(function_id.clone(), 1usize);
        }
    }

    let mut rounds = Vec::new();
    let mut last_plan_signature = None;
    let mut independence_broken = BTreeSet::new();

    let mut final_plan = BTreeMap::new();
    for round in 1..=max_rank {
        let (plan, selected_rank_by_function, blocked) = compose_round_plan(
            candidates_by_function,
            &rank_by_function,
            &mut independence_broken,
        );

        let plan_signature = plan_signature(&plan);
        let evaluation = evaluate(&plan)?;

        let mut unresolved = evaluation.unresolved_function_ids.clone();
        unresolved.extend(blocked.iter().cloned());
        unresolved.extend(independence_broken.iter().cloned());

        let resolved: BTreeSet<String> = target_function_ids
            .difference(&unresolved)
            .cloned()
            .collect();

        rounds.push(FunctionVerifyRoundRecord {
            round,
            selected_rank_by_function,
            plan_file_count: plan.len(),
            plan_action_count: plan.values().map(Vec::len).sum(),
            check_error_total: evaluation.check_error_total,
            check_error_by_code: evaluation.check_error_by_code.clone(),
            resolved_function_ids: resolved.iter().cloned().collect(),
            unresolved_function_ids: unresolved.iter().cloned().collect(),
            independence_broken_function_ids: independence_broken.iter().cloned().collect(),
        });

        final_plan = plan;
        let final_unresolved = unresolved.clone();

        if final_unresolved.is_empty() {
            break;
        }

        let mut changed = false;
        for function_id in &final_unresolved {
            if independence_broken.contains(function_id) {
                continue;
            }
            let Some(rank) = rank_by_function.get_mut(function_id) else {
                continue;
            };
            if *rank < max_rank {
                *rank += 1;
                changed = true;
            }
        }

        let same_as_last = last_plan_signature
            .as_ref()
            .map(|previous| previous == &plan_signature)
            .unwrap_or(false);

        let unresolved_all_at_max = final_unresolved.iter().all(|function_id| {
            rank_by_function
                .get(function_id)
                .map(|rank| *rank >= max_rank)
                .unwrap_or(true)
        });

        if !changed {
            break;
        }
        if same_as_last && unresolved_all_at_max {
            break;
        }

        last_plan_signature = Some(plan_signature);
    }

    Ok(FunctionOptimisticVerifyResult {
        rounds,
        final_plan,
        independence_broken_function_ids: independence_broken,
    })
}

fn compose_round_plan(
    candidates_by_function: &BTreeMap<String, Vec<FunctionRuleCandidate>>,
    rank_by_function: &BTreeMap<String, usize>,
    independence_broken: &mut BTreeSet<String>,
) -> (
    BTreeMap<PathBuf, Vec<FixAction>>,
    BTreeMap<String, usize>,
    BTreeSet<String>,
) {
    let mut plan: BTreeMap<PathBuf, Vec<FixAction>> = BTreeMap::new();
    let mut selected_rank_by_function = BTreeMap::new();
    let mut blocked = BTreeSet::new();

    for (function_id, candidates) in candidates_by_function {
        if independence_broken.contains(function_id) {
            continue;
        }
        if candidates.is_empty() {
            continue;
        }
        let rank = rank_by_function.get(function_id).copied().unwrap_or(1);
        let idx = (rank - 1).min(candidates.len() - 1);
        let candidate = &candidates[idx];

        if merge_actions_without_conflict(&mut plan, &candidate.actions) {
            selected_rank_by_function.insert(function_id.clone(), candidate.rank);
        } else {
            blocked.insert(function_id.clone());
            independence_broken.insert(function_id.clone());
        }
    }

    (plan, selected_rank_by_function, blocked)
}

fn merge_actions_without_conflict(
    plan: &mut BTreeMap<PathBuf, Vec<FixAction>>,
    actions: &[FixAction],
) -> bool {
    let mut next = plan.clone();
    for action in actions {
        let file = action_file(action);
        next.entry(file).or_default().push(action.clone());
    }

    for file_actions in next.values_mut() {
        file_actions.sort_by(|a, b| {
            let (a_start, a_end) = action_range(a);
            let (b_start, b_end) = action_range(b);
            a_start.cmp(&b_start).then_with(|| a_end.cmp(&b_end))
        });
        if CodeTransformer::ensure_no_conflicts(file_actions).is_err() {
            return false;
        }
    }

    *plan = next;
    true
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

fn plan_signature(plan: &BTreeMap<PathBuf, Vec<FixAction>>) -> String {
    let mut parts = Vec::new();
    for (file, actions) in plan {
        let mut tokens = Vec::new();
        for action in actions {
            let (start, end) = action_range(action);
            let token = match action {
                FixAction::Replace { new_content, .. } => {
                    format!("R:{start}:{end}:{new_content}")
                }
                FixAction::Insert { content, .. } => format!("I:{start}:{end}:{content}"),
                FixAction::Delete { .. } => format!("D:{start}:{end}"),
            };
            tokens.push(token);
        }
        parts.push(format!("{}|{}", file.display(), tokens.join(",")));
    }
    parts.join(";")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruter::core::SpanInfo;

    fn action(start: usize, end: usize, content: &str) -> FixAction {
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

    fn candidate(
        function_id: &str,
        rank: usize,
        start: usize,
        end: usize,
        content: &str,
    ) -> FunctionRuleCandidate {
        FunctionRuleCandidate {
            function_id: function_id.to_string(),
            rank,
            score: 0,
            actions: vec![action(start, end, content)],
            diagnostic_trace: vec![],
            zip_alignment_meta: Default::default(),
        }
    }

    #[test]
    fn optimistic_round_batch_downgrade() {
        let mut by_function = BTreeMap::new();
        by_function.insert(
            "f1".to_string(),
            vec![
                candidate("f1", 1, 0, 1, "a1"),
                candidate("f1", 2, 0, 1, "a2"),
            ],
        );
        by_function.insert(
            "f2".to_string(),
            vec![
                candidate("f2", 1, 10, 11, "b1"),
                candidate("f2", 2, 10, 11, "b2"),
            ],
        );
        let targets = BTreeSet::from(["f1".to_string(), "f2".to_string()]);

        let mut call = 0usize;
        let result = run_optimistic_greedy(&by_function, &targets, 2, |_plan| {
            call += 1;
            if call == 1 {
                Ok(RoundEvaluation {
                    check_error_total: 1,
                    check_error_by_code: BTreeMap::new(),
                    unresolved_function_ids: BTreeSet::from(["f1".to_string(), "f2".to_string()]),
                })
            } else {
                Ok(RoundEvaluation {
                    check_error_total: 0,
                    check_error_by_code: BTreeMap::new(),
                    unresolved_function_ids: BTreeSet::new(),
                })
            }
        })
        .unwrap();

        assert_eq!(result.rounds.len(), 2);
        assert!(
            result
                .rounds
                .last()
                .map(|round| round.unresolved_function_ids.is_empty())
                .unwrap_or(false)
        );
    }

    #[test]
    fn independence_broken_goes_to_handoff() {
        let mut by_function = BTreeMap::new();
        by_function.insert("f1".to_string(), vec![candidate("f1", 1, 0, 2, "a1")]);
        by_function.insert(
            "f2".to_string(),
            vec![candidate("f2", 1, 1, 3, "b1")], // overlap with f1
        );
        let targets = BTreeSet::from(["f1".to_string(), "f2".to_string()]);

        let result = run_optimistic_greedy(&by_function, &targets, 1, |_plan| {
            Ok(RoundEvaluation {
                check_error_total: 1,
                check_error_by_code: BTreeMap::new(),
                unresolved_function_ids: BTreeSet::from(["f2".to_string()]),
            })
        })
        .unwrap();

        assert!(result.independence_broken_function_ids.contains("f2"));
        assert!(
            result
                .rounds
                .last()
                .map(|round| {
                    round
                        .unresolved_function_ids
                        .iter()
                        .any(|function_id| function_id == "f2")
                })
                .unwrap_or(false)
        );
    }
}
