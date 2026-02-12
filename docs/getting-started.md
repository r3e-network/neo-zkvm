# Getting Started with Neo zkVM

This guide will help you get up and running with Neo zkVM quickly.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (1.75 or later) - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository
- **SP1** (optional) - For real proof generation

### Check Your Environment

```bash
# Check Rust version
rustc --version
# Should be 1.75.0 or later

# Check Cargo
cargo --version
```

## Installation

### Option 1: From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/neo-project/neo-zkvm.git
cd neo-zkvm

# Build all crates
cargo build --release

# Run tests to verify installation
cargo test
```

### Option 2: Add as Dependency

Add Neo zkVM to your `Cargo.toml`:

```toml
[dependencies]
neo-vm-core = { git = "https://github.com/neo-project/neo-zkvm" }
neo-zkvm-prover = { git = "https://github.com/neo-project/neo-zkvm" }
neo-zkvm-verifier = { git = "https://github.com/neo-project/neo-zkvm" }
```

### Installing the CLI

```bash
# Install the CLI tool from this repo
cargo install --path crates/neo-zkvm-cli

# Install from crates.io (recommended target-dir workaround for SP1 deps)
CARGO_TARGET_DIR=/tmp/target cargo install neo-zkvm-cli

# Verify installation
neo-zkvm --version
```

## Quick Start

### Your First Script

Let's create a simple script that adds two numbers.

#### Using Rust API

```rust
use neo_vm_core::{NeoVM, VMState, StackItem};

fn main() {
    // Create a VM with 1 million gas limit
    let mut vm = NeoVM::new(1_000_000);
    
    // Load a script: PUSH2, PUSH3, ADD, RET
    // This computes 2 + 3 = 5
    vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]);
    
    // Execute until completion
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }
    
    // Check the result
    println!("State: {:?}", vm.state);
    println!("Gas consumed: {}", vm.gas_consumed);
    println!("Result: {:?}", vm.eval_stack.pop());
}
```

#### Using the CLI

```bash
# Run a hex-encoded script
neo-zkvm run 12139E40

# Output:
# Executing script...
#
# ═══════════════════════════════════════
#   EXECUTION RESULT
# ═══════════════════════════════════════
#   State:        Halt
#   Gas consumed: 12
#   Stack depth:  1
# ───────────────────────────────────────
#   Stack (top → bottom):
#     [0] Integer(5)
# ═══════════════════════════════════════
```

### Assembly Language

Neo zkVM supports a simple assembly language for writing scripts.

Create a file `add.neoasm`:

```asm
; Simple addition: 2 + 3
PUSH2       ; Push 2 onto stack
PUSH3       ; Push 3 onto stack
ADD         ; Pop two values, push sum
RET         ; Return
```

Assemble and run:

```bash
# Assemble to bytecode
neo-zkvm asm add.neoasm
# Output: 12139e40

# Run the assembled bytecode
neo-zkvm run 12139e40
```

## Generating Proofs

The real power of Neo zkVM is generating zero-knowledge proofs of execution.

### Basic Proof Generation

```rust
use neo_vm_core::StackItem;
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProverConfig, ProofMode};
use neo_zkvm_verifier::verify;

fn main() {
    // Prepare the input
    let input = ProofInput {
        script: vec![0x12, 0x13, 0x9E, 0x40], // 2 + 3
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    
    // Create prover with mock mode (fast, for testing)
    let config = ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    };
    let prover = NeoProver::new(config);
    
    // Generate proof
    let proof = prover.prove(input);
    
    // Verify the proof
    let is_valid = verify(&proof);
    
    println!("Execution result: {:?}", proof.output.result);
    println!("Gas consumed: {}", proof.output.gas_consumed);
    println!("Proof valid: {}", is_valid);
}
```

### Using the CLI

```bash
# Generate a proof for a script (default: sp1)
neo-zkvm prove 12139e40

# Fast local development/debug mode
neo-zkvm prove 12139e40 -m mock

# SP1 proving in release mode
cargo run -p neo-zkvm-cli --release -- prove 12139e40 -m sp1

# You can also use execute/plonk/groth16
# neo-zkvm prove 12139e40 -m execute
# neo-zkvm prove 12139e40 -m plonk
# neo-zkvm prove 12139e40 -m groth16

# If SP1 is unavailable but you want to proceed with mock fallback
neo-zkvm prove 12139e40 -m sp1 --allow-fallback

# Valid modes for --proof-mode / -m:
# execute, mock, sp1, plonk, groth16

# Output:
# Result: [Integer(5)]
# Verified: true
```

For production SP1 proofs from a crates.io install, either build from source or reinstall with `NEO_ZKVM_PROGRAM_DIR=/path/to/neo-zkvm-program` so the guest ELF is compiled during installation.

Explicit `-m sp1/plonk/groth16` fails on downgrade to `mock` unless `--allow-fallback` is set.

## Working with Storage

Neo zkVM supports persistent storage operations with Merkle proof support.

```rust
use neo_vm_core::{MemoryStorage, StorageBackend, StorageContext, TrackedStorage};

fn main() {
    // Create in-memory storage
    let mut storage = MemoryStorage::new();
    
    // Create a storage context
    let ctx = StorageContext::default();
    
    // Store a value
    storage.put(&ctx, b"mykey", b"myvalue");
    
    // Retrieve the value
    let value = storage.get(&ctx, b"mykey");
    println!("Value: {:?}", value);
    
    // Use tracked storage for change logging
    let mut tracked = TrackedStorage::new();
    tracked.put(&ctx, b"key2", b"value2");
    
    // Get all changes
    println!("Changes: {:?}", tracked.changes());
    
    // Compute Merkle root
    println!("Merkle root: {:?}", tracked.merkle_root());
}
```

## Native Contracts

Neo zkVM includes built-in native contracts for common operations.

```rust
use neo_vm_core::{NativeContract, NativeRegistry, StdLib, CryptoLib, StackItem};

fn main() {
    // Create registry with built-in contracts
    let registry = NativeRegistry::new();
    
    // Use StdLib for serialization
    let stdlib = StdLib::new();
    let item = StackItem::Integer(42);
    let serialized = stdlib.invoke("serialize", vec![item]).unwrap();
    println!("Serialized: {:?}", serialized);
    
    // Use CryptoLib for hashing
    let cryptolib = CryptoLib::new();
    let data = StackItem::ByteString(b"Hello, Neo!".to_vec());
    let hash = cryptolib.invoke("sha256", vec![data]).unwrap();
    println!("SHA256: {:?}", hash);
}
```

## Example: Fibonacci Calculator

Here's a more complex example that calculates Fibonacci numbers.

Create `fibonacci.neoasm`:

```asm
; Calculate Fibonacci(10)
; Result: 55

PUSH10      ; n = 10
PUSH0       ; a = 0
PUSH1       ; b = 1

; Loop: while n > 0
:loop
ROT         ; bring n to top
DUP         ; duplicate n
PUSH0       ; push 0
JMPLE end   ; if n <= 0, exit

; Calculate next Fibonacci
DEC         ; n = n - 1
ROT         ; bring a to top
ROT         ; bring b to top  
OVER        ; copy a
ADD         ; new_b = a + b
SWAP        ; swap to get (new_a=old_b, new_b)
ROT         ; put n back on top
JMP loop    ; continue loop

:end
DROP        ; remove n
DROP        ; remove a
RET         ; return b (the result)
```

## Proof Modes

Neo zkVM supports different proving modes for various use cases:

| Mode | Speed | Use Case |
|------|-------|----------|
| `Execute` | Instant | Development, debugging |
| `Mock` | Fast | Testing, CI/CD |
| `Sp1` | Slow | Off-chain verification |
| `Plonk` | Slowest | On-chain verification (Ethereum) |
| `Groth16` | Slowest | On-chain verification (smallest proof) |

```rust
use neo_zkvm_prover::{ProverConfig, ProofMode};

// For development
let dev_config = ProverConfig {
    proof_mode: ProofMode::Execute,
    ..Default::default()
};

// For testing
let test_config = ProverConfig {
    proof_mode: ProofMode::Mock,
    ..Default::default()
};

// For production
let prod_config = ProverConfig {
    proof_mode: ProofMode::Sp1,
    ..Default::default()
};
```

## Execution Tracing

Enable tracing to capture execution details for debugging:

```rust
use neo_vm_core::{NeoVM, VMState};

fn main() {
    let mut vm = NeoVM::new(1_000_000);
    
    // Enable tracing before execution
    vm.enable_tracing();
    
    vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]);
    
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }
    
    // Access the execution trace
    println!("Trace steps: {}", vm.trace.steps.len());
    for step in &vm.trace.steps {
        println!("  IP: {}, Op: 0x{:02X}, Gas: {}", 
            step.ip, step.opcode, step.gas_consumed);
    }
}
```

## Next Steps

Now that you have the basics, explore these resources:

- **[Architecture](architecture.md)** - Deep dive into system design
- **[Opcodes Reference](opcodes.md)** - Complete opcode documentation
- **[API Reference](api-reference.md)** - Full API documentation
- **[Production Readiness Report](../PRODUCTION_READINESS_REPORT.md)** - Current release readiness status
- **[SP1 v6 Migration Notes](sp1-v6-migration-notes.md)** - Future upgrade prerequisites and risks
- **[Examples](../examples/)** - More code examples

## Troubleshooting

### Common Issues

**Build fails with SP1 errors:**
```bash
# Install/update SP1 toolchain
curl -L https://sp1.succinct.xyz | bash
sp1up
```

If you are evaluating SP1 v6 pre-releases, review **[SP1 v6 Migration Notes](sp1-v6-migration-notes.md)** first for additional prerequisites (for example `protoc` and target/toolchain alignment).

**`cargo install neo-zkvm-cli` fails with `OUT_DIR does not have parent called "target"`:**
```bash
CARGO_TARGET_DIR=/tmp/target cargo install neo-zkvm-cli
```

**Out of gas error:**
```rust
// Increase gas limit
let mut vm = NeoVM::new(10_000_000);
```

**Stack underflow:**
- Check that you have enough values on the stack
- Use `DEPTH` opcode to debug stack size

## Getting Help

- **GitHub Issues**: [neo-project/neo-zkvm](https://github.com/neo-project/neo-zkvm/issues)
- **Neo Documentation**: [docs.neo.org](https://docs.neo.org)
