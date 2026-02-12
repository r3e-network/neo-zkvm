#!/bin/bash
set -e
source ~/.cargo/env 2>/dev/null || true
cargo test --all
cargo clippy --all --all-targets -- -D warnings
echo "Core checks passed (tests + clippy). For full release gates run scripts/verify-production.sh"
