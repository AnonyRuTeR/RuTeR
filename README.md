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
RUTER_LLM_API_KEY=your-key ruter --apply fix <crate_path> \
  --enable-llm --llm-mode online \
  --llm-api-url $LLM_API_URL --llm-model $LLM_MODEL \
  --artifacts-dir <output_dir>
```

See `ruter/.env.example` for all configuration options.

## Reproduce Experiments

```bash
cd experiments
cp configs/subjects.example.toml configs/subjects.toml

# RQ1: collect diagnostics
python scripts/collect_diagnostics.py --config configs/subjects.toml
python scripts/build_failure_taxonomy.py --artifacts-dir artifacts/

# RQ2: run repair and baselines
python scripts/run_repair.py --config configs/subjects.toml
python scripts/run_baselines.py --config configs/subjects.toml
python scripts/build_repair_effectiveness.py --artifacts-dir artifacts/

# RQ3: replay and coverage
python ../scripts/replay_runtime.py --manifest frozen/paired_attempt_manifest.json
```

Pre-computed data is in `data/`. See subdirectories for per-RQ README and results.

## License

MIT — see [LICENSE](LICENSE).
