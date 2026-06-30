use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use crate::core::{Applicability, Diagnostic, FixAction, Result, RuTeRError, SpanInfo};
use crate::patchers::common::compiler_suggestion::CompilerSuggestionExtractor;

static RE_UNKNOWN_FIELD: OnceLock<Regex> = OnceLock::new();

/// Structured hints extracted from one E0560 diagnostic payload.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct E0560DiagnosticHints {
    pub unknown_fields: Vec<String>,
    pub available_fields: Vec<String>,
    pub compiler_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SuggestionCandidate {
    span: SpanInfo,
    replacement: String,
    machine_preferred: bool,
}

/// Analyze one E0560 diagnostic and produce conservative field-key replacements.
///
/// Strategy:
/// - `P1`: trust rustc span suggestions with applicability in {MachineApplicable, MaybeIncorrect}
/// - `R1`: fallback to unique edit-distance=1 field rename from available-fields evidence
pub fn analyze_e0560_diagnostic(diagnostic: &Diagnostic) -> Result<Vec<FixAction>> {
    let Some(primary_span) = primary_span(diagnostic) else {
        return Ok(Vec::new());
    };
    let source = load_source(&primary_span.file_path)?;

    let p1_actions = build_p1_actions(diagnostic, &source);
    if !p1_actions.is_empty() {
        return Ok(p1_actions);
    }

    if let Some(r1_action) = build_r1_action(diagnostic, &source) {
        return Ok(vec![r1_action]);
    }

    Ok(Vec::new())
}

/// Extract digest hints for preflight prompt injection.
pub fn analyze_e0560_hints(diagnostic: &Diagnostic) -> E0560DiagnosticHints {
    let unknown_fields = collect_unknown_fields(diagnostic);
    let available_fields = collect_available_fields(diagnostic);
    let compiler_hints = collect_compiler_hints(diagnostic);

    E0560DiagnosticHints {
        unknown_fields,
        available_fields,
        compiler_hints,
    }
}

fn build_p1_actions(diagnostic: &Diagnostic, source: &str) -> Vec<FixAction> {
    let allowed_hint_set = CompilerSuggestionExtractor::extract_all(diagnostic)
        .into_iter()
        .filter(|item| item.executable && is_allowed_applicability(item.applicability))
        .filter_map(|item| item.normalized_text)
        .filter(|replacement| looks_like_identifier(replacement))
        .collect::<BTreeSet<_>>();

    if allowed_hint_set.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    collect_span_suggestion_candidates(diagnostic, source, &allowed_hint_set, &mut candidates);
    if candidates.is_empty() {
        return Vec::new();
    }

    candidates.sort_by(|left, right| {
        left.span
            .file_path
            .cmp(&right.span.file_path)
            .then_with(|| left.span.byte_start.cmp(&right.span.byte_start))
            .then_with(|| left.span.byte_end.cmp(&right.span.byte_end))
            .then_with(|| right.machine_preferred.cmp(&left.machine_preferred))
            .then_with(|| left.replacement.cmp(&right.replacement))
    });

    let mut dedup = BTreeSet::new();
    candidates.retain(|candidate| {
        dedup.insert(format!(
            "{}:{}:{}:{}",
            candidate.span.file_path.display(),
            candidate.span.byte_start,
            candidate.span.byte_end,
            candidate.replacement
        ))
    });

    let mut accepted = Vec::new();
    let mut last_end_by_file = std::collections::BTreeMap::new();
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

fn collect_span_suggestion_candidates(
    diagnostic: &Diagnostic,
    source: &str,
    allowed_hint_set: &BTreeSet<String>,
    out: &mut Vec<SuggestionCandidate>,
) {
    for span in &diagnostic.span {
        let Some(replacement) = span.suggested_replacement.as_deref() else {
            continue;
        };
        let replacement = replacement.trim();
        if !allowed_hint_set.contains(replacement) {
            continue;
        }
        if !is_allowed_applicability(span.suggestion_applicability) {
            continue;
        }
        if !is_valid_range(span, source) || !is_explicit_named_field_span(span, source) {
            continue;
        }
        out.push(SuggestionCandidate {
            span: span.clone(),
            replacement: replacement.to_string(),
            machine_preferred: span.suggestion_applicability
                == Some(Applicability::MachineApplicable),
        });
    }

    for child in &diagnostic.children {
        collect_span_suggestion_candidates(child, source, allowed_hint_set, out);
    }
}

fn build_r1_action(diagnostic: &Diagnostic, source: &str) -> Option<FixAction> {
    let span = primary_span(diagnostic)?;
    if !is_valid_range(span, source) || !is_explicit_named_field_span(span, source) {
        return None;
    }

    let unknown_fields = collect_unknown_fields(diagnostic);
    if unknown_fields.len() != 1 {
        return None;
    }
    let offending = unknown_fields[0].as_str();

    let candidates = collect_available_fields(diagnostic)
        .into_iter()
        .filter(|field| edit_distance_is_one_ascii_ci(offending, field))
        .collect::<Vec<_>>();

    if candidates.len() != 1 {
        return None;
    }

    Some(FixAction::Replace {
        span: span.clone(),
        new_content: candidates[0].clone(),
    })
}

fn collect_unknown_fields(diagnostic: &Diagnostic) -> Vec<String> {
    let mut out = Vec::new();
    collect_unknown_fields_recursive(diagnostic, &mut out);
    dedup_keep_order(out)
}

fn collect_unknown_fields_recursive(diagnostic: &Diagnostic, out: &mut Vec<String>) {
    for value in extract_unknown_fields_from_text(&diagnostic.message) {
        out.push(value);
    }
    for child in &diagnostic.children {
        collect_unknown_fields_recursive(child, out);
    }
}

fn extract_unknown_fields_from_text(text: &str) -> Vec<String> {
    let re = RE_UNKNOWN_FIELD.get_or_init(|| {
        Regex::new(r"has no field named `([^`]+)`").expect("valid e0560 unknown-field regex")
    });
    re.captures_iter(text)
        .filter_map(|caps| caps.get(1).map(|item| item.as_str().trim().to_string()))
        .filter(|field| looks_like_identifier(field))
        .collect()
}

fn collect_available_fields(diagnostic: &Diagnostic) -> Vec<String> {
    let mut out = Vec::new();
    collect_available_fields_recursive(diagnostic, &mut out);
    dedup_keep_order(out)
}

fn collect_available_fields_recursive(diagnostic: &Diagnostic, out: &mut Vec<String>) {
    let lowered = diagnostic.message.to_ascii_lowercase();
    if lowered.contains("available fields are") {
        for ident in extract_backticked_identifiers(&diagnostic.message) {
            if looks_like_identifier(&ident) {
                out.push(ident);
            }
        }
    }

    for span in &diagnostic.span {
        if let Some(replacement) = span.suggested_replacement.as_deref() {
            let replacement = replacement.trim();
            if looks_like_identifier(replacement)
                && is_allowed_applicability(span.suggestion_applicability)
            {
                out.push(replacement.to_string());
            }
        }
    }

    for child in &diagnostic.children {
        collect_available_fields_recursive(child, out);
    }
}

fn collect_compiler_hints(diagnostic: &Diagnostic) -> Vec<String> {
    CompilerSuggestionExtractor::extract_all(diagnostic)
        .into_iter()
        .filter(|item| item.executable && is_allowed_applicability(item.applicability))
        .filter_map(|item| item.normalized_text)
        .filter(|text| looks_like_identifier(text))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn extract_backticked_identifiers(text: &str) -> Vec<String> {
    text.split('`')
        .enumerate()
        .filter_map(|(idx, chunk)| {
            if idx % 2 == 1 && looks_like_identifier(chunk) {
                Some(chunk.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn dedup_keep_order(items: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for item in items {
        if seen.insert(item.clone()) {
            out.push(item);
        }
    }
    out
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

fn is_allowed_applicability(applicability: Option<Applicability>) -> bool {
    matches!(
        applicability,
        Some(Applicability::MachineApplicable) | Some(Applicability::MaybeIncorrect)
    )
}

fn is_valid_range(span: &SpanInfo, source: &str) -> bool {
    if span.byte_start >= span.byte_end || span.byte_end > source.len() {
        return false;
    }
    source.is_char_boundary(span.byte_start) && source.is_char_boundary(span.byte_end)
}

/// Ensure replacement stays on explicit `field: expr` syntax, never shorthand.
fn is_explicit_named_field_span(span: &SpanInfo, source: &str) -> bool {
    if !is_valid_range(span, source) {
        return false;
    }
    let bytes = source.as_bytes();

    let mut right = span.byte_end;
    while right < bytes.len() && bytes[right].is_ascii_whitespace() {
        right += 1;
    }
    if right >= bytes.len() || bytes[right] != b':' {
        return false;
    }

    let mut left = span.byte_start;
    while left > 0 {
        left -= 1;
        if bytes[left].is_ascii_whitespace() {
            continue;
        }
        return bytes[left] == b'{' || bytes[left] == b',';
    }
    false
}

fn looks_like_identifier(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

/// Optimized equality check for edit distance exactly one (ASCII case-insensitive).
fn edit_distance_is_one_ascii_ci(left: &str, right: &str) -> bool {
    let a = left.to_ascii_lowercase().into_bytes();
    let b = right.to_ascii_lowercase().into_bytes();
    if a == b {
        return false;
    }

    let len_a = a.len();
    let len_b = b.len();
    if len_a.abs_diff(len_b) > 1 {
        return false;
    }

    let (short, long, same_len) = if len_a <= len_b {
        (&a, &b, len_a == len_b)
    } else {
        (&b, &a, false)
    };

    let mut i = 0usize;
    let mut j = 0usize;
    let mut edits = 0usize;

    while i < short.len() && j < long.len() {
        if short[i] == long[j] {
            i += 1;
            j += 1;
            continue;
        }

        edits += 1;
        if edits > 1 {
            return false;
        }
        if same_len {
            i += 1;
            j += 1;
        } else {
            j += 1;
        }
    }

    if j < long.len() || i < short.len() {
        edits += 1;
    }

    edits == 1
}

#[cfg(test)]
mod tests;
