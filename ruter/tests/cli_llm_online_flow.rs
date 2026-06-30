use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use serde_json::Value;
use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ruter")
}

fn create_five_function_project(project_dir: &Path) {
    fs::create_dir_all(project_dir.join("src")).expect("create src");

    fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "cli_case_online"
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
    fn case_a() {
        assert_eq!(1, 1);
        let _ = State::new();
    }
    #[test]
    fn case_b() {
        assert_eq!(1, 1);
        let _ = State::new();
    }
    #[test]
    fn case_c() {
        assert_eq!(1, 1);
        let _ = State::new();
    }
    #[test]
    fn case_d() {
        assert_eq!(1, 1);
        let _ = State::new();
    }
    #[test]
    fn case_e() {
        assert_eq!(1, 1);
        let _ = State::new();
    }
}
"#,
    )
    .expect("write main");
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("run ruter")
}

fn online_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(bin());
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.output().expect("run ruter with env")
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

fn inject_rule_drift_fixture(crate_path: &Path, artifacts: &Path) {
    let handoff: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_handoff.json")).unwrap())
            .expect("parse llm handoff");
    let items = handoff.as_array().expect("handoff array");
    let item = items.first().expect("handoff should contain one item");
    let function_id = item
        .get("function_id")
        .and_then(Value::as_str)
        .expect("function_id")
        .to_string();
    let fn_name = item
        .get("fn_name")
        .and_then(Value::as_str)
        .expect("fn_name")
        .to_string();

    let source_path = crate_path.join("src/main.rs");
    let mut source = fs::read_to_string(&source_path).expect("read source");
    let old_block = format!(
        "fn {fn_name}() {{\n        assert_eq!(1, 1);\n        let _ = State::new();\n    }}"
    );
    let new_block = format!(
        "fn {fn_name}() {{\n        assert_eq!(1, 1);\n        let _ = crate::foo::State::missing();\n    }}"
    );
    assert!(
        source.contains(&old_block),
        "target function block should exist before injecting drift"
    );
    source = source.replacen(&old_block, &new_block, 1);
    fs::write(&source_path, source).expect("write source");

    let mut dispatch: Value = serde_json::from_str(
        &fs::read_to_string(artifacts.join("3_function_dispatch_report.json"))
            .expect("read dispatch report"),
    )
    .expect("parse dispatch report");
    let dispatch_arr = dispatch.as_array_mut().expect("dispatch report array");
    for entry in dispatch_arr.iter_mut() {
        if entry.get("function_id").and_then(Value::as_str) == Some(function_id.as_str()) {
            entry["decision"] = Value::String("RulePatcher".to_string());
            entry["error_code_counts"] =
                serde_json::json!({ "E0433": 1u64, "E0599": 0u64, "E0308": 0u64 });
        }
    }
    fs::write(
        artifacts.join("3_function_dispatch_report.json"),
        serde_json::to_string_pretty(&dispatch).expect("serialize dispatch"),
    )
    .expect("write dispatch report");

    let mut selected_rank_by_function = serde_json::Map::new();
    selected_rank_by_function.insert(function_id.clone(), Value::from(1u64));
    let rounds = Value::Array(vec![Value::Object(
        [
            ("round".to_string(), Value::from(1u64)),
            (
                "selected_rank_by_function".to_string(),
                Value::Object(selected_rank_by_function),
            ),
            ("plan_file_count".to_string(), Value::from(1u64)),
            ("plan_action_count".to_string(), Value::from(1u64)),
            ("check_error_total".to_string(), Value::from(1u64)),
            (
                "check_error_by_code".to_string(),
                serde_json::json!({ "E0433": 1u64 }),
            ),
            ("resolved_function_ids".to_string(), Value::Array(vec![])),
            (
                "unresolved_function_ids".to_string(),
                Value::Array(vec![Value::from(function_id)]),
            ),
            (
                "independence_broken_function_ids".to_string(),
                Value::Array(vec![]),
            ),
        ]
        .into_iter()
        .collect(),
    )]);
    fs::create_dir_all(artifacts.join("verify")).expect("create verify dir");
    fs::write(
        artifacts.join("verify/4_function_verify_rounds.json"),
        serde_json::to_string_pretty(&rounds).expect("serialize rounds"),
    )
    .expect("write verify rounds");
}

#[derive(Clone, Copy)]
enum MockBehavior {
    Status(u16),
    Timeout,
    RustFencedFixFromPrompt,
    FailThenRecoverWithHistory,
}

struct MockServer {
    api_url: String,
    handle: thread::JoinHandle<()>,
}

fn status_text(code: u16) -> &'static str {
    match code {
        401 => "Unauthorized",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Error",
    }
}

fn spawn_mock_server(behavior: MockBehavior) -> Option<MockServer> {
    let listener = (0..20).find_map(|_| match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => Some(listener),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            thread::sleep(Duration::from_millis(20));
            None
        }
        Err(err) => panic!("bind mock listener failed: {err}"),
    })?;
    let addr = listener.local_addr().expect("local addr");
    let request_count = Arc::new(AtomicUsize::new(0));
    let request_count_for_thread = Arc::clone(&request_count);
    let handle = thread::spawn(move || {
        let max_requests = match behavior {
            MockBehavior::FailThenRecoverWithHistory => 2,
            _ => 1,
        };

        for _ in 0..max_requests {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let req_text = read_http_request(&mut stream);
            let request_no = request_count_for_thread.fetch_add(1, Ordering::SeqCst) + 1;

            match behavior {
                MockBehavior::Status(code) => {
                    respond_http_status(
                        &mut stream,
                        code,
                        &format!("{{\"error\":\"mock-{code}\"}}"),
                    );
                }
                MockBehavior::Timeout => {
                    thread::sleep(Duration::from_secs(3));
                }
                MockBehavior::RustFencedFixFromPrompt => {
                    let user_prompt = extract_user_prompt_from_http_request(&req_text);
                    let fn_name = user_prompt
                        .as_deref()
                        .and_then(extract_location_fn_name)
                        .unwrap_or("case_e");
                    let patched =
                        format!("#[test]\nfn {fn_name}() {{ let _ = crate::foo::State::new(); }}");
                    let content = format!("```rust\n{}\n```", patched.trim_end());
                    respond_chat_completion_content(&mut stream, &content);
                }
                MockBehavior::FailThenRecoverWithHistory => {
                    let user_prompt =
                        extract_user_prompt_from_http_request(&req_text).unwrap_or_default();
                    if request_no == 1 {
                        // 首轮故意返回可解析候选但不可归一化的函数文本，触发 LLM_OUTPUT_INVALID_SCHEMA。
                        let invalid_candidate = r#"{"candidates":[{"candidate_id":"c1","patched_function_text":"not a rust function"}]}"#;
                        respond_chat_completion_content(&mut stream, invalid_candidate);
                    } else {
                        let has_history_header =
                            user_prompt.contains("Previous round failures to avoid repeating");
                        let has_failure_kind = user_prompt.contains("LLM_OUTPUT_INVALID_SCHEMA");
                        if has_history_header && has_failure_kind {
                            let fn_name =
                                extract_location_fn_name(&user_prompt).unwrap_or("case_e");
                            let patched = format!(
                                "#[test]\nfn {fn_name}() {{ let _ = crate::foo::State::new(); }}"
                            );
                            let content = format!("```rust\n{}\n```", patched.trim_end());
                            respond_chat_completion_content(&mut stream, &content);
                        } else {
                            let invalid_candidate = r#"{"candidates":[{"candidate_id":"c1","patched_function_text":"still invalid"}]}"#;
                            respond_chat_completion_content(&mut stream, invalid_candidate);
                        }
                    }
                }
            }
        }
    });

    Some(MockServer {
        api_url: format!("http://{addr}/v1"),
        handle,
    })
}

fn respond_http_status(stream: &mut TcpStream, code: u16, body: &str) {
    let response = format!(
        "HTTP/1.1 {code} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_text(code),
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn respond_chat_completion_content(stream: &mut TcpStream, content: &str) {
    let body = serde_json::json!({
        "choices": [
            {
                "message": {
                    "content": content
                }
            }
        ]
    })
    .to_string();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn read_http_request(stream: &mut TcpStream) -> String {
    let mut buf = Vec::<u8>::new();
    let mut chunk = [0u8; 4096];
    let mut header_end: Option<usize> = None;
    let mut expected_body_len = 0usize;

    loop {
        let read = stream.read(&mut chunk).unwrap_or(0);
        if read == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..read]);

        if header_end.is_none()
            && let Some(pos) = find_subsequence(&buf, b"\r\n\r\n")
        {
            let end = pos + 4;
            header_end = Some(end);
            let headers = String::from_utf8_lossy(&buf[..end]).to_string();
            expected_body_len = parse_content_length(&headers).unwrap_or(0);
        }

        if let Some(end) = header_end
            && buf.len() >= end + expected_body_len
        {
            break;
        }
    }

    String::from_utf8_lossy(&buf).to_string()
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn parse_content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if !name.eq_ignore_ascii_case("content-length") {
            return None;
        }
        value.trim().parse::<usize>().ok()
    })
}

fn extract_user_prompt_from_http_request(raw_request: &str) -> Option<String> {
    let body = raw_request.split("\r\n\r\n").nth(1)?;
    let root: Value = serde_json::from_str(body).ok()?;
    root.get("messages")
        .and_then(Value::as_array)?
        .iter()
        .find(|msg| msg.get("role").and_then(Value::as_str) == Some("user"))
        .and_then(|msg| msg.get("content").and_then(Value::as_str))
        .map(ToString::to_string)
}

fn extract_location_fn_name(user_prompt: &str) -> Option<&str> {
    let marker = "Location: ";
    let start = user_prompt.find(marker)? + marker.len();
    let line = user_prompt[start..].lines().next()?;
    let fn_marker = "fn=";
    let fn_start = line.find(fn_marker)? + fn_marker.len();
    let tail = &line[fn_start..];
    let fn_end = tail.find(',').unwrap_or(tail.len());
    let name = tail[..fn_end].trim();
    if name.is_empty() { None } else { Some(name) }
}

fn run_online_failure_case(
    case_name: &str,
    behavior: MockBehavior,
    detail_match: fn(&str) -> bool,
) {
    let _guard = online_test_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join(case_name);
    create_five_function_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    prepare_single_unresolved_partial_plan(&crate_path, &artifacts);

    let Some(server) = spawn_mock_server(behavior) else {
        eprintln!("skip online mock test in restricted sandbox: cannot bind localhost listener");
        return;
    };
    let output = run_with_env(
        &[
            "step",
            "verify",
            crate_path.to_str().unwrap(),
            "--enable-llm",
            "--llm-mode",
            "online",
            "--llm-api-url",
            server.api_url.as_str(),
            "--llm-model",
            "mock-model",
            "--llm-timeout-secs",
            "1",
            "--llm-max-rounds",
            "1",
            "--llm-debug-dump-full-io",
            "--artifacts-dir",
            artifacts.to_str().unwrap(),
        ],
        &[("RUTER_LLM_API_KEY", "test-key")],
    );
    let _ = server.handle.join();

    assert_eq!(
        output.status.code(),
        Some(7),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let attempts: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_attempts.json")).unwrap())
            .unwrap();
    assert_eq!(
        attempts.get("mode").and_then(Value::as_str),
        Some("online"),
        "attempt artifact should indicate online mode"
    );
    let arr = attempts
        .get("attempts")
        .and_then(Value::as_array)
        .expect("attempt array");
    let request_failed = arr.iter().find(|item| {
        item.get("phase").and_then(Value::as_str) == Some("request")
            && item.get("failure_kind").and_then(Value::as_str) == Some("LLM_REQUEST_FAILED")
    });
    let detail = request_failed
        .and_then(|item| item.get("failure_detail").and_then(Value::as_str))
        .expect("request failure detail must exist");
    assert!(detail_match(detail), "unexpected failure detail: {detail}");

    let io_debug: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_io_debug.json")).unwrap())
            .unwrap();
    let io_arr = io_debug.as_array().expect("io debug array");
    assert!(
        !io_arr.is_empty(),
        "io debug artifact should contain at least one request record"
    );
    let first = &io_arr[0];
    assert!(
        first
            .get("function_id")
            .and_then(Value::as_str)
            .map(|id| !id.trim().is_empty())
            .unwrap_or(false),
        "io debug should include function_id"
    );
    assert!(
        first
            .get("request_user_prompt")
            .and_then(Value::as_str)
            .map(|text| !text.trim().is_empty())
            .unwrap_or(false),
        "io debug should include user prompt"
    );
}

fn run_online_rust_fenced_chain_case(case_name: &str) {
    let _guard = online_test_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join(case_name);
    create_five_function_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    prepare_single_unresolved_partial_plan(&crate_path, &artifacts);

    let Some(server) = spawn_mock_server(MockBehavior::RustFencedFixFromPrompt) else {
        eprintln!("skip online mock test in restricted sandbox: cannot bind localhost listener");
        return;
    };
    let output = run_with_env(
        &[
            "step",
            "verify",
            crate_path.to_str().unwrap(),
            "--enable-llm",
            "--llm-mode",
            "online",
            "--llm-api-url",
            server.api_url.as_str(),
            "--llm-model",
            "mock-model",
            "--llm-timeout-secs",
            "5",
            "--llm-max-rounds",
            "1",
            "--llm-debug-dump-full-io",
            "--artifacts-dir",
            artifacts.to_str().unwrap(),
        ],
        &[("RUTER_LLM_API_KEY", "test-key")],
    );
    let _ = server.handle.join();

    assert!(
        matches!(output.status.code(), Some(0) | Some(7)),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let attempts: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_attempts.json")).unwrap())
            .unwrap();
    let arr = attempts
        .get("attempts")
        .and_then(Value::as_array)
        .expect("attempt array");
    let attempts_dump = serde_json::to_string_pretty(&attempts).unwrap_or_default();
    assert!(
        arr.iter().any(|item| {
            matches!(
                item.get("phase").and_then(Value::as_str),
                Some("normalize") | Some("verify")
            ) && item.get("candidate_id").and_then(Value::as_str).is_some()
        }),
        "expected fenced rust response to reach normalize/verify phase, attempts={attempts_dump}"
    );
    assert!(
        !arr.iter().any(|item| {
            item.get("phase").and_then(Value::as_str) == Some("request")
                && item.get("failure_kind").and_then(Value::as_str) == Some("LLM_REQUEST_FAILED")
        }),
        "request phase should not fail when mock returns 200"
    );

    let io_debug: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_io_debug.json")).unwrap())
            .unwrap();
    let io_arr = io_debug.as_array().expect("io debug array");
    assert!(
        io_arr.iter().any(|item| {
            item.get("response_content")
                .and_then(Value::as_str)
                .map(|content| content.contains("```rust"))
                .unwrap_or(false)
        }),
        "io debug should include fenced rust response"
    );
}

fn run_online_retry_history_memory_case(case_name: &str) {
    let _guard = online_test_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join(case_name);
    create_five_function_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    prepare_single_unresolved_partial_plan(&crate_path, &artifacts);

    let Some(server) = spawn_mock_server(MockBehavior::FailThenRecoverWithHistory) else {
        eprintln!("skip online mock test in restricted sandbox: cannot bind localhost listener");
        return;
    };
    let output = run_with_env(
        &[
            "step",
            "verify",
            crate_path.to_str().unwrap(),
            "--enable-llm",
            "--llm-mode",
            "online",
            "--llm-api-url",
            server.api_url.as_str(),
            "--llm-model",
            "mock-model",
            "--llm-timeout-secs",
            "5",
            "--llm-max-rounds",
            "2",
            "--llm-debug-dump-full-io",
            "--artifacts-dir",
            artifacts.to_str().unwrap(),
        ],
        &[("RUTER_LLM_API_KEY", "test-key")],
    );
    let _ = server.handle.join();

    assert!(
        matches!(output.status.code(), Some(0) | Some(7)),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let attempts: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_attempts.json")).unwrap())
            .unwrap();
    let arr = attempts
        .get("attempts")
        .and_then(Value::as_array)
        .expect("attempt array");
    assert!(
        arr.iter().any(|item| {
            item.get("round").and_then(Value::as_u64) == Some(1)
                && item.get("phase").and_then(Value::as_str) == Some("normalize")
                && item.get("failure_kind").and_then(Value::as_str)
                    == Some("LLM_OUTPUT_INVALID_SCHEMA")
        }),
        "round 1 should include invalid schema failure"
    );
    assert!(
        arr.iter()
            .any(|item| item.get("round").and_then(Value::as_u64) == Some(2)),
        "round 2 attempts should exist after retry"
    );

    let io_debug: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_io_debug.json")).unwrap())
            .unwrap();
    let io_arr = io_debug.as_array().expect("io debug array");
    assert!(io_arr.len() >= 2, "expected at least two online requests");
    let first_prompt = io_arr
        .first()
        .and_then(|item| item.get("request_user_prompt"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        first_prompt.contains("Local rule patcher failure summary"),
        "round 1 prompt should include local rule failure summary section"
    );
    assert!(
        first_prompt.contains("remaining_error_by_code=E0433=1"),
        "round 1 prompt should include local rule failure error summary"
    );
    let second_prompt = io_arr
        .get(1)
        .and_then(|item| item.get("request_user_prompt"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        second_prompt.contains("Previous round failures to avoid repeating"),
        "round 2 prompt should contain retry history section"
    );
    assert!(
        second_prompt.contains("LLM_OUTPUT_INVALID_SCHEMA"),
        "round 2 prompt should include previous failure kind"
    );
}

fn run_online_rule_drift_prompt_case(case_name: &str) {
    let _guard = online_test_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join(case_name);
    create_five_function_project(&crate_path);

    let artifacts = dir.path().join("artifacts");
    prepare_single_unresolved_partial_plan(&crate_path, &artifacts);
    inject_rule_drift_fixture(&crate_path, &artifacts);

    let Some(server) = spawn_mock_server(MockBehavior::RustFencedFixFromPrompt) else {
        eprintln!("skip online mock test in restricted sandbox: cannot bind localhost listener");
        return;
    };
    let output = run_with_env(
        &[
            "step",
            "verify",
            crate_path.to_str().unwrap(),
            "--enable-llm",
            "--llm-mode",
            "online",
            "--llm-api-url",
            server.api_url.as_str(),
            "--llm-model",
            "mock-model",
            "--llm-timeout-secs",
            "5",
            "--llm-max-rounds",
            "1",
            "--llm-debug-dump-full-io",
            "--artifacts-dir",
            artifacts.to_str().unwrap(),
        ],
        &[("RUTER_LLM_API_KEY", "test-key")],
    );
    let _ = server.handle.join();
    assert!(
        matches!(output.status.code(), Some(0) | Some(7)),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let io_debug: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_io_debug.json")).unwrap())
            .unwrap();
    let first_prompt = io_debug
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("request_user_prompt"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        first_prompt.contains("Rule patch drift hints"),
        "prompt should contain rule drift section"
    );
    assert!(
        first_prompt.contains("drift_pairs=E0433->E0599"),
        "prompt should include E0433->E0599 drift pair"
    );
    assert!(
        first_prompt.contains("r1 rank=1"),
        "prompt should include selected rule fix summary"
    );
    assert!(
        first_prompt.contains("original target function source (before rule patch)"),
        "prompt should include original target function source section"
    );
}

fn detail_contains_status_401(detail: &str) -> bool {
    detail.contains("status 401")
}

fn detail_contains_status_429(detail: &str) -> bool {
    detail.contains("status 429")
}

fn detail_contains_status_500(detail: &str) -> bool {
    detail.contains("status 500")
}

fn detail_contains_timeout(detail: &str) -> bool {
    let lower = detail.to_ascii_lowercase();
    lower.contains("timed out") || lower.contains("timeout") || lower.contains("deadline")
}

#[test]
fn online_401_exit7_and_attempt_request_failed() {
    run_online_failure_case(
        "proj_online_401",
        MockBehavior::Status(401),
        detail_contains_status_401,
    );
}

#[test]
fn online_429_exit7_and_attempt_request_failed() {
    run_online_failure_case(
        "proj_online_429",
        MockBehavior::Status(429),
        detail_contains_status_429,
    );
}

#[test]
fn online_5xx_exit7_and_attempt_request_failed() {
    run_online_failure_case(
        "proj_online_500",
        MockBehavior::Status(500),
        detail_contains_status_500,
    );
}

#[test]
fn online_timeout_exit7_and_attempt_request_failed() {
    run_online_failure_case(
        "proj_online_timeout",
        MockBehavior::Timeout,
        detail_contains_timeout,
    );
}

#[test]
fn online_rust_fenced_response_enters_normalize_or_verify_chain() {
    run_online_rust_fenced_chain_case("proj_online_rust_fenced_chain");
}

#[test]
fn online_retry_prompt_injects_previous_round_failure_digest() {
    run_online_retry_history_memory_case("proj_online_retry_history_memory");
}

#[test]
fn online_prompt_injects_rule_error_drift_digest() {
    run_online_rule_drift_prompt_case("proj_online_rule_drift_digest");
}

#[test]
#[ignore = "requires explicit online endpoint credentials"]
fn online_smoke_real_endpoint_ignored_by_default() {
    let Some(api_url) = std::env::var("RUTER_LLM_API_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
    else {
        return;
    };
    let Some(model) = std::env::var("RUTER_LLM_MODEL")
        .ok()
        .filter(|v| !v.trim().is_empty())
    else {
        return;
    };
    let Some(api_key) = std::env::var("RUTER_LLM_API_KEY")
        .ok()
        .filter(|v| !v.trim().is_empty())
    else {
        return;
    };

    let dir = tempdir().expect("tempdir");
    let crate_path = dir.path().join("proj");
    create_five_function_project(&crate_path);
    let artifacts = dir.path().join("artifacts");
    prepare_single_unresolved_partial_plan(&crate_path, &artifacts);

    let output = run_with_env(
        &[
            "step",
            "verify",
            crate_path.to_str().unwrap(),
            "--enable-llm",
            "--llm-mode",
            "online",
            "--llm-api-url",
            api_url.as_str(),
            "--llm-model",
            model.as_str(),
            "--llm-max-rounds",
            "1",
            "--artifacts-dir",
            artifacts.to_str().unwrap(),
        ],
        &[("RUTER_LLM_API_KEY", api_key.as_str())],
    );

    assert!(
        matches!(output.status.code(), Some(0) | Some(7)),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let attempts: Value =
        serde_json::from_str(&fs::read_to_string(artifacts.join("4_llm_attempts.json")).unwrap())
            .unwrap();
    assert_eq!(attempts.get("mode").and_then(Value::as_str), Some("online"));
    assert!(
        attempts
            .get("attempts")
            .and_then(Value::as_array)
            .map(|arr| !arr.is_empty())
            .unwrap_or(false),
        "smoke run should write non-empty attempts"
    );
}
