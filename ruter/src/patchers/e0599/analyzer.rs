use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use quote::ToTokens;
use regex::Regex;
use serde::{Deserialize, Serialize};
use syn::{ImplItem, Item, ItemImpl, Type};
use walkdir::WalkDir;

static RE_ASSOCIATED_ITEM: OnceLock<Regex> = OnceLock::new();
static RE_METHOD_ON_TYPE: OnceLock<Regex> = OnceLock::new();

/// Extracted target from one E0599 diagnostic message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct E0599Target {
    pub method_name: String,
    pub raw_type_name: String,
    pub normalized_type_name: String,
    pub raw_kind: String,
}

/// Cross-validation classification for E0599.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum E0599Classification {
    SevereNoStruct,
    SevereEmptyImpl,
    PartialMissingMethod,
    MisplacedFreeFunction,
    MinimalPass,
}

/// Analysis output used by runtime pre-flight and prompt injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E0599Analysis {
    pub classification: E0599Classification,
    pub target: Option<E0599Target>,
    pub matched_impl_count: usize,
    #[serde(default)]
    pub available_method_signatures: Vec<String>,
    #[serde(default)]
    pub related_free_function_signatures: Vec<String>,
    #[serde(default)]
    pub recommended_call_forms: Vec<String>,
    pub scope: String,
    pub summary: String,
}

#[derive(Debug, Default)]
struct ImplScanResult {
    impl_count: usize,
    has_any_method: bool,
    target_method_found: bool,
    method_signatures: Vec<String>,
}

#[derive(Debug, Default)]
struct FreeFnScanResult {
    exact_name_match: bool,
    matched_function_name: Option<String>,
    function_signatures: Vec<String>,
    call_forms: Vec<String>,
}

/// Analyze one E0599 message by scanning impl blocks from same file first,
/// and then falling back to crate-wide scan when same file has no hit.
pub fn analyze_e0599_against_crate(
    crate_path: &Path,
    same_file: &Path,
    message: &str,
) -> E0599Analysis {
    let Some(target) = extract_target_from_message(message) else {
        return E0599Analysis {
            classification: E0599Classification::MinimalPass,
            target: None,
            matched_impl_count: 0,
            available_method_signatures: Vec::new(),
            related_free_function_signatures: Vec::new(),
            recommended_call_forms: Vec::new(),
            scope: "none".to_string(),
            summary: "e0599 message does not match associated-item pattern".to_string(),
        };
    };

    if target.raw_kind != "struct" {
        return E0599Analysis {
            classification: E0599Classification::MinimalPass,
            target: Some(target),
            matched_impl_count: 0,
            available_method_signatures: Vec::new(),
            related_free_function_signatures: Vec::new(),
            recommended_call_forms: Vec::new(),
            scope: "none".to_string(),
            summary: "e0599 target kind is not struct; keep conservative pass-through".to_string(),
        };
    }

    let same_file_scan = scan_impls_in_file(same_file, &target);
    let (scan, scope) = if same_file_scan.impl_count > 0 {
        (same_file_scan, "same_file")
    } else {
        (
            scan_impls_in_crate(crate_path, same_file, &target),
            "crate_fallback",
        )
    };

    let mut related_free_function_signatures = Vec::new();
    let mut recommended_call_forms = Vec::new();
    let classification = if scan.impl_count == 0 {
        E0599Classification::SevereNoStruct
    } else if !scan.has_any_method {
        E0599Classification::SevereEmptyImpl
    } else if !scan.target_method_found {
        let free_fn_scan = scan_related_free_functions(crate_path, same_file, &target);
        related_free_function_signatures = free_fn_scan.function_signatures;
        recommended_call_forms = free_fn_scan.call_forms;
        if free_fn_scan.matched_function_name.is_some() {
            E0599Classification::MisplacedFreeFunction
        } else {
            E0599Classification::PartialMissingMethod
        }
    } else {
        E0599Classification::MinimalPass
    };

    let summary = match classification {
        E0599Classification::SevereNoStruct => format!(
            "struct `{}` impl not found in {}",
            target.normalized_type_name, scope
        ),
        E0599Classification::SevereEmptyImpl => format!(
            "struct `{}` has impl blocks but no methods in {}",
            target.normalized_type_name, scope
        ),
        E0599Classification::PartialMissingMethod => format!(
            "method `{}` not found for struct `{}`; collected {} method signatures",
            target.method_name,
            target.normalized_type_name,
            scan.method_signatures.len()
        ),
        E0599Classification::MisplacedFreeFunction => {
            let matched_name = related_free_function_signatures
                .first()
                .and_then(|sig| {
                    sig.split_whitespace()
                        .skip_while(|part| *part != "fn")
                        .nth(1)
                        .map(|part| part.trim_matches(|ch| ch == '(' || ch == '{'))
                })
                .unwrap_or(&target.method_name);
            format!(
                "method `{}` is likely a free function `{}`; struct `{}` has no associated item with that name",
                target.method_name, matched_name, target.normalized_type_name
            )
        }
        E0599Classification::MinimalPass => format!(
            "method `{}` exists or cannot be confirmed as hallucination",
            target.method_name
        ),
    };

    E0599Analysis {
        classification,
        target: Some(target),
        matched_impl_count: scan.impl_count,
        available_method_signatures: scan.method_signatures,
        related_free_function_signatures,
        recommended_call_forms,
        scope: scope.to_string(),
        summary,
    }
}

fn scan_related_free_functions(
    crate_path: &Path,
    same_file: &Path,
    target: &E0599Target,
) -> FreeFnScanResult {
    let same_file_scan = scan_free_functions_in_file(same_file, target);
    if same_file_scan.matched_function_name.is_some() {
        return same_file_scan;
    }
    scan_free_functions_in_crate(crate_path, same_file, target, same_file_scan)
}

fn scan_free_functions_in_crate(
    crate_path: &Path,
    skip_file: &Path,
    target: &E0599Target,
    mut seed: FreeFnScanResult,
) -> FreeFnScanResult {
    let mut fuzzy_names = Vec::<String>::new();
    for entry in WalkDir::new(crate_path) {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path == skip_file {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if should_skip_path(path) {
            continue;
        }
        merge_free_fn_scan_result(
            &mut seed,
            scan_free_functions_in_file_with_fuzzy(path, target, &mut fuzzy_names),
        );
    }

    maybe_upgrade_with_unique_fuzzy(&mut seed, target, fuzzy_names);
    if seed.matched_function_name.is_none() {
        seed.function_signatures.clear();
        seed.call_forms.clear();
    }
    sort_and_dedup_signatures(&mut seed.function_signatures);
    sort_and_dedup_signatures(&mut seed.call_forms);
    seed
}

fn scan_free_functions_in_file(file_path: &Path, target: &E0599Target) -> FreeFnScanResult {
    let mut fuzzy_names = Vec::<String>::new();
    let mut out = scan_free_functions_in_file_with_fuzzy(file_path, target, &mut fuzzy_names);
    maybe_upgrade_with_unique_fuzzy(&mut out, target, fuzzy_names);
    if out.matched_function_name.is_none() {
        out.function_signatures.clear();
        out.call_forms.clear();
    }
    sort_and_dedup_signatures(&mut out.function_signatures);
    sort_and_dedup_signatures(&mut out.call_forms);
    out
}

fn scan_free_functions_in_file_with_fuzzy(
    file_path: &Path,
    target: &E0599Target,
    fuzzy_names: &mut Vec<String>,
) -> FreeFnScanResult {
    let mut out = FreeFnScanResult::default();
    let Ok(source) = fs::read_to_string(file_path) else {
        return out;
    };
    let Ok(syntax) = syn::parse_file(&source) else {
        return out;
    };
    scan_items_for_free_functions(&syntax.items, target, &mut out, fuzzy_names);
    out
}

fn scan_items_for_free_functions(
    items: &[Item],
    target: &E0599Target,
    out: &mut FreeFnScanResult,
    fuzzy_names: &mut Vec<String>,
) {
    for item in items {
        match item {
            Item::Fn(item_fn) => {
                let ident = item_fn.sig.ident.to_string();
                if ident == target.method_name {
                    out.exact_name_match = true;
                    out.matched_function_name = Some(ident.clone());
                    out.function_signatures
                        .push(item_fn.sig.to_token_stream().to_string());
                    out.call_forms.push(format!("{ident}(...)"));
                    out.call_forms.push(format!("crate::{ident}(...)"));
                    continue;
                }
                if edit_distance_at_most_one(&ident, &target.method_name) == Some(1) {
                    fuzzy_names.push(ident.clone());
                    out.function_signatures
                        .push(item_fn.sig.to_token_stream().to_string());
                }
            }
            Item::Mod(item_mod) => {
                if let Some((_, nested_items)) = &item_mod.content {
                    scan_items_for_free_functions(nested_items, target, out, fuzzy_names);
                }
            }
            _ => {}
        }
    }
}

fn maybe_upgrade_with_unique_fuzzy(
    out: &mut FreeFnScanResult,
    _target: &E0599Target,
    fuzzy_names: Vec<String>,
) {
    if out.matched_function_name.is_some() {
        return;
    }
    let mut dedup = BTreeSet::new();
    for name in fuzzy_names {
        dedup.insert(name);
    }
    if dedup.len() != 1 {
        return;
    }
    let Some(name) = dedup.into_iter().next() else {
        return;
    };
    out.matched_function_name = Some(name.clone());
    out.call_forms.push(format!("{name}(...)"));
    out.call_forms.push(format!("crate::{name}(...)"));
}

fn merge_free_fn_scan_result(base: &mut FreeFnScanResult, patch: FreeFnScanResult) {
    base.exact_name_match |= patch.exact_name_match;
    if base.matched_function_name.is_none() {
        base.matched_function_name = patch.matched_function_name;
    }
    base.function_signatures.extend(patch.function_signatures);
    base.call_forms.extend(patch.call_forms);
}

fn edit_distance_at_most_one(left: &str, right: &str) -> Option<usize> {
    if left == right {
        return Some(0);
    }
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    let left_len = left_chars.len();
    let right_len = right_chars.len();
    if left_len.abs_diff(right_len) > 1 {
        return None;
    }

    let (short, long, short_len, long_len) = if left_len <= right_len {
        (&left_chars, &right_chars, left_len, right_len)
    } else {
        (&right_chars, &left_chars, right_len, left_len)
    };

    if short_len == long_len {
        let mut mismatch = 0usize;
        for idx in 0..short_len {
            if short[idx] != long[idx] {
                mismatch += 1;
                if mismatch > 1 {
                    return None;
                }
            }
        }
        return Some(mismatch);
    }

    let mut i = 0usize;
    let mut j = 0usize;
    let mut mismatch = 0usize;
    while i < short_len && j < long_len {
        if short[i] == long[j] {
            i += 1;
            j += 1;
            continue;
        }
        mismatch += 1;
        if mismatch > 1 {
            return None;
        }
        j += 1;
    }
    Some(1)
}

fn extract_target_from_message(message: &str) -> Option<E0599Target> {
    let re_associated = RE_ASSOCIATED_ITEM.get_or_init(|| {
        Regex::new(
            r"no function or associated item named `([^`]+)` found for (struct|enum|trait|union|type) `([^`]+)`",
        )
        .expect("valid e0599 associated-item regex")
    });
    let re_method_on_type = RE_METHOD_ON_TYPE.get_or_init(|| {
        Regex::new(r"no method named `([^`]+)` found for type `([^`]+)`")
            .expect("valid e0599 method-on-type regex")
    });

    let (method_name, raw_kind, raw_type_name) =
        if let Some(captures) = re_associated.captures(message) {
            (
                captures.get(1)?.as_str().to_string(),
                captures.get(2)?.as_str().to_ascii_lowercase(),
                captures.get(3)?.as_str().to_string(),
            )
        } else if let Some(captures) = re_method_on_type.captures(message) {
            (
                captures.get(1)?.as_str().to_string(),
                "type".to_string(),
                captures.get(2)?.as_str().to_string(),
            )
        } else {
            return None;
        };

    let normalized_type_name = normalize_type_name(&raw_type_name);
    if normalized_type_name.is_empty() || method_name.is_empty() {
        return None;
    }

    Some(E0599Target {
        method_name,
        raw_type_name,
        normalized_type_name,
        raw_kind,
    })
}

fn normalize_type_name(raw: &str) -> String {
    let without_generics = raw.split('<').next().unwrap_or(raw).trim();
    without_generics
        .split("::")
        .last()
        .unwrap_or(without_generics)
        .trim()
        .to_string()
}

fn scan_impls_in_crate(
    crate_path: &Path,
    skip_file: &Path,
    target: &E0599Target,
) -> ImplScanResult {
    let mut out = ImplScanResult::default();
    for entry in WalkDir::new(crate_path) {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path == skip_file {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if should_skip_path(path) {
            continue;
        }

        merge_scan_result(&mut out, scan_impls_in_file(path, target));
    }
    sort_and_dedup_signatures(&mut out.method_signatures);
    out
}

fn scan_impls_in_file(file_path: &Path, target: &E0599Target) -> ImplScanResult {
    let mut out = ImplScanResult::default();
    let Ok(source) = fs::read_to_string(file_path) else {
        return out;
    };
    let Ok(syntax) = syn::parse_file(&source) else {
        return out;
    };
    scan_items_for_impls(&syntax.items, target, &mut out);
    sort_and_dedup_signatures(&mut out.method_signatures);
    out
}

fn scan_items_for_impls(items: &[Item], target: &E0599Target, out: &mut ImplScanResult) {
    for item in items {
        match item {
            Item::Impl(item_impl) => collect_impl(item_impl, target, out),
            Item::Mod(item_mod) => {
                if let Some((_, nested_items)) = &item_mod.content {
                    scan_items_for_impls(nested_items, target, out);
                }
            }
            _ => {}
        }
    }
}

fn collect_impl(item_impl: &ItemImpl, target: &E0599Target, out: &mut ImplScanResult) {
    if !impl_self_type_matches(item_impl.self_ty.as_ref(), &target.normalized_type_name) {
        return;
    }

    out.impl_count += 1;
    for impl_item in &item_impl.items {
        let ImplItem::Fn(func) = impl_item else {
            continue;
        };
        out.has_any_method = true;
        if func.sig.ident == target.method_name {
            out.target_method_found = true;
        }
        out.method_signatures
            .push(func.sig.to_token_stream().to_string());
    }
}

fn impl_self_type_matches(self_ty: &Type, expected_type: &str) -> bool {
    match self_ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == expected_type)
            .unwrap_or(false),
        _ => false,
    }
}

fn merge_scan_result(base: &mut ImplScanResult, patch: ImplScanResult) {
    base.impl_count += patch.impl_count;
    base.has_any_method |= patch.has_any_method;
    base.target_method_found |= patch.target_method_found;
    base.method_signatures.extend(patch.method_signatures);
}

fn sort_and_dedup_signatures(signatures: &mut Vec<String>) {
    let mut set = BTreeSet::new();
    for sig in signatures.drain(..) {
        set.insert(sig);
    }
    *signatures = set.into_iter().collect();
}

fn should_skip_path(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str();
        value == "target" || value == ".git" || value == ".ruter"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn extract_target_handles_generic_struct_name() {
        let message = "no function or associated item named `new` found for struct `Parser<'a>` in the current scope";
        let target = extract_target_from_message(message).expect("target should exist");
        assert_eq!(target.method_name, "new");
        assert_eq!(target.raw_type_name, "Parser<'a>");
        assert_eq!(target.normalized_type_name, "Parser");
        assert_eq!(target.raw_kind, "struct");
    }

    #[test]
    fn classify_partial_missing_method_when_other_methods_exist() {
        let dir = tempdir().unwrap();
        write_file(
            &dir.path().join("src/lib.rs"),
            r#"
pub struct Parser;

impl Parser {
    pub fn parse(&self) {}
    fn off(&self) {}
}
"#,
        );

        let message = "no function or associated item named `new` found for struct `Parser` in the current scope";
        let analysis =
            analyze_e0599_against_crate(dir.path(), &dir.path().join("src/lib.rs"), message);
        assert_eq!(
            analysis.classification,
            E0599Classification::PartialMissingMethod
        );
        assert!(
            analysis
                .available_method_signatures
                .iter()
                .any(|sig| sig.contains("parse"))
        );
        assert!(
            analysis
                .available_method_signatures
                .iter()
                .any(|sig| sig.contains("off"))
        );
    }

    #[test]
    fn classify_misplaced_free_function_when_exact_same_file_match_exists() {
        let dir = tempdir().unwrap();
        write_file(
            &dir.path().join("src/lib.rs"),
            r#"
pub struct Parser;
pub fn parse(input: &str) -> Parser { Parser }
impl Parser {
    pub fn get_ref(&self) {}
}
"#,
        );

        let message = "no function or associated item named `parse` found for struct `Parser` in the current scope";
        let analysis =
            analyze_e0599_against_crate(dir.path(), &dir.path().join("src/lib.rs"), message);
        assert_eq!(
            analysis.classification,
            E0599Classification::MisplacedFreeFunction
        );
        assert!(
            analysis
                .related_free_function_signatures
                .iter()
                .any(|sig| sig.contains("parse"))
        );
        assert!(
            analysis
                .recommended_call_forms
                .iter()
                .any(|call| call == "parse(...)")
        );
        assert!(
            analysis
                .recommended_call_forms
                .iter()
                .any(|call| call == "crate::parse(...)")
        );
    }

    #[test]
    fn classify_misplaced_free_function_when_crate_fallback_finds_match() {
        let dir = tempdir().unwrap();
        write_file(
            &dir.path().join("src/lib.rs"),
            r#"
mod date;
pub use self::date::format_rfc3339;
pub struct Rfc3339Timestamp;
impl Rfc3339Timestamp {
    pub fn get_ref(&self) {}
}
"#,
        );
        write_file(
            &dir.path().join("src/date.rs"),
            r#"
use super::Rfc3339Timestamp;
pub fn format_rfc3339() -> Rfc3339Timestamp { Rfc3339Timestamp }
"#,
        );

        let message = "no function or associated item named `format_rfc3339` found for struct `Rfc3339Timestamp` in the current scope";
        let analysis =
            analyze_e0599_against_crate(dir.path(), &dir.path().join("src/lib.rs"), message);
        assert_eq!(
            analysis.classification,
            E0599Classification::MisplacedFreeFunction
        );
        assert!(
            analysis
                .related_free_function_signatures
                .iter()
                .any(|sig| sig.contains("format_rfc3339"))
        );
    }

    #[test]
    fn keep_partial_missing_when_fuzzy_candidates_are_ambiguous() {
        let dir = tempdir().unwrap();
        write_file(
            &dir.path().join("src/lib.rs"),
            r#"
pub struct Parser;
pub fn parsex() -> Parser { Parser }
pub fn parsey() -> Parser { Parser }
impl Parser {
    pub fn get_ref(&self) {}
}
"#,
        );

        let message = "no function or associated item named `parse` found for struct `Parser` in the current scope";
        let analysis =
            analyze_e0599_against_crate(dir.path(), &dir.path().join("src/lib.rs"), message);
        assert_eq!(
            analysis.classification,
            E0599Classification::PartialMissingMethod
        );
        assert!(analysis.recommended_call_forms.is_empty());
    }

    #[test]
    fn classify_severe_no_struct_when_impl_not_found() {
        let dir = tempdir().unwrap();
        write_file(
            &dir.path().join("src/lib.rs"),
            r#"
pub struct Other;
impl Other { pub fn new() -> Self { Self } }
"#,
        );
        let message = "no function or associated item named `new` found for struct `Parser` in the current scope";
        let analysis =
            analyze_e0599_against_crate(dir.path(), &dir.path().join("src/lib.rs"), message);
        assert_eq!(analysis.classification, E0599Classification::SevereNoStruct);
    }

    #[test]
    fn classify_severe_empty_impl_when_no_methods_in_impl() {
        let dir = tempdir().unwrap();
        write_file(
            &dir.path().join("src/lib.rs"),
            r#"
pub struct Parser;
impl Parser {}
"#,
        );
        let message = "no function or associated item named `new` found for struct `Parser` in the current scope";
        let analysis =
            analyze_e0599_against_crate(dir.path(), &dir.path().join("src/lib.rs"), message);
        assert_eq!(
            analysis.classification,
            E0599Classification::SevereEmptyImpl
        );
    }

    #[test]
    fn classify_minimal_pass_when_method_exists() {
        let dir = tempdir().unwrap();
        write_file(
            &dir.path().join("src/lib.rs"),
            r#"
pub struct Parser;
impl Parser {
    pub fn new() -> Self { Self }
}
"#,
        );
        let message = "no function or associated item named `new` found for struct `Parser` in the current scope";
        let analysis =
            analyze_e0599_against_crate(dir.path(), &dir.path().join("src/lib.rs"), message);
        assert_eq!(analysis.classification, E0599Classification::MinimalPass);
    }

    #[test]
    fn extract_target_handles_no_method_named_for_type_pattern() {
        let message = "no method named `to_string` found for type `isize` in the current scope";
        let target = extract_target_from_message(message).expect("target should exist");
        assert_eq!(target.method_name, "to_string");
        assert_eq!(target.raw_type_name, "isize");
        assert_eq!(target.normalized_type_name, "isize");
        assert_eq!(target.raw_kind, "type");
    }
}
