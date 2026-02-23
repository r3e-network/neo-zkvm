#!/bin/bash
set -euo pipefail

cargo build --release --locked -p neo-zkvm-cli
cp target/release/neo-zkvm ~/.cargo/bin/
echo "Installed neo-zkvm"
