#!/usr/bin/env python3
"""Re-run frozen RuTeR failures and collect provider-reported token usage."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import random
import re
import shutil
import signal
import subprocess
import sys
import time
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit, urlunsplit


SCRIPT_PATH = Path(__file__).resolve()
RUTER_REPO_ROOT = SCRIPT_PATH.parents[2]
PROJECT_ROOT = RUTER_REPO_ROOT
DEFAULT_MANIFEST = PROJECT_ROOT / "experiments/inputs/frozen_attempt_manifest.json"
DEFAULT_CRATES_ROOT = PROJECT_ROOT / "experiments/inputs/crates"
SUPPORTED_CODES = {"E0433", "E0432", "E0560", "E0599", "E0308"}
MOD_DECL_RE = re.compile(r"\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{", re.M)

# Chosen from the frozen Phase 1A artifacts to cover rule-only, one-round,
# two-round, and three-or-more-round historical behavior across four crates.
SMOKE_ATTEMPT_UIDS = [
    "humantime_claude-3-5-haiku-20241022_20251109_133407::"
    "<wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref::1",
    "itoa_gemini-2.5-flash-nothinking_20251127_010109::"
    "<impl private::Sealed for i32>::write::1",
    "log_gemini-2.5-flash-nothinking_20251127_025911::"
    "<NopLogger as Log>::enabled::1",
    "mio_gemini-2.5-flash-nothinking_20251127_012706::"
    "<&'a event::events::Events as std::iter::IntoIterator>::into_iter::1",
]


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def load_json(path: Path, default: Any = None) -> Any:
    if not path.exists():
        return default
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def normalize_api_url(raw: str) -> str:
    """Treat a bare host as an OpenAI-compatible /v1 endpoint."""
    value = raw.strip().rstrip("/")
    parts = urlsplit(value)
    if not parts.scheme or not parts.netloc:
        raise ValueError("API URL must be an absolute http(s) URL")
    if parts.scheme not in {"http", "https"}:
        raise ValueError("API URL scheme must be http or https")
    path = parts.path.rstrip("/")
    if not path:
        path = "/v1"
    return urlunsplit((parts.scheme, parts.netloc, path, parts.query, ""))


def classify_difficulty(attempt: dict[str, Any]) -> str:
    codes = {str(code) for code in attempt.get("error_codes") or []}
    if not codes:
        return "no_error_code"
    supported = codes & SUPPORTED_CODES
    if len(codes) == 1:
        return "single_supported" if supported else "single_unsupported"
    if len(supported) == len(codes):
        return "multi_supported_only"
    if supported:
        return "multi_mixed"
    return "multi_unsupported_only"


def sampling_stratum(attempt: dict[str, Any]) -> str:
    return f"{attempt.get('crate', 'unknown')}|{classify_difficulty(attempt)}"


def proportional_sample(
    attempts: list[dict[str, Any]], sample_size: int, seed: int
) -> list[dict[str, Any]]:
    if sample_size <= 0:
        raise ValueError("sample size must be positive")
    if sample_size >= len(attempts):
        selected = list(attempts)
        for attempt in selected:
            attempt["_sampling_stratum"] = sampling_stratum(attempt)
            attempt["_sampling_weight"] = 1.0
        return selected

    groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for attempt in attempts:
        groups[sampling_stratum(attempt)].append(attempt)

    rng = random.Random(seed)
    for items in groups.values():
        rng.shuffle(items)

    keys = sorted(groups)
    allocation = {key: 0 for key in keys}
    remaining = sample_size
    # At n=100 this retains every crate-by-difficulty cell and normally gives
    # each non-singleton cell at least two observations for variance estimation.
    two_each = sum(min(2, len(groups[key])) for key in keys)
    if sample_size >= two_each:
        for key in keys:
            allocation[key] = min(2, len(groups[key]))
            remaining -= allocation[key]
    elif sample_size >= len(keys):
        for key in keys:
            allocation[key] = 1
            remaining -= 1

    capacities = {key: len(groups[key]) - allocation[key] for key in keys}
    capacity_total = sum(capacities.values())
    exact: dict[str, float] = {}
    for key in keys:
        share = remaining * capacities[key] / capacity_total if capacity_total else 0.0
        extra = min(capacities[key], math.floor(share))
        allocation[key] += extra
        exact[key] = share - extra

    unassigned = sample_size - sum(allocation.values())
    tie_order = list(keys)
    rng.shuffle(tie_order)
    tie_rank = {key: idx for idx, key in enumerate(tie_order)}
    ranked = sorted(keys, key=lambda key: (-exact[key], tie_rank[key]))
    while unassigned:
        progressed = False
        for key in ranked:
            if allocation[key] >= len(groups[key]):
                continue
            allocation[key] += 1
            unassigned -= 1
            progressed = True
            if not unassigned:
                break
        if not progressed:
            raise RuntimeError("unable to allocate requested stratified sample")

    selected: list[dict[str, Any]] = []
    for key in keys:
        take = allocation[key]
        population = len(groups[key])
        weight = population / take if take else 0.0
        for attempt in groups[key][:take]:
            attempt["_sampling_stratum"] = key
            attempt["_sampling_weight"] = weight
            selected.append(attempt)
    rng.shuffle(selected)
    return selected


def select_attempts(
    attempts: list[dict[str, Any]],
    *,
    smoke: bool,
    requested_uids: list[str],
    sample_size: int,
    seed: int,
) -> list[dict[str, Any]]:
    by_uid = {str(item["attempt_uid"]): item for item in attempts}
    if smoke or requested_uids:
        uids = SMOKE_ATTEMPT_UIDS if smoke else requested_uids
        missing = [uid for uid in uids if uid not in by_uid]
        if missing:
            raise ValueError(f"attempt UID not found: {missing[0]}")
        selected = []
        for uid in uids:
            attempt = by_uid[uid]
            attempt["_sampling_stratum"] = "purposeful_smoke"
            attempt["_sampling_weight"] = 1.0
            selected.append(attempt)
        return selected
    return proportional_sample(attempts, sample_size, seed)


def case_id(index: int, attempt_uid: str) -> str:
    digest = hashlib.sha256(attempt_uid.encode("utf-8")).hexdigest()[:12]
    return f"case_{index:04d}_{digest}"


def extract_mod_name(code: str) -> str | None:
    match = MOD_DECL_RE.search(code)
    return match.group(1) if match else None


def rename_first_mod(code: str, new_name: str) -> str:
    def replace(match: re.Match[str]) -> str:
        return match.group(0).replace(match.group(1), new_name, 1)

    return MOD_DECL_RE.sub(replace, code, count=1)


def prepare_workspace(
    attempt: dict[str, Any], case_root: Path, clean_crates_root: Path
) -> tuple[Path, dict[str, Any]]:
    crate = str(attempt["crate"])
    clean_crate = clean_crates_root / crate
    if not clean_crate.is_dir():
        raise FileNotFoundError(f"clean crate not found: {clean_crate}")

    workspace = case_root / "workspace"
    shutil.copytree(
        clean_crate,
        workspace,
        ignore=shutil.ignore_patterns(".git", ".git.bak", "target"),
    )
    src_path = Path(str(attempt["src_path"]))
    if src_path.is_absolute() or ".." in src_path.parts:
        raise ValueError(f"unsafe source path in manifest: {src_path}")
    target = workspace / src_path
    if not target.is_file():
        raise FileNotFoundError(f"injection target not found: {target}")

    source = target.read_text(encoding="utf-8")
    code = str(attempt.get("injected_code") or "").rstrip() + "\n"
    if not code.strip():
        raise ValueError("frozen attempt has empty injected_code")
    original_mod = extract_mod_name(code)
    effective_mod = original_mod
    existing_mods = set(MOD_DECL_RE.findall(source))
    if effective_mod and effective_mod in existing_mods:
        suffix = int(attempt.get("attempt_seq") or 0)
        effective_mod = f"{effective_mod}__token_case_{suffix:04d}"
        while effective_mod in existing_mods:
            effective_mod += "_x"
        code = rename_first_mod(code, effective_mod)

    block = (
        "\n\n// >>> TOKEN COST EXPERIMENT INJECTION BEGIN >>>\n"
        f"// attempt_uid={attempt['attempt_uid']}\n"
        f"{code.rstrip()}\n"
        "// <<< TOKEN COST EXPERIMENT INJECTION END <<<\n"
    )
    target.write_text(source.rstrip() + block, encoding="utf-8")
    return workspace, {
        "clean_crate": str(clean_crate),
        "src_path": src_path.as_posix(),
        "original_mod_name": original_mod,
        "effective_mod_name": effective_mod,
    }


def sanitized_environment(cargo_target_dir: Path) -> dict[str, str]:
    env = dict(os.environ)
    for key in list(env):
        if key.startswith("CARGO_FEATURE_") or key.startswith("CARGO_CFG_"):
            env.pop(key, None)
        if key.startswith("RUTER_LLM_") and key != "RUTER_LLM_API_KEY":
            env.pop(key, None)
    env["CARGO_TERM_COLOR"] = "never"
    env["CARGO_TARGET_DIR"] = str(cargo_target_dir)
    return env


def run_with_timeout(
    command: list[str], cwd: Path, env: dict[str, str], timeout: int
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
    duration = round(time.monotonic() - started, 3)
    return proc.returncode, stdout or "", stderr or "", timed_out, duration


def strict_success(summary: dict[str, Any], eligible: bool) -> bool:
    return (
        eligible
        and summary.get("patch_verify_check_passed") is True
        and int(summary.get("remaining_error_total") or 0) == 0
        and int(summary.get("unresolved_test_function_count") or 0) == 0
    )


def initial_error_count(artifacts: Path, summary: dict[str, Any]) -> int:
    summary_count = int(summary.get("initial_error_total") or 0)
    diagnostics = load_json(artifacts / "1_compile_diagnostics.json", [])
    diagnostic_count = 0
    if isinstance(diagnostics, list):
        diagnostic_count = sum(
            isinstance(item, dict)
            and str(item.get("level") or "").lower() == "error"
            for item in diagnostics
        )
    return max(summary_count, diagnostic_count)


def refresh_existing_result(result_path: Path) -> None:
    result = load_json(result_path, {})
    if not isinstance(result, dict):
        return
    artifact_raw = result.get("artifact_dir") or "ruter_artifacts"
    artifacts = Path(str(artifact_raw))
    if not artifacts.is_absolute():
        artifacts = result_path.parent / artifacts
    summary = load_json(artifacts / "6_summary.json", {})
    count = initial_error_count(artifacts, summary if isinstance(summary, dict) else {})
    result["initial_error_total"] = count
    result["eligible"] = count > 0
    if count > 0:
        result["exclusion_reason"] = None
    elif not result.get("exclusion_reason"):
        result["exclusion_reason"] = "reconstructed input has zero compiler errors"
    result["strict_success"] = strict_success(
        summary if isinstance(summary, dict) else {}, count > 0
    )
    write_json(result_path, result)


def is_retryable_infrastructure_failure(result_path: Path) -> bool:
    """Return whether a completed case lacks provider-complete usage metadata.

    Timeouts remain method outcomes.  This predicate is deliberately limited to
    missing/failed provider accounting or a runtime failure without a usage
    artifact, so ``--resume`` cannot silently turn ordinary repair failures into
    retries.
    """
    result = load_json(result_path, {})
    if not isinstance(result, dict) or result.get("timed_out") is True:
        return False
    artifact_raw = result.get("artifact_dir") or "ruter_artifacts"
    artifacts = Path(str(artifact_raw))
    if not artifacts.is_absolute():
        artifacts = result_path.parent / artifacts
    usage_path = artifacts / "4_llm_usage.json"
    usage = load_json(usage_path, {})
    usage_summary = usage.get("summary", {}) if isinstance(usage, dict) else {}
    request_count = int(usage_summary.get("request_count") or 0)
    usage_reported = int(
        usage_summary.get("usage_reported_request_count") or 0
    )
    usage_missing = int(usage_summary.get("usage_missing_request_count") or 0)
    if usage_path.is_file():
        allowed_models = {str(usage.get("configured_model") or result.get("repair_model") or "")}
        if "gemini-2.5-flash-nothinking" in allowed_models:
            allowed_models.add("gemini-2.5-flash")
        requests = usage.get("requests") or []
        model_invalid = request_count > 0 and (
            len(requests) != request_count
            or any(str(item.get("returned_model") or "") not in allowed_models for item in requests)
        )
        # The aggregate failed_request_count also includes invalid candidate
        # JSON returned by a successful, fully accounted provider request.
        # Those are method failures, not API failures: retrying them would give
        # the repair method extra attempts and bias the experiment.
        provider_failed = any(
            item.get("http_status") is not None
            and int(item["http_status"]) >= 400
            for item in requests
        )
        return (
            usage_missing > 0
            or provider_failed
            or usage_reported != request_count
            or model_invalid
        )
    return result.get("status") == "runtime_failed"


def infrastructure_failure_record(stage: str, result_path: Path) -> dict[str, Any]:
    """Metadata-only failure record; never include prompts, headers or keys."""
    result = load_json(result_path, {})
    statuses: list[int] = []
    if stage == "ruter":
        artifacts = Path(result.get("artifact_dir") or "ruter_artifacts")
        if not artifacts.is_absolute():
            artifacts = result_path.parent / artifacts
        usage = load_json(artifacts / "4_llm_usage.json", {})
        records = usage.get("requests") or []
        usage_summary = usage.get("summary") or {}
        request_count = int(usage_summary.get("request_count") or 0)
    else:
        records = []
        proxy_log = result_path.parent / "proxy_usage.jsonl"
        if proxy_log.is_file():
            for line in proxy_log.read_text(encoding="utf-8").splitlines():
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if isinstance(row, dict) and row.get("request_kind") == "messages":
                    records.append(row)
        request_count = int((result.get("usage") or {}).get("request_count") or 0)
    for row in records:
        code = row.get("http_status") if stage == "ruter" else row.get("status")
        if isinstance(code, int):
            statuses.append(code)
    return {
        "recorded_at_utc": utc_now(),
        "system": stage,
        "case_id": result.get("case_id"),
        "attempt_uid": result.get("attempt_uid"),
        "crate": result.get("crate"),
        "attempt_started_at_utc": result.get("started_at_utc"),
        "attempt_finished_at_utc": result.get("finished_at_utc"),
        "case_status": result.get("status"),
        "request_count": request_count,
        "http_statuses": statuses,
        "http_402_count": statuses.count(402),
        "result_path": str(result_path.resolve()),
        "included_in_primary_analysis": False,
    }


def record_infrastructure_failure(out_root: Path, stage: str, result_path: Path) -> None:
    record = infrastructure_failure_record(stage, result_path)
    with (out_root / "api_failures.jsonl").open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(record, ensure_ascii=False) + "\n")


def git_commit() -> str | None:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=RUTER_REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    value = result.stdout.strip()
    return value if result.returncode == 0 and value else None


def resolve_binary(raw: str | None) -> Path:
    if raw:
        candidate = Path(raw).expanduser().resolve()
        if not candidate.is_file():
            raise FileNotFoundError(f"RuTeR binary not found: {candidate}")
        return candidate
    candidates = [
        RUTER_REPO_ROOT / "ruter/target/release/ruter",
        RUTER_REPO_ROOT / "ruter/target/debug/ruter",
    ]
    for candidate in candidates:
        if candidate.is_file():
            return candidate.resolve()
    raise FileNotFoundError(
        "RuTeR binary not found; run `cargo build --release --bin ruter` in RuTeR/ruter"
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Re-run a reproducible sample and measure RuTeR token cost."
    )
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    parser.add_argument("--clean-crates-root", default=str(DEFAULT_CRATES_ROOT))
    parser.add_argument("--ruter-bin")
    parser.add_argument("--out", required=True)
    parser.add_argument("--api-url", default=os.environ.get("RUTER_LLM_API_URL"))
    parser.add_argument("--model", default=os.environ.get("RUTER_LLM_MODEL"))
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--smoke", action="store_true")
    selection.add_argument("--case-id", action="append", dest="attempt_uids", default=[])
    selection.add_argument("--sample-size", type=int, default=100)
    parser.add_argument("--seed", type=int, default=20260924)
    parser.add_argument("--topk", type=int, default=3)
    parser.add_argument("--max-rounds", type=int, default=3)
    parser.add_argument("--timeout-secs", type=int, default=900)
    parser.add_argument("--llm-timeout-secs", type=int, default=120)
    parser.add_argument(
        "--cargo-target-root",
        help="Shared Cargo target root (default: <out>/_cargo_target)",
    )
    parser.add_argument(
        "--discard-workspaces",
        action="store_true",
        help="Delete reconstructed workspaces after each completed case",
    )
    parser.add_argument(
        "--reuse-results-from",
        help="Reuse completed cases from a compatible smaller experiment",
    )
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--resume", action="store_true")
    parser.add_argument(
        "--retry-infrastructure-failures",
        action="store_true",
        help=(
            "With --resume, archive and rerun non-timeout cases whose provider "
            "usage is missing, incomplete, or records failed API requests"
        ),
    )
    parser.add_argument(
        "--stop-on-infrastructure-failure",
        action="store_true",
        help="Save the case and exit 3 on incomplete/failed API accounting",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    manifest_path = Path(args.manifest).resolve()
    crates_root = Path(args.clean_crates_root).resolve()
    out_root = Path(args.out).resolve()
    if out_root.exists() and any(out_root.iterdir()) and not args.resume:
        raise SystemExit(f"output directory is not empty; use --resume: {out_root}")
    out_root.mkdir(parents=True, exist_ok=True)
    cargo_target_root = (
        Path(args.cargo_target_root).resolve()
        if args.cargo_target_root
        else out_root / "_cargo_target"
    )

    manifest = load_json(manifest_path, {})
    attempts = manifest.get("attempts") if isinstance(manifest, dict) else None
    if not isinstance(attempts, list) or not attempts:
        raise SystemExit(f"manifest contains no attempts: {manifest_path}")
    selected = select_attempts(
        attempts,
        smoke=args.smoke,
        requested_uids=args.attempt_uids,
        sample_size=args.sample_size,
        seed=args.seed,
    )

    api_url = None
    model = args.model.strip() if isinstance(args.model, str) else None
    binary = None
    if not args.prepare_only:
        if not args.api_url:
            raise SystemExit("--api-url (or RUTER_LLM_API_URL) is required")
        if not model:
            raise SystemExit("--model (or RUTER_LLM_MODEL) is required")
        if not os.environ.get("RUTER_LLM_API_KEY"):
            raise SystemExit("RUTER_LLM_API_KEY must be set in the environment")
        api_url = normalize_api_url(args.api_url)
        binary = resolve_binary(args.ruter_bin)
    elif args.api_url:
        api_url = normalize_api_url(args.api_url)

    reusable: dict[str, Path] = {}
    reuse_root = Path(args.reuse_results_from).resolve() if args.reuse_results_from else None
    if reuse_root:
        reuse_manifest = load_json(reuse_root / "experiment_manifest.json", {})
        expected = {
            "repair_model": model,
            "api_url": api_url,
            "topk": args.topk,
            "max_rounds": args.max_rounds,
        }
        observed = {
            "repair_model": reuse_manifest.get("repair_model"),
            "api_url": reuse_manifest.get("api_url"),
            "topk": (reuse_manifest.get("configuration") or {}).get("topk"),
            "max_rounds": (reuse_manifest.get("configuration") or {}).get("max_rounds"),
        }
        if observed != expected:
            raise SystemExit(
                f"reuse experiment configuration mismatch: expected={expected} observed={observed}"
            )
        for old_result in sorted(reuse_root.glob("cases/*/case_result.json")):
            old = load_json(old_result, {})
            uid = old.get("attempt_uid") if isinstance(old, dict) else None
            if uid:
                reusable[str(uid)] = old_result

    cases: list[dict[str, Any]] = []
    for index, attempt in enumerate(selected, start=1):
        cases.append(
            {
                "case_id": case_id(index, str(attempt["attempt_uid"])),
                "attempt_uid": attempt["attempt_uid"],
                "attempt_seq": attempt.get("attempt_seq"),
                "run_id": attempt.get("run_id"),
                "crate": attempt.get("crate"),
                "generation_model": attempt.get("model"),
                "node_id": attempt.get("node_id"),
                "src_path": attempt.get("src_path"),
                "error_codes": attempt.get("error_codes") or [],
                "difficulty": classify_difficulty(attempt),
                "sampling_stratum": attempt.get("_sampling_stratum"),
                "sampling_weight": attempt.get("_sampling_weight", 1.0),
            }
        )

    experiment = {
        "schema_version": "1",
        "created_at_utc": utc_now(),
        "manifest": str(manifest_path),
        "manifest_created_at_utc": (manifest.get("meta") or {}).get("created_at_utc"),
        "clean_crates_root": str(crates_root),
        "ruter_git_commit": git_commit(),
        "ruter_binary": str(binary) if binary else None,
        "repair_model": model,
        "api_url": api_url,
        "api_key_source": "RUTER_LLM_API_KEY environment variable",
        "api_key_persisted": False,
        "reuse_results_from": str(reuse_root) if reuse_root else None,
        "selection": {
            "mode": "smoke" if args.smoke else "explicit" if args.attempt_uids else "stratified",
            "seed": args.seed,
            "population_size": len(attempts),
            "sample_size": len(selected),
            "stratification": "crate x error difficulty",
        },
        "configuration": {
            "topk": args.topk,
            "max_rounds": args.max_rounds,
            "case_timeout_secs": args.timeout_secs,
            "llm_timeout_secs": args.llm_timeout_secs,
            "dry_run": True,
            "debug_full_io": False,
            "cargo_target_root": str(cargo_target_root),
            "discard_workspaces": args.discard_workspaces,
        },
        "cases": cases,
    }
    write_json(out_root / "experiment_manifest.json", experiment)

    attempts_by_uid = {str(item["attempt_uid"]): item for item in attempts}
    completed = 0
    for index, case in enumerate(cases, start=1):
        case_root = out_root / "cases" / str(case["case_id"])
        result_path = case_root / "case_result.json"
        if args.resume and result_path.exists():
            should_retry = (
                args.retry_infrastructure_failures
                and is_retryable_infrastructure_failure(result_path)
            )
            if not should_retry:
                if is_retryable_infrastructure_failure(result_path):
                    record_infrastructure_failure(out_root, "ruter", result_path)
                refresh_existing_result(result_path)
                print(
                    f"[{index}/{len(cases)}] resume skip {case['case_id']}",
                    flush=True,
                )
                completed += 1
                continue
            print(
                f"[{index}/{len(cases)}] retry infrastructure failure "
                f"{case['case_id']}",
                flush=True,
            )
        if args.resume and case_root.exists():
            aborted_root = out_root / "_aborted_cases"
            aborted_root.mkdir(exist_ok=True)
            suffix = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
            archived = aborted_root / f"{case['case_id']}_{suffix}"
            shutil.move(str(case_root), str(archived))
            print(
                f"[{index}/{len(cases)}] archived incomplete case at {archived}",
                flush=True,
            )
        if case_root.exists():
            raise SystemExit(f"case directory already exists without result: {case_root}")
        reusable_result = reusable.get(str(case["attempt_uid"]))
        if reusable_result and not args.prepare_only:
            shutil.copytree(reusable_result.parent, case_root)
            reused = load_json(case_root / "case_result.json", {})
            reused.update(case)
            reused["reused_from"] = str(reusable_result)
            write_json(case_root / "case_result.json", reused)
            refresh_existing_result(case_root / "case_result.json")
            completed += 1
            print(
                f"[{index}/{len(cases)}] reused {case['case_id']} from {reuse_root}",
                flush=True,
            )
            continue
        case_root.mkdir(parents=True)
        attempt = attempts_by_uid[str(case["attempt_uid"])]
        print(
            f"[{index}/{len(cases)}] prepare {case['case_id']} crate={case['crate']}",
            flush=True,
        )
        try:
            workspace, injection = prepare_workspace(attempt, case_root, crates_root)
        except Exception as error:
            write_json(
                result_path,
                {
                    **case,
                    "status": "prepare_failed",
                    "eligible": False,
                    "exclusion_reason": str(error),
                    "artifact_dir": "ruter_artifacts",
                },
            )
            completed += 1
            continue

        if args.prepare_only:
            write_json(
                result_path,
                {
                    **case,
                    "status": "prepared",
                    "eligible": False,
                    "exclusion_reason": "prepare-only run",
                    "artifact_dir": "ruter_artifacts",
                    "injection": injection,
                },
            )
            completed += 1
            continue

        artifacts = case_root / "ruter_artifacts"
        artifacts.mkdir()
        command = [
            str(binary),
            "fix",
            str(workspace),
            "--artifacts-dir",
            str(artifacts),
            "--topk",
            str(args.topk),
            "--enable-llm",
            "--llm-mode",
            "online",
            "--llm-api-url",
            str(api_url),
            "--llm-model",
            str(model),
            "--llm-max-rounds",
            str(args.max_rounds),
            "--llm-timeout-secs",
            str(args.llm_timeout_secs),
        ]
        env = sanitized_environment(cargo_target_root / str(case["crate"]))
        started_at = utc_now()
        exit_code, stdout, stderr, timed_out, duration = run_with_timeout(
            command, RUTER_REPO_ROOT, env, args.timeout_secs
        )
        (case_root / "stdout.log").write_text(stdout, encoding="utf-8")
        (case_root / "stderr.log").write_text(stderr, encoding="utf-8")
        summary = load_json(artifacts / "6_summary.json", {})
        usage = load_json(artifacts / "4_llm_usage.json", {})
        initial_errors = initial_error_count(artifacts, summary)
        artifacts_ready = bool(summary)
        eligible = initial_errors > 0
        if timed_out:
            status = "timeout"
            exclusion = None if eligible else "RuTeR case timeout before reproduction"
        elif not artifacts_ready:
            status = "runtime_failed"
            exclusion = None if eligible else "6_summary.json was not produced"
        elif initial_errors <= 0:
            status = "reproduction_mismatch"
            exclusion = "frozen failure reproduced with zero initial errors"
        elif exit_code not in {0, 7}:
            status = "runtime_failed"
            exclusion = f"unexpected RuTeR exit code {exit_code}"
        elif strict_success(summary, eligible):
            status = "repaired"
            exclusion = None
        else:
            status = "not_repaired"
            exclusion = None
        request_count = int(((usage.get("summary") or {}).get("request_count") or 0))
        write_json(
            result_path,
            {
                **case,
                "repair_model": model,
                "status": status,
                "eligible": eligible,
                "exclusion_reason": exclusion,
                "strict_success": strict_success(summary, eligible),
                "exit_code": exit_code,
                "timed_out": timed_out,
                "duration_sec": duration,
                "started_at_utc": started_at,
                "finished_at_utc": utc_now(),
                "artifact_dir": "ruter_artifacts",
                "workspace_dir": "workspace",
                "workspace_retained": not args.discard_workspaces,
                "injection": injection,
                "initial_error_total": initial_errors,
                "remaining_error_total": summary.get("remaining_error_total"),
                "llm_request_count": request_count,
                "command": command,
            },
        )
        if args.discard_workspaces:
            shutil.rmtree(workspace)
        completed += 1
        print(
            f"[{index}/{len(cases)}] {status} requests={request_count} "
            f"duration={duration:.1f}s",
            flush=True,
        )
        if is_retryable_infrastructure_failure(result_path):
            record_infrastructure_failure(out_root, "ruter", result_path)
            if args.stop_on_infrastructure_failure:
                print("API accounting incomplete; stopping safely for resume", flush=True)
                return 3
            print("API failure recorded; continuing with the next case", flush=True)

    print(f"completed {completed}/{len(cases)} cases under {out_root}")
    return 0 if completed == len(cases) else 2


if __name__ == "__main__":
    raise SystemExit(main())
