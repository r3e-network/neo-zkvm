# neo-vm-guest Source-Level Learning Guide

This guide is generated from the crate's actual `Cargo.toml`, Rust source files, public symbols, and test functions. It is meant to help a reader understand what this crate owns before reading implementation details.

## What This Crate Is

| Topic | Detail |
| --- | --- |
| Layer | zkVM guest facade |
| Purpose | Guest-facing adapter that exposes shared NeoVM execution APIs in zkVM-compatible form. |
| Inputs | guest bytecode, stack input, shared VM crate |
| Responsibilities | Call shared VM, Keep guest ABI small, Return deterministic result |
| Outputs | guest execution result, public output seed, fault reason |
| Consumers | neo-zkvm-program, neo-zkvm-prover, examples |

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
| 10 | [Implementation atlas](figures/implementation-atlas.svg) | A dense one-page map of purpose, source entrypoints, API, workflow, dataflow, dependencies, tests, and change checks. |

## Source File Map

| File | Role | Public symbols | Tests |
| --- | --- | ---: | ---: |
| `src/lib.rs` | crate root, public exports, and top-level documentation | 21 | 7 |
| `tests/shared_vm.rs` | external behavior or integration test | 0 | 5 |

## Public API Surface

| Symbol | File |
| --- | --- |
| `type BincodeEncodeError` | `src/lib.rs` |
| `type BincodeDecodeError` | `src/lib.rs` |
| `const PROOF_FORMAT_VERSION` | `src/lib.rs` |
| `const PROOF_MAX_SCRIPT_SIZE` | `src/lib.rs` |
| `fn bincode_options` | `src/lib.rs` |
| `fn bincode_serialize` | `src/lib.rs` |
| `fn bincode_deserialize` | `src/lib.rs` |
| `struct ProofInput` | `src/lib.rs` |
| `struct ProofOutput` | `src/lib.rs` |
| `enum ProofMode` | `src/lib.rs` |
| `struct PublicInputs` | `src/lib.rs` |
| `struct NeoProof` | `src/lib.rs` |
| `struct MockProof` | `src/lib.rs` |
| `fn hash_data` | `src/lib.rs` |
| `fn try_hash_proof_output` | `src/lib.rs` |
| `fn hash_proof_output` | `src/lib.rs` |
| `fn compute_commitment` | `src/lib.rs` |
| `fn public_inputs_equal` | `src/lib.rs` |
| `fn output_matches_public_inputs` | `src/lib.rs` |
| `fn deserialize_neoproof` | `src/lib.rs` |
| `fn execute` | `src/lib.rs` |

## Module and Re-Export Signals

| Signal |
| --- |
| `src/lib.rs: pub use neo_vm_rs::{interop_hash, pop_byte_arg, StackValue as StackItem}` |

## Test Evidence

| Test | File |
| --- | --- |
| `test_output_matches_public_inputs_roundtrip` | `src/lib.rs` |
| `test_output_mismatch_detected` | `src/lib.rs` |
| `test_compute_commitment_changes_when_inputs_change` | `src/lib.rs` |
| `test_execute_reports_stack_overflow_for_excessive_arguments` | `src/lib.rs` |
| `test_execute_reports_runtime_fault_error` | `src/lib.rs` |
| `test_try_hash_proof_output_rejects_oversized_output` | `src/lib.rs` |
| `test_legacy_neoproof_deserializes_with_default_format_version` | `src/lib.rs` |
| `proof_input_stack_item_is_the_shared_neo_vm_rs_type` | `tests/shared_vm.rs` |
| `proof_guest_does_not_depend_on_legacy_neo_vm_core` | `tests/shared_vm.rs` |
| `proof_guest_uses_shared_byte_arg_helper` | `tests/shared_vm.rs` |
| `proof_execution_rejects_non_canonical_throwifnot_byte` | `tests/shared_vm.rs` |
| `deterministic_crypto_uses_syscall_not_pseudo_opcode` | `tests/shared_vm.rs` |

## Dependency Boundary

| Dependency | Kind |
| --- | --- |
| `bincode` | runtime |
| `neo-vm-rs` | runtime |
| `serde` | runtime |
| `sha2` | runtime |

## Suggested Reading Path

1. Read `src/lib.rs`: crate root, public exports, and top-level documentation.
2. Read `tests/shared_vm.rs`: external behavior or integration test.

## Change Safety Checklist

- Keep the stated responsibility boundary intact: Call shared VM, Keep guest ABI small, Return deterministic result.
- Update the workflow and dataflow diagrams when adding or removing major execution steps.
- Add or update tests in the files listed under Test Evidence when public API or state-transition behavior changes.
- Re-run `python tools/docs/generate_crate_visual_docs.py` from the Neo N4 repository root after source layout changes.
