# Phase 1B Summary: Humantime Cross-Source Replay

## Purpose

Phase 1B evaluates whether RuTeR's repair behavior transfers beyond the main RUG Phase 1A source distribution. The phase fixes the crate to `humantime` and compares failed tests from multiple generation sources under the same RuTeR repair protocol.

## Experimental Objects

Crate:

- `humantime`

Sources:

| Source ID | Generator family | Attempts |
|---|---|---:|
| `rug_existing_humantime` | Existing RUG run from Phase 1A | 85 |
| `rugae_base4_humantime_replay` | RUG-AE GPT-4 base replay artifact | 11 |
| `rugae_hybrid4_humantime_replay` | RUG-AE GPT-4 hybrid replay artifact | 11 |
| `direct_llm_1shot` | Local DirectLLM-1shot source-generation records | 46 |
| `rustyunit_humantime_replay` | RustyUnit replay artifact | 21 |

Dataset scale:

| Metric | Value |
|---|---:|
| Raw attempts | 174 |
| Eligible attempts | 174 |
| Batches | 174 |
| Source groups | 5 |
| Unmatched attempts | 0 |

RUG-AE source files are not stored as a local external checkout in `eval/`. Exact file paths and hashes used for import are recorded in `phase_1B/frozen/source_freeze_manifest.json`.

## Method

Phase 1B imports heterogeneous source artifacts into one frozen attempt schema:

- RUG existing `humantime` failures come from Phase 1A RUG records.
- RUG-AE base/hybrid artifacts are normalized from upstream replay files.
- DirectLLM-1shot failures are imported from local source-generation records.
- RustyUnit replay artifacts are imported through the same schema.

Each frozen attempt is then repaired with the same RuTeR protocol used in Phase 1A:

- 1 attempt / batch.
- Rule-only stage followed by Full RuTeR stage.
- Same compile/check oracle.
- Same strict and normalized success semantics.

Primary artifacts:

| Path | Contents |
|---|---|
| `phase_1B/frozen/source_freeze_manifest.json` | Source files and hashes used for import. |
| `phase_1B/frozen/frozen_attempt_manifest.json` | Canonical Phase 1B sample list. |
| `phase_1B/frozen/target_name_mapping.json` | Target-name normalization map. |
| `phase_1B/directllm_1shot/attempts_failed.json` | DirectLLM input failures. |
| `phase_1B/artifacts/**/batch_result.json` | Per-attempt repair outcomes. |
| `phase_1B/aggregates/all_metrics.json` | Main aggregate metrics. |
| `phase_1B/aggregates/all_source_metrics.csv` | Source-level comparison table. |

## Results

Overall repair outcome:

| Metric | Value |
|---|---:|
| Attempt total | 174 |
| Strict success | 79 |
| Strict success rate | 45.40% |
| Normalized success | 84 |
| Normalized success rate | 48.28% |
| Function total | 62 |
| Function salvaged | 32 |
| Function salvage rate | 51.61% |

Source-level results:

| Source | Attempts | Strict success rate | Normalized success rate |
|---|---:|---:|---:|
| `rug_existing_humantime` | 85 | 63.53% | 63.53% |
| `rugae_base4_humantime_replay` | 11 | 36.36% | 54.55% |
| `rugae_hybrid4_humantime_replay` | 11 | 36.36% | 54.55% |
| `direct_llm_1shot` | 46 | 30.43% | 30.43% |
| `rustyunit_humantime_replay` | 21 | 14.29% | 19.05% |

Difficulty buckets:

| Bucket | Attempts | Repaired | Repair rate |
|---|---:|---:|---:|
| Single error | 140 | 79 | 56.43% |
| Multi mixed | 23 | 4 | 17.39% |
| Multi supported only | 9 | 0 | 0.00% |
| Multi unsupported only | 2 | 0 | 0.00% |

Cost summary:

| Metric | Value |
|---|---:|
| Median Rule stage duration | 6.283 s |
| Median Full stage duration | 21.371 s |
| Median verify rounds | 1 |
| LLM attempt total | 2,168 |

## Interpretation

Phase 1B supports a limited transfer claim: RuTeR remains useful outside the main Phase 1A RUG distribution, especially for single-error failures and RUG-AE base/hybrid inputs. The result is weaker on RustyUnit replay inputs and multi-error cases. Source shift therefore reduces reliability but does not eliminate repair utility.

## Data Boundaries

- The crate is fixed to `humantime`; this phase does not establish cross-crate transfer for non-RUG sources.
- Source sizes differ substantially, so source-level rates should be read with counts.
- RUG-AE base and hybrid artifacts are identical in several imported file hashes; the two source IDs are retained because they correspond to distinct upstream tracks.
- The result evaluates compile/check repair, not semantic correctness of generated assertions.

