use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use walkdir::WalkDir;

use crate::runtime::reporter::Reporter;

use super::VerifyStageResult;

pub(super) fn changed_files(
    before: &HashMap<PathBuf, String>,
    after: &HashMap<PathBuf, String>,
) -> Vec<PathBuf> {
    let mut changed = Vec::new();
    for (file, old_content) in before {
        if let Some(new_content) = after.get(file)
            && new_content != old_content
        {
            changed.push(file.clone());
        }
    }
    changed.sort();
    changed
}

pub(super) fn write_backups(
    crate_path: &Path,
    backups_dir: &Path,
    changed: &[PathBuf],
    before: &HashMap<PathBuf, String>,
) -> Result<()> {
    fs::create_dir_all(backups_dir)?;

    for file in changed {
        let backup_path = backup_path_for(crate_path, backups_dir, file);
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = before
            .get(file)
            .ok_or_else(|| anyhow!("missing source snapshot for {}", file.display()))?;
        fs::write(&backup_path, content)
            .with_context(|| format!("failed to write backup {}", backup_path.display()))?;
    }

    Ok(())
}

pub(super) fn write_updated_sources(
    crate_path: &Path,
    updated_dir: &Path,
    updated: &HashMap<PathBuf, String>,
) -> Result<()> {
    fs::create_dir_all(updated_dir)?;

    for (file, content) in updated {
        let target = updated_snapshot_path_for(crate_path, updated_dir, file);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, content)
            .with_context(|| format!("failed to write updated source {}", target.display()))?;
    }

    Ok(())
}

pub(super) fn updated_snapshot_path_for(
    crate_path: &Path,
    updated_dir: &Path,
    file: &Path,
) -> PathBuf {
    let relative = file.strip_prefix(crate_path).unwrap_or(file);
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let stem = relative
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("updated");
    let ext = relative.extension().and_then(|e| e.to_str()).unwrap_or("");
    let file_name = if ext.is_empty() {
        format!("{stem}_updated")
    } else {
        format!("{stem}_updated.{ext}")
    };

    updated_dir.join(parent).join(file_name)
}

pub(super) fn write_files_with_rollback(
    crate_path: &Path,
    backups_dir: &Path,
    backup_enabled: bool,
    changed: &[PathBuf],
    updated: &HashMap<PathBuf, String>,
    reporter: &mut Reporter,
) -> Result<()> {
    let mut written = Vec::new();

    for file in changed {
        let content = updated
            .get(file)
            .ok_or_else(|| anyhow!("missing updated source for {}", file.display()))?;

        if let Err(err) = fs::write(file, content) {
            reporter.error(format!("write failed on {}: {}", file.display(), err));

            if backup_enabled {
                rollback_written_files(crate_path, backups_dir, &written)?;
                reporter.kv(0, "rollback", "completed from backups");
            }

            return Err(err).with_context(|| format!("failed to write source {}", file.display()));
        }

        written.push(file.clone());
    }

    Ok(())
}

pub(super) fn rollback_written_files(
    crate_path: &Path,
    backups_dir: &Path,
    written: &[PathBuf],
) -> Result<()> {
    for file in written {
        let backup_path = backup_path_for(crate_path, backups_dir, file);
        if backup_path.exists() {
            let content = fs::read_to_string(&backup_path)?;
            fs::write(file, content)?;
        }
    }
    Ok(())
}

pub(super) fn backup_path_for(crate_path: &Path, backups_dir: &Path, file: &Path) -> PathBuf {
    let relative = file.strip_prefix(crate_path).unwrap_or(file);
    backups_dir.join(relative).with_extension("bak")
}

pub(super) fn create_temp_workspace_dir(crate_path: &Path) -> Result<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let name = crate_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("crate");
    let dir = std::env::temp_dir().join(format!("ruter_verify_{name}_{ts}"));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub(super) fn copy_project_tree(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src) {
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

pub(super) fn should_skip_path(path: &Path) -> bool {
    path.components().any(|component| {
        let part = component.as_os_str();
        part == OsStr::new("target")
            || part == OsStr::new(".git")
            || part == OsStr::new(".ruter")
    })
}

pub(super) fn preserve_failed_workspace(
    temp_crate: &Path,
    failed_workspace_dir: &Path,
) -> Result<PathBuf> {
    if failed_workspace_dir.exists() {
        fs::remove_dir_all(failed_workspace_dir)?;
    }
    fs::create_dir_all(failed_workspace_dir)?;
    copy_project_tree(temp_crate, failed_workspace_dir)?;
    Ok(failed_workspace_dir.to_path_buf())
}

pub(super) fn write_command_logs(
    output: &Output,
    stdout_log: &Path,
    stderr_log: &Path,
) -> Result<()> {
    if let Some(parent) = stdout_log.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = stderr_log.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(stdout_log, String::from_utf8_lossy(&output.stdout).as_ref())?;
    fs::write(stderr_log, String::from_utf8_lossy(&output.stderr).as_ref())?;
    Ok(())
}

pub(super) fn write_verify_report(path: &Path, report: &VerifyStageResult) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(report)?;
    fs::write(path, content)?;
    Ok(())
}
