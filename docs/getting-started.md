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
# Install the CLI tool
cargo install --path crates/neo-zkvm-cli

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
neo-zkvm run 1213 9E40

# Output:
# State: Halt
# Gas: 11
# Stack: [Integer(5)]
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
use neo_zkvm_prover::{NeoProver, ProverConfig, ProveMode};
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
        prove_mode: ProveMode::Mock,
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
# Generate a proof for a script
neo-zkvm prove 12139e40

# Output:
# Result: [Integer(5)]
# Verified: true
```

## Working with Storage

Neo zkVM supports persistent storage operations.

```rust
use neo_vm_core::{MemoryStorage, StorageBackend, TrackedStorage};

fn main() {
    // Create in-memory storage
    let mut storage = MemoryStorage::new();
    
    // Store a value
    storage.put(b"mykey".to_vec(), b"myvalue".to_vec());
    
    // Retrieve the value
    let value = storage.get(b"mykey");
    println!("Value: {:?}", value);
    
    // Use tracked storage for change logging
    let mut tracked = TrackedStorage::new(storage);
    tracked.put(b"key2".to_vec(), b"value2".to_vec());
    
    // Get all changes
    println!("Changes: {:?}", tracked.get_changes());
    
    // Compute Merkle root
    println!("Merkle root: {:?}", tracked.compute_merkle_root());
}
```

## Native Contracts

Neo zkVM includes built-in native contracts for common operations.

```rust
use neo_vm_core::{NativeRegistry, StdLib, CryptoLib};

fn main() {
    // Create registry and register contracts
    let mut registry = NativeRegistry::new();
    registry.register(Box::new(StdLib));
    registry.register(Box::new(CryptoLib));
    
    // Use StdLib for base64 encoding
    let data = b"Hello, Neo!";
    let encoded = StdLib::base64_encode(data);
    println!("Base64: {}", encoded);
    
    // Use CryptoLib for hashing
    let hash = CryptoLib::sha256(data);
    println!("SHA256: {}", hex::encode(&hash));
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
| `Sp1Plonk` | Slowest | On-chain verification |

```rust
use neo_zkvm_prover::{ProverConfig, ProveMode};

// For development
let dev_config = ProverConfig {
    prove_mode: ProveMode::Execute,
    max_cycles: 1_000_000,
};

// For testing
let test_config = ProverConfig {
    prove_mode: ProveMode::Mock,
    max_cycles: 1_000_000,
};

// For production
let prod_config = ProverConfig {
    prove_mode: ProveMode::Sp1,
    max_cycles: 10_000_000,
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
- **[Examples](../examples/)** - More code examples

## Troubleshooting

### Common Issues

**Build fails with SP1 errors:**
```bash
# Install SP1 toolchain
curl -L https://sp1.succinct.xyz | bash
sp1up
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
```
```
