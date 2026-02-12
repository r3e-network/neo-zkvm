# Neo zkVM Examples

This directory mirrors runnable examples from `crates/neo-zkvm-examples/src`.

## Run a Single Example

```bash
cargo run -p neo-zkvm-examples --bin <name>
```

## Core Examples

- `basic` - Minimal VM execution (`2 + 3 = 5`).
- `proof_generation` - End-to-end proof generation + verification flow.
- `native_contracts` - StdLib/CryptoLib contract operations.
- `storage_example` - Storage operations + Merkle-root state commitments.

## Common zkVM Usage Patterns

- `batch_verification` - Prove and verify many jobs together (batch pipeline).
- `private_inputs` - Commit private inputs via input-hash commitments.
- `tamper_resistance` - Verify untrusted proofs and reject tampered payloads.

## Validation

Run all example tests:

```bash
cargo test -p neo-zkvm-examples
```

Run all example binaries:

```bash
cargo run -p neo-zkvm-examples --bin basic
cargo run -p neo-zkvm-examples --bin native_contracts
cargo run -p neo-zkvm-examples --bin storage_example
cargo run -p neo-zkvm-examples --bin proof_generation
cargo run -p neo-zkvm-examples --bin batch_verification
cargo run -p neo-zkvm-examples --bin private_inputs
cargo run -p neo-zkvm-examples --bin tamper_resistance
```
