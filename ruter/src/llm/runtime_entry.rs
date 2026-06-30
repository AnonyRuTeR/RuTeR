use anyhow::Result;

use crate::cli::Cli;
use crate::config::LlmMode;
use crate::runtime::artifacts::{ArtifactPaths, write_json};

/// Validate CLI/config combinations for LLM mode before runtime orchestration starts.
pub fn validate_llm_usage(
    cli: &Cli,
    resolved: &crate::config::ResolvedConfig,
) -> std::result::Result<(), String> {
    if cli.llm_replay_file.is_some() && !resolved.llm.enabled {
        return Err("--llm-replay-file requires --enable-llm".to_string());
    }

    if !resolved.llm.enabled {
        return Ok(());
    }

    match resolved.llm.mode {
        LlmMode::Replay => {
            if resolved.llm.replay_file.is_none() {
                return Err(
                    "--enable-llm requires --llm-replay-file <PATH> when --llm-mode replay"
                        .to_string(),
                );
            }
        }
        LlmMode::Online => {
            if resolved.llm.api_url.is_none() {
                return Err("llm online mode requires api url".to_string());
            }
            if resolved.llm.model.is_none() {
                return Err("llm online mode requires model".to_string());
            }
            if resolved.llm.api_key.is_none() {
                return Err(
                    "llm online mode requires api key via RUTER_LLM_API_KEY".to_string(),
                );
            }
        }
    }

    Ok(())
}

/// Bootstrap attempts artifact when LLM is enabled.
pub fn bootstrap_llm_artifact_if_enabled(
    resolved: &crate::config::ResolvedConfig,
    artifacts: &ArtifactPaths,
) -> Result<()> {
    if !resolved.llm.enabled {
        return Ok(());
    }

    let mode = match resolved.llm.mode {
        LlmMode::Replay => "replay",
        LlmMode::Online => "online",
    };
    let bootstrap = crate::llm::schema::LlmAttemptsArtifact::bootstrap(
        mode,
        resolved.llm.replay_file.clone(),
        resolved.llm.api_url.clone(),
        resolved.llm.model.clone(),
        resolved.llm.max_rounds,
        resolved.llm.max_candidates_per_round,
    );
    write_json(&artifacts.llm_attempts_json, &bootstrap)?;

    Ok(())
}
