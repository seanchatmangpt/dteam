#!/usr/bin/env bash
set -e

echo "=========================================================="
echo "Starting DDS Ralph Overnight Orchestration Loop (DTEAM Arena Focus)"
echo "Target Models: gemini-3.1-pro-preview"
echo "Workspace: /Users/sac/unibit"
echo "Concurrency: 5"
echo "=========================================================="

# Ensure we are in the unibit directory
cd /Users/sac/unibit

echo "Running pre-flight structural checks in unibit..."
cargo check
cargo test --lib

echo "Pre-flight checks passed. Unleashing Ralph on the DTEAM Arena backlog..."

# Execute Ralph from dteam but in the unibit workspace
# We use the ralph binary built in dteam
RUST_LOG=info cargo run --release --manifest-path /Users/sac/dteam/Cargo.toml --bin ralph -- \
    --model "gemini-3.1-pro-preview" \
    --concurrency 5 \
    --offset 0

echo "=========================================================="
echo "Ralph execution complete. Please check the unibit dev branch for merged artifacts."
echo "=========================================================="
