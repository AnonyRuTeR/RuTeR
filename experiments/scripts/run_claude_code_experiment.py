#!/usr/bin/env python3
"""Run Claude Code on the frozen RuTeR sample and collect paired token usage."""

from __future__ import annotations

import argparse
import csv
import difflib
import getpass
import hashlib
import json
import os
import random
import re
import shutil
import signal
import statistics
import subprocess
import sys
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from run_token_cost_experiment import (
    load_json,
    prepare_workspace,
    record_infrastructure_failure,
    sanitized_environment,
    write_json,
)


SCRIPT_PATH = Path(__file__).resolve()
RUTER_REPO_ROOT = SCRIPT_PATH.parents[2]
PROJECT_ROOT = RUTER_REPO_ROOT
DEFAULT_PAIRED_MANIFEST = (
    PROJECT_ROOT
    / "experiments/artifacts/token_cost/ruter/experiment_manifest.json"
)
CONFIG_ROOT = PROJECT_ROOT / "experiments/configs/claude_code"
DEFAULT_SETTINGS = CONFIG_ROOT / "settings.json"
DEFAULT_PROMPT = CONFIG_ROOT / "repair_prompt.txt"
DEFAULT_VERIFY_HOOK = CONFIG_ROOT / "verify_repair.py"
DEFAULT_PROXY = SCRIPT_PATH.with_name("anthropic_usage_proxy.py")
DEFAULT_MODEL = "gemini-2.5-flash-nothinking"
DEFAULT_UPSTREAM = "https://4sapi.org"
BEGIN_MARKER = "// >>> TOKEN COST EXPERIMENT INJECTION BEGIN >>>"
END_MARKER = "// <<< TOKEN COST EXPERIMENT INJECTION END <<<"
USAGE_FIELDS = (
    "input_tokens",
    "cache_creation_input_tokens",
    "cache_read_input_tokens",
    "output_tokens",
)
PILOT_CASE_IDS = (
    # Same frozen n=200 sample: zero, low, median, and high RuTeR token cost.
    "case_0109_a2d5469e6c8c",
    "case_0038_2aaf3ebff17d",
    "case_0002_55fd0c07ea51",
    "case_0053_67ea3d48e993",
)


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def claude_version(binary: str) -> str | None:
    result = subprocess.run(
        [binary, "--version"], text=True, capture_output=True, check=False
    )
    value = (result.stdout or result.stderr).strip()
    return value if value else None


def run_command(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    timeout: int,
) -> tuple[int | None, str, str, bool, float]:
    started = time.monotonic()
    proc = subprocess.Popen(
        command,
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    timed_out = False
    try:
        stdout, stderr = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(proc.pid, signal.SIGTERM)
        try:
            stdout, stderr = proc.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            stdout, stderr = proc.communicate()
    except BaseException:
        # Workers own a separate Claude/Cargo process group. Do not orphan it
        # when a parallel run is interrupted or switched to another worker count.
        if proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.communicate()
        raise
    return (
        proc.returncode,
        stdout or "",
        stderr or "",
        timed_out,
        round(time.monotonic() - started, 3),
    )


def cargo_check(
    workspace: Path,
    env: dict[str, str],
    timeout: int,
) -> tuple[int | None, str, str, bool, float, int]:
    result = run_command(
        ["cargo", "check", "--tests", "--message-format=json"],
        cwd=workspace,
        env=env,
        timeout=timeout,
    )
    errors = 0
    for line in result[1].splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        message = event.get("message") if isinstance(event, dict) else None
        if (
            event.get("reason") == "compiler-message"
            and isinstance(message, dict)
            and message.get("level") == "error"
        ):
            errors += 1
    return (*result, errors)


def snapshot_workspace(workspace: Path) -> dict[str, str]:
    snapshot: dict[str, str] = {}
    for path in sorted(workspace.rglob("*")):
        if not path.is_file():
            continue
        relative = path.relative_to(workspace)
        if any(part in {"target", ".git"} for part in relative.parts):
            continue
        snapshot[relative.as_posix()] = sha256_file(path)
    return snapshot


def injected_parts(source: str) -> tuple[str, str, str] | None:
    if source.count(BEGIN_MARKER) != 1 or source.count(END_MARKER) != 1:
        return None
    prefix, remainder = source.split(BEGIN_MARKER, 1)
    injected, suffix = remainder.split(END_MARKER, 1)
    return prefix, injected, suffix


def count_pattern(pattern: str, source: str) -> int:
    return len(re.findall(pattern, source, flags=re.MULTILINE))


def validate_repair(
    *,
    workspace: Path,
    source_path: str,
    original_source: str,
    original_snapshot: dict[str, str],
) -> dict[str, Any]:
    target = workspace / source_path
    final_source = target.read_text(encoding="utf-8") if target.is_file() else ""
    original_parts = injected_parts(original_source)
    final_parts = injected_parts(final_source)
    violations: list[str] = []
    if original_parts is None or final_parts is None:
        violations.append("injection markers were removed or duplicated")
        original_block = ""
        final_block = ""
    else:
        original_prefix, original_block, original_suffix = original_parts
        final_prefix, final_block, final_suffix = final_parts
        if final_prefix != original_prefix or final_suffix != original_suffix:
            violations.append("code outside the injected block changed")

    final_snapshot = snapshot_workspace(workspace)
    changed_other_files = sorted(
        path
        for path in set(original_snapshot) | set(final_snapshot)
        if path != source_path and original_snapshot.get(path) != final_snapshot.get(path)
    )
    if changed_other_files:
        violations.append("files outside the injection target changed")

    test_pattern = r"#\s*\[\s*test\s*\]"
    assert_pattern = r"\b(?:debug_)?assert(?:_eq|_ne)?\s*!"
    ignore_pattern = r"#\s*\[\s*ignore(?:\s*\([^]]*\))?\s*\]"
    cfg_pattern = r"#\s*\[\s*cfg\s*\("
    function_pattern = r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("
    original_tests = count_pattern(test_pattern, original_block)
    final_tests = count_pattern(test_pattern, final_block)
    original_assertions = count_pattern(assert_pattern, original_block)
    final_assertions = count_pattern(assert_pattern, final_block)
    original_functions = set(re.findall(function_pattern, original_block))
    final_functions = set(re.findall(function_pattern, final_block))
    if final_tests < original_tests:
        violations.append("one or more #[test] attributes were removed")
    if final_assertions < original_assertions:
        violations.append("one or more assertion macros were removed")
    if not original_functions.issubset(final_functions):
        violations.append("one or more injected functions were removed or renamed")
    if count_pattern(ignore_pattern, final_block) > count_pattern(
        ignore_pattern, original_block
    ):
        violations.append("an #[ignore] attribute was added")
    if count_pattern(cfg_pattern, final_block) > count_pattern(cfg_pattern, original_block):
        violations.append("a conditional-compilation attribute was added")

    return {
        "scope_valid": not changed_other_files
        and original_parts is not None
        and final_parts is not None
        and original_parts[0] == final_parts[0]
        and original_parts[2] == final_parts[2],
        "integrity_valid": not violations,
        "violations": violations,
        "changed_other_files": changed_other_files,
        "original_test_count": original_tests,
        "final_test_count": final_tests,
        "original_assertion_count": original_assertions,
        "final_assertion_count": final_assertions,
        "original_function_count": len(original_functions),
        "final_function_count": len(final_functions),
        "final_source": final_source,
    }


def wait_for_proxy(port: int, process: subprocess.Popen[str], timeout: float = 10.0) -> None:
    deadline = time.monotonic() + timeout
    url = f"http://127.0.0.1:{port}/healthz"
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"usage proxy exited with code {process.returncode}")
        try:
            with urllib.request.urlopen(url, timeout=0.5) as response:
                if response.status == 200:
                    return
        except Exception as exc:  # pragma: no cover - startup race
            last_error = exc
        time.sleep(0.1)
    raise RuntimeError(f"usage proxy did not become ready: {last_error}")


def start_proxy(
    *,
    proxy_script: Path,
    upstream: str,
    port: int,
    log_path: Path,
    api_key: str,
    cwd: Path,
) -> subprocess.Popen[str]:
    env = dict(os.environ)
    env["CLAUDE_EXPERIMENT_API_KEY"] = api_key
    process = subprocess.Popen(
        [
            sys.executable,
            str(proxy_script),
            "--upstream",
            upstream,
            "--port",
            str(port),
            "--log",
            str(log_path),
        ],
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    try:
        wait_for_proxy(port, process)
    except Exception:
        os.killpg(process.pid, signal.SIGTERM)
        stdout, stderr = process.communicate(timeout=10)
        raise RuntimeError(f"proxy startup failed: stdout={stdout!r} stderr={stderr!r}")
    return process


def stop_proxy(process: subprocess.Popen[str]) -> tuple[str, str]:
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGTERM)
    try:
        return process.communicate(timeout=10)
    except subprocess.TimeoutExpired:  # pragma: no cover - defensive shutdown
        os.killpg(process.pid, signal.SIGKILL)
        return process.communicate()


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    if not path.is_file():
        return records
    for line in path.read_text(encoding="utf-8").splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(record, dict):
            records.append(record)
    return records


def proxy_usage(records: list[dict[str, Any]]) -> dict[str, Any]:
    message_records = [
        record for record in records if record.get("request_kind") == "messages"
    ]
    totals = {field: 0 for field in USAGE_FIELDS}
    for record in message_records:
        usage = record.get("usage")
        if not isinstance(usage, dict):
            continue
        for field in USAGE_FIELDS:
            try:
                totals[field] += int(usage.get(field) or 0)
            except (TypeError, ValueError):
                pass
    totals["gross_tokens"] = sum(totals[field] for field in USAGE_FIELDS)
    provider_totals = {
        "provider_input_tokens": 0,
        "provider_output_tokens": 0,
        "provider_total_tokens": 0,
    }
    provider_complete = True
    for record in message_records:
        original = record.get("upstream_openai_usage")
        if not isinstance(original, dict):
            provider_complete = False
            continue
        try:
            provider_totals["provider_input_tokens"] += int(original["prompt_tokens"])
            provider_totals["provider_output_tokens"] += int(
                original["completion_tokens"]
            )
            provider_totals["provider_total_tokens"] += int(original["total_tokens"])
        except (KeyError, TypeError, ValueError):
            provider_complete = False
    if provider_complete and message_records:
        totals["total_tokens"] = provider_totals["provider_total_tokens"]
        primary_source = "billing_usage.openai_usage.total_tokens"
    else:
        totals["total_tokens"] = totals["gross_tokens"]
        primary_source = "anthropic_surface_usage_sum"
    returned_models = sorted(
        {str(record["returned_model"]) for record in message_records if record.get("returned_model")}
    )
    return {
        **totals,
        **provider_totals,
        "primary_token_source": primary_source,
        "provider_usage_complete": provider_complete and bool(message_records),
        "request_count": len(message_records),
        "successful_request_count": sum(
            isinstance(record.get("status"), int) and int(record["status"]) < 400
            for record in message_records
        ),
        "usage_complete": bool(message_records)
        and all(record.get("usage_complete") is True for record in message_records),
        "returned_models": returned_models,
    }


def final_claude_result(stdout: str) -> dict[str, Any] | None:
    final: dict[str, Any] | None = None
    for line in stdout.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(event, dict) and event.get("type") == "result":
            final = event
    return final


def result_usage(result: dict[str, Any] | None) -> dict[str, int] | None:
    usage = result.get("usage") if isinstance(result, dict) else None
    if not isinstance(usage, dict):
        return None
    normalized = {}
    for field in USAGE_FIELDS:
        try:
            normalized[field] = int(usage.get(field) or 0)
        except (TypeError, ValueError):
            return None
    normalized["gross_tokens"] = sum(normalized.values())
    return normalized


def public_result_metadata(result: dict[str, Any] | None) -> dict[str, Any] | None:
    if not isinstance(result, dict):
        return None
    # Exclude the assistant's textual result; retain only accounting metadata.
    return {
        key: result.get(key)
        for key in (
            "subtype",
            "is_error",
            "duration_ms",
            "duration_api_ms",
            "num_turns",
            "total_cost_usd",
            "session_id",
            "usage",
            "modelUsage",
        )
        if key in result
    }


def model_valid(returned_models: list[str], requested_model: str) -> bool:
    allowed = {requested_model}
    if requested_model == "gemini-2.5-flash-nothinking":
        allowed.add("gemini-2.5-flash")
    return bool(returned_models) and set(returned_models).issubset(allowed)


def is_retryable_infrastructure_failure(result: dict[str, Any]) -> bool:
    """Identify cases where no complete, model-valid provider run was observed."""
    usage = result.get("usage")
    if not isinstance(usage, dict):
        return result.get("status") in {"claude_failed", "infrastructure_failed"}
    return (
        result.get("status") in {"claude_failed", "infrastructure_failed"}
        and (
            usage.get("provider_usage_complete") is not True
            or result.get("model_valid") is not True
            or int(usage.get("successful_request_count") or 0)
            < int(usage.get("request_count") or 0)
        )
    )


def load_ruter_rows(paired_manifest_path: Path) -> dict[str, dict[str, str]]:
    csv_path = paired_manifest_path.parent / "summary/token_usage_cases.csv"
    if not csv_path.is_file():
        return {}
    with csv_path.open(newline="", encoding="utf-8") as handle:
        return {row["case_id"]: row for row in csv.DictReader(handle)}


def paired_ruter_values(row: dict[str, str]) -> dict[str, Any]:
    # In the deferred first sweep, RuTeR can still have incomplete API cases.
    # Their partial tokens/outcomes must not be exposed as a valid paired result.
    valid = bool(row) and row.get("analysis_included", "True") == "True"
    return {
        "ruter_total_tokens": int(row.get("total_tokens") or 0) if valid else None,
        "ruter_strict_success": row.get("strict_success") == "True" if valid else None,
    }


def choose_cases(
    all_cases: list[dict[str, Any]],
    *,
    pilot: bool,
    requested: list[str],
    limit: int | None,
    sample_size: int | None,
    seed: int,
) -> list[dict[str, Any]]:
    if pilot:
        wanted = set(PILOT_CASE_IDS)
        selected = [case for case in all_cases if case.get("case_id") in wanted]
        if len(selected) != len(wanted):
            missing = wanted - {str(case.get("case_id")) for case in selected}
            raise ValueError(f"pilot case missing from paired manifest: {sorted(missing)}")
        order = {case_id: index for index, case_id in enumerate(PILOT_CASE_IDS)}
        return sorted(selected, key=lambda case: order[str(case["case_id"])])
    if requested:
        wanted = set(requested)
        selected = [
            case
            for case in all_cases
            if str(case.get("case_id")) in wanted
            or str(case.get("attempt_uid")) in wanted
        ]
        found = {
            value
            for case in selected
            for value in (str(case.get("case_id")), str(case.get("attempt_uid")))
            if value in wanted
        }
        if found != wanted:
            raise ValueError(f"case not found in paired manifest: {sorted(wanted - found)}")
        return selected
    if limit is not None:
        if limit <= 0:
            raise ValueError("--limit must be positive")
        return all_cases[:limit]
    if sample_size is not None:
        if sample_size <= 0 or sample_size > len(all_cases):
            raise ValueError(
                f"--sample-size must be between 1 and {len(all_cases)}"
            )
        # A second-stage SRS from the frozen paired frame. The fixed inclusion
        # probability preserves the first-stage design after multiplying weights.
        return random.Random(seed).sample(all_cases, sample_size)
    return list(all_cases)


def compiler_error_text(cargo_stdout: str) -> str:
    rendered: list[str] = []
    for line in cargo_stdout.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        message = event.get("message") if isinstance(event, dict) else None
        if (
            event.get("reason") == "compiler-message"
            and isinstance(message, dict)
            and message.get("level") == "error"
        ):
            text = message.get("rendered") or message.get("message")
            if text:
                rendered.append(str(text).strip())
    return "\n\n".join(rendered) or "cargo check --tests exited nonzero"


def render_prompt(
    template: str,
    injection: dict[str, Any],
    original_source: str,
    initial_cargo_stdout: str,
) -> str:
    parts = injected_parts(original_source)
    failed_test = parts[1].strip() if parts else "unavailable"
    return template.format(
        source_path=injection["src_path"],
        module_name=injection.get("effective_mod_name") or "unknown",
        failed_test=failed_test,
        compiler_errors=compiler_error_text(initial_cargo_stdout),
    )


def experiment_environment(cargo_target_dir: Path, port: int, model: str) -> dict[str, str]:
    env = sanitized_environment(cargo_target_dir)
    for key in list(env):
        if key.startswith("ANTHROPIC_") or key.startswith("CLAUDE_CODE_"):
            env.pop(key, None)
    # Claude receives only a loopback placeholder. The real credential exists only
    # in the proxy subprocess environment.
    env.update(
        {
            "ANTHROPIC_BASE_URL": f"http://127.0.0.1:{port}",
            "ANTHROPIC_AUTH_TOKEN": "local-token-meter-proxy",
            "ANTHROPIC_API_KEY": "local-token-meter-proxy",
            "ANTHROPIC_MODEL": model,
            "ANTHROPIC_DEFAULT_HAIKU_MODEL": model,
            "ANTHROPIC_DEFAULT_SONNET_MODEL": model,
            "ANTHROPIC_DEFAULT_OPUS_MODEL": model,
            "CLAUDE_CODE_SUBAGENT_MODEL": model,
            "NO_PROXY": "127.0.0.1,localhost",
            "no_proxy": "127.0.0.1,localhost",
            # Old subject crates emit thousands of characters of modern-rustc
            # warnings. They obscure the actual error in Claude's tool result but
            # do not affect the pass/fail criterion.
            "RUSTFLAGS": "-Awarnings",
        }
    )
    env.pop("CLAUDE_EXPERIMENT_API_KEY", None)
    env.pop("RUTER_LLM_API_KEY", None)
    env.pop("FULL_PAIRED_API_KEY", None)
    return env


def percentile(values: list[float], probability: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    position = (len(ordered) - 1) * probability
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    fraction = position - lower
    return ordered[lower] * (1 - fraction) + ordered[upper] * fraction


def weighted_mean(results: list[dict[str, Any]], value_fn: Any) -> float | None:
    denominator = sum(float(result.get("sampling_weight") or 0.0) for result in results)
    if not denominator:
        return None
    return sum(
        float(result.get("sampling_weight") or 0.0) * float(value_fn(result))
        for result in results
    ) / denominator


def bootstrap_weighted_mean_ci(
    results: list[dict[str, Any]], value_fn: Any, *, seed: int, replicates: int = 5000
) -> list[float] | None:
    if len(results) < 2:
        return None
    rng = random.Random(seed)
    estimates: list[float] = []
    for _ in range(replicates):
        sample = [results[rng.randrange(len(results))] for _ in results]
        estimate = weighted_mean(sample, value_fn)
        if estimate is not None:
            estimates.append(estimate)
    low = percentile(estimates, 0.025)
    high = percentile(estimates, 0.975)
    return [low, high] if low is not None and high is not None else None


def summarize(
    out_root: Path,
    complete_paired_sample: bool,
    allow_weighted_estimates: bool,
    analysis_seed: int,
) -> dict[str, Any]:
    executed_results = [
        load_json(path, {}) for path in sorted(out_root.glob("cases/*/case_result.json"))
    ]
    executed_results = [result for result in executed_results if isinstance(result, dict)]
    results = [
        result for result in executed_results
        if result.get("eligible") is True
        and (result.get("usage") or {}).get("provider_usage_complete") is True
        and (result.get("usage") or {}).get("usage_complete") is True
        and result.get("model_valid") is True
    ]
    excluded = [result for result in executed_results if result not in results]
    complete_paired_sample = complete_paired_sample and not excluded
    allow_weighted_estimates = allow_weighted_estimates and not excluded
    token_values = [int((result.get("usage") or {}).get("total_tokens") or 0) for result in results]
    input_values = [int((result.get("usage") or {}).get("provider_input_tokens") or 0) for result in results]
    output_values = [int((result.get("usage") or {}).get("provider_output_tokens") or 0) for result in results]
    successes = sum(result.get("strict_success") is True for result in results)
    surface_crosschecks = sum(
        isinstance(result.get("claude_result_usage"), dict)
        and all(
            int(result["claude_result_usage"].get(field) or 0)
            == int(
                (result.get("usage") or {}).get(
                    "provider_input_tokens"
                    if field == "input_tokens"
                    else "provider_output_tokens"
                    if field == "output_tokens"
                    else field
                )
                or 0
            )
            for field in USAGE_FIELDS
        )
        for result in results
    )
    status_counts: dict[str, int] = {}
    outcome_counts: dict[str, int] = {}
    for result in executed_results:
        status = str(result.get("status") or "unknown")
        status_counts[status] = status_counts.get(status, 0) + 1
    for result in results:
        if result.get("ruter_strict_success") is None:
            continue
        cc = "success" if result.get("strict_success") is True else "failure"
        ruter = "success" if result.get("ruter_strict_success") is True else "failure"
        outcome = f"claude_{cc}__ruter_{ruter}"
        outcome_counts[outcome] = outcome_counts.get(outcome, 0) + 1
    summary: dict[str, Any] = {
        "schema_version": "1",
        "created_at_utc": utc_now(),
        "case_count": len(executed_results),
        "analyzed_case_count": len(results),
        "excluded_case_count": len(excluded),
        "excluded_cases": [
            {"case_id": result.get("case_id"), "status": result.get("status")}
            for result in excluded
        ],
        "complete_paired_sample": complete_paired_sample,
        "token_definition": (
            "primary total = sum of 4sapi billing_usage.openai_usage.total_tokens "
            "over all /v1/messages responses, including failed repairs; cache-read "
            "tokens are already included in OpenAI prompt_tokens and are not added twice"
        ),
        "sample_total_tokens": sum(token_values),
        "arithmetic_mean_tokens_per_case": statistics.fmean(token_values) if token_values else None,
        "median_tokens_per_case": statistics.median(token_values) if token_values else None,
        "p95_tokens_per_case": percentile([float(value) for value in token_values], 0.95),
        "arithmetic_mean_input_tokens": statistics.fmean(input_values) if input_values else None,
        "arithmetic_mean_output_tokens": statistics.fmean(output_values) if output_values else None,
        "strict_success_count": successes,
        "strict_success_rate": successes / len(results) if results else None,
        "tokens_per_strict_success": sum(token_values) / successes if successes else None,
        "usage_complete_count": sum(
            (result.get("usage") or {}).get("usage_complete") is True for result in executed_results
        ),
        "model_valid_count": sum(result.get("model_valid") is True for result in executed_results),
        "status_counts": status_counts,
        "paired_outcome_counts": outcome_counts,
        "claude_result_present_count": sum(
            isinstance(result.get("claude_result_usage"), dict) for result in results
        ),
        "surface_usage_crosscheck_count": surface_crosschecks,
        "claude_llm_invoked_count": sum(
            int((result.get("usage") or {}).get("request_count") or 0) > 0
            for result in results
        ),
        "ruter_llm_invoked_count": sum(
            int(result.get("ruter_total_tokens") or 0) > 0 for result in results
        ),
    }
    paired = [result for result in results if result.get("ruter_total_tokens") is not None]
    if paired:
        ruter_values = [int(result["ruter_total_tokens"]) for result in paired]
        summary["paired"] = {
            "case_count": len(paired),
            "claude_mean_tokens": statistics.fmean(
                int((result.get("usage") or {}).get("total_tokens") or 0)
                for result in paired
            ),
            "ruter_mean_tokens": statistics.fmean(ruter_values),
            "mean_token_difference_claude_minus_ruter": statistics.fmean(
                int((result.get("usage") or {}).get("total_tokens") or 0)
                - int(result["ruter_total_tokens"])
                for result in paired
            ),
            "aggregate_token_ratio_claude_over_ruter": (
                sum(
                    int((result.get("usage") or {}).get("total_tokens") or 0)
                    for result in paired
                )
                / sum(ruter_values)
                if sum(ruter_values)
                else None
            ),
        }
    if allow_weighted_estimates and results:
        token_fn = lambda result: int(
            (result.get("usage") or {}).get("total_tokens") or 0
        )
        success_fn = lambda result: 1 if result.get("strict_success") is True else 0
        summary["weighted_mean_tokens_per_case"] = weighted_mean(results, token_fn)
        summary["weighted_mean_tokens_ci95_case_bootstrap"] = bootstrap_weighted_mean_ci(
            results, token_fn, seed=analysis_seed
        )
        summary["weighted_strict_success_rate"] = weighted_mean(results, success_fn)
        weighted_successes = sum(
            float(result.get("sampling_weight") or 0.0) * success_fn(result)
            for result in results
        )
        weighted_tokens = sum(
            float(result.get("sampling_weight") or 0.0) * token_fn(result)
            for result in results
        )
        summary["weighted_tokens_per_strict_success"] = (
            weighted_tokens / weighted_successes if weighted_successes else None
        )
        summary["weighted_estimate_note"] = (
            "Hajek weighted point estimates; the percentile case-bootstrap CI is "
            "descriptive and does not fully reproduce the two-stage sampling design"
        )
        if paired:
            delta_fn = lambda result: token_fn(result) - int(
                result.get("ruter_total_tokens") or 0
            )
            ruter_fn = lambda result: int(result.get("ruter_total_tokens") or 0)
            weighted_cc = weighted_mean(paired, token_fn)
            weighted_ruter = weighted_mean(paired, ruter_fn)
            summary["paired"].update(
                {
                    "weighted_claude_mean_tokens": weighted_cc,
                    "weighted_ruter_mean_tokens": weighted_ruter,
                    "weighted_mean_token_difference_claude_minus_ruter": weighted_mean(
                        paired, delta_fn
                    ),
                    "weighted_mean_difference_ci95_case_bootstrap": bootstrap_weighted_mean_ci(
                        paired, delta_fn, seed=analysis_seed + 1
                    ),
                    "weighted_aggregate_token_ratio_claude_over_ruter": (
                        weighted_cc / weighted_ruter
                        if weighted_cc is not None
                        and weighted_ruter not in {None, 0.0}
                        else None
                    ),
                }
            )
    write_json(out_root / "summary/claude_code_summary.json", summary)
    markdown = [
        "# Claude Code token-cost summary",
        "",
        f"- Cases: {summary['case_count']}",
        f"- Analyzed cases: {summary['analyzed_case_count']}",
        f"- Excluded cases (reproduction/API/model validation): {summary['excluded_case_count']}",
        f"- Total tokens: {summary['sample_total_tokens']}",
        f"- Arithmetic mean tokens/case: {summary['arithmetic_mean_tokens_per_case']}",
        f"- Median tokens/case: {summary['median_tokens_per_case']}",
        f"- P95 tokens/case: {summary['p95_tokens_per_case']}",
        f"- Strict repairs: {successes}/{len(results)}",
        f"- Complete usage records: {summary['usage_complete_count']}/{len(executed_results)}",
        f"- Verified Gemini model records: {summary['model_valid_count']}/{len(executed_results)}",
        f"- Claude result usage crosschecks (component-wise): {summary['surface_usage_crosscheck_count']}/{len(results)}",
    ]
    if allow_weighted_estimates and summary.get("weighted_mean_tokens_per_case") is not None:
        markdown.extend(
            [
                f"- Stratified weighted mean tokens/case: {summary['weighted_mean_tokens_per_case']}",
                f"- Stratified weighted success rate: {summary['weighted_strict_success_rate']}",
            ]
        )
    else:
        markdown.append("- Inferential weighted estimates: omitted for this pilot/subset")
    markdown.extend(["", summary["token_definition"], ""])
    md_path = out_root / "summary/claude_code_summary.md"
    md_path.parent.mkdir(parents=True, exist_ok=True)
    md_path.write_text("\n".join(markdown), encoding="utf-8")
    return summary


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--paired-manifest", default=str(DEFAULT_PAIRED_MANIFEST))
    parser.add_argument("--out", required=True)
    parser.add_argument("--settings", default=str(DEFAULT_SETTINGS))
    parser.add_argument("--prompt-template", default=str(DEFAULT_PROMPT))
    parser.add_argument("--verify-hook", default=str(DEFAULT_VERIFY_HOOK))
    parser.add_argument("--proxy-script", default=str(DEFAULT_PROXY))
    parser.add_argument("--upstream", default=DEFAULT_UPSTREAM)
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--claude-bin", default="claude")
    parser.add_argument("--proxy-port", type=int, default=18766)
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--pilot", action="store_true")
    selection.add_argument("--case-id", action="append", default=[])
    selection.add_argument("--case-list", help="JSON array of frozen case IDs assigned by a coordinator")
    selection.add_argument("--limit", type=int)
    selection.add_argument("--sample-size", type=int)
    parser.add_argument("--seed", type=int, default=20260924)
    parser.add_argument("--max-turns", type=int, default=20)
    parser.add_argument("--timeout-secs", type=int, default=900)
    parser.add_argument("--cargo-timeout-secs", type=int, default=600)
    parser.add_argument("--cargo-target-root")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--discard-workspaces", action="store_true")
    parser.add_argument("--resume", action="store_true")
    parser.add_argument("--reuse-only", action="store_true", help="Initialize metadata and copy reusable results without new repairs")
    parser.add_argument("--completion-markers", action="store_true", help="Publish a marker only after all case artifacts are finalized")
    parser.add_argument(
        "--retry-infrastructure-failures",
        action="store_true",
        help=(
            "With --resume, archive and rerun cases whose provider usage is "
            "incomplete or whose returned model was not validated"
        ),
    )
    parser.add_argument(
        "--reuse-results-from",
        help="Reuse byte-identical-protocol cases from an earlier experiment",
    )
    parser.add_argument(
        "--stop-on-infrastructure-failure",
        action="store_true",
        help="Save results/summary and exit 3 on incomplete usage or wrong model",
    )
    return parser


def mark_case_complete(case_root: Path, enabled: bool) -> None:
    if enabled:
        path = case_root / "runner_complete.json"
        temporary = path.with_suffix(".tmp")
        write_json(temporary, {"case_id": case_root.name, "completed_at_utc": utc_now()})
        temporary.replace(path)


def main() -> int:
    args = build_parser().parse_args()
    if args.completion_markers:
        def interrupted(_signum: int, _frame: Any) -> None:
            raise KeyboardInterrupt("worker interrupted")
        signal.signal(signal.SIGTERM, interrupted)
    requested = args.case_id
    if args.case_list:
        requested = load_json(Path(args.case_list), [])
        if not isinstance(requested, list) or not requested or not all(isinstance(x, str) for x in requested):
            raise SystemExit("--case-list must contain a nonempty JSON array of case IDs")
        if len(set(requested)) != len(requested):
            raise SystemExit("--case-list contains duplicate case IDs")
    paired_manifest_path = Path(args.paired_manifest).resolve()
    paired_manifest = load_json(paired_manifest_path, {})
    all_cases = paired_manifest.get("cases") if isinstance(paired_manifest, dict) else None
    if not isinstance(all_cases, list) or not all_cases:
        raise SystemExit(f"paired manifest contains no cases: {paired_manifest_path}")
    try:
        cases = choose_cases(
            all_cases,
            pilot=args.pilot,
            requested=requested,
            limit=args.limit,
            sample_size=args.sample_size,
            seed=args.seed,
        )
    except ValueError as error:
        raise SystemExit(str(error)) from error

    frozen_manifest_path = Path(str(paired_manifest["manifest"])).resolve()
    clean_crates_root = Path(str(paired_manifest["clean_crates_root"])).resolve()
    frozen_manifest = load_json(frozen_manifest_path, {})
    attempts = frozen_manifest.get("attempts") if isinstance(frozen_manifest, dict) else None
    if not isinstance(attempts, list):
        raise SystemExit(f"frozen manifest contains no attempts: {frozen_manifest_path}")
    attempts_by_uid = {str(attempt["attempt_uid"]): attempt for attempt in attempts}

    out_root = Path(args.out).resolve()
    if out_root.exists() and any(out_root.iterdir()) and not args.resume:
        raise SystemExit(f"output directory is not empty; use --resume: {out_root}")
    out_root.mkdir(parents=True, exist_ok=True)
    settings_path = Path(args.settings).resolve()
    prompt_path = Path(args.prompt_template).resolve()
    verify_hook_path = Path(args.verify_hook).resolve()
    proxy_script = Path(args.proxy_script).resolve()
    for required in (settings_path, prompt_path, verify_hook_path, proxy_script):
        if not required.is_file():
            raise SystemExit(f"required experiment file not found: {required}")
    prompt_template = prompt_path.read_text(encoding="utf-8")
    cargo_target_root = (
        Path(args.cargo_target_root).resolve()
        if args.cargo_target_root
        else out_root / "_cargo_target"
    )
    ruter_rows = load_ruter_rows(paired_manifest_path)

    api_key: str | None = None
    if not args.prepare_only and not args.reuse_only:
        api_key = os.environ.get("CLAUDE_EXPERIMENT_API_KEY")
        if not api_key and sys.stdin.isatty():
            api_key = getpass.getpass("4sapi key (not stored): ")
        if not api_key:
            raise SystemExit(
                "CLAUDE_EXPERIMENT_API_KEY must be set (or run interactively to enter it)"
            )

    cases = [dict(case) for case in cases]
    if args.sample_size is not None:
        second_stage_multiplier = len(all_cases) / len(cases)
        for case in cases:
            parent_weight = float(case.get("sampling_weight") or 0.0)
            case["parent_sampling_weight"] = parent_weight
            case["sampling_weight"] = parent_weight * second_stage_multiplier

    complete_paired_sample = (
        len(cases) == len(all_cases)
        and {str(case["attempt_uid"]) for case in cases}
        == {str(case["attempt_uid"]) for case in all_cases}
    )
    allow_weighted_estimates = complete_paired_sample or args.sample_size is not None
    reuse_root = Path(args.reuse_results_from).resolve() if args.reuse_results_from else None
    reusable: dict[str, Path] = {}
    if reuse_root:
        reuse_manifest = load_json(reuse_root / "experiment_manifest.json", {})
        expected_reuse = {
            "claude_version": claude_version(args.claude_bin),
            "model_requested": args.model,
            "upstream": args.upstream,
            "settings_sha256": sha256_file(settings_path),
            "prompt_sha256": sha256_file(prompt_path),
            "verification_hook_sha256": sha256_file(verify_hook_path),
            "proxy_script_sha256": sha256_file(proxy_script),
            "max_turns": args.max_turns,
            "timeout_secs": args.timeout_secs,
        }
        observed_reuse = {
            "claude_version": reuse_manifest.get("claude_version"),
            "model_requested": reuse_manifest.get("model_requested"),
            "upstream": reuse_manifest.get("upstream"),
            "settings_sha256": reuse_manifest.get("settings_sha256"),
            "prompt_sha256": reuse_manifest.get("prompt_sha256"),
            "verification_hook_sha256": reuse_manifest.get(
                "verification_hook_sha256"
            ),
            "proxy_script_sha256": reuse_manifest.get("proxy_script_sha256"),
            "max_turns": (reuse_manifest.get("configuration") or {}).get("max_turns"),
            "timeout_secs": (reuse_manifest.get("configuration") or {}).get(
                "timeout_secs"
            ),
        }
        if observed_reuse != expected_reuse:
            raise SystemExit(
                "reuse experiment protocol mismatch: "
                f"expected={expected_reuse} observed={observed_reuse}"
            )
        for old_result in reuse_root.glob("cases/*/case_result.json"):
            data = load_json(old_result, {})
            if (
                isinstance(data, dict)
                and data.get("attempt_uid")
                and not is_retryable_infrastructure_failure(data)
            ):
                reusable[str(data["attempt_uid"])] = old_result
    experiment = {
        "schema_version": "1",
        "created_at_utc": utc_now(),
        "design": "paired Claude Code comparison on exact frozen RuTeR attempts",
        "paired_ruter_manifest": str(paired_manifest_path),
        "paired_ruter_manifest_sha256": sha256_file(paired_manifest_path),
        "frozen_attempt_manifest": str(frozen_manifest_path),
        "clean_crates_root": str(clean_crates_root),
        "claude_version": claude_version(args.claude_bin),
        "model_requested": args.model,
        "upstream": args.upstream,
        "api_key_source": "CLAUDE_EXPERIMENT_API_KEY or hidden interactive input",
        "api_key_persisted": False,
        "settings": str(settings_path),
        "settings_sha256": sha256_file(settings_path),
        "prompt_template": str(prompt_path),
        "prompt_sha256": sha256_file(prompt_path),
        "verification_hook": str(verify_hook_path),
        "verification_hook_sha256": sha256_file(verify_hook_path),
        "proxy_script": str(proxy_script),
        "proxy_script_sha256": sha256_file(proxy_script),
        "selection": {
            "mode": "pilot" if args.pilot else "explicit" if args.case_id or args.case_list else "prefix" if args.limit else "paired_frame_srs" if args.sample_size else "full_paired_sample",
            "seed": args.seed,
            "source_case_count": len(all_cases),
            "case_count": len(cases),
            "complete_paired_sample": complete_paired_sample,
            "second_stage_inclusion_probability": (
                len(cases) / len(all_cases) if args.sample_size else 1.0
            ),
            "weighted_estimates_enabled": allow_weighted_estimates,
        },
        "reuse_results_from": str(reuse_root) if reuse_root else None,
        "configuration": {
            "max_turns": args.max_turns,
            "timeout_secs": args.timeout_secs,
            "cargo_timeout_secs": args.cargo_timeout_secs,
            "proxy_port": args.proxy_port,
            "cargo_target_root": str(cargo_target_root),
            "discard_workspaces": args.discard_workspaces,
            "prepare_only": args.prepare_only,
            "claude_output_format": "stream-json",
            "permission_mode": "dontAsk",
            "settings_sources": "project only",
        },
        "cases": cases,
    }
    write_json(out_root / "experiment_manifest.json", experiment)

    for index, case in enumerate(cases, start=1):
        case_name = str(case["case_id"])
        case_root = out_root / "cases" / case_name
        result_path = case_root / "case_result.json"
        if args.resume and result_path.is_file():
            existing_result = load_json(result_path, {})
            should_retry = (
                args.retry_infrastructure_failures
                and isinstance(existing_result, dict)
                and is_retryable_infrastructure_failure(existing_result)
            )
            if not should_retry:
                if is_retryable_infrastructure_failure(existing_result):
                    record_infrastructure_failure(out_root, "claude", result_path)
                paired_values = paired_ruter_values(ruter_rows.get(case_name, {}))
                if any(existing_result.get(k) != v for k, v in paired_values.items()):
                    existing_result.update(paired_values)
                    write_json(result_path, existing_result)
                print(f"[{index}/{len(cases)}] resume skip {case_name}", flush=True)
                mark_case_complete(case_root, args.completion_markers)
                continue
            print(
                f"[{index}/{len(cases)}] retry infrastructure failure {case_name}",
                flush=True,
            )
        if case_root.exists():
            if args.resume:
                archive = out_root / "_aborted_cases" / (
                    f"{case_name}_{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
                )
                archive.parent.mkdir(parents=True, exist_ok=True)
                shutil.move(str(case_root), str(archive))
            else:
                raise SystemExit(f"case directory already exists: {case_root}")
        reusable_result = reusable.get(str(case["attempt_uid"]))
        if reusable_result and not args.prepare_only:
            shutil.copytree(reusable_result.parent, case_root)
            reused = load_json(case_root / "case_result.json", {})
            reused.update(case)
            if "parent_sampling_weight" not in case:
                reused.pop("parent_sampling_weight", None)
            reused.update(paired_ruter_values(ruter_rows.get(case_name, {})))
            reused["reused_from"] = str(reusable_result)
            write_json(case_root / "case_result.json", reused)
            print(
                f"[{index}/{len(cases)}] reused {case_name} from {reuse_root}",
                flush=True,
            )
            mark_case_complete(case_root, args.completion_markers)
            continue
        if args.reuse_only:
            continue
        case_root.mkdir(parents=True)
        attempt = attempts_by_uid.get(str(case["attempt_uid"]))
        if attempt is None:
            write_json(result_path, {**case, "status": "prepare_failed", "error": "attempt UID missing"})
            mark_case_complete(case_root, args.completion_markers)
            continue

        print(f"[{index}/{len(cases)}] prepare {case_name} crate={case['crate']}", flush=True)
        try:
            workspace, injection = prepare_workspace(attempt, case_root, clean_crates_root)
        except Exception as error:
            write_json(result_path, {**case, "status": "prepare_failed", "error": str(error)})
            mark_case_complete(case_root, args.completion_markers)
            continue
        workspace_hook = workspace / ".claude/hooks/verify_repair.py"
        workspace_hook.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(verify_hook_path, workspace_hook)
        source_file = workspace / injection["src_path"]
        original_source = source_file.read_text(encoding="utf-8")
        env = experiment_environment(
            cargo_target_root / str(case["crate"]), args.proxy_port, args.model
        )
        initial = cargo_check(workspace, env, args.cargo_timeout_secs)
        (case_root / "initial_cargo.stdout.jsonl").write_text(initial[1], encoding="utf-8")
        (case_root / "initial_cargo.stderr.log").write_text(initial[2], encoding="utf-8")
        initial_errors = initial[5]
        original_snapshot = snapshot_workspace(workspace)
        prompt = render_prompt(prompt_template, injection, original_source, initial[1])
        (case_root / "prompt.txt").write_text(prompt, encoding="utf-8")

        if args.prepare_only:
            write_json(
                result_path,
                {
                    **case,
                    "status": "prepared",
                    "injection": injection,
                    "initial_cargo_exit_code": initial[0],
                    "initial_error_total": initial_errors,
                },
            )
            mark_case_complete(case_root, args.completion_markers)
            continue
        if initial[3] or initial[0] == 0 or initial_errors == 0:
            write_json(
                result_path,
                {
                    **case,
                    "status": "reproduction_mismatch",
                    "eligible": False,
                    "strict_success": False,
                    "injection": injection,
                    "initial_cargo_exit_code": initial[0],
                    "initial_cargo_timed_out": initial[3],
                    "initial_error_total": initial_errors,
                    "usage": {**{field: 0 for field in USAGE_FIELDS}, "gross_tokens": 0, "total_tokens": 0, "request_count": 0, "usage_complete": True},
                    **paired_ruter_values(ruter_rows.get(case_name, {})),
                },
            )
            mark_case_complete(case_root, args.completion_markers)
            continue

        proxy_log = case_root / "proxy_usage.jsonl"
        proxy = start_proxy(
            proxy_script=proxy_script,
            upstream=args.upstream,
            port=args.proxy_port,
            log_path=proxy_log,
            api_key=str(api_key),
            cwd=PROJECT_ROOT,
        )
        command = [
            args.claude_bin,
            "-p",
            prompt,
            "--output-format",
            "stream-json",
            "--verbose",
            "--model",
            args.model,
            "--max-turns",
            str(args.max_turns),
            "--tools",
            "Read,Edit,Glob,Grep,Bash",
            "--permission-mode",
            "dontAsk",
            "--permission-prompts",
            "none",
            "--no-session-persistence",
            "--disable-slash-commands",
            "--strict-mcp-config",
            "--mcp-config",
            '{"mcpServers":{}}',
            "--setting-sources",
            "project",
            "--settings",
            str(settings_path),
        ]
        started_at = utc_now()
        try:
            claude_run = run_command(
                command,
                cwd=workspace,
                env=env,
                timeout=args.timeout_secs,
            )
        finally:
            proxy_stdout, proxy_stderr = stop_proxy(proxy)
            (case_root / "proxy.stdout.log").write_text(proxy_stdout, encoding="utf-8")
            (case_root / "proxy.stderr.log").write_text(proxy_stderr, encoding="utf-8")
        (case_root / "claude.stream.jsonl").write_text(claude_run[1], encoding="utf-8")
        (case_root / "claude.stderr.log").write_text(claude_run[2], encoding="utf-8")

        final = cargo_check(workspace, env, args.cargo_timeout_secs)
        (case_root / "final_cargo.stdout.jsonl").write_text(final[1], encoding="utf-8")
        (case_root / "final_cargo.stderr.log").write_text(final[2], encoding="utf-8")
        validation = validate_repair(
            workspace=workspace,
            source_path=injection["src_path"],
            original_source=original_source,
            original_snapshot=original_snapshot,
        )
        final_source = validation.pop("final_source")
        diff = "".join(
            difflib.unified_diff(
                original_source.splitlines(keepends=True),
                final_source.splitlines(keepends=True),
                fromfile=f"before/{injection['src_path']}",
                tofile=f"after/{injection['src_path']}",
            )
        )
        (case_root / "repair.diff").write_text(diff, encoding="utf-8")

        records = load_jsonl(proxy_log)
        usage = proxy_usage(records)
        claude_result = final_claude_result(claude_run[1])
        claude_usage = result_usage(claude_result)
        # Claude's final result reports the same four Anthropic usage components;
        # its derived surface sum intentionally differs from the gateway's primary
        # OpenAI total when cached prompt tokens are present. Compare components,
        # not the two different derived totals.
        usage_crosscheck = claude_usage is not None and all(
            claude_usage[field]
            == usage[
                "provider_input_tokens"
                if field == "input_tokens"
                else "provider_output_tokens"
                if field == "output_tokens"
                else field
            ]
            for field in USAGE_FIELDS
        )
        valid_model = model_valid(usage["returned_models"], args.model)
        infrastructure_failed = (
            usage["provider_usage_complete"] is not True
            or usage["usage_complete"] is not True
            or not valid_model
        )
        repaired = (
            not final[3]
            and final[0] == 0
            and final[5] == 0
            and validation["scope_valid"]
            and validation["integrity_valid"]
        )
        if infrastructure_failed:
            status = "infrastructure_failed"
        elif claude_run[3]:
            status = "timeout"
        elif claude_run[0] != 0:
            status = "claude_failed"
        elif repaired:
            status = "repaired"
        else:
            status = "not_repaired"
        ruter_row = ruter_rows.get(case_name, {})
        write_json(
            result_path,
            {
                **case,
                "status": status,
                "eligible": True,
                "strict_success": repaired,
                "started_at_utc": started_at,
                "finished_at_utc": utc_now(),
                "injection": injection,
                "initial_cargo_exit_code": initial[0],
                "initial_error_total": initial_errors,
                "final_cargo_exit_code": final[0],
                "final_cargo_timed_out": final[3],
                "final_error_total": final[5],
                "claude_exit_code": claude_run[0],
                "claude_timed_out": claude_run[3],
                "claude_duration_sec": claude_run[4],
                "claude_result": public_result_metadata(claude_result),
                "claude_result_usage": claude_usage,
                "usage": usage,
                "usage_crosscheck_with_claude_result": usage_crosscheck,
                "model_requested": args.model,
                "model_valid": valid_model,
                "validation": validation,
                **paired_ruter_values(ruter_row),
                "workspace_retained": not args.discard_workspaces,
                "command": ["<claude>", "-p", "<fixed prompt>", *command[3:]],
            },
        )
        if args.discard_workspaces:
            shutil.rmtree(workspace)
        print(
            f"[{index}/{len(cases)}] {status} tokens={usage['total_tokens']} "
            f"requests={usage['request_count']} model_ok={valid_model}",
            flush=True,
        )
        if infrastructure_failed:
            record_infrastructure_failure(out_root, "claude", result_path)
            mark_case_complete(case_root, args.completion_markers)
            if args.stop_on_infrastructure_failure:
                summarize(out_root, False, False, args.seed)
                print("API accounting incomplete; stopping safely for resume", flush=True)
                return 3
            print("API failure recorded; continuing with the next case", flush=True)
        mark_case_complete(case_root, args.completion_markers)

    summary = summarize(
        out_root,
        complete_paired_sample and not args.reuse_only,
        allow_weighted_estimates and not args.reuse_only,
        args.seed,
    )
    print(
        f"completed {summary['case_count']}/{len(cases)} cases under {out_root}",
        flush=True,
    )
    return 0 if args.reuse_only or summary["case_count"] == len(cases) else 2


if __name__ == "__main__":
    raise SystemExit(main())
