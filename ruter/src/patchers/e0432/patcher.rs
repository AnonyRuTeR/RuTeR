use crate::core::{Diagnostic, ErrorCode, FixAction, Result};
use crate::patchers::Patcher;

use super::analyzer::analyze_e0432_diagnostic;

/// E0432 rule patcher.
///
/// It follows function-scoped tracks:
/// - P1: MachineApplicable compiler suggestions
/// - R1: package-name head rewrite to `crate::`
/// - R2: strict test-context line comment fallback
#[derive(Debug, Default)]
pub struct E0432Patcher;

impl E0432Patcher {
    pub fn new() -> Self {
        Self
    }
}

impl Patcher for E0432Patcher {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::E0432
    }

    fn analyze(&self, diagnostic: &Diagnostic) -> Result<Vec<FixAction>> {
        if !self.can_handle(diagnostic) {
            return Ok(Vec::new());
        }
        analyze_e0432_diagnostic(diagnostic)
    }

    fn description(&self) -> &'static str {
        "Patcher for E0432: function-scoped unresolved-import fixer (P1/R1/R2)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CompilerCode, Severity};

    #[test]
    fn e0432_patcher_reports_target_code() {
        let patcher = E0432Patcher::new();
        assert_eq!(patcher.error_code(), ErrorCode::E0432);
    }

    #[test]
    fn e0432_patcher_returns_empty_actions() {
        let patcher = E0432Patcher::new();
        let diagnostic = Diagnostic {
            message_type: Some("diagnostic".to_string()),
            code: Some(CompilerCode {
                code: ErrorCode::E0432,
                raw_code: None,
                explanation: None,
            }),
            message: "unresolved import `foo::bar`".to_string(),
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
