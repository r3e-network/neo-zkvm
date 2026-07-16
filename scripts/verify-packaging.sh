#!/bin/bash
# Dry-run crates.io package assembly for publishable crates.
#
# Note: packaging requires every non-path dependency to resolve on crates.io.
# The workspace currently pins `neo-vm-rs` via git+rev for reproducible builds.
# Until `neo-vm-rs` is published to crates.io at the declared version, full
# `cargo package` will fail — this script reports that clearly.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Order matches RELEASE.md publish graph (leaf → dependents).
CRATES=(
  neo-vm-guest
  neo-zkvm-prover
  neo-zkvm-verifier
  neo-zkvm-cli
  neo-zkvm-program
)

fail=0

# Preflight: git-only deps without a crates.io counterpart break packaging.
if ! cargo search neo-vm-rs --limit 1 2>/dev/null | grep -q '^neo-vm-rs ='; then
  echo "WARNING: neo-vm-rs is not published on crates.io (or crates.io is unreachable)."
  echo "         Workspace builds use git+rev and are fine."
  echo "         crates.io package dry-run will fail until neo-vm-rs is published."
  echo "         Continuing with package attempts for diagnostics..."
  echo
fi

for crate in "${CRATES[@]}"; do
  echo "=== cargo package -p ${crate} --list ==="
  if ! cargo package -p "$crate" --list --allow-dirty --no-verify 2>&1; then
    echo "FAIL: list packaging for ${crate}"
    fail=1
    continue
  fi
  echo "=== cargo package -p ${crate} (no verify compile) ==="
  if ! cargo package -p "$crate" --allow-dirty --no-verify 2>&1; then
    echo "FAIL: package ${crate}"
    fail=1
    continue
  fi
  echo "OK: ${crate}"
done

if [ "$fail" -ne 0 ]; then
  cat <<'EOF'

Packaging dry-run failed.
Common cause: `neo-vm-rs` is consumed via git+rev and is not (yet) on crates.io.
Until it is published:
  - local/workspace development remains valid
  - do not run `cargo publish` for neo-zkvm crates
  - publish `neo-vm-rs` first, then re-run this script

EOF
  exit 1
fi

echo "Packaging dry-run passed for: ${CRATES[*]}"
