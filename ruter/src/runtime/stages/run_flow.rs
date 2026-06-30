use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use ruter::coordinator::{GlobalCandidatePlan, PatchCoordinator};
use ruter::core::Diagnostic;
use ruter::transformer::CodeTransformer;

use crate::runtime::reporter::Reporter;

use super::{CompileStageResult, build_default_rule_registry};

pub fn compile_stage(crate_path: &Path, reporter: &mut Reporter) -> Result<CompileStageResult> {
    reporter.section("compile command");
    reporter.kv(0, "command", "cargo check --tests --message-format=json");
    reporter.kv(1, "cwd", crate_path.display().to_string());

    let output = Command::new("cargo")
        .arg("check")
        .arg("--tests")
        .arg("--message-format=json")
        .current_dir(crate_path)
        .output()
        .with_context(|| format!("failed to execute cargo check in {}", crate_path.display()))?;

    let diagnostics_ndjson = super::extract_diagnostic_json_lines(&output.stdout)?;
    reporter.section("compile result");
    reporter.kv(0, "cargo_check_passed", output.status.success().to_string());
    reporter.kv(
        0,
        "compiler_diagnostic_count",
        diagnostics_ndjson.lines().count().to_string(),
    );
    super::report_stderr_snippet("cargo_check", &output, reporter);

    Ok(CompileStageResult {
        status_success: output.status.success(),
        diagnostics_ndjson,
    })
}

pub fn analyze_stage(
    crate_path: &Path,
    diagnostics_ndjson: &str,
    reporter: &mut Reporter,
) -> Result<Vec<Diagnostic>> {
    let diagnostics = super::parse_diagnostics_for_crate(crate_path, diagnostics_ndjson)?;

    reporter.section("analyze result");
    reporter.kv(0, "diagnostic_count", diagnostics.len().to_string());
    super::report_diagnostics(&diagnostics, reporter);

    Ok(diagnostics)
}

pub fn plan_top_k_stage(
    diagnostics: &[Diagnostic],
    k: usize,
    reporter: &mut Reporter,
) -> Result<Vec<GlobalCandidatePlan>> {
    let registry = build_default_rule_registry();
    let coordinator = PatchCoordinator::new(registry, CodeTransformer::new());

    let plans = coordinator
        .plan_top_k(diagnostics, k)
        .context("failed to build top-k patch candidates")?;

    reporter.section("top-k plan result");
    reporter.kv(0, "topk_size", k.to_string());
    reporter.kv(0, "candidate_count", plans.len().to_string());
    for candidate in &plans {
        let action_count: usize = candidate.plan.values().map(Vec::len).sum();
        reporter.item(
            0,
            format!(
                "{} score={} file_count={} action_count={}",
                candidate.candidate_id,
                candidate.score,
                candidate.plan.len(),
                action_count
            ),
        );
    }

    Ok(plans)
}
