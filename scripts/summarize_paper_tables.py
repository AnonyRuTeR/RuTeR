#!/usr/bin/env python3
"""Print submitted-paper aggregate checks from committed RuTeR artifacts.

This script is read-only and dependency-free. It does not rerun LLM-backed
experiments; it prints the submitted-paper table values together with the local
aggregate files that support the checks.
"""

from __future__ import annotations

import csv
import json
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "data"

RQ1_SUBJECT_ROWS = [
    ("crc32fast", 39, "74.4%"),
    ("ryu", 52, "67.3%"),
    ("itoa", 29, "51.7%"),
    ("semver", 130, "40.0%"),
    ("rand", 682, "25.5%"),
    ("humantime", 124, "25.0%"),
    ("chrono", 1510, "22.1%"),
    ("log", 213, "21.1%"),
    ("rustc-demangle", 180, "6.7%"),
    ("mio", 224, "1.8%"),
    ("Total", 3183, "22.97%"),
]

RQ1_CROSS_MODEL_ROWS = [
    ("Gemini 2.5 Flash (think)", 98, 28, "28.4%"),
    ("Gemini 2.5 Flash (no think)", 113, 28, "25.0%"),
    ("gpt-4.1-mini", 123, 22, "18.0%"),
    ("deepseek-v3", 99, 19, "19.0%"),
    ("gpt-5-nano", 143, 21, "14.4%"),
    ("gpt-4o-mini", 140, 19, "13.8%"),
    ("gpt-4.1-nano", 146, 18, "12.1%"),
    ("claude-3-5-haiku", 140, 17, "12.1%"),
    ("gpt-3.5-turbo", 130, 16, "12.1%"),
]

RQ2_EFFECTIVENESS_ROWS = [
    ("Compiler-suggestion-only", 133, "7.2%", "---"),
    ("RUG Retry", 195, "10.5%", "100 / 566 (17.7%)"),
    ("DirectLLM-1shot", 217, "11.7%", "146 / 566 (25.8%)"),
    ("Rule-only", 393, "21.2%", "---"),
    ("DirectAgent-3round", 565, "30.4%", "281 / 566 (49.6%)"),
    ("Full RuTeR", 869, "46.8%", "366 / 566 (64.7%)"),
]

RQ3_CROSS_MODEL_REPAIR_ROWS = [
    ("gpt-4.1-mini", 101, "73.3%", "18.0%"),
    ("gpt-4.1-nano", 128, "68.8%", "12.1%"),
    ("deepseek-v3", 80, "67.5%", "19.0%"),
    ("claude-3-5-haiku", 123, "65.0%", "12.1%"),
    ("Gemini 2.5 Flash (no think)", 85, "63.5%", "25.0%"),
    ("Gemini 2.5 Flash (think)", 70, "60.0%", "28.4%"),
    ("gpt-4o-mini", 121, "51.2%", "13.8%"),
    ("gpt-5-nano", 122, "50.0%", "14.4%"),
    ("gpt-3.5-turbo", 114, "42.1%", "12.1%"),
]


def pct(value: float, digits: int = 1) -> str:
    return f"{value * 100:.{digits}f}%"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def markdown_table(headers: Iterable[str], rows: Iterable[Iterable[object]]) -> str:
    headers = list(headers)
    lines = ["| " + " | ".join(headers) + " |"]
    lines.append("| " + " | ".join("---" for _ in headers) + " |")
    for row in rows:
        lines.append("| " + " | ".join(str(cell) for cell in row) + " |")
    return "\n".join(lines)


def top_errors() -> str:
    path = DATA / "rq1" / "rq1_error_code_occurrence_distribution.csv"
    with path.open(newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))[:5]
    return ", ".join(
        f"{row['error_code']}={int(row['count'])} ({float(row['percent']):.1f}%)" for row in rows
    )


def rq1() -> str:
    return "\n\n".join(
        [
            markdown_table(
                ["RQ1 subject crate", "Generate attempts", "Compile success rate"],
                RQ1_SUBJECT_ROWS,
            ),
            markdown_table(
                ["RQ1 cross-model pilot", "Total attempts", "Compiled", "Compile rate"],
                RQ1_CROSS_MODEL_ROWS,
            ),
            markdown_table(
                ["RQ1 diagnostic check", "Value", "Source"],
                [["Top compiler errors", top_errors(), "data/rq1/rq1_error_code_occurrence_distribution.csv"]],
            ),
        ]
    )


def rq2() -> str:
    full = read_json(DATA / "rq2" / "aggregates" / "all_metrics.json")
    retry = read_json(DATA / "rq2" / "aggregates" / "rug_retry_baseline_metrics.json")

    expected = {
        "Full RuTeR": full["attempt_level"]["strict_success"],
        "Rule-only": full["rq2"]["by_resolution_path"]["rule"],
        "RUG Retry": retry["attempt_level"]["rug_retry_success_total"],
    }
    for system, value in expected.items():
        row = next(r for r in RQ2_EFFECTIVENESS_ROWS if r[0] == system)
        if row[1] != value:
            raise ValueError(f"{system} mismatch: table={row[1]}, aggregate={value}")
    if full["function_level"]["resolved"] != 366 or full["function_level"]["total"] != 566:
        raise ValueError("Full RuTeR function-level aggregate mismatch")

    return markdown_table(
        ["RQ2 system", "Repair count", "Rate", "Function salvage", "Primary local source"],
        [
            [*row, "data/rq2/aggregates/all_metrics.json"]
            if row[0] in {"Rule-only", "Full RuTeR"}
            else [*row, "data/rq2/aggregates/rug_retry_baseline_metrics.json"]
            if row[0] == "RUG Retry"
            else [*row, "data/rq2/Phase1A_conclusion.md"]
            for row in RQ2_EFFECTIVENESS_ROWS
        ],
    )


def rq3() -> str:
    coverage = read_json(DATA / "rq3" / "aggregates" / "coverage_recovery_metrics.json")
    utility = read_json(DATA / "rq3" / "aggregates" / "utility_recovery_ratio_summary.json")
    run_metrics = list(csv.DictReader((DATA / "rq2" / "aggregates" / "all_run_metrics.csv").open(encoding="utf-8")))
    summary = coverage["canonical_recovery"]["summary"]

    coverage_rows = [
        [
            "Lines",
            int(summary["original_success_added_coverage_lines_total"]),
            int(summary["original_success_added_coverage_lines_total"] + summary["repaired_added_coverage_lines_total"]),
            int(summary["repaired_added_coverage_lines_total"]),
            utility["totals"]["aggregate_ratio_display"],
        ],
        [
            "Functions",
            int(summary["original_success_added_coverage_functions_total"]),
            int(summary["original_success_added_coverage_functions_total"] + summary["repaired_added_coverage_functions_total"]),
            int(summary["repaired_added_coverage_functions_total"]),
            pct(summary["repaired_added_coverage_functions_total"] / summary["original_success_added_coverage_functions_total"]),
        ],
        [
            "Regions",
            int(summary["original_success_added_coverage_regions_total"]),
            int(summary["original_success_added_coverage_regions_total"] + summary["repaired_added_coverage_regions_total"]),
            int(summary["repaired_added_coverage_regions_total"]),
            pct(summary["repaired_added_coverage_regions_total"] / summary["original_success_added_coverage_regions_total"]),
        ],
    ]

    repair_by_model = {
        row["run_id"].split("humantime_", 1)[1].rsplit("_202", 1)[0]: (
            int(row["attempt_total"]), pct(float(row["strict_success_rate"]), 1)
        )
        for row in run_metrics
        if row["run_id"].startswith("humantime_")
    }
    for model, failures, rate, _compile_rate in RQ3_CROSS_MODEL_REPAIR_ROWS:
        key = {
            "Gemini 2.5 Flash (no think)": "gemini-2.5-flash-nothinking",
            "Gemini 2.5 Flash (think)": "gemini-2.5-flash-thinking",
            "claude-3-5-haiku": "claude-3-5-haiku-20241022",
        }.get(model, model)
        aggregate_failures, aggregate_rate = repair_by_model[key]
        if failures != aggregate_failures or rate != aggregate_rate:
            raise ValueError(f"{model} repair mismatch: table={failures}/{rate}, aggregate={aggregate_failures}/{aggregate_rate}")

    return "\n\n".join(
        [
            markdown_table(
                ["RQ3 coverage metric", "Baseline", "After repair", "Improvement", "Improvement %"],
                coverage_rows,
            ),
            markdown_table(
                ["RQ3 distribution", "Value", "Source"],
                [
                    ["Positive repaired runs", f"{summary['positive_repaired_run_total']} / {coverage['canonical_recovery']['run_total']}", "data/rq3/aggregates/coverage_recovery_metrics.json"],
                    ["Positive repaired crates", f"{summary['positive_repaired_crate_total']} / {len(coverage['selected_crates'])}", "data/rq3/aggregates/coverage_recovery_metrics.json"],
                ],
            ),
            markdown_table(
                ["RQ3 humantime model", "Failure attempts", "Repair rate", "RQ1 compile rate"],
                RQ3_CROSS_MODEL_REPAIR_ROWS,
            ),
        ]
    )


def main() -> None:
    print("# RuTeR submitted-paper aggregate table checks")
    print()
    print(rq1())
    print()
    print(rq2())
    print()
    print(rq3())


if __name__ == "__main__":
    main()
