#!/usr/bin/env python3
"""Run disjoint Claude Code shards and atomically publish per-case results."""

from __future__ import annotations

import fcntl
import getpass
import json
import os
import shutil
import signal
import socket
import subprocess
import sys
import threading
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import run_claude_code_experiment as serial
from run_token_cost_experiment import load_json, record_infrastructure_failure, write_json


def atomic_json(path: Path, value: Any) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    write_json(temporary, value)
    temporary.replace(path)


def validate_cases(cases: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    by_id = {str(case["case_id"]): case for case in cases}
    if len(by_id) != len(cases) or len({case["attempt_uid"] for case in cases}) != len(cases):
        raise ValueError("case IDs and attempt UIDs must both be unique")
    for case_id in by_id:
        if Path(case_id).name != case_id or case_id in {".", ".."}:
            raise ValueError(f"unsafe case ID: {case_id}")
    return by_id


def pending_cases(cases: list[dict[str, Any]], out_root: Path, retry: bool) -> list[dict[str, Any]]:
    pending = []
    for case in cases:
        path = out_root / "cases" / case["case_id"] / "case_result.json"
        if not path.is_file():
            pending.append(case)
            continue
        result = load_json(path, {})
        if result.get("attempt_uid") != case["attempt_uid"]:
            raise ValueError(f"saved case UID differs: {path}")
        if retry and serial.is_retryable_infrastructure_failure(result):
            pending.append(case)
    return pending


def shard_cases(cases: list[dict[str, Any]], workers: int) -> list[list[dict[str, Any]]]:
    validate_cases(cases)
    return [cases[index::workers] for index in range(workers)]


def publish_result(
    source: Path, out_root: Path, expected: dict[str, dict[str, Any]],
    worker_id: int, workers: int,
) -> bool:
    """Only the coordinator publishes; existing method outcomes never change."""
    marker = load_json(source / "runner_complete.json", {})
    if marker.get("case_id") != source.name:
        return False
    result = load_json(source / "case_result.json", {})
    case_id = str(result.get("case_id") or "")
    case = expected.get(case_id)
    if case is None or result.get("attempt_uid") != case["attempt_uid"] or source.name != case_id:
        raise ValueError(f"worker returned an unexpected case: {source}")
    destination = out_root / "cases" / case_id
    existing = load_json(destination / "case_result.json", {})
    if existing:
        if existing.get("attempt_uid") != case["attempt_uid"]:
            raise ValueError(f"destination UID differs: {destination}")
        if not serial.is_retryable_infrastructure_failure(existing):
            return False
        if (serial.is_retryable_infrastructure_failure(result)
                and str(result.get("finished_at_utc") or "") <= str(existing.get("finished_at_utc") or "")):
            return False
    token = uuid.uuid4().hex
    staging = out_root / "_publish_staging" / f"{case_id}_{token}"
    staging.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(source, staging)
    result.update(execution_worker_id=worker_id, execution_worker_count=workers,
                  published_from=str(source.resolve()))
    write_json(staging / "case_result.json", result)
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists():
        archive = out_root / "_aborted_cases" / f"{case_id}_parallel_{token}"
        archive.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(destination), str(archive))
    staging.rename(destination)
    if serial.is_retryable_infrastructure_failure(result):
        record_infrastructure_failure(out_root, "claude", destination / "case_result.json")
    print(f"[publish worker={worker_id}] {case_id} {result.get('status')} "
          f"tokens={(result.get('usage') or {}).get('total_tokens')}", flush=True)
    return True


def worker_settings(template: Path, destination: Path, port: int) -> Path:
    settings = load_json(template, {})
    # Claude's explicit settings env takes precedence over the process env.
    # The base template is frozen; clone it and override ONLY loopback routing.
    settings.setdefault("env", {})["ANTHROPIC_BASE_URL"] = f"http://127.0.0.1:{port}"
    atomic_json(destination, settings)
    return destination


def check_proxy_port(port: int) -> None:
    """Match HTTPServer's address reuse without permitting another listener.

    Completed proxy connections can leave TIME_WAIT entries after the proxy
    exits. A plain bind falsely rejects those during the next retry sweep.
    SO_REUSEADDR allows that safe restart, but not an occupied listening port.
    """
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            probe.bind(("127.0.0.1", port))
            probe.listen(1)
        except OSError as error:
            raise RuntimeError(f"proxy port {port} is unavailable; existing services will not be stopped") from error


def serial_command(args: Any, out_root: Path, case_list: Path, port: int, target: Path,
                   settings_override: Path | None = None) -> list[str]:
    command = [sys.executable, "-u", str(serial.SCRIPT_PATH),
               "--paired-manifest", str(Path(args.paired_manifest).resolve()),
               "--out", str(out_root), "--case-list", str(case_list),
               "--proxy-port", str(port), "--cargo-target-root", str(target), "--resume"]
    for option, value in (
        ("--settings", settings_override or args.settings), ("--prompt-template", args.prompt_template),
        ("--verify-hook", args.verify_hook), ("--proxy-script", args.proxy_script),
        ("--upstream", args.upstream), ("--model", args.model),
        ("--claude-bin", args.claude_bin), ("--max-turns", args.max_turns),
        ("--timeout-secs", args.timeout_secs), ("--cargo-timeout-secs", args.cargo_timeout_secs),
        ("--seed", args.seed),
    ):
        command.extend([option, str(value)])
    if args.discard_workspaces:
        command.append("--discard-workspaces")
    if args.prepare_only:
        command.append("--prepare-only")
    return command


def stop_process(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    os.killpg(process.pid, signal.SIGINT)
    try:
        process.wait(timeout=30)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()


def bootstrap(command: list[str], env: dict[str, str], log_path: Path) -> None:
    with log_path.open("a", encoding="utf-8") as log:
        process = subprocess.Popen(command, cwd=serial.PROJECT_ROOT, env=env,
                                   stdout=log, stderr=subprocess.STDOUT, text=True,
                                   start_new_session=True)
        try:
            if process.wait() != 0:
                raise RuntimeError(f"result initialization failed; see {log_path}")
        finally:
            stop_process(process)


def capture_output(process: subprocess.Popen[str], path: Path, worker_id: int) -> None:
    with path.open("a", encoding="utf-8") as log:
        assert process.stdout is not None
        for line in process.stdout:
            log.write(line)
            log.flush()
            print(f"[worker {worker_id}] {line}", end="", flush=True)


def run_parallel(args: Any, cases: list[dict[str, Any]], env: dict[str, str]) -> int:
    out_root = Path(args.out).resolve()
    expected = validate_cases(cases)
    previous = load_json(out_root / "experiment_manifest.json", {})
    if previous and {r["case_id"] for r in previous.get("cases", [])} != set(expected):
        raise ValueError("saved selection differs; use a separate output root")
    run_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "_" + uuid.uuid4().hex[:6]
    run_root = out_root / "_parallel_workers" / run_id
    run_root.mkdir(parents=True)
    target_root = Path(args.cargo_target_root).resolve() if args.cargo_target_root else out_root / "_cargo_target"
    selected = run_root / "selected_cases.json"
    write_json(selected, list(expected))
    init_command = serial_command(args, out_root, selected, args.proxy_port, target_root)
    init_command.append("--reuse-only")
    if args.reuse_results_from:
        init_command.extend(["--reuse-results-from", args.reuse_results_from])
    bootstrap(init_command, env, run_root / "initialize.log")
    manifest = load_json(out_root / "experiment_manifest.json", {})
    manifest["parallel_execution"] = {
        "worker_count": args.workers, "cargo_jobs_per_worker": args.cargo_jobs_per_worker,
        "scheduling": "round_robin_pending_frozen_case_order",
        "proxy_ports": [args.proxy_port + i + 1 for i in range(args.workers)],
        "per_case_protocol_unchanged": True,
        "worker_settings_override_fields": ["env.ANTHROPIC_BASE_URL"],
    }
    atomic_json(out_root / "experiment_manifest.json", manifest)
    # Recover finished-but-unpublished work before deciding which calls to make.
    for marker in sorted((out_root / "_parallel_workers").glob("*/worker_*/cases/*/runner_complete.json")):
        source = marker.parent
        worker_id = int(source.parents[1].name.split("_")[-1])
        publish_result(source, out_root, expected, worker_id, args.workers)
    pending = pending_cases(cases, out_root, args.retry_infrastructure_failures)
    assignments = shard_cases(pending, args.workers)
    with (out_root / "parallel_execution_history.jsonl").open("a", encoding="utf-8") as stream:
        stream.write(json.dumps({"started_at_utc": serial.utc_now(), "run_id": run_id,
                                "worker_count": args.workers, "case_count": len(cases),
                                "scheduled_cases": len(pending), "api_key_persisted": False}) + "\n")
    processes: list[tuple[int, subprocess.Popen[str], Path, threading.Thread]] = []
    seen: set[Path] = set()
    state_path = out_root / "parallel_state.json"
    last_summary = 0.0
    published = 0
    try:
        for index, shard in enumerate(assignments):
            if not shard:
                continue
            port = args.proxy_port + index + 1
            check_proxy_port(port)
            worker_root = run_root / f"worker_{index}"
            assignment_path = run_root / f"worker_{index}_cases.json"
            write_json(assignment_path, [case["case_id"] for case in shard])
            settings = worker_settings(Path(args.settings), run_root / f"worker_{index}_settings.json", port)
            command = serial_command(args, worker_root, assignment_path, port,
                                     target_root / f"worker_{index}", settings)
            command.append("--completion-markers")
            worker_env = {**env, "CARGO_BUILD_JOBS": str(args.cargo_jobs_per_worker)}
            process = subprocess.Popen(command, cwd=serial.PROJECT_ROOT, env=worker_env,
                                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                       text=True, bufsize=1, start_new_session=True)
            reader = threading.Thread(target=capture_output,
                                      args=(process, run_root / f"worker_{index}.log", index), daemon=True)
            processes.append((index, process, worker_root, reader))
            reader.start()
            print(f"worker {index}: pid={process.pid} port={port} cases={len(shard)}", flush=True)
        while processes:
            worker_states = []
            for index, process, worker_root, _reader in processes:
                markers = sorted(worker_root.glob("cases/*/runner_complete.json"))
                for marker in markers:
                    if marker in seen:
                        continue
                    if publish_result(marker.parent, out_root, expected, index, args.workers):
                        published += 1
                    seen.add(marker)
                worker_states.append({"worker_id": index, "pid": process.pid,
                                      "port": args.proxy_port + index + 1,
                                      "assigned_cases": len(assignments[index]),
                                      "finished_cases": len(markers), "exit_code": process.poll()})
            count = len(list(out_root.glob("cases/*/case_result.json")))
            atomic_json(state_path, {"updated_at_utc": serial.utc_now(), "status": "running",
                                    "coordinator_pid": os.getpid(), "worker_count": args.workers,
                                    "case_count": len(cases), "saved_case_count": count,
                                    "scheduled_cases": len(pending), "published_this_run": published,
                                    "workers": worker_states})
            if time.monotonic() - last_summary >= 60:
                serial.summarize(out_root, False, False, args.seed)
                last_summary = time.monotonic()
            failed = [state for state in worker_states if state["exit_code"] not in {None, 0}]
            if failed:
                raise RuntimeError(f"worker exited unexpectedly: {failed}; see {run_root}")
            if all(process.poll() is not None for _, process, _, _ in processes):
                break
            time.sleep(args.poll_secs)
        for _, process, _, reader in processes:
            process.wait()
            reader.join(timeout=5)
        # A worker can finalize its last marker between the last scan and exit.
        # Drain again after all processes have stopped before aggregating.
        for index, _, worker_root, _ in processes:
            for marker in sorted(worker_root.glob("cases/*/runner_complete.json")):
                if marker not in seen:
                    if publish_result(marker.parent, out_root, expected, index, args.workers):
                        published += 1
                    seen.add(marker)
    except BaseException:
        for _, process, _, _ in processes:
            stop_process(process)
        atomic_json(state_path, {"updated_at_utc": serial.utc_now(), "status": "interrupted",
                                "coordinator_pid": os.getpid(), "worker_count": args.workers})
        raise
    finally:
        for _, process, _, _ in processes:
            stop_process(process)
    full_frame = load_json(Path(args.paired_manifest), {}).get("cases", [])
    complete = {r["attempt_uid"] for r in cases} == {r["attempt_uid"] for r in full_frame}
    summary = serial.summarize(out_root, complete, complete, args.seed)
    missing = [case for case in cases if not (out_root / "cases" / case["case_id"] / "case_result.json").is_file()]
    atomic_json(state_path, {"updated_at_utc": serial.utc_now(), "status": "finished" if not missing else "incomplete",
                            "coordinator_pid": os.getpid(), "worker_count": args.workers,
                            "case_count": len(cases), "saved_case_count": summary["case_count"],
                            "workers": worker_states if processes else []})
    if missing:
        return 2
    if args.stop_on_infrastructure_failure and any(
        serial.is_retryable_infrastructure_failure(load_json(out_root / "cases" / r["case_id"] / "case_result.json", {}))
        for r in cases
    ):
        return 3
    return 0


def main() -> int:
    parser = serial.build_parser()
    parser.description = __doc__
    parser.add_argument("--workers", type=int, choices=(2, 3), default=2)
    parser.add_argument("--cargo-jobs-per-worker", type=int, default=2)
    parser.add_argument("--poll-secs", type=float, default=2.0)
    args = parser.parse_args()
    if args.poll_secs <= 0 or args.cargo_jobs_per_worker <= 0 or args.proxy_port + args.workers > 65535:
        raise SystemExit("invalid polling interval, Cargo jobs or proxy port range")
    if args.reuse_only or args.completion_markers or args.sample_size:
        raise SystemExit("reuse-only, completion-markers and sampling are coordinator-internal/unsupported options")
    frame = load_json(Path(args.paired_manifest), {}).get("cases", [])
    requested = args.case_id
    if args.case_list:
        requested = load_json(Path(args.case_list), [])
        if not isinstance(requested, list) or not requested or not all(isinstance(x, str) for x in requested):
            raise SystemExit("--case-list must contain a nonempty JSON array")
        if len(set(requested)) != len(requested):
            raise SystemExit("duplicate case IDs")
    cases = serial.choose_cases(frame, pilot=args.pilot, requested=requested, limit=args.limit,
                                sample_size=None, seed=args.seed)
    validate_cases(cases)
    out_root = Path(args.out).resolve()
    if out_root.exists() and any(out_root.iterdir()) and not args.resume:
        raise SystemExit("output directory is nonempty; use --resume")
    out_root.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["DISABLE_AUTOUPDATER"] = "1"
    key = env.get("CLAUDE_EXPERIMENT_API_KEY")
    if not args.prepare_only and not key:
        key = getpass.getpass("4sapi key (not stored): ") if sys.stdin.isatty() else None
        if not key:
            raise SystemExit("CLAUDE_EXPERIMENT_API_KEY or hidden interactive input required")
        env["CLAUDE_EXPERIMENT_API_KEY"] = key
    with (out_root / ".parallel.lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise SystemExit("another coordinator owns this output root")
        return run_parallel(args, cases, env)


if __name__ == "__main__":
    raise SystemExit(main())
