# Neo zkVM

[![CI](https://github.com/neo-project/neo-zkvm/actions/workflows/ci.yml/badge.svg)](https://github.com/neo-project/neo-zkvm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A **production-grade** zero-knowledge virtual machine for Neo N3, enabling verifiable computation with cryptographic proofs.

## Features

- 🔐 **Real ZK Proofs** - SP1 integration for production-grade proving
- ⚡ **High Performance** - Optimized VM execution (~85ns per arithmetic op)
- 🔄 **Neo N3 Compatible** - 100+ opcodes, full Neo VM compatibility
- 💾 **Storage Support** - Merkle-proven key-value storage
- 🏛️ **Native Contracts** - StdLib, CryptoLib built-in
- 🛠️ **Developer Tools** - CLI with assembler, disassembler, debugger

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Neo zkVM Stack                        │
├─────────────────────────────────────────────────────────┤
│  neo-zkvm-cli     │ CLI tools (run, prove, asm, disasm) │
├───────────────────┼─────────────────────────────────────┤
│  neo-zkvm-prover  │ SP1 proof generation (PLONK/Groth16)│
├───────────────────┼─────────────────────────────────────┤
│  neo-zkvm-verifier│ Cryptographic proof verification    │
├───────────────────┼─────────────────────────────────────┤
│  neo-zkvm-program │ SP1 guest program (zkVM execution)  │
├───────────────────┼─────────────────────────────────────┤
│  neo-vm-core      │ VM engine, storage, native contracts│
└─────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Install
cargo install neo-zkvm-cli

# Run a script
neo-zkvm run 12139E40  # 2 + 3

# Generate ZK proof
neo-zkvm prove 12139E40
```

## Installation

### From Source

```bash
git clone https://github.com/neo-project/neo-zkvm
cd neo-zkvm
cargo build --release
```

### As Library

```toml
[dependencies]
neo-vm-core = "0.1"
neo-zkvm-prover = "0.1"
neo-zkvm-verifier = "0.1"
```

## Usage

### Execute Script

```rust
use neo_vm_core::{NeoVM, VMState, StackItem};

let mut vm = NeoVM::new(1_000_000);
vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]); // 2 + 3

while !matches!(vm.state, VMState::Halt | VMState::Fault) {
    vm.execute_next().unwrap();
}

assert_eq!(vm.eval_stack.pop(), Some(StackItem::Integer(5)));
```

### Generate & Verify Proof

```rust
use neo_zkvm_prover::{NeoProver, ProverConfig, ProveMode};
use neo_zkvm_verifier::verify;
use neo_vm_guest::ProofInput;

let prover = NeoProver::new(ProverConfig {
    prove_mode: ProveMode::Sp1,
    ..Default::default()
});

let input = ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40],
    arguments: vec![],
    gas_limit: 1_000_000,
};

let proof = prover.prove(input);
assert!(verify(&proof));
```

### Use Storage

```rust
use neo_vm_core::{TrackedStorage, StorageContext, StorageBackend};

let mut storage = TrackedStorage::new();
let ctx = StorageContext::default();

storage.put(&ctx, b"key", b"value");
assert_eq!(storage.get(&ctx, b"key"), Some(b"value".to_vec()));

// Get Merkle root for ZK proof
let root = storage.merkle_root();
```

## Supported Opcodes

| Category | Count | Examples |
|----------|-------|----------|
| Constants | 25+ | PUSH0-16, PUSHDATA1-4, PUSHINT* |
| Flow Control | 20+ | JMP, JMPIF, CALL, RET, ASSERT |
| Stack | 15+ | DUP, SWAP, ROT, PICK, ROLL |
| Arithmetic | 20+ | ADD, SUB, MUL, DIV, MOD, POW |
| Bitwise | 10+ | AND, OR, XOR, SHL, SHR |
| Compound | 15+ | PACK, NEWARRAY, PICKITEM |
| Slots | 20+ | LDLOC, STLOC, LDARG, STARG |

## Benchmarks

```
arithmetic/add      time: [82.3 ns 85.1 ns 88.2 ns]
arithmetic/mul      time: [84.7 ns 87.3 ns 90.1 ns]
stack/dup           time: [45.2 ns 46.8 ns 48.5 ns]
loop/1000           time: [8.2 µs 8.5 µs 8.8 µs]
```

## Documentation

- [Architecture](docs/architecture.md)
- [Opcodes Reference](docs/opcodes.md)
- [CLI Reference](docs/cli.md)
- [Formal Verification](docs/formal-verification.md)
- [Completeness Proofs](docs/completeness-proof.md)
- [Examples](examples/)

## License

MIT License - see [LICENSE](LICENSE)
