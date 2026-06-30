use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use quote::ToTokens;
use ruter::core::FixAction;
use serde::{Deserialize, Serialize};
use syn::ItemFn;

pub const DEFAULT_RAW_RESPONSE_MAX_CHARS: usize = 4096;
pub const ATTEMPTS_SCHEMA_VERSION: &str = "2";

/// Failure categories for LLM attempts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LlmFailureKind {
    LlmOutputInvalidSchema,
    LlmActionOutOfScope,
    LlmActionConflict,
    LlmVerifyFailed,
    LlmBudgetExhausted,
    FunctionMappingFailed,
    LlmRequestFailed,
    ContextTooLarge,
}

/// One replay file used by offline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReplayFile {
    #[serde(default = "default_replay_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub functions: Vec<LlmReplayFunction>,
}

/// Replay content for one function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReplayFunction {
    pub function_id: String,
    #[serde(default)]
    pub rounds: Vec<LlmReplayRound>,
}

/// Replay content for one round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReplayRound {
    pub round: u8,
    #[serde(default)]
    pub raw_response: Option<String>,
    #[serde(default)]
    pub raw_transport_response: Option<String>,
    #[serde(default)]
    pub candidates: Vec<LlmReplayCandidate>,
}

/// A replay candidate supports both new and legacy payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReplayCandidate {
    pub candidate_id: String,
    #[serde(default)]
    pub patched_function_text: Option<String>,
    #[serde(default)]
    pub actions: Option<Vec<FixAction>>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub risk_flags: Vec<String>,
    #[serde(default)]
    pub raw_response: Option<String>,
}

/// Replay execution artifact written for observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAttemptsArtifact {
    #[serde(default = "default_attempts_schema_version")]
    pub schema_version: String,
    pub enabled: bool,
    pub mode: String,
    pub replay_file: Option<PathBuf>,
    #[serde(default)]
    pub online_api_url: Option<String>,
    #[serde(default)]
    pub online_model: Option<String>,
    #[serde(default)]
    pub max_rounds: u8,
    #[serde(default)]
    pub max_candidates_per_round: usize,
    #[serde(default)]
    pub attempts: Vec<LlmAttemptRecord>,
}

/// One candidate evaluation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAttemptRecord {
    pub function_id: String,
    pub round: u8,
    #[serde(default)]
    pub phase: String,
    pub candidate_id: Option<String>,
    pub accepted: bool,
    #[serde(default)]
    pub failure_kind: Option<LlmFailureKind>,
    #[serde(default)]
    pub failure_detail: Option<String>,
    #[serde(default)]
    pub previous_unresolved_function_ids: Vec<String>,
    #[serde(default)]
    pub unresolved_function_ids: Vec<String>,
    #[serde(default)]
    pub check_error_total: Option<usize>,
    #[serde(default)]
    pub check_error_by_code: BTreeMap<String, usize>,
    #[serde(default)]
    pub check_stdout_log: Option<PathBuf>,
    #[serde(default)]
    pub check_stderr_log: Option<PathBuf>,
    #[serde(default)]
    pub raw_response_excerpt: Option<String>,
    #[serde(default)]
    pub prompt_excerpt: Option<String>,
    #[serde(default)]
    pub normalized_candidate: Option<NormalizedCandidateEvidence>,
}

/// Normalized candidate evidence for playback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedCandidateEvidence {
    pub candidate_id: String,
    pub source_kind: String,
    #[serde(default)]
    pub merge_strategy: Option<String>,
    pub action_count: usize,
    #[serde(default)]
    pub action_summaries: Vec<String>,
    #[serde(default)]
    pub normalized_text_excerpt: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub risk_flags: Vec<String>,
}

impl LlmReplayFile {
    /// Read replay file from disk.
    pub fn read_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read llm replay file: {}", path.display()))?;
        let replay: LlmReplayFile = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse llm replay json: {}", path.display()))?;
        Ok(replay)
    }

    /// Find round payload for one function.
    pub fn find_round(&self, function_id: &str, round: u8) -> Option<&LlmReplayRound> {
        self.functions
            .iter()
            .find(|item| item.function_id == function_id)
            .and_then(|item| item.rounds.iter().find(|r| r.round == round))
    }
}

impl LlmAttemptsArtifact {
    pub fn bootstrap(
        mode: &str,
        replay_file: Option<PathBuf>,
        online_api_url: Option<String>,
        online_model: Option<String>,
        max_rounds: u8,
        max_candidates_per_round: usize,
    ) -> Self {
        Self {
            schema_version: ATTEMPTS_SCHEMA_VERSION.to_string(),
            enabled: true,
            mode: mode.to_string(),
            replay_file,
            online_api_url,
            online_model,
            max_rounds,
            max_candidates_per_round,
            attempts: Vec::new(),
        }
    }
}

fn default_replay_schema_version() -> String {
    "1".to_string()
}

fn default_attempts_schema_version() -> String {
    ATTEMPTS_SCHEMA_VERSION.to_string()
}

/// Normalize replay text conservatively.
///
/// Behavior:
/// - remove BOM
/// - normalize newlines to `\n`
/// - trim trailing blank lines
/// - keep one final newline for non-empty input
pub fn normalize_replay_text(raw: &str) -> String {
    let mut text = raw.replace("\r\n", "\n").replace('\r', "\n");
    if text.starts_with('\u{feff}') {
        text = text.trim_start_matches('\u{feff}').to_string();
    }

    let mut lines = text.lines().collect::<Vec<_>>();
    while lines
        .last()
        .map(|line| line.trim().is_empty())
        .unwrap_or(false)
    {
        let _ = lines.pop();
    }

    if lines.is_empty() {
        return String::new();
    }

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Try to normalize style with local rustfmt.
///
/// If rustfmt is unavailable or formatting fails, return `None` and caller
/// should fallback to conservative normalized text.
pub fn normalize_with_rustfmt(input: &str) -> Option<String> {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let stdin = child.stdin.as_mut()?;
    if stdin.write_all(input.as_bytes()).is_err() {
        return None;
    }

    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }

    let formatted = String::from_utf8_lossy(&output.stdout);
    Some(normalize_replay_text(&formatted))
}

/// Normalize one patched function text.
///
/// The text must parse as `syn::ItemFn` before rustfmt normalization.
pub fn normalize_function_text(raw: &str) -> std::result::Result<String, String> {
    let conservative = normalize_replay_text(raw);
    if conservative.is_empty() {
        return Err("patched_function_text is empty after normalization".to_string());
    }
    parse_item_fn(&conservative)?;
    Ok(normalize_with_rustfmt(&conservative).unwrap_or(conservative))
}

/// Parse function text into AST.
pub fn parse_item_fn(text: &str) -> std::result::Result<ItemFn, String> {
    syn::parse_str::<ItemFn>(text)
        .map_err(|err| format!("patched_function_text is not valid Rust function item: {err}"))
}

/// Validate function signature and attributes stay unchanged.
///
/// We only allow function-body-level edits in M2/M3.
pub fn validate_signature_and_attrs_unchanged(
    original_function_text: &str,
    patched_function_text: &str,
) -> std::result::Result<(), String> {
    let original = parse_item_fn(original_function_text)?;
    let patched = parse_item_fn(patched_function_text)?;

    let original_sig = original.sig.to_token_stream().to_string();
    let patched_sig = patched.sig.to_token_stream().to_string();
    if original_sig != patched_sig {
        return Err("function signature changed".to_string());
    }

    let original_attrs = original
        .attrs
        .iter()
        .map(|attr| attr.to_token_stream().to_string())
        .collect::<Vec<_>>();
    let patched_attrs = patched
        .attrs
        .iter()
        .map(|attr| attr.to_token_stream().to_string())
        .collect::<Vec<_>>();
    if original_attrs != patched_attrs {
        return Err("function attributes changed".to_string());
    }

    Ok(())
}

/// Truncate long evidence text for artifacts.
pub fn truncate_for_artifact(raw: &str, max_chars: usize) -> String {
    if raw.chars().count() <= max_chars {
        return raw.to_string();
    }

    let truncated = raw.chars().take(max_chars).collect::<String>();
    format!("{truncated}...(truncated)")
}

/// Sanitize an id for filesystem path segment.
pub fn sanitize_path_component(raw: &str) -> String {
    let mapped = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    let trimmed = mapped.trim_matches('_');
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.chars().take(96).collect()
    }
}

/// Render concise action summaries for attempt artifacts.
pub fn summarize_actions(actions: &[FixAction]) -> Vec<String> {
    actions
        .iter()
        .map(|action| match action {
            FixAction::Replace { span, new_content } => format!(
                "Replace {}:{}..{} => {}",
                span.file_path.display(),
                span.byte_start,
                span.byte_end,
                truncate_for_artifact(new_content, 120)
            ),
            FixAction::Insert { span, content } => format!(
                "Insert {}:{}..{} => {}",
                span.file_path.display(),
                span.byte_start,
                span.byte_end,
                truncate_for_artifact(content, 120)
            ),
            FixAction::Delete { span } => format!(
                "Delete {}:{}..{}",
                span.file_path.display(),
                span.byte_start,
                span.byte_end
            ),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_supports_patched_function_text_schema() {
        let raw = r##"
{
  "schema_version": "1",
  "functions": [
    {
      "function_id": "f1",
      "rounds": [
        {
          "round": 1,
          "candidates": [
            {
              "candidate_id": "c1",
              "patched_function_text": "#[test]\\nfn a() { let _ = 1; }\\n"
            }
          ]
        }
      ]
    }
  ]
}
"##;
        let replay: LlmReplayFile = serde_json::from_str(raw).unwrap();
        let round = replay.find_round("f1", 1).unwrap();
        assert_eq!(round.candidates.len(), 1);
        assert!(round.candidates[0].patched_function_text.is_some());
        assert!(round.candidates[0].actions.is_none());
    }

    #[test]
    fn normalize_replay_text_unifies_crlf_and_trailing_blanks() {
        let raw = "\u{feff}fn a() {\r\n    let _ = 1;\r\n}\r\n\r\n";
        let normalized = normalize_replay_text(raw);
        assert_eq!(normalized, "fn a() {\n    let _ = 1;\n}\n");
    }

    #[test]
    fn validate_signature_rejects_signature_change() {
        let original = "#[test]\nfn case_a() { let _ = 1; }\n";
        let patched = "#[test]\nfn case_a(v: i32) { let _ = v; }\n";
        let err = validate_signature_and_attrs_unchanged(original, patched).unwrap_err();
        assert!(err.contains("signature"));
    }

    #[test]
    fn validate_signature_rejects_attr_change() {
        let original = "#[test]\nfn case_a() { let _ = 1; }\n";
        let patched = "#[tokio::test]\nfn case_a() { let _ = 1; }\n";
        let err = validate_signature_and_attrs_unchanged(original, patched).unwrap_err();
        assert!(err.contains("attributes"));
    }

    #[test]
    fn sanitize_path_component_removes_special_chars() {
        let sanitized = sanitize_path_component("src/lib.rs::tests::case_a:10:20");
        assert!(sanitized.contains("src_lib_rs__tests__case_a_10_20"));
    }

    #[test]
    fn llm_attempts_bootstrap_is_schema_v2() {
        let artifact = LlmAttemptsArtifact::bootstrap(
            "online",
            None,
            Some("https://example.test".to_string()),
            Some("gpt".to_string()),
            3,
            3,
        );
        assert_eq!(artifact.schema_version, "2");
        assert_eq!(artifact.max_rounds, 3);
    }
}
