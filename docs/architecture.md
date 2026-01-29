# Neo zkVM Architecture

## Overview

Neo zkVM consists of four main components:

```
┌─────────────────┐     ┌─────────────────┐
│   neo-vm-core   │────▶│  neo-vm-guest   │
│   (VM Engine)   │     │ (Guest Program) │
└─────────────────┘     └─────────────────┘
         │                      │
         ▼                      ▼
┌─────────────────┐     ┌─────────────────┐
│neo-zkvm-prover  │────▶│neo-zkvm-verifier│
│ (Proof Gen)     │     │ (Proof Verify)  │
└─────────────────┘     └─────────────────┘
```

## Components

### neo-vm-core
Core VM engine with Neo N3 opcode support.

### neo-vm-guest  
Guest program wrapper for zkVM proving.

### neo-zkvm-prover
Proof generation using SP1 framework.

### neo-zkvm-verifier
Proof verification.

## Execution Flow

1. Load script into VM
2. Execute with tracing enabled
3. Generate execution trace
4. Create ZK proof from trace
5. Verify proof
