#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path

from common import ensure_dir, write_json


CATEGORY_BY_CODE = {
    "E0432": "unresolved import/path",
    "E0433": "unresolved import/path",
    "E0599": "method or associated item not found",
    "E0308": "type mismatch",
    "E0560": "struct field / constructor issue",
    "E0603": "visibility issue",
    "E0382": "ownership / borrow issue",
    "E0499": "ownership / borrow issue",
    "E0502": "ownership / borrow issue",
}


def category_for(code: str | None) -> str:
    if not code:
        return "unknown / other"
    return CATEGORY_BY_CODE.get(code, "unknown / other")


def load_jsonl(path: Path) -> list[dict]:
    rows = []
    if not path.exists():
        return rows
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        rows.append(json.loads(line))
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="Build smoke-test failure taxonomy tables for RQ1.")
    parser.add_argument("--input", required=True, help="Diagnostics artifact root")
    parser.add_argument("--out", required=True, help="Results root")
    args = parser.parse_args()

    input_root = Path(args.input).resolve()
    out_root = Path(args.out).resolve()
    tables_dir = ensure_dir(out_root / "tables")
    figures_dir = ensure_dir(out_root / "figures")
    raw_dir = ensure_dir(out_root / "raw")

    subject_rows = []
    by_code: dict[str, dict] = {}
    total_subjects = 0
    total_diagnostics = 0
    total_error_like = 0
    total_with_suggestions = 0
    total_without_suggestions = 0

    for subject_dir in sorted(path for path in input_root.iterdir() if path.is_dir()):
        total_subjects += 1
        metadata_path = subject_dir / "metadata.json"
        metadata = json.loads(metadata_path.read_text(encoding="utf-8")) if metadata_path.exists() else {}
        diagnostics = load_jsonl(subject_dir / "diagnostics.jsonl")
        total_diagnostics += len(diagnostics)
        with_suggestions = sum(1 for row in diagnostics if row.get("suggestions"))
        without_suggestions = len(diagnostics) - with_suggestions
        error_count = sum(1 for row in diagnostics if row.get("diagnostic_level") in {"error", "error: internal compiler error"})
        total_error_like += error_count
        total_with_suggestions += with_suggestions
        total_without_suggestions += without_suggestions

        code_counts: dict[str, int] = {}
        for row in diagnostics:
            code = row.get("error_code") or "NO_CODE"
            code_counts[code] = code_counts.get(code, 0) + 1
            entry = by_code.setdefault(
                code,
                {
                    "error_code": code,
                    "category": category_for(code),
                    "diagnostic_count": 0,
                    "subjects": set(),
                    "with_suggestions": 0,
                    "without_suggestions": 0,
                },
            )
            entry["diagnostic_count"] += 1
            entry["subjects"].add(subject_dir.name)
            if row.get("suggestions"):
                entry["with_suggestions"] += 1
            else:
                entry["without_suggestions"] += 1

        subject_rows.append(
            {
                "subject_id": subject_dir.name,
                "exit_code": metadata.get("exit_code"),
                "diagnostic_count": len(diagnostics),
                "error_diagnostic_count": error_count,
                "diagnostics_with_suggestions": with_suggestions,
                "diagnostics_without_suggestions": without_suggestions,
                "error_code_counts": json.dumps(code_counts, ensure_ascii=False, sort_keys=True),
                "status": metadata.get("status", "unknown"),
            }
        )

    taxonomy_rows = []
    for code, entry in sorted(by_code.items(), key=lambda item: (-item[1]["diagnostic_count"], item[0])):
        taxonomy_rows.append(
            {
                "error_code": code,
                "category": entry["category"],
                "diagnostic_count": entry["diagnostic_count"],
                "subject_count": len(entry["subjects"]),
                "with_suggestions": entry["with_suggestions"],
                "without_suggestions": entry["without_suggestions"],
            }
        )

    with (tables_dir / "rq1_failure_taxonomy.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=["error_code", "category", "diagnostic_count", "subject_count", "with_suggestions", "without_suggestions"])
        writer.writeheader()
        writer.writerows(taxonomy_rows)

    with (tables_dir / "rq1_subject_summary.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=["subject_id", "exit_code", "diagnostic_count", "error_diagnostic_count", "diagnostics_with_suggestions", "diagnostics_without_suggestions", "error_code_counts", "status"])
        writer.writeheader()
        writer.writerows(subject_rows)

    with (figures_dir / "rq1_error_code_counts.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=["error_code", "count", "category"])
        writer.writeheader()
        for row in taxonomy_rows:
            writer.writerow({"error_code": row["error_code"], "count": row["diagnostic_count"], "category": row["category"]})

    summary = {
        "subjects": total_subjects,
        "diagnostics": total_diagnostics,
        "error_like_diagnostics": total_error_like,
        "diagnostics_with_suggestions": total_with_suggestions,
        "diagnostics_without_suggestions": total_without_suggestions,
    }
    write_json(raw_dir / "rq1_failure_taxonomy_summary.json", summary)

    md_lines = [
        "# RQ1 Failure Taxonomy (Smoke Test Only)",
        "",
        "This preview is generated from the current **fixture-only** smoke-test subjects.",
        "This dataset was generated by the RuTeR pipeline.",
        "",
        f"- Subjects: {total_subjects}",
        f"- Diagnostics: {total_diagnostics}",
        f"- Error-like diagnostics: {total_error_like}",
        f"- Diagnostics with suggestions: {total_with_suggestions}",
        f"- Diagnostics without suggestions: {total_without_suggestions}",
        "",
        "| Error code | Category | Diagnostics | Subjects | With suggestions | Without suggestions |",
        "|---|---:|---:|---:|---:|---:|",
    ]
    for row in taxonomy_rows:
        md_lines.append(
            f"| {row['error_code']} | {row['category']} | {row['diagnostic_count']} | {row['subject_count']} | {row['with_suggestions']} | {row['without_suggestions']} |"
        )
    (tables_dir / "rq1_failure_taxonomy.md").write_text("\n".join(md_lines) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
