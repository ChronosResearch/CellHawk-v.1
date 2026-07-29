#!/bin/bash
set -e

echo "=========================================="
echo "    CELLHAWK FIELD TRIAL ORCHESTRATOR     "
echo "=========================================="

export RUST_LOG=info

# Optionally activate python environment for analysis
if [ -d "../venv" ]; then
    source ../venv/bin/activate
fi

echo "[*] Building cellhawk-field..."
cargo build --release -p cellhawk-field

echo "[*] Starting Orchestrator..."
./target/release/cellhawk-field "$@"

echo "[*] Run complete."
