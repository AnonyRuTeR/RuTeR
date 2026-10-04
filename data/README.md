# Experiment Data

This directory contains pre-computed aggregate artifacts for the RuTeR evaluation.
The artifacts are intended for result inspection and table cross-checking without
requiring online LLM APIs.

## Layout

- `claude_code/` — supplementary 1,858-case paired token-cost rerun: Excel workbook, compact per-case CSV and 5×/10× post-hoc repair/salvage tables. Separate from the original submitted-paper aggregates; see `experiments/CLAUDE_CODE_EXPERIMENT.md`.
- `rq1/analysis/` — aggregate generation-failure reports for the main 10-crate campaign and the `humantime` cross-model pilot.
- `rq1/rq1_error_code_occurrence_distribution.csv` — compiler error-code occurrence distribution for the main campaign.
- `rq2/aggregates/` — frozen repair-evaluation aggregates for Full RuTeR and replay baselines.
- `rq2/Phase1A_conclusion.md` and `rq2/Phase1A_agent_baseline.md` — aggregate repair summaries.
- `rq3/aggregates/` — coverage-recovery, utility-ratio, and baseline-comparison aggregates.
- `rq3/phase_data/` — Phase 1A+ aggregate CSV/JSON tables used by the downstream analysis.

## Quick aggregate check

Run the repository-level summary script to print the main submitted-paper table
values and the committed aggregate sources used to cross-check them:

```bash
python3 scripts/summarize_paper_tables.py
```

The script reads only local files and does not require network access or LLM
credentials.
