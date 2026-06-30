use std::collections::BTreeSet;
use std::sync::OnceLock;

use regex::Regex;
use ruter::core::FunctionDiagnostic;

static RE_UNKNOWN_FIELD: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct E0560PreflightAnalysis {
    pub count: usize,
    pub unknown_fields: Vec<String>,
    pub available_fields: Vec<String>,
    pub hints: Vec<String>,
}

pub fn analyze_e0560_preflight(
    function_diags: &[FunctionDiagnostic],
) -> Option<E0560PreflightAnalysis> {
    let mut count = 0usize;
    let mut unknown_fields = Vec::new();
    let mut available_fields = Vec::new();
    let mut hints = Vec::new();
    let mut seen_unknown = BTreeSet::new();
    let mut seen_available = BTreeSet::new();
    let mut seen_hints = BTreeSet::new();

    for diag in function_diags.iter().filter(|diag| diag.code == "E0560") {
        count += 1;

        for value in extract_unknown_fields(&diag.message) {
            push_ident(&value, &mut seen_unknown, &mut unknown_fields);
        }
        if let Some(label) = diag.label.as_deref() {
            for value in extract_unknown_fields(label) {
                push_ident(&value, &mut seen_unknown, &mut unknown_fields);
            }
        }

        for line in &diag.children_note_messages {
            collect_available_fields_line(line, &mut seen_available, &mut available_fields);
        }
        for line in &diag.children_help_messages {
            collect_available_fields_line(line, &mut seen_available, &mut available_fields);
        }

        if let Some(value) = diag.suggested_replacement.as_deref() {
            push_ident(value, &mut seen_hints, &mut hints);
        }
        for value in &diag.children_suggested_replacements {
            push_ident(value, &mut seen_hints, &mut hints);
        }
    }

    if count == 0 {
        return None;
    }

    Some(E0560PreflightAnalysis {
        count,
        unknown_fields,
        available_fields,
        hints,
    })
}

fn extract_unknown_fields(text: &str) -> Vec<String> {
    let re = RE_UNKNOWN_FIELD.get_or_init(|| {
        Regex::new(r"has no field named `([^`]+)`").expect("valid e0560 unknown-field regex")
    });
    re.captures_iter(text)
        .filter_map(|caps| caps.get(1).map(|item| item.as_str().to_string()))
        .collect()
}

fn collect_available_fields_line(line: &str, seen: &mut BTreeSet<String>, out: &mut Vec<String>) {
    if !line.to_ascii_lowercase().contains("available fields are") {
        return;
    }
    for value in extract_backticked_identifiers(line) {
        push_ident(&value, seen, out);
    }
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

fn push_ident(raw: &str, seen: &mut BTreeSet<String>, out: &mut Vec<String>) {
    let value = raw.trim();
    if !looks_like_identifier(value) {
        return;
    }
    if seen.insert(value.to_string()) {
        out.push(value.to_string());
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn diag(
        code: &str,
        message: &str,
        label: Option<&str>,
        notes: &[&str],
        helps: &[&str],
        replacement: Option<&str>,
        child_replacements: &[&str],
    ) -> FunctionDiagnostic {
        FunctionDiagnostic {
            code: code.to_string(),
            message: message.to_string(),
            primary_span: None,
            label: label.map(ToString::to_string),
            suggested_replacement: replacement.map(ToString::to_string),
            children_note_messages: notes.iter().map(|item| item.to_string()).collect(),
            children_help_messages: helps.iter().map(|item| item.to_string()).collect(),
            children_suggested_replacements: child_replacements
                .iter()
                .map(|item| item.to_string())
                .collect(),
        }
    }

    #[test]
    fn analyze_e0560_preflight_collects_four_dimensions() {
        let diagnostics = vec![
            diag(
                "E0560",
                "struct `Parser` has no field named `ofset`",
                None,
                &["available fields are: `offset`, `pos`"],
                &["available fields are: `offset`, `start`"],
                None,
                &["offset"],
            ),
            diag(
                "E0560",
                "struct `Parser` has no field named `cur`",
                Some("unknown field"),
                &[],
                &[],
                Some("cursor"),
                &[],
            ),
            diag("E0433", "ignore", None, &[], &[], None, &[]),
        ];

        let analysis = analyze_e0560_preflight(&diagnostics).expect("analysis");
        assert_eq!(analysis.count, 2);
        assert_eq!(
            analysis.unknown_fields,
            vec!["ofset".to_string(), "cur".to_string()]
        );
        assert_eq!(
            analysis.available_fields,
            vec!["offset".to_string(), "pos".to_string(), "start".to_string()]
        );
        assert_eq!(
            analysis.hints,
            vec!["offset".to_string(), "cursor".to_string()]
        );
    }

    #[test]
    fn analyze_e0560_preflight_returns_none_without_target_code() {
        let diagnostics = vec![diag("E0433", "x", None, &[], &[], None, &[])];
        assert!(analyze_e0560_preflight(&diagnostics).is_none());
    }
}
