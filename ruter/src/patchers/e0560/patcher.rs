use crate::core::{Diagnostic, ErrorCode, FixAction, Result};
use crate::patchers::Patcher;

use super::analyzer::analyze_e0560_diagnostic;

/// E0560 rule patcher.
///
/// Fixes high-confidence unknown-field cases on struct literals in test code.
#[derive(Debug, Default)]
pub struct E0560Patcher;

impl E0560Patcher {
    pub fn new() -> Self {
        Self
    }
}

impl Patcher for E0560Patcher {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::E0560
    }

    fn analyze(&self, diagnostic: &Diagnostic) -> Result<Vec<FixAction>> {
        if !self.can_handle(diagnostic) {
            return Ok(Vec::new());
        }
        analyze_e0560_diagnostic(diagnostic)
    }

    fn description(&self) -> &'static str {
        "Patcher for E0560: struct unknown-field fixer (P1/R1)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CompilerCode, Severity};

    #[test]
    fn e0560_patcher_reports_target_code() {
        let patcher = E0560Patcher::new();
        assert_eq!(patcher.error_code(), ErrorCode::E0560);
    }

    #[test]
    fn e0560_patcher_returns_empty_for_non_target_code() {
        let patcher = E0560Patcher::new();
        let diagnostic = Diagnostic {
            message_type: Some("diagnostic".to_string()),
            code: Some(CompilerCode {
                code: ErrorCode::E0433,
                raw_code: None,
                explanation: None,
            }),
            message: "failed to resolve".to_string(),
            span: vec![],
            severity: Severity::Error,
            children: vec![],
            rendered: None,
        };

        let actions = patcher
            .analyze(&diagnostic)
            .expect("analyze should succeed");
        assert!(actions.is_empty());
    }
}
