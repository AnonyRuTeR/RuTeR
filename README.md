# RuTeR

An automated repair framework for compilation errors in LLM-generated Rust unit tests.

## Build

```bash
cd ruter
cargo build --release
```

Requires Rust toolchain (stable).

## Usage

```bash
# Dry run
ruter fix <crate_path> --artifacts-dir <output_dir>

# Apply fixes
ruter --apply fix <crate_path> --artifacts-dir <output_dir>

# With LLM fallback
RUTER_LLM_API_KEY=<your-api-key> ruter --apply fix <crate_path> \
  --enable-llm --llm-mode online \
  --llm-api-url $LLM_API_URL --llm-model $LLM_MODEL \
  --artifacts-dir <output_dir>
```

See `ruter/.env.example` for all configuration options.

## Reproduce Experiments

```bash
cd experiments
cp configs/subjects.example.toml configs/subjects.toml

# RQ1: collect diagnostics and build taxonomy
python scripts/collect_diagnostics.py --manifest configs/subjects.toml --out artifacts/diagnostics
python scripts/build_failure_taxonomy.py --input artifacts/diagnostics --out results/rq1

# RQ2: run repair, baselines, and build effectiveness
python scripts/run_repair.py --manifest configs/subjects.toml --out artifacts/repair
python scripts/run_baselines.py --manifest configs/subjects.toml --out artifacts/baselines
python scripts/build_repair_effectiveness.py --repair artifacts/repair --baselines artifacts/baselines --out results/rq2

# RQ3: coverage replay (requires EVAL_DIR and eval crates/ data)
EVAL_DIR=../data python ../scripts/replay_runtime.py
```

Pre-computed aggregate data is in `data/`. Run `python3 scripts/summarize_paper_tables.py` to print the main paper-facing table checks.

## Supplementary Claude Code comparison

The full paired rerun covers 1,858 failed attempts and 566 function units, using
the same Gemini model/API gateway for RuTeR and the Claude Code agent harness.
See [the experiment protocol](experiments/CLAUDE_CODE_EXPERIMENT.md) for the
scripts, prompt, token accounting, success guards and rerun prerequisites.

[The Excel workbook](data/claude_code/claude_code_results.xlsx) and compact
per-case CSV support offline recomputation, including post-hoc 5×/10× token
cutoffs and function salvage rates. Raw logs/workspaces are not distributed.
The supplementary rerun is separate from the original submitted-paper results.

```bash
python3 experiments/scripts/export_claude_code_results.py \
  --cases-csv data/claude_code/paired_cases.csv
```

## License

MIT — see [LICENSE](LICENSE).
