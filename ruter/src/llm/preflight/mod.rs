use std::path::Path;

use ruter::patchers::e0308::{E0308Analysis, E0308Classification};
use ruter::patchers::e0599::{
    E0599Analysis, E0599Classification, analyze_e0599_against_crate,
};

use self::e0432::E0432PreflightAnalysis;
use self::e0560::E0560PreflightAnalysis;
use crate::llm::context_builder::PreflightInterceptorDigestV1;
use crate::llm::schema::truncate_for_artifact;
use crate::runtime::function::low_value::{
    LowValueAnalysis, LowValueStatus, analyze_test_function_low_value,
};
use ruter::core::{FunctionDiagnostic, TestFunction};

mod comment_out;
mod decision;
mod e0308;
mod e0432;
mod e0560;

pub use comment_out::build_comment_out_action;
pub use decision::{PreflightDecision, decide_preflight, should_try_budget_e0308_fallback};
pub use e0308::analyze_e0308_preflight;
pub use e0432::analyze_e0432_preflight;
pub use e0560::analyze_e0560_preflight;

const PREFLIGHT_NOTE_MAX_CHARS: usize = 240;

#[derive(Debug, Clone)]
pub struct PreflightAnalyses {
    pub crate_env_no_std: bool,
    pub low_value: LowValueAnalysis,
    pub e0599: Option<E0599Analysis>,
    pub e0308: Option<E0308Analysis>,
    pub e0432: Option<E0432PreflightAnalysis>,
    pub e0560: Option<E0560PreflightAnalysis>,
}

pub fn analyze_preflight_interceptors(
    crate_path: &Path,
    root_function: &TestFunction,
    function_diags: &[FunctionDiagnostic],
) -> PreflightAnalyses {
    PreflightAnalyses {
        crate_env_no_std: detect_crate_no_std(crate_path),
        low_value: analyze_test_function_low_value(root_function),
        e0599: analyze_e0599_preflight(crate_path, root_function, function_diags),
        e0308: analyze_e0308_preflight(function_diags),
        e0432: analyze_e0432_preflight(function_diags),
        e0560: analyze_e0560_preflight(function_diags),
    }
}

pub fn build_preflight_interceptor_digest(
    analyses: &PreflightAnalyses,
) -> Option<PreflightInterceptorDigestV1> {
    let mut notes = Vec::new();
    notes.push(format!(
        "LOW_VALUE_STATUS={} reason={}",
        low_value_status_name(&analyses.low_value.status),
        truncate_for_artifact(&analyses.low_value.reason, PREFLIGHT_NOTE_MAX_CHARS)
    ));
    notes.push(format!("CRATE_ENV_NO_STD={}", analyses.crate_env_no_std));
    if !analyses.low_value.markers.is_empty() {
        notes.push(format!(
            "LOW_VALUE_MARKERS={}",
            truncate_for_artifact(
                &analyses.low_value.markers.join(","),
                PREFLIGHT_NOTE_MAX_CHARS,
            )
        ));
    }

    if let Some(analysis) = analyses.e0599.as_ref() {
        notes.push(format!(
            "E0599_CLASSIFICATION={} scope={}",
            e0599_classification_name(&analysis.classification),
            analysis.scope
        ));
        notes.push(format!(
            "E0599_SUMMARY={}",
            truncate_for_artifact(&analysis.summary, PREFLIGHT_NOTE_MAX_CHARS)
        ));
        if !analysis.available_method_signatures.is_empty() {
            notes.push(format!(
                "E0599_METHOD_SIGNATURES={}",
                truncate_for_artifact(
                    &analysis
                        .available_method_signatures
                        .iter()
                        .take(6)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREFLIGHT_NOTE_MAX_CHARS,
                )
            ));
        }
        if !analysis.related_free_function_signatures.is_empty() {
            notes.push(format!(
                "E0599_RELATED_FREE_FN_SIGNATURES={}",
                truncate_for_artifact(
                    &analysis
                        .related_free_function_signatures
                        .iter()
                        .take(6)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREFLIGHT_NOTE_MAX_CHARS,
                )
            ));
        }
        if matches!(
            analysis.classification,
            E0599Classification::MisplacedFreeFunction
        ) {
            if let Some(target) = analysis.target.as_ref() {
                notes.push(format!(
                    "E0599_NAMESPACE_HINT=The target {} is a free function, not an associated function of {}. Remove {}:: prefix and call {}(...) or crate::{}(...) depending on scope.",
                    target.method_name,
                    target.normalized_type_name,
                    target.normalized_type_name,
                    target.method_name,
                    target.method_name
                ));
            }
        }
        if let Some(primitive_type) = no_std_primitive_to_string_type(analyses) {
            notes.push(format!(
                "E0599_NO_STD_PRIMITIVE_TOSTRING=true type={primitive_type}"
            ));
        }
    }

    if let Some(analysis) = analyses.e0308.as_ref() {
        notes.push(format!(
            "E0308_CLASSIFICATION={}",
            e0308_classification_name(&analysis.classification)
        ));
        if let Some(pair) = analysis.pair.as_ref() {
            notes.push(format!(
                "E0308_EXPECTED={}",
                truncate_for_artifact(&pair.expected_type, PREFLIGHT_NOTE_MAX_CHARS)
            ));
            notes.push(format!(
                "E0308_FOUND={}",
                truncate_for_artifact(&pair.found_type, PREFLIGHT_NOTE_MAX_CHARS)
            ));
            let expected_len = extract_array_len(&pair.expected_type);
            let found_len = extract_array_len(&pair.found_type);
            if let Some(expected_len) = expected_len {
                notes.push(format!("E0308_EXPECTED_ARRAY_LEN={expected_len}"));
            }
            if let Some(found_len) = found_len {
                notes.push(format!("E0308_FOUND_ARRAY_LEN={found_len}"));
            }
            if let (Some(expected_len), Some(found_len)) = (expected_len, found_len) {
                notes.push(format!("E0308_LEN_DELTA={}", found_len - expected_len));
            }
        }
        if !analysis.hints.is_empty() {
            notes.push(format!(
                "E0308_HINTS={}",
                truncate_for_artifact(
                    &analysis
                        .hints
                        .iter()
                        .take(4)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREFLIGHT_NOTE_MAX_CHARS,
                )
            ));
        }
    }

    if let Some(analysis) = analyses.e0432.as_ref() {
        notes.push(format!("E0432_COUNT={}", analysis.count));
        notes.push(format!(
            "E0432_SUMMARY={}",
            truncate_for_artifact(&analysis.summary, PREFLIGHT_NOTE_MAX_CHARS)
        ));
        if !analysis.hints.is_empty() {
            notes.push(format!(
                "E0432_HINTS={}",
                truncate_for_artifact(
                    &analysis
                        .hints
                        .iter()
                        .take(4)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREFLIGHT_NOTE_MAX_CHARS,
                )
            ));
        }
    }

    if let Some(analysis) = analyses.e0560.as_ref() {
        notes.push(format!("E0560_COUNT={}", analysis.count));
        if !analysis.unknown_fields.is_empty() {
            notes.push(format!(
                "E0560_UNKNOWN_FIELDS={}",
                truncate_for_artifact(
                    &analysis
                        .unknown_fields
                        .iter()
                        .take(6)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREFLIGHT_NOTE_MAX_CHARS,
                )
            ));
        }
        if !analysis.available_fields.is_empty() {
            notes.push(format!(
                "E0560_AVAILABLE_FIELDS={}",
                truncate_for_artifact(
                    &analysis
                        .available_fields
                        .iter()
                        .take(8)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREFLIGHT_NOTE_MAX_CHARS,
                )
            ));
        }
        if !analysis.hints.is_empty() {
            notes.push(format!(
                "E0560_HINTS={}",
                truncate_for_artifact(
                    &analysis
                        .hints
                        .iter()
                        .take(4)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREFLIGHT_NOTE_MAX_CHARS,
                )
            ));
        }
    }

    if notes.is_empty() {
        None
    } else {
        Some(PreflightInterceptorDigestV1 { notes })
    }
}

fn analyze_e0599_preflight(
    crate_path: &Path,
    root_function: &TestFunction,
    function_diags: &[FunctionDiagnostic],
) -> Option<E0599Analysis> {
    let diag = function_diags.iter().find(|diag| diag.code == "E0599")?;
    Some(analyze_e0599_against_crate(
        crate_path,
        &root_function.file_path,
        &diag.message,
    ))
}

fn detect_crate_no_std(crate_path: &Path) -> bool {
    for relative in ["src/lib.rs", "src/main.rs"] {
        let candidate = crate_path.join(relative);
        let Ok(source) = std::fs::read_to_string(&candidate) else {
            continue;
        };
        if source.contains("#![no_std]") {
            return true;
        }
    }
    false
}

fn extract_array_len(raw_type: &str) -> Option<isize> {
    let Some((_, tail)) = raw_type.rsplit_once(';') else {
        return None;
    };
    let numeric = tail
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit() || *ch == '_')
        .collect::<String>();
    if numeric.is_empty() {
        return None;
    }
    numeric.replace('_', "").parse::<isize>().ok()
}

fn low_value_status_name(status: &LowValueStatus) -> &'static str {
    match status {
        LowValueStatus::LowValue => "LOW_VALUE",
        LowValueStatus::HasTestSemantics => "HAS_TEST_SEMANTICS",
        LowValueStatus::Unknown => "UNKNOWN",
    }
}

fn e0599_classification_name(kind: &E0599Classification) -> &'static str {
    match kind {
        E0599Classification::SevereNoStruct => "SEVERE_NO_STRUCT",
        E0599Classification::SevereEmptyImpl => "SEVERE_EMPTY_IMPL",
        E0599Classification::PartialMissingMethod => "PARTIAL_MISSING_METHOD",
        E0599Classification::MisplacedFreeFunction => "MISPLACED_FREE_FUNCTION",
        E0599Classification::MinimalPass => "MINIMAL_PASS",
    }
}

fn no_std_primitive_to_string_type(analyses: &PreflightAnalyses) -> Option<&str> {
    if !analyses.crate_env_no_std {
        return None;
    }
    let analysis = analyses.e0599.as_ref()?;
    let target = analysis.target.as_ref()?;
    if target.method_name != "to_string" || target.raw_kind != "type" {
        return None;
    }
    if !matches!(
        target.normalized_type_name.as_str(),
        "bool"
            | "char"
            | "str"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
    ) {
        return None;
    }
    Some(target.normalized_type_name.as_str())
}

fn e0308_classification_name(kind: &E0308Classification) -> &'static str {
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
    use ruter::patchers::e0308::{
        ExpectedFoundConfidence, ExpectedFoundPair, ExpectedFoundSource,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn preflight_digest_includes_e0308_keys() {
        let analyses = PreflightAnalyses {
            crate_env_no_std: false,
            low_value: LowValueAnalysis {
                status: LowValueStatus::HasTestSemantics,
                reason: "markers".to_string(),
                markers: vec!["macro:assert_eq!".to_string()],
            },
            e0599: None,
            e0308: Some(E0308Analysis {
                pair: Some(ExpectedFoundPair {
                    expected_type: "Option<String>".to_string(),
                    found_type: "String".to_string(),
                    source: ExpectedFoundSource::PrimaryLabel,
                    confidence: ExpectedFoundConfidence::High,
                }),
                classification: E0308Classification::WrapperMismatch,
                hints: vec!["wrap with Some(...)".to_string()],
                summary: "s".to_string(),
            }),
            e0432: None,
            e0560: None,
        };

        let digest = build_preflight_interceptor_digest(&analyses).expect("digest");
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("CRATE_ENV_NO_STD=false"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0308_CLASSIFICATION=WRAPPER_MISMATCH"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0308_EXPECTED=Option<String>"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0308_FOUND=String"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0308_HINTS=wrap with Some(...)"))
        );
    }

    #[test]
    fn preflight_digest_includes_e0308_array_len_keys_when_available() {
        let analyses = PreflightAnalyses {
            crate_env_no_std: true,
            low_value: LowValueAnalysis {
                status: LowValueStatus::HasTestSemantics,
                reason: "markers".to_string(),
                markers: vec![],
            },
            e0599: None,
            e0308: Some(E0308Analysis {
                pair: Some(ExpectedFoundPair {
                    expected_type: "&mut [MaybeUninit<u8>; 40]".to_string(),
                    found_type: "&mut [MaybeUninit<u8>; 42]".to_string(),
                    source: ExpectedFoundSource::Message,
                    confidence: ExpectedFoundConfidence::Medium,
                }),
                classification: E0308Classification::Unknown,
                hints: vec![],
                summary: "s".to_string(),
            }),
            e0432: None,
            e0560: None,
        };

        let digest = build_preflight_interceptor_digest(&analyses).expect("digest");
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("CRATE_ENV_NO_STD=true"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0308_EXPECTED_ARRAY_LEN=40"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0308_FOUND_ARRAY_LEN=42"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0308_LEN_DELTA=2"))
        );
    }

    #[test]
    fn preflight_digest_includes_e0432_keys() {
        let analyses = PreflightAnalyses {
            crate_env_no_std: false,
            low_value: LowValueAnalysis {
                status: LowValueStatus::HasTestSemantics,
                reason: "markers".to_string(),
                markers: vec![],
            },
            e0599: None,
            e0308: None,
            e0432: Some(E0432PreflightAnalysis {
                count: 2,
                hints: vec!["crate::duration".to_string(), "crate::foo::Bar".to_string()],
                summary: "E0432 unresolved imports detected: count=2, replacement_hints=2"
                    .to_string(),
            }),
            e0560: None,
        };

        let digest = build_preflight_interceptor_digest(&analyses).expect("digest");
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0432_COUNT=2"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0432_SUMMARY="))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0432_HINTS=crate::duration | crate::foo::Bar"))
        );
    }

    #[test]
    fn preflight_digest_marks_no_std_primitive_to_string_guard() {
        let analyses = PreflightAnalyses {
            crate_env_no_std: true,
            low_value: LowValueAnalysis {
                status: LowValueStatus::HasTestSemantics,
                reason: "markers".to_string(),
                markers: vec![],
            },
            e0599: Some(E0599Analysis {
                classification: E0599Classification::MinimalPass,
                target: Some(ruter::patchers::e0599::E0599Target {
                    method_name: "to_string".to_string(),
                    raw_type_name: "usize".to_string(),
                    normalized_type_name: "usize".to_string(),
                    raw_kind: "type".to_string(),
                }),
                matched_impl_count: 0,
                available_method_signatures: vec![],
                related_free_function_signatures: vec![],
                recommended_call_forms: vec![],
                scope: "none".to_string(),
                summary: "x".to_string(),
            }),
            e0308: None,
            e0432: None,
            e0560: None,
        };

        let digest = build_preflight_interceptor_digest(&analyses).expect("digest");
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0599_NO_STD_PRIMITIVE_TOSTRING=true type=usize"))
        );
    }

    #[test]
    fn preflight_digest_includes_e0599_free_function_namespace_hints() {
        let analyses = PreflightAnalyses {
            crate_env_no_std: false,
            low_value: LowValueAnalysis {
                status: LowValueStatus::HasTestSemantics,
                reason: "markers".to_string(),
                markers: vec![],
            },
            e0599: Some(E0599Analysis {
                classification: E0599Classification::MisplacedFreeFunction,
                target: Some(ruter::patchers::e0599::E0599Target {
                    method_name: "format_rfc3339".to_string(),
                    raw_type_name: "Rfc3339Timestamp".to_string(),
                    normalized_type_name: "Rfc3339Timestamp".to_string(),
                    raw_kind: "struct".to_string(),
                }),
                matched_impl_count: 1,
                available_method_signatures: vec!["fn get_ref (& self)".to_string()],
                related_free_function_signatures: vec![
                    "fn format_rfc3339 (system_time : SystemTime) -> Rfc3339Timestamp".to_string(),
                ],
                recommended_call_forms: vec![
                    "format_rfc3339(...)".to_string(),
                    "crate::format_rfc3339(...)".to_string(),
                ],
                scope: "crate_fallback".to_string(),
                summary: "s".to_string(),
            }),
            e0308: None,
            e0432: None,
            e0560: None,
        };

        let digest = build_preflight_interceptor_digest(&analyses).expect("digest");
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0599_CLASSIFICATION=MISPLACED_FREE_FUNCTION"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0599_RELATED_FREE_FN_SIGNATURES="))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0599_NAMESPACE_HINT="))
        );
    }

    #[test]
    fn preflight_digest_includes_e0560_keys() {
        let analyses = PreflightAnalyses {
            crate_env_no_std: false,
            low_value: LowValueAnalysis {
                status: LowValueStatus::HasTestSemantics,
                reason: "markers".to_string(),
                markers: vec![],
            },
            e0599: None,
            e0308: None,
            e0432: None,
            e0560: Some(E0560PreflightAnalysis {
                count: 2,
                unknown_fields: vec!["ofset".to_string(), "cur".to_string()],
                available_fields: vec!["offset".to_string(), "pos".to_string()],
                hints: vec!["offset".to_string()],
            }),
        };

        let digest = build_preflight_interceptor_digest(&analyses).expect("digest");
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0560_COUNT=2"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0560_UNKNOWN_FIELDS=ofset | cur"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0560_AVAILABLE_FIELDS=offset | pos"))
        );
        assert!(
            digest
                .notes
                .iter()
                .any(|line| line.contains("E0560_HINTS=offset"))
        );
    }

    #[test]
    fn preflight_digest_e0560_keys_are_limited_to_contract() {
        let analyses = PreflightAnalyses {
            crate_env_no_std: false,
            low_value: LowValueAnalysis {
                status: LowValueStatus::HasTestSemantics,
                reason: "markers".to_string(),
                markers: vec![],
            },
            e0599: None,
            e0308: None,
            e0432: None,
            e0560: Some(E0560PreflightAnalysis {
                count: 1,
                unknown_fields: vec!["ofset".to_string()],
                available_fields: vec!["offset".to_string()],
                hints: vec!["offset".to_string()],
            }),
        };

        let digest = build_preflight_interceptor_digest(&analyses).expect("digest");
        let e0560_keys = digest
            .notes
            .iter()
            .filter_map(|line| line.split_once('=').map(|(key, _)| key))
            .filter(|key| key.starts_with("E0560_"))
            .collect::<Vec<_>>();
        assert_eq!(
            e0560_keys,
            vec![
                "E0560_COUNT",
                "E0560_UNKNOWN_FIELDS",
                "E0560_AVAILABLE_FIELDS",
                "E0560_HINTS"
            ]
        );
    }

    #[test]
    fn detect_crate_no_std_reads_lib_and_main() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("lib.rs"), "#![no_std]\npub fn a() {}\n").expect("write lib");
        assert!(detect_crate_no_std(dir.path()));

        fs::write(src.join("lib.rs"), "pub fn a() {}\n").expect("rewrite lib");
        fs::write(src.join("main.rs"), "#![no_std]\nfn main() {}\n").expect("write main");
        assert!(detect_crate_no_std(dir.path()));
    }
}
