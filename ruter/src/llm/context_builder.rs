use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use ruter::core::{FunctionDiagnostic, TestFunction};

const CONTEXT_SCHEMA_VERSION: &str = "1";
const DEFAULT_DIAGNOSTIC_ITEMS_LIMIT: usize = 4;

mod budget;
mod symbols;

use budget::prune_bundle_to_budget;
use symbols::{
    collect_diagnostic_symbol_hints, collect_direct_symbols, collect_related_fn_defs,
    collect_related_impl_blocks, collect_related_imports, collect_related_type_defs,
};

/// Function-level context artifact consumed by prompt builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionContextBundleV1 {
    pub context_schema_version: String,
    pub function_id: String,
    pub location: ContextLocation,
    pub target_function_text: String,
    pub diagnostics_digest: DiagnosticsDigest,
    pub constraints: ContextConstraints,
    #[serde(default)]
    pub related_imports: Vec<String>,
    #[serde(default)]
    pub related_type_defs: Vec<String>,
    #[serde(default)]
    pub related_impl_blocks: Vec<String>,
    #[serde(default)]
    pub related_fn_defs: Vec<String>,
    #[serde(default)]
    pub neighbor_fix_digest: Vec<String>,
    #[serde(default)]
    pub local_rule_failure_digest: Option<LocalRuleFailureDigestV1>,
    #[serde(default)]
    pub rule_error_drift_digest: Option<RuleErrorDriftDigestV1>,
    #[serde(default)]
    pub preflight_interceptor_digest: Option<PreflightInterceptorDigestV1>,
    #[serde(default)]
    pub previous_round_failure_digest: Option<PreviousRoundFailureDigestV1>,
    #[serde(default)]
    pub truncated_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLocation {
    pub file_path_redacted: String,
    pub module_path: String,
    pub fn_name: String,
    pub line_span: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsDigest {
    pub error_code_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub primary_items: Vec<PrimaryDiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryDiagnosticItem {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub primary_span: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub suggested_replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConstraints {
    pub test_only: bool,
    pub same_function_only: bool,
    pub signature_attrs_immutable: bool,
    pub no_non_target_regression: bool,
}

/// 上一轮失败摘要（仅同 function_id 且仅 round-1）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreviousRoundFailureDigestV1 {
    pub round: u8,
    #[serde(default)]
    pub dominant_failure_kinds: Vec<String>,
    #[serde(default)]
    pub candidate_failures: Vec<PreviousCandidateFailureDigestV1>,
}

/// 上一轮某个候选（或轮级事件）的失败摘要。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreviousCandidateFailureDigestV1 {
    pub candidate_id: String,
    pub patch_summary: String,
    pub failure_kind: String,
    #[serde(default)]
    pub failure_detail: Option<String>,
    #[serde(default)]
    pub unresolved_error_by_code: BTreeMap<String, usize>,
    #[serde(default)]
    pub introduced_error_by_code: BTreeMap<String, usize>,
}

/// 本地规则修复失败摘要（来自 rule patcher 本地验证阶段）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalRuleFailureDigestV1 {
    #[serde(default)]
    pub summary_lines: Vec<String>,
}

/// 规则修复后错误码漂移摘要（例如 E0433 -> E0599）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleErrorDriftDigestV1 {
    #[serde(default)]
    pub original_error_by_code: BTreeMap<String, usize>,
    #[serde(default)]
    pub current_error_by_code: BTreeMap<String, usize>,
    #[serde(default)]
    pub drift_pairs: Vec<RuleErrorDriftPairV1>,
    #[serde(default)]
    pub failed_rule_fix_summaries: Vec<String>,
    #[serde(default)]
    pub original_target_function_text: Option<String>,
}

/// 单个错误码漂移映射（from -> to）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleErrorDriftPairV1 {
    pub from_code: String,
    pub to_code: String,
    pub from_count: usize,
    pub to_count: usize,
}

/// Pre-flight interceptor summary (LowValue / E0599).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreflightInterceptorDigestV1 {
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContextBuildConfig {
    pub max_chars: usize,
    pub target_fn_hard_limit_chars: usize,
    pub primary_items_limit: usize,
}

impl Default for ContextBuildConfig {
    fn default() -> Self {
        Self {
            max_chars: 12_000,
            target_fn_hard_limit_chars: 8_000,
            primary_items_limit: DEFAULT_DIAGNOSTIC_ITEMS_LIMIT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContextBuildOutcome {
    Ready {
        bundle: FunctionContextBundleV1,
        char_count: usize,
    },
    TooLarge {
        function_id: String,
        char_count: usize,
        max_chars: usize,
        reason: String,
    },
}

/// Build context artifact for one target function.
pub fn build_context_bundle_v1(
    _crate_path: &Path,
    function: &TestFunction,
    source: &str,
    diagnostics: &[FunctionDiagnostic],
    neighbor_fix_digest: Vec<String>,
    local_rule_failure_digest: Option<LocalRuleFailureDigestV1>,
    rule_error_drift_digest: Option<RuleErrorDriftDigestV1>,
    preflight_interceptor_digest: Option<PreflightInterceptorDigestV1>,
    previous_round_failure_digest: Option<PreviousRoundFailureDigestV1>,
    cfg: &ContextBuildConfig,
) -> ContextBuildOutcome {
    let start = function.byte_start.min(source.len());
    let end = function.byte_end.min(source.len());
    if start >= end {
        return ContextBuildOutcome::TooLarge {
            function_id: function.id.clone(),
            char_count: 0,
            max_chars: cfg.max_chars,
            reason: "invalid function byte range".to_string(),
        };
    }

    let target_function_text = source[start..end].to_string();
    let target_chars = target_function_text.chars().count();
    if target_chars > cfg.target_fn_hard_limit_chars {
        return ContextBuildOutcome::TooLarge {
            function_id: function.id.clone(),
            char_count: target_chars,
            max_chars: cfg.target_fn_hard_limit_chars,
            reason: "target function text exceeds hard limit".to_string(),
        };
    }

    let syntax = match syn::parse_file(source) {
        Ok(file) => file,
        Err(err) => {
            return ContextBuildOutcome::TooLarge {
                function_id: function.id.clone(),
                char_count: target_chars,
                max_chars: cfg.max_chars,
                reason: format!("failed to parse source file for context extraction: {err}"),
            };
        }
    };

    let direct_symbols = collect_direct_symbols(&target_function_text);
    let diagnostic_symbol_hints = collect_diagnostic_symbol_hints(diagnostics);

    let mut related_imports = collect_related_imports(&syntax, &direct_symbols);
    let mut related_type_defs = collect_related_type_defs(&syntax, &direct_symbols);
    let mut related_impl_blocks = collect_related_impl_blocks(&syntax, &direct_symbols);
    let mut related_fn_defs = collect_related_fn_defs(&syntax, &diagnostic_symbol_hints);

    let diagnostics_digest = build_diagnostics_digest(
        diagnostics,
        cfg.primary_items_limit,
        &function.relative_file,
    );
    let location = ContextLocation {
        file_path_redacted: redact_file_path(&function.relative_file),
        module_path: function.module_path.join("::"),
        fn_name: function.fn_name.clone(),
        line_span: format!("{}:{}", function.line_start, function.line_end),
    };

    let mut bundle = FunctionContextBundleV1 {
        context_schema_version: CONTEXT_SCHEMA_VERSION.to_string(),
        function_id: function.id.clone(),
        location,
        target_function_text,
        diagnostics_digest,
        constraints: ContextConstraints {
            test_only: true,
            same_function_only: true,
            signature_attrs_immutable: true,
            no_non_target_regression: true,
        },
        related_imports: Vec::new(),
        related_type_defs: Vec::new(),
        related_impl_blocks: Vec::new(),
        related_fn_defs: Vec::new(),
        neighbor_fix_digest,
        local_rule_failure_digest,
        rule_error_drift_digest,
        preflight_interceptor_digest,
        previous_round_failure_digest,
        truncated_sections: Vec::new(),
    };

    // 先全量填充，再按固定顺序裁剪。
    bundle.related_imports.append(&mut related_imports);
    bundle.related_type_defs.append(&mut related_type_defs);
    bundle.related_impl_blocks.append(&mut related_impl_blocks);
    bundle.related_fn_defs.append(&mut related_fn_defs);

    prune_bundle_to_budget(&mut bundle, cfg.max_chars)
}

fn build_diagnostics_digest(
    diagnostics: &[FunctionDiagnostic],
    primary_items_limit: usize,
    relative_file: &Path,
) -> DiagnosticsDigest {
    let mut error_code_counts = BTreeMap::new();
    for diag in diagnostics {
        *error_code_counts.entry(diag.code.clone()).or_insert(0) += 1;
    }

    let mut primary_items = diagnostics
        .iter()
        .take(primary_items_limit)
        .map(|diag| PrimaryDiagnosticItem {
            code: diag.code.clone(),
            message: diag.message.clone(),
            primary_span: diag
                .primary_span
                .as_deref()
                .map(|span| redact_primary_span(span, relative_file)),
            label: diag.label.clone(),
            suggested_replacement: diag.suggested_replacement.clone(),
        })
        .collect::<Vec<_>>();

    primary_items.sort_by(|a, b| {
        a.code
            .cmp(&b.code)
            .then_with(|| a.message.cmp(&b.message))
            .then_with(|| a.primary_span.cmp(&b.primary_span))
    });

    DiagnosticsDigest {
        error_code_counts,
        primary_items,
    }
}

fn redact_file_path(relative_file: &Path) -> String {
    let rel = if relative_file.is_absolute() {
        relative_file
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("unknown.rs"))
    } else {
        relative_file.to_path_buf()
    };
    format!("<CRATE_ROOT>/{}", rel.display())
}

fn redact_primary_span(span: &str, relative_file: &Path) -> String {
    let redacted_file = redact_file_path(relative_file);
    let mut suffix_start = None;
    for (idx, ch) in span.char_indices() {
        if ch != ':' {
            continue;
        }
        if span
            .get(idx + 1..)
            .and_then(|tail| tail.chars().next())
            .map(|next| next.is_ascii_digit())
            .unwrap_or(false)
        {
            suffix_start = Some(idx + 1);
            break;
        }
    }

    match suffix_start {
        Some(start) => format!("{redacted_file}:{}", &span[start..]),
        None => redacted_file,
    }
}

#[cfg(test)]
mod tests;
