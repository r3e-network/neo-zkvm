#!/bin/bash
# Install the neo-zkvm CLI into ~/.cargo/bin (or CARGO_HOME/bin).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

source ~/.cargo/env 2>/dev/null || true

BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$BIN_DIR"

cargo build --release --locked -p neo-zkvm-cli
install -m 755 target/release/neo-zkvm "$BIN_DIR/neo-zkvm"

echo "Installed neo-zkvm $(target/release/neo-zkvm --version) to ${BIN_DIR}"
