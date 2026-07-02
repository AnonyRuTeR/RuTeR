# Experiment Data

This directory contains the aggregate artifacts used to check the paper-facing
results for each research question. The files are pre-computed outputs from the
RuTeR evaluation pipeline; they are intended for result inspection and table
cross-checking without requiring online LLM APIs.

## RQ1: Failure Characteristics

`rq1/` contains aggregate failure-characterization outputs, including:

- `Pre_rug_gen_conclusion.md` — summary of the RUG generation study.
- `rq1_error_code_occurrence_distribution.csv` — compiler error-code occurrence distribution.
- `rq1_failure_taxonomy.{csv,md}` — failure-category taxonomy.

## RQ2: Repair Effectiveness

`rq2/` contains aggregate repair-effectiveness outputs, including:

- `Phase1A_conclusion.md` — paper-facing summary of the frozen replay evaluation.
- `Phase1A_agent_baseline.md` — DirectAgent-3round baseline summary.
- `aggregates/all_metrics.json` — Full RuTeR aggregate metrics.
- `aggregates/rug_retry_baseline_metrics.json` — RUG Retry baseline metrics.

## RQ3: Downstream Utility and Generalization

`rq3/` contains aggregate downstream-utility outputs, including:

- `Phase1A_Plus_conclusion.md` and `Phase1B_conclusion.md` — summary reports.
- `aggregates/coverage_recovery_metrics.json` — canonical coverage-recovery totals.
- `aggregates/utility_recovery_ratio_*` — utility-recovery ratio tables and summaries.
- `phase_data/` — Phase 1A+ aggregate CSV/JSON tables used by the downstream analysis.

## Quick aggregate check

Run the repository-level summary script to print the main aggregate values used
for paper table checks:

```bash
python3 scripts/summarize_paper_tables.py
```

The script reads only files committed under `data/` and does not require network
access or LLM credentials.
