use std::path::PathBuf;

use anyhow::{Context, Result};
use ruter::core::Diagnostic;

use crate::cli::{Cli, Stage};
use crate::config::ResolvedConfig;
use crate::runtime::artifacts::{
    ArtifactPaths, read_analyzed_diagnostics, read_compile_diagnostics, read_plan,
    read_plan_candidates, write_analyzed_diagnostics, write_compile_diagnostics, write_json,
    write_plan, write_plan_candidates,
};
use crate::runtime::function::index::FunctionIndex;
use crate::runtime::reporter::Reporter;
use crate::runtime::stages::{
    analyze_stage, apply_stage, build_function_planning_stage, compile_stage,
    compose_function_union_plan, ensure_artifact_exists, function_rule_candidates_artifact,
    plan_top_k_stage, prepare_patch_stage, verify_partial_union_plan, verify_patch_stage,
    verify_topk_candidates_stage,
};

use super::common::complete_partial_plan_path;
use super::{EXIT_OK, EXIT_PARTIAL_PENDING_LLM};

pub(super) fn run_step_mode(
    cli: &Cli,
    resolved: &ResolvedConfig,
    stage: Stage,
    crate_path: &PathBuf,
    artifacts: &ArtifactPaths,
    reporter: &mut Reporter,
) -> Result<u8> {
    let diff_path = cli
        .diff_file
        .clone()
        .unwrap_or_else(|| artifacts.default_diff_file.clone());

    match stage {
        Stage::Compile => {
            let started = reporter.stage_start("compile");
            let compile = compile_stage(crate_path, reporter)?;
            write_compile_diagnostics(
                &artifacts.compile_diagnostics_json,
                &compile.diagnostics_ndjson,
            )
            .context("failed to write compile diagnostics artifact")?;
            reporter.stage_end("compile", started);
            Ok(EXIT_OK)
        }
        Stage::Analyze => {
            let started = reporter.stage_start("analyze");
            ensure_artifact_exists(&artifacts.compile_diagnostics_json, "analyze")?;
            let raw = read_compile_diagnostics(&artifacts.compile_diagnostics_json)
                .context("failed to read compile diagnostics artifact")?;
            let diagnostics = analyze_stage(crate_path, &raw, reporter)?;
            write_analyzed_diagnostics(&artifacts.analyzed_diagnostics_json, &diagnostics)
                .context("failed to write analyzed diagnostics artifact")?;
            reporter.stage_end("analyze", started);
            Ok(EXIT_OK)
        }
        Stage::Plan => {
            let started = reporter.stage_start("plan");
            ensure_artifact_exists(&artifacts.analyzed_diagnostics_json, "plan")?;
            let diagnostics = read_analyzed_diagnostics(&artifacts.analyzed_diagnostics_json)
                .context("failed to read analyzed diagnostics artifact")?;
            let planning = build_function_planning_stage(
                crate_path,
                &diagnostics,
                resolved.topk_size,
                reporter,
            )?;
            write_json(
                &artifacts.function_dispatch_report_json,
                &planning.dispatch.report_items,
            )
            .context("failed to write function dispatch report")?;
            write_json(
                &artifacts.function_rule_candidates_json,
                &function_rule_candidates_artifact(&planning.rule_plan),
            )
            .context("failed to write function rule candidates artifact")?;
            let legacy_candidates = plan_top_k_stage(&diagnostics, resolved.topk_size, reporter)?;
            write_plan_candidates(&artifacts.plan_candidates_json, &legacy_candidates)
                .context("failed to write plan candidates artifact")?;
            let first_plan = planning
                .dispatch
                .rule_function_ids
                .iter()
                .filter_map(|function_id| {
                    planning.rule_plan.candidates_by_function.get(function_id)
                })
                .filter_map(|candidates| candidates.first())
                .flat_map(|candidate| candidate.actions.clone())
                .collect::<Vec<_>>();
            let mut grouped = std::collections::BTreeMap::new();
            for action in first_plan {
                let file = match &action {
                    ruter::core::FixAction::Insert { span, .. }
                    | ruter::core::FixAction::Replace { span, .. }
                    | ruter::core::FixAction::Delete { span } => span.file_path.clone(),
                };
                grouped.entry(file).or_insert_with(Vec::new).push(action);
            }
            write_plan(&artifacts.plan_json, &grouped).context("failed to write plan artifact")?;
            reporter.stage_end("plan", started);
            Ok(EXIT_OK)
        }
        Stage::Apply => {
            let started = reporter.stage_start("apply");
            let result = run_apply_or_verify_mode(
                true, cli, resolved, crate_path, artifacts, &diff_path, reporter,
            );
            reporter.stage_end("apply", started);
            result
        }
        Stage::Verify => {
            let started = reporter.stage_start("verify");
            let result = run_apply_or_verify_mode(
                false, cli, resolved, crate_path, artifacts, &diff_path, reporter,
            );
            reporter.stage_end("verify", started);
            result
        }
    }
}

fn run_apply_or_verify_mode(
    should_apply: bool,
    cli: &Cli,
    resolved: &ResolvedConfig,
    crate_path: &PathBuf,
    artifacts: &ArtifactPaths,
    diff_path: &PathBuf,
    reporter: &mut Reporter,
) -> Result<u8> {
    let diagnostics: Vec<Diagnostic> = if artifacts.analyzed_diagnostics_json.exists() {
        read_analyzed_diagnostics(&artifacts.analyzed_diagnostics_json)
            .context("failed to read analyzed diagnostics artifact")?
    } else {
        Vec::new()
    };
    let function_index = FunctionIndex::build(crate_path)?;
    let target_function_ids =
        function_index.target_function_ids_for_errors(&diagnostics, crate_path);

    if artifacts.partial_plan_json.exists() {
        let partial_plan = read_plan(&artifacts.partial_plan_json)
            .context("failed to read partial plan artifact")?;
        let partial_verify = verify_partial_union_plan(
            crate_path,
            &partial_plan,
            &target_function_ids,
            artifacts,
            cli.keep_updated_sources,
            reporter,
        )?;
        let partial_outcome = complete_partial_plan_path(
            resolved,
            crate_path,
            artifacts,
            &target_function_ids,
            &partial_plan,
            partial_verify,
            &function_index,
            None,
            reporter,
        )?;
        if should_apply {
            apply_stage(
                crate_path,
                &partial_outcome.partial_verify.prepared,
                cli.apply,
                !cli.no_backup,
                &artifacts.backups_dir,
                reporter,
            )?;
        }
        return Ok(
            if partial_outcome
                .partial_verify
                .unresolved_function_ids
                .is_empty()
            {
                EXIT_OK
            } else {
                EXIT_PARTIAL_PENDING_LLM
            },
        );
    }

    if artifacts.plan_candidates_json.exists() {
        let stage_label = if should_apply { "apply" } else { "verify" };
        ensure_artifact_exists(&artifacts.analyzed_diagnostics_json, stage_label)?;
        let diagnostics = read_analyzed_diagnostics(&artifacts.analyzed_diagnostics_json)
            .context("failed to read analyzed diagnostics artifact")?;
        let plan_candidates = read_plan_candidates(&artifacts.plan_candidates_json)
            .context("failed to read plan candidates artifact")?;

        if !plan_candidates.is_empty() {
            let verify_topk = verify_topk_candidates_stage(
                crate_path,
                &plan_candidates,
                &diagnostics,
                artifacts,
                cli.keep_updated_sources,
                reporter,
            )?;
            write_json(
                &artifacts.topk_function_report_json,
                &verify_topk.candidate_function_reports,
            )
            .context("failed to write top-k function report")?;

            if verify_topk.verify.patch_usable {
                if should_apply {
                    let prepared = verify_topk
                        .selected_prepared
                        .expect("selected_prepared must exist when patch is usable");
                    apply_stage(
                        crate_path,
                        &prepared,
                        cli.apply,
                        !cli.no_backup,
                        &artifacts.backups_dir,
                        reporter,
                    )?;
                }
                return Ok(EXIT_OK);
            }

            let union = compose_function_union_plan(
                crate_path,
                &plan_candidates,
                &verify_topk.candidate_function_reports,
                &verify_topk.target_function_ids,
                reporter,
            )?;
            write_plan(&artifacts.partial_plan_json, &union.plan)
                .context("failed to write partial plan artifact")?;
            let partial_verify = verify_partial_union_plan(
                crate_path,
                &union.plan,
                &verify_topk.target_function_ids,
                artifacts,
                cli.keep_updated_sources,
                reporter,
            )?;
            let partial_outcome = complete_partial_plan_path(
                resolved,
                crate_path,
                artifacts,
                &verify_topk.target_function_ids,
                &union.plan,
                partial_verify,
                &function_index,
                Some(&verify_topk.verify),
                reporter,
            )?;
            if should_apply {
                apply_stage(
                    crate_path,
                    &partial_outcome.partial_verify.prepared,
                    cli.apply,
                    !cli.no_backup,
                    &artifacts.backups_dir,
                    reporter,
                )?;
            }
            return Ok(
                if partial_outcome
                    .partial_verify
                    .unresolved_function_ids
                    .is_empty()
                {
                    EXIT_OK
                } else {
                    EXIT_PARTIAL_PENDING_LLM
                },
            );
        }
    }

    let stage_label = if should_apply { "apply" } else { "verify" };
    ensure_artifact_exists(&artifacts.plan_json, stage_label)?;
    let plan = read_plan(&artifacts.plan_json).context("failed to read plan artifact")?;
    let prepared = prepare_patch_stage(
        crate_path,
        &plan,
        diff_path,
        &artifacts.updated_sources_dir,
        cli.keep_updated_sources,
        reporter,
    )?;
    let verify = verify_patch_stage(crate_path, &prepared, cli.run_tests, artifacts, reporter)?;
    if !verify.patch_usable {
        return Ok(EXIT_PARTIAL_PENDING_LLM);
    }

    if should_apply {
        apply_stage(
            crate_path,
            &prepared,
            cli.apply,
            !cli.no_backup,
            &artifacts.backups_dir,
            reporter,
        )?;
    }

    Ok(EXIT_OK)
}
