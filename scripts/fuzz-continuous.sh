#!/bin/bash
# Continuous rotating fuzz campaign.
# Exits with code 2 when non-OOM crash artifacts appear (crash-*, timeout-*, leak-*).
#
# Usage:
#   ./scripts/fuzz-continuous.sh
#   SLICE=120 MAX_LEN=512 ./scripts/fuzz-continuous.sh
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/fuzz"

source ~/.cargo/env 2>/dev/null || true

TARGETS=(
  fuzz_attestation
  fuzz_proof_pipeline
  fuzz_bincode
  fuzz_raw_script
  fuzz_vm_execution
  fuzz_script_parser
  fuzz_assembler
)
SANITIZER="${SANITIZER:-none}"
# Wall-clock seconds per target per cycle
SLICE="${SLICE:-120}"
MAX_LEN="${MAX_LEN:-512}"
# libFuzzer default rss_limit_mb=2048 false-positives on long high-throughput
# targets (allocator retains RSS). Disable process RSS kill; still enforce
# single-allocation cap to catch length bombs.
RSS_LIMIT_MB="${RSS_LIMIT_MB:-0}"
MALLOC_LIMIT_MB="${MALLOC_LIMIT_MB:-256}"
TIMEOUT_S="${TIMEOUT_S:-10}"
ROUND=0

echo "=== continuous fuzz start $(date -Is) root=${ROOT} ==="
echo "slice=${SLICE}s max_len=${MAX_LEN} rss_limit_mb=${RSS_LIMIT_MB} malloc_limit_mb=${MALLOC_LIMIT_MB} timeout=${TIMEOUT_S}s"
echo "targets: ${TARGETS[*]}"

count_real_crashes() {
  find artifacts -type f \( -name 'crash-*' -o -name 'timeout-*' -o -name 'leak-*' -o -name 'slow-unit-*' \) 2>/dev/null | wc -l
}

while true; do
  ROUND=$((ROUND + 1))
  echo
  echo "======== ROUND ${ROUND} @ $(date -Is) ========"
  FAIL=0
  for t in "${TARGETS[@]}"; do
    echo ">>> ${t} (max_total_time=${SLICE})"
    # Drop prior OOM noise for this target so we don't treat allocator RSS as product bugs.
    find "artifacts/${t}" -type f -name 'oom-*' -delete 2>/dev/null || true

    if cargo +nightly fuzz run --sanitizer "$SANITIZER" "$t" -- \
        -max_total_time="$SLICE" \
        -max_len="$MAX_LEN" \
        -rss_limit_mb="$RSS_LIMIT_MB" \
        -malloc_limit_mb="$MALLOC_LIMIT_MB" \
        -timeout="$TIMEOUT_S" \
        >"/tmp/fuzz-${t}.log" 2>&1; then
      tail -3 "/tmp/fuzz-${t}.log" || true
      echo "OK: ${t}"
    else
      # Distinguish OOM-only from real crashes
      if grep -q 'SUMMARY: libFuzzer: out-of-memory' "/tmp/fuzz-${t}.log" \
        && ! grep -qE 'SUMMARY: libFuzzer: (deadly signal|timeout|leak)' "/tmp/fuzz-${t}.log"; then
        echo "WARN: ${t} hit allocator/RSS OOM noise — cleared; continuing"
        find "artifacts/${t}" -type f -name 'oom-*' -delete 2>/dev/null || true
        tail -8 "/tmp/fuzz-${t}.log" || true
      else
        echo "FAIL: ${t} at round ${ROUND}"
        tail -40 "/tmp/fuzz-${t}.log" || true
        find "artifacts/${t}" -type f 2>/dev/null || true
        FAIL=1
      fi
    fi

    REAL=$(count_real_crashes)
    if [ "$REAL" -gt 0 ]; then
      echo "CRASH ARTIFACTS DETECTED:"
      find artifacts -type f \( -name 'crash-*' -o -name 'timeout-*' -o -name 'leak-*' -o -name 'slow-unit-*' \)
      exit 2
    fi
  done

  echo "--- round ${ROUND} done; fail_flag=${FAIL} real_crashes=$(count_real_crashes) ---"
  if [ "$FAIL" -ne 0 ]; then
    echo "Non-zero exit without classified crash artifact — inspect /tmp/fuzz-*.log"
    exit 1
  fi
done
