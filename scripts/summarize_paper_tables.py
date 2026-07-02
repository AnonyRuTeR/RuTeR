#!/usr/bin/env python3
"""Print paper-facing aggregate checks from committed RuTeR data files.

This script is intentionally read-only and dependency-free. It does not rerun
LLM-backed experiments; it extracts the main aggregate values from files under
``data/`` so reviewers can cross-check paper tables against the open artifact.
"""

from __future__ import annotations

import csv
import json
import re
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "data"


def pct(value: float) -> str:
    return f"{value * 100:.2f}%"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def extract_int(text: str, label: str) -> int:
    pattern = rf"{re.escape(label)}:\s*([0-9,]+)"
    match = re.search(pattern, text)
    if not match:
        raise ValueError(f"Could not find '{label}' in source text")
    return int(match.group(1).replace(",", ""))


def extract_percent(text: str, label: str) -> str:
    pattern = rf"{re.escape(label)}:\s*([0-9]+(?:\.[0-9]+)?%)"
    match = re.search(pattern, text)
    if not match:
        raise ValueError(f"Could not find '{label}' in source text")
    return match.group(1)


def extract_table_value(text: str, label: str) -> str:
    pattern = rf"^\|\s*{re.escape(label)}\s*\|\s*([^|]+?)\s*\|"
    match = re.search(pattern, text, re.MULTILINE)
    if not match:
        raise ValueError(f"Could not find table row '{label}' in source text")
    return match.group(1).strip()


def markdown_table(headers: Iterable[str], rows: Iterable[Iterable[object]]) -> str:
    headers = list(headers)
    lines = ["| " + " | ".join(headers) + " |"]
    lines.append("| " + " | ".join("---" for _ in headers) + " |")
    for row in rows:
        lines.append("| " + " | ".join(str(cell) for cell in row) + " |")
    return "\n".join(lines)


def rq1() -> str:
    conclusion = (DATA / "rq1" / "Pre_rug_gen_conclusion.md").read_text(encoding="utf-8")
    generated = extract_int(conclusion, "Generated samples")
    successes = extract_int(conclusion, "Compile successes")
    success_rate = extract_percent(conclusion, "Compile success rate")

    with (DATA / "rq1" / "rq1_error_code_occurrence_distribution.csv").open(newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))[:5]

    top_errors = ", ".join(
        f"{row['error_code']}={int(row['count'])} ({float(row['percent']):.2f}%)" for row in rows
    )
    return markdown_table(
        ["RQ", "Metric", "Value", "Source"],
        [
            ["RQ1", "Generated samples", generated, "data/rq1/Pre_rug_gen_conclusion.md"],
            ["RQ1", "Compile successes", successes, "data/rq1/Pre_rug_gen_conclusion.md"],
            ["RQ1", "Compile success rate", success_rate, "data/rq1/Pre_rug_gen_conclusion.md"],
            ["RQ1", "Top compiler errors", top_errors, "data/rq1/rq1_error_code_occurrence_distribution.csv"],
        ],
    )


def rq2() -> str:
    full = read_json(DATA / "rq2" / "aggregates" / "all_metrics.json")
    retry = read_json(DATA / "rq2" / "aggregates" / "rug_retry_baseline_metrics.json")
    phase = (DATA / "rq2" / "Phase1A_conclusion.md").read_text(encoding="utf-8")
    agent = (DATA / "rq2" / "Phase1A_agent_baseline.md").read_text(encoding="utf-8")

    direct_llm = re.search(
        r"\| DirectLLM-1shot \| ([0-9,]+) / ([0-9,]+) \| ([0-9.]+%) \| ([0-9,]+) / [0-9,]+ \| ([0-9.]+%) \|",
        phase,
    )
    if not direct_llm:
        raise ValueError("Could not parse DirectLLM-1shot row")

    return markdown_table(
        ["RQ", "System/metric", "Value", "Source"],
        [
            ["RQ2", "Full RuTeR strict success", f"{full['attempt_level']['strict_success']} / {full['attempt_level']['total']} ({pct(full['attempt_level']['strict_success_rate'])})", "data/rq2/aggregates/all_metrics.json"],
            ["RQ2", "Full RuTeR normalized success", f"{full['attempt_level']['normalized_success']} / {full['attempt_level']['total']} ({pct(full['attempt_level']['normalized_success_rate'])})", "data/rq2/aggregates/all_metrics.json"],
            ["RQ2", "Full RuTeR function salvage", f"{full['function_level']['resolved']} / {full['function_level']['total']} ({pct(full['function_level']['salvage_rate'])})", "data/rq2/aggregates/all_metrics.json"],
            ["RQ2", "Rule / LLM / unresolved paths", f"{full['rq2']['by_resolution_path']['rule']} / {full['rq2']['by_resolution_path']['full']} / {full['rq2']['by_resolution_path']['unresolved']}", "data/rq2/aggregates/all_metrics.json"],
            ["RQ2", "RUG Retry strict success", f"{retry['attempt_level']['rug_retry_success_total']} / {retry['attempt_level']['total']} ({pct(retry['attempt_level']['rug_retry_success_rate'])})", "data/rq2/aggregates/rug_retry_baseline_metrics.json"],
            ["RQ2", "DirectLLM-1shot strict success", f"{direct_llm.group(1)} / {direct_llm.group(2)} ({direct_llm.group(3)})", "data/rq2/Phase1A_conclusion.md"],
            ["RQ2", "DirectAgent-3round strict success", f"{extract_table_value(agent, 'Strict success')} / {full['attempt_level']['total']} ({extract_table_value(agent, 'Strict success rate')})", "data/rq2/Phase1A_agent_baseline.md"],
        ],
    )


def rq3() -> str:
    coverage = read_json(DATA / "rq3" / "aggregates" / "coverage_recovery_metrics.json")
    utility = read_json(DATA / "rq3" / "aggregates" / "utility_recovery_ratio_summary.json")
    summary = coverage["canonical_recovery"]["summary"]

    return markdown_table(
        ["RQ", "Metric", "Value", "Source"],
        [
            ["RQ3", "Original added coverage (lines/functions/regions)", f"{summary['original_success_added_coverage_lines_total']:.0f} / {summary['original_success_added_coverage_functions_total']:.0f} / {summary['original_success_added_coverage_regions_total']:.0f}", "data/rq3/aggregates/coverage_recovery_metrics.json"],
            ["RQ3", "Repaired added coverage (lines/functions/regions)", f"{summary['repaired_added_coverage_lines_total']:.0f} / {summary['repaired_added_coverage_functions_total']:.0f} / {summary['repaired_added_coverage_regions_total']:.0f}", "data/rq3/aggregates/coverage_recovery_metrics.json"],
            ["RQ3", "Replay eligible / executable / contributing", f"{summary['replay_eligible_repaired_total']} / {summary['compile_executable_repaired_total']} / {summary['coverage_contributing_repaired_total']}", "data/rq3/aggregates/coverage_recovery_metrics.json"],
            ["RQ3", "Positive repaired runs / crates", f"{summary['positive_repaired_run_total']} / {summary['positive_repaired_crate_total']}", "data/rq3/aggregates/coverage_recovery_metrics.json"],
            ["RQ3", "Aggregate utility recovery ratio", utility['totals']['aggregate_ratio_display'], "data/rq3/aggregates/utility_recovery_ratio_summary.json"],
            ["RQ3", "Finite per-run mean utility ratio", utility['totals']['finite_per_run_mean_ratio_display'], "data/rq3/aggregates/utility_recovery_ratio_summary.json"],
        ],
    )


def main() -> None:
    print("# RuTeR paper aggregate table checks")
    print()
    print(rq1())
    print()
    print(rq2())
    print()
    print(rq3())


if __name__ == "__main__":
    main()
