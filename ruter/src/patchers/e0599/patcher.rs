use crate::core::{Diagnostic, ErrorCode, FixAction, Result};
use crate::patchers::Patcher;

/// E0599 rule patcher placeholder.
///
/// v1 only marks E0599 as an implemented rule code so function dispatch
/// can keep RuleFirst coverage. Concrete code actions are still handled
/// by preflight analyzer + LLM path.
#[derive(Debug, Default)]
pub struct E0599Patcher;

impl E0599Patcher {
    pub fn new() -> Self {
        Self
    }
}

impl Patcher for E0599Patcher {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::E0599
    }

    fn analyze(&self, diagnostic: &Diagnostic) -> Result<Vec<FixAction>> {
        if !self.can_handle(diagnostic) {
            return Ok(Vec::new());
        }
        Ok(Vec::new())
    }

    fn description(&self) -> &'static str {
        "Patcher for E0599: analysis-only placeholder for RuleFirst coverage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CompilerCode, Severity};

    #[test]
    fn e0599_patcher_reports_target_code() {
        let patcher = E0599Patcher::new();
        assert_eq!(patcher.error_code(), ErrorCode::E0599);
    }

    #[test]
    fn e0599_patcher_returns_empty_actions() {
        let patcher = E0599Patcher::new();
        let diagnostic = Diagnostic {
            message_type: Some("diagnostic".to_string()),
            code: Some(CompilerCode {
                code: ErrorCode::E0599,
                raw_code: None,
                explanation: None,
            }),
            message: "no function or associated item named `new` found for struct `Parser`"
                .to_string(),
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
