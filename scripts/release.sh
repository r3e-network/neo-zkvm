#!/bin/bash
# Print or run the release plan. Tagging/publishing require explicit non-plan mode.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---plan}"

VERSION="$(
  sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1
)"

plan() {
  cat <<EOF
Neo zkVM release plan for v${VERSION}

1. Metadata
   ./scripts/verify-release-metadata.sh

2. Packaging dry-run
   ./scripts/verify-packaging.sh

3. Full production gates
   ./scripts/verify-production.sh

4. Git tag (manual)
   git tag -a v${VERSION} -m "neo-zkvm ${VERSION}"
   git push origin v${VERSION}

5. Publish crates (manual, after tag)
   ./scripts/publish-crates.sh --plan
   ./scripts/publish-crates.sh

Notes:
- Production SP1 proofs require a real guest ELF (not SP1_FORCE_DUMMY).
- neo-zkvm-examples is not published (publish = false).
- Default publish order: guest → prover → verifier → attestation → cli.
EOF
}

case "$MODE" in
  --plan | plan)
    plan
    ;;
  --verify | verify)
    ./scripts/verify-release-metadata.sh
    ./scripts/verify-packaging.sh
    ./scripts/verify-production.sh
    ;;
  *)
    echo "Usage: $0 [--plan|--verify]" >&2
    exit 2
    ;;
esac
