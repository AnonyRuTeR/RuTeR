#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import math
import statistics
from collections import defaultdict
from pathlib import Path
from typing import Any


def load_json(path: Path, default: Any = None) -> Any:
    if not path.exists():
        return default
    return json.loads(path.read_text(encoding="utf-8"))


def percentile(values: list[int], p: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    if len(ordered) == 1:
        return float(ordered[0])
    rank = (len(ordered) - 1) * p
    lower = math.floor(rank)
    upper = math.ceil(rank)
    if lower == upper:
        return float(ordered[lower])
    weight = rank - lower
    return ordered[lower] * (1.0 - weight) + ordered[upper] * weight


def describe(values: list[int]) -> dict[str, float | int | None]:
    if not values:
        return {
            "count": 0,
            "sum": 0,
            "mean": None,
            "median": None,
            "p25": None,
            "p75": None,
            "p95": None,
            "mean_ci95_low": None,
            "mean_ci95_high": None,
        }
    mean = statistics.fmean(values)
    if len(values) >= 2:
        half_width = 1.96 * statistics.stdev(values) / math.sqrt(len(values))
        ci_low = mean - half_width
        ci_high = mean + half_width
    else:
        ci_low = None
        ci_high = None
    return {
        "count": len(values),
        "sum": sum(values),
        "mean": mean,
        "median": statistics.median(values),
        "p25": percentile(values, 0.25),
        "p75": percentile(values, 0.75),
        "p95": percentile(values, 0.95),
        "mean_ci95_low": ci_low,
        "mean_ci95_high": ci_high,
    }


def stratified_weighted_mean_ci95(
    rows: list[dict[str, Any]], field: str
) -> dict[str, float | bool | None]:
    if not rows:
        return {
            "mean": None,
            "ci95_low": None,
            "ci95_high": None,
            "variance_estimable": False,
        }
    total_population_weight = sum(float(row["sampling_weight"]) for row in rows)
    weighted_total = sum(
        float(row["sampling_weight"]) * float(row[field]) for row in rows
    )
    mean = weighted_total / total_population_weight
    if any(str(row["sampling_stratum"]) == "purposeful_smoke" for row in rows):
        ordinary = describe([int(row[field]) for row in rows])
        return {
            "mean": mean,
            "ci95_low": ordinary["mean_ci95_low"],
            "ci95_high": ordinary["mean_ci95_high"],
            "variance_estimable": len(rows) >= 2,
        }
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        grouped[str(row["sampling_stratum"])].append(row)
    variance = 0.0
    estimable = True
    for items in grouped.values():
        n_h = len(items)
        population_h = sum(float(item["sampling_weight"]) for item in items)
        if n_h < 2:
            if population_h > n_h + 1e-9:
                estimable = False
            continue
        values = [float(item[field]) for item in items]
        sample_variance = statistics.variance(values)
        fraction = min(1.0, n_h / population_h) if population_h else 1.0
        variance += (
            (population_h / total_population_weight) ** 2
            * (1.0 - fraction)
            * sample_variance
            / n_h
        )
    if not estimable:
        return {
            "mean": mean,
            "ci95_low": None,
            "ci95_high": None,
            "variance_estimable": False,
        }
    half_width = 1.96 * math.sqrt(max(0.0, variance))
    return {
        "mean": mean,
        "ci95_low": max(0.0, mean - half_width),
        "ci95_high": mean + half_width,
        "variance_estimable": True,
    }


def is_strict_success(summary: dict[str, Any], case_result: dict[str, Any]) -> bool:
    if isinstance(case_result.get("strict_success"), bool):
        return bool(case_result["strict_success"])
    return bool(summary.get("patch_verify_check_passed")) and int(
        summary.get("remaining_error_total") or 0
    ) == 0


def collect_case_rows(root: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    # Archived infrastructure attempts are retained for audit, not counted twice.
    for result_path in sorted(root.glob("cases/*/case_result.json")):
        case_result = load_json(result_path, {})
        if not isinstance(case_result, dict):
            continue
        artifact_dir_raw = case_result.get("artifact_dir")
        artifact_dir = (
            Path(artifact_dir_raw)
            if isinstance(artifact_dir_raw, str) and artifact_dir_raw
            else result_path.parent / "ruter_artifacts"
        )
        if not artifact_dir.is_absolute():
            artifact_dir = (result_path.parent / artifact_dir).resolve()
        usage_path = artifact_dir / "4_llm_usage.json"
        summary_path = artifact_dir / "6_summary.json"
        usage_artifact = load_json(usage_path, {})
        summary = load_json(summary_path, {})
        usage_summary = (
            usage_artifact.get("summary", {})
            if isinstance(usage_artifact, dict)
            else {}
        )
        request_count = int(usage_summary.get("request_count") or 0)
        usage_reported = int(usage_summary.get("usage_reported_request_count") or 0)
        usage_missing = int(usage_summary.get("usage_missing_request_count") or 0)
        eligible = bool(case_result.get("eligible", True))
        usage_complete = (
            usage_path.exists()
            and usage_missing == 0
            and usage_reported == request_count
        )
        request_rows = (
            usage_artifact.get("requests", [])
            if isinstance(usage_artifact, dict)
            else []
        )
        returned_models = sorted(
            {
                str(item["returned_model"])
                for item in request_rows
                if isinstance(item, dict) and item.get("returned_model")
            }
        )
        configured_model = (
            usage_artifact.get("configured_model")
            if isinstance(usage_artifact, dict)
            else case_result.get("repair_model")
        ) or case_result.get("repair_model") or ""
        allowed_models = {configured_model}
        if configured_model == "gemini-2.5-flash-nothinking":
            allowed_models.add("gemini-2.5-flash")
        model_valid = request_count == 0 or (
            len(request_rows) == request_count
            and all(
                isinstance(item, dict)
                and str(item.get("returned_model") or "") in allowed_models
                for item in request_rows
            )
        )
        row = {
            "case_id": case_result.get("case_id") or result_path.parent.name,
            "attempt_uid": case_result.get("attempt_uid") or "",
            "run_id": case_result.get("run_id") or "",
            "crate": case_result.get("crate") or "",
            "generation_model": case_result.get("generation_model") or "",
            "repair_model": configured_model,
            "returned_models": ",".join(returned_models),
            "returned_model_differs": bool(returned_models)
            and any(model != configured_model for model in returned_models),
            "returned_model_valid": model_valid,
            "error_codes": ",".join(case_result.get("error_codes") or []),
            "difficulty": case_result.get("difficulty") or "",
            "sampling_stratum": case_result.get("sampling_stratum") or "",
            "sampling_weight": float(case_result.get("sampling_weight") or 1.0),
            "exit_code": case_result.get("exit_code"),
            "status": case_result.get("status") or "",
            "eligible": eligible,
            "exclusion_reason": case_result.get("exclusion_reason") or "",
            "strict_success": is_strict_success(
                summary if isinstance(summary, dict) else {}, case_result
            ),
            "usage_artifact_present": usage_path.exists(),
            "request_count": request_count,
            "successful_request_count": int(
                usage_summary.get("successful_request_count") or 0
            ),
            "failed_request_count": int(usage_summary.get("failed_request_count") or 0),
            "usage_reported_request_count": usage_reported,
            "usage_missing_request_count": usage_missing,
            "usage_complete": usage_complete,
            "analysis_included": eligible and usage_complete and model_valid,
            "input_tokens": int(usage_summary.get("input_tokens") or 0),
            "output_tokens": int(usage_summary.get("output_tokens") or 0),
            "total_tokens": int(usage_summary.get("total_tokens") or 0),
            "cached_input_tokens": int(
                usage_summary.get("cached_input_tokens") or 0
            ),
            "reasoning_tokens": int(usage_summary.get("reasoning_tokens") or 0),
            "duration_sec": case_result.get("duration_sec"),
            "result_path": str(result_path),
            "usage_path": str(usage_path),
        }
        rows.append(row)
    return rows


def aggregate(rows: list[dict[str, Any]]) -> dict[str, Any]:
    analyzed = [row for row in rows if row["analysis_included"]]
    excluded = [row for row in rows if not row["analysis_included"]]
    all_tokens = [int(row["total_tokens"]) for row in analyzed]
    invoked = [row for row in analyzed if int(row["request_count"]) > 0]
    invoked_tokens = [int(row["total_tokens"]) for row in invoked]
    strict = [row for row in analyzed if row["strict_success"]]
    complete = [row for row in rows if row["usage_complete"]]
    request_count = sum(int(row["request_count"]) for row in analyzed)
    usage_reported = sum(
        int(row["usage_reported_request_count"]) for row in analyzed
    )
    usage_missing = sum(int(row["usage_missing_request_count"]) for row in analyzed)
    totals = {
        "input_tokens": sum(int(row["input_tokens"]) for row in analyzed),
        "output_tokens": sum(int(row["output_tokens"]) for row in analyzed),
        "total_tokens": sum(int(row["total_tokens"]) for row in analyzed),
        "cached_input_tokens": sum(
            int(row["cached_input_tokens"]) for row in analyzed
        ),
        "reasoning_tokens": sum(int(row["reasoning_tokens"]) for row in analyzed),
    }
    weighted_token_sum = sum(
        float(row["sampling_weight"]) * int(row["total_tokens"])
        for row in analyzed
    )
    invoked_weight_sum = sum(float(row["sampling_weight"]) for row in invoked)
    invoked_weighted_token_sum = sum(
        float(row["sampling_weight"]) * int(row["total_tokens"])
        for row in invoked
    )
    strict_weight_sum = sum(float(row["sampling_weight"]) for row in strict)
    weighted_all = stratified_weighted_mean_ci95(analyzed, "total_tokens")
    weighted_input = stratified_weighted_mean_ci95(analyzed, "input_tokens")
    weighted_output = stratified_weighted_mean_ci95(analyzed, "output_tokens")
    by_crate: dict[str, Any] = {}
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in analyzed:
        grouped[str(row["crate"])].append(row)
    for crate, items in sorted(grouped.items()):
        values = [int(item["total_tokens"]) for item in items]
        by_crate[crate] = {
            "case_count": len(items),
            "llm_invoked_case_count": sum(
                int(item["request_count"]) > 0 for item in items
            ),
            "strict_success_count": sum(bool(item["strict_success"]) for item in items),
            "total_tokens": sum(values),
            "total_tokens_distribution": describe(values),
        }
    return {
        "case_count": len(rows),
        "executed_case_count": len(rows),
        "analyzed_case_count": len(analyzed),
        "excluded_case_count": len(excluded),
        "excluded_cases": [
            {
                "case_id": row["case_id"],
                "status": row["status"],
                "reason": row["exclusion_reason"]
                or ("token usage incomplete" if not row["usage_complete"] else "ineligible"),
            }
            for row in excluded
        ],
        "llm_invoked_case_count": len(invoked),
        "zero_llm_case_count": len(analyzed) - len(invoked),
        "strict_success_count": len(strict),
        "strict_success_rate": len(strict) / len(analyzed) if analyzed else None,
        "sampling_weighted_strict_success_rate": (
            strict_weight_sum
            / sum(float(row["sampling_weight"]) for row in analyzed)
            if analyzed
            else None
        ),
        "sampling_weighted_llm_invocation_rate": (
            invoked_weight_sum
            / sum(float(row["sampling_weight"]) for row in analyzed)
            if analyzed
            else None
        ),
        "usage_complete_case_count": len(complete),
        "usage_incomplete_case_count": len(rows) - len(complete),
        "request_count": request_count,
        "usage_reported_request_count": usage_reported,
        "usage_missing_request_count": usage_missing,
        "usage_coverage_rate": usage_reported / request_count if request_count else 1.0,
        "returned_model_mismatch_case_count": sum(
            bool(row["returned_model_differs"]) for row in analyzed
        ),
        "returned_model_invalid_case_count": sum(
            not bool(row["returned_model_valid"]) for row in rows
        ),
        "totals": totals,
        "all_cases_total_tokens": describe(all_tokens),
        "llm_invoked_cases_total_tokens": describe(invoked_tokens),
        "sampling_weighted_mean_tokens_all_cases": weighted_all["mean"],
        "sampling_weighted_mean_tokens_all_cases_ci95_low": weighted_all["ci95_low"],
        "sampling_weighted_mean_tokens_all_cases_ci95_high": weighted_all["ci95_high"],
        "sampling_weighted_variance_estimable": weighted_all["variance_estimable"],
        "sampling_weighted_mean_input_tokens_all_cases": weighted_input["mean"],
        "sampling_weighted_mean_output_tokens_all_cases": weighted_output["mean"],
        "sampling_weighted_mean_tokens_llm_invoked_cases": (
            invoked_weighted_token_sum / invoked_weight_sum
            if invoked_weight_sum
            else None
        ),
        "tokens_per_strict_repair": (
            totals["total_tokens"] / len(strict) if strict else None
        ),
        "sampling_weighted_tokens_per_strict_repair": (
            weighted_token_sum / strict_weight_sum if strict_weight_sum else None
        ),
        "api_requests_per_analyzed_case": (
            request_count / len(analyzed) if analyzed else None
        ),
        "api_requests_per_llm_invoked_case": (
            request_count / len(invoked) if invoked else None
        ),
        "tokens_per_api_request": (
            totals["total_tokens"] / request_count if request_count else None
        ),
        "by_crate": by_crate,
    }


def render_markdown(summary: dict[str, Any]) -> str:
    overall = summary["all_cases_total_tokens"]
    invoked = summary["llm_invoked_cases_total_tokens"]

    def percentage(value: Any) -> str:
        return "N/A" if value is None else f"{float(value):.2%}"

    lines = [
        "# RuTeR Token Usage Summary",
        "",
        "| Metric | Value |",
        "|---|---:|",
        f"| Executed cases | {summary['executed_case_count']} |",
        f"| Analyzed cases | {summary['analyzed_case_count']} |",
        f"| Excluded cases | {summary['excluded_case_count']} |",
        f"| LLM-invoked cases | {summary['llm_invoked_case_count']} |",
        f"| Zero-LLM cases | {summary['zero_llm_case_count']} |",
        f"| Strict repairs | {summary['strict_success_count']} |",
        f"| Strict repair rate (sample) | {percentage(summary['strict_success_rate'])} |",
        f"| Strict repair rate (weighted) | {percentage(summary['sampling_weighted_strict_success_rate'])} |",
        f"| API requests | {summary['request_count']} |",
        f"| Usage coverage | {percentage(summary['usage_coverage_rate'])} |",
        f"| Returned-model mismatch cases | {summary['returned_model_mismatch_case_count']} |",
        f"| Input tokens | {summary['totals']['input_tokens']} |",
        f"| Output tokens | {summary['totals']['output_tokens']} |",
        f"| Total tokens | {summary['totals']['total_tokens']} |",
        f"| Mean tokens / all case | {overall['mean'] or 0:.2f} |",
        f"| Approx. 95% CI for mean | [{overall['mean_ci95_low'] or 0:.2f}, {overall['mean_ci95_high'] or 0:.2f}] |",
        f"| Sampling-weighted mean tokens / all case | {summary['sampling_weighted_mean_tokens_all_cases'] or 0:.2f} |",
        f"| Weighted 95% CI for mean | [{summary['sampling_weighted_mean_tokens_all_cases_ci95_low'] or 0:.2f}, {summary['sampling_weighted_mean_tokens_all_cases_ci95_high'] or 0:.2f}] |",
        f"| Median tokens / all case | {overall['median'] or 0:.2f} |",
        f"| P95 tokens / all case | {overall['p95'] or 0:.2f} |",
        f"| Mean tokens / LLM-invoked case | {invoked['mean'] or 0:.2f} |",
        f"| Weighted mean tokens / LLM-invoked case | {summary['sampling_weighted_mean_tokens_llm_invoked_cases'] or 0:.2f} |",
        f"| Weighted mean input tokens / all case | {summary['sampling_weighted_mean_input_tokens_all_cases'] or 0:.2f} |",
        f"| Weighted mean output tokens / all case | {summary['sampling_weighted_mean_output_tokens_all_cases'] or 0:.2f} |",
        f"| API requests / analyzed case | {summary['api_requests_per_analyzed_case'] or 0:.2f} |",
        f"| Tokens / API request | {summary['tokens_per_api_request'] or 0:.2f} |",
        f"| Tokens / strict repair | {summary['tokens_per_strict_repair'] or 0:.2f} |",
        f"| Weighted tokens / strict repair | {summary['sampling_weighted_tokens_per_strict_repair'] or 0:.2f} |",
        "",
        "Missing usage is never treated as a zero-token request. Zero-token cases are only cases with a present usage artifact and no API request.",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Aggregate RuTeR provider-reported token usage.")
    parser.add_argument("--experiment-root", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    experiment_root = Path(args.experiment_root).resolve()
    out = Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    rows = collect_case_rows(experiment_root)
    if not rows:
        raise SystemExit(f"no case_result.json files found under {experiment_root}")
    summary = aggregate(rows)

    fieldnames = list(rows[0].keys())
    with (out / "token_usage_cases.csv").open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    (out / "token_usage_summary.json").write_text(
        json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    (out / "token_usage_summary.md").write_text(
        render_markdown(summary), encoding="utf-8"
    )
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
