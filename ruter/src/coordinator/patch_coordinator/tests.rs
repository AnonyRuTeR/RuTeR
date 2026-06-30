use super::*;
use crate::core::{Applicability, CompilerCode, ErrorCode, Severity, SpanInfo};
use crate::patchers::{E0433Patcher, Patcher};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn make_registry_with_e0433() -> PatcherRegistry {
    let mut registry = PatcherRegistry::new();
    registry.register(Box::new(E0433Patcher::new()));
    registry
}

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

fn make_e0433_diagnostic(file: &Path, source: &str, replacement: &str) -> Diagnostic {
    let primary = make_span(file.to_path_buf(), source, "State", true, None, None);
    let child = make_span(
        file.to_path_buf(),
        source,
        "State",
        false,
        Some(replacement),
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

fn make_e0433_diagnostic_with_replacements(
    file: &Path,
    source: &str,
    needle: &str,
    replacements: &[&str],
) -> Diagnostic {
    let primary = make_span(file.to_path_buf(), source, needle, true, None, None);
    let child_spans = replacements
        .iter()
        .map(|replacement| {
            make_span(
                file.to_path_buf(),
                source,
                needle,
                false,
                Some(replacement),
                Some(Applicability::MachineApplicable),
            )
        })
        .collect::<Vec<_>>();
    let children = child_spans
        .into_iter()
        .map(|span| Diagnostic {
            message_type: None,
            code: None,
            message: "help: consider importing this type".to_string(),
            span: vec![span],
            severity: Severity::Help,
            children: vec![],
            rendered: None,
        })
        .collect();

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
        children,
        rendered: None,
    }
}

#[derive(Debug)]
struct MockPatcher {
    code: ErrorCode,
    fixes: Vec<FixAction>,
}

impl Patcher for MockPatcher {
    fn error_code(&self) -> ErrorCode {
        self.code
    }

    fn analyze(&self, _diagnostic: &Diagnostic) -> Result<Vec<FixAction>> {
        Ok(self.fixes.clone())
    }

    fn description(&self) -> &'static str {
        "mock"
    }
}

#[test]
fn plan_single_file_single_diagnostic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("main.rs");
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn coordinator_case() { let _ = State::new(); }
}
"#;
    fs::write(&file, source).unwrap();

    let diagnostics = vec![make_e0433_diagnostic(&file, source, "crate::foo::State")];
    let coordinator = PatchCoordinator::new(make_registry_with_e0433(), CodeTransformer::new());

    let plan = coordinator.plan(&diagnostics).unwrap();
    assert_eq!(plan.len(), 1);
    assert_eq!(plan.get(&file).unwrap().len(), 1);
}

#[test]
fn plan_single_file_multiple_diagnostics_no_conflict() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("main.rs");
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn coordinator_case() { let _a = State::new(); let _b = State::new(); }
}
"#;
    fs::write(&file, source).unwrap();

    let first = make_e0433_diagnostic(&file, source, "crate::foo::State");
    let mut second = make_e0433_diagnostic(&file, source, "crate::bar::State");
    let second_start = source.rfind("State").unwrap();
    let second_end = second_start + "State".len();
    second.span[0].byte_start = second_start;
    second.span[0].byte_end = second_end;
    second.children[0].span[0].byte_start = second_start;
    second.children[0].span[0].byte_end = second_end;

    let coordinator = PatchCoordinator::new(make_registry_with_e0433(), CodeTransformer::new());
    let plan = coordinator.plan(&[first, second]).unwrap();

    assert_eq!(plan.len(), 1);
    assert_eq!(plan.get(&file).unwrap().len(), 2);
}

#[test]
fn plan_multiple_files_grouped() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("a.rs");
    let file2 = dir.path().join("b.rs");
    let src = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn coordinator_case() { let _ = State::new(); }
}
"#;
    fs::write(&file1, src).unwrap();
    fs::write(&file2, src).unwrap();

    let d1 = make_e0433_diagnostic(&file1, src, "crate::a::State");
    let d2 = make_e0433_diagnostic(&file2, src, "crate::b::State");

    let coordinator = PatchCoordinator::new(make_registry_with_e0433(), CodeTransformer::new());
    let plan = coordinator.plan(&[d1, d2]).unwrap();

    assert_eq!(plan.len(), 2);
    assert!(plan.contains_key(&file1));
    assert!(plan.contains_key(&file2));
}

#[test]
fn plan_conflict_returns_error() {
    let mut registry = PatcherRegistry::new();
    let file = PathBuf::from("src/main.rs");

    let fixes = vec![
        FixAction::Replace {
            span: SpanInfo {
                file_path: file.clone(),
                byte_start: 1,
                byte_end: 4,
                line_start: 1,
                line_end: 1,
                col_start: 2,
                col_end: 5,
                is_primary: true,
                text: vec![],
                label: None,
                suggested_replacement: None,
                suggestion_applicability: None,
                expansion: None,
            },
            new_content: "AAA".to_string(),
        },
        FixAction::Replace {
            span: SpanInfo {
                file_path: file.clone(),
                byte_start: 3,
                byte_end: 5,
                line_start: 1,
                line_end: 1,
                col_start: 4,
                col_end: 6,
                is_primary: true,
                text: vec![],
                label: None,
                suggested_replacement: None,
                suggestion_applicability: None,
                expansion: None,
            },
            new_content: "BBB".to_string(),
        },
    ];

    registry.register(Box::new(MockPatcher {
        code: ErrorCode::E0433,
        fixes,
    }));

    let diag = Diagnostic {
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

    let coordinator = PatchCoordinator::new(registry, CodeTransformer::new());
    let err = coordinator.plan(&[diag]).expect_err("must fail on overlap");
    assert!(matches!(err, RuTeRError::ConflictingFixActions(_)));
}

#[test]
fn plan_skips_when_no_patcher_registered() {
    let registry = PatcherRegistry::new();
    let coordinator = PatchCoordinator::new(registry, CodeTransformer::new());

    let diag = Diagnostic {
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

    let plan = coordinator.plan(&[diag]).unwrap();
    assert!(plan.is_empty());
}

#[test]
fn plan_top_k_returns_ranked_global_candidates() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("main.rs");
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn coordinator_case() {
        let _a = State::new();
        let _b = Other::new();
    }
}
"#;
    fs::write(&file, source).unwrap();

    let first = make_e0433_diagnostic_with_replacements(
        &file,
        source,
        "State",
        &["crate::best::State", "external::best::State"],
    );
    let second = make_e0433_diagnostic_with_replacements(
        &file,
        source,
        "Other",
        &["crate::best::Other", "external::best::Other"],
    );

    let coordinator = PatchCoordinator::new(make_registry_with_e0433(), CodeTransformer::new());
    let candidates = coordinator.plan_top_k(&[first, second], 3).unwrap();

    assert_eq!(candidates.len(), 3);
    assert!(candidates[0].score >= candidates[1].score);
    assert!(candidates[1].score >= candidates[2].score);
    assert!(!candidates[0].plan.is_empty());
}

#[test]
fn plan_top_k_prunes_conflicting_combinations() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("main.rs");
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn coordinator_case() { let _ = State::new(); }
}
"#;
    fs::write(&file, source).unwrap();

    let first = make_e0433_diagnostic_with_replacements(
        &file,
        source,
        "State",
        &["crate::foo::State", "crate::bar::State"],
    );
    let mut second = make_e0433_diagnostic_with_replacements(
        &file,
        source,
        "State",
        &["crate::foo::State", "crate::bar::State"],
    );
    // Force same span to simulate conflicting alternative combinations.
    second.span[0].byte_start = first.span[0].byte_start;
    second.span[0].byte_end = first.span[0].byte_end;
    for child in &mut second.children {
        child.span[0].byte_start = first.span[0].byte_start;
        child.span[0].byte_end = first.span[0].byte_end;
    }

    let coordinator = PatchCoordinator::new(make_registry_with_e0433(), CodeTransformer::new());
    let candidates = coordinator.plan_top_k(&[first, second], 3).unwrap();

    assert!(
        candidates.is_empty(),
        "all conflicting combinations should be pruned"
    );
}
