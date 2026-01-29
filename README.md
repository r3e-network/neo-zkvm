# Neo zkVM

[![CI](https://github.com/neo-project/neo-zkvm/actions/workflows/ci.yml/badge.svg)](https://github.com/neo-project/neo-zkvm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A zero-knowledge virtual machine for Neo N3, enabling verifiable computation with cryptographic proofs.

## Features

- 🔐 **Zero-Knowledge Proofs** - Prove computation correctness without revealing inputs
- ⚡ **High Performance** - Optimized VM execution (~85ns per arithmetic op)
- 🔄 **Neo N3 Compatible** - Full opcode compatibility with Neo VM
- 🛠️ **Developer Tools** - CLI with assembler, disassembler, and debugger

## Quick Start

```bash
# Install
cargo install neo-zkvm-cli

# Run a script
neo-zkvm run 12139E40  # 2 + 3

# Generate proof
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
use neo_vm_core::{NeoVM, VMState};

let mut vm = NeoVM::new(1_000_000);
vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]); // 2 + 3

while !matches!(vm.state, VMState::Halt | VMState::Fault) {
    vm.execute_next().unwrap();
}

println!("Result: {:?}", vm.eval_stack.pop());
```

### Generate Proof

```rust
use neo_zkvm_prover::{NeoProver, ProverConfig};
use neo_zkvm_verifier::verify;

let prover = NeoProver::new(ProverConfig::default());
let proof = prover.prove(input);
assert!(verify(&proof));
```

## Documentation

- [Architecture](docs/architecture.md)
- [Opcodes](docs/opcodes.md)
- [CLI Reference](docs/cli.md)
- [Examples](examples/)

## License

MIT License - see [LICENSE](LICENSE)
