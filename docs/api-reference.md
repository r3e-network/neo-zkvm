# Neo zkVM API Reference

This reference documents the public API surface that remains after consolidation on `neo-vm-rs`.

## `neo-vm-guest`

```rust
use neo_vm_guest::{execute, ProofInput, StackItem};

let output = execute(ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40],
    arguments: vec![],
    gas_limit: 1_000_000,
});

assert_eq!(output.result, Some(StackItem::Integer(5)));
```

### Key Exports

| Item | Purpose |
| --- | --- |
| `ProofInput` | Script, argument stack, and gas limit supplied to the guest. |
| `ProofOutput` | VM state, top result, gas consumed, and fault message. |
| `StackItem` | Canonical stack value type re-exported from `neo-vm-rs`. |
| `execute` | Deterministic execution through the shared interpreter. |
| `bincode_serialize` / `bincode_deserialize` | Canonical serialization helpers for proof data. |
| `try_hash_proof_output` | Fallible hash of serialized output for public input binding. |
| `hash_data` | Domain-separated double-SHA256 (`neo-zkvm-data-hash-v1:`), **not** Neo protocol Hash256. |
| `hash256` | True Neo Hash256: `SHA256(SHA256(x))` (crypto helpers). |

## `neo-zkvm-prover`

```rust
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};

let prover = NeoProver::new(ProverConfig {
    proof_mode: ProofMode::Mock,
    ..Default::default()
});

let proof = prover.prove(ProofInput {
    script: vec![0x12, 0x13, 0x9E, 0x40],
    arguments: vec![],
    gas_limit: 1_000_000,
});
```

### Proof Modes

| Mode | Behavior |
| --- | --- |
| `Execute` | Runs the shared VM and produces no proof bytes. |
| `Mock` | Produces a deterministic test proof envelope. |
| `Sp1` | Uses SP1 compressed proof generation. |
| `Plonk` | Uses SP1 PLONK proof generation. |
| `Groth16` | Uses SP1 Groth16 proof generation. |

## `neo-zkvm-verifier`

```rust
use neo_zkvm_prover::ProofMode;
use neo_zkvm_verifier::verify_for_mode;

// Pin a verifier-chosen constant — NEVER pass proof.proof_mode here.
// That would make the mode check a tautology and accept forgeable Mock proofs.
// Demo / tests:
assert!(verify_for_mode(&proof, ProofMode::Mock));
// Production value-bearing decisions:
// assert!(verify_for_mode(&proof, ProofMode::Groth16));
```

The verifier checks mode consistency, public input hashes, verification key binding, and proof bytes for real proof modes. See `SECURITY.md` for the full trust model.

## `neo-zkvm-cli`

```bash
neo-zkvm run 12139E40
neo-zkvm prove 12139E40 -m mock
neo-zkvm verify proof.bin -m mock
neo-zkvm asm script.neoasm
neo-zkvm disasm 12139E40
neo-zkvm debug 12139E40
neo-zkvm inspect 12139E40
```

`debug` is an execution trace command backed by `neo-vm-rs`; it is not a separate VM debugger.