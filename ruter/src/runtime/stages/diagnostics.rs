use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use ruter::core::Diagnostic;
use ruter::parser::JsonParser;
use serde_json::Value;

use super::{DiagnosticStats, ErrorDetail};

pub(super) fn extract_diagnostic_json_lines(stdout: &[u8]) -> Result<String> {
    let raw = String::from_utf8_lossy(stdout);
    let mut out = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if value.get("reason").and_then(|v| v.as_str()) != Some("compiler-message") {
            continue;
        }
        let message = value.get("message").cloned().unwrap_or(Value::Null);
        if !message.is_object() {
            continue;
        }
        let is_non_diagnostic = message
            .get("$message_type")
            .and_then(|v| v.as_str())
            .map(|v| v != "diagnostic")
            .unwrap_or(false);
        if is_non_diagnostic {
            continue;
        }
        out.push(message.to_string());
    }

    Ok(out.join("\n"))
}

pub(super) fn normalize_paths(diagnostic: &mut Diagnostic, crate_path: &Path) {
    for span in &mut diagnostic.span {
        if span.file_path.is_relative() {
            span.file_path = crate_path.join(&span.file_path);
        }
    }

    for child in &mut diagnostic.children {
        normalize_paths(child, crate_path);
    }
}

pub(super) fn parse_diagnostics_for_crate(
    crate_path: &Path,
    diagnostics_ndjson: &str,
) -> Result<Vec<Diagnostic>> {
    let mut diagnostics =
        JsonParser::parse(diagnostics_ndjson).context("failed to parse diagnostics")?;
    for diag in &mut diagnostics {
        normalize_paths(diag, crate_path);
    }
    Ok(diagnostics)
}

pub(super) fn collect_error_stats(diagnostics: &[Diagnostic]) -> DiagnosticStats {
    let mut stats = DiagnosticStats::default();

    for diag in diagnostics {
        if !matches!(diag.severity, ruter::core::Severity::Error) {
            continue;
        }

        let code = diagnostic_code_key(diag);
        *stats.error_by_code.entry(code.clone()).or_insert(0) += 1;
        stats.error_total += 1;

        let location = diag
            .span
            .first()
            .map(|s| format!("{}:{}:{}", s.file_path.display(), s.line_start, s.col_start))
            .unwrap_or_else(|| "unknown".to_string());
        stats.error_details.push(ErrorDetail {
            code,
            message: diag.message.clone(),
            location,
        });
    }

    stats
}

fn is_e_code(raw: &str) -> bool {
    let mut chars = raw.chars();
    if chars.next() != Some('E') {
        return false;
    }
    let digits: String = chars.collect();
    digits.len() == 4 && digits.chars().all(|ch| ch.is_ascii_digit())
}

fn diagnostic_code_key(diag: &Diagnostic) -> String {
    let Some(code) = diag.code.as_ref() else {
        return "NO_CODE".to_string();
    };
    if let Some(raw) = code.raw_code.as_ref() {
        let normalized = raw.trim();
        if normalized.is_empty() {
            return "Unknown".to_string();
        }
        if is_e_code(normalized) {
            return normalized.to_string();
        }
        return normalized.to_string();
    }
    if code.code != ruter::core::ErrorCode::Unknown {
        return code.code.to_string();
    }
    "Unknown".to_string()
}

pub(super) fn format_error_code_counts(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }

    counts
        .iter()
        .map(|(code, count)| format!("{code}={count}"))
        .collect::<Vec<_>>()
        .join(",")
}
