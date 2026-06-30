mod common;
mod fix_flow;
mod step_flow;

use std::path::Path;
use std::process::ExitCode;

use crate::cli::{Cli, Command};
use crate::config::{LlmMode, resolve_config};
use crate::runtime::artifacts::ArtifactPaths;
use crate::runtime::reporter::Reporter;

pub(super) const EXIT_OK: u8 = 0;
pub(super) const EXIT_APPLY_FAILED: u8 = 3;
pub(super) const EXIT_USAGE_OR_ARTIFACT: u8 = 5;
pub(super) const EXIT_PARTIAL_PENDING_LLM: u8 = 7;

pub fn run(cli: Cli) -> ExitCode {
    match run_inner(cli) {
        Ok(code) => ExitCode::from(code),
        Err((code, msg)) => {
            eprintln!("\x1b[1;31mRUN FAILED\x1b[0m");
            eprintln!("exit_code: {code}");
            eprintln!("reason:\n{msg}");
            ExitCode::from(code)
        }
    }
}

fn run_inner(cli: Cli) -> std::result::Result<u8, (u8, String)> {
    // Avoid applying fixes without backup.
    if cli.no_backup && !cli.apply {
        return Err((
            EXIT_USAGE_OR_ARTIFACT,
            "--no-backup only works with --apply".to_string(),
        ));
    }

    // Extract crate path and stage (if step mode) from the command.
    let (crate_path, stage) = match &cli.command {
        Command::Fix { crate_path } => (crate_path.clone(), None),
        Command::Step { stage, crate_path } => (crate_path.clone(), Some(*stage)),
    };

    ensure_crate_path(&crate_path).map_err(|e| (EXIT_USAGE_OR_ARTIFACT, e))?;

    let resolved = resolve_config(&cli).map_err(|err| {
        (
            EXIT_USAGE_OR_ARTIFACT,
            format!("failed to resolve configuration: {err:#}"),
        )
    })?;
    crate::llm::runtime_entry::validate_llm_usage(&cli, &resolved)
        .map_err(|msg| (EXIT_USAGE_OR_ARTIFACT, msg))?;

    let artifacts = match cli.artifacts_dir.as_ref() {
        Some(path) => ArtifactPaths::from_root(path.clone()),
        None => ArtifactPaths::default_for(&crate_path),
    };

    let mut reporter = Reporter::new(cli.verbose, cli.log_file.as_deref()).map_err(|e| {
        (
            EXIT_USAGE_OR_ARTIFACT,
            format!("failed to initialize reporter: {e}"),
        )
    })?;

    reporter.info("ruter run started");
    reporter.kv(0, "crate", crate_path.display().to_string());
    reporter.kv(0, "apply", cli.apply.to_string());
    reporter.kv(0, "run_tests", cli.run_tests.to_string());
    reporter.kv(
        0,
        "keep_updated_sources",
        cli.keep_updated_sources.to_string(),
    );
    reporter.kv(0, "topk_size", resolved.topk_size.to_string());
    reporter.kv(0, "llm_enabled", resolved.llm.enabled.to_string());
    reporter.kv(
        0,
        "llm_mode",
        match resolved.llm.mode {
            LlmMode::Replay => "replay".to_string(),
            LlmMode::Online => "online".to_string(),
        },
    );
    reporter.kv(
        0,
        "llm_replay_file",
        resolved
            .llm
            .replay_file
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    reporter.kv(
        0,
        "llm_api_url",
        resolved
            .llm
            .api_url
            .clone()
            .unwrap_or_else(|| "none".to_string()),
    );
    reporter.kv(
        0,
        "llm_model",
        resolved
            .llm
            .model
            .clone()
            .unwrap_or_else(|| "none".to_string()),
    );
    reporter.kv(0, "llm_timeout_secs", resolved.llm.timeout_secs.to_string());
    reporter.kv(0, "llm_max_rounds", resolved.llm.max_rounds.to_string());
    reporter.kv(
        0,
        "llm_context_max_chars",
        resolved.llm.context_max_chars.to_string(),
    );
    reporter.kv(
        0,
        "llm_debug_dump_full_io",
        resolved.llm.debug_dump_full_io.to_string(),
    );
    reporter.kv(0, "artifacts", artifacts.root.display().to_string());

    artifacts.ensure_root().map_err(|e| {
        (
            EXIT_USAGE_OR_ARTIFACT,
            format!("failed to create artifacts dir: {e}"),
        )
    })?;
    crate::llm::runtime_entry::bootstrap_llm_artifact_if_enabled(&resolved, &artifacts).map_err(
        |err| {
            (
                EXIT_USAGE_OR_ARTIFACT,
                format!("failed to bootstrap llm attempts artifact: {err:#}"),
            )
        },
    )?;

    let result = if let Some(stage) = stage {
        step_flow::run_step_mode(
            &cli,
            &resolved,
            stage,
            &crate_path,
            &artifacts,
            &mut reporter,
        )
    } else {
        fix_flow::run_fix_mode(&cli, &resolved, &crate_path, &artifacts, &mut reporter)
    };

    result.map_err(map_runtime_error)
}

fn ensure_crate_path(crate_path: &Path) -> std::result::Result<(), String> {
    if !crate_path.exists() {
        return Err(format!(
            "crate path does not exist: {}",
            crate_path.display()
        ));
    }
    if !crate_path.join("Cargo.toml").exists() {
        return Err(format!(
            "Cargo.toml not found under {}",
            crate_path.display()
        ));
    }
    Ok(())
}

fn map_runtime_error(err: anyhow::Error) -> (u8, String) {
    let msg = format!("{err:#}");

    if msg.contains("required artifact missing") || msg.contains("artifact") {
        return (EXIT_USAGE_OR_ARTIFACT, msg);
    }

    (EXIT_APPLY_FAILED, msg)
}
