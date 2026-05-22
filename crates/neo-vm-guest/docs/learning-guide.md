# neo-vm-guest Technical Learning Guide

This guide explains `neo-vm-guest` as a Neo N4 technical unit. It is written for architecture learning: what the unit is responsible for, which assumptions make it correct, how data moves, how state changes, how evidence is checked, and where it plugs into the wider Neo N4 stack.

## Technical Contract

| Aspect | Meaning |
| --- | --- |
| Layer | zkVM guest facade |
| Purpose | Guest-facing adapter that exposes shared NeoVM execution APIs in zkVM-compatible form. |
| Inputs | guest bytecode <br> stack input <br> shared VM crate |
| Responsibilities | Call shared VM <br> Keep guest ABI small <br> Return deterministic result |
| Outputs | guest execution result <br> public output seed <br> fault reason |
| Consumers | neo-zkvm-program <br> neo-zkvm-prover <br> examples |

## Diagram Set

| # | Diagram | What to learn |
| --- | --- | --- |
| 1 | [System Position](figures/position.svg) | where this crate sits in Neo N4. |
| 2 | [Technical Principles](figures/principles.svg) | the rules that make the design correct. |
| 3 | [Conceptual Architecture](figures/architecture.svg) | major technical blocks and boundaries. |
| 4 | [Workflow](figures/workflow.svg) | the ordered runtime process. |
| 5 | [Data Flow](figures/dataflow.svg) | how information, commitments, and evidence move. |
| 6 | [State Model](figures/state-model.svg) | state ownership, transitions, and finality. |
| 7 | [Proof and Evidence Flow](figures/proof-flow.svg) | how claims become verifiable evidence. |
| 8 | [Trust Boundaries](figures/trust-boundaries.svg) | what is trusted, checked, rejected, or observed. |
| 9 | [Integration Map](figures/integration-map.svg) | how this unit connects to the wider N4 stack. |
| 10 | [Runtime Lifecycle](figures/lifecycle.svg) | from configuration through execution, evidence, and operation. |

## Architecture Model

`neo-vm-guest` receives guest bytecode | stack input | shared VM crate and owns this boundary: Call shared VM | Keep guest ABI small | Return deterministic result. It emits guest execution result | public output seed | fault reason, which are consumed by neo-zkvm-program | neo-zkvm-prover | examples.

Layering rule: VM semantics stay separate from host context and chain policy.

## Workflow

1. Receive guest input
2. Invoke shared VM
3. Collect output
4. Commit result

Failure path: invalid opcode, stack mismatch, gas exhaustion, syscall rejection, or fault.

## Data Flow

1. script + args
2. guest facade
3. VM result
4. proof public values

Commitment signal: script hash, final stack digest, halt/fault status, and gas.

## State, Proof, and Trust

- State transition: opcode semantics, stack rules, gas, and syscall contracts define transitions.
- Finality: VM halts successfully and host accepts final stack and effects.
- Trust model: trust canonical NeoVM semantics, not script authors.
- Validation boundary: opcode, stack types, gas, jump target, and syscall response must be valid.
- Replay and ordering: VM context binds script and host state.

## Integration and Operation

- NeoFS DA: NeoFS stores batch data, witness or trace summaries, and retrievable evidence.
- Proof system: The proof system compresses L2 execution claims into verifiable evidence.
- Gateway/API: Gateway handles user routing, queries, submission, and health aggregation.
- Bridge and heterogeneous chains: Bridge rules unify L1-L2, L2-L2, and heterogeneous-chain messages and assets.
- Observable evidence: opcode progress, gas, stack digest, fault reason, and syscall boundary.

Regenerate these technical diagrams from the Neo N4 repository root with:

```powershell
python tools/docs/generate_crate_visual_docs.py
```
