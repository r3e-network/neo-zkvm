# neo-zkvm-examples Source-Level Learning Guide

This guide is generated from the crate's actual `Cargo.toml`, Rust source files, public symbols, and test functions. It is meant to help a reader understand what this crate owns before reading implementation details.

## What This Crate Is

| Topic | Detail |
| --- | --- |
| Layer | Neo zkVM stack |
| Purpose | Runnable examples that demonstrate common proof flows and application patterns. |
| Inputs | sample script, example input, local prover |
| Responsibilities | Demonstrate APIs, Exercise edge cases, Document expected outputs |
| Outputs | example proof, tutorial output, regression sample |
| Consumers | zkVM users, L2 prover service, L1 verification adapter |

## Visual Reading Order

| Step | Diagram | Use it to learn |
| ---: | --- | --- |
| 1 | [Position](figures/position.svg) | Why this crate exists and where it sits in Neo N4. |
| 2 | [Principles](figures/principles.svg) | The invariants and boundaries this crate must protect. |
| 3 | [Module map](figures/module-map.svg) | Which files are the best entry points. |
| 4 | [Public API surface](figures/api-surface.svg) | Which exported symbols form the crate contract. |
| 5 | [Architecture](figures/architecture.svg) | How inputs, internal components, dependencies, and outputs connect. |
| 6 | [Workflow](figures/workflow.svg) | The normal execution path. |
| 7 | [Dataflow](figures/dataflow.svg) | How data is transformed across the crate boundary. |
| 8 | [Test evidence](figures/test-map.svg) | Which tests protect the behavior. |
| 9 | [Dependency map](figures/dependency-map.svg) | Which dependencies are runtime, test, or build-only. |

## Source File Map

| File | Role | Public symbols | Tests |
| --- | --- | ---: | ---: |
| `src/batch_verification.rs` | implementation detail or helper module | 0 | 2 |
| `src/private_inputs.rs` | implementation detail or helper module | 0 | 2 |
| `src/tamper_resistance.rs` | implementation detail or helper module | 0 | 2 |
| `src/basic.rs` | implementation detail or helper module | 0 | 0 |
| `src/proof_generation.rs` | proof object, layout, and verification evidence | 0 | 0 |
| `src/zk_dao_voting.rs` | implementation detail or helper module | 0 | 0 |
| `src/zk_dex_rollup.rs` | implementation detail or helper module | 0 | 0 |
| `src/zk_preimage.rs` | implementation detail or helper module | 0 | 0 |
| `src/zk_scaling.rs` | implementation detail or helper module | 0 | 0 |

## Public API Surface

No public Rust symbols were scanned.

## Module and Re-Export Signals

No `mod` or `pub use` declarations were scanned.

## Test Evidence

| Test | File |
| --- | --- |
| `test_batch_verification_accepts_valid_proofs` | `src/batch_verification.rs` |
| `test_batch_verification_rejects_tampered_proof` | `src/batch_verification.rs` |
| `test_private_input_proof_verifies_and_returns_square` | `src/private_inputs.rs` |
| `test_different_private_inputs_produce_different_input_hashes` | `src/private_inputs.rs` |
| `test_output_tampering_is_rejected` | `src/tamper_resistance.rs` |
| `test_proof_format_tampering_is_rejected` | `src/tamper_resistance.rs` |

## Dependency Boundary

| Dependency | Kind |
| --- | --- |
| `neo-vm-guest` | runtime |
| `neo-zkvm-prover` | runtime |
| `neo-zkvm-verifier` | runtime |

## Suggested Reading Path

1. Read `src/batch_verification.rs`: implementation detail or helper module.
2. Read `src/private_inputs.rs`: implementation detail or helper module.
3. Read `src/tamper_resistance.rs`: implementation detail or helper module.
4. Read `src/basic.rs`: implementation detail or helper module.
5. Read `src/proof_generation.rs`: proof object, layout, and verification evidence.
6. Read `src/zk_dao_voting.rs`: implementation detail or helper module.

## Change Safety Checklist

- Keep the stated responsibility boundary intact: Demonstrate APIs, Exercise edge cases, Document expected outputs.
- Update the workflow and dataflow diagrams when adding or removing major execution steps.
- Add or update tests in the files listed under Test Evidence when public API or state-transition behavior changes.
- Re-run `python tools/docs/generate_crate_visual_docs.py` from the Neo N4 repository root after source layout changes.
