use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use ruter::coordinator::GlobalCandidatePlan;
use ruter::core::{Diagnostic, FixAction};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ArtifactPaths {
    pub root: PathBuf,
    pub compile_diagnostics_json: PathBuf,
    pub analyzed_diagnostics_json: PathBuf,
    pub plan_json: PathBuf,
    pub plan_candidates_json: PathBuf,
    pub summary_json: PathBuf,
    pub default_diff_file: PathBuf,
    pub updated_sources_dir: PathBuf,
    pub verify_report_json: PathBuf,
    pub verify_attempts_dir: PathBuf,
    pub topk_function_report_json: PathBuf,
    pub partial_plan_json: PathBuf,
    pub llm_handoff_json: PathBuf,
    pub llm_attempts_json: PathBuf,
    pub llm_contexts_json: PathBuf,
    pub llm_io_debug_json: PathBuf,
    pub function_dispatch_report_json: PathBuf,
    pub function_rule_candidates_json: PathBuf,
    pub function_verify_rounds_json: PathBuf,
    pub verify_check_stdout_log: PathBuf,
    pub verify_check_stderr_log: PathBuf,
    pub verify_test_stdout_log: PathBuf,
    pub verify_test_stderr_log: PathBuf,
    pub verify_failed_workspace_dir: PathBuf,
    pub backups_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanArtifact {
    pub files: Vec<FilePlan>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilePlan {
    pub file_path: PathBuf,
    pub actions: Vec<FixAction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunSummary {
    pub initial_compile_passed: bool,
    pub diagnostic_count: usize,
    #[serde(default)]
    pub initial_error_total: usize,
    #[serde(default)]
    pub initial_error_by_code: BTreeMap<String, usize>,
    #[serde(default)]
    pub remaining_error_total: usize,
    #[serde(default)]
    pub remaining_error_by_code: BTreeMap<String, usize>,
    pub planned_file_count: usize,
    pub planned_action_count: usize,
    pub patch_candidate_file_count: usize,
    pub patch_verify_check_passed: Option<bool>,
    pub patch_verify_tests_passed: Option<bool>,
    pub patch_applied: bool,
    pub applied_file_count: usize,
    pub diff_file: PathBuf,
    pub verify_report_file: PathBuf,
    #[serde(default)]
    pub topk_enabled: bool,
    #[serde(default)]
    pub topk_size: usize,
    #[serde(default)]
    pub topk_attempted: usize,
    #[serde(default)]
    pub topk_selected_candidate_id: Option<String>,
    #[serde(default)]
    pub topk_exhausted: bool,
    #[serde(default)]
    pub partial_pending_llm: bool,
    #[serde(default)]
    pub llm_handoff_count: usize,
    #[serde(default)]
    pub preflight_skipped_llm_count: usize,
    #[serde(default)]
    pub preflight_skipped_llm_by_code: BTreeMap<String, usize>,
    #[serde(default)]
    pub resolved_test_function_count: usize,
    #[serde(default)]
    pub unresolved_test_function_count: usize,
}

impl ArtifactPaths {
    pub fn from_root(root: PathBuf) -> Self {
        let verify_dir = root.join("verify");
        Self {
            compile_diagnostics_json: root.join("1_compile_diagnostics.json"),
            analyzed_diagnostics_json: root.join("2_analyzed_diagnostics.json"),
            plan_json: root.join("3_plan.json"),
            plan_candidates_json: root.join("3_plan_candidates.json"),
            summary_json: root.join("6_summary.json"),
            default_diff_file: root.join("5_changes.diff"),
            updated_sources_dir: root.join("5_updated"),
            verify_report_json: verify_dir.join("4_report.json"),
            verify_attempts_dir: verify_dir.join("4_attempts"),
            topk_function_report_json: verify_dir.join("4_topk_function_report.json"),
            partial_plan_json: root.join("4_partial_plan.json"),
            llm_handoff_json: root.join("4_llm_handoff.json"),
            llm_attempts_json: root.join("4_llm_attempts.json"),
            llm_contexts_json: root.join("4_llm_contexts.json"),
            llm_io_debug_json: root.join("4_llm_io_debug.json"),
            function_dispatch_report_json: root.join("3_function_dispatch_report.json"),
            function_rule_candidates_json: root.join("3_function_rule_candidates.json"),
            function_verify_rounds_json: verify_dir.join("4_function_verify_rounds.json"),
            verify_check_stdout_log: verify_dir.join("4_check.stdout.log"),
            verify_check_stderr_log: verify_dir.join("4_check.stderr.log"),
            verify_test_stdout_log: verify_dir.join("4_test.stdout.log"),
            verify_test_stderr_log: verify_dir.join("4_test.stderr.log"),
            verify_failed_workspace_dir: verify_dir.join("4_failed_workspace"),
            backups_dir: root.join("5_backups"),
            root,
        }
    }

    pub fn default_for(crate_path: &Path) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let root = crate_path.join(".ruter").join(format!("run_{ts}"));
        Self::from_root(root)
    }

    pub fn ensure_root(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)
    }
}

pub fn write_compile_diagnostics(path: &Path, diagnostics_ndjson: &str) -> std::io::Result<()> {
    let diagnostics: Vec<Value> = diagnostics_ndjson
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect();

    let pretty = serde_json::to_string_pretty(&diagnostics).unwrap_or_else(|_| "[]".to_string());
    fs::write(path, pretty)
}

pub fn read_compile_diagnostics(path: &Path) -> std::io::Result<String> {
    let raw = fs::read_to_string(path)?;
    let values: Vec<Value> = serde_json::from_str(&raw).unwrap_or_default();
    let ndjson = values
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(ndjson)
}

pub fn write_analyzed_diagnostics(path: &Path, diagnostics: &[Diagnostic]) -> Result<()> {
    let content = serde_json::to_string_pretty(diagnostics)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn read_analyzed_diagnostics(path: &Path) -> Result<Vec<Diagnostic>> {
    let content = fs::read_to_string(path)?;
    let diagnostics = serde_json::from_str(&content)?;
    Ok(diagnostics)
}

pub fn write_plan(path: &Path, plan: &BTreeMap<PathBuf, Vec<FixAction>>) -> Result<()> {
    let file_plans = plan
        .iter()
        .map(|(file_path, actions)| FilePlan {
            file_path: file_path.clone(),
            actions: actions.clone(),
        })
        .collect();

    let artifact = PlanArtifact { files: file_plans };
    let content = serde_json::to_string_pretty(&artifact)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn read_plan(path: &Path) -> Result<BTreeMap<PathBuf, Vec<FixAction>>> {
    let content = fs::read_to_string(path)?;
    let artifact: PlanArtifact = serde_json::from_str(&content)?;

    let mut map = BTreeMap::new();
    for file in artifact.files {
        map.insert(file.file_path, file.actions);
    }

    Ok(map)
}

pub fn write_summary(path: &Path, summary: &RunSummary) -> Result<()> {
    let content = serde_json::to_string_pretty(summary)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn write_plan_candidates(path: &Path, plans: &[GlobalCandidatePlan]) -> Result<()> {
    let content = serde_json::to_string_pretty(plans)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn read_plan_candidates(path: &Path) -> Result<Vec<GlobalCandidatePlan>> {
    let content = fs::read_to_string(path)?;
    let plans = serde_json::from_str(&content)?;
    Ok(plans)
}

pub fn write_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(data)?;
    fs::write(path, content)?;
    Ok(())
}
