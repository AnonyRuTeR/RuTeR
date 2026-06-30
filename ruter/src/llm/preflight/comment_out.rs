use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use ruter::core::TestFunction;
use ruter::core::{FixAction, SpanInfo};

use crate::runtime::function::index::ScopeRange;

/// Build a replace action that comments out either the enclosing test module
/// scope or the target function range as fallback.
pub fn build_comment_out_action(
    crate_path: &Path,
    root_function: &TestFunction,
    scope: Option<&ScopeRange>,
    reason: &str,
) -> Result<FixAction> {
    let selected_scope = scope
        .filter(|candidate| candidate.file_path == root_function.file_path)
        .cloned();

    let root_file_path = selected_scope
        .as_ref()
        .map(|candidate| candidate.file_path.clone())
        .unwrap_or_else(|| crate_path.join(&root_function.relative_file));
    let source = fs::read_to_string(&root_file_path).with_context(|| {
        format!(
            "failed to read root source for preflight comment action: {}",
            root_file_path.display()
        )
    })?;

    // 中文说明：优先使用同测试模块范围；若不存在或不安全，回退为函数范围。
    let (byte_start, byte_end, line_start, line_end) = selected_scope
        .map(|candidate| {
            (
                candidate.byte_start,
                candidate.byte_end,
                candidate.line_start,
                candidate.line_end,
            )
        })
        .unwrap_or((
            root_function.byte_start,
            root_function.byte_end,
            root_function.line_start,
            root_function.line_end,
        ));

    let start = byte_start.min(source.len());
    let end = byte_end.min(source.len());
    if start >= end || !source.is_char_boundary(start) || !source.is_char_boundary(end) {
        anyhow::bail!("invalid root function span for preflight comment action");
    }

    let original = &source[start..end];
    let mut commented_lines = Vec::new();
    for line in original.lines() {
        if line.is_empty() {
            commented_lines.push("//".to_string());
        } else {
            commented_lines.push(format!("// {line}"));
        }
    }
    let mut replacement = format!("// ruter: disabled by preflight interceptor ({reason}).");
    if !commented_lines.is_empty() {
        replacement.push('\n');
        replacement.push_str(&commented_lines.join("\n"));
    }
    replacement.push('\n');

    Ok(FixAction::Replace {
        span: SpanInfo {
            file_path: root_file_path,
            byte_start,
            byte_end,
            line_start,
            line_end,
            col_start: 1,
            col_end: 1,
            is_primary: true,
            text: vec![],
            label: None,
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        },
        new_content: replacement,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn build_comment_out_action_wraps_function_body() {
        let dir = tempdir().expect("tempdir");
        let crate_root = dir.path();
        let file = crate_root.join("tests/basic.rs");
        std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
        let source = "#[test]\nfn case_a() {\n    let _ = 1;\n}\n";
        std::fs::write(&file, source).expect("write source");

        let function = TestFunction {
            id: "tests/basic.rs::tests::case_a:1:4".to_string(),
            relative_file: PathBuf::from("tests/basic.rs"),
            file_path: file.clone(),
            module_path: vec!["tests".to_string()],
            fn_name: "case_a".to_string(),
            byte_start: 0,
            byte_end: source.len(),
            line_start: 1,
            line_end: 4,
        };

        let action = build_comment_out_action(crate_root, &function, None, "E0308 setup hell")
            .expect("action");
        match action {
            FixAction::Replace { new_content, .. } => {
                assert!(new_content.contains("disabled by preflight interceptor"));
                assert!(new_content.contains("// #[test]"));
            }
            _ => panic!("expected replace action"),
        }
    }

    #[test]
    fn build_comment_out_action_prefers_module_scope_when_available() {
        let dir = tempdir().expect("tempdir");
        let crate_root = dir.path();
        let file = crate_root.join("src/lib.rs");
        std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
        let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        let _ = 1;
    }

    #[test]
    fn case_b() {
        let _ = 2;
    }
}
"#;
        std::fs::write(&file, source).expect("write source");

        let fn_start = source.find("fn case_a").expect("case_a");
        let fn_end = source.find("fn case_b").expect("case_b");
        let function = TestFunction {
            id: "src/lib.rs::tests::case_a:4:7".to_string(),
            relative_file: PathBuf::from("src/lib.rs"),
            file_path: file.clone(),
            module_path: vec!["tests".to_string()],
            fn_name: "case_a".to_string(),
            byte_start: fn_start,
            byte_end: fn_end,
            line_start: 4,
            line_end: 7,
        };

        let scope = ScopeRange {
            relative_file: PathBuf::from("src/lib.rs"),
            file_path: file.clone(),
            module_path: vec!["tests".to_string()],
            line_start: 2,
            line_end: 13,
            byte_start: source.find("#[cfg(test)]").expect("scope start"),
            byte_end: source.len(),
        };

        let action = build_comment_out_action(crate_root, &function, Some(&scope), "module")
            .expect("action");
        match action {
            FixAction::Replace { new_content, span } => {
                assert_eq!(span.line_start, 2);
                assert!(new_content.contains("// mod tests {"));
                assert!(new_content.contains("//     fn case_b() {"));
            }
            _ => panic!("expected replace action"),
        }
    }
}
