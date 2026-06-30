use std::path::Path;

use crate::core::SpanInfo;

/// Detect whether a diagnostic span is inside mutable test code context.
///
/// Rules:
/// 1) File path contains `tests/`
/// 2) Source prefix before span contains `#[cfg(test)]`
/// 3) Source prefix before span contains `#[test]`
pub struct TestContextDetector;

impl TestContextDetector {
    /// Check if current span is in test context.
    ///
    /// `source` is required:
    /// Unit tests under `src/` need attribute probing.
    pub fn is_test_context(span: &SpanInfo, source: &str) -> bool {
        if Self::is_in_integration_tests_dir(&span.file_path) {
            return true;
        }

        // M1-1: Use lightweight marker search first.
        // Upgrade to AST/LSP parsing later.
        Self::has_test_attributes_before_span(source, span.byte_start)
    }

    fn is_in_integration_tests_dir(file_path: &Path) -> bool {
        file_path
            .components()
            .any(|comp| comp.as_os_str() == "tests")
    }

    fn has_test_attributes_before_span(source: &str, byte_start: usize) -> bool {
        let end = byte_start.min(source.len());
        let prefix = &source[..end];
        prefix.contains("#[cfg(test)]") || prefix.contains("#[test]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn span(path: &str, byte_start: usize) -> SpanInfo {
        SpanInfo {
            file_path: PathBuf::from(path),
            byte_start,
            byte_end: byte_start + 5,
            line_start: 1,
            line_end: 1,
            col_start: 1,
            col_end: 6,
            is_primary: true,
            text: vec![],
            label: None,
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        }
    }

    #[test]
    fn test_context_detector_accepts_tests_dir() {
        let src = "fn it_works() { let _ = State::new(); }";
        let s = span("tests/e2e.rs", src.find("State").unwrap());
        assert!(TestContextDetector::is_test_context(&s, src));
    }

    #[test]
    fn test_context_detector_accepts_cfg_test_module() {
        let src = r#"
#[cfg(test)]
mod tests {
    fn demo() {
        let _ = State::new();
    }
}
"#;
        let s = span("src/lib.rs", src.find("State").unwrap());
        assert!(TestContextDetector::is_test_context(&s, src));
    }

    #[test]
    fn test_context_detector_accepts_test_fn() {
        let src = r#"
#[test]
fn demo() {
    let _ = State::new();
}
"#;
        let s = span("src/lib.rs", src.find("State").unwrap());
        assert!(TestContextDetector::is_test_context(&s, src));
    }

    #[test]
    fn test_context_detector_rejects_non_test_context() {
        let src = "fn main() { let _ = State::new(); }";
        let s = span("src/main.rs", src.find("State").unwrap());
        assert!(!TestContextDetector::is_test_context(&s, src));
    }
}
