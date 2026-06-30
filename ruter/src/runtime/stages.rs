use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Output;

mod diagnostics;
mod diff;
mod io;
mod run_flow;
mod verify_engine;
mod verify_flow;

use crate::llm::schema::LlmAttemptsArtifact;
use crate::runtime::function::dispatch::{FunctionDispatchOutput, build_dispatch_output};
use crate::runtime::function::index::{FunctionDiagnostic, FunctionIndex};
use crate::runtime::function::rule_plan::{
    FunctionRuleCandidatesArtifact, FunctionRulePlanResult, plan_rule_candidates,
};
use crate::runtime::function::verify::{
    FunctionOptimisticVerifyResult, RoundEvaluation, run_optimistic_greedy,
};
use anyhow::Result;
use ruter::core::{Diagnostic, FixAction};
use ruter::patchers::{
    E0308Patcher, E0432Patcher, E0433Patcher, E0560Patcher, E0599Patcher, PatcherRegistry,
};
use serde::{Deserialize, Serialize};

use crate::runtime::artifacts::ArtifactPaths;
use crate::runtime::reporter::Reporter;

#[derive(Debug)]
pub struct CompileStageResult {
    pub status_success: bool,
    pub diagnostics_ndjson: String,
}

#[derive(Debug)]
pub struct ApplyStageResult {
    pub applied_file_count: usize,
    pub patch_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticStats {
    pub error_total: usize,
    pub error_by_code: BTreeMap<String, usize>,
    pub error_details: Vec<ErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyAttempt {
    pub candidate_id: String,
    pub check_passed: bool,
    pub check_error_total: usize,
    pub check_error_by_code: BTreeMap<String, usize>,
    pub check_stdout_log: PathBuf,
    pub check_stderr_log: PathBuf,
    pub preserved_workspace: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateFunctionReport {
    pub candidate_id: String,
    pub score: i32,
    pub check_passed: bool,
    pub resolved_function_ids: Vec<String>,
    pub unresolved_function_ids: Vec<String>,
    pub error_by_function: BTreeMap<String, BTreeMap<String, usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyStageResult {
    pub strategy: String,
    pub check_command: String,
    pub tests_command: Option<String>,
    pub temp_workspace: PathBuf,
    pub preserved_workspace: Option<PathBuf>,
    pub check_passed: bool,
    pub tests_passed: Option<bool>,
    pub patch_usable: bool,
    pub check_stdout_log: PathBuf,
    pub check_stderr_log: PathBuf,
    pub tests_stdout_log: Option<PathBuf>,
    pub tests_stderr_log: Option<PathBuf>,
    pub check_error_total: usize,
    pub check_error_by_code: BTreeMap<String, usize>,
    #[serde(default)]
    pub selected_candidate_id: Option<String>,
    #[serde(default)]
    pub attempts: Vec<VerifyAttempt>,
    #[serde(default)]
    pub target_test_function_count: usize,
    #[serde(default)]
    pub resolved_test_function_count: usize,
    #[serde(default)]
    pub unresolved_test_function_count: usize,
    #[serde(default)]
    pub unresolved_function_ids: Vec<String>,
    #[serde(default)]
    pub dispatch_target_function_count: usize,
    #[serde(default)]
    pub rule_function_count: usize,
    #[serde(default)]
    pub llm_routed_function_count: usize,
    #[serde(default)]
    pub optimistic_round_count: usize,
    #[serde(default)]
    pub independence_broken_function_ids: Vec<String>,
}

#[derive(Debug)]
pub struct PreparedPatch {
    source_map: HashMap<PathBuf, String>,
    updated_sources: HashMap<PathBuf, String>,
    changed_files: Vec<PathBuf>,
    pub diff_path: PathBuf,
}

#[derive(Debug)]
pub struct VerifyTopKCandidatesResult {
    pub verify: VerifyStageResult,
    pub selected_prepared: Option<PreparedPatch>,
    pub candidate_function_reports: Vec<CandidateFunctionReport>,
    pub target_function_ids: BTreeSet<String>,
}

#[derive(Debug)]
pub struct PartialUnionPlanResult {
    pub plan: BTreeMap<PathBuf, Vec<FixAction>>,
}

#[derive(Debug)]
pub struct PartialUnionVerifyResult {
    pub prepared: PreparedPatch,
    pub resolved_function_ids: BTreeSet<String>,
    pub unresolved_function_ids: BTreeSet<String>,
    pub check_error_total: usize,
    pub check_error_by_code: BTreeMap<String, usize>,
    pub error_diagnostics_by_function: BTreeMap<String, Vec<FunctionDiagnostic>>,
    pub check_stdout_log: PathBuf,
    pub check_stderr_log: PathBuf,
}

#[derive(Debug)]
pub struct LlmReplayStageResult {
    pub plan: BTreeMap<PathBuf, Vec<FixAction>>,
    pub partial_verify: PartialUnionVerifyResult,
    pub attempts_artifact: LlmAttemptsArtifact,
    pub preflight_skipped_llm_count: usize,
    pub preflight_skipped_llm_by_code: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct FunctionPlanningResult {
    pub dispatch: FunctionDispatchOutput,
    pub rule_plan: FunctionRulePlanResult,
}

#[derive(Debug)]
pub struct FunctionVerifyExecutionResult {
    pub verify: VerifyStageResult,
    pub optimistic: FunctionOptimisticVerifyResult,
    pub partial_verify: PartialUnionVerifyResult,
}

impl PreparedPatch {
    pub fn changed_file_count(&self) -> usize {
        self.changed_files.len()
    }

    pub fn source_for_file(&self, file_path: &Path) -> Option<String> {
        self.updated_sources
            .get(file_path)
            .or_else(|| self.source_map.get(file_path))
            .cloned()
    }
}

pub use run_flow::{analyze_stage, compile_stage, plan_top_k_stage};
pub use verify_flow::{
    apply_stage, compose_function_union_plan, ensure_artifact_exists, prepare_patch_stage,
    verify_partial_union_plan, verify_partial_union_plan_with_tag, verify_patch_stage,
    verify_topk_candidates_stage,
};

fn build_default_rule_registry() -> PatcherRegistry {
    let mut registry = PatcherRegistry::new();
    registry.register(Box::new(E0433Patcher::new()));
    registry.register(Box::new(E0432Patcher::new()));
    registry.register(Box::new(E0599Patcher::new()));
    registry.register(Box::new(E0308Patcher::new()));
    registry.register(Box::new(E0560Patcher::new()));
    registry
}

/// 构建函数级任务、分发决策、以及规则路径候选（Zip 对齐）。
pub fn build_function_planning_stage(
    crate_path: &Path,
    diagnostics: &[Diagnostic],
    k: usize,
    reporter: &mut Reporter,
) -> Result<FunctionPlanningResult> {
    let registry = build_default_rule_registry();
    let index = FunctionIndex::build(crate_path)?;
    let implemented_codes = registry.implemented_error_codes();

    let dispatch = build_dispatch_output(diagnostics, crate_path, &index, &implemented_codes);
    let rule_plan = plan_rule_candidates(&dispatch, &registry, k)?;

    reporter.section("function dispatch result");
    reporter.kv(
        0,
        "dispatch_target_function_count",
        dispatch.target_function_ids.len().to_string(),
    );
    reporter.kv(
        0,
        "rule_function_count",
        dispatch.rule_function_ids.len().to_string(),
    );
    reporter.kv(
        0,
        "llm_routed_function_count",
        dispatch.llm_routed_function_ids.len().to_string(),
    );
    reporter.kv(
        0,
        "function_rule_candidate_count",
        rule_plan.flat_candidates.len().to_string(),
    );
    for item in &dispatch.report_items {
        reporter.item(
            0,
            format!(
                "{} decision={} selected_diagnostics={} suppressed={}",
                item.function_id,
                item.decision,
                item.selected_diagnostic_indexes.len(),
                item.suppressed_diagnostics.len()
            ),
        );
    }

    Ok(FunctionPlanningResult {
        dispatch,
        rule_plan,
    })
}

pub fn function_rule_candidates_artifact(
    rule_plan: &FunctionRulePlanResult,
) -> Vec<FunctionRuleCandidatesArtifact> {
    let mut out = Vec::new();
    for (function_id, candidates) in &rule_plan.candidates_by_function {
        out.push(FunctionRuleCandidatesArtifact {
            function_id: function_id.clone(),
            candidates: candidates.clone(),
        });
    }
    out
}

pub fn verify_function_optimistic_stage(
    crate_path: &Path,
    planning: &FunctionPlanningResult,
    artifacts: &ArtifactPaths,
    keep_updated_sources: bool,
    reporter: &mut Reporter,
) -> Result<FunctionVerifyExecutionResult> {
    let target_function_ids = &planning.dispatch.target_function_ids;
    let candidates_by_function = &planning.rule_plan.candidates_by_function;
    let max_rank = planning.rule_plan.max_rank.max(1);

    let optimistic = run_optimistic_greedy(
        candidates_by_function,
        target_function_ids,
        max_rank,
        |plan| {
            let round_verify = verify_partial_union_plan(
                crate_path,
                plan,
                target_function_ids,
                artifacts,
                keep_updated_sources,
                reporter,
            )?;
            Ok(RoundEvaluation {
                check_error_total: round_verify.check_error_total,
                check_error_by_code: round_verify.check_error_by_code.clone(),
                unresolved_function_ids: round_verify.unresolved_function_ids,
            })
        },
    )?;

    let partial_verify = verify_partial_union_plan(
        crate_path,
        &optimistic.final_plan,
        target_function_ids,
        artifacts,
        keep_updated_sources,
        reporter,
    )?;

    let attempts = optimistic
        .rounds
        .iter()
        .map(|round| VerifyAttempt {
            candidate_id: format!("round-{}", round.round),
            check_passed: round.unresolved_function_ids.is_empty(),
            check_error_total: round.check_error_total,
            check_error_by_code: round.check_error_by_code.clone(),
            check_stdout_log: artifacts
                .verify_attempts_dir
                .join("partial_union")
                .join("check.stdout.log"),
            check_stderr_log: artifacts
                .verify_attempts_dir
                .join("partial_union")
                .join("check.stderr.log"),
            preserved_workspace: None,
        })
        .collect::<Vec<_>>();

    let verify = VerifyStageResult {
        strategy: "verify function-level optimistic-greedy patch in isolated temp workspace"
            .to_string(),
        check_command: "cargo check --tests --message-format=json".to_string(),
        tests_command: None,
        temp_workspace: PathBuf::new(),
        preserved_workspace: None,
        check_passed: partial_verify.unresolved_function_ids.is_empty(),
        tests_passed: None,
        patch_usable: partial_verify.unresolved_function_ids.is_empty(),
        check_stdout_log: artifacts
            .verify_attempts_dir
            .join("partial_union")
            .join("check.stdout.log"),
        check_stderr_log: artifacts
            .verify_attempts_dir
            .join("partial_union")
            .join("check.stderr.log"),
        tests_stdout_log: None,
        tests_stderr_log: None,
        check_error_total: partial_verify.check_error_total,
        check_error_by_code: partial_verify.check_error_by_code.clone(),
        selected_candidate_id: Some("function-optimistic".to_string()),
        attempts,
        target_test_function_count: target_function_ids.len(),
        resolved_test_function_count: partial_verify.resolved_function_ids.len(),
        unresolved_test_function_count: partial_verify.unresolved_function_ids.len(),
        unresolved_function_ids: partial_verify
            .unresolved_function_ids
            .iter()
            .cloned()
            .collect(),
        dispatch_target_function_count: target_function_ids.len(),
        rule_function_count: planning.dispatch.rule_function_ids.len(),
        llm_routed_function_count: planning.dispatch.llm_routed_function_ids.len(),
        optimistic_round_count: optimistic.rounds.len(),
        independence_broken_function_ids: optimistic
            .independence_broken_function_ids
            .iter()
            .cloned()
            .collect(),
    };
    write_verify_report(&artifacts.verify_report_json, &verify)?;

    reporter.section("function optimistic verify");
    reporter.kv(
        0,
        "optimistic_round_count",
        optimistic.rounds.len().to_string(),
    );
    reporter.kv(
        0,
        "resolved_test_function_count",
        verify.resolved_test_function_count.to_string(),
    );
    reporter.kv(
        0,
        "unresolved_test_function_count",
        verify.unresolved_test_function_count.to_string(),
    );

    Ok(FunctionVerifyExecutionResult {
        verify,
        optimistic,
        partial_verify,
    })
}

pub fn collect_error_stats(diagnostics: &[Diagnostic]) -> DiagnosticStats {
    diagnostics::collect_error_stats(diagnostics)
}

fn extract_diagnostic_json_lines(stdout: &[u8]) -> Result<String> {
    diagnostics::extract_diagnostic_json_lines(stdout)
}

fn parse_diagnostics_for_crate(
    crate_path: &Path,
    diagnostics_ndjson: &str,
) -> Result<Vec<Diagnostic>> {
    diagnostics::parse_diagnostics_for_crate(crate_path, diagnostics_ndjson)
}

fn changed_files(
    before: &HashMap<PathBuf, String>,
    after: &HashMap<PathBuf, String>,
) -> Vec<PathBuf> {
    io::changed_files(before, after)
}

fn write_backups(
    crate_path: &Path,
    backups_dir: &Path,
    changed: &[PathBuf],
    before: &HashMap<PathBuf, String>,
) -> Result<()> {
    io::write_backups(crate_path, backups_dir, changed, before)
}

fn write_updated_sources(
    crate_path: &Path,
    updated_dir: &Path,
    updated: &HashMap<PathBuf, String>,
) -> Result<()> {
    io::write_updated_sources(crate_path, updated_dir, updated)
}

fn write_files_with_rollback(
    crate_path: &Path,
    backups_dir: &Path,
    backup_enabled: bool,
    changed: &[PathBuf],
    updated: &HashMap<PathBuf, String>,
    reporter: &mut Reporter,
) -> Result<()> {
    io::write_files_with_rollback(
        crate_path,
        backups_dir,
        backup_enabled,
        changed,
        updated,
        reporter,
    )
}

fn preserve_failed_workspace(temp_crate: &Path, failed_workspace_dir: &Path) -> Result<PathBuf> {
    io::preserve_failed_workspace(temp_crate, failed_workspace_dir)
}

fn write_command_logs(output: &Output, stdout_log: &Path, stderr_log: &Path) -> Result<()> {
    io::write_command_logs(output, stdout_log, stderr_log)
}

fn write_verify_report(path: &Path, report: &VerifyStageResult) -> Result<()> {
    io::write_verify_report(path, report)
}

fn report_diagnostics(diagnostics: &[Diagnostic], reporter: &mut Reporter) {
    let mut shown = 0usize;
    for diag in diagnostics {
        let code = if let Some(raw) = diag
            .code
            .as_ref()
            .and_then(|c| c.raw_code.as_ref())
            .map(|s| s.trim().to_string())
        {
            if raw.starts_with('E')
                && raw.len() == 5
                && raw[1..].chars().all(|ch| ch.is_ascii_digit())
            {
                raw
            } else {
                "Unknown".to_string()
            }
        } else if let Some(enum_code) = diag.code.as_ref().map(|c| c.code) {
            if enum_code != ruter::core::ErrorCode::Unknown {
                enum_code.to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "NO_CODE".to_string()
        };

        let location = diag
            .span
            .first()
            .map(|s| format!("{}:{}:{}", s.file_path.display(), s.line_start, s.col_start))
            .unwrap_or_else(|| "unknown".to_string());

        reporter.item(
            0,
            format!(
                "{} [{}] {} @ {}",
                diag.severity_string(),
                code,
                diag.message,
                location
            ),
        );

        shown += 1;
        if shown >= 8 {
            if diagnostics.len() > shown {
                reporter.item(
                    0,
                    format!("... {} more diagnostics", diagnostics.len() - shown),
                );
            }
            break;
        }
    }
}

fn report_stderr_snippet(key_prefix: &str, output: &Output, reporter: &mut Reporter) {
    if output.stderr.is_empty() {
        return;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut lines = stderr.lines();

    reporter.kv(
        1,
        format!("{key_prefix}_stderr_line_count"),
        stderr.lines().count().to_string(),
    );

    for idx in 0..6 {
        let Some(line) = lines.next() else {
            break;
        };
        reporter.kv(1, format!("{key_prefix}_stderr_preview[{idx}]"), line);
    }
}

pub fn format_error_code_counts(counts: &BTreeMap<String, usize>) -> String {
    diagnostics::format_error_code_counts(counts)
}

trait SeverityString {
    fn severity_string(&self) -> &'static str;
}

impl SeverityString for Diagnostic {
    fn severity_string(&self) -> &'static str {
        match self.severity {
            ruter::core::Severity::Error => "error",
            ruter::core::Severity::Warning => "warning",
            ruter::core::Severity::Note => "note",
            ruter::core::Severity::Help => "help",
            ruter::core::Severity::FailureNote => "failure-note",
            ruter::core::Severity::InternalCompilerError => "internal-compiler-error",
        }
    }
}

fn render_diff(
    crate_path: &Path,
    before: &HashMap<PathBuf, String>,
    after: &HashMap<PathBuf, String>,
) -> String {
    diff::render_diff(crate_path, before, after)
}

#[cfg(test)]
#[path = "stages/stages_tests.rs"]
mod tests;
