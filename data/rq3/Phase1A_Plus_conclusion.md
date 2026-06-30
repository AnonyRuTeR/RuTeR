# Phase 1A+ Summary: Coverage Recovery

## Purpose

Phase 1A+ evaluates whether repaired tests from Phase 1A provide downstream coverage value. Phase 1A measures compilability; Phase 1A+ measures whether repaired tests can be materialized, executed, and contribute additional source coverage under a canonical replay setup.

This phase uses an LLM-only replay scope. It does not include RUG fuzzing or coverage-guided post-processing.

## Experimental Objects

Input repair sources:

| Source | Input artifacts |
|---|---|
| RuTeR | `phase_1A/artifacts/**/batch_result.json`, `phase_1A/artifacts/**/batch_llm.json`, `phase_1A/artifacts-Rule/**/batch_result.json` |
| DirectLLM-1shot | `phase_1A_oneshot/artifacts/**/batch_result.json`, `phase_1A_oneshot/artifacts/**/batch_llm.json` |
| DirectAgent-3round | `phase_1A_agent/artifacts/**/batch_result.json`, `phase_1A_agent/artifacts/**/direct_agent_3round/agent_rounds.json` |

Replay crates:

`humantime`, `itoa`, `log`, `mio`, `rand`, `rust-crc32fast`, `rustc-demangle`, `ryu`, `semver`.

Coverage metrics:

- Lines
- Functions
- Regions

Branch coverage is not used because the retained raw export does not provide a stable branch-total field for this phase.

## Method

The replay creates canonical crate workspaces from clean crate snapshots, materializes repaired tests, and compares coverage against baseline and original-success scenarios.

Main scenarios:

| Scenario | Meaning |
|---|---|
| Baseline | Clean crate without generated tests. |
| Successful-only | Baseline plus RUG tests that originally compiled. |
| Repaired-only | Baseline plus repaired tests recovered from failed attempts. |
| Successful-plus-repaired | Combined successful original tests and repaired tests. |

Replay eligibility is stricter than Phase 1A success. A repaired attempt must have recoverable patched code and must be materializable in a clean canonical crate.

Primary artifacts:

| Path | Contents |
|---|---|
| `phase_1A_plus/frozen/paired_attempt_manifest.json` | RuTeR paired repair/replay manifest. |
| `phase_1A_plus/frozen/paired_attempt_manifest_direct_llm.json` | DirectLLM paired manifest. |
| `phase_1A_plus/frozen/paired_attempt_manifest_direct_agent.json` | DirectAgent paired manifest. |
| `phase_1A_plus/aggregates/coverage_recovery_metrics.json` | RuTeR coverage metrics. |
| `phase_1A_plus/aggregates/coverage_recovery_metrics_direct_llm.json` | DirectLLM coverage metrics. |
| `phase_1A_plus/aggregates/coverage_recovery_metrics_direct_agent.json` | DirectAgent coverage metrics. |
| `phase_1A_plus/aggregates/direct_baseline_coverage_comparison.json` | Cross-baseline coverage comparison. |
| `phase_1A_plus/aggregates/phase1a_plus_summary.json` | Consolidated Phase 1A+ summary. |

## RuTeR Coverage Results

RuTeR Phase 1A input scale:

| Metric | Value |
|---|---:|
| Phase 1A failed attempts | 1,858 |
| Replay-eligible repaired attempts | 738 |
| Compile-executable repaired tests in replay | 42 |
| Coverage-contributing repaired tests | 42 |
| Runtime-quarantined repaired tests | 7 |

Coverage added by repaired tests:

| Coverage metric | Added by repaired tests |
|---|---:|
| Lines | 679 |
| Functions | 77 |
| Regions | 1,174 |

Distribution:

| Metric | Value |
|---|---:|
| Runs with positive repaired coverage | 9 / 17 |
| Crates with positive repaired coverage | 6 / 9 |
| Coverage-effective rate among replayed run results | 52.94% |

Positive repaired coverage appears in:

`humantime`, `itoa`, `log`, `mio`, `rand`, `rustc-demangle`.

## Relationship to Original Successful RUG Tests

Original successful RUG-generated tests contribute:

| Coverage metric | Added by original successful tests |
|---|---:|
| Lines | 4,410 |
| Functions | 779 |
| Regions | 9,166 |

Repaired tests add smaller absolute coverage than original successful tests, but they recover value from attempts that otherwise failed compilation and would have contributed no executable test asset.

## Interpretation

Phase 1A+ shows that a subset of RuTeR's compile-repaired tests can survive clean replay and add measurable coverage. The coverage result is not a replacement for Phase 1A's repair-rate result: it is a downstream utility check on the repaired-test assets. The main finding is that RuTeR converts part of the originally failed RUG generation output into executable, coverage-contributing tests.

## Data Boundaries

- The phase reports LLM-only replay coverage, not fuzz-enhanced RUG full-pipeline coverage.
- Coverage is reported for lines, functions and regions only.
- Replay eligibility is lower than compile repair success because patched code must be recoverable and materializable in a clean crate.
- Coverage magnitude is sensitive to crate size, existing generated tests and the canonical replay setup.
