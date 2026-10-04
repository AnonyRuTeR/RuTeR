# Supplementary paired Claude Code results

This directory is a compact anonymous export of the full 1,858-case,
566-function paired rerun. It is separate from the submitted-paper RQ tables.

- `claude_code_results.xlsx`: verified Summary, per-case input/output/total
  tokens, pseudonymous function/run IDs, and editable budget calculations with
  cached formulas; no raw logs are required.
- `paired_cases.csv`: the 1,858 allowlisted observations underlying the Excel
  workbook. Both ordinary failures and valid rule-only zero-token cases are included.
- `summary.csv`: unrestricted RuTeR / Claude and post-hoc 5× / 10× results.

Run from the repository root without network or paid APIs:

```bash
python3 experiments/scripts/export_claude_code_results.py \
  --cases-csv data/claude_code/paired_cases.csv
```

Function salvage groups the generation-run/function pair, not just crate/function.
The IDs are stable pseudonyms; there is no private identity mapping in this
release. Token caps are per attempt and use the exact unrounded population mean.
A post-hoc cutoff is not equivalent to stopping an agent during a repair.

The workbook's Summary is a verified snapshot; Calculations, Cases and Functions
are formula-based and recalculate when multipliers are changed. Full-run token
means/ratios are not reduced by the post-hoc cutoff.

See [the experiment protocol](../../experiments/CLAUDE_CODE_EXPERIMENT.md) for
the model, versions, prompts, guards, retry policy, limitations and paid-rerun
input requirements. Raw repair inputs, workspaces, transcripts, account settings
and infrastructure retry logs are intentionally not included.
