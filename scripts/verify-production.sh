#!/bin/bash
set -euo pipefail

source ~/.cargo/env 2>/dev/null || true

echo "[1/4] cargo fmt"
cargo fmt --all -- --check

echo "[2/4] cargo clippy"
cargo clippy --all --all-targets -- -D warnings

echo "[3/4] cargo test"
cargo test --all -q

echo "[4/4] cargo deny advisories"
cargo deny check advisories --show-stats

echo "Production readiness checks passed"
