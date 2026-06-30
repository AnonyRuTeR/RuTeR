use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use ruter::coordinator::PatchCoordinator;
use ruter::core::FixAction;
use ruter::patchers::PatcherRegistry;
use ruter::transformer::CodeTransformer;

pub(super) fn build_workspace_for_plan(
    crate_path: &Path,
    plan: &BTreeMap<PathBuf, Vec<FixAction>>,
) -> Result<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let name = crate_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("crate");
    let workspace = std::env::temp_dir().join(format!("ruter_llm_workspace_{name}_{ts}"));
    fs::create_dir_all(&workspace)?;

    copy_project_tree(crate_path, &workspace)?;
    if plan.is_empty() {
        return Ok(workspace);
    }

    let mut source_map = HashMap::new();
    for file in plan.keys() {
        let source = fs::read_to_string(file)
            .with_context(|| format!("failed to read source {}", file.display()))?;
        source_map.insert(file.clone(), source);
    }
    let coordinator = PatchCoordinator::new(PatcherRegistry::new(), CodeTransformer::new());
    let updated_sources = coordinator
        .apply_planned(plan, &source_map)
        .context("failed to materialize current partial plan sources")?;

    for (origin_file, content) in updated_sources {
        let relative = origin_file.strip_prefix(crate_path).unwrap_or(&origin_file);
        let target = workspace.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, content)
            .with_context(|| format!("failed to write workspace source {}", target.display()))?;
    }

    Ok(workspace)
}

fn copy_project_tree(src: &Path, dst: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();

        if should_skip_path(path) {
            continue;
        }

        let relative = path.strip_prefix(src)?;
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target)?;
        }
    }

    Ok(())
}

fn should_skip_path(path: &Path) -> bool {
    path.components().any(|component| {
        let part = component.as_os_str();
        part == "target" || part == ".git" || part == ".ruter"
    })
}
