#!/bin/bash
# Dry-run crates.io package assembly for publishable crates.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Order matches RELEASE.md publish graph (leaf → dependents).
# neo-zkvm-program is built into the prover ELF and is not always published
# as a standalone crate consumers link against; include it for packaging sanity.
CRATES=(
  neo-vm-guest
  neo-zkvm-prover
  neo-zkvm-verifier
  neo-zkvm-cli
  neo-zkvm-program
)

for crate in "${CRATES[@]}"; do
  echo "=== cargo package -p ${crate} --list ==="
  cargo package -p "$crate" --list --allow-dirty --no-verify 2>&1 | tail -20
  echo "=== cargo package -p ${crate} (no verify compile) ==="
  cargo package -p "$crate" --allow-dirty --no-verify
done

echo "Packaging dry-run passed for: ${CRATES[*]}"
