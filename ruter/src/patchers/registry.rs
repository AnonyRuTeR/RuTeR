use super::Patcher;
use crate::core::{Diagnostic, ErrorCode};
use std::collections::BTreeSet;

/// Registry to hold all available patchers
///
/// # Example
/// ```ignore
/// use ruter::patchers::{PatcherRegistry, E0433Patcher};
///
/// let mut registry = PatcherRegistry::new();
/// registry.register(Box::new(E0433Patcher::new()));
///
/// // lookup patcher by diagnostic
/// if let Some(patcher) = registry.find_patcher(&diagnostic) {
///    // use the patcher
///    let fixes = patcher.analyze(&diagnostic)?;
/// }
/// ```
pub struct PatcherRegistry {
    patchers: Vec<Box<dyn Patcher>>,
}

impl PatcherRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            patchers: Vec::new(),
        }
    }

    /// Register a new patcher
    ///
    /// # Arguments
    /// - 'patcher': Box<dyn Patcher> - The patcher to register
    pub fn register(&mut self, patcher: Box<dyn Patcher>) {
        self.patchers.push(patcher);
    }

    /// Lookup a patcher that can handle the given diagnostic
    ///
    /// # Arguments
    /// - 'diagnostic': &Diagnostic - The diagnostic to find a patcher for
    ///
    /// # Returns
    /// - 'Some(&dyn Patcher)': Finds a suitable patcher
    /// - 'None': No suitable patcher found
    pub fn find_patcher(&self, diagnostic: &Diagnostic) -> Option<&dyn Patcher> {
        self.patchers
            .iter()
            .find(|p| p.can_handle(diagnostic))
            // .map(|p| p.as_ref())
            .map(|p| p.as_ref())
    }

    /// Lookup patcher by exact rustc error code.
    pub fn find_patcher_by_code(&self, code: ErrorCode) -> Option<&dyn Patcher> {
        self.patchers
            .iter()
            .find(|patcher| patcher.error_code() == code)
            .map(|patcher| patcher.as_ref())
    }

    /// Return the set of rustc error codes currently covered by rule patchers.
    pub fn implemented_error_codes(&self) -> BTreeSet<ErrorCode> {
        self.patchers
            .iter()
            .map(|patcher| patcher.error_code())
            .collect()
    }

    /// Get the number of registered patchers
    pub fn len(&self) -> usize {
        self.patchers.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.patchers.is_empty()
    }
}

/// Intergration test for PatcherRegistry
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Applicability, CompilerCode, Diagnostic, ErrorCode, FixAction, RuTeRError, Severity,
        SpanInfo,
    };
    use crate::patchers::e0308::E0308Patcher;
    use crate::patchers::e0432::E0432Patcher;
    use crate::patchers::e0433::E0433Patcher;
    use crate::patchers::e0560::E0560Patcher;
    use crate::patchers::e0599::E0599Patcher;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn make_span(
        file_path: PathBuf,
        source: &str,
        needle: &str,
        is_primary: bool,
        replacement: Option<&str>,
        applicability: Option<Applicability>,
    ) -> SpanInfo {
        let start = source.find(needle).expect("needle must exist");
        let end = start + needle.len();
        SpanInfo {
            file_path,
            byte_start: start,
            byte_end: end,
            line_start: 1,
            line_end: 1,
            col_start: start + 1,
            col_end: end + 1,
            is_primary,
            text: vec![],
            label: None,
            suggested_replacement: replacement.map(|s| s.to_string()),
            suggestion_applicability: applicability,
            expansion: None,
        }
    }

    fn make_e0433_diagnostic(file: &Path, source: &str) -> Diagnostic {
        let primary = make_span(file.to_path_buf(), source, "State", true, None, None);
        let child = make_span(
            file.to_path_buf(),
            source,
            "State",
            false,
            Some("crate::foo::State"),
            Some(Applicability::MachineApplicable),
        );

        Diagnostic {
            message_type: Some("diagnostic".to_string()),
            code: Some(CompilerCode {
                code: ErrorCode::E0433,
                raw_code: None,
                explanation: None,
            }),
            message: "failed to resolve: use of undeclared type `State`".to_string(),
            span: vec![primary],
            severity: Severity::Error,
            children: vec![Diagnostic {
                message_type: None,
                code: None,
                message: "help: consider importing this type".to_string(),
                span: vec![child],
                severity: Severity::Help,
                children: vec![],
                rendered: None,
            }],
            rendered: None,
        }
    }

    #[test]
    fn registry_routes_to_e0433_and_generates_fix() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("main.rs");
        let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn registry_case() { let _ = State::new(); }
}
"#;
        fs::write(&file, source).expect("write source");

        let diagnostic = make_e0433_diagnostic(&file, source);
        let mut registry = PatcherRegistry::new();
        registry.register(Box::new(E0433Patcher::new()));

        let patcher = registry
            .find_patcher(&diagnostic)
            .expect("patcher must be found");
        let fixes = patcher
            .analyze(&diagnostic)
            .expect("analyze should succeed");

        assert_eq!(fixes.len(), 1);
        match &fixes[0] {
            FixAction::Replace { new_content, .. } => {
                assert_eq!(new_content, "crate::foo::State::new");
            }
            _ => panic!("expected Replace fix"),
        }
    }

    #[test]
    fn registry_returns_none_when_unregistered() {
        let file = PathBuf::from("dummy.rs");
        let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn registry_case() { let _ = State::new(); }
}
"#;
        let diagnostic = make_e0433_diagnostic(&file, source);
        let registry = PatcherRegistry::new();
        assert!(registry.find_patcher(&diagnostic).is_none());
    }

    #[test]
    fn registry_returns_none_for_non_target_error_code() {
        let mut registry = PatcherRegistry::new();
        registry.register(Box::new(E0433Patcher::new()));

        let diagnostic = Diagnostic {
            message_type: Some("diagnostic".to_string()),
            code: Some(CompilerCode {
                code: ErrorCode::E0308,
                raw_code: None,
                explanation: None,
            }),
            message: "mismatched types".to_string(),
            span: vec![],
            severity: Severity::Error,
            children: vec![],
            rendered: None,
        };

        assert!(registry.find_patcher(&diagnostic).is_none());
    }

    #[test]
    fn registry_propagates_patcher_error() {
        let mut registry = PatcherRegistry::new();
        registry.register(Box::new(E0433Patcher::new()));

        let fake_source = "State";
        let diagnostic = Diagnostic {
            message_type: Some("diagnostic".to_string()),
            code: Some(CompilerCode {
                code: ErrorCode::E0433,
                raw_code: None,
                explanation: None,
            }),
            message: "failed to resolve: use of undeclared type `State`".to_string(),
            span: vec![SpanInfo {
                file_path: PathBuf::from("/definitely/not/found/main.rs"),
                byte_start: 0,
                byte_end: 5,
                line_start: 1,
                line_end: 1,
                col_start: 1,
                col_end: 6,
                is_primary: true,
                text: vec![],
                label: None,
                suggested_replacement: Some("crate::foo::State".to_string()),
                suggestion_applicability: Some(Applicability::MachineApplicable),
                expansion: None,
            }],
            severity: Severity::Error,
            children: vec![],
            rendered: None,
        };

        let patcher = registry
            .find_patcher(&diagnostic)
            .expect("patcher must be found");
        let err = patcher
            .analyze(&diagnostic)
            .expect_err("must fail with source not found");

        match err {
            RuTeRError::SourceFileNotFound(path) => {
                assert!(path.contains("/definitely/not/found/main.rs"));
                assert_eq!(fake_source, "State");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn implemented_error_codes_includes_all_registered_patchers() {
        let mut registry = PatcherRegistry::new();
        registry.register(Box::new(E0433Patcher::new()));
        registry.register(Box::new(E0432Patcher::new()));
        registry.register(Box::new(E0599Patcher::new()));
        registry.register(Box::new(E0308Patcher::new()));
        registry.register(Box::new(E0560Patcher::new()));

        let codes = registry.implemented_error_codes();
        assert!(codes.contains(&ErrorCode::E0433));
        assert!(codes.contains(&ErrorCode::E0432));
        assert!(codes.contains(&ErrorCode::E0599));
        assert!(codes.contains(&ErrorCode::E0308));
        assert!(codes.contains(&ErrorCode::E0560));
    }
}
