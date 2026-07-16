#!/bin/bash
set -euo pipefail

source ~/.cargo/env 2>/dev/null || true

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

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

cargo_deny_advisories() {
  if cargo deny --all-features --locked check advisories --show-stats; then
    return 0
  fi

  echo "Online advisory check failed; retrying with the existing local advisory database."
  cargo deny --all-features --locked --offline check advisories --show-stats
}

run_examples() {
  local bins=(
    basic
    proof_generation
    batch_verification
    private_inputs
    tamper_resistance
    zk_preimage
    zk_scaling
    zk_dao_voting
    zk_dex_rollup
    zk_factors
    zk_range
    zk_membership
    zk_selective_disclosure
    zk_merkle_inclusion
    zk_attestation_settlement
  )
  for bin in "${bins[@]}"; do
    retry 3 cargo run --locked --bin "$bin"
  done
}

echo "[1/12] cargo fmt"
cargo fmt --all -- --check

echo "[2/12] cargo clippy"
cargo clippy --workspace --all-targets --locked -- -D warnings

echo "[3/12] SP1 feature clippy"
SP1_FORCE_DUMMY=true cargo clippy -p neo-zkvm-prover -p neo-zkvm-verifier -p neo-zkvm-cli --features sp1 --locked -- -D warnings

echo "[4/12] cargo test"
retry 3 cargo test --workspace --all-targets --locked -q

echo "[5/12] cargo build --release"
retry 3 cargo build --release --workspace --locked

echo "[6/12] cargo doc"
retry 3 cargo doc --workspace --locked --no-deps

echo "[7/12] cargo test --doc"
retry 3 cargo test --doc --workspace --locked

echo "[8/12] run examples"
run_examples

echo "[9/12] release mock proof smoke"
retry 3 cargo run --release -p neo-zkvm-cli -- prove 12139E40 -m mock

# Optional: only when SP1 toolchain is present and not forced dummy.
if [ "${SP1_FORCE_DUMMY:-}" != "true" ] && command -v cargo-prove >/dev/null 2>&1; then
  echo "[9b/12] release SP1 proof smoke"
  retry 3 cargo run --release -p neo-zkvm-cli --features sp1 -- prove 12139E40 -m sp1
else
  echo "[9b/12] skip SP1 proof smoke (no cargo-prove / SP1_FORCE_DUMMY set)"
fi

echo "[10/12] cargo deny advisories"
cargo_deny_advisories

echo "[11/12] cargo deny dependency policy"
cargo deny --all-features --locked -L error check bans sources --hide-inclusion-graph --show-stats

echo "[12/12] cargo deny licenses"
cargo deny --all-features --locked check licenses --show-stats

echo "Production readiness checks passed"
