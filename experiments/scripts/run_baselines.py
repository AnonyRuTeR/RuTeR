#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from common import (
    ROOT,
    copy_subject_to_temp,
    default_subject_command,
    ensure_dir,
    error_code_counts,
    extract_diagnostics_from_cargo_stdout,
    load_manifest,
    machine_applicable_suggestions,
    apply_suggestions_in_place,
    run_command,
    utc_now,
    write_json,
)


def write_result(path: Path, data: dict) -> None:
    ensure_dir(path.parent)
    write_json(path, data)


def run_no_repair(subject, out_root: Path) -> None:
    subject_out = ensure_dir(out_root / "no_repair" / subject.subject_id)
    resolved = subject.resolved_path()
    result_json = {
        "baseline": "no_repair",
        "subject_id": subject.subject_id,
        "timestamp": utc_now(),
        "status": "unsupported",
        "initial_error_codes": {},
        "final_error_codes": {},
        "patch_count": 0,
        "modified_files": [],
        "production_files_modified": False,
        "verification_passed": "unknown",
        "notes": [],
    }
    if subject.source_type != "local_fixture" or resolved is None or not resolved.exists():
        result_json["notes"].append("only existing local_fixture subjects are supported by no_repair baseline")
        return write_result(subject_out / "baseline_result.json", result_json)

    command = default_subject_command(resolved)
    result = run_command(command, cwd=resolved)
    diagnostics = extract_diagnostics_from_cargo_stdout(subject.subject_id, resolved, result.stdout)
    (subject_out / "raw_stdout.log").write_text(result.stdout, encoding="utf-8")
    (subject_out / "raw_stderr.log").write_text(result.stderr, encoding="utf-8")
    result_json.update(
        {
            "command": command,
            "cwd": str(resolved),
            "exit_code": result.returncode,
            "initial_error_codes": error_code_counts(diagnostics),
            "final_error_codes": error_code_counts(diagnostics),
            "verification_passed": result.returncode == 0,
            "status": "compile_passed" if result.returncode == 0 else "compile_failed",
        }
    )
    write_result(subject_out / "baseline_result.json", result_json)


def run_compiler_suggestion_only(subject, out_root: Path) -> None:
    subject_out = ensure_dir(out_root / "compiler_suggestion_only" / subject.subject_id)
    resolved = subject.resolved_path()
    result_json = {
        "baseline": "compiler_suggestion_only",
        "subject_id": subject.subject_id,
        "timestamp": utc_now(),
        "status": "unsupported",
        "initial_error_codes": {},
        "final_error_codes": {},
        "patch_count": "unknown",
        "modified_files": [],
        "production_files_modified": "unknown",
        "verification_passed": "unknown",
        "notes": [],
    }
    if subject.source_type != "local_fixture" or resolved is None or not resolved.exists():
        result_json["notes"].append("only existing local_fixture subjects are supported by compiler_suggestion_only baseline")
        return write_result(subject_out / "baseline_result.json", result_json)

    initial = run_command(default_subject_command(resolved), cwd=resolved)
    initial_diags = extract_diagnostics_from_cargo_stdout(subject.subject_id, resolved, initial.stdout)
    suggestions = machine_applicable_suggestions(initial_diags, resolved)
    (subject_out / "raw_initial_stdout.log").write_text(initial.stdout, encoding="utf-8")
    (subject_out / "raw_initial_stderr.log").write_text(initial.stderr, encoding="utf-8")
    write_json(subject_out / "suggestions.json", suggestions)
    result_json["initial_error_codes"] = error_code_counts(initial_diags)

    if not suggestions:
        result_json["status"] = "unsupported"
        result_json["final_error_codes"] = error_code_counts(initial_diags)
        result_json["patch_count"] = 0
        result_json["notes"].append("no MachineApplicable local suggestions were available")
        return write_result(subject_out / "baseline_result.json", result_json)

    temp_root, temp_handle = copy_subject_to_temp(resolved)
    try:
        remapped = []
        for item in suggestions:
            rel = Path(item["abs_path"]).resolve().relative_to(resolved.resolve())
            remapped.append({**item, "abs_path": str((temp_root / rel).resolve())})
        modified_files = apply_suggestions_in_place(temp_root, remapped)
        second = run_command(default_subject_command(temp_root), cwd=temp_root)
        second_diags = extract_diagnostics_from_cargo_stdout(subject.subject_id, temp_root, second.stdout)
        (subject_out / "raw_final_stdout.log").write_text(second.stdout, encoding="utf-8")
        (subject_out / "raw_final_stderr.log").write_text(second.stderr, encoding="utf-8")
        result_json.update(
            {
                "exit_code": second.returncode,
                "patch_count": len(remapped),
                "modified_files": modified_files,
                "final_error_codes": error_code_counts(second_diags),
                "verification_passed": second.returncode == 0,
            }
        )
        if second.returncode == 0:
            result_json["status"] = "repaired"
        else:
            result_json["status"] = "not_repaired"
    finally:
        temp_handle.cleanup()

    write_result(subject_out / "baseline_result.json", result_json)


def run_rugpatcher_full(subject, out_root: Path, repair_root: Path) -> None:
    subject_out = ensure_dir(out_root / "rugpatcher_full" / subject.subject_id)
    result_json = {
        "baseline": "rugpatcher_full",
        "subject_id": subject.subject_id,
        "timestamp": utc_now(),
        "status": "unsupported",
        "notes": [],
    }
    repair_path = repair_root / subject.subject_id / "repair_result.json"
    if not repair_path.exists():
        result_json["notes"].append("repair_result.json not found; run run_repair.py first")
        return write_result(subject_out / "baseline_result.json", result_json)

    repair_result = json.loads(repair_path.read_text(encoding="utf-8"))
    repair_result = {**repair_result, "baseline": "rugpatcher_full", "copied_from": str(repair_path), "timestamp": utc_now()}
    write_result(subject_out / "baseline_result.json", repair_result)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run honest minimal baselines for RQ2 smoke tests.")
    parser.add_argument("--manifest", required=True, help="Subject manifest TOML path")
    parser.add_argument("--out", required=True, help="Baseline artifact output root")
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    out_root = ensure_dir(Path(args.out).resolve())
    repair_root = ROOT / "experiments" / "artifacts" / "repair_runs"
    subjects = load_manifest(manifest_path)

    for subject in subjects:
        run_no_repair(subject, out_root)
        run_compiler_suggestion_only(subject, out_root)
        run_rugpatcher_full(subject, out_root, repair_root)

    write_json(
        out_root / "TODO_baselines.json",
        {
            "todo_baselines": [
                "rule_only",
                "llm_raw_error",
                "llm_structured_context",
            ],
            "notes": "These baselines are intentionally not implemented yet in the smoke-test pipeline.",
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
