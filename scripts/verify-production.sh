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

echo "[1/10] cargo fmt"
cargo fmt --all -- --check

echo "[2/10] cargo clippy"
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

echo "[3/10] cargo test"
retry 3 cargo test --workspace --all-targets --locked -q

echo "[4/10] cargo build --release"
retry 3 cargo build --release --workspace --locked

echo "[5/10] cargo doc"
retry 3 cargo doc --workspace --locked --no-deps

echo "[6/10] cargo test --doc"
retry 3 cargo test --doc --workspace --locked

echo "[7/10] run examples"
retry 3 cargo run --locked --bin basic
retry 3 cargo run --locked --bin storage_example
retry 3 cargo run --locked --bin native_contracts
retry 3 cargo run --locked --bin proof_generation
retry 3 cargo run --locked --bin batch_verification
retry 3 cargo run --locked --bin private_inputs
retry 3 cargo run --locked --bin tamper_resistance

echo "[8/10] cargo deny advisories"
cargo deny --locked check advisories --show-stats

echo "[9/10] cargo deny dependency policy"
cargo deny --locked -L error check bans sources --hide-inclusion-graph --show-stats

echo "[10/10] cargo deny licenses"
cargo deny --locked check licenses --show-stats

echo "Production readiness checks passed"
