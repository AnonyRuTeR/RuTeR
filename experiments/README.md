# Experiments Pipeline

This directory contains the Python experiment pipeline All scripts share `common.py` for the Subject model and shared I/O utilities.

## Pipeline Overview

### RQ1: Failure Characteristics
1. **`collect_diagnostics.py`** — Collects rustc compilation diagnostics for subject crates
2. **`build_failure_taxonomy.py`** — Builds error code taxonomy and failure statistics

### RQ2: Repair Effectiveness
3. **`run_repair.py`** — Runs the RuTeR repair pipeline on subject crates
4. **`run_baselines.py`** — Runs baseline repair systems for comparison
5. **`build_repair_effectiveness.py`** — Aggregates and analyzes repair results

## Configuration

Subject manifests are in `configs/`. Copy and customize:
```bash
cp configs/subjects.example.toml configs/subjects.toml
# Edit subjects.toml to point to your crate paths
```

## Usage

```bash
# Set up environment
export RUTER_BIN=/path/to/ruter/target/release/ruter

# Collect diagnostics
python scripts/collect_diagnostics.py --config configs/subjects.toml

# Run repair (requires LLM API key for online mode)
python scripts/run_repair.py --config configs/subjects.toml

# Run baselines
python scripts/run_baselines.py --config configs/subjects.toml

# Build effectiveness report
python scripts/build_repair_effectiveness.py --artifacts-dir artifacts/
```

## Requirements

- Python 3.10+
- Rust toolchain (for building subject crates)
- RuTeR binary (see `../ruter/` for build instructions)
