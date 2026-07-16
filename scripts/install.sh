#!/bin/bash
# Install the neo-zkvm CLI into ~/.cargo/bin (or CARGO_HOME/bin).
#
# Default build: no SP1 feature (prove defaults to mock).
# For production SP1 proving:
#   FEATURES=sp1 ./scripts/install.sh
# Requires protoc + Succinct/SP1 toolchain and a real guest ELF.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

source ~/.cargo/env 2>/dev/null || true

BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$BIN_DIR"

FEATURES="${FEATURES:-}"
if [ -n "$FEATURES" ]; then
  cargo build --release --locked -p neo-zkvm-cli --features "$FEATURES"
else
  cargo build --release --locked -p neo-zkvm-cli
fi
install -m 755 target/release/neo-zkvm "$BIN_DIR/neo-zkvm"

echo "Installed neo-zkvm $(target/release/neo-zkvm --version) to ${BIN_DIR}"
if [ -z "$FEATURES" ]; then
  echo "Note: built without --features sp1 (default prove mode: mock)."
  echo "      For SP1: FEATURES=sp1 $0"
fi
