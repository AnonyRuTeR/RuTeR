use super::*;
use crate::core::{CompilerCode, ErrorCode, Severity};
use tempfile::TempDir;

fn write_fixture(source: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    fs::write(
        dir.path().join("Cargo.toml"),
        r#"[package]
name = "demo_pkg"
version = "0.1.0"
edition = "2021"
"#,
    )
    .expect("write manifest");

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("mkdir src");
    let file = src_dir.join("lib.rs");
    fs::write(&file, source).expect("write source");
    (dir, file)
}

fn line_and_col(source: &str, byte: usize) -> (usize, usize) {
    let prefix = &source[..byte];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
    let col = prefix
        .rsplit('\n')
        .next()
        .map(|value| value.len())
        .unwrap_or(0)
        + 1;
    (line, col)
}

fn make_span(
    source: &str,
    file: &Path,
    needle: &str,
    is_primary: bool,
    replacement: Option<&str>,
    applicability: Option<Applicability>,
) -> SpanInfo {
    let start = source.find(needle).expect("needle must exist");
    let end = start + needle.len();
    let (line_start, col_start) = line_and_col(source, start);
    let (line_end, col_end) = line_and_col(source, end);
    SpanInfo {
        file_path: file.to_path_buf(),
        byte_start: start,
        byte_end: end,
        line_start,
        line_end,
        col_start,
        col_end,
        is_primary,
        text: vec![],
        label: None,
        suggested_replacement: replacement.map(ToString::to_string),
        suggestion_applicability: applicability,
        expansion: None,
    }
}

fn make_diag(message: &str, spans: Vec<SpanInfo>, children: Vec<Diagnostic>) -> Diagnostic {
    Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0432,
            raw_code: None,
            explanation: None,
        }),
        message: message.to_string(),
        span: spans,
        severity: Severity::Error,
        children,
        rendered: None,
    }
}

fn child_help(span: SpanInfo) -> Diagnostic {
    Diagnostic {
        message_type: None,
        code: None,
        message: "help".to_string(),
        span: vec![span],
        severity: Severity::Help,
        children: vec![],
        rendered: None,
    }
}

#[test]
fn p1_accepts_single_machine_applicable_suggestion() {
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        use demo_pkg::duration;
    }
}
"#;
    let (_dir, file) = write_fixture(source);
    let span = make_span(
        source,
        &file,
        "demo_pkg::duration",
        true,
        Some("crate::duration"),
        Some(Applicability::MachineApplicable),
    );
    let diag = make_diag("unresolved import `demo_pkg::duration`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate::duration");
        }
        _ => panic!("expected replace"),
    }
}

#[test]
fn p1_collects_non_overlapping_suggestions_from_children() {
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        use missing_a::ThingA;
        use missing_b::ThingB;
    }
}
"#;
    let (_dir, file) = write_fixture(source);
    let span1 = make_span(
        source,
        &file,
        "missing_a::ThingA",
        true,
        Some("crate::ThingA"),
        Some(Applicability::MachineApplicable),
    );
    let span2 = make_span(
        source,
        &file,
        "missing_b::ThingB",
        false,
        Some("crate::ThingB"),
        Some(Applicability::MachineApplicable),
    );
    let diag = make_diag(
        "unresolved import `missing_a::ThingA`",
        vec![span1],
        vec![child_help(span2)],
    );

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert_eq!(actions.len(), 2);
}

#[test]
fn p1_keeps_first_when_suggestions_overlap() {
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        use demo_pkg::alpha::beta;
    }
}
"#;
    let (_dir, file) = write_fixture(source);
    let first = make_span(
        source,
        &file,
        "demo_pkg",
        true,
        Some("crate"),
        Some(Applicability::MachineApplicable),
    );
    let second = make_span(
        source,
        &file,
        "demo_pkg::alpha",
        false,
        Some("crate::alpha"),
        Some(Applicability::MachineApplicable),
    );
    let diag = make_diag(
        "unresolved import `demo_pkg::alpha::beta`",
        vec![first],
        vec![child_help(second)],
    );

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate");
        }
        _ => panic!("expected replace"),
    }
}

#[test]
fn p1_ignores_non_machine_applicable_suggestions() {
    let source = "fn case_a() { use missing::Thing; }";
    let (_dir, file) = write_fixture(source);
    let span = make_span(
        source,
        &file,
        "missing::Thing",
        true,
        Some("crate::Thing"),
        Some(Applicability::MaybeIncorrect),
    );
    let diag = make_diag("unresolved import `missing::Thing`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn r1_rewrites_package_head_to_crate_when_exact_match() {
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        use demo_pkg::duration;
    }
}
"#;
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "demo_pkg::duration", true, None, None);
    let diag = make_diag("unresolved import `demo_pkg::duration`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate::duration");
        }
        _ => panic!("expected replace"),
    }
}

#[test]
fn r1_rejects_non_use_statement() {
    let source = "fn case_a() { demo_pkg::duration(); }";
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "demo_pkg::duration", true, None, None);
    let diag = make_diag("unresolved import `demo_pkg::duration`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn r1_requires_head_to_equal_package_name() {
    let source = "fn case_a() { use other_pkg::duration; }";
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "other_pkg::duration", true, None, None);
    let diag = make_diag(
        "unresolved import `other_pkg::duration`",
        vec![span],
        vec![],
    );

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn p1_has_higher_priority_than_r1() {
    let source = "fn case_a() { use demo_pkg::duration; }";
    let (_dir, file) = write_fixture(source);
    let span = make_span(
        source,
        &file,
        "demo_pkg::duration",
        true,
        Some("crate::duration"),
        Some(Applicability::MachineApplicable),
    );
    let diag = make_diag("unresolved import `demo_pkg::duration`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        FixAction::Replace { new_content, .. } => assert_eq!(new_content, "crate::duration"),
        _ => panic!("expected replace"),
    }
}

#[test]
fn r2_applies_under_test_attribute_context() {
    let source = r#"
#[test]
fn case_a() {
    use missing::Thing;
}
"#;
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "missing::Thing", true, None, None);
    let diag = make_diag("unresolved import `missing::Thing`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(
                new_content,
                "    // ruter(e0432-r2): disabled unresolved import: use missing::Thing;"
            );
        }
        _ => panic!("expected replace"),
    }
}

#[test]
fn r2_applies_under_tokio_test_attribute_context() {
    let source = r#"
#[tokio::test]
async fn case_a() {
    use missing::Thing;
}
"#;
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "missing::Thing", true, None, None);
    let diag = make_diag("unresolved import `missing::Thing`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert_eq!(actions.len(), 1);
}

#[test]
fn r2_applies_under_rstest_attribute_context() {
    let source = r#"
#[rstest]
fn case_a() {
    use missing::Thing;
}
"#;
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "missing::Thing", true, None, None);
    let diag = make_diag("unresolved import `missing::Thing`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert_eq!(actions.len(), 1);
}

#[test]
fn r2_rejects_non_test_context() {
    let source = r#"
fn case_a() {
    use missing::Thing;
}
"#;
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "missing::Thing", true, None, None);
    let diag = make_diag("unresolved import `missing::Thing`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn r2_rejects_non_use_line() {
    let source = r#"
#[test]
fn case_a() {
    missing::Thing::new();
}
"#;
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "missing::Thing", true, None, None);
    let diag = make_diag("unresolved import `missing::Thing`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert!(actions.is_empty());
}

#[test]
fn r2_rejects_multiline_use_statement() {
    let source = r#"
#[test]
fn case_a() {
    use missing::{
        Thing,
    };
}
"#;
    let (_dir, file) = write_fixture(source);
    let span = make_span(source, &file, "missing", true, None, None);
    let diag = make_diag("unresolved import `missing::Thing`", vec![span], vec![]);

    let actions = analyze_e0432_diagnostic(&diag).expect("analyze");
    assert!(actions.is_empty());
}
