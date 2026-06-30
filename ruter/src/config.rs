use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::cli::{Cli, LlmModeArg};
use crate::llm::schema::DEFAULT_RAW_RESPONSE_MAX_CHARS;

const DEFAULT_TOPK_SIZE: usize = 3;
const DEFAULT_LLM_MAX_ROUNDS: u8 = 3;
const DEFAULT_LLM_MAX_CANDIDATES_PER_ROUND: usize = 3;
const DEFAULT_LLM_TIMEOUT_SECS: u64 = 60;
const DEFAULT_LLM_CONTEXT_MAX_CHARS: usize = 12_000;
const DEFAULT_LLM_TARGET_FN_HARD_LIMIT_CHARS: usize = 8_000;
const DEFAULT_LLM_OUTPUT_TOKEN_RATIO: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmMode {
    Replay,
    Online,
}

impl LlmMode {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "replay" => Some(Self::Replay),
            "online" => Some(Self::Online),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub topk_size: usize,
    pub llm: ResolvedLlmConfig,
}

#[derive(Debug, Clone)]
pub struct ResolvedLlmConfig {
    pub enabled: bool,
    pub mode: LlmMode,
    pub replay_file: Option<PathBuf>,
    pub api_url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub max_rounds: u8,
    pub max_candidates_per_round: usize,
    pub context_max_chars: usize,
    pub target_fn_hard_limit_chars: usize,
    pub raw_response_max_chars: usize,
    pub output_token_ratio: f64,
    pub debug_dump_full_io: bool,
    pub keep_updated_sources: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct FileConfig {
    #[serde(default)]
    topk: TopkConfig,
    #[serde(default)]
    llm: LlmConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct TopkConfig {
    size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LlmConfig {
    enabled: Option<bool>,
    mode: Option<String>,
    replay_file: Option<PathBuf>,
    api_url: Option<String>,
    model: Option<String>,
    timeout_secs: Option<u64>,
    max_rounds: Option<u8>,
    max_candidates_per_round: Option<usize>,
    raw_response_max_chars: Option<usize>,
    output_token_ratio: Option<f64>,
    debug_dump_full_io: Option<bool>,
    #[serde(default)]
    context: ContextBudgetConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ContextBudgetConfig {
    max_chars: Option<usize>,
    target_fn_hard_limit_chars: Option<usize>,
}

pub fn resolve_config(cli: &Cli) -> Result<ResolvedConfig> {
    let file_cfg = load_file_config(cli.config.as_ref())?;

    let topk_size = select_usize(
        cli.topk,
        env_usize("RUTER_TOPK_SIZE")?,
        file_cfg.topk.size,
        DEFAULT_TOPK_SIZE,
    )
    .max(1);

    let mode = select_mode(
        cli.llm_mode,
        env_mode("RUTER_LLM_MODE")?,
        file_cfg.llm.mode.as_deref().and_then(LlmMode::parse),
        LlmMode::Replay,
    );

    let enabled = select_bool(
        cli.enable_llm.then_some(true),
        env_bool("RUTER_LLM_ENABLED")?,
        file_cfg.llm.enabled,
        false,
    );

    let replay_file = select_path(
        cli.llm_replay_file.clone(),
        env_path("RUTER_LLM_REPLAY_FILE"),
        file_cfg.llm.replay_file.clone(),
    );

    let api_url = select_string(
        cli.llm_api_url.clone(),
        env_string("RUTER_LLM_API_URL"),
        file_cfg.llm.api_url.clone(),
    );

    let model = select_string(
        cli.llm_model.clone(),
        env_string("RUTER_LLM_MODEL"),
        file_cfg.llm.model.clone(),
    );

    let api_key = env_string("RUTER_LLM_API_KEY");

    let timeout_secs = select_u64(
        cli.llm_timeout_secs,
        env_u64("RUTER_LLM_TIMEOUT_SECS")?,
        file_cfg.llm.timeout_secs,
        DEFAULT_LLM_TIMEOUT_SECS,
    )
    .max(1);

    let max_rounds = select_u8(
        cli.llm_max_rounds,
        env_u8("RUTER_LLM_MAX_ROUNDS")?,
        file_cfg.llm.max_rounds,
        DEFAULT_LLM_MAX_ROUNDS,
    )
    .max(1);

    let max_candidates_per_round = select_usize(
        cli.llm_max_candidates,
        env_usize("RUTER_LLM_MAX_CANDIDATES_PER_ROUND")?,
        file_cfg.llm.max_candidates_per_round,
        DEFAULT_LLM_MAX_CANDIDATES_PER_ROUND,
    )
    .max(1);

    let context_max_chars = select_usize(
        cli.llm_context_max_chars,
        env_usize("RUTER_LLM_CONTEXT_MAX_CHARS")?,
        file_cfg.llm.context.max_chars,
        DEFAULT_LLM_CONTEXT_MAX_CHARS,
    )
    .max(256);

    let target_fn_hard_limit_chars = select_usize(
        cli.llm_target_fn_hard_limit_chars,
        env_usize("RUTER_LLM_TARGET_FN_HARD_LIMIT_CHARS")?,
        file_cfg.llm.context.target_fn_hard_limit_chars,
        DEFAULT_LLM_TARGET_FN_HARD_LIMIT_CHARS,
    )
    .max(128);

    let raw_response_max_chars = select_usize(
        cli.llm_raw_excerpt_max_chars,
        env_usize("RUTER_LLM_RAW_EXCERPT_MAX_CHARS")?,
        file_cfg.llm.raw_response_max_chars,
        DEFAULT_RAW_RESPONSE_MAX_CHARS,
    )
    .max(64);

    let output_token_ratio = select_f64(
        cli.llm_output_token_ratio,
        env_f64("RUTER_LLM_OUTPUT_TOKEN_RATIO")?,
        file_cfg.llm.output_token_ratio,
        DEFAULT_LLM_OUTPUT_TOKEN_RATIO,
    );
    let output_token_ratio = normalize_output_token_ratio(output_token_ratio);

    let debug_dump_full_io = select_bool(
        cli.llm_debug_dump_full_io.then_some(true),
        env_bool("RUTER_LLM_DEBUG_DUMP_FULL_IO")?,
        file_cfg.llm.debug_dump_full_io,
        false,
    );

    Ok(ResolvedConfig {
        topk_size,
        llm: ResolvedLlmConfig {
            enabled,
            mode,
            replay_file,
            api_url,
            model,
            api_key,
            timeout_secs,
            max_rounds,
            max_candidates_per_round,
            context_max_chars,
            target_fn_hard_limit_chars,
            raw_response_max_chars,
            output_token_ratio,
            debug_dump_full_io,
            keep_updated_sources: cli.keep_updated_sources,
        },
    })
}

fn load_file_config(config_path: Option<&PathBuf>) -> Result<FileConfig> {
    let path = config_path
        .cloned()
        .unwrap_or_else(|| PathBuf::from("ruter.toml"));

    if !path.exists() {
        if config_path.is_some() {
            bail!("config file not found: {}", path.display());
        }
        return Ok(FileConfig::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;
    let cfg: FileConfig = toml::from_str(&content)
        .with_context(|| format!("failed to parse config file: {}", path.display()))?;
    Ok(cfg)
}

fn select_string(cli: Option<String>, env: Option<String>, file: Option<String>) -> Option<String> {
    cli.or(env).or(file)
}

fn select_path(
    cli: Option<PathBuf>,
    env: Option<PathBuf>,
    file: Option<PathBuf>,
) -> Option<PathBuf> {
    cli.or(env).or(file)
}

fn select_bool(cli: Option<bool>, env: Option<bool>, file: Option<bool>, default: bool) -> bool {
    cli.or(env).or(file).unwrap_or(default)
}

fn select_mode(
    cli: Option<LlmModeArg>,
    env: Option<LlmMode>,
    file: Option<LlmMode>,
    default: LlmMode,
) -> LlmMode {
    cli.map(|m| match m {
        LlmModeArg::Replay => LlmMode::Replay,
        LlmModeArg::Online => LlmMode::Online,
    })
    .or(env)
    .or(file)
    .unwrap_or(default)
}

fn select_u64(cli: Option<u64>, env: Option<u64>, file: Option<u64>, default: u64) -> u64 {
    cli.or(env).or(file).unwrap_or(default)
}

fn select_u8(cli: Option<u8>, env: Option<u8>, file: Option<u8>, default: u8) -> u8 {
    cli.or(env).or(file).unwrap_or(default)
}

fn select_usize(
    cli: Option<usize>,
    env: Option<usize>,
    file: Option<usize>,
    default: usize,
) -> usize {
    cli.or(env).or(file).unwrap_or(default)
}

fn select_f64(cli: Option<f64>, env: Option<f64>, file: Option<f64>, default: f64) -> f64 {
    cli.or(env).or(file).unwrap_or(default)
}

fn normalize_output_token_ratio(raw: f64) -> f64 {
    if raw.is_finite() && raw > 0.0 {
        raw
    } else {
        DEFAULT_LLM_OUTPUT_TOKEN_RATIO
    }
}

fn env_string(name: &str) -> Option<String> {
    env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn env_path(name: &str) -> Option<PathBuf> {
    env_string(name).map(PathBuf::from)
}

fn env_mode(name: &str) -> Result<Option<LlmMode>> {
    let Some(raw) = env_string(name) else {
        return Ok(None);
    };
    LlmMode::parse(&raw)
        .map(Some)
        .ok_or_else(|| anyhow::anyhow!("invalid env {name}={raw}, expected replay|online"))
}

fn env_u64(name: &str) -> Result<Option<u64>> {
    parse_env_num(name)
}

fn env_u8(name: &str) -> Result<Option<u8>> {
    parse_env_num(name)
}

fn env_usize(name: &str) -> Result<Option<usize>> {
    parse_env_num(name)
}

fn env_f64(name: &str) -> Result<Option<f64>> {
    parse_env_num(name)
}

fn parse_env_num<T>(name: &str) -> Result<Option<T>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let Some(raw) = env_string(name) else {
        return Ok(None);
    };
    let parsed = raw
        .parse::<T>()
        .map_err(|err| anyhow::anyhow!("invalid env {name}={raw}: {err}"))?;
    Ok(Some(parsed))
}

fn env_bool(name: &str) -> Result<Option<bool>> {
    let Some(raw) = env_string(name) else {
        return Ok(None);
    };

    let value = match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => bail!("invalid env {name}={raw}, expected boolean"),
    };
    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_mode_parse_works() {
        assert_eq!(LlmMode::parse("replay"), Some(LlmMode::Replay));
        assert_eq!(LlmMode::parse("online"), Some(LlmMode::Online));
        assert_eq!(LlmMode::parse("invalid"), None);
    }

    #[test]
    fn normalize_output_token_ratio_falls_back_to_default_for_invalid_values() {
        assert_eq!(
            normalize_output_token_ratio(-1.0),
            DEFAULT_LLM_OUTPUT_TOKEN_RATIO
        );
        assert_eq!(
            normalize_output_token_ratio(0.0),
            DEFAULT_LLM_OUTPUT_TOKEN_RATIO
        );
        assert_eq!(
            normalize_output_token_ratio(f64::NAN),
            DEFAULT_LLM_OUTPUT_TOKEN_RATIO
        );
        assert_eq!(normalize_output_token_ratio(1.5), 1.5);
    }
}
