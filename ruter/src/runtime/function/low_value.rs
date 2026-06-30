use std::collections::BTreeSet;
use std::fs;

use serde::{Deserialize, Serialize};
use syn::visit::Visit;

use crate::runtime::function::index::TestFunction;

/// LowValue classification for test function semantics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LowValueStatus {
    LowValue,
    HasTestSemantics,
    Unknown,
}

/// Result of low-value analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowValueAnalysis {
    pub status: LowValueStatus,
    pub reason: String,
    #[serde(default)]
    pub markers: Vec<String>,
}

/// Analyze whether a test function has meaningful test semantics.
pub fn analyze_test_function_low_value(function: &TestFunction) -> LowValueAnalysis {
    let source = match fs::read_to_string(&function.file_path) {
        Ok(value) => value,
        Err(err) => {
            return LowValueAnalysis {
                status: LowValueStatus::Unknown,
                reason: format!("failed to read function source: {err}"),
                markers: Vec::new(),
            };
        }
    };

    let start = function.byte_start.min(source.len());
    let end = function.byte_end.min(source.len());
    if start >= end {
        return LowValueAnalysis {
            status: LowValueStatus::Unknown,
            reason: "invalid function byte range".to_string(),
            markers: Vec::new(),
        };
    }

    analyze_function_text_low_value(&source[start..end])
}

/// Analyze one function item text.
pub fn analyze_function_text_low_value(function_text: &str) -> LowValueAnalysis {
    let parsed = match syn::parse_str::<syn::ItemFn>(function_text) {
        Ok(value) => value,
        Err(err) => {
            return LowValueAnalysis {
                status: LowValueStatus::Unknown,
                reason: format!("failed to parse function item: {err}"),
                markers: Vec::new(),
            };
        }
    };

    let mut collector = TestSemanticMarkerCollector::default();
    collector.visit_item_fn(&parsed);
    let markers = collector.into_markers();
    if markers.is_empty() {
        return LowValueAnalysis {
            status: LowValueStatus::LowValue,
            reason: "no test semantics marker found".to_string(),
            markers,
        };
    }

    LowValueAnalysis {
        status: LowValueStatus::HasTestSemantics,
        reason: "test semantics markers detected".to_string(),
        markers,
    }
}

#[derive(Default)]
struct TestSemanticMarkerCollector {
    markers: BTreeSet<String>,
}

impl TestSemanticMarkerCollector {
    fn into_markers(self) -> Vec<String> {
        self.markers.into_iter().collect()
    }

    fn record_macro_ident(&mut self, ident: &str) {
        match ident {
            "assert" => {
                self.markers.insert("macro:assert!".to_string());
            }
            "assert_eq" => {
                self.markers.insert("macro:assert_eq!".to_string());
            }
            "assert_ne" => {
                self.markers.insert("macro:assert_ne!".to_string());
            }
            "debug_assert" => {
                self.markers.insert("macro:debug_assert!".to_string());
            }
            "debug_assert_eq" => {
                self.markers.insert("macro:debug_assert_eq!".to_string());
            }
            "debug_assert_ne" => {
                self.markers.insert("macro:debug_assert_ne!".to_string());
            }
            "assert_matches" => {
                self.markers.insert("macro:assert_matches!".to_string());
            }
            "panic" => {
                self.markers.insert("macro:panic!".to_string());
            }
            "unreachable" => {
                self.markers.insert("macro:unreachable!".to_string());
            }
            "todo" => {
                self.markers.insert("macro:todo!".to_string());
            }
            "unimplemented" => {
                self.markers.insert("macro:unimplemented!".to_string());
            }
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for TestSemanticMarkerCollector {
    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if let Some(ident) = node
            .mac
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        {
            self.record_macro_ident(&ident);
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(ident) = node
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        {
            self.record_macro_ident(&ident);
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        match node.method.to_string().as_str() {
            "unwrap" => {
                self.markers.insert("method:unwrap".to_string());
            }
            "expect" => {
                self.markers.insert("method:expect".to_string());
            }
            _ => {}
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_try(&mut self, node: &'ast syn::ExprTry) {
        self.markers.insert("operator:?".to_string());
        syn::visit::visit_expr_try(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_value_detects_assert_and_debug_assert_markers() {
        let text = r#"
#[test]
fn case_a() {
    assert_eq!(1, 1);
    debug_assert_ne!(1, 2);
}
"#;
        let analysis = analyze_function_text_low_value(text);
        assert_eq!(analysis.status, LowValueStatus::HasTestSemantics);
        assert!(
            analysis
                .markers
                .iter()
                .any(|marker| marker == "macro:assert_eq!")
        );
        assert!(
            analysis
                .markers
                .iter()
                .any(|marker| marker == "macro:debug_assert_ne!")
        );
    }

    #[test]
    fn low_value_detects_try_operator_and_expect() {
        let text = r#"
#[test]
fn case_b() -> Result<(), String> {
    let _ = Some(1).expect("v");
    let _ = Ok::<_, String>(()).map_err(|e| e)?;
    Ok(())
}
"#;
        let analysis = analyze_function_text_low_value(text);
        assert_eq!(analysis.status, LowValueStatus::HasTestSemantics);
        assert!(
            analysis
                .markers
                .iter()
                .any(|marker| marker == "method:expect")
        );
        assert!(analysis.markers.iter().any(|marker| marker == "operator:?"));
    }

    #[test]
    fn low_value_marks_empty_semantics_as_low_value() {
        let text = r#"
#[test]
fn case_c() {
    let _ = 1 + 2;
}
"#;
        let analysis = analyze_function_text_low_value(text);
        assert_eq!(analysis.status, LowValueStatus::LowValue);
        assert!(analysis.markers.is_empty());
    }

    #[test]
    fn low_value_marks_parse_failure_as_unknown() {
        let text = r#"
#[test]
fn case_d() {
    let _ = 1
"#;
        let analysis = analyze_function_text_low_value(text);
        assert_eq!(analysis.status, LowValueStatus::Unknown);
        assert!(analysis.reason.contains("failed to parse"));
    }

    #[test]
    fn low_value_detects_assert_matches_macro() {
        let text = r#"
#[test]
fn case_e() {
    let v = Ok::<_, ()>(1);
    assert_matches!(v, Ok(_));
}
"#;
        let analysis = analyze_function_text_low_value(text);
        assert_eq!(analysis.status, LowValueStatus::HasTestSemantics);
        assert!(
            analysis
                .markers
                .iter()
                .any(|marker| marker == "macro:assert_matches!")
        );
    }
}
