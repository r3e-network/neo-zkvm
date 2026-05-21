# Neo zkVM Examples

Runnable Rust examples live in `crates/neo-zkvm-examples/src`.
This top-level directory is reserved for hand-authored `.neoasm` scripts and
small fixtures that are useful outside Cargo.

## Run a Single Example

```bash
cargo run -p neo-zkvm-examples --bin <name>
```

## Runnable Examples

- `basic` - Minimal VM execution (`2 + 3 = 5`).
- `proof_generation` - End-to-end proof generation + verification flow.
- `batch_verification` - Prove and verify many jobs together (batch pipeline).
- `private_inputs` - Commit private inputs via input-hash commitments.
- `tamper_resistance` - Verify untrusted proofs and reject tampered payloads.
- `zk_preimage` - Hash preimage proof for private data flows.
- `zk_scaling` - Scaling-oriented loop execution example.
- `zk_dao_voting` - Anonymous vote validity example.
- `zk_dex_rollup` - L2 DEX batch proving example.

## Validation

Run all example tests:

```bash
cargo test -p neo-zkvm-examples
```

Run all example binaries:

```bash
cargo run -p neo-zkvm-examples --bin basic
cargo run -p neo-zkvm-examples --bin proof_generation
cargo run -p neo-zkvm-examples --bin batch_verification
cargo run -p neo-zkvm-examples --bin private_inputs
cargo run -p neo-zkvm-examples --bin tamper_resistance
cargo run -p neo-zkvm-examples --bin zk_preimage
cargo run -p neo-zkvm-examples --bin zk_scaling
cargo run -p neo-zkvm-examples --bin zk_dao_voting
cargo run -p neo-zkvm-examples --bin zk_dex_rollup
```
