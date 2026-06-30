# Experiment Data

This directory contains the aggregate experiment data organized by research question.

## RQ1: Failure Characteristics
`rq1/` — Compile success rates, error code Pareto distributions, cross-crate/cross-model analysis.
18 RUG runs across 10 crates and 9 LLM models.

## RQ2: Repair Effectiveness
`rq2/` — 5-system comparison (RuTeR Full, Rule-only, DirectLLM-1shot, DirectAgent-3round, RUG Retry),
1,858 frozen attempts each. Per-error, per-crate, per-model repair rates.

## RQ3: Downstream Utility & Generalization
`rq3/` — Coverage recovery metrics (Lines/Functions/Regions), quality guard metrics (oracle strength),
cost-utility analysis, and cross-source generalization (5 test generators).

## Data Completeness

Data completeness (audited 2026-05-27):
- **RQ1**: 85% — Core data intact; chrono/mio detailed_log missing
- **RQ2**: 95% — Complete per-attempt data for all 5 systems
- **RQ3**: 70% — Coverage recovery complete; mutation testing not yet implemented

For per-attempt raw data (batch_result.json for each of 9,290 attempts), see the
accompanying data archive or contact the authors.
