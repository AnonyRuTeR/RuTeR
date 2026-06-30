# RuTeR Expansion Experiment — Environment Setup Summary

Date: 2026-06-25  
Machine: macOS aarch64 (Apple Silicon)

## What was configured

| Component | Status | Details |
|---|---|---|
| Rust nightly-2022-12-10 toolchain | ✓ Installed | rustc 1.68.0-nightly, with rustc-dev, rust-src, llvm-tools-preview |
| safefinder/cargorunner (safefinder plugin) | ✓ Built for macOS aarch64 | `target/release/{cargorunner,safefinder}` — Mach-O arm64 |
| Hardcoded target fix | ✓ Patched | Changed `--target x86_64-unknown-linux-gnu` → `version_info().host` in cargorunner.rs |
| cargo-workspaces | ✓ Installed | v0.4.2 (`cargo ws --version`) |
| Python venv | ✓ Created | `venv/` with openai 2.44.0, tiktoken 0.13.0 |
| RUG main.py | ✓ Patched | `rug_runner/main_patched.py` — reads LLM_API_KEY, LLM_BASE_URL, LLM_MODEL from env |
| Subject crates | ✓ Cloned | 10 crates under `subjects/` |

## Critical blocker: macOS stack limit

**safefinder works on trivial crates but fails on real-world crates due to macOS stack size limits.**

- macOS hard stack limit: ~64 MB
- Linux default: unlimited / much higher
- Simple crates (cargo new template): ✓ works, produces preprocess.json
- Real crates: Stack overflow in rustc analysis phase

Crate compatibility test results (nightly-2022-12-10, ulimit -s 65520):

| Crate | Result | Issue |
|---|---|---|
| anyhow | No output | Proc-macro crate, nothing to analyze |
| bytes | Stack overflow | Complex crate, deep type recursion |
| indexmap | No output | Dependency resolution issue |
| once_cell | Stack overflow | Rustc analysis recursion |
| parking_lot | Fails | Requires rustc ≥1.71, nightly is 1.68 |
| regex | No output | Dependency/build issue |
| smallvec | Fails | Requires rustc ≥1.83 |
| thiserror | No output | Proc-macro crate |
| url | No output | Dependency resolution issue |
| uuid | No output | rand dependency feature conflict |

## Recommended path forward

**Run RUG generation on a Linux machine** (or Docker container). The safefinder tool was designed for Linux and has fundamental macOS incompatibilities:

1. **Docker** (if installable): Use a Ubuntu 22.04 image with pre-built safefinder
2. **Remote Linux server**: SSH in, set up the same environment
3. **Update safefinder for newer nightly**: Significant engineering effort (rustc internal APIs change every nightly)

## Quick-start commands (on Linux)

```bash
# Build safefinder
cd tools/rug-ae/rug_ae1/source/safefinder
cargo +nightly-2022-12-10 build --release
export PATH="$PWD/target/release:$PATH"

# Set up Python
cd /path/to/RustTestLab_expansion_2026_06_25
source venv/bin/activate  # or create a new one

# API config
export LLM_API_KEY='<your-key>'
export LLM_BASE_URL='https://api.openai.com/v1'
export LLM_MODEL='gemini-2.5-flash-nothinking'

# Run RUG on a crate
cd subjects/bytes
cargo clean
CHAT_UNIT=1 cargorunner rudra        # Step 1: preprocess.json
UNIT_GEN=s1 cargorunner rudra        # Step 2: .out.txt
# Then run rug_runner/main_patched.py
```

## Files created/modified

- `rug_runner/main_patched.py` — Env-variable-based RUG runner
- `venv/` — Python virtual environment with openai and tiktoken
- `tools/rug-ae/.../safefinder/src/bin/cargorunner.rs` — Fixed hardcoded target
- `tools/rug-ae/.../safefinder/target/release/{cargorunner,safefinder}` — macOS ARM64 binaries
- `/tmp/patch_stack.py` — Mach-O stack size patcher (failed due to codesigning)
