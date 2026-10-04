# RuTeR token measurement

The paired experiment's full protocol and results are described in
[CLAUDE_CODE_EXPERIMENT.md](CLAUDE_CODE_EXPERIMENT.md). This page describes the
standalone RuTeR measurement component.

## Measurement

Each online request records configured/returned model, provider input, output
and total tokens, cache/reasoning details, request outcome and missing-usage
status in `4_llm_usage.json`. Bootstrapping writes this artifact even when no
LLM is invoked, making valid rule-only cases distinguishable from missing logs.
Invalid candidate output remains a method outcome, not an API failure.

The primary mean includes all attempted repairs with complete provider usage,
including failures and zero-token rule repairs. Missing usage or unexpected
models are flagged, not treated as zeros. Input already includes cached tokens;
reasoning is an output detail, not an extra additive cost.

## Standalone rerun

Supply the frozen attempt manifest and matching clean crate snapshots as
described in the paired protocol. Nothing depends on a parent workspace layout.

```bash
cargo build --manifest-path ruter/Cargo.toml --release --locked --bin ruter

# The runner obtains the key only from RUTER_LLM_API_KEY.
python3 experiments/scripts/run_token_cost_experiment.py \
  --manifest /path/to/frozen_attempt_manifest.json \
  --clean-crates-root /path/to/clean-crates \
  --sample-size 1858 --seed 20260924 \
  --model gemini-2.5-flash-nothinking --api-url https://4sapi.org/v1 \
  --out experiments/artifacts/token_cost/ruter --discard-workspaces

python3 experiments/scripts/summarize_token_usage.py \
  --experiment-root experiments/artifacts/token_cost/ruter \
  --out experiments/artifacts/token_cost/ruter/summary
```

The sampler supports smaller proportionally stratified samples by crate and
compiler-error difficulty; it records stratum weights. For a full population
all weights equal one. The released paired workbook uses the full population,
not the earlier pilot or a weighted small-sample estimate.

Runtime summary files contain local paths and are private by default. Use
`export_claude_code_results.py` for an allowlisted anonymous public export.
