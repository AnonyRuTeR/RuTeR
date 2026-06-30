use ruter::coordinator::PatchCoordinator;
use ruter::core::{Diagnostic, ErrorCode};
use ruter::parser::JsonParser;
use ruter::patchers::{E0433Patcher, PatcherRegistry};
use ruter::transformer::CodeTransformer;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn create_project(project_dir: &Path) {
    fs::create_dir_all(project_dir.join("src")).expect("create src");

    fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "loop_case"
version = "0.1.0"
edition = "2024"
"#,
    )
    .expect("write Cargo.toml");

    fs::write(
        project_dir.join("src/main.rs"),
        r#"mod foo {
    pub struct State;

    impl State {
        pub fn new() -> Self {
            State
        }
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    #[test]
    fn closure_loop_case() {
        fn main() {
            let _ = State::new();
        }
    }
}
"#,
    )
    .expect("write main.rs");
}

fn run_cargo_check_json(project_dir: &Path) -> std::process::Output {
    Command::new("cargo")
        .arg("check")
        .arg("--tests")
        .arg("--message-format=json")
        .current_dir(project_dir)
        .output()
        .expect("run cargo check")
}

fn extract_diagnostic_json_lines(stdout: &[u8]) -> String {
    let raw = String::from_utf8_lossy(stdout);
    let mut out = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        let message = value.get("message").cloned().unwrap_or(Value::Null);
        let is_diagnostic = message
            .get("$message_type")
            .and_then(|v| v.as_str())
            .map(|v| v == "diagnostic")
            .unwrap_or(false);

        if is_diagnostic {
            out.push(message.to_string());
        }
    }

    out.join("\n")
}

fn normalize_paths(diagnostic: &mut Diagnostic, project_dir: &Path) {
    for span in &mut diagnostic.span {
        if span.file_path.is_relative() {
            span.file_path = project_dir.join(&span.file_path);
        }
    }
    for child in &mut diagnostic.children {
        normalize_paths(child, project_dir);
    }
}

fn contains_e0433(diags: &[Diagnostic]) -> bool {
    diags.iter().any(|d| {
        d.code
            .as_ref()
            .map(|c| c.code == ErrorCode::E0433)
            .unwrap_or(false)
    })
}

#[test]
fn compile_fix_recompile_closure_single_file() {
    let dir = tempdir().expect("tempdir");
    let project_dir = dir.path();
    create_project(project_dir);

    let first = run_cargo_check_json(project_dir);
    assert!(
        !first.status.success(),
        "first compile should fail due to E0433"
    );

    let first_diag_json = extract_diagnostic_json_lines(&first.stdout);
    let mut diagnostics = JsonParser::parse(&first_diag_json).expect("parse first diagnostics");
    for diagnostic in &mut diagnostics {
        normalize_paths(diagnostic, project_dir);
    }
    assert!(
        contains_e0433(&diagnostics),
        "first pass must include E0433"
    );

    let mut registry = PatcherRegistry::new();
    registry.register(Box::new(E0433Patcher::new()));
    let coordinator = PatchCoordinator::new(registry, CodeTransformer::new());

    let plan = coordinator.plan(&diagnostics).expect("build plan");
    assert!(!plan.is_empty(), "plan should contain replacements");

    let main_file = PathBuf::from(project_dir.join("src/main.rs"));
    let before = fs::read_to_string(&main_file).expect("read before");

    let mut sources = HashMap::new();
    sources.insert(main_file.clone(), before.clone());

    let updated = coordinator
        .apply_planned(&plan, &sources)
        .expect("apply planned");

    let after = updated
        .get(&main_file)
        .expect("updated main.rs should exist")
        .clone();
    assert_ne!(before, after, "source must change after fix application");
    fs::write(&main_file, &after).expect("write fixed file");

    let second = run_cargo_check_json(project_dir);
    assert!(
        second.status.success(),
        "second compile should succeed after applying fix"
    );

    let second_diag_json = extract_diagnostic_json_lines(&second.stdout);
    let second_diags = JsonParser::parse(&second_diag_json).unwrap_or_default();
    assert!(
        !contains_e0433(&second_diags),
        "second pass must not include E0433"
    );
}
