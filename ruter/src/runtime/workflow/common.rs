use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ruter::core::FixAction;

use crate::config::ResolvedConfig;
use crate::llm::handoff::build_llm_handoff_items;
use crate::runtime::artifacts::{ArtifactPaths, RunSummary, write_json, write_plan};
use crate::runtime::function::index::FunctionIndex;
use crate::runtime::reporter::Reporter;
use crate::runtime::stages::{
    PartialUnionVerifyResult, VerifyStageResult, format_error_code_counts,
};

pub(super) struct PartialPlanOutcome {
    pub plan: BTreeMap<PathBuf, Vec<FixAction>>,
    pub partial_verify: PartialUnionVerifyResult,
    pub llm_handoff_count: usize,
    pub preflight_skipped_llm_count: usize,
    pub preflight_skipped_llm_by_code: BTreeMap<String, usize>,
}

pub(super) fn maybe_run_llm_stage(
    resolved: &ResolvedConfig,
    crate_path: &Path,
    artifacts: &ArtifactPaths,
    target_function_ids: &BTreeSet<String>,
    plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    partial_verify: PartialUnionVerifyResult,
    reporter: &mut Reporter,
) -> Result<(
    BTreeMap<PathBuf, Vec<FixAction>>,
    PartialUnionVerifyResult,
    usize,
    BTreeMap<String, usize>,
)> {
    if !resolved.llm.enabled {
        return Ok((plan.clone(), partial_verify, 0, BTreeMap::new()));
    }

    let llm_result = crate::llm::executor::run_llm_stage(
        crate_path,
        artifacts,
        target_function_ids,
        plan,
        partial_verify,
        &resolved.llm,
        reporter,
    )?;
    write_json(&artifacts.llm_attempts_json, &llm_result.attempts_artifact)
        .context("failed to write llm attempts artifact")?;
    Ok((
        llm_result.plan,
        llm_result.partial_verify,
        llm_result.preflight_skipped_llm_count,
        llm_result.preflight_skipped_llm_by_code,
    ))
}

pub(super) fn complete_partial_plan_path(
    resolved: &ResolvedConfig,
    crate_path: &Path,
    artifacts: &ArtifactPaths,
    target_function_ids: &BTreeSet<String>,
    initial_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    initial_partial_verify: PartialUnionVerifyResult,
    function_index: &FunctionIndex,
    base_verify: Option<&VerifyStageResult>,
    reporter: &mut Reporter,
) -> Result<PartialPlanOutcome> {
    let (active_plan, partial_verify, preflight_skipped_llm_count, preflight_skipped_llm_by_code) =
        maybe_run_llm_stage(
            resolved,
            crate_path,
            artifacts,
            target_function_ids,
            initial_plan,
            initial_partial_verify,
            reporter,
        )?;

    write_plan(&artifacts.partial_plan_json, &active_plan)
        .context("failed to rewrite partial plan artifact")?;

    let llm_handoff = build_llm_handoff_items(
        function_index,
        &partial_verify.unresolved_function_ids,
        &partial_verify.error_diagnostics_by_function,
        &active_plan,
        &partial_verify.prepared,
    );
    write_json(&artifacts.llm_handoff_json, &llm_handoff)
        .context("failed to write llm handoff artifact")?;
    let partial_report =
        build_partial_verify_report(base_verify, target_function_ids, &partial_verify);
    write_json(&artifacts.verify_report_json, &partial_report)
        .context("failed to write partial verify report")?;

    Ok(PartialPlanOutcome {
        plan: active_plan,
        partial_verify,
        llm_handoff_count: llm_handoff.len(),
        preflight_skipped_llm_count,
        preflight_skipped_llm_by_code,
    })
}

pub(super) fn build_partial_verify_report(
    base: Option<&VerifyStageResult>,
    target_function_ids: &BTreeSet<String>,
    partial_verify: &PartialUnionVerifyResult,
) -> VerifyStageResult {
    let check_stdout_log = base
        .map(|value| value.check_stdout_log.clone())
        .unwrap_or_default();
    let check_stderr_log = base
        .map(|value| value.check_stderr_log.clone())
        .unwrap_or_default();
    let unresolved_function_ids: Vec<String> = partial_verify
        .unresolved_function_ids
        .iter()
        .cloned()
        .collect();

    let mut report = base.cloned().unwrap_or(VerifyStageResult {
        strategy: "verify partial-union patch in isolated temp workspace".to_string(),
        check_command: "cargo check --tests --message-format=json".to_string(),
        tests_command: None,
        temp_workspace: PathBuf::new(),
        preserved_workspace: None,
        check_passed: partial_verify.unresolved_function_ids.is_empty(),
        tests_passed: None,
        patch_usable: partial_verify.unresolved_function_ids.is_empty(),
        check_stdout_log: check_stdout_log.clone(),
        check_stderr_log: check_stderr_log.clone(),
        tests_stdout_log: None,
        tests_stderr_log: None,
        check_error_total: partial_verify.check_error_total,
        check_error_by_code: partial_verify.check_error_by_code.clone(),
        selected_candidate_id: Some("partial-union".to_string()),
        attempts: Vec::new(),
        target_test_function_count: target_function_ids.len(),
        resolved_test_function_count: partial_verify.resolved_function_ids.len(),
        unresolved_test_function_count: partial_verify.unresolved_function_ids.len(),
        unresolved_function_ids: unresolved_function_ids.clone(),
        dispatch_target_function_count: target_function_ids.len(),
        rule_function_count: 0,
        llm_routed_function_count: 0,
        optimistic_round_count: 0,
        independence_broken_function_ids: Vec::new(),
    });

    report.strategy = "verify partial-union patch in isolated temp workspace".to_string();
    report.check_passed = partial_verify.unresolved_function_ids.is_empty();
    report.patch_usable = partial_verify.unresolved_function_ids.is_empty();
    report.check_error_total = partial_verify.check_error_total;
    report.check_error_by_code = partial_verify.check_error_by_code.clone();
    report.selected_candidate_id = Some("partial-union".to_string());
    report.check_stdout_log = check_stdout_log;
    report.check_stderr_log = check_stderr_log;
    report.target_test_function_count = target_function_ids.len();
    report.resolved_test_function_count = partial_verify.resolved_function_ids.len();
    report.unresolved_test_function_count = partial_verify.unresolved_function_ids.len();
    report.unresolved_function_ids = unresolved_function_ids;
    report.dispatch_target_function_count = target_function_ids.len();
    report
}

pub(super) fn report_run_summary(reporter: &mut Reporter, exit_code: u8, summary: &RunSummary) {
    reporter.section("run summary");
    let total_pending_function_count =
        summary.resolved_test_function_count + summary.unresolved_test_function_count;

    reporter.section_colored("compile-and-repair-stats", "cyan");
    reporter.kv(0, "exit_code", exit_code.to_string());
    reporter.kv(
        0,
        "initial_compile_passed",
        summary.initial_compile_passed.to_string(),
    );
    reporter.kv(0, "diagnostic_count", summary.diagnostic_count.to_string());
    reporter.kv(
        0,
        "pending_error_total",
        summary.initial_error_total.to_string(),
    );
    reporter.kv(
        0,
        "remaining_error_total",
        summary.remaining_error_total.to_string(),
    );
    reporter.kv(
        0,
        "error_function_total",
        total_pending_function_count.to_string(),
    );
    reporter.kv(
        0,
        "remaining_error_function_count",
        summary.unresolved_test_function_count.to_string(),
    );
    reporter.kv(
        0,
        "fixed_error_function_count",
        summary.resolved_test_function_count.to_string(),
    );
    reporter.kv(
        0,
        "pending_error_by_code",
        format_error_code_counts(&summary.initial_error_by_code),
    );
    reporter.kv(
        0,
        "remaining_error_by_code",
        format_error_code_counts(&summary.remaining_error_by_code),
    );
    reporter.kv(
        0,
        "partial_pending_llm",
        summary.partial_pending_llm.to_string(),
    );
    reporter.kv(
        0,
        "llm_handoff_count",
        summary.llm_handoff_count.to_string(),
    );
    reporter.kv(
        0,
        "preflight_skipped_llm_count",
        summary.preflight_skipped_llm_count.to_string(),
    );
    reporter.kv(
        0,
        "preflight_skipped_llm_by_code",
        format_error_code_counts(&summary.preflight_skipped_llm_by_code),
    );

    reporter.section_colored("top-k", "green");
    reporter.kv(0, "topk_enabled", summary.topk_enabled.to_string());
    reporter.kv(0, "topk_size", summary.topk_size.to_string());
    reporter.kv(0, "topk_attempted", summary.topk_attempted.to_string());
    reporter.kv(0, "topk_exhausted", summary.topk_exhausted.to_string());
    reporter.kv(
        0,
        "topk_selected_candidate_id",
        summary
            .topk_selected_candidate_id
            .clone()
            .unwrap_or_else(|| "none".to_string()),
    );

    reporter.section_colored("patch", "magenta");
    reporter.kv(
        0,
        "planned_file_count",
        summary.planned_file_count.to_string(),
    );
    reporter.kv(
        0,
        "planned_action_count",
        summary.planned_action_count.to_string(),
    );
    reporter.kv(
        0,
        "patch_candidate_file_count",
        summary.patch_candidate_file_count.to_string(),
    );
    reporter.kv(
        0,
        "patch_verify_check_passed",
        summary
            .patch_verify_check_passed
            .map(|v| v.to_string())
            .unwrap_or_else(|| "not-run".to_string()),
    );
    reporter.kv(
        0,
        "patch_verify_tests_passed",
        summary
            .patch_verify_tests_passed
            .map(|v| v.to_string())
            .unwrap_or_else(|| "not-run".to_string()),
    );
    reporter.kv(0, "patch_applied", summary.patch_applied.to_string());
    reporter.kv(
        0,
        "applied_file_count",
        summary.applied_file_count.to_string(),
    );

    reporter.section_colored("artifacts", "blue");
    reporter.kv(0, "diff_file", summary.diff_file.display().to_string());
    reporter.kv(
        0,
        "verify_report_file",
        summary.verify_report_file.display().to_string(),
    );
}
