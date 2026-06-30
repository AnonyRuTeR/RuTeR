use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cli::Cli;
use crate::config::ResolvedConfig;
use crate::runtime::artifacts::{
    ArtifactPaths, RunSummary, write_analyzed_diagnostics, write_compile_diagnostics, write_json,
    write_plan, write_summary,
};
use crate::runtime::function::index::FunctionIndex;
use crate::runtime::reporter::Reporter;
use crate::runtime::stages::{
    analyze_stage, apply_stage, build_function_planning_stage, collect_error_stats, compile_stage,
    function_rule_candidates_artifact, verify_function_optimistic_stage,
};

use super::common::{complete_partial_plan_path, report_run_summary};
use super::{EXIT_OK, EXIT_PARTIAL_PENDING_LLM};

pub(super) fn run_fix_mode(
    cli: &Cli,
    resolved: &ResolvedConfig,
    crate_path: &PathBuf,
    artifacts: &ArtifactPaths,
    reporter: &mut Reporter,
) -> Result<u8> {
    let diff_path = cli
        .diff_file
        .clone()
        .unwrap_or_else(|| artifacts.default_diff_file.clone());

    let compile_start = reporter.stage_start("compile");
    let compile = compile_stage(crate_path, reporter)?;
    write_compile_diagnostics(
        &artifacts.compile_diagnostics_json,
        &compile.diagnostics_ndjson,
    )
    .context("failed to write compile diagnostics artifact")?;
    reporter.stage_end("compile", compile_start);

    let analyze_start = reporter.stage_start("analyze");
    let diagnostics = analyze_stage(crate_path, &compile.diagnostics_ndjson, reporter)?;
    let initial_error_stats = collect_error_stats(&diagnostics);
    write_analyzed_diagnostics(&artifacts.analyzed_diagnostics_json, &diagnostics)
        .context("failed to write analyzed diagnostics artifact")?;
    reporter.stage_end("analyze", analyze_start);

    let plan_start = reporter.stage_start("plan");
    let planning =
        build_function_planning_stage(crate_path, &diagnostics, resolved.topk_size, reporter)?;
    write_json(
        &artifacts.function_dispatch_report_json,
        &planning.dispatch.report_items,
    )
    .context("failed to write function dispatch report")?;
    write_json(
        &artifacts.function_rule_candidates_json,
        &function_rule_candidates_artifact(&planning.rule_plan),
    )
    .context("failed to write function rule candidates report")?;
    let first_plan = planning
        .dispatch
        .rule_function_ids
        .iter()
        .filter_map(|function_id| planning.rule_plan.candidates_by_function.get(function_id))
        .filter_map(|candidates| candidates.first())
        .flat_map(|candidate| candidate.actions.clone())
        .collect::<Vec<_>>();
    let mut first_plan_grouped = std::collections::BTreeMap::new();
    for action in first_plan {
        let file = match &action {
            ruter::core::FixAction::Insert { span, .. }
            | ruter::core::FixAction::Replace { span, .. }
            | ruter::core::FixAction::Delete { span } => span.file_path.clone(),
        };
        first_plan_grouped
            .entry(file)
            .or_insert_with(Vec::new)
            .push(action);
    }
    write_plan(&artifacts.plan_json, &first_plan_grouped)
        .context("failed to write plan artifact")?;
    reporter.stage_end("plan", plan_start);

    let verify_start = reporter.stage_start("verify");
    let function_verify = verify_function_optimistic_stage(
        crate_path,
        &planning,
        artifacts,
        cli.keep_updated_sources,
        reporter,
    )?;
    write_json(
        &artifacts.function_verify_rounds_json,
        &function_verify.optimistic.rounds,
    )
    .context("failed to write function verify rounds artifact")?;
    reporter.stage_end("verify", verify_start);

    let index = FunctionIndex::build(crate_path)?;
    let partial_outcome = complete_partial_plan_path(
        resolved,
        crate_path,
        artifacts,
        &planning.dispatch.target_function_ids,
        &function_verify.optimistic.final_plan,
        function_verify.partial_verify,
        &index,
        Some(&function_verify.verify),
        reporter,
    )?;

    let apply_start = reporter.stage_start("apply");
    let apply = apply_stage(
        crate_path,
        &partial_outcome.partial_verify.prepared,
        cli.apply,
        !cli.no_backup,
        &artifacts.backups_dir,
        reporter,
    )?;
    reporter.stage_end("apply", apply_start);

    if partial_outcome.partial_verify.prepared.diff_path != diff_path {
        if let Some(parent) = diff_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(
            &partial_outcome.partial_verify.prepared.diff_path,
            &diff_path,
        )
        .context("failed to sync selected diff")?;
    }

    let pending_llm = !partial_outcome
        .partial_verify
        .unresolved_function_ids
        .is_empty();
    let exit_code = if pending_llm {
        EXIT_PARTIAL_PENDING_LLM
    } else {
        EXIT_OK
    };

    let summary = RunSummary {
        initial_compile_passed: compile.status_success,
        diagnostic_count: diagnostics.len(),
        initial_error_total: initial_error_stats.error_total,
        initial_error_by_code: initial_error_stats.error_by_code.clone(),
        remaining_error_total: partial_outcome.partial_verify.check_error_total,
        remaining_error_by_code: partial_outcome.partial_verify.check_error_by_code.clone(),
        planned_file_count: partial_outcome.plan.len(),
        planned_action_count: partial_outcome.plan.values().map(Vec::len).sum(),
        patch_candidate_file_count: partial_outcome.partial_verify.prepared.changed_file_count(),
        patch_verify_check_passed: Some(
            partial_outcome
                .partial_verify
                .unresolved_function_ids
                .is_empty(),
        ),
        patch_verify_tests_passed: function_verify.verify.tests_passed,
        patch_applied: apply.patch_applied,
        applied_file_count: apply.applied_file_count,
        diff_file: partial_outcome.partial_verify.prepared.diff_path.clone(),
        verify_report_file: artifacts.verify_report_json.clone(),
        topk_enabled: false,
        topk_size: resolved.topk_size,
        topk_attempted: function_verify.optimistic.rounds.len(),
        topk_selected_candidate_id: function_verify.verify.selected_candidate_id.clone(),
        topk_exhausted: pending_llm,
        partial_pending_llm: pending_llm,
        llm_handoff_count: partial_outcome.llm_handoff_count,
        preflight_skipped_llm_count: partial_outcome.preflight_skipped_llm_count,
        preflight_skipped_llm_by_code: partial_outcome.preflight_skipped_llm_by_code.clone(),
        resolved_test_function_count: partial_outcome.partial_verify.resolved_function_ids.len(),
        unresolved_test_function_count: partial_outcome
            .partial_verify
            .unresolved_function_ids
            .len(),
    };
    let summarize_start = reporter.stage_start("summarize");
    write_summary(&artifacts.summary_json, &summary).context("failed to write summary artifact")?;
    report_run_summary(reporter, exit_code, &summary);
    reporter.stage_end("summarize", summarize_start);

    Ok(exit_code)
}
