# Neo zkVM Technical Whitepaper

## Abstract

Neo zkVM proves Neo VM execution without maintaining a second VM implementation. The zkVM guest delegates execution semantics to `neo-vm-rs`, serializes deterministic proof input/output, and lets SP1-backed proving modes generate verifiable evidence for off-chain computation.

## Motivation

A zkVM is useful only if the proven execution matches the execution model used elsewhere in the platform. Duplicated VM engines create drift: an opcode bug fixed in one engine can remain in another. Neo zkVM avoids this by making `neo-vm-rs` the shared execution core consumed by zkVM and RISC-V VM tooling.

## Architecture

```text
Neo script -> ProofInput -> neo-vm-guest -> neo-vm-rs -> ProofOutput
                                      |                 |
                                      v                 v
                                public hashes       gas/result/state
                                      |
                                      v
                            SP1 / PLONK / Groth16 proof
                                      |
                                      v
                              verifier policy checks
```

## Execution Semantics

The authoritative VM behavior is outside this repository in `neo-vm-rs`. The guest crate re-exports `StackItem` from that shared crate and invokes the shared interpreter. This gives zkVM and RISC-V VM profiles the same stack values, opcode behavior, VM states, and syscall metadata.

## Proof Binding

A proof binds:

| Field | Purpose |
| --- | --- |
| `script_hash` | Commits to bytecode. |
| `input_hash` | Commits to serialized `ProofInput`. |
| `output_hash` | Commits to serialized `ProofOutput`. |
| `gas_consumed` | Prevents verifier-side ambiguity in resource accounting. |
| `execution_success` | Distinguishes successful halt from faulted execution. |
| `vkey_hash` | Binds proof bytes to the expected verifier key. |

## Proving Modes

`Execute` and `Mock` are development modes. `Sp1`, `Plonk`, and `Groth16` are production-oriented modes and require the SP1 toolchain plus a real guest ELF. Production commands should request the desired mode explicitly and should not allow fallback unless the operator is intentionally doing a degraded smoke test.

## Security Model

The prover may be untrusted. The verifier must validate proof mode, public input hashes, verification key binding, and proof bytes. Chain-state-dependent syscalls must not read hidden host state during proof execution; they need explicit inputs, commitments, or deterministic adapters.

## Non-Goals

Neo zkVM is not a native contract runtime, storage engine, or independent NeoVM implementation. Those responsibilities belong to the platform core and shared VM crates.