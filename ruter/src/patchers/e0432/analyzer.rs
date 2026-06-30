use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;
use toml::Value as TomlValue;

use crate::core::{Applicability, Diagnostic, FixAction, Result, RuTeRError, SpanInfo};

static RE_UNRESOLVED_IMPORT: OnceLock<Regex> = OnceLock::new();
static RE_USE_LINE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
struct SuggestionCandidate {
    span: SpanInfo,
    replacement: String,
}

pub fn analyze_e0432_diagnostic(diagnostic: &Diagnostic) -> Result<Vec<FixAction>> {
    let Some(primary_span) = primary_span(diagnostic) else {
        return Ok(Vec::new());
    };

    let source = load_source(&primary_span.file_path)?;

    let p1_actions = build_p1_actions(diagnostic, &source);
    if !p1_actions.is_empty() {
        return Ok(p1_actions);
    }

    if let Some(r1_action) = build_r1_action(diagnostic, primary_span, &source) {
        return Ok(vec![r1_action]);
    }

    if let Some(r2_action) = build_r2_action(primary_span, &source) {
        return Ok(vec![r2_action]);
    }

    Ok(Vec::new())
}

fn primary_span(diagnostic: &Diagnostic) -> Option<&SpanInfo> {
    diagnostic
        .span
        .iter()
        .find(|span| span.is_primary)
        .or_else(|| diagnostic.span.first())
}

fn load_source(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            RuTeRError::SourceFileNotFound(path.display().to_string())
        } else {
            RuTeRError::IoError(err)
        }
    })
}

fn build_p1_actions(diagnostic: &Diagnostic, source: &str) -> Vec<FixAction> {
    let mut candidates = Vec::new();
    collect_machine_applicable_suggestions(diagnostic, source, &mut candidates);
    if candidates.is_empty() {
        return Vec::new();
    }

    candidates.sort_by(|left, right| {
        left.span
            .file_path
            .cmp(&right.span.file_path)
            .then_with(|| left.span.byte_start.cmp(&right.span.byte_start))
            .then_with(|| left.span.byte_end.cmp(&right.span.byte_end))
    });

    let mut accepted = Vec::new();
    let mut last_end_by_file: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for candidate in candidates {
        let file = candidate.span.file_path.clone();
        if let Some(last_end) = last_end_by_file.get(&file)
            && candidate.span.byte_start < *last_end
        {
            continue;
        }
        last_end_by_file.insert(file, candidate.span.byte_end);
        accepted.push(FixAction::Replace {
            span: candidate.span,
            new_content: candidate.replacement,
        });
    }

    accepted
}

fn collect_machine_applicable_suggestions(
    diagnostic: &Diagnostic,
    source: &str,
    out: &mut Vec<SuggestionCandidate>,
) {
    for span in &diagnostic.span {
        let Some(replacement) = span.suggested_replacement.as_ref() else {
            continue;
        };
        if span.suggestion_applicability != Some(Applicability::MachineApplicable) {
            continue;
        }
        if replacement.trim().is_empty() {
            continue;
        }
        if !is_valid_range(span, source) {
            continue;
        }
        out.push(SuggestionCandidate {
            span: span.clone(),
            replacement: replacement.clone(),
        });
    }

    for child in &diagnostic.children {
        collect_machine_applicable_suggestions(child, source, out);
    }
}

fn build_r1_action(
    diagnostic: &Diagnostic,
    primary_span: &SpanInfo,
    source: &str,
) -> Option<FixAction> {
    if !is_valid_range(primary_span, source) {
        return None;
    }
    if !is_use_statement_span(primary_span, source) {
        return None;
    }

    let unresolved_head = extract_unresolved_head(&diagnostic.message)?;
    let package_name = load_package_name_for_file(&primary_span.file_path)?;
    if unresolved_head != package_name {
        return None;
    }

    let span_text = &source[primary_span.byte_start..primary_span.byte_end];
    let span_head = extract_path_head(span_text)?;
    if span_head != package_name {
        return None;
    }

    let rewritten = rewrite_path_head_to_crate(span_text, &package_name)?;
    Some(FixAction::Replace {
        span: primary_span.clone(),
        new_content: rewritten,
    })
}

fn build_r2_action(primary_span: &SpanInfo, source: &str) -> Option<FixAction> {
    if !is_valid_range(primary_span, source) {
        return None;
    }
    if primary_span.line_start != primary_span.line_end {
        return None;
    }

    let (line_start, line_end) = line_bounds(source, primary_span.byte_start)?;
    if line_start >= line_end || line_end > source.len() {
        return None;
    }
    if !source.is_char_boundary(line_start) || !source.is_char_boundary(line_end) {
        return None;
    }

    let line = &source[line_start..line_end];
    if !is_use_line(line) {
        return None;
    }
    if !line.trim_end().ends_with(';') {
        return None;
    }
    if !is_strict_test_context(source, line_start) {
        return None;
    }

    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let original_use = line.trim();
    let commented =
        format!("{indent}// ruter(e0432-r2): disabled unresolved import: {original_use}");

    Some(FixAction::Replace {
        span: SpanInfo {
            file_path: primary_span.file_path.clone(),
            byte_start: line_start,
            byte_end: line_end,
            line_start: primary_span.line_start,
            line_end: primary_span.line_end,
            col_start: 1,
            col_end: line.len().saturating_add(1),
            is_primary: true,
            text: vec![],
            label: None,
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        },
        new_content: commented,
    })
}

fn rewrite_path_head_to_crate(path: &str, package_name: &str) -> Option<String> {
    let trimmed_len = path.len() - path.trim_start().len();
    let leading = &path[..trimmed_len];
    let trimmed = &path[trimmed_len..];

    if let Some(rest) = trimmed.strip_prefix(&format!("{package_name}::")) {
        return Some(format!("{leading}crate::{rest}"));
    }
    if trimmed == package_name {
        return Some(format!("{leading}crate"));
    }

    None
}

fn extract_unresolved_head(message: &str) -> Option<String> {
    let re = RE_UNRESOLVED_IMPORT.get_or_init(|| {
        Regex::new(r"unresolved import `([^`]+)`").expect("valid unresolved import regex")
    });
    let caps = re.captures(message)?;
    let path = caps.get(1)?.as_str();
    extract_path_head(path)
}

fn extract_path_head(path: &str) -> Option<String> {
    let trimmed = path.trim().trim_start_matches("::");
    if trimmed.is_empty() {
        return None;
    }

    let head = trimmed.split("::").next()?.trim();
    if head.is_empty() {
        return None;
    }
    Some(head.to_string())
}

fn line_bounds(source: &str, offset: usize) -> Option<(usize, usize)> {
    if offset > source.len() || !source.is_char_boundary(offset) {
        return None;
    }

    let start = source[..offset].rfind('\n').map_or(0, |idx| idx + 1);
    let end = source[offset..]
        .find('\n')
        .map_or(source.len(), |idx| offset + idx);
    Some((start, end))
}

fn is_use_statement_span(span: &SpanInfo, source: &str) -> bool {
    let Some((line_start, line_end)) = line_bounds(source, span.byte_start) else {
        return false;
    };
    if line_start >= line_end || line_end > source.len() {
        return false;
    }
    is_use_line(&source[line_start..line_end])
}

fn is_use_line(line: &str) -> bool {
    let re = RE_USE_LINE.get_or_init(|| {
        Regex::new(r"^(?:pub(?:\([^)]*\))?\s+)?use\s+").expect("valid use-line regex")
    });
    re.is_match(line.trim_start())
}

fn is_strict_test_context(source: &str, offset: usize) -> bool {
    let end = offset.min(source.len());
    let mut markers = HashSet::new();
    markers.insert("#[cfg(test)]");
    markers.insert("#[test]");
    markers.insert("#[tokio::test]");
    markers.insert("#[rstest]");

    let prefix = &source[..end];
    markers.into_iter().any(|marker| prefix.contains(marker))
}

fn is_valid_range(span: &SpanInfo, source: &str) -> bool {
    if span.byte_start >= span.byte_end || span.byte_end > source.len() {
        return false;
    }
    source.is_char_boundary(span.byte_start) && source.is_char_boundary(span.byte_end)
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
    let manifest_path = locate_manifest(source_file)?;
    let manifest_raw = fs::read_to_string(manifest_path).ok()?;
    let manifest: TomlValue = manifest_raw.parse().ok()?;
    manifest
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(TomlValue::as_str)
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests;
