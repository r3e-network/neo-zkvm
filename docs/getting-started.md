# Getting Started

## Prerequisites

- Rust stable toolchain
- `protoc` only when building SP1-enabled crates
- SP1 toolchain only for production proof generation

## Build

```bash
git clone https://github.com/r3e-network/neo-zkvm
cd neo-zkvm
cargo build --workspace
```

## Run a Script

```bash
cargo run -p neo-zkvm-cli -- run 12139E40
```

`12139E40` means `PUSH2 PUSH3 ADD RET` and returns `Integer(5)`.

## Use as a Library

```toml
[dependencies]
neo-vm-guest = "0.2"
neo-zkvm-prover = "0.2"
neo-zkvm-verifier = "0.2"
```

```rust
use neo_vm_guest::{execute, ProofInput, StackItem};

let output = execute(ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40],
    arguments: vec![],
    gas_limit: 1_000_000,
});

assert_eq!(output.result, Some(StackItem::Integer(5)));
```

## Generate and Verify a Mock Proof

```rust
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

let prover = NeoProver::new(ProverConfig {
    proof_mode: ProofMode::Mock,
    ..Default::default()
});

let proof = prover.prove(ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40],
    arguments: vec![],
    gas_limit: 1_000_000,
});

assert!(verify_for_mode(&proof, ProofMode::Mock));
```

CLI prove/verify round-trip (mock for local tooling only):

```bash
cargo run -p neo-zkvm-cli -- prove 12139E40 -m mock -o /tmp/proof.bin
cargo run -p neo-zkvm-cli -- verify /tmp/proof.bin -m mock
```

## Real SP1 Proofs

```bash
cargo build --workspace --features sp1
cargo run -p neo-zkvm-cli --features sp1 -- prove 12139E40 -m sp1
```

For production use, provide a real guest ELF and do not rely on dummy/fallback proof behavior. Explicit `sp1`, `plonk`, and `groth16` requests fail on fallback unless `--allow-fallback` is supplied.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```