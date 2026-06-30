# Phase 1A Baseline Summary: DirectAgent-3round

## Purpose

DirectAgent-3round evaluates a generic agent-style LLM repair loop on the same Phase 1A frozen dataset. It measures whether multiple LLM repair rounds can match RuTeR without RuTeR's rule-first design and safety boundaries.

## Experimental Objects

Dataset:

- Same 1,858 frozen failed attempts as Phase 1A.
- Same batch granularity: 1 attempt / batch.
- Same compile/check oracle as Phase 1A.

Primary artifacts:

| Path | Contents |
|---|---|
| `phase_1A_agent/frozen/frozen_attempt_manifest.json` | Copied frozen Phase 1A sample list. |
| `phase_1A_agent/artifacts/**/batch_result.json` | Final per-attempt outcome. |
| `phase_1A_agent/artifacts/**/direct_agent_3round/agent_rounds.json` | Round-level prompts, responses, actions, validation feedback and edit-scope records. |
| `phase_1A_agent/aggregates/all_agent_metrics.json` | Main aggregate metrics. |
| `phase_1A_agent/aggregates/all_agent_attempt_results.csv` | Attempt-level table. |

## Method

The baseline allows up to 3 LLM repair rounds. Each round receives compiler feedback and returns exact text replacement actions. The evaluator applies actions, reruns validation and records whether edits stay inside the target test function.

The baseline does not use:

- RuTeR's deterministic repair rules.
- Function-level merge gate.
- Non-target regression gate.
- External-file safety policy.
- Structured handoff from rule analysis.

## Results

| Metric | Value |
|---|---:|
| Attempt total | 1,858 |
| Strict success | 565 |
| Strict success rate | 30.41% |
| Normalized success | 583 |
| Normalized success rate | 31.38% |
| Function total | 566 |
| Function salvaged | 281 |
| Function salvage rate | 49.65% |

Compared with Full RuTeR:

| Comparison | Strict delta | Normalized delta |
|---|---:|---:|
| DirectAgent-3round vs Full RuTeR | -16.36 pp | -17.49 pp |

On the Rule-unresolved subset:

| Metric | DirectAgent-3round | Full RuTeR |
|---|---:|---:|
| Strict success rate | 19.25% | 31.30% |
| Normalized success rate | 20.50% | 34.07% |

## Safety Results

The agent baseline frequently edits beyond the target test function:

| Edit-scope metric | Count | Rate |
|---|---:|---:|
| Function-scope escape | 1,006 | 54.14% |
| Business-code escape | 167 | 8.99% |
| External-file escape | 186 | 10.01% |
| Strict successes involving business-code escape | 16 | n/a |
| Business-code escape followed by unresolved result | 151 | n/a |

These records are in `agent_rounds.json` and aggregate safety fields.

## Interpretation

DirectAgent-3round is stronger than DirectLLM-1shot but still below Full RuTeR in repair rate. Its higher edit-scope escape rate shows a clear safety tradeoff: generic agent repair can recover more tests than one-shot prompting, but it more often modifies code outside the intended generated-test boundary.

