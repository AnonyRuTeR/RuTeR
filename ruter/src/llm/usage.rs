use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const LLM_USAGE_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmTokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    #[serde(default)]
    pub cached_input_tokens: Option<u64>,
    #[serde(default)]
    pub reasoning_tokens: Option<u64>,
    pub source: String,
    #[serde(default)]
    pub total_matches_components: Option<bool>,
    #[serde(default)]
    pub raw_usage: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmRequestUsageRecord {
    pub function_id: String,
    pub round: u8,
    pub configured_model: String,
    #[serde(default)]
    pub returned_model: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub http_status: Option<u16>,
    pub outcome: String,
    pub latency_ms: u64,
    pub max_output_tokens: u64,
    pub usage: LlmTokenUsage,
    #[serde(default)]
    pub error: Option<String>,
}

impl LlmRequestUsageRecord {
    pub fn pending(
        function_id: &str,
        round: u8,
        configured_model: &str,
        max_output_tokens: u64,
    ) -> Self {
        Self {
            function_id: function_id.to_string(),
            round,
            configured_model: configured_model.to_string(),
            returned_model: None,
            request_id: None,
            http_status: None,
            outcome: "pending".to_string(),
            latency_ms: 0,
            max_output_tokens,
            usage: LlmTokenUsage {
                source: "missing".to_string(),
                ..LlmTokenUsage::default()
            },
            error: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmUsageSummary {
    pub request_count: usize,
    pub successful_request_count: usize,
    pub failed_request_count: usize,
    pub usage_reported_request_count: usize,
    pub usage_missing_request_count: usize,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub cached_input_tokens: u64,
    pub reasoning_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmUsageArtifact {
    pub schema_version: String,
    pub mode: String,
    #[serde(default)]
    pub configured_model: Option<String>,
    #[serde(default)]
    pub requests: Vec<LlmRequestUsageRecord>,
    pub summary: LlmUsageSummary,
}

impl LlmUsageArtifact {
    pub fn new(mode: &str, configured_model: Option<String>) -> Self {
        Self {
            schema_version: LLM_USAGE_SCHEMA_VERSION.to_string(),
            mode: mode.to_string(),
            configured_model,
            requests: Vec::new(),
            summary: LlmUsageSummary::default(),
        }
    }

    pub fn push(&mut self, record: LlmRequestUsageRecord) {
        self.requests.push(record);
        self.refresh_summary();
    }

    pub fn refresh_summary(&mut self) {
        let mut summary = LlmUsageSummary {
            request_count: self.requests.len(),
            ..LlmUsageSummary::default()
        };
        for record in &self.requests {
            if record.outcome == "success" {
                summary.successful_request_count += 1;
            } else {
                summary.failed_request_count += 1;
            }
            if record.usage.source == "missing" {
                summary.usage_missing_request_count += 1;
            } else {
                summary.usage_reported_request_count += 1;
            }
            summary.input_tokens += record.usage.input_tokens.unwrap_or(0);
            summary.output_tokens += record.usage.output_tokens.unwrap_or(0);
            summary.total_tokens += record.usage.total_tokens.unwrap_or(0);
            summary.cached_input_tokens += record.usage.cached_input_tokens.unwrap_or(0);
            summary.reasoning_tokens += record.usage.reasoning_tokens.unwrap_or(0);
        }
        self.summary = summary;
    }
}

pub fn extract_token_usage(root: &Value) -> LlmTokenUsage {
    let usage = root
        .get("usage")
        .or_else(|| root.get("usageMetadata"))
        .cloned();
    let Some(raw_usage) = usage else {
        return LlmTokenUsage {
            source: "missing".to_string(),
            ..LlmTokenUsage::default()
        };
    };

    let input_tokens = first_u64(
        &raw_usage,
        &["prompt_tokens", "input_tokens", "promptTokenCount"],
    );
    let output_tokens = first_u64(
        &raw_usage,
        &[
            "completion_tokens",
            "output_tokens",
            "candidatesTokenCount",
        ],
    );
    let provider_total = first_u64(&raw_usage, &["total_tokens", "totalTokenCount"]);
    let computed_total = input_tokens.zip(output_tokens).map(|(input, output)| input + output);
    let total_tokens = provider_total.or(computed_total);
    let cached_input_tokens = raw_usage
        .pointer("/prompt_tokens_details/cached_tokens")
        .and_then(Value::as_u64)
        .or_else(|| first_u64(&raw_usage, &["cache_read_input_tokens", "cachedContentTokenCount"]));
    let reasoning_tokens = raw_usage
        .pointer("/completion_tokens_details/reasoning_tokens")
        .and_then(Value::as_u64)
        .or_else(|| first_u64(&raw_usage, &["reasoning_tokens", "thoughtsTokenCount"]));
    let any_reported = input_tokens.is_some()
        || output_tokens.is_some()
        || total_tokens.is_some()
        || cached_input_tokens.is_some()
        || reasoning_tokens.is_some();

    LlmTokenUsage {
        input_tokens,
        output_tokens,
        total_tokens,
        cached_input_tokens,
        reasoning_tokens,
        source: if any_reported {
            if provider_total.is_some() {
                "provider_reported".to_string()
            } else {
                "computed_from_components".to_string()
            }
        } else {
            "missing".to_string()
        },
        total_matches_components: provider_total.zip(computed_total).map(|(a, b)| a == b),
        raw_usage: Some(raw_usage),
    }
}

fn first_u64(root: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| root.get(*key).and_then(Value::as_u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_chat_completion_fields_over_zero_aliases() {
        let root = serde_json::json!({
            "usage": {
                "prompt_tokens": 3024,
                "completion_tokens": 1190,
                "total_tokens": 4214,
                "input_tokens": 0,
                "output_tokens": 0,
                "prompt_tokens_details": {"cached_tokens": 128},
                "completion_tokens_details": {"reasoning_tokens": 64}
            }
        });
        let usage = extract_token_usage(&root);
        assert_eq!(usage.input_tokens, Some(3024));
        assert_eq!(usage.output_tokens, Some(1190));
        assert_eq!(usage.total_tokens, Some(4214));
        assert_eq!(usage.cached_input_tokens, Some(128));
        assert_eq!(usage.reasoning_tokens, Some(64));
        assert_eq!(usage.total_matches_components, Some(true));
    }

    #[test]
    fn supports_gemini_usage_metadata() {
        let root = serde_json::json!({
            "usageMetadata": {
                "promptTokenCount": 100,
                "candidatesTokenCount": 25,
                "totalTokenCount": 125,
                "cachedContentTokenCount": 10,
                "thoughtsTokenCount": 5
            }
        });
        let usage = extract_token_usage(&root);
        assert_eq!(usage.input_tokens, Some(100));
        assert_eq!(usage.output_tokens, Some(25));
        assert_eq!(usage.total_tokens, Some(125));
        assert_eq!(usage.cached_input_tokens, Some(10));
        assert_eq!(usage.reasoning_tokens, Some(5));
    }

    #[test]
    fn computes_total_when_provider_omits_it() {
        let root = serde_json::json!({
            "usage": {"input_tokens": 7, "output_tokens": 3}
        });
        let usage = extract_token_usage(&root);
        assert_eq!(usage.total_tokens, Some(10));
        assert_eq!(usage.source, "computed_from_components");
    }

    #[test]
    fn marks_usage_missing_without_treating_it_as_zero() {
        let usage = extract_token_usage(&serde_json::json!({"choices": []}));
        assert_eq!(usage.input_tokens, None);
        assert_eq!(usage.total_tokens, None);
        assert_eq!(usage.source, "missing");
    }
}
