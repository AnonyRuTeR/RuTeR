use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;
use serde_json::Value;

use crate::llm::schema::{LlmReplayCandidate, LlmReplayFile, LlmReplayRound};

#[derive(Debug, Clone)]
pub struct OnlineLlmClientConfig {
    pub api_url: String,
    pub model: String,
    pub api_key: String,
    pub timeout_secs: u64,
    pub output_token_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct OnlineLlmClient {
    cfg: OnlineLlmClientConfig,
    http: Client,
}

impl OnlineLlmClient {
    pub fn new(cfg: OnlineLlmClientConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(cfg.timeout_secs.max(1)))
            .build()
            .context("failed to build reqwest blocking client")?;
        Ok(Self { cfg, http })
    }

    pub fn request_round(
        &self,
        function_id: &str,
        round: u8,
        system_prompt: &str,
        user_prompt: &str,
        max_candidates: usize,
    ) -> Result<LlmReplayRound> {
        let url = chat_completions_url(&self.cfg.api_url);
        let body = build_chat_completion_body(
            &self.cfg.model,
            system_prompt,
            user_prompt,
            self.cfg.output_token_ratio,
        );

        let response = self
            .http
            .post(url)
            .bearer_auth(&self.cfg.api_key)
            .json(&body)
            .send()
            .context("online llm request failed")?;

        let status = response.status();
        let response_text = response
            .text()
            .context("failed to read online llm response body")?;

        if !status.is_success() {
            bail!(
                "online llm request failed with status {} body={}",
                status,
                response_text
            );
        }

        let response_json = parse_json_with_fallback(&response_text).with_context(|| {
            format!("failed to parse online llm response as json: {response_text}")
        })?;

        let content = extract_chat_content(&response_json)
            .ok_or_else(|| anyhow!("online llm response missing choices[0].message.content"))?;

        let candidates = parse_candidates_from_content(&content, function_id, round)?;
        let mut capped = candidates;
        if capped.len() > max_candidates {
            capped.truncate(max_candidates);
        }

        Ok(LlmReplayRound {
            round,
            raw_response: Some(content),
            raw_transport_response: Some(response_text),
            candidates: capped,
        })
    }
}

fn build_chat_completion_body(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    output_token_ratio: f64,
) -> Value {
    let input_tokens_est = estimate_input_tokens(system_prompt, user_prompt);
    let cap_tokens = compute_cap_tokens(input_tokens_est, output_token_ratio);

    serde_json::json!({
        "model": model,
        "temperature": 0.0,
        "max_tokens": cap_tokens,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ]
    })
}

fn estimate_input_tokens(system_prompt: &str, user_prompt: &str) -> u64 {
    estimate_tokens_from_text(system_prompt) + estimate_tokens_from_text(user_prompt)
}

fn estimate_tokens_from_text(raw: &str) -> u64 {
    // 近似估算：1 token ~= 4 chars，且至少保留 1，避免 cap 计算为 0。
    let chars = raw.chars().count().max(1);
    chars.div_ceil(4) as u64
}

fn compute_cap_tokens(input_tokens_est: u64, output_token_ratio: f64) -> u64 {
    let normalized_ratio = if output_token_ratio.is_finite() && output_token_ratio > 0.0 {
        output_token_ratio
    } else {
        2.0
    };
    ((input_tokens_est as f64 * normalized_ratio).floor() as u64).max(1)
}

fn chat_completions_url(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/chat/completions")
    }
}

fn extract_chat_content(root: &Value) -> Option<String> {
    let content = root
        .get("choices")?
        .as_array()?
        .first()?
        .get("message")?
        .get("content")?;

    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }

    // 兼容 content 数组格式：[{"type":"text","text":"..."}]
    if let Some(items) = content.as_array() {
        let joined = items
            .iter()
            .filter_map(|item| item.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n");
        if !joined.trim().is_empty() {
            return Some(joined);
        }
    }

    None
}

pub fn parse_candidates_from_content(
    content: &str,
    function_id: &str,
    round: u8,
) -> Result<Vec<LlmReplayCandidate>> {
    if let Some(rust_block) = extract_first_rust_fenced_block(content)
        && let Some(function_text) = extract_first_function_item_from_text(&rust_block)
    {
        return Ok(vec![build_single_function_candidate(
            &function_text,
            content,
        )]);
    }

    if let Some(function_text) = extract_first_function_item_from_text(content) {
        return Ok(vec![build_single_function_candidate(
            &function_text,
            content,
        )]);
    }

    let value = parse_json_with_fallback(content)
        .with_context(|| "llm content is not valid json and fallback extraction failed")?;

    // shape A: full replay file
    if let Ok(replay) = serde_json::from_value::<LlmReplayFile>(value.clone()) {
        if let Some(payload) = replay.find_round(function_id, round) {
            return Ok(payload.candidates.clone());
        }
    }

    // shape B: {"candidates": [...]}
    if let Some(candidates) = value.get("candidates").and_then(Value::as_array) {
        return normalize_candidate_array(candidates);
    }

    // shape C: [candidate, ...]
    if let Some(candidates) = value.as_array() {
        return normalize_candidate_array(candidates);
    }

    // shape D: single candidate object
    if value.is_object() {
        let one = normalize_candidate_item(&value, 0)?;
        return Ok(vec![one]);
    }

    bail!("unsupported llm content json shape")
}

fn build_single_function_candidate(function_text: &str, raw_content: &str) -> LlmReplayCandidate {
    LlmReplayCandidate {
        candidate_id: "c1".to_string(),
        patched_function_text: Some(function_text.to_string()),
        actions: None,
        rationale: None,
        risk_flags: Vec::new(),
        raw_response: Some(raw_content.to_string()),
    }
}

fn normalize_candidate_array(items: &[Value]) -> Result<Vec<LlmReplayCandidate>> {
    let mut out = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        out.push(normalize_candidate_item(item, idx)?);
    }
    Ok(out)
}

fn normalize_candidate_item(item: &Value, idx: usize) -> Result<LlmReplayCandidate> {
    let mut candidate: LlmReplayCandidate = serde_json::from_value(item.clone())
        .with_context(|| format!("invalid candidate item at index {idx}"))?;
    if candidate.candidate_id.trim().is_empty() {
        candidate.candidate_id = format!("c{}", idx + 1);
    }
    Ok(candidate)
}

pub fn parse_json_with_fallback(raw: &str) -> Result<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        return Ok(value);
    }

    if let Some(first_object) = extract_first_json_object(raw)
        && let Ok(value) = serde_json::from_str::<Value>(&first_object)
    {
        return Ok(value);
    }

    bail!("json parse failed and no valid first json object found")
}

fn extract_first_rust_fenced_block(raw: &str) -> Option<String> {
    let mut in_fence = false;
    let mut fence_is_rust = false;
    let mut block_lines = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim_start();
        if let Some(fence_meta) = trimmed.strip_prefix("```") {
            if !in_fence {
                let lang = fence_meta.trim().to_ascii_lowercase();
                fence_is_rust = lang == "rust" || lang == "rs";
                in_fence = true;
                block_lines.clear();
            } else {
                if fence_is_rust {
                    let block = block_lines.join("\n");
                    if !block.trim().is_empty() {
                        return Some(block);
                    }
                }
                in_fence = false;
                fence_is_rust = false;
                block_lines.clear();
            }
            continue;
        }

        if in_fence {
            block_lines.push(line);
        }
    }

    None
}

fn extract_first_function_item_from_text(raw: &str) -> Option<String> {
    let lines = raw.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }

    for start in 0..lines.len() {
        let mut candidate = String::new();
        for end in start..lines.len() {
            if !candidate.is_empty() {
                candidate.push('\n');
            }
            candidate.push_str(lines[end]);
            candidate.push('\n');

            if syn::parse_str::<syn::ItemFn>(&candidate).is_ok() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Extract the first complete JSON object from text.
pub fn extract_first_json_object(raw: &str) -> Option<String> {
    let text = raw.replace("```json", "").replace("```", "");
    let bytes = text.as_bytes();

    let mut start = None;
    let mut depth = 0usize;
    let mut in_str = false;
    let mut escaped = false;

    for (i, b) in bytes.iter().enumerate() {
        let ch = *b as char;

        if in_str {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                in_str = false;
            }
            continue;
        }

        if ch == '"' {
            in_str = true;
            continue;
        }

        if ch == '{' {
            if start.is_none() {
                start = Some(i);
            }
            depth += 1;
            continue;
        }

        if ch == '}' {
            if depth == 0 {
                continue;
            }
            depth -= 1;
            if depth == 0
                && let Some(begin) = start
            {
                return Some(text[begin..=i].to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_client_extracts_first_json_object_from_markdown() {
        let raw = "prefix\n```json\n{\"candidates\":[{\"candidate_id\":\"c1\",\"patched_function_text\":\"fn a(){}\"}]}\n```\nsuffix";
        let obj = extract_first_json_object(raw).expect("json object");
        assert!(obj.contains("candidates"));
    }

    #[test]
    fn llm_client_parses_candidates_from_candidates_object() {
        let raw = r##"{"candidates":[{"candidate_id":"c1","patched_function_text":"#[test]\nfn a() {}\n"}]}"##;
        let candidates = parse_candidates_from_content(raw, "f", 1).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].candidate_id, "c1");
    }

    #[test]
    fn llm_client_parses_candidates_with_fallback_object() {
        let raw =
            "answer: {\"candidate_id\":\"\",\"patched_function_text\":\"#[test]\\nfn a() {}\\n\"}";
        let candidates = parse_candidates_from_content(raw, "f", 1).unwrap();
        assert_eq!(candidates[0].candidate_id, "c1");
    }

    #[test]
    fn llm_client_parses_fenced_rust_function_first() {
        let raw = "```rust\n#[test]\nfn a() { let _ = 1; }\n```\n{\"candidates\":[]}";
        let candidates = parse_candidates_from_content(raw, "f", 1).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].candidate_id, "c1");
        assert!(
            candidates[0]
                .patched_function_text
                .as_deref()
                .unwrap_or_default()
                .contains("fn a()")
        );
    }

    #[test]
    fn llm_client_parses_plain_function_item_before_json_fallback() {
        let raw = "Here is the fix:\n#[test]\nfn a() { let _ = 2; }\n";
        let candidates = parse_candidates_from_content(raw, "f", 1).unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(
            candidates[0]
                .patched_function_text
                .as_deref()
                .unwrap_or_default()
                .contains("fn a()")
        );
    }

    #[test]
    fn request_body_sets_max_tokens_by_input_estimate_and_ratio() {
        let body = build_chat_completion_body("mock-model", "12345678", "1234", 2.0);
        // input_tokens_est = ceil(8/4) + ceil(4/4) = 2 + 1 = 3
        // cap_tokens = floor(3 * 2.0) = 6
        assert_eq!(body.get("max_tokens").and_then(Value::as_u64), Some(6));
    }

    #[test]
    fn cap_tokens_falls_back_to_default_ratio_when_ratio_invalid() {
        assert_eq!(compute_cap_tokens(10, 0.0), 20);
        assert_eq!(compute_cap_tokens(10, -1.0), 20);
        assert_eq!(compute_cap_tokens(10, f64::NAN), 20);
    }
}
