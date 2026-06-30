use super::*;
use crate::core::{Applicability, CompilerCode, Diagnostic, Severity};
use tempfile::tempdir;

fn make_span(
    file_path: PathBuf,
    source: &str,
    needle: &str,
    is_primary: bool,
    replacement: Option<&str>,
    applicability: Option<Applicability>,
) -> SpanInfo {
    let start = source.find(needle).expect("needle must exist in source");
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

fn make_diagnostic(
    file: &Path,
    source: &str,
    primary_needle: &str,
    child_replacement: Option<&str>,
) -> Diagnostic {
    let primary = make_span(file.to_path_buf(), source, primary_needle, true, None, None);

    let child_span = child_replacement.map(|rep| {
        make_span(
            file.to_path_buf(),
            source,
            primary_needle,
            false,
            Some(rep),
            Some(Applicability::MachineApplicable),
        )
    });

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
        children: match child_span {
            Some(span) => vec![Diagnostic {
                message_type: None,
                code: None,
                message: "help: consider importing this type".to_string(),
                span: vec![span],
                severity: Severity::Help,
                children: vec![],
                rendered: None,
            }],
            None => vec![],
        },
        rendered: None,
    }
}

#[test]
fn analyze_generates_replace_fix_with_suffix() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("main.rs");
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn e0433_case() { let _ = State::new(); }
}
"#;
    fs::write(&file, source).expect("write source");

    let diagnostic = make_diagnostic(&file, source, "State", Some("crate::foo::State"));
    let patcher = E0433Patcher::new();

    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);

    match &fixes[0] {
        FixAction::Replace { span, new_content } => {
            assert_eq!(new_content, "crate::foo::State::new");
            assert_eq!(span.byte_start, source.find("State").unwrap());
            assert_eq!(
                span.byte_end,
                source.find("State").unwrap() + "State::new".len()
            );
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn analyze_returns_empty_for_non_e0433() {
    let patcher = E0433Patcher::new();
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

    let fixes = patcher
        .analyze(&diagnostic)
        .expect("non-target code should not error");
    assert!(fixes.is_empty());
}

#[test]
fn analyze_returns_empty_when_no_candidate_suggestion() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("main.rs");
    let source = "fn main() { let _ = unknown(); }";
    fs::write(&file, source).expect("write source");

    let diagnostic = Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: "some unrelated error message".to_string(),
        span: vec![make_span(file.clone(), source, "unknown", true, None, None)],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    };

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("should degrade safely");
    assert!(fixes.is_empty());
}

#[test]
fn analyze_returns_empty_for_non_test_context_even_with_candidate() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("main.rs");
    let source = "fn main() { let _ = State::new(); }";
    fs::write(&file, source).expect("write source");

    let diagnostic = make_diagnostic(&file, source, "State", Some("crate::foo::State"));
    let patcher = E0433Patcher::new();

    let fixes = patcher
        .analyze(&diagnostic)
        .expect("non-test context should be skipped safely");
    assert!(
        fixes.is_empty(),
        "non-test context must not produce fix actions"
    );
}

#[test]
fn analyze_returns_source_not_found_error() {
    let file = PathBuf::from("/definitely/not/found/main.rs");
    let source_stub = "State";

    let diagnostic = Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: "failed to resolve: use of undeclared type `State`".to_string(),
        span: vec![SpanInfo {
            file_path: file.clone(),
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

    let patcher = E0433Patcher::new();
    let err = patcher
        .analyze(&diagnostic)
        .expect_err("must return source-not-found");

    match err {
        RuTeRError::SourceFileNotFound(path) => {
            assert!(path.contains("/definitely/not/found/main.rs"));
            assert_eq!(source_stub, "State");
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

fn write_test_crate_layout_with_manifest(dir: &Path, manifest: &str, source: &str) -> PathBuf {
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::write(dir.join("Cargo.toml"), manifest).expect("write manifest");
    let file = src_dir.join("lib.rs");
    fs::write(&file, source).expect("write source");
    file
}

fn write_test_crate_layout(dir: &Path, source: &str) -> PathBuf {
    write_test_crate_layout_with_manifest(
        dir,
        r#"[package]
name = "humantime"
version = "0.1.0"
edition = "2024"
"#,
        source,
    )
}

fn write_extra_src_file(dir: &Path, rel: &str, source: &str) {
    let path = dir.join("src").join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create extra src parent");
    }
    fs::write(path, source).expect("write extra src file");
}

fn make_unlinked_crate_diagnostic(
    file: &Path,
    source: &str,
    needle: &str,
    ident: &str,
) -> Diagnostic {
    Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: format!("failed to resolve: use of unresolved module or unlinked crate `{ident}`"),
        span: vec![make_span(
            file.to_path_buf(),
            source,
            needle,
            true,
            None,
            None,
        )],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    }
}

fn make_crate_root_not_found_diagnostic(
    file: &Path,
    source: &str,
    needle: &str,
    ident: &str,
) -> Diagnostic {
    Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: format!("failed to resolve: could not find `{ident}` in the crate root"),
        span: vec![make_span(
            file.to_path_buf(),
            source,
            needle,
            true,
            None,
            None,
        )],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    }
}

fn make_missing_crate_diagnostic(
    file: &Path,
    source: &str,
    needle: &str,
    ident: &str,
) -> Diagnostic {
    Diagnostic {
        message_type: Some("diagnostic".to_string()),
        code: Some(CompilerCode {
            code: ErrorCode::E0433,
            raw_code: None,
            explanation: None,
        }),
        message: format!("failed to resolve: you might be missing crate `{ident}`"),
        span: vec![make_span(
            file.to_path_buf(),
            source,
            needle,
            true,
            None,
            None,
        )],
        severity: Severity::Error,
        children: vec![],
        rendered: None,
    }
}

#[test]
fn patcher_injects_crate_candidate_for_package_prefixed_function_call() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime::format_rfc3339(std::time::UNIX_EPOCH).to_string();
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic = make_unlinked_crate_diagnostic(&file, source, "humantime", "humantime");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate::format_rfc3339");
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn patcher_injects_crate_candidate_for_package_prefixed_type_path() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime::Timestamp::from(std::time::UNIX_EPOCH);
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic = make_unlinked_crate_diagnostic(&file, source, "humantime", "humantime");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate::Timestamp::from");
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn patcher_does_not_inject_when_ident_not_package_name() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = chrono::DateTime::UNIX_EPOCH;
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic = make_unlinked_crate_diagnostic(&file, source, "chrono", "chrono");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert!(
        fixes.is_empty(),
        "package-name heuristic should not trigger for non-package prefix"
    );
}

#[test]
fn unresolved_head_generates_crate_candidate_for_unique_function() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
pub fn format_duration(_v: std::time::Duration) -> String {
    "ok".to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime_core::format_duration(std::time::Duration::new(1, 0));
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic =
        make_unlinked_crate_diagnostic(&file, source, "humantime_core", "humantime_core");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate::format_duration");
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn unresolved_head_generates_crate_candidate_for_unique_type_path() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
pub struct Timestamp;
impl Timestamp {
    pub fn from(_v: std::time::SystemTime) -> Self {
        Timestamp
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime_core::Timestamp::from(std::time::UNIX_EPOCH);
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic =
        make_unlinked_crate_diagnostic(&file, source, "humantime_core", "humantime_core");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate::Timestamp::from");
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn unresolved_head_generates_crate_candidate_for_crate_root_not_found_message() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
mod wrapper {
    pub struct Duration;
}

#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = crate::humantime_gpt_4_1_nano_20251109_132117::wrapper::Duration;
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic = make_crate_root_not_found_diagnostic(
        &file,
        source,
        "humantime_gpt_4_1_nano_20251109_132117",
        "humantime_gpt_4_1_nano_20251109_132117",
    );

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "wrapper::Duration");
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn missing_crate_head_drops_leading_crate_prefix_for_implicit_core() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
#[cfg(test)]
mod tests {
    use crate::core::clone::Clone;

    #[test]
    fn demo() {
        let _ = 1_i32.clone();
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic = make_missing_crate_diagnostic(&file, source, "core", "core");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { span, new_content } => {
            assert_eq!(new_content, "core::clone::Clone");
            assert_eq!(span.byte_start, source.find("crate::core::clone::Clone").unwrap());
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn missing_crate_head_drops_leading_crate_prefix_for_declared_dependency() {
    let dir = tempdir().expect("tempdir");
    let manifest = r#"[package]
name = "humantime"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = "1.0"
"#;
    let source = r#"
#[cfg(test)]
mod tests {
    use crate::serde::Serialize;

    #[test]
    fn demo() {}
}
"#;
    let file = write_test_crate_layout_with_manifest(dir.path(), manifest, source);
    let diagnostic = make_crate_root_not_found_diagnostic(&file, source, "serde", "serde");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "serde::Serialize");
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn unresolved_head_keeps_head_when_head_is_crate_root_module_for_type_path() {
    let dir = tempdir().expect("tempdir");
    let lib_source = r#"
mod v0;
"#;
    let _lib_file = write_test_crate_layout(dir.path(), lib_source);
    let v0_source = r#"
struct Parser {
    next: usize,
}

#[cfg(test)]
mod tests_rug {
    use super::*;

    #[test]
    fn demo() {
        let _ = v0::Parser { next: 1 };
    }
}
"#;
    write_extra_src_file(dir.path(), "v0.rs", v0_source);
    let v0_file = dir.path().join("src/v0.rs");
    let diagnostic = make_unlinked_crate_diagnostic(&v0_file, v0_source, "v0", "v0");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate::v0::Parser");
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn unresolved_head_keeps_head_when_head_is_crate_root_module_for_ufcs_path() {
    let dir = tempdir().expect("tempdir");
    let lib_source = r#"
mod v0;
"#;
    let _lib_file = write_test_crate_layout(dir.path(), lib_source);
    let v0_source = r#"
pub enum ParseError {
    Invalid,
}

impl ParseError {
    pub fn message(&self) -> &'static str {
        "invalid"
    }
}

#[cfg(test)]
mod tests_rug {
    use super::*;

    #[test]
    fn demo() {
        let p0 = ParseError::Invalid;
        let _ = <v0::ParseError>::message(&p0);
    }
}
"#;
    write_extra_src_file(dir.path(), "v0.rs", v0_source);
    let v0_file = dir.path().join("src/v0.rs");
    let diagnostic = make_unlinked_crate_diagnostic(&v0_file, v0_source, "v0", "v0");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert_eq!(fixes.len(), 1);
    match &fixes[0] {
        FixAction::Replace { new_content, .. } => {
            assert_eq!(new_content, "crate::v0::ParseError");
        }
        _ => panic!("expected Replace fix"),
    }
}

#[test]
fn unresolved_head_skips_when_segments_len_lt_2() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
pub fn format_duration(_v: std::time::Duration) -> String {
    "ok".to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime_core;
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic =
        make_unlinked_crate_diagnostic(&file, source, "humantime_core", "humantime_core");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert!(
        fixes.is_empty(),
        "segments.len() < 2 should not produce rewrite candidate"
    );
}

#[test]
fn unresolved_head_skips_when_dependency_exists() {
    let dir = tempdir().expect("tempdir");
    let manifest = r#"[package]
name = "humantime"
version = "0.1.0"
edition = "2024"

[dependencies]
humantime_core = "1.0"
"#;
    let source = r#"
pub fn format_duration(_v: std::time::Duration) -> String {
    "ok".to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime_core::format_duration(std::time::Duration::new(1, 0));
    }
}
"#;
    let file = write_test_crate_layout_with_manifest(dir.path(), manifest, source);
    let diagnostic =
        make_unlinked_crate_diagnostic(&file, source, "humantime_core", "humantime_core");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert!(
        fixes.is_empty(),
        "existing dependency should block unresolved-head rewrite"
    );
}

#[test]
fn unresolved_head_skips_when_dependency_renamed_package_exists() {
    let dir = tempdir().expect("tempdir");
    let manifest = r#"[package]
name = "humantime"
version = "0.1.0"
edition = "2024"

[dependencies]
ht_core = { package = "humantime_core", version = "1.0" }
"#;
    let source = r#"
pub fn format_duration(_v: std::time::Duration) -> String {
    "ok".to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime_core::format_duration(std::time::Duration::new(1, 0));
    }
}
"#;
    let file = write_test_crate_layout_with_manifest(dir.path(), manifest, source);
    let diagnostic =
        make_unlinked_crate_diagnostic(&file, source, "humantime_core", "humantime_core");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert!(
        fixes.is_empty(),
        "renamed package dependency should also block rewrite"
    );
}

#[test]
fn unresolved_head_skips_when_tail_unreachable() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
pub fn other_symbol() -> String {
    "ok".to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime_core::format_duration(std::time::Duration::new(1, 0));
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    let diagnostic =
        make_unlinked_crate_diagnostic(&file, source, "humantime_core", "humantime_core");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert!(
        fixes.is_empty(),
        "tail symbol not found should degrade to no candidate"
    );
}

#[test]
fn unresolved_head_skips_when_tail_ambiguous() {
    let dir = tempdir().expect("tempdir");
    let source = r#"
mod a;
mod b;

#[cfg(test)]
mod tests {
    #[test]
    fn demo() {
        let _ = humantime_core::format_duration(std::time::Duration::new(1, 0));
    }
}
"#;
    let file = write_test_crate_layout(dir.path(), source);
    write_extra_src_file(
        dir.path(),
        "a.rs",
        r#"
pub fn format_duration(_v: std::time::Duration) -> String {
    "a".to_string()
}
"#,
    );
    write_extra_src_file(
        dir.path(),
        "b.rs",
        r#"
pub fn format_duration(_v: std::time::Duration) -> String {
    "b".to_string()
}
"#,
    );
    let diagnostic =
        make_unlinked_crate_diagnostic(&file, source, "humantime_core", "humantime_core");

    let patcher = E0433Patcher::new();
    let fixes = patcher.analyze(&diagnostic).expect("analyze success");
    assert!(
        fixes.is_empty(),
        "multiple reachable tail definitions should be treated as ambiguous"
    );
}
