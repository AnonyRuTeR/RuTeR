use crate::core::{Applicability, Diagnostic, SpanInfo};
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

static RE_USE: OnceLock<Regex> = OnceLock::new();
static RE_CONSIDER_IMPORTING: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuggestionSource {
    Span,
    ChildMessage,
    MainMessage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedSuggestion {
    pub raw: String,
    pub normalized_text: Option<String>,
    pub source: SuggestionSource,
    pub applicability: Option<Applicability>,
    pub executable: bool,
    pub blocked_reason: Option<String>,
}

pub struct CompilerSuggestionExtractor;

impl CompilerSuggestionExtractor {
    pub fn extract(diagnostic: &Diagnostic) -> Vec<NormalizedSuggestion> {
        let mut suggestions = Self::extract_all(diagnostic)
            .into_iter()
            .filter(|item| item.executable)
            .collect::<Vec<_>>();
        Self::dedup(&mut suggestions);
        suggestions
    }

    pub fn extract_all(diagnostic: &Diagnostic) -> Vec<NormalizedSuggestion> {
        let mut out = Vec::new();

        Self::extract_from_spans(&diagnostic.span, &mut out);

        for child in &diagnostic.children {
            Self::extract_from_spans(&child.span, &mut out);
            if let Some(raw) = Self::extract_path_from_message(&child.message) {
                out.push(Self::normalize_raw_suggestion(
                    &raw,
                    SuggestionSource::ChildMessage,
                    None,
                ));
            }
        }

        if let Some(raw) = Self::extract_path_from_message(&diagnostic.message) {
            out.push(Self::normalize_raw_suggestion(
                &raw,
                SuggestionSource::MainMessage,
                None,
            ));
        }

        Self::dedup(&mut out);
        out
    }

    fn extract_from_spans(spans: &[SpanInfo], out: &mut Vec<NormalizedSuggestion>) {
        for span in spans {
            if let Some(raw) = span.suggested_replacement.as_deref() {
                out.push(Self::normalize_raw_suggestion(
                    raw,
                    SuggestionSource::Span,
                    span.suggestion_applicability,
                ));
            }
        }
    }

    fn normalize_raw_suggestion(
        raw: &str,
        source: SuggestionSource,
        applicability: Option<Applicability>,
    ) -> NormalizedSuggestion {
        let raw_trimmed = raw.trim().to_string();

        if Self::is_cargo_command(&raw_trimmed) {
            return NormalizedSuggestion {
                raw: raw_trimmed,
                normalized_text: None,
                source,
                applicability,
                executable: false,
                blocked_reason: Some("command hint from diagnostic help".to_string()),
            };
        }

        if let Some(path) = Self::normalize_use_suggestion(&raw_trimmed) {
            if Self::is_rust_path_candidate(&path) {
                return NormalizedSuggestion {
                    raw: raw_trimmed,
                    normalized_text: Some(path),
                    source,
                    applicability,
                    executable: true,
                    blocked_reason: None,
                };
            }

            return NormalizedSuggestion {
                raw: raw_trimmed,
                normalized_text: None,
                source,
                applicability,
                executable: false,
                blocked_reason: Some("unsupported `use` suggestion text".to_string()),
            };
        }

        if Self::is_rust_path_candidate(&raw_trimmed) {
            return NormalizedSuggestion {
                raw: raw_trimmed.clone(),
                normalized_text: Some(raw_trimmed),
                source,
                applicability,
                executable: true,
                blocked_reason: None,
            };
        }

        NormalizedSuggestion {
            raw: raw_trimmed,
            normalized_text: None,
            source,
            applicability,
            executable: false,
            blocked_reason: Some("non-code suggestion text".to_string()),
        }
    }

    fn normalize_use_suggestion(raw: &str) -> Option<String> {
        let stripped = raw.strip_prefix("use ")?;
        let path = stripped.split(';').next().unwrap_or(stripped).trim();
        if path.is_empty() {
            return None;
        }
        Some(path.to_string())
    }

    fn is_cargo_command(raw: &str) -> bool {
        raw.trim().starts_with("cargo ")
    }

    fn extract_path_from_message(message: &str) -> Option<String> {
        let re = RE_USE.get_or_init(|| Regex::new(r"use `([^`]+)`").expect("valid use regex"));
        if let Some(caps) = re.captures(message) {
            return Some(caps[1].to_string());
        }

        let re = RE_CONSIDER_IMPORTING.get_or_init(|| {
            Regex::new(
                r"consider importing (?:this|the) (?:module|crate|type|value|method|struct|enum|trait|function): `([^`]+)`",
            )
            .expect("valid consider-importing regex")
        });
        if let Some(caps) = re.captures(message) {
            return Some(caps[1].to_string());
        }

        None
    }

    fn is_rust_path_candidate(raw: &str) -> bool {
        let trimmed = raw.trim();
        if trimmed.is_empty()
            || trimmed.contains(' ')
            || trimmed.contains('`')
            || trimmed.contains('\n')
        {
            return false;
        }

        trimmed.split("::").all(Self::is_valid_segment)
    }

    fn is_valid_segment(segment: &str) -> bool {
        if segment.is_empty() {
            return false;
        }

        if matches!(segment, "crate" | "self" | "super") {
            return true;
        }

        let mut chars = segment.chars();
        let Some(first) = chars.next() else {
            return false;
        };

        if !(first == '_' || first.is_ascii_alphabetic()) {
            return false;
        }

        chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    }

    fn dedup(suggestions: &mut Vec<NormalizedSuggestion>) {
        let mut seen = HashSet::new();
        suggestions.retain(|item| {
            let key = format!(
                "{}|{}|{}",
                item.normalized_text.as_deref().unwrap_or(""),
                item.executable,
                item.blocked_reason.as_deref().unwrap_or("")
            );
            seen.insert(key)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CompilerCode, Diagnostic, ErrorCode, Severity};
    use std::path::PathBuf;

    fn make_diag(
        message: &str,
        replacement: Option<&str>,
        child_message: Option<&str>,
        child_replacement: Option<&str>,
    ) -> Diagnostic {
        Diagnostic {
            message_type: Some("diagnostic".to_string()),
            code: Some(CompilerCode {
                code: ErrorCode::E0433,
                raw_code: None,
                explanation: None,
            }),
            message: message.to_string(),
            span: vec![SpanInfo {
                file_path: PathBuf::from("src/lib.rs"),
                byte_start: 0,
                byte_end: 1,
                line_start: 1,
                line_end: 1,
                col_start: 1,
                col_end: 2,
                is_primary: true,
                text: vec![],
                label: None,
                suggested_replacement: replacement.map(|value| value.to_string()),
                suggestion_applicability: Some(Applicability::MachineApplicable),
                expansion: None,
            }],
            severity: Severity::Error,
            children: child_message
                .map(|child_message| Diagnostic {
                    message_type: None,
                    code: None,
                    message: child_message.to_string(),
                    span: vec![SpanInfo {
                        file_path: PathBuf::from("src/lib.rs"),
                        byte_start: 0,
                        byte_end: 1,
                        line_start: 1,
                        line_end: 1,
                        col_start: 1,
                        col_end: 2,
                        is_primary: false,
                        text: vec![],
                        label: None,
                        suggested_replacement: child_replacement.map(|value| value.to_string()),
                        suggestion_applicability: Some(Applicability::MaybeIncorrect),
                        expansion: None,
                    }],
                    severity: Severity::Help,
                    children: vec![],
                    rendered: None,
                })
                .into_iter()
                .collect(),
            rendered: None,
        }
    }

    #[test]
    fn extract_from_span_normalizes_use_statement() {
        let diag = make_diag(
            "failed to resolve",
            Some("use crate::foo::State;\n"),
            None,
            None,
        );
        let out = CompilerSuggestionExtractor::extract(&diag);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].normalized_text.as_deref(), Some("crate::foo::State"));
        assert!(out[0].executable);
        assert_eq!(out[0].source, SuggestionSource::Span);
    }

    #[test]
    fn extract_from_child_message_backticks() {
        let diag = make_diag(
            "failed to resolve",
            None,
            Some("consider importing this module: `std::collections::HashMap`"),
            None,
        );
        let out = CompilerSuggestionExtractor::extract(&diag);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].normalized_text.as_deref(),
            Some("std::collections::HashMap")
        );
        assert_eq!(out[0].source, SuggestionSource::ChildMessage);
    }

    #[test]
    fn extract_filters_out_cargo_command_in_executable_list() {
        let diag = make_diag(
            "failed to resolve",
            None,
            Some("help: use `cargo add humantime` to add it"),
            None,
        );
        let out = CompilerSuggestionExtractor::extract(&diag);
        assert!(out.is_empty());
    }

    #[test]
    fn extract_all_keeps_non_executable_items() {
        let diag = make_diag(
            "failed to resolve",
            None,
            Some("help: use `cargo add humantime` to add it"),
            None,
        );
        let out = CompilerSuggestionExtractor::extract_all(&diag);
        assert_eq!(out.len(), 1);
        assert!(!out[0].executable);
        assert!(out[0].blocked_reason.is_some());
    }

    #[test]
    fn extract_dedups_equivalent_suggestions() {
        let diag = make_diag(
            "failed to resolve",
            Some("crate::foo::State"),
            Some("consider importing this struct: `crate::foo::State`"),
            None,
        );
        let out = CompilerSuggestionExtractor::extract(&diag);
        assert_eq!(out.len(), 1);
    }
}
