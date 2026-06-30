use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{Context, Result};
use ruter::core::Diagnostic;

use super::DiagnosticStats;
use super::diagnostics::{
    collect_error_stats, extract_diagnostic_json_lines, parse_diagnostics_for_crate,
};
use super::io::{copy_project_tree, create_temp_workspace_dir};

pub(super) struct CheckWorkspaceResult {
    pub temp_workspace: PathBuf,
    pub check_output: Output,
    pub check_diagnostics: Vec<Diagnostic>,
    pub check_error_stats: DiagnosticStats,
}

pub(super) fn run_check_with_updated_sources(
    crate_path: &Path,
    updated_sources: &HashMap<PathBuf, String>,
) -> Result<CheckWorkspaceResult> {
    let temp_workspace = create_temp_workspace_dir(crate_path)?;
    copy_project_tree(crate_path, &temp_workspace)?;

    for (orig, content) in updated_sources {
        let relative = orig.strip_prefix(crate_path).unwrap_or(orig);
        let target = temp_workspace.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, content)
            .with_context(|| format!("failed to write temp source {}", target.display()))?;
    }

    let check_output = Command::new("cargo")
        .arg("check")
        .arg("--tests")
        .arg("--message-format=json")
        .current_dir(&temp_workspace)
        .output()
        .with_context(|| format!("failed to run cargo check in {}", temp_workspace.display()))?;

    let check_diagnostics_ndjson = extract_diagnostic_json_lines(&check_output.stdout)?;
    let check_diagnostics =
        parse_diagnostics_for_crate(&temp_workspace, &check_diagnostics_ndjson)?;
    let check_error_stats = collect_error_stats(&check_diagnostics);

    Ok(CheckWorkspaceResult {
        temp_workspace,
        check_output,
        check_diagnostics,
        check_error_stats,
    })
}
