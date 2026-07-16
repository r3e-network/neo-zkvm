#!/bin/bash
# Publish workspace crates to crates.io in dependency order.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---plan}"

# Publish order: dependencies first.
# verifier depends on prover only under the optional `sp1` feature; both must
# still be published so consumers can enable SP1 verification.
CRATES=(
  neo-vm-guest
  neo-zkvm-prover
  neo-zkvm-verifier
  neo-zkvm-attestation
  neo-zkvm-cli
)

plan() {
  echo "Publish order for crates.io:"
  local i=1
  for crate in "${CRATES[@]}"; do
    echo "  ${i}. cargo publish -p ${crate} --locked"
    i=$((i + 1))
  done
  echo
  echo "Skipped (publish = false): neo-zkvm-examples"
  echo "Optional: neo-zkvm-program (guest ELF source; publish only if consumers need it)"
}

case "$MODE" in
  --plan | plan)
    plan
    ;;
  --dry-run | dry-run)
    for crate in "${CRATES[@]}"; do
      echo "=== dry-run publish ${crate} ==="
      cargo publish -p "$crate" --locked --dry-run
    done
    ;;
  --execute | execute | publish)
    for crate in "${CRATES[@]}"; do
      echo "=== publishing ${crate} ==="
      cargo publish -p "$crate" --locked
    done
    ;;
  *)
    echo "Usage: $0 [--plan|--dry-run|--execute]" >&2
    exit 2
    ;;
esac
