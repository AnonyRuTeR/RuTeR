mod attempt_history;
mod candidate_resolution;
mod preflight_flow;
mod round_runner;
pub mod verify_port;
mod workspace_ops;

use crate::llm::context_builder::ContextBuildOutcome;
use crate::runtime::artifacts::ArtifactPaths;
use ruter::core::FixAction;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::config::ResolvedLlmConfig;
use crate::runtime::reporter::Reporter;
use crate::runtime::stages::{LlmReplayStageResult, PartialUnionVerifyResult};
use anyhow::Result;

const HISTORY_PATCH_SUMMARY_MAX_CHARS: usize = 256;
const HISTORY_FAILURE_DETAIL_MAX_CHARS: usize = 256;
const LOCAL_RULE_SUMMARY_LINE_MAX_CHARS: usize = 200;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmContextRecord {
    pub function_id: String,
    pub round: u8,
    pub outcome: ContextBuildOutcome,
}

#[derive(Debug, Clone)]
struct OnlinePromptState {
    user_prompt: String,
}

/// Run LLM stage in replay/online mode with per-function budget scheduling.
pub fn run_llm_stage(
    crate_path: &Path,
    artifacts: &ArtifactPaths,
    target_function_ids: &BTreeSet<String>,
    initial_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    initial_partial_verify: PartialUnionVerifyResult,
    cfg: &ResolvedLlmConfig,
    reporter: &mut Reporter,
) -> Result<LlmReplayStageResult> {
    let verify_port = crate::runtime::llm_verify_port::RuntimeLlmVerifyPort;
    round_runner::run_llm_stage_with_port(
        crate_path,
        artifacts,
        target_function_ids,
        initial_plan,
        initial_partial_verify,
        cfg,
        &verify_port,
        reporter,
    )
}

#[cfg(test)]
mod tests;
