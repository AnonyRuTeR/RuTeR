use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ruter")
}

fn create_project(project_dir: &Path) {
    fs::create_dir_all(project_dir.join("src")).expect("create src");

    fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "cli_case"
version = "0.1.0"
edition = "2024"
"#,
    )
    .expect("write cargo toml");

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
    fn cli_flow_case() {
        fn main() {
            let _ = State::new();
        }
    }
}
"#,
    )
    .expect("write main");
}

fn create_non_test_project(project_dir: &Path) {
    fs::create_dir_all(project_dir.join("src")).expect("create src");

    fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "cli_case_non_test"
version = "0.1.0"
edition = "2024"
"#,
    )
    .expect("write cargo toml");

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

fn main() {
    let _ = State::new();
}
"#,
    )
    .expect("write main");
}

fn create_five_function_project(project_dir: &Path) {
    fs::create_dir_all(project_dir.join("src")).expect("create src");

    fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "cli_case_five"
version = "0.1.0"
edition = "2024"
"#,
    )
    .expect("write cargo toml");

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
    fn case_a() { let _ = State::new(); }
    #[test]
    fn case_b() { let _ = State::new(); }
    #[test]
    fn case_c() { let _ = State::new(); }
    #[test]
    fn case_d() { let _ = State::new(); }
    #[test]
    fn case_e() { let _ = State::new(); }
}
"#,
    )
    .expect("write main");
}

fn add_assert_to_case_e(project_dir: &Path) {
    let path = project_dir.join("src/main.rs");
    let source = fs::read_to_string(&path).expect("read source");
    let updated = source.replace(
        "fn case_e() { let _ = State::new(); }",
        "fn case_e() { assert_eq!(1, 1); let _ = State::new(); }",
    );
    fs::write(path, updated).expect("write source");
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("run ruter")
}

fn assert_ok(output: &Output) {
    assert!(
        output.status.success(),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn mutate_first_replace_action(action: &mut Value, suffix: &str) -> bool {
    if let Some(replace) = action.get_mut("Replace")
        && let Some(new_content) = replace.get_mut("new_content")
        && let Some(text) = new_content.as_str()
    {
        *new_content = Value::String(format!("{text}{suffix}"));
        return true;
    }
    false
}

fn count_occurrences(content: &str, needle: &str) -> usize {
    content.matches(needle).count()
}

#[test]
fn dry_run_keeps_source_and_writes_diff() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let before = fs::read_to_string(crate_path.join("src/main.rs")).expect("read before");

    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
        "--keep-updated-sources",
    ]);
    assert_ok(&output);

    let after = fs::read_to_string(crate_path.join("src/main.rs")).expect("read after");
    assert_eq!(before, after, "dry-run should not mutate source");

    let diff_path = artifacts.join("5_changes.diff");
    assert!(diff_path.exists(), "diff should be generated in dry-run");
    let diff = fs::read_to_string(diff_path).expect("read diff");
    assert!(diff.contains("crate::foo::State"));

    let updated_file = if artifacts.join("5_updated/src/main_updated.rs").exists() {
        artifacts.join("5_updated/src/main_updated.rs")
    } else if artifacts
        .join("5_updated/partial_union/src/main_updated.rs")
        .exists()
    {
        artifacts.join("5_updated/partial_union/src/main_updated.rs")
    } else {
        artifacts.join("5_updated/candidate-1/src/main_updated.rs")
    };
    assert!(
        updated_file.exists(),
        "dry-run should persist updated source snapshots"
    );
    let updated_content = fs::read_to_string(updated_file).expect("read updated snapshot");
    assert!(updated_content.contains("crate::foo::State::new"));
}

#[test]
fn dry_run_does_not_persist_updated_snapshots_by_default() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_ok(&output);

    assert!(
        !artifacts.join("5_updated").exists(),
        "updated snapshots should be disabled by default"
    );
}

#[test]
fn apply_writes_source_and_backup() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--apply",
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_ok(&output);

    let updated = fs::read_to_string(crate_path.join("src/main.rs")).expect("read updated");
    assert!(updated.contains("crate::foo::State::new"));

    let backup = artifacts.join("5_backups/src/main.bak");
    assert!(backup.exists(), "backup should exist by default");

    let diff_path = artifacts.join("5_changes.diff");
    assert!(diff_path.exists());
}

#[test]
fn step_plan_without_analyze_artifact_returns_5() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let output = run(&[
        "step",
        "plan",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);

    assert_eq!(output.status.code(), Some(5));
}

#[test]
fn step_full_chain_succeeds() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let artifacts_str = artifacts.to_str().unwrap();
    let crate_str = crate_path.to_str().unwrap();

    assert_ok(&run(&[
        "step",
        "compile",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "analyze",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "plan",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "verify",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "apply",
        crate_str,
        "--apply",
        "--artifacts-dir",
        artifacts_str,
    ]));
}

#[test]
fn log_file_contains_stage_markers() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let log_file = dir.path().join("run.log");

    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
        "--log-file",
        log_file.to_str().unwrap(),
    ]);
    assert_ok(&output);

    let log = fs::read_to_string(log_file).expect("read log");
    let compile_idx = log.find("=== STAGE: COMPILE ===").expect("compile stage");
    let analyze_idx = log.find("=== STAGE: ANALYZE ===").expect("analyze stage");
    let plan_idx = log.find("=== STAGE: PLAN ===").expect("plan stage");
    let verify_idx = log.find("=== STAGE: VERIFY ===").expect("verify stage");
    let apply_idx = log.find("=== STAGE: APPLY ===").expect("apply stage");
    let summarize_idx = log
        .find("=== STAGE: SUMMARIZE ===")
        .expect("summarize stage");

    assert!(compile_idx < analyze_idx);
    assert!(analyze_idx < plan_idx);
    assert!(plan_idx < verify_idx);
    assert!(verify_idx < apply_idx);
    assert!(apply_idx < summarize_idx);
}

#[test]
fn pre_apply_verify_failure_does_not_write_files() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let artifacts_str = artifacts.to_str().unwrap();
    let crate_str = crate_path.to_str().unwrap();

    assert_ok(&run(&[
        "step",
        "compile",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "analyze",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "plan",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));

    let _ = fs::remove_file(artifacts.join("3_plan_candidates.json"));
    let plan_path = artifacts.join("3_plan.json");
    let original_plan = fs::read_to_string(&plan_path).expect("read plan");
    let broken_plan = original_plan.replace("crate::foo::State::new", "crate::foo::State::new(");
    fs::write(&plan_path, broken_plan).expect("write broken plan");

    let before = fs::read_to_string(crate_path.join("src/main.rs")).expect("read before apply");
    let output = run(&[
        "step",
        "apply",
        crate_str,
        "--apply",
        "--artifacts-dir",
        artifacts_str,
    ]);

    assert_eq!(output.status.code(), Some(7));
    let after = fs::read_to_string(crate_path.join("src/main.rs")).expect("read after apply");
    assert_eq!(
        before, after,
        "file should stay unchanged when preflight fails"
    );
}

#[test]
fn dry_run_verify_failure_returns_4_and_does_not_write_files() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let artifacts_str = artifacts.to_str().unwrap();
    let crate_str = crate_path.to_str().unwrap();

    assert_ok(&run(&[
        "step",
        "compile",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "analyze",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "plan",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));

    let _ = fs::remove_file(artifacts.join("3_plan_candidates.json"));
    let plan_path = artifacts.join("3_plan.json");
    let original_plan = fs::read_to_string(&plan_path).expect("read plan");
    let broken_plan = original_plan.replace("crate::foo::State::new", "crate::foo::State::new(");
    fs::write(&plan_path, broken_plan).expect("write broken plan");

    let before = fs::read_to_string(crate_path.join("src/main.rs")).expect("read before apply");
    let output = run(&["step", "apply", crate_str, "--artifacts-dir", artifacts_str]);

    assert_eq!(output.status.code(), Some(7));
    let after = fs::read_to_string(crate_path.join("src/main.rs")).expect("read after apply");
    assert_eq!(
        before, after,
        "file should stay unchanged when dry-run patch verification fails"
    );
}

#[test]
fn non_test_context_logs_skip_reason_and_returns_7() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_non_test_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let log_file = dir.path().join("run.log");

    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
        "--log-file",
        log_file.to_str().unwrap(),
    ]);

    assert_eq!(output.status.code(), Some(7));
    let log = fs::read_to_string(log_file).expect("read log");
    assert!(log.contains("diagnostic_count:"));
    assert!(log.contains("exit_code: 7"));
}

#[test]
fn summary_no_plan_remaining_equals_initial() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_non_test_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(7));

    let summary_raw = fs::read_to_string(artifacts.join("6_summary.json")).expect("read summary");
    let summary: Value = serde_json::from_str(&summary_raw).expect("parse summary");

    let initial = summary
        .get("initial_error_total")
        .and_then(|v| v.as_u64())
        .expect("initial_error_total");
    let remaining = summary
        .get("remaining_error_total")
        .and_then(|v| v.as_u64())
        .expect("remaining_error_total");

    assert_eq!(initial, remaining);
    assert!(initial >= 1);
}

#[test]
fn summary_verify_success_remaining_zero() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_ok(&output);

    let summary_raw = fs::read_to_string(artifacts.join("6_summary.json")).expect("read summary");
    let summary: Value = serde_json::from_str(&summary_raw).expect("parse summary");

    let initial = summary
        .get("initial_error_total")
        .and_then(|v| v.as_u64())
        .expect("initial_error_total");
    let remaining = summary
        .get("remaining_error_total")
        .and_then(|v| v.as_u64())
        .expect("remaining_error_total");

    assert!(initial >= 1);
    assert_eq!(remaining, 0);
}

#[test]
fn topk_first_candidate_fails_second_candidate_passes() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let artifacts_str = artifacts.to_str().unwrap();
    let crate_str = crate_path.to_str().unwrap();

    assert_ok(&run(&[
        "step",
        "compile",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "analyze",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "plan",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));

    let candidates_path = artifacts.join("3_plan_candidates.json");
    let mut candidates: Value =
        serde_json::from_str(&fs::read_to_string(&candidates_path).expect("read candidates"))
            .expect("parse candidates");
    let arr = candidates.as_array_mut().expect("candidate array");
    assert!(!arr.is_empty(), "plan candidates should not be empty");
    if arr.len() == 1 {
        let mut cloned = arr[0].clone();
        cloned["candidate_id"] = Value::String("candidate-2".to_string());
        arr.push(cloned);
    }
    arr[0]["candidate_id"] = Value::String("candidate-1".to_string());
    arr[1]["candidate_id"] = Value::String("candidate-2".to_string());

    let files = arr[0]["plan"].as_object_mut().expect("plan map");
    let mut modified = false;
    for actions in files.values_mut() {
        if let Some(actions_arr) = actions.as_array_mut() {
            for action in actions_arr {
                if mutate_first_replace_action(action, "(") {
                    modified = true;
                    break;
                }
            }
        }
        if modified {
            break;
        }
    }
    assert!(modified, "should mutate first candidate action");
    fs::write(
        &candidates_path,
        serde_json::to_string_pretty(&candidates).expect("serialize candidates"),
    )
    .expect("write candidates");

    let output = run(&[
        "step",
        "apply",
        crate_str,
        "--apply",
        "--artifacts-dir",
        artifacts_str,
    ]);
    assert_ok(&output);

    let verify_report: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("verify/4_report.json")).unwrap())
            .unwrap();
    assert_eq!(
        verify_report
            .get("selected_candidate_id")
            .and_then(|v| v.as_str()),
        Some("candidate-2")
    );
}

#[test]
fn topk_exhausted_returns_7_and_writes_attempt_evidence() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let artifacts_str = artifacts.to_str().unwrap();
    let crate_str = crate_path.to_str().unwrap();

    assert_ok(&run(&[
        "step",
        "compile",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "analyze",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "plan",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));

    let candidates_path = artifacts.join("3_plan_candidates.json");
    let mut candidates: Value =
        serde_json::from_str(&fs::read_to_string(&candidates_path).expect("read candidates"))
            .expect("parse candidates");
    let arr = candidates.as_array_mut().expect("candidate array");
    assert!(!arr.is_empty(), "plan candidates should not be empty");
    if arr.len() == 1 {
        let mut cloned = arr[0].clone();
        cloned["candidate_id"] = Value::String("candidate-2".to_string());
        arr.push(cloned);
    }
    for (idx, candidate) in arr.iter_mut().enumerate() {
        candidate["candidate_id"] = Value::String(format!("candidate-{}", idx + 1));
        let files = candidate["plan"].as_object_mut().expect("plan map");
        let mut modified = false;
        for actions in files.values_mut() {
            if let Some(actions_arr) = actions.as_array_mut() {
                for action in actions_arr {
                    if mutate_first_replace_action(action, "(") {
                        modified = true;
                        break;
                    }
                }
            }
            if modified {
                break;
            }
        }
        assert!(modified, "should mutate candidate action");
    }
    fs::write(
        &candidates_path,
        serde_json::to_string_pretty(&candidates).expect("serialize candidates"),
    )
    .expect("write candidates");

    let output = run(&[
        "step",
        "verify",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]);
    assert_eq!(output.status.code(), Some(7));

    let verify_report: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("verify/4_report.json")).unwrap())
            .unwrap();
    assert_eq!(
        verify_report
            .get("selected_candidate_id")
            .and_then(|v| v.as_str()),
        Some("partial-union")
    );
    let attempts = verify_report
        .get("attempts")
        .and_then(|v| v.as_array())
        .expect("attempts");
    assert!(attempts.len() >= 2);
}

#[test]
fn topk_union_improves_from_four_to_five_and_returns_0() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_five_function_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let artifacts_str = artifacts.to_str().unwrap();
    let crate_str = crate_path.to_str().unwrap();

    assert_ok(&run(&[
        "step",
        "compile",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "analyze",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "plan",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));

    let candidates_path = artifacts.join("3_plan_candidates.json");
    let mut candidates: Value =
        serde_json::from_str(&fs::read_to_string(&candidates_path).expect("read candidates"))
            .expect("parse candidates");
    let arr = candidates.as_array_mut().expect("candidate array");
    assert!(!arr.is_empty(), "plan candidates should not be empty");
    while arr.len() < 3 {
        let mut cloned = arr[0].clone();
        cloned["candidate_id"] = Value::String(format!("candidate-{}", arr.len() + 1));
        arr.push(cloned);
    }
    for (idx, missing_idx) in [4usize, 3usize, 2usize].iter().enumerate() {
        let candidate = arr.get_mut(idx).expect("candidate exists");
        candidate["candidate_id"] = Value::String(format!("candidate-{}", idx + 1));
        let plan_map = candidate
            .get_mut("plan")
            .and_then(Value::as_object_mut)
            .expect("plan map");
        let actions = plan_map
            .values_mut()
            .next()
            .and_then(Value::as_array_mut)
            .expect("actions");
        assert!(actions.len() >= 5, "expected at least 5 actions");
        actions.remove(*missing_idx);
    }
    fs::write(
        &candidates_path,
        serde_json::to_string_pretty(&candidates).expect("serialize candidates"),
    )
    .expect("write candidates");

    let output = run(&[
        "step",
        "verify",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(
            "candidate-1 independent_resolved_test_function_count=4 independent_unresolved_test_function_count=1"
        ),
        "stdout={stdout}"
    );

    let verify_report: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("verify/4_report.json")).unwrap())
            .unwrap();
    assert_eq!(
        verify_report
            .get("selected_candidate_id")
            .and_then(|v| v.as_str()),
        Some("partial-union")
    );
    assert_eq!(
        verify_report
            .get("resolved_test_function_count")
            .and_then(|v| v.as_u64()),
        Some(5)
    );
    assert_eq!(
        verify_report
            .get("unresolved_test_function_count")
            .and_then(|v| v.as_u64()),
        Some(0)
    );
}

#[test]
fn topk_union_keeps_partial_and_handoffs_single_function() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_five_function_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let artifacts_str = artifacts.to_str().unwrap();
    let crate_str = crate_path.to_str().unwrap();

    assert_ok(&run(&[
        "step",
        "compile",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "analyze",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "plan",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));

    let candidates_path = artifacts.join("3_plan_candidates.json");
    let mut candidates: Value =
        serde_json::from_str(&fs::read_to_string(&candidates_path).expect("read candidates"))
            .expect("parse candidates");
    let arr = candidates.as_array_mut().expect("candidate array");
    assert!(!arr.is_empty(), "plan candidates should not be empty");
    while arr.len() < 3 {
        let mut cloned = arr[0].clone();
        cloned["candidate_id"] = Value::String(format!("candidate-{}", arr.len() + 1));
        arr.push(cloned);
    }
    for (idx, candidate) in arr.iter_mut().take(3).enumerate() {
        candidate["candidate_id"] = Value::String(format!("candidate-{}", idx + 1));
        let plan_map = candidate
            .get_mut("plan")
            .and_then(Value::as_object_mut)
            .expect("plan map");
        let actions = plan_map
            .values_mut()
            .next()
            .and_then(Value::as_array_mut)
            .expect("actions");
        assert!(actions.len() >= 5, "expected at least 5 actions");
        actions.remove(4);
    }
    fs::write(
        &candidates_path,
        serde_json::to_string_pretty(&candidates).expect("serialize candidates"),
    )
    .expect("write candidates");

    let output = run(&[
        "step",
        "apply",
        crate_str,
        "--apply",
        "--artifacts-dir",
        artifacts_str,
    ]);
    assert_eq!(output.status.code(), Some(7));

    let source = fs::read_to_string(crate_path.join("src/main.rs")).expect("read source");
    assert_eq!(
        count_occurrences(&source, "crate::foo::State::new"),
        4,
        "four functions should keep local repair"
    );
    assert_eq!(
        count_occurrences(&source, "let _ = State::new();"),
        1,
        "one function should remain for llm handoff"
    );

    let llm_handoff: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_handoff.json")).unwrap())
            .unwrap();
    let items = llm_handoff.as_array().expect("llm handoff array");
    assert_eq!(items.len(), 1);
}

#[test]
fn cli_dispatch_report_observable() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_ok(&output);

    let raw = fs::read_to_string(artifacts.join("3_function_dispatch_report.json"))
        .expect("read function dispatch report");
    let report: Value = serde_json::from_str(&raw).expect("parse report");
    let arr = report.as_array().expect("array");
    assert!(!arr.is_empty(), "dispatch report should not be empty");
    assert_eq!(
        arr[0].get("decision").and_then(|v| v.as_str()),
        Some("RulePatcher")
    );
}

#[test]
fn cli_verify_report_contains_optimistic_round_fields() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    let output = run(&[
        "fix",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_ok(&output);

    let raw =
        fs::read_to_string(artifacts.join("verify/4_report.json")).expect("read verify report");
    let verify: Value = serde_json::from_str(&raw).expect("parse verify report");
    assert!(verify.get("optimistic_round_count").is_some());
    assert!(verify.get("dispatch_target_function_count").is_some());
    assert!(verify.get("rule_function_count").is_some());
    assert!(verify.get("llm_routed_function_count").is_some());
    assert!(verify.get("independence_broken_function_ids").is_some());
}

fn prepare_single_unresolved_partial_plan(crate_path: &Path, artifacts: &Path) {
    let artifacts_str = artifacts.to_str().unwrap();
    let crate_str = crate_path.to_str().unwrap();

    assert_ok(&run(&[
        "step",
        "compile",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "analyze",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));
    assert_ok(&run(&[
        "step",
        "plan",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]));

    let candidates_path = artifacts.join("3_plan_candidates.json");
    let mut candidates: Value =
        serde_json::from_str(&fs::read_to_string(&candidates_path).expect("read candidates"))
            .expect("parse candidates");
    let arr = candidates.as_array_mut().expect("candidate array");
    assert!(!arr.is_empty(), "plan candidates should not be empty");
    while arr.len() < 3 {
        let mut cloned = arr[0].clone();
        cloned["candidate_id"] = Value::String(format!("candidate-{}", arr.len() + 1));
        arr.push(cloned);
    }
    for (idx, candidate) in arr.iter_mut().take(3).enumerate() {
        candidate["candidate_id"] = Value::String(format!("candidate-{}", idx + 1));
        let plan_map = candidate
            .get_mut("plan")
            .and_then(Value::as_object_mut)
            .expect("plan map");
        let actions = plan_map
            .values_mut()
            .next()
            .and_then(Value::as_array_mut)
            .expect("actions");
        assert!(actions.len() >= 5, "expected at least 5 actions");
        actions.remove(4);
    }
    fs::write(
        &candidates_path,
        serde_json::to_string_pretty(&candidates).expect("serialize candidates"),
    )
    .expect("write candidates");

    let verify_output = run(&[
        "step",
        "verify",
        crate_str,
        "--artifacts-dir",
        artifacts_str,
    ]);
    assert_eq!(
        verify_output.status.code(),
        Some(7),
        "status={:?}\nstdout={}\nstderr={}",
        verify_output.status.code(),
        String::from_utf8_lossy(&verify_output.stdout),
        String::from_utf8_lossy(&verify_output.stderr)
    );
}

#[test]
fn llm_flags_require_pair() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_project(&crate_path);
    let artifacts = dir.path().join("artifacts");
    let crate_str = crate_path.to_str().unwrap();
    let artifacts_str = artifacts.to_str().unwrap();

    let missing_replay = run(&[
        "fix",
        crate_str,
        "--enable-llm",
        "--artifacts-dir",
        artifacts_str,
    ]);
    assert_eq!(missing_replay.status.code(), Some(5));

    let replay_file = dir.path().join("replay.json");
    fs::write(&replay_file, "{\"schema_version\":\"1\",\"functions\":[]}").expect("write replay");
    let missing_flag = run(&[
        "fix",
        crate_str,
        "--llm-replay-file",
        replay_file.to_str().unwrap(),
        "--artifacts-dir",
        artifacts_str,
    ]);
    assert_eq!(missing_flag.status.code(), Some(5));
}

#[test]
fn step_verify_with_existing_partial_plan_keeps_source_and_partial_union_report() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_five_function_project(&crate_path);
    let artifacts = dir.path().join("artifacts");
    prepare_single_unresolved_partial_plan(&crate_path, &artifacts);

    let before = fs::read_to_string(crate_path.join("src/main.rs")).expect("read before");
    let output = run(&[
        "step",
        "verify",
        crate_path.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(7));

    let after = fs::read_to_string(crate_path.join("src/main.rs")).expect("read after");
    assert_eq!(before, after, "verify stage should not mutate source");

    let verify_report: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("verify/4_report.json")).unwrap())
            .unwrap();
    assert_eq!(
        verify_report
            .get("selected_candidate_id")
            .and_then(|v| v.as_str()),
        Some("partial-union")
    );
}

#[test]
fn llm_replay_patched_function_text_can_close_pending_function() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_five_function_project(&crate_path);
    add_assert_to_case_e(&crate_path);
    let artifacts = dir.path().join("artifacts");
    prepare_single_unresolved_partial_plan(&crate_path, &artifacts);

    let llm_handoff: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_handoff.json")).unwrap())
            .unwrap();
    let items = llm_handoff.as_array().expect("llm handoff array");
    assert_eq!(items.len(), 1);
    let function_id = items[0]
        .get("function_id")
        .and_then(Value::as_str)
        .expect("function id");
    let fn_name = items[0]
        .get("fn_name")
        .and_then(Value::as_str)
        .expect("fn_name");

    let patched_function_text =
        format!("#[test]\nfn {fn_name}() {{ let _ = crate::foo::State::new(); }}\n");
    let replay = serde_json::json!({
        "schema_version": "1",
        "functions": [
            {
                "function_id": function_id,
                "rounds": [
                    {
                        "round": 1,
                        "candidates": [
                            {
                                "candidate_id": "replay-1",
                                "patched_function_text": patched_function_text
                            }
                        ]
                    }
                ]
            }
        ]
    });
    let replay_file = dir.path().join("replay.json");
    fs::write(&replay_file, serde_json::to_string_pretty(&replay).unwrap()).unwrap();

    let output = run(&[
        "step",
        "apply",
        crate_path.to_str().unwrap(),
        "--apply",
        "--enable-llm",
        "--llm-replay-file",
        replay_file.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0));

    let source = fs::read_to_string(crate_path.join("src/main.rs")).expect("read source");
    assert_eq!(count_occurrences(&source, "crate::foo::State::new"), 5);

    let attempts: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_attempts.json")).unwrap())
            .unwrap();
    let arr = attempts
        .get("attempts")
        .and_then(Value::as_array)
        .expect("attempt array");
    assert!(!arr.is_empty(), "llm attempts should not be empty");
    assert!(
        arr.iter()
            .any(|item| item.get("accepted").and_then(Value::as_bool) == Some(true)),
        "at least one llm candidate should be accepted"
    );
}

#[test]
fn llm_replay_empty_candidates_low_value_does_not_force_skip_without_code_match() {
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_five_function_project(&crate_path);
    let artifacts = dir.path().join("artifacts");
    prepare_single_unresolved_partial_plan(&crate_path, &artifacts);

    let llm_handoff: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_handoff.json")).unwrap())
            .unwrap();
    let items = llm_handoff.as_array().expect("llm handoff array");
    assert_eq!(items.len(), 1);
    let function_id = items[0]
        .get("function_id")
        .and_then(Value::as_str)
        .expect("function id");

    let replay = serde_json::json!({
        "schema_version": "1",
        "functions": [
            {
                "function_id": function_id,
                "rounds": [
                    {
                        "round": 1,
                        "candidates": []
                    }
                ]
            }
        ]
    });
    let replay_file = dir.path().join("replay_empty.json");
    fs::write(&replay_file, serde_json::to_string_pretty(&replay).unwrap()).unwrap();

    let output = run(&[
        "step",
        "apply",
        crate_path.to_str().unwrap(),
        "--apply",
        "--enable-llm",
        "--llm-replay-file",
        replay_file.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(7));

    let attempts: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_attempts.json")).unwrap())
            .unwrap();
    let arr = attempts
        .get("attempts")
        .and_then(Value::as_array)
        .expect("attempt array");
    assert!(
        arr.iter().any(|item| {
            item.get("phase").and_then(Value::as_str) == Some("preflight_decision")
                && item.get("accepted").and_then(Value::as_bool) == Some(true)
        }),
        "preflight decision should be observable"
    );
    assert!(
        arr.iter().any(|item| {
            item.get("phase").and_then(Value::as_str) == Some("budget")
                && item.get("failure_kind").and_then(Value::as_str) == Some("LLM_BUDGET_EXHAUSTED")
        }),
        "budget exhaustion should be recorded when replay candidates are empty"
    );

    let source = fs::read_to_string(crate_path.join("src/main.rs")).expect("read source");
    assert!(
        !source.contains("disabled by preflight interceptor"),
        "function should not be commented out by low-value-only signal"
    );
}
