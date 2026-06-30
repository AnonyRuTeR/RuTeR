# RQ2: Repair Effectiveness

> How effectively does RuTeR repair uncompilable LLM-generated Rust unit tests compared with baselines?

## Data Files

| File | Description |
|------|-------------|
| `Phase1A_conclusion.md` | Full RQ2 conclusion: 5-system comparison, per-error, per-crate, per-model |
| `Phase1A_agent_baseline.md` | DirectAgent baseline safety and effectiveness analysis |
| `CaseStudy.md` | 3 detailed case studies (E0433 rule success, E0308 LLM success, E0599 failure) |
| `aggregates/all_metrics.json` | Aggregate repair metrics across 5 systems |
| `aggregates/all_attempt_results.csv` | Per-attempt results: 1,858 attempts × 5 systems |
| `aggregates/rug_retry_baseline_metrics.json` | RUG retry baseline metrics |
| `rq2_repair_effectiveness.csv` / `.md` | Summary repair effectiveness table |

## Systems Compared

| System | Description |
|--------|-------------|
| RuTeR (Full) | Rule patchers + LLM, function-scoped, verification-gated |
| RuTeR (Rule-only) | Rule patchers only, no LLM |
| DirectLLM-1shot | Single LLM prompt with diagnostics + source context |
| DirectAgent-3round | Multi-round LLM agent with tool access |
| RUG Retry | Re-running the test generator (stochastic baseline) |

## Key Results

- RuTeR Full: 46.43% strict repair rate across 1,858 attempts
- RuTeR Rule-only: 7.20% (deterministic, instant)
- DirectLLM-1shot: 14.20%
- DirectAgent-3round: 25.59%
- RUG Retry: 10.23%
- Function salvage rate: 64.66% (566 functions)
- RuTeR: 0 business-code escapes (vs DirectAgent: 167 escapes)

## Reproduction

```bash
cd ../experiments
python scripts/run_repair.py --config configs/subjects.toml
python scripts/run_baselines.py --config configs/subjects.toml
python scripts/build_repair_effectiveness.py --artifacts-dir artifacts/
```
