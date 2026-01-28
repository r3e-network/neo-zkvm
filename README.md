# Neo zkVM

A zero-knowledge virtual machine implementation for Neo N3, enabling verifiable computation with cryptographic proofs.

## Overview

Neo zkVM allows executing Neo smart contract scripts and generating zero-knowledge proofs of correct execution. This enables:

- **Verifiable Computation**: Prove that a computation was executed correctly without revealing inputs
- **Privacy**: Execute sensitive logic while only revealing the result
- **Scalability**: Verify proofs instead of re-executing computations

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   neo-vm-core   │────▶│  neo-vm-guest   │────▶│ neo-zkvm-prover │
│   (VM Engine)   │     │ (Guest Program) │     │ (Proof Gen)     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
                                               ┌─────────────────┐
                                               │neo-zkvm-verifier│
                                               │ (Proof Verify)  │
                                               └─────────────────┘
```

## Crates

| Crate | Description |
|-------|-------------|
| `neo-vm-core` | Core VM engine with opcodes and execution |
| `neo-vm-guest` | Guest program for zkVM proving |
| `neo-zkvm-prover` | Proof generation with SP1 integration |
| `neo-zkvm-verifier` | Proof verification |
| `neo-zkvm-cli` | Command-line interface |

## Supported Operations

### Arithmetic
- ADD, SUB, MUL, DIV, MOD

### Comparison
- LT, LE, GT, GE, EQUAL

### Logical
- AND, OR, XOR, NOT, BOOLAND, BOOLOR

### Stack
- PUSH, DUP, DROP, SWAP, ROT, PICK, ROLL

### Control Flow
- JMP, JMPIF, JMPIFNOT, CALL, RET

### Cryptographic
- SHA256, RIPEMD160, Hash160, CHECKSIG (ECDSA)

## Quick Start

```rust
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProverConfig};
use neo_zkvm_verifier::verify;

// Script: 2 + 3
let script = vec![0x12, 0x13, 0x9E, 0x40];

let input = ProofInput {
    script,
    arguments: vec![],
    gas_limit: 1_000_000,
};

let prover = NeoProver::new(ProverConfig::default());
let proof = prover.prove(input);

assert!(verify(&proof));
println!("Result: {:?}", proof.output.result);
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Benchmarks

```bash
cargo bench -p neo-vm-core
```

## License

MIT License
