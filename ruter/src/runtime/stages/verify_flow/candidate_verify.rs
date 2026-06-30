use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use ruter::coordinator::GlobalCandidatePlan;
use ruter::coordinator::PatchCoordinator;
use ruter::core::{Diagnostic, FixAction};
use ruter::patchers::PatcherRegistry;
use ruter::transformer::CodeTransformer;

use crate::runtime::artifacts::ArtifactPaths;
use crate::runtime::function::index::FunctionIndex;
use crate::runtime::reporter::Reporter;

use super::super::verify_engine;
use super::super::{
    CandidateFunctionReport, PreparedPatch, VerifyAttempt, VerifyStageResult,
    VerifyTopKCandidatesResult,
};
use super::{collapse_function_errors, unresolved_target_functions};

pub fn prepare_patch_stage(
    crate_path: &Path,
    plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    diff_path: &Path,
    updated_sources_dir: &Path,
    keep_updated_sources: bool,
    reporter: &mut Reporter,
) -> Result<PreparedPatch> {
    let coordinator = PatchCoordinator::new(PatcherRegistry::new(), CodeTransformer::new());
    let mut source_map = HashMap::new();

    for file in plan.keys() {
        let source = fs::read_to_string(file)
            .with_context(|| format!("failed to read source {}", file.display()))?;
        source_map.insert(file.clone(), source);
    }

    let updated_sources = coordinator
        .apply_planned(plan, &source_map)
        .context("failed to apply planned replacements")?;

    let changed_files = super::super::changed_files(&source_map, &updated_sources);
    let diff_content = super::super::render_diff(crate_path, &source_map, &updated_sources);

    if let Some(parent) = diff_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(diff_path, diff_content)?;

    if keep_updated_sources {
        super::super::write_updated_sources(crate_path, updated_sources_dir, &updated_sources)?;
    }

    reporter.section("patch preview");
    reporter.kv(
        0,
        "patch_candidate_file_count",
        changed_files.len().to_string(),
    );
    reporter.kv(0, "diff_file", diff_path.display().to_string());
    reporter.kv(
        0,
        "updated_sources_dir",
        if keep_updated_sources {
            updated_sources_dir.display().to_string()
        } else {
            "disabled (pass --keep-updated-sources to enable)".to_string()
        },
    );

    Ok(PreparedPatch {
        source_map,
        updated_sources,
        changed_files,
        diff_path: diff_path.to_path_buf(),
    })
}

pub fn verify_patch_stage(
    crate_path: &Path,
    prepared: &PreparedPatch,
    run_tests: bool,
    artifacts: &ArtifactPaths,
    reporter: &mut Reporter,
) -> Result<VerifyStageResult> {
    let strategy = "verify patched sources in isolated temp workspace".to_string();
    let check_command = "cargo check --tests --message-format=json".to_string();
    let tests_command = run_tests.then(|| "cargo test".to_string());

    reporter.section("verify strategy");
    reporter.kv(0, "strategy", &strategy);
    reporter.kv(0, "verification_target", "planned patch candidate");
    reporter.kv(0, "check_command", &check_command);
    if let Some(test_cmd) = tests_command.as_ref() {
        reporter.kv(0, "tests_command", test_cmd);
    }

    let verify_run =
        verify_engine::run_check_with_updated_sources(crate_path, &prepared.updated_sources)?;
    reporter.kv(
        0,
        "temp_workspace",
        verify_run.temp_workspace.display().to_string(),
    );

    super::super::write_command_logs(
        &verify_run.check_output,
        &artifacts.verify_check_stdout_log,
        &artifacts.verify_check_stderr_log,
    )?;
    super::super::report_stderr_snippet("verify_check", &verify_run.check_output, reporter);
    let check_passed = verify_run.check_output.status.success();

    let mut tests_passed = None;
    if check_passed && run_tests {
        let test = Command::new("cargo")
            .arg("test")
            .current_dir(&verify_run.temp_workspace)
            .output()
            .with_context(|| {
                format!(
                    "failed to run cargo test in {}",
                    verify_run.temp_workspace.display()
                )
            })?;
        super::super::write_command_logs(
            &test,
            &artifacts.verify_test_stdout_log,
            &artifacts.verify_test_stderr_log,
        )?;
        super::super::report_stderr_snippet("verify_test", &test, reporter);
        tests_passed = Some(test.status.success());
    }

    let patch_usable = check_passed && tests_passed != Some(false);
    let preserved_workspace = if patch_usable {
        None
    } else {
        Some(super::super::preserve_failed_workspace(
            &verify_run.temp_workspace,
            &artifacts.verify_failed_workspace_dir,
        )?)
    };

    let _ = fs::remove_dir_all(&verify_run.temp_workspace);

    let result = VerifyStageResult {
        strategy,
        check_command,
        tests_command,
        temp_workspace: verify_run.temp_workspace.clone(),
        preserved_workspace: preserved_workspace.clone(),
        check_passed,
        tests_passed,
        patch_usable,
        check_stdout_log: artifacts.verify_check_stdout_log.clone(),
        check_stderr_log: artifacts.verify_check_stderr_log.clone(),
        tests_stdout_log: run_tests.then(|| artifacts.verify_test_stdout_log.clone()),
        tests_stderr_log: run_tests.then(|| artifacts.verify_test_stderr_log.clone()),
        check_error_total: verify_run.check_error_stats.error_total,
        check_error_by_code: verify_run.check_error_stats.error_by_code.clone(),
        selected_candidate_id: Some("single-plan".to_string()),
        attempts: vec![VerifyAttempt {
            candidate_id: "single-plan".to_string(),
            check_passed,
            check_error_total: verify_run.check_error_stats.error_total,
            check_error_by_code: verify_run.check_error_stats.error_by_code.clone(),
            check_stdout_log: artifacts.verify_check_stdout_log.clone(),
            check_stderr_log: artifacts.verify_check_stderr_log.clone(),
            preserved_workspace: preserved_workspace.clone(),
        }],
        target_test_function_count: 0,
        resolved_test_function_count: 0,
        unresolved_test_function_count: 0,
        unresolved_function_ids: Vec::new(),
        dispatch_target_function_count: 0,
        rule_function_count: 0,
        llm_routed_function_count: 0,
        optimistic_round_count: 0,
        independence_broken_function_ids: Vec::new(),
    };
    super::super::write_verify_report(&artifacts.verify_report_json, &result)?;

    reporter.section("verify result");
    reporter.kv(0, "check_passed", result.check_passed.to_string());
    reporter.kv(
        0,
        "remaining_error_total",
        result.check_error_total.to_string(),
    );
    reporter.kv(
        0,
        "remaining_error_by_code",
        super::super::format_error_code_counts(&result.check_error_by_code),
    );
    reporter.kv(
        0,
        "tests_passed",
        result
            .tests_passed
            .map(|s| s.to_string())
            .unwrap_or_else(|| "not-run".to_string()),
    );
    reporter.kv(0, "patch_usable", result.patch_usable.to_string());
    reporter.kv(
        0,
        "verify_report",
        artifacts.verify_report_json.display().to_string(),
    );
    if let Some(path) = result.preserved_workspace.as_ref() {
        reporter.kv(0, "preserved_failed_workspace", path.display().to_string());
    }

    Ok(result)
}

pub fn verify_topk_candidates_stage(
    crate_path: &Path,
    candidates: &[GlobalCandidatePlan],
    initial_diagnostics: &[Diagnostic],
    artifacts: &ArtifactPaths,
    keep_updated_sources: bool,
    reporter: &mut Reporter,
) -> Result<VerifyTopKCandidatesResult> {
    let root_index = FunctionIndex::build(crate_path)?;
    let target_function_ids =
        root_index.target_function_ids_for_errors(initial_diagnostics, crate_path);

    let strategy = "verify top-k candidate plans in isolated temp workspace".to_string();
    let check_command = "cargo check --tests --message-format=json".to_string();

    reporter.section("verify strategy");
    reporter.kv(0, "strategy", &strategy);
    reporter.kv(0, "verification_target", "top-k patch candidates");
    reporter.kv(0, "check_command", &check_command);
    reporter.kv(0, "candidate_count", candidates.len().to_string());
    reporter.kv(
        0,
        "target_test_function_count",
        target_function_ids.len().to_string(),
    );

    let mut attempts = Vec::new();
    let mut selected_prepared = None;
    let mut selected_candidate_id = None;
    let mut selected_check_error_total = 0usize;
    let mut selected_check_error_by_code = BTreeMap::new();
    let mut selected_unresolved_ids: BTreeSet<String> = BTreeSet::new();
    let mut candidate_function_reports = Vec::new();

    for candidate in candidates {
        let attempt_dir = artifacts.verify_attempts_dir.join(&candidate.candidate_id);
        fs::create_dir_all(&attempt_dir)?;
        let candidate_diff_path = attempt_dir.join("4_changes.diff");
        let candidate_updated_dir = artifacts.updated_sources_dir.join(&candidate.candidate_id);
        let candidate_failed_workspace_dir = attempt_dir.join("4_failed_workspace");

        let prepared = prepare_patch_stage(
            crate_path,
            &candidate.plan,
            &candidate_diff_path,
            &candidate_updated_dir,
            keep_updated_sources,
            reporter,
        )?;

        let verify_run =
            verify_engine::run_check_with_updated_sources(crate_path, &prepared.updated_sources)?;
        let error_diags_by_function = root_index.error_diagnostics_by_function(
            &verify_run.check_diagnostics,
            &verify_run.temp_workspace,
        );
        let error_by_function = collapse_function_errors(&error_diags_by_function);
        let unresolved_ids = unresolved_target_functions(&target_function_ids, &error_by_function);
        let resolved_ids: BTreeSet<String> = target_function_ids
            .difference(&unresolved_ids)
            .cloned()
            .collect();
        let check_stdout_log = attempt_dir.join("check.stdout.log");
        let check_stderr_log = attempt_dir.join("check.stderr.log");
        super::super::write_command_logs(
            &verify_run.check_output,
            &check_stdout_log,
            &check_stderr_log,
        )?;
        super::super::report_stderr_snippet(
            &format!("verify_topk_{}", candidate.candidate_id),
            &verify_run.check_output,
            reporter,
        );
        let check_passed = verify_run.check_output.status.success();

        let preserved_workspace = if check_passed {
            None
        } else {
            Some(super::super::preserve_failed_workspace(
                &verify_run.temp_workspace,
                &candidate_failed_workspace_dir,
            )?)
        };
        let _ = fs::remove_dir_all(&verify_run.temp_workspace);

        reporter.item(
            0,
            format!(
                "{} check_passed={} remaining_error_total={} resolved_test_function_count={} unresolved_test_function_count={}",
                candidate.candidate_id,
                check_passed,
                verify_run.check_error_stats.error_total,
                resolved_ids.len(),
                unresolved_ids.len()
            ),
        );

        attempts.push(VerifyAttempt {
            candidate_id: candidate.candidate_id.clone(),
            check_passed,
            check_error_total: verify_run.check_error_stats.error_total,
            check_error_by_code: verify_run.check_error_stats.error_by_code.clone(),
            check_stdout_log,
            check_stderr_log,
            preserved_workspace,
        });
        candidate_function_reports.push(CandidateFunctionReport {
            candidate_id: candidate.candidate_id.clone(),
            score: candidate.score,
            check_passed,
            resolved_function_ids: resolved_ids.iter().cloned().collect(),
            unresolved_function_ids: unresolved_ids.iter().cloned().collect(),
            error_by_function,
        });

        if check_passed {
            selected_check_error_total = verify_run.check_error_stats.error_total;
            selected_check_error_by_code = verify_run.check_error_stats.error_by_code;
            selected_candidate_id = Some(candidate.candidate_id.clone());
            selected_prepared = Some(prepared);
            selected_unresolved_ids = unresolved_ids;
            break;
        }
    }

    let patch_usable = selected_candidate_id.is_some();
    let (resolved_count, unresolved_function_ids) = if patch_usable {
        let unresolved: Vec<String> = selected_unresolved_ids.iter().cloned().collect();
        (
            target_function_ids.len().saturating_sub(unresolved.len()),
            unresolved,
        )
    } else if let Some(best) = candidate_function_reports.iter().max_by(|a, b| {
        a.resolved_function_ids
            .len()
            .cmp(&b.resolved_function_ids.len())
            .then_with(|| a.score.cmp(&b.score))
            .then_with(|| b.candidate_id.cmp(&a.candidate_id))
    }) {
        (
            best.resolved_function_ids.len(),
            best.unresolved_function_ids.clone(),
        )
    } else {
        (0usize, target_function_ids.iter().cloned().collect())
    };

    let result = VerifyStageResult {
        strategy,
        check_command,
        tests_command: None,
        temp_workspace: PathBuf::new(),
        preserved_workspace: attempts
            .iter()
            .rev()
            .find_map(|attempt| attempt.preserved_workspace.clone()),
        check_passed: patch_usable,
        tests_passed: None,
        patch_usable,
        check_stdout_log: artifacts.verify_check_stdout_log.clone(),
        check_stderr_log: artifacts.verify_check_stderr_log.clone(),
        tests_stdout_log: None,
        tests_stderr_log: None,
        check_error_total: selected_check_error_total,
        check_error_by_code: selected_check_error_by_code,
        selected_candidate_id: selected_candidate_id.clone(),
        attempts,
        target_test_function_count: target_function_ids.len(),
        resolved_test_function_count: resolved_count,
        unresolved_test_function_count: unresolved_function_ids.len(),
        unresolved_function_ids,
        dispatch_target_function_count: 0,
        rule_function_count: 0,
        llm_routed_function_count: 0,
        optimistic_round_count: 0,
        independence_broken_function_ids: Vec::new(),
    };
    super::super::write_verify_report(&artifacts.verify_report_json, &result)?;

    reporter.section("verify result");
    reporter.kv(0, "check_passed", result.check_passed.to_string());
    reporter.kv(0, "patch_usable", result.patch_usable.to_string());
    reporter.kv(
        0,
        "selected_candidate_id",
        result
            .selected_candidate_id
            .clone()
            .unwrap_or_else(|| "none".to_string()),
    );
    reporter.kv(0, "attempt_count", result.attempts.len().to_string());
    reporter.kv(
        0,
        "verify_report",
        artifacts.verify_report_json.display().to_string(),
    );

    Ok(VerifyTopKCandidatesResult {
        verify: result,
        selected_prepared,
        candidate_function_reports,
        target_function_ids,
    })
}
