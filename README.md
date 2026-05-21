# Neo zkVM

[![CI](https://github.com/r3e-network/neo-zkvm/actions/workflows/ci.yml/badge.svg)](https://github.com/r3e-network/neo-zkvm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A **production-grade** zero-knowledge virtual machine for Neo N3, enabling verifiable computation with cryptographic proofs.

## Features

- ðŸ” **Real ZK Proofs** - SP1 integration for production-grade proving
- Shared VM semantics - proof execution uses the canonical `neo-vm-rs` interpreter shared with Neo RISC-V VM
- Neo N3 compatible - canonical NeoVM opcodes, stack values, and deterministic zk syscall adapters
- ðŸ’¾ **Storage Support** - Merkle-proven key-value storage
- ðŸ›ï¸ **Native Contracts** - StdLib, CryptoLib built-in
- ðŸ› ï¸ **Developer Tools** - CLI with assembler, disassembler, debugger

## Architecture

```text
neo-zkvm-cli
  |-- run / prove / asm / disasm / debug
  |
  +--> neo-zkvm-prover -----> SP1 or mock proof generation
  |          |
  |          +--> neo-vm-guest -----> neo-vm-rs canonical interpreter
  |                         |         shared StackValue / ExecutionResult
  |                         +-------> deterministic zk syscall adapters
  |
  +--> neo-zkvm-verifier ---> proof policy and public-input checks

neo-vm-core remains a compatibility/native-utility crate for storage,
StdLib/CryptoLib helpers, and legacy debugger surfaces. It is not the proof
execution engine.
```

## Quick Start

```bash
# Install from crates.io
CARGO_TARGET_DIR=/tmp/target cargo install neo-zkvm-cli

# Run a script
neo-zkvm run 12139E40  # 2 + 3

# Generate ZK proof (default: SP1)
neo-zkvm prove 12139E40

# Select proof mode explicitly (--proof-mode or -m)
neo-zkvm prove 12139E40 -m mock
neo-zkvm prove 12139E40 -m groth16

# Explicit SP1 mode with fallback allowed
neo-zkvm prove 12139E40 -m sp1 --allow-fallback

# Valid modes: execute | mock | sp1 | plonk | groth16
```

> For production SP1 proofs from a crates.io install, build from source with `--features sp1`, or reinstall with `NEO_ZKVM_PROGRAM_DIR=/path/to/neo-zkvm-program` so the SP1 guest ELF is compiled at install time.
>
> Explicit `-m sp1/plonk/groth16` now fails if it downgrades to `mock` unless you pass `--allow-fallback`.

## Feature Model

Neo zkVM separates deterministic local development from production SP1 proving:

| Feature set | Purpose | External prerequisites |
| --- | --- | --- |
| default | Neo VM execution, mock proofs, CLI, verifier policy, tests | Rust toolchain only |
| `sp1` | SP1 6.2.1 host prover/verifier integration | `protoc`, Succinct/SP1 toolchain, real guest ELF for production proofs |
| `SP1_FORCE_DUMMY=true` | CI/API compile check for SP1 feature | `protoc`; not valid for production proofs |

Recommended local gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
SP1_FORCE_DUMMY=true cargo clippy -p neo-zkvm-prover -p neo-zkvm-verifier -p neo-zkvm-cli --features sp1 --locked -- -D warnings
```

```bash
# Run full production-readiness gates locally
# Requires: cargo install cargo-deny
# Requires protoc and the Succinct/SP1 toolchain for the release proof smoke
./scripts/verify-production.sh

# Validate release notes/version metadata only
./scripts/verify-release-metadata.sh

# Validate crates.io package assembly only
./scripts/verify-packaging.sh

# Print the full release plan without tagging or publishing
./scripts/release.sh --plan

# Print the crates.io publish order without publishing
./scripts/publish-crates.sh --plan
```

A manual GitHub Actions release workflow is also available via `Actions -> Release -> Run workflow`
with `plan` or `verify` mode for remote operator runs.

## Installation

### From Source

```bash
git clone https://github.com/r3e-network/neo-zkvm
cd neo-zkvm
cargo build --release
```

### As Library

```toml
[dependencies]
neo-vm-guest = "0.2"
neo-zkvm-prover = "0.2"
neo-zkvm-verifier = "0.2"
```

## Usage

### Execute Script

```rust
use neo_vm_guest::{execute, ProofInput, StackItem};

let output = execute(ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40], // 2 + 3
    arguments: vec![],
    gas_limit: 1_000_000,
});

assert_eq!(output.result, Some(StackItem::Integer(5)));
```

### Generate & Verify Proof

```rust
use neo_zkvm_prover::{NeoProver, ProverConfig, ProofMode};
use neo_zkvm_verifier::verify_for_mode;
use neo_vm_guest::ProofInput;

let prover = NeoProver::new(ProverConfig {
    proof_mode: ProofMode::Mock, // Use Mock for testing, Sp1/Plonk/Groth16 for production
    ..Default::default()
});

let input = ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40],
    arguments: vec![],
    gas_limit: 1_000_000,
};

let proof = prover.prove(input);
assert!(verify_for_mode(&proof, ProofMode::Mock));
```

### Use Storage

```rust
use neo_vm_core::{TrackedStorage, StorageContext, StorageBackend};

let mut storage = TrackedStorage::new();
let ctx = StorageContext::default();

storage.put(&ctx, b"key", b"value").unwrap();
assert_eq!(storage.get(&ctx, b"key"), Some(b"value".to_vec()));

// Get Merkle root for ZK proof
let root = storage.merkle_root();
```

## Supported Opcodes

| Category     | Count | Examples                         |
| ------------ | ----- | -------------------------------- |
| Constants    | 25+   | PUSH0-16, PUSHDATA1-4, PUSHINT\* |
| Flow Control | 20+   | JMP, JMPIF, CALL, RET, ASSERT    |
| Stack        | 15+   | DUP, SWAP, ROT, PICK, ROLL       |
| Arithmetic   | 20+   | ADD, SUB, MUL, DIV, MOD, POW     |
| Bitwise      | 10+   | AND, OR, XOR, SHL, SHR           |
| Compound     | 15+   | PACK, NEWARRAY, PICKITEM         |
| Slots        | 20+   | LDLOC, STLOC, LDARG, STARG       |

## Benchmarks

```
arithmetic/add      time: [82.3 ns 85.1 ns 88.2 ns]
arithmetic/mul      time: [84.7 ns 87.3 ns 90.1 ns]
stack/dup           time: [45.2 ns 46.8 ns 48.5 ns]
loop/1000           time: [8.2 Âµs 8.5 Âµs 8.8 Âµs]
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [Architecture](docs/architecture.md)
- [Opcodes Reference](docs/opcodes.md)
- [CLI Reference](docs/cli.md)
- [Formal Verification](docs/formal-verification.md)
- [Completeness Proofs](docs/completeness-proof.md)
- [Production Readiness Report](PRODUCTION_READINESS_REPORT.md)
- [SP1 v6 Migration Notes](docs/sp1-v6-migration-notes.md)
- [Examples](examples/)

### Use Cases Included in Examples
- **zk_dao_voting**: Anonymous DAO voting logic proving validity without revealing vote choices.
- **zk_dex_rollup**: Zero-cost Layer 2 order matching proving thousands of transactions in a single verified root.
- **zk_preimage**: Hash preimage proof demonstrating secret password handling.
- **zk_scaling**: Complex algorithm scaling loop via Off-chain Computation.

## License

MIT License - see [LICENSE](LICENSE)
