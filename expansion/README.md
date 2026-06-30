# Expansion Experiment

This directory contains the setup for the RustTestLab expansion experiment, which adds 10 additional crates to the RuTeR evaluation beyond the original 9 crates in the main paper.

## Contents

| File | Description |
|------|-------------|
| `env_setup.sh` | Environment configuration script — source this before running experiments |
| `manifest.md` | Crate manifest for the expansion set |
| `ENV_SETUP_SUMMARY.md` | Full environment setup status and configuration |
| `rug_runner/main_patched.py` | Patched RUG runner with env var support for LLM API key |

## New Crates

The expansion adds: `bytes`, `once_cell`, `smallvec`, `regex`, `url`, `anyhow`, and others (see `manifest.md`).

## Usage

```bash
# Source the environment setup
source env_setup.sh

# Run the patched RUG runner
python rug_runner/main_patched.py --crate <crate_name>
```

## Requirements

- Linux (safefinder/cargorunner has macOS stack size limitations)
- Rust toolchain
- Python 3.10+
- LLM API key set via `LLM_API_KEY` environment variable
