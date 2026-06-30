use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::tempdir;
use walkdir::WalkDir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ruter")
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("run ruter")
}

fn copy_tree(src: &Path, dst: &Path) {
    for entry in WalkDir::new(src) {
        let entry = entry.expect("walkdir entry");
        let path = entry.path();
        let rel = path.strip_prefix(src).expect("relative path");
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).expect("create dir");
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::copy(path, &target).expect("copy file");
        }
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn fixture_gemini_crate_fix_flow_smoke_regression() {
    let root = workspace_root();
    let fixture_src = root.join("tests/humantime-e0433-gemini-fixture");
    assert!(
        fixture_src.join("Cargo.toml").exists(),
        "fixture crate must exist"
    );

    let dir = tempdir().expect("tempdir");
    let fixture_copy = dir.path().join("fixture");
    copy_tree(&fixture_src, &fixture_copy);
    let artifacts = dir.path().join("artifacts");

    let output = run(&[
        "fix",
        fixture_copy.to_str().unwrap(),
        "--artifacts-dir",
        artifacts.to_str().unwrap(),
        "--topk",
        "3",
    ]);

    assert!(
        matches!(output.status.code(), Some(0) | Some(7)),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let must_exist = [
        "1_compile_diagnostics.json",
        "2_analyzed_diagnostics.json",
        "3_function_dispatch_report.json",
        "verify/4_report.json",
        "6_summary.json",
    ];
    for rel in must_exist {
        let path = artifacts.join(rel);
        assert!(path.exists(), "artifact should exist: {}", path.display());
    }

    let summary: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("6_summary.json")).unwrap())
            .unwrap();
    assert!(
        summary
            .get("diagnostic_count")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            > 0,
        "fixture run should observe diagnostics"
    );
}
