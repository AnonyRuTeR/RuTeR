# Phase 1A Summary: Frozen RUG Replay

## Purpose

Phase 1A evaluates RuTeR on frozen failed RUG-generated Rust unit tests. It measures whether a structured repair pipeline can turn compiler-failing generated tests into compilable tests under a fixed replay protocol.

## Experimental Objects

Input source:

- `rug_runs/*/detailed_log.json`
- Failed RUG `test_generation` attempts whose injected test module matches `tests_rug_*`
- `chrono` is excluded from this phase

Dataset:

| Item | Count |
|---|---:|
| Frozen failed attempts | 1,858 |
| Batch size | 1 attempt / batch |
| Run count | 17 |
| Source function count | 566 |
| Crates | 9 non-chrono crates |

The dataset covers a fixed-model cross-crate slice and a fixed-crate cross-model slice:

- Cross-crate: `gemini-2.5-flash-nothinking` over `humantime`, `itoa`, `log`, `mio`, `rand`, `rust-crc32fast`, `rustc-demangle`, `ryu`, `semver`.
- Cross-model: `humantime` over 9 models.

## Methods

Compared systems:

| System | Description |
|---|---|
| RUG Retry | Counts later successful RUG retries for the same frozen failed attempt source. No repair is applied. |
| Rule-only RuTeR | Applies deterministic repair rules and validation. |
| Full RuTeR | Runs Rule-only first, then uses LLM handoff for unresolved cases. |
| DirectLLM-1shot | One LLM repair request without rules, history or structured retry. |
| DirectAgent-3round | Up to 3 exact-replacement LLM repair rounds. |

Success semantics:

- Strict success: the repaired batch passes the compile/check oracle.
- Normalized success: strict success plus cases accepted by the normalization policy used in the aggregate metrics.
- Function salvage: a target function is counted as salvaged if at least one attempt for that function is repaired.

Primary artifacts:

| Path | Contents |
|---|---|
| `phase_1A/frozen/frozen_attempt_manifest.json` | Canonical frozen sample list. |
| `phase_1A/artifacts/**/batch_result.json` | Full RuTeR per-attempt outcomes. |
| `phase_1A/artifacts/**/batch_llm.json` | LLM handoff prompts/responses for Full runs when used. |
| `phase_1A/artifacts-Rule/**/batch_result.json` | Rule-only outcomes. |
| `phase_1A/aggregates/all_metrics.json` | Main aggregate metrics. |
| `phase_1A/aggregates/all_attempt_results.csv` | Attempt-level analysis table. |
| `phase_1A/aggregates/rug_retry_baseline_metrics.json` | RUG Retry baseline. |

## Results

### Overall Repair Effectiveness

| System | Strict success | Strict rate | Normalized success | Normalized rate |
|---|---:|---:|---:|---:|
| Full RuTeR | 869 / 1,858 | 46.77% | 908 / 1,858 | 48.87% |
| DirectAgent-3round | 565 / 1,858 | 30.41% | 583 / 1,858 | 31.38% |
| DirectLLM-1shot | 217 / 1,858 | 11.68% | 248 / 1,858 | 13.35% |
| RUG Retry | 195 / 1,858 | 10.50% | n/a | n/a |

Full RuTeR also salvages 366 / 566 source functions, a function-level salvage rate of 64.66%.

### Rule and LLM Contribution

Full RuTeR uses a staged protocol:

- Rule-only handles cases that can be fixed deterministically.
- LLM handoff is used for unresolved cases.
- Validation gates reject non-compiling candidates.

The final Full run records:

| Metric | Value |
|---|---:|
| Median Rule stage duration | 18.444 s |
| Median Full stage duration | 43.844 s |
| Median verify rounds | 1 |
| LLM attempt total | 9,151 |

### Error-Code Results

High-volume error codes show different repairability:

| Error code | Attempts | Strict repaired | Strict rate |
|---|---:|---:|---:|
| `E0433` | 1,082 | 564 | 52.13% |
| `E0599` | 441 | 93 | 21.09% |
| `E0432` | 341 | 86 | 25.22% |
| `E0308` | 203 | 88 | 43.35% |
| `E0560` | 128 | 77 | 60.16% |
| `E0063` | 97 | 48 | 49.48% |
| `E0658` | 84 | 9 | 10.71% |
| `E0277` | 71 | 13 | 18.31% |
| `E0061` | 69 | 10 | 14.49% |
| `E0412` | 39 | 8 | 20.51% |

Path and field-structure errors are substantially more repairable than method/trait/unstable-feature failures.

### Robustness Slices

Across `humantime` models, strict repair rates range from 42.11% to 73.27%, with the highest rate on `gpt-4.1-mini`. Across fixed-model crates, rates vary more strongly; `mio` remains difficult while small crates such as `rust-crc32fast`, `itoa`, and `ryu` have higher repair rates.

Difficulty buckets:

| Bucket | Attempts | Repaired | Rate |
|---|---:|---:|---:|
| Single error | 1,166 | 729 | 62.52% |
| Multi supported only | 281 | 69 | 24.56% |
| Multi mixed | 338 | 63 | 18.64% |
| Multi unsupported only | 18 | 2 | 11.11% |
| No error code | 55 | 6 | 10.91% |

## Interpretation

Phase 1A shows that RuTeR substantially improves compile success over native RUG retry and direct LLM baselines on the frozen RUG failure set. The strongest evidence is in structured compiler-diagnostic failures, especially path/import and field-shape errors. Multi-error, unsupported and method/trait-heavy cases remain the main limitation.

