use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use ruter::coordinator::GlobalCandidatePlan;
use ruter::core::{FixAction, SpanInfo};
use ruter::transformer::CodeTransformer;

use crate::runtime::artifacts::ArtifactPaths;
use crate::runtime::function::index::{FunctionDiagnostic, FunctionIndex};
use crate::runtime::reporter::Reporter;

use super::verify_engine;
use super::{
    ApplyStageResult, CandidateFunctionReport, PartialUnionPlanResult, PartialUnionVerifyResult,
    PreparedPatch,
};

mod candidate_verify;

pub use candidate_verify::{prepare_patch_stage, verify_patch_stage, verify_topk_candidates_stage};

pub fn compose_function_union_plan(
    crate_path: &Path,
    candidates: &[GlobalCandidatePlan],
    candidate_reports: &[CandidateFunctionReport],
    target_function_ids: &BTreeSet<String>,
    reporter: &mut Reporter,
) -> Result<PartialUnionPlanResult> {
    let index = FunctionIndex::build(crate_path)?;
    let candidate_resolved_map: HashMap<String, BTreeSet<String>> = candidate_reports
        .iter()
        .map(|report| {
            (
                report.candidate_id.clone(),
                report.resolved_function_ids.iter().cloned().collect(),
            )
        })
        .collect();

    let mut actions_by_candidate_and_function: HashMap<String, BTreeMap<String, Vec<FixAction>>> =
        HashMap::new();
    for candidate in candidates {
        actions_by_candidate_and_function.insert(
            candidate.candidate_id.clone(),
            index.actions_grouped_by_function(&candidate.plan, crate_path),
        );
    }

    let mut selected_candidate_for_function: Vec<(String, String, i32)> = Vec::new();
    let mut unresolved = target_function_ids.clone();

    for function_id in target_function_ids {
        let mut available = Vec::new();
        for candidate in candidates {
            let Some(resolved_ids) = candidate_resolved_map.get(&candidate.candidate_id) else {
                continue;
            };
            if !resolved_ids.contains(function_id) {
                continue;
            }
            let has_actions = actions_by_candidate_and_function
                .get(&candidate.candidate_id)
                .and_then(|map| map.get(function_id))
                .map(|actions| !actions.is_empty())
                .unwrap_or(false);
            if !has_actions {
                continue;
            }
            available.push((candidate.candidate_id.clone(), candidate.score));
        }

        available.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        if let Some((candidate_id, score)) = available.first() {
            selected_candidate_for_function.push((
                function_id.clone(),
                candidate_id.clone(),
                *score,
            ));
        }
    }

    selected_candidate_for_function.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    let mut union_plan: BTreeMap<PathBuf, Vec<FixAction>> = BTreeMap::new();
    let mut resolved = BTreeSet::new();
    for (function_id, candidate_id, _) in selected_candidate_for_function {
        let Some(action_map) = actions_by_candidate_and_function.get(&candidate_id) else {
            continue;
        };
        let Some(actions) = action_map.get(&function_id) else {
            continue;
        };

        if merge_actions_without_conflict(&mut union_plan, actions) {
            resolved.insert(function_id.clone());
            unresolved.remove(&function_id);
        }
    }

    reporter.section("partial union plan");
    reporter.kv(
        0,
        "target_test_function_count",
        target_function_ids.len().to_string(),
    );
    reporter.kv(
        0,
        "resolved_test_function_count",
        resolved.len().to_string(),
    );
    reporter.kv(
        0,
        "unresolved_test_function_count",
        unresolved.len().to_string(),
    );
    reporter.kv(0, "partial_plan_file_count", union_plan.len().to_string());
    reporter.kv(
        0,
        "partial_plan_action_count",
        union_plan.values().map(Vec::len).sum::<usize>().to_string(),
    );
    for report in candidate_reports {
        reporter.item(
            0,
            format!(
                "{} independent_resolved_test_function_count={} independent_unresolved_test_function_count={}",
                report.candidate_id,
                report.resolved_function_ids.len(),
                report.unresolved_function_ids.len()
            ),
        );
    }

    // Keep unresolved entries for candidates that never resolved this function.
    for function_id in target_function_ids {
        if resolved.contains(function_id) {
            continue;
        }
        let can_resolve = candidate_reports.iter().any(|report| {
            report
                .resolved_function_ids
                .iter()
                .any(|resolved_id| resolved_id == function_id)
        });
        if !can_resolve {
            unresolved.insert(function_id.clone());
        }
    }

    Ok(PartialUnionPlanResult { plan: union_plan })
}

pub fn verify_partial_union_plan(
    crate_path: &Path,
    partial_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    target_function_ids: &BTreeSet<String>,
    artifacts: &ArtifactPaths,
    keep_updated_sources: bool,
    reporter: &mut Reporter,
) -> Result<PartialUnionVerifyResult> {
    verify_partial_union_plan_with_tag(
        crate_path,
        partial_plan,
        target_function_ids,
        artifacts,
        keep_updated_sources,
        "partial_union",
        reporter,
    )
}

pub fn verify_partial_union_plan_with_tag(
    crate_path: &Path,
    partial_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    target_function_ids: &BTreeSet<String>,
    artifacts: &ArtifactPaths,
    keep_updated_sources: bool,
    attempt_tag: &str,
    reporter: &mut Reporter,
) -> Result<PartialUnionVerifyResult> {
    let partial_updated_dir = artifacts.updated_sources_dir.join("partial_union");
    let partial_diff_path = artifacts.default_diff_file.clone();
    let prepared = prepare_patch_stage(
        crate_path,
        partial_plan,
        &partial_diff_path,
        &partial_updated_dir,
        keep_updated_sources,
        reporter,
    )?;

    let verify_run =
        verify_engine::run_check_with_updated_sources(crate_path, &prepared.updated_sources)?;
    let partial_attempt_dir = artifacts.verify_attempts_dir.join(attempt_tag);
    fs::create_dir_all(&partial_attempt_dir)?;
    let check_stdout_log = partial_attempt_dir.join("check.stdout.log");
    let check_stderr_log = partial_attempt_dir.join("check.stderr.log");
    super::write_command_logs(
        &verify_run.check_output,
        &check_stdout_log,
        &check_stderr_log,
    )?;
    super::report_stderr_snippet("verify_partial_union", &verify_run.check_output, reporter);

    let root_index = FunctionIndex::build(crate_path)?;
    let error_diagnostics_by_function = root_index
        .error_diagnostics_by_function(&verify_run.check_diagnostics, &verify_run.temp_workspace);
    let error_by_function = collapse_function_errors(&error_diagnostics_by_function);
    let mut unresolved_function_ids =
        unresolved_target_functions(target_function_ids, &error_by_function);
    if verify_run.check_error_stats.error_total > 0 && unresolved_function_ids.is_empty() {
        unresolved_function_ids.insert("__UNMAPPED_ERRORS__".to_string());
    }

    let _ = fs::remove_dir_all(&verify_run.temp_workspace);

    let resolved_function_ids: BTreeSet<String> = target_function_ids
        .difference(&unresolved_function_ids)
        .cloned()
        .collect();

    reporter.section("partial union verify");
    reporter.kv(
        0,
        "check_passed",
        verify_run.check_output.status.success().to_string(),
    );
    reporter.kv(
        0,
        "resolved_test_function_count",
        resolved_function_ids.len().to_string(),
    );
    reporter.kv(
        0,
        "unresolved_test_function_count",
        unresolved_function_ids.len().to_string(),
    );
    reporter.kv(
        0,
        "remaining_error_total",
        verify_run.check_error_stats.error_total.to_string(),
    );

    Ok(PartialUnionVerifyResult {
        prepared,
        resolved_function_ids,
        unresolved_function_ids,
        check_error_total: verify_run.check_error_stats.error_total,
        check_error_by_code: verify_run.check_error_stats.error_by_code,
        error_diagnostics_by_function,
        check_stdout_log,
        check_stderr_log,
    })
}

pub fn apply_stage(
    crate_path: &Path,
    prepared: &PreparedPatch,
    apply: bool,
    backup_enabled: bool,
    backups_dir: &Path,
    reporter: &mut Reporter,
) -> Result<ApplyStageResult> {
    reporter.section("apply plan");
    reporter.kv(
        0,
        "patch_apply_mode",
        if apply { "apply" } else { "dry-run" },
    );
    reporter.kv(
        0,
        "patch_candidate_file_count",
        prepared.changed_files.len().to_string(),
    );

    if apply {
        if backup_enabled {
            super::write_backups(
                crate_path,
                backups_dir,
                &prepared.changed_files,
                &prepared.source_map,
            )?;
            reporter.kv(0, "backup_dir", backups_dir.display().to_string());
        }

        super::write_files_with_rollback(
            crate_path,
            backups_dir,
            backup_enabled,
            &prepared.changed_files,
            &prepared.updated_sources,
            reporter,
        )?;
        reporter.kv(0, "apply_status", "committed");
    } else {
        reporter.kv(0, "apply_status", "skipped (dry-run)");
    }

    Ok(ApplyStageResult {
        applied_file_count: if apply {
            prepared.changed_files.len()
        } else {
            0
        },
        patch_applied: apply,
    })
}

pub fn ensure_artifact_exists(path: &Path, stage: &str) -> Result<()> {
    if !path.exists() {
        bail!(
            "required artifact missing for stage {stage}: {}",
            path.display()
        );
    }
    Ok(())
}

fn collapse_function_errors(
    error_diags_by_function: &BTreeMap<String, Vec<FunctionDiagnostic>>,
) -> BTreeMap<String, BTreeMap<String, usize>> {
    let mut out: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for (function_id, diags) in error_diags_by_function {
        let mut code_counts = BTreeMap::new();
        for diag in diags {
            *code_counts.entry(diag.code.clone()).or_insert(0) += 1;
        }
        out.insert(function_id.clone(), code_counts);
    }
    out
}

fn unresolved_target_functions(
    target_function_ids: &BTreeSet<String>,
    error_by_function: &BTreeMap<String, BTreeMap<String, usize>>,
) -> BTreeSet<String> {
    let mut unresolved = BTreeSet::new();
    for function_id in target_function_ids {
        if error_by_function.contains_key(function_id) {
            unresolved.insert(function_id.clone());
        }
    }
    unresolved
}

fn merge_actions_without_conflict(
    union_plan: &mut BTreeMap<PathBuf, Vec<FixAction>>,
    actions: &[FixAction],
) -> bool {
    let mut next_plan = union_plan.clone();
    for action in actions {
        let file_path = match action {
            FixAction::Insert { span, .. }
            | FixAction::Replace { span, .. }
            | FixAction::Delete { span } => span.file_path.clone(),
        };
        next_plan.entry(file_path).or_default().push(action.clone());
    }

    for file_actions in next_plan.values_mut() {
        file_actions.sort_by(|a, b| {
            let (a_start, a_end) = action_range(a);
            let (b_start, b_end) = action_range(b);
            a_start.cmp(&b_start).then_with(|| a_end.cmp(&b_end))
        });
        if CodeTransformer::ensure_no_conflicts(file_actions).is_err() {
            return false;
        }
    }

    *union_plan = next_plan;
    true
}

fn action_range(action: &FixAction) -> (usize, usize) {
    match action {
        FixAction::Insert { span, .. }
        | FixAction::Replace { span, .. }
        | FixAction::Delete { span } => (span.byte_start, span.byte_end),
    }
}

#[allow(dead_code)]
fn action_span(action: &FixAction) -> Option<&SpanInfo> {
    match action {
        FixAction::Insert { span, .. }
        | FixAction::Replace { span, .. }
        | FixAction::Delete { span } => Some(span),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn function_diag(code: &str) -> FunctionDiagnostic {
        FunctionDiagnostic {
            code: code.to_string(),
            message: format!("synthetic {code}"),
            primary_span: None,
            label: None,
            suggested_replacement: None,
            children_note_messages: Vec::new(),
            children_help_messages: Vec::new(),
            children_suggested_replacements: Vec::new(),
        }
    }

    #[test]
    fn collapse_function_errors_counts_e0063_and_e0451() {
        let mut by_function = BTreeMap::new();
        by_function.insert(
            "tests::case::f:1:10".to_string(),
            vec![
                function_diag("E0063"),
                function_diag("E0451"),
                function_diag("E0451"),
            ],
        );

        let collapsed = collapse_function_errors(&by_function);
        let counts = collapsed
            .get("tests::case::f:1:10")
            .expect("counts for target function");
        assert_eq!(counts.get("E0063"), Some(&1));
        assert_eq!(counts.get("E0451"), Some(&2));
    }

    #[test]
    fn unresolved_target_functions_marks_target_for_e0063_or_e0451() {
        let target_ids = BTreeSet::from(["tests::case::f:1:10".to_string()]);
        let mut error_by_function = BTreeMap::new();
        error_by_function.insert(
            "tests::case::f:1:10".to_string(),
            BTreeMap::from([("E0063".to_string(), 1usize), ("E0451".to_string(), 1usize)]),
        );

        let unresolved = unresolved_target_functions(&target_ids, &error_by_function);
        assert!(unresolved.contains("tests::case::f:1:10"));
    }
}
