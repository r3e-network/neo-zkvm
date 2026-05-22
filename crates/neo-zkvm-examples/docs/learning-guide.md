# neo-zkvm-examples Technical Learning Guide

This guide explains `neo-zkvm-examples` as a Neo N4 technical unit. It is written for architecture learning: what the unit is responsible for, which assumptions make it correct, how data moves, how state changes, how evidence is checked, and where it plugs into the wider Neo N4 stack.

## Technical Contract

| Aspect | Meaning |
| --- | --- |
| Layer | Neo zkVM stack |
| Purpose | Runnable examples that demonstrate common proof flows and application patterns. |
| Inputs | sample script <br> example input <br> local prover |
| Responsibilities | Demonstrate APIs <br> Exercise edge cases <br> Document expected outputs |
| Outputs | example proof <br> tutorial output <br> regression sample |
| Consumers | zkVM users <br> L2 prover service <br> L1 verification adapter |

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

`neo-zkvm-examples` receives sample script | example input | local prover and owns this boundary: Demonstrate APIs | Exercise edge cases | Document expected outputs. It emits example proof | tutorial output | regression sample, which are consumed by zkVM users | L2 prover service | L1 verification adapter.

Layering rule: guest proves computation, host orchestrates, L1 verifies compact results.

## Workflow

1. Prepare input
2. Run guest/host logic
3. Generate or check proof
4. Record evidence

Failure path: proving fails, local verification fails, public output mismatches, or verifier rejects.

## Data Flow

1. execution input
2. neo-zkvm-examples
3. proof/evidence artifact
4. Neo N4 verification flow

Commitment signal: state root, public values, verification key, and proof digest.

## State, Proof, and Trust

- State transition: guest execution is constrained by public values and verifier rules.
- Finality: verifier accepts proof and public output matches target state.
- Trust model: trust verification keys and verifiers, not prover runtime environments.
- Validation boundary: public input, proof envelope, verification key, and public output must match.
- Replay and ordering: proof binds batch range and state root to prevent cross-batch reuse.

## Integration and Operation

- NeoFS DA: NeoFS stores batch data, witness or trace summaries, and retrievable evidence.
- Proof system: The proof system compresses L2 execution claims into verifiable evidence.
- Gateway/API: Gateway handles user routing, queries, submission, and health aggregation.
- Bridge and heterogeneous chains: Bridge rules unify L1-L2, L2-L2, and heterogeneous-chain messages and assets.
- Observable evidence: proof id, public output, verification result, duration, and failure reason.

Regenerate these technical diagrams from the Neo N4 repository root with:

```powershell
python tools/docs/generate_crate_visual_docs.py
```
