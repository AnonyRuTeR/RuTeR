# RQ1: Failure Characteristics

> What compilation failures do LLM-generated Rust unit tests exhibit, and to what extent are they localized and repairable?

## Data Files

| File | Description |
|------|-------------|
| `Pre_rug_gen_conclusion.md` | Full RQ1 conclusion: compile rates, error Pareto, cross-crate/cross-model analysis |
| `analysis/AGGREGATE_01_longitudinal_gemini.md` | Cross-crate aggregate: 3,183 samples, 10 crates, 51 error codes |
| `analysis/AGGREGATE_02_horizontal_humantime.md` | Cross-model aggregate: 9 models on humantime |
| `analysis/per_crate/*/stats_raw.json` | Per-crate per-model error statistics |
| `rq1_failure_taxonomy.csv` / `.md` | Failure taxonomy data |
| `rq1_error_code_occurrence_distribution.csv` | Error code occurrence distribution |
| `rq1_subject_summary.csv` | Per-subject summary statistics |
| `rq1_error_code_distribution.pdf` / `.svg` | Error code distribution figure |

## Key Results

- 18 RUG runs across 10 crates and 9 LLM models
- Compile success rate varies widely: 1.79% (mio) to 73.91% (semver) for gemini-2.5-flash
- Top 10 error codes account for 93.27% of all failures
- Error codes E0433 (unresolved import) and E0308 (type mismatch) dominate

## Reproduction

```bash
cd ../experiments
python scripts/collect_diagnostics.py --config configs/subjects.toml
python scripts/build_failure_taxonomy.py --artifacts-dir artifacts/
```
