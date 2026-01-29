#!/bin/bash
set -e
source ~/.cargo/env 2>/dev/null || true
cargo test --all
cargo clippy --all
echo "All checks passed"
