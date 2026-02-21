#!/bin/bash
set -euo pipefail

source ~/.cargo/env 2>/dev/null || true

retry() {
  local attempts="$1"
  shift
  local try=1
  while true; do
    if "$@"; then
      return 0
    fi
    if [ "$try" -ge "$attempts" ]; then
      return 1
    fi
    echo "Command failed (attempt ${try}/${attempts}); retrying in 5s..."
    try=$((try + 1))
    sleep 5
  done
}

echo "[1/6] cargo fmt"
cargo fmt --all -- --check

echo "[2/6] cargo clippy"
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

echo "[3/6] cargo test"
retry 3 cargo test --workspace --all-targets --locked

echo "[4/6] cargo deny advisories"
cargo deny --locked check advisories --show-stats

echo "[5/6] cargo deny dependency policy"
cargo deny --locked -L error check bans sources --hide-inclusion-graph --show-stats

echo "[6/6] cargo deny licenses"
cargo deny --locked check licenses --show-stats

echo "Core production checks passed"
