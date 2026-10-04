#!/usr/bin/env python3
"""Run the complete RuTeR -> Claude Code paired token experiment.

The launcher performs authenticated OpenAI- and Anthropic-protocol preflights,
runs RuTeR on every frozen failure, materializes its summary, and then runs
Claude Code on the exact resulting case manifest.  Both systems therefore use
one API account, endpoint, model alias, and frozen population.
"""

from __future__ import annotations

import argparse
import getpass
import hashlib
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


SCRIPT_PATH = Path(__file__).resolve()
RUTER_ROOT = SCRIPT_PATH.parents[2]
PROJECT_ROOT = RUTER_ROOT
SCRIPTS_ROOT = SCRIPT_PATH.parent
DEFAULT_MANIFEST = PROJECT_ROOT / "experiments/inputs/frozen_attempt_manifest.json"
DEFAULT_CRATES_ROOT = PROJECT_ROOT / "experiments/inputs/crates"
ARTIFACT_ROOT = PROJECT_ROOT / "experiments/artifacts/token_cost"
DEFAULT_RUTER_OUT = ARTIFACT_ROOT / "ruter"
DEFAULT_CLAUDE_OUT = ARTIFACT_ROOT / "claude"
DEFAULT_CONTROL_ROOT = ARTIFACT_ROOT / "control"


def load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise RuntimeError(f"expected JSON object: {path}")
    return value


def normalized_base(raw: str) -> str:
    value = raw.strip().rstrip("/")
    if value.endswith("/v1"):
        value = value[:-3]
    if not value.startswith(("https://", "http://")):
        raise ValueError("--api-base must be an absolute HTTP(S) URL")
    return value.rstrip("/")


def request_json(
    url: str,
    key: str,
    *,
    payload: dict[str, Any] | None = None,
    anthropic: bool = False,
) -> dict[str, Any]:
    headers = {"Authorization": f"Bearer {key}"}
    if payload is not None:
        headers["Content-Type"] = "application/json"
    if anthropic:
        headers["x-api-key"] = key
        headers["anthropic-version"] = "2023-06-01"
    request = Request(
        url,
        data=(
            json.dumps(payload, ensure_ascii=False).encode("utf-8")
            if payload is not None
            else None
        ),
        headers=headers,
        method="POST" if payload is not None else "GET",
    )
    try:
        with urlopen(request, timeout=60) as response:
            value = json.load(response)
    except HTTPError as error:
        body = error.read(1000).decode("utf-8", errors="replace")
        raise RuntimeError(f"{url} returned HTTP {error.code}: {body}") from error
    except URLError as error:
        raise RuntimeError(f"cannot reach {url}: {error}") from error
    if not isinstance(value, dict):
        raise RuntimeError(f"{url} returned a non-object JSON response")
    return value


def preflight(api_base: str, key: str, model: str) -> None:
    from run_claude_code_experiment import model_valid

    models = request_json(f"{api_base}/v1/models", key)
    ids = {
        str(item.get("id"))
        for item in models.get("data", [])
        if isinstance(item, dict) and item.get("id")
    }
    if model not in ids:
        raise RuntimeError(
            f"requested model {model!r} is absent from {api_base}/v1/models"
        )
    openai = request_json(
        f"{api_base}/v1/chat/completions",
        key,
        payload={
            "model": model,
            "messages": [{"role": "user", "content": "Reply only OK"}],
            "max_tokens": 8,
        },
    )
    if not openai.get("usage") and not openai.get("billing_usage"):
        raise RuntimeError("OpenAI preflight succeeded without provider usage metadata")
    anthropic = request_json(
        f"{api_base}/v1/messages",
        key,
        anthropic=True,
        payload={
            "model": model,
            "messages": [{"role": "user", "content": "Reply only OK"}],
            "max_tokens": 8,
        },
    )
    if not anthropic.get("usage"):
        raise RuntimeError("Anthropic preflight succeeded without usage metadata")
    billing = (anthropic.get("usage") or {}).get("billing_usage") or {}
    if not isinstance(billing.get("openai_usage"), dict):
        raise RuntimeError("Anthropic preflight lacks comparable OpenAI billing usage")
    for response in (openai, anthropic):
        if not model_valid([str(response.get("model") or "")], model):
            raise RuntimeError(f"preflight returned unexpected model {response.get('model')}")
    print(
        f"preflight passed: endpoint={api_base} model={model} "
        f"openai_model={openai.get('model')} anthropic_model={anthropic.get('model')}",
        flush=True,
    )


def run(command: list[str], env: dict[str, str], log_path: Path) -> int:
    print("+ " + " ".join(command), flush=True)
    with log_path.open("a", encoding="utf-8") as log:
        with subprocess.Popen(
            command, cwd=PROJECT_ROOT, env=env, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, text=True, bufsize=1,
        ) as process:
            assert process.stdout is not None
            for line in process.stdout:
                print(line, end="", flush=True)
                log.write(line)
                log.flush()
            return process.wait()


def write_status(control_root: Path, **fields: Any) -> None:
    path = control_root / "pipeline_status.json"
    status = {
        "updated_at_utc": datetime.now(timezone.utc).isoformat(),
        "launcher_pid": os.getpid(),
        **fields,
    }
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(status, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def run_stage(
    stage: str, command: list[str], env: dict[str, str], root: Path,
    control_root: Path,
) -> None:
    """Stop a runner at a bad API case and retry it at most three times.

    The runner archives prior attempts when resumed. Normal repair failures do
    not trigger exit 3 and are never retried by this wrapper.
    """
    from run_token_cost_experiment import is_retryable_infrastructure_failure as ruter_bad
    from run_claude_code_experiment import is_retryable_infrastructure_failure as claude_bad

    retry_counts: dict[str, int] = {}
    while True:
        write_status(control_root, status="running", stage=stage, output=str(root))
        code = run(command, env, control_root / f"{stage}.log")
        if code == 0:
            return
        if code != 3:
            raise RuntimeError(f"{stage} runner exited {code}; see {control_root}/{stage}.log")
        failed = []
        for path in root.glob("cases/*/case_result.json"):
            if (ruter_bad(path) if stage == "ruter" else claude_bad(load_json(path))):
                failed.append(path.parent.name)
        if not failed:
            raise RuntimeError(f"{stage} stopped for infrastructure without a saved bad case")
        for case_id in failed:
            retry_counts[case_id] = retry_counts.get(case_id, 0) + 1
        if any(retry_counts[case_id] >= 3 for case_id in failed):
            raise RuntimeError(f"{stage} API failed on the same case three times: {failed}")
        print(f"{stage}: API failure {failed}; waiting 60s before resuming", flush=True)
        write_status(
            control_root, status="retrying_infrastructure", stage=stage,
            cases=failed, retry_counts=retry_counts, cooldown_secs=60,
        )
        time.sleep(60)


def assert_ruter_complete(root: Path, expected: int) -> None:
    summary_path = root / "summary/token_usage_summary.json"
    summary = load_json(summary_path)
    if int(summary.get("case_count") or 0) != expected:
        raise RuntimeError(
            f"RuTeR summary has {summary.get('case_count')} cases; expected {expected}"
        )
    if int(summary.get("usage_incomplete_case_count") or 0) != 0:
        raise RuntimeError(
            "RuTeR has incomplete provider usage; resume with "
            "--retry-infrastructure-failures before running Claude Code"
        )
    if int(summary.get("returned_model_invalid_case_count") or 0) != 0:
        raise RuntimeError("RuTeR received an unexpected provider model")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--api-base", default="https://4sapi.org")
    parser.add_argument("--model", default="gemini-2.5-flash-nothinking")
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--clean-crates-root", type=Path, default=DEFAULT_CRATES_ROOT)
    parser.add_argument("--ruter-out", type=Path, default=DEFAULT_RUTER_OUT)
    parser.add_argument("--claude-out", type=Path, default=DEFAULT_CLAUDE_OUT)
    parser.add_argument("--control-root", type=Path, default=DEFAULT_CONTROL_ROOT)
    parser.add_argument(
        "--reuse-ruter-results-from", type=Path,
        default=None,
    )
    parser.add_argument(
        "--reuse-claude-results-from", type=Path,
        default=None,
    )
    parser.add_argument("--preflight-only", action="store_true")
    parser.add_argument("--claude-workers", type=int, choices=(1, 2, 3), default=1)
    parser.add_argument(
        "--defer-infrastructure-retries", action="store_true",
        help="Finish both full sweeps before retrying saved API failures",
    )
    parser.add_argument(
        "--deferred-retry-passes", type=int, default=2,
        help="Maximum deferred retry sweeps after both first sweeps (default: 2)",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if args.deferred_retry_passes < 0:
        raise SystemExit("--deferred-retry-passes must not be negative")
    api_base = normalized_base(args.api_base)
    key = os.environ.get("FULL_PAIRED_API_KEY")
    if not key and sys.stdin.isatty():
        key = getpass.getpass("4sapi key (not stored): ")
    if not key:
        raise SystemExit("FULL_PAIRED_API_KEY must be set or entered interactively")

    if args.preflight_only:
        preflight(api_base, key, args.model)
        return 0

    manifest = load_json(args.manifest.resolve())
    attempts = manifest.get("attempts")
    if not isinstance(attempts, list) or not attempts:
        raise SystemExit(f"frozen manifest contains no attempts: {args.manifest}")
    population_size = len(attempts)
    if len({item["attempt_uid"] for item in attempts}) != population_size:
        raise SystemExit("frozen population contains duplicate attempt UIDs")
    if not args.clean_crates_root.is_dir():
        raise SystemExit(f"clean crate snapshots are required: {args.clean_crates_root}")

    ruter_out = args.ruter_out.resolve()
    claude_out = args.claude_out.resolve()
    control_root = args.control_root.resolve()
    control_root.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    for name in ("FULL_PAIRED_API_KEY", "RUTER_LLM_API_KEY", "CLAUDE_EXPERIMENT_API_KEY"):
        env.pop(name, None)
    env["DISABLE_AUTOUPDATER"] = "1"
    ruter_env = {**env, "RUTER_LLM_API_KEY": key}
    claude_env = {**env, "CLAUDE_EXPERIMENT_API_KEY": key}
    from run_claude_code_experiment import DEFAULT_SETTINGS, DEFAULT_PROMPT, DEFAULT_VERIFY_HOOK

    protocol_files = [
        args.manifest.resolve(),
        DEFAULT_SETTINGS,
        DEFAULT_PROMPT,
        DEFAULT_VERIFY_HOOK,
        SCRIPTS_ROOT / "anthropic_usage_proxy.py",
    ]
    from run_token_cost_experiment import resolve_binary
    from run_claude_code_experiment import claude_version

    binary = resolve_binary(None)
    protocol = {
        "population_size": population_size,
        "api_base": api_base,
        "repair_model": args.model,
        "claude_version": claude_version("claude"),
        "ruter_binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "ruter_out": str(ruter_out),
        "claude_out": str(claude_out),
        "clean_crates_root": str(args.clean_crates_root.resolve()),
        "reuse_ruter_results_from": str(args.reuse_ruter_results_from.resolve()) if args.reuse_ruter_results_from else None,
        "reuse_claude_results_from": str(args.reuse_claude_results_from.resolve()) if args.reuse_claude_results_from else None,
        "frozen_file_sha256": {
            str(path): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in protocol_files
        },
        "api_key_persisted": False,
        "worker_count": 1,
        "infrastructure_attempts_per_case": 3,
        "infrastructure_cooldown_secs": 60,
    }
    policy_fields = {"infrastructure_failure_policy", "deferred_retry_passes",
                     "claude_worker_count", "claude_scheduling"}
    if args.claude_workers > 1:
        protocol.update(claude_worker_count=args.claude_workers,
                        claude_scheduling="round_robin_pending_frozen_case_order")
    if args.defer_infrastructure_retries:
        protocol.update(
            infrastructure_failure_policy="record_and_defer_until_both_full_sweeps",
            deferred_retry_passes=args.deferred_retry_passes,
        )
    protocol_path = control_root / "pipeline_manifest.json"
    if protocol_path.is_file():
        saved_protocol = load_json(protocol_path)
        saved_core = {k: v for k, v in saved_protocol.items() if k not in policy_fields}
        current_core = {k: v for k, v in protocol.items() if k not in policy_fields}
        if saved_core != current_core:
            raise SystemExit("saved full-experiment protocol differs; use new output roots")
        if saved_protocol != protocol:
            history_path = control_root / "infrastructure_policy_history.jsonl"
            with history_path.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps({
                    "changed_at_utc": datetime.now(timezone.utc).isoformat(),
                    "previous_policy": saved_protocol.get("infrastructure_failure_policy", "stop_and_retry_immediately"),
                    "new_policy": protocol.get("infrastructure_failure_policy", "stop_and_retry_immediately"),
                    "deferred_retry_passes": args.deferred_retry_passes,
                    "previous_claude_workers": saved_protocol.get("claude_worker_count", 1),
                    "new_claude_workers": args.claude_workers,
                    "core_repair_protocol_unchanged": True,
                }) + "\n")
    protocol_path.write_text(json.dumps(protocol, indent=2) + "\n", encoding="utf-8")
    write_status(control_root, status="preflight", stage="preflight")
    try:
        for attempt in range(1, 4):
            try:
                preflight(api_base, key, args.model)
                break
            except RuntimeError as error:
                if attempt == 3:
                    raise
                print(f"preflight attempt {attempt} failed; retrying in 60s: {error}", flush=True)
                write_status(
                    control_root, status="retrying_infrastructure", stage="preflight",
                    attempt=attempt, cooldown_secs=60,
                )
                time.sleep(60)
        complete = run_pipeline(args, population_size, api_base, ruter_out, claude_out, control_root, ruter_env, claude_env, env)
    except Exception as error:
        write_status(control_root, status="stopped", error=str(error).replace(key, "[redacted]"))
        raise
    if complete:
        write_status(control_root, status="complete", stage="complete")
        print(f"full paired experiment complete: {claude_out}", flush=True)
        return 0
    write_status(control_root, status="needs_retry", stage="deferred_retry",
                 failure_manifest=str(control_root / "pending_api_failures.json"))
    print("Full sweeps finished; some API cases remain pending. See pending_api_failures.json.", flush=True)
    return 3


def run_pipeline(
    args: argparse.Namespace, population_size: int, api_base: str,
    ruter_out: Path, claude_out: Path, control_root: Path,
    ruter_env: dict[str, str], claude_env: dict[str, str], env: dict[str, str],
) -> bool:
    ruter_command = [
            sys.executable,
            str(SCRIPTS_ROOT / "run_token_cost_experiment.py"),
            "--manifest",
            str(args.manifest.resolve()),
            "--clean-crates-root",
            str(args.clean_crates_root.resolve()),
            "--sample-size",
            str(population_size),
            "--seed",
            "20260924",
            "--out",
            str(ruter_out),
            "--api-url",
            f"{api_base}/v1",
            "--model",
            args.model,
            "--max-rounds",
            "3",
            "--timeout-secs",
            "900",
            "--llm-timeout-secs",
            "120",
            "--discard-workspaces",
            "--cargo-target-root",
            str(ruter_out / "_cargo_target"),
            "--resume",
            "--retry-infrastructure-failures",
            "--stop-on-infrastructure-failure",
        ]
    if args.reuse_ruter_results_from:
        ruter_command.extend(["--reuse-results-from", str(args.reuse_ruter_results_from.resolve())])
    summary_command = [
            sys.executable,
            str(SCRIPTS_ROOT / "summarize_token_usage.py"),
            "--experiment-root",
            str(ruter_out),
            "--out",
            str(ruter_out / "summary"),
        ]
    claude_command = [
            sys.executable,
            str(SCRIPTS_ROOT / "run_claude_code_experiment.py"),
            "--paired-manifest",
            str(ruter_out / "experiment_manifest.json"),
            "--out",
            str(claude_out),
            "--upstream",
            api_base,
            "--model",
            args.model,
            "--discard-workspaces",
            "--cargo-target-root",
            str(claude_out / "_cargo_target"),
            "--max-turns",
            "20",
            "--timeout-secs",
            "900",
            "--cargo-timeout-secs",
            "600",
            "--resume",
            "--retry-infrastructure-failures",
            "--stop-on-infrastructure-failure",
        ]
    if args.reuse_claude_results_from:
        claude_command.extend(["--reuse-results-from", str(args.reuse_claude_results_from.resolve())])
    if getattr(args, "claude_workers", 1) > 1:
        claude_command[1] = str(SCRIPTS_ROOT / "run_parallel_claude_code_experiment.py")
        claude_command.extend(["--workers", str(args.claude_workers)])
    if args.defer_infrastructure_retries:
        return run_deferred_pipeline(
            args, population_size, ruter_command, claude_command, summary_command,
            ruter_out, claude_out, control_root, ruter_env, claude_env, env,
        )
    run_stage("ruter", ruter_command, ruter_env, ruter_out, control_root)
    if run(summary_command, env, control_root / "ruter_summary.log") != 0:
        raise RuntimeError("RuTeR aggregation failed")
    assert_ruter_complete(ruter_out, population_size)
    run_stage("claude", claude_command, claude_env, claude_out, control_root)
    assert_claude_complete(claude_out, population_size)
    return True


def assert_claude_complete(claude_out: Path, population_size: int) -> None:
    summary = load_json(claude_out / "summary/claude_code_summary.json")
    if (
        summary.get("case_count") != population_size
        or summary.get("excluded_case_count", 0) != 0
        or summary.get("usage_complete_count") != population_size
        or summary.get("model_valid_count") != population_size
    ):
        raise RuntimeError("Claude Code finished with incomplete/mismatched cases")


def write_pending_api_failures(control_root: Path, ruter_out: Path, claude_out: Path) -> dict[str, list[dict[str, Any]]]:
    from run_token_cost_experiment import is_retryable_infrastructure_failure as ruter_bad
    from run_token_cost_experiment import infrastructure_failure_record
    from run_claude_code_experiment import is_retryable_infrastructure_failure as claude_bad

    pending: dict[str, list[dict[str, Any]]] = {"ruter": [], "claude": []}
    for stage, root in (("ruter", ruter_out), ("claude", claude_out)):
        for path in sorted(root.glob("cases/*/case_result.json")):
            bad = ruter_bad(path) if stage == "ruter" else claude_bad(load_json(path))
            if bad:
                pending[stage].append(infrastructure_failure_record(stage, path))
    payload = {
        "updated_at_utc": datetime.now(timezone.utc).isoformat(),
        "policy": "deferred_until_both_full_sweeps",
        "pending_counts": {stage: len(items) for stage, items in pending.items()},
        "cases": pending,
        "ordinary_method_failures_retried": False,
    }
    path = control_root / "pending_api_failures.json"
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)
    return pending


def run_deferred_pipeline(
    args: argparse.Namespace, population_size: int,
    ruter_command: list[str], claude_command: list[str], summary_command: list[str],
    ruter_out: Path, claude_out: Path, control_root: Path,
    ruter_env: dict[str, str], claude_env: dict[str, str], env: dict[str, str],
) -> bool:
    # No stopping, cooldown, or case-level retry in either initial sweep.
    def sweep(stage: str, command: list[str], stage_env: dict[str, str], root: Path, retry_pass: int = 0) -> None:
        command = [item for item in command if item not in {
            "--stop-on-infrastructure-failure", "--retry-infrastructure-failures",
        }]
        if retry_pass:
            command.append("--retry-infrastructure-failures")
        write_status(control_root, status="running", stage=stage,
                     phase="deferred_retry" if retry_pass else "full_sweep",
                     retry_pass=retry_pass, output=str(root),
                     worker_count=getattr(args, "claude_workers", 1) if stage == "claude" else 1,
                     infrastructure_failure_policy="record_and_continue")
        if run(command, stage_env, control_root / f"{stage}.log") != 0:
            raise RuntimeError(f"{stage} sweep exited unexpectedly; see {stage}.log")
        write_pending_api_failures(control_root, ruter_out, claude_out)

    def summarize_ruter() -> None:
        if run(summary_command, env, control_root / "ruter_summary.log") != 0:
            raise RuntimeError("RuTeR aggregation failed")

    write_pending_api_failures(control_root, ruter_out, claude_out)
    sweep("ruter", ruter_command, ruter_env, ruter_out)
    summarize_ruter()
    # This is independent of RuTeR's outcome, including cases still pending API.
    sweep("claude", claude_command, claude_env, claude_out)
    for retry_pass in range(1, args.deferred_retry_passes + 1):
        pending = write_pending_api_failures(control_root, ruter_out, claude_out)
        if not any(pending.values()):
            break
        print(f"deferred retry sweep {retry_pass}: "
              f"ruter={len(pending['ruter'])} claude={len(pending['claude'])}", flush=True)
        if pending["ruter"]:
            sweep("ruter", ruter_command, ruter_env, ruter_out, retry_pass)
            summarize_ruter()
        # Also refresh paired RuTeR metadata in already-valid Claude results.
        sweep("claude", claude_command, claude_env, claude_out, retry_pass)
    pending = write_pending_api_failures(control_root, ruter_out, claude_out)
    if any(pending.values()):
        return False
    assert_ruter_complete(ruter_out, population_size)
    assert_claude_complete(claude_out, population_size)
    return True


if __name__ == "__main__":
    raise SystemExit(main())
