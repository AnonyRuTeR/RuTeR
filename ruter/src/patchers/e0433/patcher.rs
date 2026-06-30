use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::core::{
    Applicability, Diagnostic, ErrorCode, FixAction, Result, RuTeRError, SpanInfo,
};
use crate::patchers::Patcher;
use crate::patchers::common::compiler_suggestion::CompilerSuggestionExtractor;
use regex::Regex;
use toml::Value as TomlValue;

use super::context::TestContextDetector;
use super::fix_generator::FixGenerator;
use super::path_resolver::ResolutionSource;
use super::path_resolver::{PathResolver, RankedResolution, ResolutionResult, ResolverContext};
use super::reachability_index::SymbolReachabilityIndex;
use super::span_analyzer::SpanAnalyzer;
use super::types::{CandidateFix, CandidateKind, Confidence, ExtendedSpan, PathSegment};

static RE_CRATE_IDENT: OnceLock<Regex> = OnceLock::new();
static RE_NOT_FOUND_IN_CRATE_ROOT: OnceLock<Regex> = OnceLock::new();
static RE_MISSING_CRATE: OnceLock<Regex> = OnceLock::new();
static RE_UNDECLARED: OnceLock<Regex> = OnceLock::new();
static RE_NOT_FOUND: OnceLock<Regex> = OnceLock::new();
static RE_UNLINKED_CRATE: OnceLock<Regex> = OnceLock::new();

/// E0433 Patcher
/// Integrates suggestion extraction, path resolution,
/// span analysis and fix generation.
#[derive(Debug, Default)]
pub struct E0433Patcher;

#[derive(Debug, Clone)]
pub struct RankedFixAction {
    pub action: FixAction,
    pub score: i32,
    pub path: String,
    pub confidence: Confidence,
    pub source: ResolutionSource,
}

impl E0433Patcher {
    /// Create a new instance of E0433Patcher
    pub fn new() -> Self {
        Self
    }

    /// Pick primary span first;
    /// Fallback to first span if compiler does not provide one.
    fn primary_span(diagnostic: &Diagnostic) -> Option<&SpanInfo> {
        diagnostic
            .span
            .iter()
            .find(|s| s.is_primary)
            .or_else(|| diagnostic.span.first())
    }

    /// Load source file.
    /// And map error to RuTeRError if file not found.
    fn load_source(path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                RuTeRError::SourceFileNotFound(path.display().to_string())
            } else {
                RuTeRError::IoError(e)
            }
        })
    }

    /// Validate byte offsets
    /// Before slicing source text.
    fn validate_span(span: &SpanInfo, source: &str) -> Result<()> {
        let src_len = source.len();
        if span.byte_start > src_len || span.byte_end > src_len || span.byte_start >= span.byte_end
        {
            return Err(RuTeRError::ParseError(format!(
                "Invalid byte range {}..{} for source length = {}",
                span.byte_start, span.byte_end, src_len
            )));
        }
        Ok(())
    }

    fn extract_unresolved_head(message: &str) -> Option<String> {
        let re = RE_NOT_FOUND_IN_CRATE_ROOT.get_or_init(|| {
            Regex::new(r"could not find `([^`]+)` in (?:the )?crate root")
                .expect("valid crate-root not-found regex")
        });
        if let Some(caps) = re.captures(message) {
            return Some(caps[1].to_string());
        }

        let re = RE_MISSING_CRATE.get_or_init(|| {
            Regex::new(r"(?:you might be missing crate|maybe a missing crate) `([^`]+)`")
                .expect("valid missing-crate regex")
        });
        if let Some(caps) = re.captures(message) {
            return Some(caps[1].to_string());
        }

        let re = RE_CRATE_IDENT.get_or_init(|| {
            Regex::new(
                r"(?:unresolved module|unlinked crate|undeclared crate|undeclared module) `([^`]+)`",
            )
            .unwrap()
        });
        re.captures(message).map(|caps| caps[1].to_string())
    }

    fn extract_identifier(message: &str) -> Option<String> {
        let re = RE_UNDECLARED.get_or_init(|| {
            Regex::new(r"use of undeclared (?:crate|module|type|value|method) `([^`]+)`")
                .expect("valid undeclared regex")
        });
        if let Some(caps) = re.captures(message) {
            return Some(caps[1].to_string());
        }

        let re = RE_NOT_FOUND.get_or_init(|| {
            Regex::new(r"could not find `([^`]+)`").expect("valid not-found regex")
        });
        if let Some(caps) = re.captures(message) {
            return Some(caps[1].to_string());
        }

        let re = RE_UNLINKED_CRATE.get_or_init(|| {
            Regex::new(r"use of unresolved module or unlinked crate `([^`]+)`")
                .expect("valid unlinked-crate regex")
        });
        re.captures(message).map(|caps| caps[1].to_string())
    }

    fn extract_bare_identifier(path: &str) -> String {
        path.split("::").last().unwrap_or(path).to_string()
    }

    fn applicability_to_confidence(applicability: Option<Applicability>) -> Confidence {
        match applicability {
            Some(Applicability::MachineApplicable) => Confidence::High,
            Some(Applicability::MaybeIncorrect) => Confidence::Medium,
            Some(Applicability::HasPlaceholders) => Confidence::Low,
            Some(Applicability::Unspecified) | None => Confidence::Medium,
        }
    }

    fn promote_structured_path_confidence(path: &str, confidence: Confidence) -> Confidence {
        if confidence == Confidence::High {
            return confidence;
        }
        if path.contains("::") {
            return Confidence::High;
        }
        confidence
    }

    fn compiler_suggestion_candidates(diagnostic: &Diagnostic) -> Vec<CandidateFix> {
        let identifier_hint = Self::extract_identifier(&diagnostic.message);
        let mut out = Vec::new();

        for suggestion in CompilerSuggestionExtractor::extract(diagnostic) {
            let Some(path) = suggestion.normalized_text else {
                continue;
            };

            let confidence = Self::promote_structured_path_confidence(
                &path,
                Self::applicability_to_confidence(suggestion.applicability),
            );
            let kind = if suggestion.raw.trim_start().starts_with("use ") {
                CandidateKind::UseImport
            } else {
                CandidateKind::PathRewrite
            };

            let bare_identifier = identifier_hint
                .clone()
                .unwrap_or_else(|| Self::extract_bare_identifier(&path));

            out.push(CandidateFix {
                bare_identifier,
                suggested_path: Some(path),
                confidence,
                kind,
                blocked_reason: None,
            });
        }

        if out.is_empty() {
            if let Some(identifier) = identifier_hint {
                out.push(CandidateFix {
                    bare_identifier: identifier.clone(),
                    suggested_path: Some(identifier),
                    confidence: Confidence::Medium,
                    kind: CandidateKind::PathRewrite,
                    blocked_reason: None,
                });
            }
        }

        out
    }

    fn locate_manifest(start: &Path) -> Option<PathBuf> {
        let mut cursor = if start.is_file() {
            start.parent()?
        } else {
            start
        };

        loop {
            let candidate = cursor.join("Cargo.toml");
            if candidate.exists() {
                return Some(candidate);
            }

            cursor = cursor.parent()?;
        }
    }

    fn load_package_name_for_file(source_file: &Path) -> Option<String> {
        let manifest_path = Self::locate_manifest(source_file)?;
        let manifest_raw = fs::read_to_string(manifest_path).ok()?;
        let manifest: TomlValue = manifest_raw.parse().ok()?;

        manifest
            .get("package")
            .and_then(|pkg| pkg.get("name"))
            .and_then(|name| name.as_str())
            .map(|name| name.to_string())
    }

    fn package_name_prefix_candidate(
        diagnostic: &Diagnostic,
        primary_span: &SpanInfo,
        extended_span: &ExtendedSpan,
    ) -> Option<CandidateFix> {
        let unresolved_ident = Self::extract_unresolved_head(&diagnostic.message)?;
        let package_name = Self::load_package_name_for_file(&primary_span.file_path)?;

        if unresolved_ident != package_name {
            return None;
        }

        let first_segment = extended_span.segments.first()?;
        if first_segment.name != package_name {
            return None;
        }

        let second_segment = extended_span.segments.get(1)?;
        let suggested_path = format!("crate::{}", second_segment.name);

        Some(CandidateFix {
            bare_identifier: package_name,
            suggested_path: Some(suggested_path),
            confidence: Confidence::High,
            kind: CandidateKind::PathRewrite,
            blocked_reason: None,
        })
    }

    fn collect_candidate_crate_root(source_file: &Path) -> Option<PathBuf> {
        let manifest_path = Self::locate_manifest(source_file)?;
        manifest_path.parent().map(Path::to_path_buf)
    }

    fn manifest_declares_dependency(manifest: &TomlValue, head: &str) -> bool {
        fn table_declares(table: Option<&TomlValue>, head: &str) -> bool {
            let Some(table) = table.and_then(TomlValue::as_table) else {
                return false;
            };

            if table.contains_key(head) {
                return true;
            }

            table.values().any(|dep| {
                dep.as_table()
                    .and_then(|value| value.get("package"))
                    .and_then(TomlValue::as_str)
                    .map(|pkg| pkg == head)
                    .unwrap_or(false)
            })
        }

        if table_declares(manifest.get("dependencies"), head) {
            return true;
        }
        if table_declares(manifest.get("dev-dependencies"), head) {
            return true;
        }

        manifest
            .get("workspace")
            .and_then(|value| value.get("dependencies"))
            .map(|deps| table_declares(Some(deps), head))
            .unwrap_or(false)
    }

    fn build_r1_path_from_segments(segments: &[PathSegment]) -> Option<String> {
        if segments.len() < 2 {
            return None;
        }
        let tail = segments[1..]
            .iter()
            .map(|segment| segment.name.as_str())
            .collect::<Vec<_>>();
        Some(format!("crate::{}", tail.join("::")))
    }

    fn build_r1_path_keep_head_from_segments(segments: &[PathSegment]) -> Option<String> {
        if segments.is_empty() {
            return None;
        }
        let full = segments
            .iter()
            .map(|segment| segment.name.as_str())
            .collect::<Vec<_>>();
        Some(format!("crate::{}", full.join("::")))
    }

    fn has_immediate_crate_prefix_before_span(source: &str, byte_start: usize) -> bool {
        const PREFIX: &str = "crate::";
        let bytes = source.as_bytes();
        let mut cursor = byte_start.min(bytes.len());
        while cursor > 0 && bytes[cursor - 1].is_ascii_whitespace() {
            cursor -= 1;
        }
        if cursor < PREFIX.len() {
            return false;
        }
        source
            .get(cursor - PREFIX.len()..cursor)
            .map(|slice| slice == PREFIX)
            .unwrap_or(false)
    }

    fn is_known_implicit_extern_crate_head(head: &str) -> bool {
        matches!(head, "core" | "std" | "alloc" | "proc_macro" | "test")
    }

    fn build_path_from_segments(segments: &[PathSegment]) -> Option<String> {
        if segments.is_empty() {
            return None;
        }
        Some(
            segments
                .iter()
                .map(|segment| segment.name.as_str())
                .collect::<Vec<_>>()
                .join("::"),
        )
    }

    /// R3 heuristic:
    /// when path is `crate::<extern_crate>::...` and rustc reports missing crate,
    /// rewrite head to extern prelude path by dropping leading `crate::`.
    fn missing_crate_head_drop_prefix_candidate(
        diagnostic: &Diagnostic,
        primary_span: &SpanInfo,
        extended_span: &ExtendedSpan,
        source: &str,
        is_test_context: bool,
    ) -> Option<CandidateFix> {
        if !is_test_context {
            return None;
        }

        let head = Self::extract_unresolved_head(&diagnostic.message)?;
        let first_segment = extended_span.segments.first()?;
        if first_segment.name != head {
            return None;
        }

        if !Self::has_immediate_crate_prefix_before_span(source, extended_span.extended_span.byte_start) {
            return None;
        }

        // 避免与 `crate::<package_name>::...` 规则冲突。
        if Self::load_package_name_for_file(&primary_span.file_path)
            .map(|name| name == head)
            .unwrap_or(false)
        {
            return None;
        }

        // 轻量泛化：
        // 1) 已知隐式 extern crate（core/std/...）
        // 2) Cargo.toml 声明了同名依赖
        // 3) 诊断文案显式提到“missing crate”
        let manifest_declares = Self::locate_manifest(&primary_span.file_path)
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|raw| raw.parse::<TomlValue>().ok())
            .map(|manifest| Self::manifest_declares_dependency(&manifest, &head))
            .unwrap_or(false);
        let message_mentions_missing_crate = diagnostic.message.contains("missing crate");
        if !Self::is_known_implicit_extern_crate_head(&head)
            && !manifest_declares
            && !message_mentions_missing_crate
        {
            return None;
        }

        let suggested_path = Self::build_path_from_segments(&extended_span.segments)?;
        Some(CandidateFix {
            bare_identifier: head,
            suggested_path: Some(suggested_path),
            confidence: Confidence::High,
            kind: CandidateKind::PathRewrite,
            blocked_reason: None,
        })
    }

    /// R1 heuristic:
    /// when head is unresolved and matches crate name, and tail is uniquely reachable,
    /// suggest rewriting head to crate and keep tail.
    fn unresolved_head_to_crate_candidate(
        diagnostic: &Diagnostic,
        primary_span: &SpanInfo,
        extended_span: &ExtendedSpan,
        is_test_context: bool,
    ) -> Option<CandidateFix> {
        if !is_test_context {
            return None;
        }

        // 提取未解析的 head 标识符，并验证它是否与路径的第一个 segment 匹配。
        let head = Self::extract_unresolved_head(&diagnostic.message)?;
        let first_segment = extended_span.segments.first()?;
        if first_segment.name != head || extended_span.segments.len() < 2 {
            return None;
        }

        // 检查 Cargo.toml 中是否已经声明了同名依赖，如果有则不建议添加路径重写，避免误导用户。
        let manifest_path = Self::locate_manifest(&primary_span.file_path)?;
        let manifest_raw = fs::read_to_string(&manifest_path).ok()?;
        let manifest: TomlValue = manifest_raw.parse().ok()?;
        if Self::manifest_declares_dependency(&manifest, &head) {
            return None;
        }

        let crate_root = Self::collect_candidate_crate_root(&primary_span.file_path)?;
        let reachability = SymbolReachabilityIndex::shared(&crate_root).ok()?;

        // 如果 unresolved head 本身就是 crate 根模块（例如 `v0::Parser`），
        // 则应保留 head，改为 `crate::v0::Parser`，而不是错误地丢弃 head。
        if reachability.is_crate_visible_module(&head) {
            let suggested_path = Self::build_r1_path_keep_head_from_segments(&extended_span.segments)?;
            return Some(CandidateFix {
                bare_identifier: head,
                suggested_path: Some(suggested_path),
                confidence: Confidence::High,
                kind: CandidateKind::PathRewrite,
                blocked_reason: None,
            });
        }

        // 验证 tail 是否在当前 crate 中唯一可达，避免提供错误的修复建议。
        let tail1 = &extended_span.segments[1].name;
        // 在 unresolved_head::tail... 形态下，tail1 可能是 crate 根模块名（例如 wrapper）。
        // 这类路径在测试模块里合法，即使模块不是 `pub mod`。
        if !reachability.is_crate_visible_module(tail1) && !reachability.is_uniquely_reachable(tail1)
        {
            return None;
        }

        let suggested_path = Self::build_r1_path_from_segments(&extended_span.segments)?;
        Some(CandidateFix {
            bare_identifier: head,
            suggested_path: Some(suggested_path),
            confidence: Confidence::High,
            kind: CandidateKind::PathRewrite,
            blocked_reason: None,
        })
    }

    fn push_candidate_unique(candidates: &mut Vec<CandidateFix>, candidate: CandidateFix) {
        let duplicate = candidates.iter().any(|existing| {
            existing.suggested_path == candidate.suggested_path && existing.kind == candidate.kind
        });
        if !duplicate {
            candidates.push(candidate);
        }
    }

    fn build_candidates(
        diagnostic: &Diagnostic,
        primary_span: &SpanInfo,
        extended_span: &ExtendedSpan,
        source: &str,
        is_test_context: bool,
    ) -> Vec<CandidateFix> {
        let mut candidates = Self::compiler_suggestion_candidates(diagnostic);
        if let Some(heuristic) =
            Self::package_name_prefix_candidate(diagnostic, primary_span, extended_span)
        {
            Self::push_candidate_unique(&mut candidates, heuristic);
        }
        if let Some(heuristic) = Self::missing_crate_head_drop_prefix_candidate(
            diagnostic,
            primary_span,
            extended_span,
            source,
            is_test_context,
        ) {
            Self::push_candidate_unique(&mut candidates, heuristic);
        }
        if let Some(heuristic) = Self::unresolved_head_to_crate_candidate(
            diagnostic,
            primary_span,
            extended_span,
            is_test_context,
        ) {
            Self::push_candidate_unique(&mut candidates, heuristic);
        }

        candidates
    }

    /// Generate top-k ranked fix actions for one E0433 diagnostic.
    ///
    /// # Parameters
    /// - `diagnostic`: one compiler diagnostic.
    /// - `k`: max number of high-confidence actions to return.
    pub fn analyze_top_k(&self, diagnostic: &Diagnostic, k: usize) -> Result<Vec<RankedFixAction>> {
        if k == 0 {
            return Ok(Vec::new());
        }

        // If not E0433, return empty
        if !self.can_handle(diagnostic) {
            return Ok(vec![]);
        }

        let primary_span = Self::primary_span(diagnostic).ok_or_else(|| {
            RuTeRError::ParseError("E0433 diagnostic has no span".to_string())
        })?;

        let source = Self::load_source(&primary_span.file_path)?;
        Self::validate_span(primary_span, &source)?;

        // 只允许测试语境修复，避免误改生产代码。
        let is_test_context = TestContextDetector::is_test_context(primary_span, &source);
        if !is_test_context {
            return Ok(vec![]);
        }

        let extended_span = SpanAnalyzer::analyze(primary_span, &source);
        let candidates =
            Self::build_candidates(diagnostic, primary_span, &extended_span, &source, is_test_context);

        let ranked = PathResolver::rank_with_context(
            candidates,
            ResolverContext {
                is_test_context,
                allow_dependency_action_in_tests: false,
            },
        );

        Ok(Self::ranked_to_fix_actions(&extended_span, &source, ranked, k))
    }

    fn ranked_to_fix_actions(
        extended_span: &ExtendedSpan,
        source: &str,
        ranked: Vec<RankedResolution>,
        k: usize,
    ) -> Vec<RankedFixAction> {
        let mut out = Vec::new();
        for candidate in ranked.into_iter().take(k) {
            let resolution = ResolutionResult::Resolved {
                path: candidate.path.clone(),
                confidence: candidate.confidence,
                source: candidate.source,
            };

            if let Some(action) = FixGenerator::generate(extended_span, &resolution, source) {
                out.push(RankedFixAction {
                    action,
                    score: candidate.score,
                    path: candidate.path,
                    confidence: candidate.confidence,
                    source: candidate.source,
                });
            }
        }
        out
    }
}

impl Patcher for E0433Patcher {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::E0433
    }

    /// Analyze one diagnostic and generate fix actions
    ///
    /// # Returns:
    /// - Ok(Vec<FixAction>) if fix actions are generated successfully
    /// - Err(RuTeRError) if any error occurs during analysis or generation
    /// # Notes:
    /// - Currently E0433Patcher only generates one fix action per diagnostic
    fn analyze(&self, diagnostic: &Diagnostic) -> Result<Vec<FixAction>> {
        let top1 = self.analyze_top_k(diagnostic, 1)?;
        Ok(top1.into_iter().map(|item| item.action).collect())
    }

    fn description(&self) -> &'static str {
        "Patcher for E0433: patch unresolved module/type path errors"
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
