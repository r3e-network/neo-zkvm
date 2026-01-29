# API Reference

This document provides comprehensive API documentation for Neo zkVM.

## Table of Contents

- [neo-vm-core](#neo-vm-core)
- [neo-vm-guest](#neo-vm-guest)
- [neo-zkvm-prover](#neo-zkvm-prover)
- [neo-zkvm-verifier](#neo-zkvm-verifier)

---

## neo-vm-core

The core virtual machine implementation.

### NeoVM

Main VM execution engine.

```rust
pub struct NeoVM {
    pub state: VMState,
    pub eval_stack: Vec<StackItem>,
    pub invocation_stack: Vec<ExecutionContext>,
    pub gas_consumed: u64,
    pub gas_limit: u64,
    pub notifications: Vec<StackItem>,
    pub logs: Vec<String>,
    pub trace: ExecutionTrace,
    pub tracing_enabled: bool,
    pub local_slots: Vec<StackItem>,
    pub argument_slots: Vec<StackItem>,
    pub static_slots: Vec<StackItem>,
}
```

#### Methods

##### `new(gas_limit: u64) -> Self`

Create a new VM instance with the specified gas limit.

```rust
let vm = NeoVM::new(1_000_000);
```

##### `load_script(script: Vec<u8>)`

Load a script into the VM for execution.

```rust
vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]);
```

##### `execute_next() -> Result<(), VMError>`

Execute the next instruction.

```rust
while !matches!(vm.state, VMState::Halt | VMState::Fault) {
    vm.execute_next()?;
}
```

##### `enable_tracing()`

Enable execution tracing for proof generation.

```rust
vm.enable_tracing();
```

---

### VMState

Execution state enumeration.

```rust
pub enum VMState {
    None,   // Initial state
    Halt,   // Successful completion
    Fault,  // Error occurred
    Break,  // Breakpoint hit
}
```

#### Usage

```rust
match vm.state {
    VMState::Halt => println!("Success!"),
    VMState::Fault => println!("Error!"),
    _ => println!("Still running..."),
}
```

---

### StackItem

Stack value types.

```rust
pub enum StackItem {
    Null,
    Boolean(bool),
    Integer(i128),
    ByteString(Vec<u8>),
    Buffer(Vec<u8>),
    Array(Vec<StackItem>),
    Struct(Vec<StackItem>),
    Map(Vec<(StackItem, StackItem)>),
    Pointer(u32),
}
```

#### Methods

##### `to_integer() -> Option<i128>`

Convert to integer if possible.

```rust
if let Some(n) = item.to_integer() {
    println!("Value: {}", n);
}
```

##### `to_bool() -> bool`

Convert to boolean.

```rust
let is_true = item.to_bool();
```

---

### VMError

Error types returned by VM operations.

```rust
pub enum VMError {
    StackUnderflow,
    InvalidOpcode(u8),
    OutOfGas,
    DivisionByZero,
    InvalidType,
    UnknownSyscall(u32),
    InvalidOperation,
}
```

#### Example

```rust
match vm.execute_next() {
    Ok(()) => {},
    Err(VMError::OutOfGas) => println!("Ran out of gas!"),
    Err(VMError::StackUnderflow) => println!("Stack underflow!"),
    Err(e) => println!("Error: {}", e),
}
```

---

### ExecutionTrace

Trace of execution for proof generation.

```rust
pub struct ExecutionTrace {
    pub steps: Vec<TraceStep>,
    pub initial_state_hash: [u8; 32],
    pub final_state_hash: [u8; 32],
}

pub struct TraceStep {
    pub ip: usize,
    pub opcode: u8,
    pub stack_hash: [u8; 32],
    pub gas_consumed: u64,
}
```

---

### Storage Types

#### StorageBackend Trait

```rust
pub trait StorageBackend {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>);
    fn delete(&mut self, key: &[u8]);
    fn contains(&self, key: &[u8]) -> bool;
}
```

#### MemoryStorage

In-memory storage implementation.

```rust
use neo_vm_core::MemoryStorage;

let mut storage = MemoryStorage::new();
storage.put(b"key".to_vec(), b"value".to_vec());
let value = storage.get(b"key");
```

#### TrackedStorage

Storage with change tracking and Merkle root computation.

```rust
use neo_vm_core::TrackedStorage;

let mut tracked = TrackedStorage::new(MemoryStorage::new());
tracked.put(b"key".to_vec(), b"value".to_vec());

// Get all changes
let changes = tracked.get_changes();

// Compute Merkle root
let root = tracked.compute_merkle_root();
```

---

### Native Contracts

#### StdLib

Standard library functions.

```rust
use neo_vm_core::StdLib;

// Base64 encoding/decoding
let encoded = StdLib::base64_encode(b"hello");
let decoded = StdLib::base64_decode(&encoded);

// Number conversion
let str_num = StdLib::itoa(42, 10);  // "42"
let num = StdLib::atoi("42", 10);     // 42
```

#### CryptoLib

Cryptographic functions.

```rust
use neo_vm_core::CryptoLib;

// Hash functions
let sha256_hash = CryptoLib::sha256(b"data");
let ripemd_hash = CryptoLib::ripemd160(b"data");
let hash160 = CryptoLib::hash160(b"data");

// Murmur hash
let murmur = CryptoLib::murmur32(b"data", 0);
```

#### NativeRegistry

Registry for managing native contracts.

```rust
use neo_vm_core::{NativeRegistry, StdLib, CryptoLib};

let mut registry = NativeRegistry::new();
registry.register(Box::new(StdLib));
registry.register(Box::new(CryptoLib));

// Invoke by hash
let result = registry.invoke(hash, "method", args);
```

---

## neo-vm-guest

Guest program interface for zkVM proving.

### ProofInput

Input for proof generation.

```rust
pub struct ProofInput {
    pub script: Vec<u8>,
    pub arguments: Vec<StackItem>,
    pub gas_limit: u64,
}
```

#### Example

```rust
use neo_vm_guest::ProofInput;
use neo_vm_core::StackItem;

let input = ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40],
    arguments: vec![StackItem::Integer(42)],
    gas_limit: 1_000_000,
};
```

### ProofOutput

Output from proof execution.

```rust
pub struct ProofOutput {
    pub result: Vec<StackItem>,
    pub gas_consumed: u64,
    pub state: u8,  // 0 = Halt, 1 = Fault
}
```

### execute Function

Execute a script and return the output.

```rust
use neo_vm_guest::{execute, ProofInput};

let output = execute(input);
println!("Result: {:?}", output.result);
```

---

## neo-zkvm-prover

Proof generation using SP1 framework.

### NeoProver

Main prover struct.

```rust
use neo_zkvm_prover::{NeoProver, ProverConfig};

let prover = NeoProver::new(ProverConfig::default());
let proof = prover.prove(input);
```

### ProverConfig

Configuration for the prover.

```rust
pub struct ProverConfig {
    pub max_cycles: u64,
    pub prove_mode: ProveMode,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            max_cycles: 1_000_000,
            prove_mode: ProveMode::Mock,
        }
    }
}
```

### ProveMode

Proving mode enumeration.

```rust
pub enum ProveMode {
    Execute,    // No proof, execution only
    Mock,       // Simulated proof for testing
    Sp1,        // Real SP1 compressed proof
    Sp1Plonk,   // SP1 PLONK proof for on-chain
}
```

### NeoProof

Generated proof structure.

```rust
pub struct NeoProof {
    pub output: ProofOutput,
    pub proof_bytes: Vec<u8>,
    pub public_inputs: PublicInputs,
    pub vkey_hash: [u8; 32],
}
```

### PublicInputs

Public inputs for verification.

```rust
pub struct PublicInputs {
    pub script_hash: [u8; 32],
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub gas_consumed: u64,
    pub execution_success: bool,
}
```

---

## neo-zkvm-verifier

Proof verification layer.

### verify Function

Simple verification interface.

```rust
use neo_zkvm_verifier::verify;

let is_valid = verify(&proof);
```

### verify_detailed Function

Detailed verification with error information.

```rust
use neo_zkvm_verifier::verify_detailed;

let result = verify_detailed(&proof);
if result.valid {
    println!("Proof is valid!");
} else {
    println!("Error: {:?}", result.error);
}
```

### VerificationResult

Result of verification.

```rust
pub struct VerificationResult {
    pub valid: bool,
    pub error: Option<String>,
}
```

### ProofType

Detected proof type enumeration.

```rust
pub enum ProofType {
    Empty,
    Mock,
    Sp1Compressed,
    Sp1Plonk,
    Unknown,
}
```

---

## Complete Example

Here's a complete example using all components:

```rust
use neo_vm_core::{NeoVM, VMState, StackItem};
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProverConfig, ProveMode};
use neo_zkvm_verifier::{verify, verify_detailed};

fn main() {
    // 1. Create and test script locally
    let script = vec![0x12, 0x13, 0x9E, 0x40]; // 2 + 3
    
    let mut vm = NeoVM::new(1_000_000);
    vm.load_script(script.clone());
    
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        vm.execute_next().unwrap();
    }
    
    println!("Local result: {:?}", vm.eval_stack);
    
    // 2. Generate proof
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    
    let prover = NeoProver::new(ProverConfig {
        prove_mode: ProveMode::Mock,
        ..Default::default()
    });
    
    let proof = prover.prove(input);
    
    // 3. Verify proof
    let result = verify_detailed(&proof);
    
    println!("Proof valid: {}", result.valid);
    println!("Gas consumed: {}", proof.public_inputs.gas_consumed);
}
```
```
```
