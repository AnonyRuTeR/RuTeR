#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path

from common import (
    default_subject_command,
    ensure_dir,
    error_code_counts,
    extract_diagnostics_from_cargo_stdout,
    load_manifest,
    run_command,
    utc_now,
    write_json,
    write_jsonl,
)


def main() -> int:
    parser = argparse.ArgumentParser(description="Collect rustc JSON diagnostics for RQ1 smoke tests.")
    parser.add_argument("--manifest", required=True, help="Subject manifest TOML path")
    parser.add_argument("--out", required=True, help="Output diagnostics directory")
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    out_root = ensure_dir(Path(args.out).resolve())
    subjects = load_manifest(manifest_path)

    for subject in subjects:
        subject_dir = out_root / subject.subject_id
        ensure_dir(subject_dir)
        metadata = {
            "subject_id": subject.subject_id,
            "crate_name": subject.crate_name,
            "source_type": subject.source_type,
            "rust_toolchain": subject.rust_toolchain,
            "timestamp": utc_now(),
            "status": "unknown",
            "notes": subject.notes,
        }
        resolved = subject.resolved_path()
        if subject.source_type != "local_fixture" or resolved is None or not resolved.exists():
            metadata.update(
                {
                    "status": "unsupported",
                    "failure_reason": "only existing local_fixture subjects are supported by collect_diagnostics.py right now",
                }
            )
            write_json(subject_dir / "metadata.json", metadata)
            write_jsonl(subject_dir / "diagnostics.jsonl", [])
            (subject_dir / "raw_stdout.log").write_text("", encoding="utf-8")
            (subject_dir / "raw_stderr.log").write_text("", encoding="utf-8")
            continue

        command = default_subject_command(resolved)
        result = run_command(command, cwd=resolved)
        diagnostics = extract_diagnostics_from_cargo_stdout(subject.subject_id, resolved, result.stdout)

        (subject_dir / "raw_stdout.log").write_text(result.stdout, encoding="utf-8")
        (subject_dir / "raw_stderr.log").write_text(result.stderr, encoding="utf-8")
        write_jsonl(subject_dir / "diagnostics.jsonl", diagnostics)

        metadata.update(
            {
                "status": "ok" if result.returncode == 0 else "compile_failed",
                "command": command,
                "cwd": str(resolved),
                "exit_code": result.returncode,
                "diagnostic_count": len(diagnostics),
                "error_code_counts": error_code_counts(diagnostics),
                "diagnostics_with_suggestions": sum(1 for row in diagnostics if row.get("suggestions")),
                "diagnostics_without_suggestions": sum(1 for row in diagnostics if not row.get("suggestions")),
            }
        )
        write_json(subject_dir / "metadata.json", metadata)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
