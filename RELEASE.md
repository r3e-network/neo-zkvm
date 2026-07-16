# Release Guide

## Pre-Release Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/verify-packaging.sh
./scripts/verify-release-metadata.sh
```

For SP1-enabled release validation:

```bash
SP1_FORCE_DUMMY=true cargo clippy -p neo-zkvm-prover -p neo-zkvm-verifier -p neo-zkvm-cli --features sp1 -- -D warnings
./scripts/verify-production.sh
```

A production proof release must be validated with a real SP1 guest ELF. Dummy SP1 builds are compile smoke tests only.

## Package Order

1. `neo-vm-guest`
2. `neo-zkvm-prover`
3. `neo-zkvm-verifier` (optional `sp1` feature depends on prover for ELF/host verify)
4. `neo-zkvm-cli`
5. `neo-zkvm-examples` remains unpublished (`publish = false`)

## Operator Checks

- Confirm all crates depend on shared `neo-vm-rs` semantics.
- Confirm no crate depends on a legacy local VM engine.
- Confirm explicit `sp1`, `plonk`, and `groth16` modes fail on fallback unless `--allow-fallback` is supplied.
- Confirm release notes describe proof mode behavior and SP1 prerequisites.