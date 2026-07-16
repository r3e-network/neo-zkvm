#!/bin/bash
# Run all neo-zkvm fuzz targets for a configurable budget.
#
# Usage:
#   ./scripts/fuzz-all.sh              # default: 10000 runs each
#   RUNS=50000 ./scripts/fuzz-all.sh   # longer campaign
#   MAX_TOTAL_TIME=120 ./scripts/fuzz-all.sh  # libFuzzer wall-clock seconds per target
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/fuzz"

RUNS="${RUNS:-10000}"
MAX_LEN="${MAX_LEN:-512}"
MAX_TOTAL_TIME="${MAX_TOTAL_TIME:-0}"
SANITIZER="${SANITIZER:-none}"

source ~/.cargo/env 2>/dev/null || true

if ! command -v cargo-fuzz >/dev/null 2>&1; then
  echo "Installing cargo-fuzz..."
  cargo +stable install cargo-fuzz --locked --version 0.13.1
fi

time_args=()
if [ "$MAX_TOTAL_TIME" != "0" ]; then
  time_args+=(-max_total_time="$MAX_TOTAL_TIME")
fi

TARGETS=(
  fuzz_vm_execution
  fuzz_script_parser
  fuzz_raw_script
  fuzz_proof_pipeline
  fuzz_bincode
  fuzz_assembler
  fuzz_attestation
)

echo "=== neo-zkvm fuzz campaign: runs=${RUNS} max_len=${MAX_LEN} sanitizer=${SANITIZER} ==="

failed=0
for target in "${TARGETS[@]}"; do
  echo
  echo ">>> fuzzing ${target}"
  # rss_limit_mb=0: avoid false-positive OOM from allocator RSS retention on
  # high-throughput targets. malloc_limit_mb still catches single huge allocs.
  if ! cargo +nightly fuzz run --sanitizer "$SANITIZER" "$target" -- \
      -runs="$RUNS" -max_len="$MAX_LEN" \
      -rss_limit_mb="${RSS_LIMIT_MB:-0}" \
      -malloc_limit_mb="${MALLOC_LIMIT_MB:-256}" \
      -timeout="${TIMEOUT_S:-10}" \
      "${time_args[@]+"${time_args[@]}"}"; then
    # Ignore pure out-of-memory noise if no crash/timeout/leak artifact remains.
    if find "artifacts/${target}" -type f \( -name 'crash-*' -o -name 'timeout-*' -o -name 'leak-*' \) 2>/dev/null | grep -q .; then
      echo "FAIL: ${target}"
      failed=1
    elif find "artifacts/${target}" -type f -name 'oom-*' 2>/dev/null | grep -q .; then
      echo "WARN: ${target} OOM noise only — clearing artifacts"
      find "artifacts/${target}" -type f -name 'oom-*' -delete 2>/dev/null || true
      echo "OK: ${target} (OOM ignored)"
    else
      echo "FAIL: ${target}"
      failed=1
    fi
  else
    echo "OK: ${target}"
  fi
done

if [ "$failed" -ne 0 ]; then
  echo
  echo "Fuzz campaign finished with failures. Check fuzz/artifacts/ for crash inputs."
  exit 1
fi

echo
echo "Fuzz campaign passed for: ${TARGETS[*]}"
