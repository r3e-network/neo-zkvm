#!/bin/bash
set -e
cargo build --release
cp target/release/neo-zkvm ~/.cargo/bin/
echo "Installed neo-zkvm"
