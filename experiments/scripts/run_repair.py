#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from common import RUTER_ROOT, ensure_dir, load_manifest, run_command, utc_now, write_json


def load_json(path: Path):
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Run RuTeR over smoke-test subjects for RQ2.")
    parser.add_argument("--manifest", required=True, help="Subject manifest TOML path")
    parser.add_argument("--out", required=True, help="Repair artifact output root")
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    out_root = ensure_dir(Path(args.out).resolve())
    subjects = load_manifest(manifest_path)

    cli_available = (RUTER_ROOT / "Cargo.toml").exists()

    for subject in subjects:
        subject_out = ensure_dir(out_root / subject.subject_id)
        resolved = subject.resolved_path()
        notes = []
        repair_result = {
            "subject_id": subject.subject_id,
            "status": "unsupported",
            "initial_error_codes": {},
            "final_error_codes": {},
            "patch_count": "unknown",
            "modified_files": [],
            "production_files_modified": "unknown",
            "target_test_preserved": "unknown",
            "verification_passed": "unknown",
            "notes": notes,
        }

        if not cli_available:
            notes.append("ruter/Cargo.toml not found; RuTeR CLI unavailable")
            write_json(subject_out / "repair_result.json", repair_result)
            continue
        if subject.source_type != "local_fixture" or resolved is None or not resolved.exists():
            notes.append("only existing local_fixture subjects are supported by run_repair.py right now")
            write_json(subject_out / "repair_result.json", repair_result)
            continue

        rugpatcher_artifacts = subject_out / "rugpatcher_artifacts"
        ensure_dir(rugpatcher_artifacts)
        command = [
            "cargo",
            "run",
            "--manifest-path",
            str(RUTER_ROOT / "Cargo.toml"),
            "--",
            "fix",
            str(resolved),
            "--artifacts-dir",
            str(rugpatcher_artifacts),
        ]
        result = run_command(command, cwd=RUTER_ROOT)
        (subject_out / "raw_stdout.log").write_text(result.stdout, encoding="utf-8")
        (subject_out / "raw_stderr.log").write_text(result.stderr, encoding="utf-8")

        summary = load_json(rugpatcher_artifacts / "6_summary.json")
        plan = load_json(rugpatcher_artifacts / "3_plan.json")
        diagnostics_path = Path("experiments/artifacts/diagnostics") / subject.subject_id / "diagnostics.jsonl"
        repair_result.update(
            {
                "timestamp": utc_now(),
                "command": command,
                "cwd": str(RUTER_ROOT),
                "exit_code": result.returncode,
                "artifact_root": str(rugpatcher_artifacts),
                "input_diagnostic_path": str(diagnostics_path) if diagnostics_path.exists() else "unknown",
            }
        )

        if summary is None:
            repair_result["status"] = "failed_to_run"
            notes.append("RuTeR run did not produce 6_summary.json")
            write_json(subject_out / "repair_result.json", repair_result)
            continue

        repair_result["initial_error_codes"] = summary.get("initial_error_by_code", {})
        repair_result["final_error_codes"] = summary.get("remaining_error_by_code", {})
        repair_result["patch_count"] = summary.get("planned_action_count", "unknown")
        repair_result["verification_passed"] = summary.get("patch_verify_check_passed", "unknown")

        modified_files = []
        if isinstance(plan, dict):
            for file_entry in plan.get("files", []):
                file_path = file_entry.get("file_path")
                if file_path:
                    modified_files.append(file_path)
        repair_result["modified_files"] = modified_files

        remaining_total = summary.get("remaining_error_total")
        verification = summary.get("patch_verify_check_passed")
        if result.returncode == 0 and remaining_total == 0 and verification is True:
            repair_result["status"] = "repaired"
        elif result.returncode in {0, 7}:
            repair_result["status"] = "not_repaired"
        else:
            repair_result["status"] = "failed_to_run"

        if modified_files:
            # RuTeR edits test code inside existing crate files, often under src/*.rs.
            # Without semantic span post-analysis, whether a modified file is a production file
            # cannot always be decided from summary artifacts alone.
            if all("/tests/" in path or path.startswith("tests/") for path in modified_files):
                repair_result["production_files_modified"] = False
            else:
                repair_result["production_files_modified"] = "unknown"

        if summary.get("partial_pending_llm"):
            notes.append("run ended with partial pending LLM / unresolved functions")
        if summary.get("llm_handoff_count"):
            notes.append(f"llm_handoff_count={summary.get('llm_handoff_count')}")
        write_json(subject_out / "repair_result.json", repair_result)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
