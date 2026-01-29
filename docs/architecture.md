# Neo zkVM Architecture

## Overview

Neo zkVM is a zero-knowledge virtual machine that enables verifiable computation of Neo N3 smart contracts. It combines the Neo VM execution engine with SP1's zero-knowledge proof system to generate cryptographic proofs of correct execution.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Neo zkVM System                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐  │
│  │   Script    │───▶│  neo-vm-    │───▶│    Execution Trace      │  │
│  │   Input     │    │    core     │    │  (State Transitions)    │  │
│  └─────────────┘    └─────────────┘    └───────────┬─────────────┘  │
│                                                     │                │
│                                                     ▼                │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐  │
│  │   Proof     │◀───│neo-zkvm-    │◀───│    neo-vm-guest         │  │
│  │   Output    │    │   prover    │    │  (SP1 Guest Program)    │  │
│  └─────────────┘    └─────────────┘    └─────────────────────────┘  │
│         │                                                            │
│         ▼                                                            │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    neo-zkvm-verifier                         │    │
│  │              (Proof Verification Layer)                      │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. neo-vm-core

The core VM engine implementing the Neo N3 virtual machine specification.

**Key Features:**
- Full Neo N3 opcode support (100+ opcodes)
- Gas metering and limits
- Execution tracing for proof generation
- Cryptographic operations (SHA256, RIPEMD160, ECDSA)
- Storage backend abstraction
- Native contract support

**Module Structure:**
```
neo-vm-core/
├── src/
│   ├── lib.rs          # Public API exports
│   ├── engine.rs       # VM execution engine
│   ├── opcode.rs       # Opcode definitions
│   ├── stack_item.rs   # Stack item types
│   ├── storage.rs      # Storage backends
│   └── native.rs       # Native contracts
```

**Key Types:**
- `NeoVM` - Main VM instance
- `VMState` - Execution state (None, Halt, Fault, Break)
- `StackItem` - Stack value types
- `ExecutionContext` - Script execution context
- `ExecutionTrace` - Proof generation trace

### 2. neo-vm-guest

Guest program wrapper for zkVM proving. This crate provides the interface between the Neo VM and the SP1 proving system.

**Responsibilities:**
- Serialize/deserialize proof inputs
- Execute Neo VM within SP1 guest environment
- Produce deterministic outputs for verification

**Key Types:**
```rust
pub struct ProofInput {
    pub script: Vec<u8>,
    pub arguments: Vec<StackItem>,
    pub gas_limit: u64,
}

pub struct ProofOutput {
    pub result: Vec<StackItem>,
    pub gas_consumed: u64,
    pub state: u8,  // 0 = Halt, 1 = Fault
}
```

### 3. neo-zkvm-prover

Production-grade prover using SP1 framework for generating zero-knowledge proofs.

**Proving Modes:**
| Mode | Description | Use Case |
|------|-------------|----------|
| `Execute` | Run only, no proof | Development/testing |
| `Mock` | Simulated proof | Fast testing |
| `Sp1` | Compressed SP1 proof | Off-chain verification |
| `Sp1Plonk` | PLONK proof | On-chain verification |

**Key Types:**
```rust
pub struct NeoProof {
    pub output: ProofOutput,
    pub proof_bytes: Vec<u8>,
    pub public_inputs: PublicInputs,
    pub vkey_hash: [u8; 32],
}

pub struct PublicInputs {
    pub script_hash: [u8; 32],
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub gas_consumed: u64,
    pub execution_success: bool,
}
```

### 4. neo-zkvm-verifier

Proof verification layer supporting multiple proof types.

**Supported Proof Types:**
- Empty (execute-only mode)
- Mock (testing)
- SP1 Compressed
- SP1 PLONK

### 5. neo-zkvm-cli

Command-line interface for development and testing.

**Commands:**
- `run` - Execute scripts
- `prove` - Generate proofs
- `asm` - Assemble source code
- `disasm` - Disassemble bytecode

### 6. neo-zkvm-program

SP1 guest program that runs inside the zkVM. This is the actual program that gets proven.

## Data Flow

### Execution Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                      EXECUTION PIPELINE                           │
└──────────────────────────────────────────────────────────────────┘

     ┌─────────┐
     │ Script  │ (Neo bytecode)
     │ + Args  │
     └────┬────┘
          │
          ▼
    ┌───────────┐
    │  DECODE   │ Read opcode from script
    └─────┬─────┘
          │
          ▼
    ┌───────────┐
    │ GAS CHECK │ Verify sufficient gas
    └─────┬─────┘
          │
          ▼
    ┌───────────┐
    │  EXECUTE  │ Perform operation
    └─────┬─────┘
          │
          ▼
    ┌───────────┐
    │   TRACE   │ Record state transition
    └─────┬─────┘
          │
          ▼
    ┌───────────┐     ┌─────────┐
    │   DONE?   │─No─▶│  LOOP   │
    └─────┬─────┘     └────┬────┘
          │                │
         Yes               │
          │◀───────────────┘
          ▼
    ┌───────────┐
    │  OUTPUT   │ Final state + trace
    └───────────┘
```

### Proof Generation Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                    PROOF GENERATION PIPELINE                      │
└──────────────────────────────────────────────────────────────────┘

  ProofInput                    SP1 Guest                    NeoProof
 ┌──────────┐              ┌───────────────┐              ┌──────────┐
 │ script   │              │               │              │ output   │
 │ args     │─────────────▶│  neo-vm-guest │─────────────▶│ proof    │
 │ gas_limit│              │  execution    │              │ pub_ins  │
 └──────────┘              └───────────────┘              └──────────┘
                                  │
                                  ▼
                           ┌───────────────┐
                           │  SP1 Prover   │
                           │  (STARK/PLONK)│
                           └───────────────┘
                                  │
                                  ▼
                           ┌───────────────┐
                           │ Proof Bytes   │
                           │ + VKey Hash   │
                           └───────────────┘
```

### Verification Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                     VERIFICATION PIPELINE                         │
└──────────────────────────────────────────────────────────────────┘

    NeoProof
   ┌─────────┐
   │         │
   └────┬────┘
        │
        ▼
  ┌───────────┐
  │  DETECT   │ Identify proof type
  │   TYPE    │
  └─────┬─────┘
        │
        ├──────────────┬──────────────┬──────────────┐
        ▼              ▼              ▼              ▼
   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐
   │  Empty  │   │  Mock   │   │   SP1   │   │  PLONK  │
   │ (skip)  │   │ Verify  │   │ Verify  │   │ Verify  │
   └────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘
        │             │             │             │
        └──────────────┴──────────────┴──────────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │ VerificationResult│
                    │ { valid, error }  │
                    └───────────────────┘
```

## Component Interactions

### VM Engine Internals

```
┌─────────────────────────────────────────────────────────────────┐
│                         NeoVM Instance                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐    ┌─────────────────┐                     │
│  │  Eval Stack     │    │ Invocation Stack│                     │
│  │  ┌───┬───┬───┐  │    │  ┌───────────┐  │                     │
│  │  │ 5 │ 3 │...│  │    │  │ Context 0 │  │                     │
│  │  └───┴───┴───┘  │    │  ├───────────┤  │                     │
│  └─────────────────┘    │  │ Context 1 │  │                     │
│                         │  └───────────┘  │                     │
│  ┌─────────────────┐    └─────────────────┘                     │
│  │   Slot System   │                                            │
│  │ ┌─────┬─────┐   │    ┌─────────────────┐                     │
│  │ │Local│Args │   │    │   Gas Meter     │                     │
│  │ └─────┴─────┘   │    │ consumed: 1234  │                     │
│  └─────────────────┘    │ limit: 1000000  │                     │
│                         └─────────────────┘                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Execution Trace                       │   │
│  │  Step 0: ip=0, op=0x12, gas=1                           │   │
│  │  Step 1: ip=1, op=0x13, gas=2                           │   │
│  │  Step 2: ip=2, op=0x9E, gas=10                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Storage Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Storage System                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  StorageBackend Trait                    │   │
│  │  + get(key) -> Option<Vec<u8>>                          │   │
│  │  + put(key, value)                                       │   │
│  │  + delete(key)                                           │   │
│  │  + contains(key) -> bool                                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                   │
│              ┌───────────────┼───────────────┐                  │
│              ▼               ▼               ▼                  │
│     ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│     │MemoryStorage│  │TrackedStorage│  │  Custom    │          │
│     │ (HashMap)   │  │ (+ Changes) │  │  Backend   │          │
│     └─────────────┘  └─────────────┘  └─────────────┘          │
│                              │                                   │
│                              ▼                                   │
│                    ┌─────────────────┐                          │
│                    │  Merkle Root    │                          │
│                    │  Computation    │                          │
│                    └─────────────────┘                          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Native Contract System

```
┌─────────────────────────────────────────────────────────────────┐
│                   Native Contract Registry                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   NativeRegistry                         │   │
│  │  + register(contract)                                    │   │
│  │  + invoke(hash, method, args) -> Result                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                   │
│              ┌───────────────┴───────────────┐                  │
│              ▼                               ▼                  │
│     ┌─────────────────┐            ┌─────────────────┐         │
│     │     StdLib      │            │    CryptoLib    │         │
│     ├─────────────────┤            ├─────────────────┤         │
│     │ • serialize     │            │ • sha256        │         │
│     │ • deserialize   │            │ • ripemd160     │         │
│     │ • base64_encode │            │ • hash160       │         │
│     │ • base64_decode │            │ • verify_sig    │         │
│     │ • itoa / atoi   │            │ • murmur32      │         │
│     └─────────────────┘            └─────────────────┘         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Security Model

### Trust Assumptions

1. **SP1 Proving System**: We trust the SP1 zkVM implementation for proof generation and verification
2. **Cryptographic Primitives**: Standard assumptions on SHA256, RIPEMD160, and ECDSA
3. **Deterministic Execution**: The Neo VM must execute deterministically for valid proofs

### Verification Guarantees

| Property | Guarantee |
|----------|-----------|
| **Soundness** | Invalid executions cannot produce valid proofs |
| **Completeness** | Valid executions always produce verifiable proofs |
| **Zero-Knowledge** | Proofs reveal nothing beyond public inputs |

### Gas Metering

Gas prevents denial-of-service attacks and ensures bounded execution:

```
┌─────────────────────────────────────────────────────────────────┐
│                      Gas Cost Categories                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Low Cost (1-2 gas):                                            │
│    • Push operations (PUSH0-PUSH16)                             │
│    • Stack operations (DUP, DROP, SWAP)                         │
│    • Control flow (NOP, JMP, RET)                               │
│                                                                  │
│  Medium Cost (8 gas):                                           │
│    • Arithmetic (ADD, SUB, MUL, DIV)                            │
│    • Bitwise operations (AND, OR, XOR)                          │
│    • Comparisons (LT, GT, EQ)                                   │
│                                                                  │
│  High Cost (512 gas):                                           │
│    • Hash operations (SHA256, RIPEMD160)                        │
│                                                                  │
│  Very High Cost (32768 gas):                                    │
│    • Signature verification (CHECKSIG)                          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Performance Considerations

### Proof Generation Time

| Script Complexity | Approximate Time |
|-------------------|------------------|
| Simple (< 100 ops) | < 1 second |
| Medium (100-1000 ops) | 1-10 seconds |
| Complex (1000+ ops) | 10+ seconds |

### Proof Size

| Proof Type | Approximate Size |
|------------|------------------|
| Mock | ~200 bytes |
| SP1 Compressed | ~100 KB |
| SP1 PLONK | ~1 KB |

## Future Enhancements

1. **Parallel Proving**: Split large scripts for parallel proof generation
2. **Recursive Proofs**: Compose proofs for complex workflows
3. **On-chain Verifier**: Solidity contract for Ethereum verification
4. **Hardware Acceleration**: GPU/FPGA support for faster proving
