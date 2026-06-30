use std::collections::BTreeSet;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

static RE_EXPECTED_FOUND_BACKTICK: OnceLock<Regex> = OnceLock::new();
static RE_EXPECTED_ONLY_BACKTICK: OnceLock<Regex> = OnceLock::new();
static RE_FOUND_ONLY_BACKTICK: OnceLock<Regex> = OnceLock::new();

/// Input required for E0308 type-difference extraction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct E0308DiagnosticInput {
    pub message: String,
    pub primary_label: Option<String>,
    #[serde(default)]
    pub children_note_messages: Vec<String>,
    #[serde(default)]
    pub children_help_messages: Vec<String>,
    #[serde(default)]
    pub children_suggested_replacements: Vec<String>,
}

/// Source that produced an expected/found pair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpectedFoundSource {
    PrimaryLabel,
    ChildNote,
    ChildHelp,
    Message,
}

/// Confidence score for extracted expected/found pair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpectedFoundConfidence {
    High,
    Medium,
    Low,
}

/// Extracted expected/found pair with provenance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpectedFoundPair {
    pub expected_type: String,
    pub found_type: String,
    pub source: ExpectedFoundSource,
    pub confidence: ExpectedFoundConfidence,
}

/// E0308 preflight classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum E0308Classification {
    MechanicalMismatch,
    WrapperMismatch,
    NominalHallucination,
    SetupHell,
    Unknown,
}

/// E0308 analysis result used by preflight + prompt hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E0308Analysis {
    pub pair: Option<ExpectedFoundPair>,
    pub classification: E0308Classification,
    #[serde(default)]
    pub hints: Vec<String>,
    pub summary: String,
}

/// Analyze one E0308 diagnostic payload using multi-source extraction.
pub fn analyze_e0308_diagnostic(input: &E0308DiagnosticInput) -> E0308Analysis {
    let pair = extract_expected_found_pair(input);
    let classification = classify_e0308(pair.as_ref());
    let hints = build_hints(&classification, pair.as_ref(), input);
    let summary = build_summary(&classification, pair.as_ref());

    E0308Analysis {
        pair,
        classification,
        hints,
        summary,
    }
}

fn extract_expected_found_pair(input: &E0308DiagnosticInput) -> Option<ExpectedFoundPair> {
    if let Some(label) = input.primary_label.as_deref()
        && let Some((expected, found)) = parse_expected_found_pair(label)
    {
        return Some(ExpectedFoundPair {
            expected_type: expected,
            found_type: found,
            source: ExpectedFoundSource::PrimaryLabel,
            confidence: ExpectedFoundConfidence::High,
        });
    }

    for note in &input.children_note_messages {
        if let Some((expected, found)) = parse_expected_found_pair(note) {
            return Some(ExpectedFoundPair {
                expected_type: expected,
                found_type: found,
                source: ExpectedFoundSource::ChildNote,
                confidence: ExpectedFoundConfidence::High,
            });
        }
    }

    for help in &input.children_help_messages {
        if let Some((expected, found)) = parse_expected_found_pair(help) {
            return Some(ExpectedFoundPair {
                expected_type: expected,
                found_type: found,
                source: ExpectedFoundSource::ChildHelp,
                confidence: ExpectedFoundConfidence::Medium,
            });
        }
    }

    for replacement in &input.children_suggested_replacements {
        if let Some((expected, found)) = parse_expected_found_pair(replacement) {
            return Some(ExpectedFoundPair {
                expected_type: expected,
                found_type: found,
                source: ExpectedFoundSource::ChildHelp,
                confidence: ExpectedFoundConfidence::Medium,
            });
        }
    }

    if let Some((expected, found)) = parse_expected_found_pair(&input.message) {
        return Some(ExpectedFoundPair {
            expected_type: expected,
            found_type: found,
            source: ExpectedFoundSource::Message,
            confidence: ExpectedFoundConfidence::Low,
        });
    }

    let expected = input
        .primary_label
        .as_deref()
        .and_then(parse_expected_only)
        .or_else(|| {
            input
                .children_note_messages
                .iter()
                .find_map(|line| parse_expected_only(line))
        })
        .or_else(|| {
            input
                .children_help_messages
                .iter()
                .find_map(|line| parse_expected_only(line))
        })
        .or_else(|| parse_expected_only(&input.message));

    let found = input
        .primary_label
        .as_deref()
        .and_then(parse_found_only)
        .or_else(|| {
            input
                .children_note_messages
                .iter()
                .find_map(|line| parse_found_only(line))
        })
        .or_else(|| {
            input
                .children_help_messages
                .iter()
                .find_map(|line| parse_found_only(line))
        })
        .or_else(|| parse_found_only(&input.message));

    match (expected, found) {
        (Some(expected_type), Some(found_type)) => Some(ExpectedFoundPair {
            expected_type,
            found_type,
            source: ExpectedFoundSource::Message,
            confidence: ExpectedFoundConfidence::Medium,
        }),
        _ => None,
    }
}

fn parse_expected_found_pair(text: &str) -> Option<(String, String)> {
    let re = RE_EXPECTED_FOUND_BACKTICK.get_or_init(|| {
        Regex::new(r"expected(?: [a-zA-Z_ ]+)? `([^`]+)`[, ]+found(?: [a-zA-Z_ ]+)? `([^`]+)`")
            .expect("valid e0308 expected/found regex")
    });

    let captures = re.captures(text)?;
    let expected = captures.get(1)?.as_str().trim().to_string();
    let found = captures.get(2)?.as_str().trim().to_string();
    if expected.is_empty() || found.is_empty() {
        return None;
    }
    Some((expected, found))
}

fn parse_expected_only(text: &str) -> Option<String> {
    let re = RE_EXPECTED_ONLY_BACKTICK.get_or_init(|| {
        Regex::new(r"expected(?: [a-zA-Z_ ]+)? `([^`]+)`").expect("valid e0308 expected-only regex")
    });
    let captures = re.captures(text)?;
    let expected = captures.get(1)?.as_str().trim().to_string();
    if expected.is_empty() {
        None
    } else {
        Some(expected)
    }
}

fn parse_found_only(text: &str) -> Option<String> {
    let re = RE_FOUND_ONLY_BACKTICK.get_or_init(|| {
        Regex::new(r"found(?: [a-zA-Z_ ]+)? `([^`]+)`").expect("valid e0308 found-only regex")
    });
    let captures = re.captures(text)?;
    let found = captures.get(1)?.as_str().trim().to_string();
    if found.is_empty() { None } else { Some(found) }
}

fn classify_e0308(pair: Option<&ExpectedFoundPair>) -> E0308Classification {
    let Some(pair) = pair else {
        return E0308Classification::Unknown;
    };
    if is_setup_hell(&pair.expected_type) || is_setup_hell(&pair.found_type) {
        return E0308Classification::SetupHell;
    }
    if is_wrapper_mismatch(&pair.expected_type, &pair.found_type) {
        return E0308Classification::WrapperMismatch;
    }
    if is_mechanical_mismatch(&pair.expected_type, &pair.found_type) {
        return E0308Classification::MechanicalMismatch;
    }
    if is_nominal_hallucination(&pair.expected_type, &pair.found_type) {
        return E0308Classification::NominalHallucination;
    }
    E0308Classification::Unknown
}

fn is_setup_hell(raw_type: &str) -> bool {
    let lowered = raw_type.to_ascii_lowercase();
    let markers = [
        "fmt::formatter",
        "formatter<'",
        "*mut ",
        "*const ",
        "unsafecell<",
        "pin<",
    ];
    markers.iter().any(|marker| lowered.contains(marker))
}

fn is_wrapper_mismatch(expected: &str, found: &str) -> bool {
    let wrappers = ["Option<", "Result<", "Box<", "Rc<", "Arc<"];
    let expected_wrapper = wrappers
        .iter()
        .find(|wrapper| expected.contains(**wrapper))
        .copied();
    let found_wrapper = wrappers
        .iter()
        .find(|wrapper| found.contains(**wrapper))
        .copied();
    match (expected_wrapper, found_wrapper) {
        (Some(left), Some(right)) => left != right || expected != found,
        (Some(_), None) | (None, Some(_)) => true,
        (None, None) => false,
    }
}

fn is_mechanical_mismatch(expected: &str, found: &str) -> bool {
    let expected_base = normalize_reference_base(expected);
    let found_base = normalize_reference_base(found);
    if expected_base == found_base && expected != found {
        return true;
    }

    let stringish = ["str", "&str", "String", "std::string::String"];
    if stringish.contains(&expected.trim())
        && stringish.contains(&found.trim())
        && expected != found
    {
        return true;
    }

    let numerics = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64",
    ];
    numerics.contains(&expected.trim()) && numerics.contains(&found.trim()) && expected != found
}

fn normalize_reference_base(raw: &str) -> String {
    let mut normalized = raw.trim().to_string();
    while let Some(stripped) = normalized.strip_prefix('&') {
        normalized = stripped.trim_start().to_string();
        if let Some(mut_stripped) = normalized.strip_prefix("mut ") {
            normalized = mut_stripped.trim_start().to_string();
        }
    }
    normalized
}

fn is_nominal_hallucination(expected: &str, found: &str) -> bool {
    if expected == found {
        return false;
    }
    let expected_base = strip_generic_suffix(expected);
    let found_base = strip_generic_suffix(found);
    let expected_tail = last_path_segment(expected_base);
    let found_tail = last_path_segment(found_base);

    if expected_tail == found_tail && expected_base != found_base {
        return true;
    }
    looks_nominal(expected_base) && looks_nominal(found_base)
}

fn strip_generic_suffix(raw: &str) -> &str {
    raw.split('<').next().unwrap_or(raw).trim()
}

fn last_path_segment(raw: &str) -> &str {
    raw.rsplit("::").next().unwrap_or(raw).trim()
}

fn looks_nominal(raw: &str) -> bool {
    raw.contains("::")
        || raw
            .chars()
            .next()
            .map(|ch| ch.is_ascii_uppercase())
            .unwrap_or(false)
}

fn build_hints(
    classification: &E0308Classification,
    pair: Option<&ExpectedFoundPair>,
    input: &E0308DiagnosticInput,
) -> Vec<String> {
    let mut hints = Vec::new();

    match classification {
        E0308Classification::SetupHell => {
            hints.push("setup-heavy expected type detected; avoid constructing internal runtime-only types in tests".to_string());
        }
        E0308Classification::WrapperMismatch => {
            hints.push(
                "check wrapper alignment (Option/Result/Box/Rc/Arc) before broad rewrites"
                    .to_string(),
            );
            if let Some(pair) = pair {
                if pair.expected_type.contains("Option<") {
                    hints.push("expected Option<_>; consider wrapping with Some(...)".to_string());
                }
                if pair.expected_type.contains("Result<") {
                    hints.push(
                        "expected Result<_, _>; consider Ok(...) or error propagation with ?"
                            .to_string(),
                    );
                }
            }
        }
        E0308Classification::MechanicalMismatch => {
            hints.push(
                "check mechanical conversions: &, &mut, *, as, .into(), .to_string()".to_string(),
            );
        }
        E0308Classification::NominalHallucination => {
            hints.push("avoid redefining look-alike nominal types; use the real type path from crate context".to_string());
            if let Some(pair) = pair {
                hints.push(format!(
                    "prefer_rewrite_found_to_expected={}=>{}",
                    pair.found_type, pair.expected_type
                ));
            }
        }
        E0308Classification::Unknown => {}
    }

    for replacement in input.children_suggested_replacements.iter().take(3) {
        let clean = replacement.trim();
        if !clean.is_empty() {
            hints.push(format!("compiler_suggestion={clean}"));
        }
    }

    dedup_keep_order(hints)
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

fn build_summary(classification: &E0308Classification, pair: Option<&ExpectedFoundPair>) -> String {
    match pair {
        Some(pair) => format!(
            "e0308 classification={} expected=`{}` found=`{}` source={:?} confidence={:?}",
            classification_name(classification),
            pair.expected_type,
            pair.found_type,
            pair.source,
            pair.confidence
        ),
        None => format!(
            "e0308 classification={} expected/found not extracted",
            classification_name(classification)
        ),
    }
}

fn classification_name(kind: &E0308Classification) -> &'static str {
    match kind {
        E0308Classification::MechanicalMismatch => "MECHANICAL_MISMATCH",
        E0308Classification::WrapperMismatch => "WRAPPER_MISMATCH",
        E0308Classification::NominalHallucination => "NOMINAL_HALLUCINATION",
        E0308Classification::SetupHell => "SETUP_HELL",
        E0308Classification::Unknown => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(message: &str) -> E0308DiagnosticInput {
        E0308DiagnosticInput {
            message: message.to_string(),
            primary_label: None,
            children_note_messages: Vec::new(),
            children_help_messages: Vec::new(),
            children_suggested_replacements: Vec::new(),
        }
    }

    #[test]
    fn extracts_expected_found_from_primary_label_first() {
        let mut value = input("mismatched types");
        value.primary_label = Some("expected `i32`, found `&str`".to_string());
        value.children_note_messages = vec!["expected `u32`, found `u64`".to_string()];

        let analysis = analyze_e0308_diagnostic(&value);
        let pair = analysis.pair.expect("pair");
        assert_eq!(pair.expected_type, "i32");
        assert_eq!(pair.found_type, "&str");
        assert_eq!(pair.source, ExpectedFoundSource::PrimaryLabel);
        assert_eq!(pair.confidence, ExpectedFoundConfidence::High);
    }

    #[test]
    fn extracts_expected_found_from_child_note_when_label_missing() {
        let mut value = input("mismatched types");
        value.children_note_messages =
            vec!["expected enum `Option<i32>`, found type `i32`".to_string()];

        let analysis = analyze_e0308_diagnostic(&value);
        let pair = analysis.pair.expect("pair");
        assert_eq!(pair.expected_type, "Option<i32>");
        assert_eq!(pair.found_type, "i32");
        assert_eq!(pair.source, ExpectedFoundSource::ChildNote);
    }

    #[test]
    fn extracts_expected_found_from_help_and_classifies_wrapper_mismatch() {
        let mut value = input("mismatched types");
        value.children_help_messages = vec![
            "help: expected `Option<String>`, found `String`".to_string(),
            "consider wrapping the expression in `Some(...)`".to_string(),
        ];
        value.children_suggested_replacements = vec!["Some(".to_string(), ")".to_string()];

        let analysis = analyze_e0308_diagnostic(&value);
        let pair = analysis.pair.expect("pair");
        assert_eq!(pair.source, ExpectedFoundSource::ChildHelp);
        assert_eq!(
            analysis.classification,
            E0308Classification::WrapperMismatch
        );
        assert!(
            analysis
                .hints
                .iter()
                .any(|hint| hint.contains("Option<_>; consider wrapping"))
        );
    }

    #[test]
    fn falls_back_to_message_and_marks_low_confidence() {
        let value = input("mismatched types: expected `Foo`, found `bar::Foo`");
        let analysis = analyze_e0308_diagnostic(&value);
        let pair = analysis.pair.expect("pair");
        assert_eq!(pair.source, ExpectedFoundSource::Message);
        assert_eq!(pair.confidence, ExpectedFoundConfidence::Low);
    }

    #[test]
    fn classifies_setup_hell_before_other_kinds() {
        let mut value = input("mismatched types");
        value.primary_label = Some("expected `fmt::Formatter<'_>`, found `String`".to_string());

        let analysis = analyze_e0308_diagnostic(&value);
        assert_eq!(analysis.classification, E0308Classification::SetupHell);
    }

    #[test]
    fn classifies_mechanical_mismatch_for_reference_delta() {
        let mut value = input("mismatched types");
        value.primary_label = Some("expected `&str`, found `String`".to_string());

        let analysis = analyze_e0308_diagnostic(&value);
        assert_eq!(
            analysis.classification,
            E0308Classification::MechanicalMismatch
        );
    }
}
