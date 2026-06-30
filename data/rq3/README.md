# RQ3: Downstream Testing Utility & Generalization

> Do repaired tests provide real downstream testing utility beyond merely compiling?
> How well does RuTeR generalize across crates, LLMs, and generation pipelines?

## Data Files

### RQ3 — Coverage & Utility

| File | Description |
|------|-------------|
| `Phase1A_Plus_conclusion.md` | Full RQ3 conclusion: coverage recovery, utility ratio, safety comparison |
| `aggregates/coverage_recovery_metrics.json` | Lines/Functions/Regions coverage recovery by system |
| `aggregates/coverage_recovery_metrics_direct_llm.json` | DirectLLM coverage metrics |
| `aggregates/coverage_recovery_metrics_direct_agent.json` | DirectAgent coverage metrics |
| `aggregates/direct_baseline_coverage_comparison.json` | Cross-baseline comparison |
| `aggregates/quality_guard_metrics.json` | Oracle strength: panic/smoke/strong_assertion (227/824/807) |
| `aggregates/cost_utility_metrics.json` | Per-run cost vs utility |
| `aggregates/coverage_recovery_*_table*.md` | Per-crate/per-run coverage tables |
| `aggregates/coverage_shadow_observation_*.md` | Coverage shadow analysis |
| `aggregates/utility_recovery_ratio_*` | Utility recovery ratio (CSV/JSON/table) |

### RQ4 — Generalization

| File | Description |
|------|-------------|
| `Phase1B_conclusion.md` | Cross-source generalization: 5 sources × humantime |
| `phase_data/all_source_metrics.csv` | Source-level repair rates |

## Key Results — RQ3

| System | Replay-eligible | Coverage-contributing | Lines | Functions | Regions |
|--------|----------------:|----------------------:|------:|----------:|--------:|
| RuTeR | 738 | 42 | 679 | 77 | 1,174 |
| DirectLLM-1shot | 236 | 206 | 1,215 | 229 | 2,247 |
| DirectAgent-3round | 407 | 375 | 2,307 | 452 | 4,434 |

> DirectAgent's higher raw coverage must be interpreted alongside its 167 business-code escapes.

## Key Results — RQ4 (Generalization)

| Source | Attempts | Strict Rate |
|--------|---------:|------------:|
| RUG (humantime) | 85 | 63.53% |
| RUG-AE base4 | 11 | 36.36% |
| RUG-AE hybrid4 | 11 | 36.36% |
| DirectLLM-1shot | 46 | 30.43% |
| RustyUnit | 21 | 14.29% |

## Reproduction

```bash
cd ../experiments
# Coverage replay
python scripts/replay_runtime.py --manifest frozen/paired_attempt_manifest.json
# Then analyze coverage with tarpaulin or equivalent
```
