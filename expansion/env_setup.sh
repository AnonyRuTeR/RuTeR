#!/usr/bin/env bash
# RuTeR Expansion Experiment — Environment Setup Script
# Source this file: source env_setup.sh
# Generated: 2026-06-25

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export RUTER_EXPANSION_ROOT="$SCRIPT_DIR"

# --- safefinder/cargorunner path ---
SAFEFINDER_TARGET="$SCRIPT_DIR/tools/rug-ae/rug_ae1/source/safefinder/target/release"
if [ -d "$SAFEFINDER_TARGET" ]; then
    export PATH="$SAFEFINDER_TARGET:$PATH"
fi

# --- Rust toolchain ---
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-nightly-2022-12-10}"

# --- DYLD (macOS) / LD (Linux) library path for rustc driver ---
NIGHTLY_LIB="$HOME/.rustup/toolchains/${RUSTUP_TOOLCHAIN}-aarch64-apple-darwin/lib"
if [ -d "$NIGHTLY_LIB" ]; then
    export DYLD_LIBRARY_PATH="$NIGHTLY_LIB${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
fi
# Also check x86_64 (Linux or macOS Intel)
NIGHTLY_LIB_X86="$HOME/.rustup/toolchains/${RUSTUP_TOOLCHAIN}-x86_64-unknown-linux-gnu/lib"
if [ -d "$NIGHTLY_LIB_X86" ]; then
    export LD_LIBRARY_PATH="$NIGHTLY_LIB_X86${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi

# --- Increase stack limit (macOS workaround) ---
# macOS has a hard limit of ~64MB; Linux can use unlimited
if [[ "$(uname -s)" == "Darwin" ]]; then
    ulimit -s 65520 2>/dev/null || true
else
    ulimit -s unlimited 2>/dev/null || true
fi

# --- Rust stack for spawned threads ---
export RUST_MIN_STACK="${RUST_MIN_STACK:-33554432}"

# --- Python virtual environment ---
if [ -f "$SCRIPT_DIR/venv/bin/activate" ]; then
    source "$SCRIPT_DIR/venv/bin/activate"
fi

# --- API configuration (set these BEFORE sourcing) ---
# export LLM_API_KEY='your-key-here'
export LLM_BASE_URL="${LLM_BASE_URL:-https://api.openai.com/v1}"
export LLM_MODEL="${LLM_MODEL:-gemini-2.5-flash-nothinking}"

echo "[env_setup] RUTER_EXPANSION_ROOT=$RUTER_EXPANSION_ROOT"
echo "[env_setup] RUSTUP_TOOLCHAIN=$RUSTUP_TOOLCHAIN"
echo "[env_setup] LLM_MODEL=$LLM_MODEL"
echo "[env_setup] LLM_BASE_URL=$LLM_BASE_URL"
echo "[env_setup] safefinder: $(command -v cargorunner 2>/dev/null || echo 'NOT FOUND')"
echo "[env_setup] cargo-ws: $(cargo ws --version 2>/dev/null || echo 'NOT FOUND')"
echo "[env_setup] python openai: $(python3 -c 'import openai; print(openai.__version__)' 2>/dev/null || echo 'NOT FOUND')"
echo "[env_setup] Ready. Set LLM_API_KEY before running RUG."
