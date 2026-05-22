# neo-zkvm-prover Source-Level Learning Guide

This guide is generated from the crate's actual `Cargo.toml`, Rust source files, public symbols, and test functions. It is meant to help a reader understand what this crate owns before reading implementation details.

## What This Crate Is

| Topic | Detail |
| --- | --- |
| Layer | Neo zkVM stack |
| Purpose | Proof generation library that turns NeoVM execution inputs into verifiable proof artifacts. |
| Inputs | proof input, guest program, prover mode |
| Responsibilities | Hash inputs, Run guest execution, Build proof envelope |
| Outputs | NeoProof, public output, prover report |
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
| 10 | [Implementation atlas](figures/implementation-atlas.svg) | A dense one-page map of purpose, source entrypoints, API, workflow, dataflow, dependencies, tests, and change checks. |

## Source File Map

| File | Role | Public symbols | Tests |
| --- | --- | ---: | ---: |
| `src/lib.rs` | crate root, public exports, and top-level documentation | 9 | 23 |
| `src/elf_markers.rs` | implementation detail or helper module | 4 | 0 |
| `build.rs` | implementation detail or helper module | 0 | 0 |

## Public API Surface

| Symbol | File |
| --- | --- |
| `const DUMMY_ELF_NO_PROGRAM_SOURCE` | `src/elf_markers.rs` |
| `const DUMMY_ELF_NOT_FOR_PRODUCTION` | `src/elf_markers.rs` |
| `const DUMMY_ELF_FOR_CLIPPY` | `src/elf_markers.rs` |
| `const DUMMY_ELF_BUILD_FAILED` | `src/elf_markers.rs` |
| `const NEO_ZKVM_ELF` | `src/lib.rs` |
| `struct ProverConfig` | `src/lib.rs` |
| `struct NeoProver` | `src/lib.rs` |
| `fn is_elf_available` | `src/lib.rs` |
| `fn new` | `src/lib.rs` |
| `fn hash_proof_input` | `src/lib.rs` |
| `fn prove` | `src/lib.rs` |
| `fn prove_strict` | `src/lib.rs` |
| `fn verify` | `src/lib.rs` |

## Module and Re-Export Signals

| Signal |
| --- |
| `build.rs: mod elf_markers` |
| `src/lib.rs: mod elf_markers` |
| `src/lib.rs: pub use neo_vm_guest::{MockProof, NeoProof, ProofMode, PublicInputs, PROOF_FORMAT_VERSION}` |

## Test Evidence

| Test | File |
| --- | --- |
| `test_mock_proof` | `src/lib.rs` |
| `test_execute_only` | `src/lib.rs` |
| `test_prove_strict_allows_execute_mode` | `src/lib.rs` |
| `test_prove_strict_rejects_crypto_fallback` | `src/lib.rs` |
| `test_prove_sp1_mode_fails_closed_in_debug_without_fallback_opt_in` | `src/lib.rs` |
| `test_prove_sp1_mode_supports_explicit_fallback_opt_in` | `src/lib.rs` |
| `test_input_hash_matches_serialized_proof_input` | `src/lib.rs` |
| `test_input_hash_distinguishes_complex_argument_variants` | `src/lib.rs` |
| `test_hash_proof_input_rejects_oversized_arguments` | `src/lib.rs` |
| `test_prove_rejects_unhashable_input_arguments` | `src/lib.rs` |
| `test_mock_proof_tamper_detection` | `src/lib.rs` |
| `test_proof_output_contains_result` | `src/lib.rs` |
| `test_output_tampering_is_detected` | `src/lib.rs` |
| `test_faulting_script_produces_failed_proof` | `src/lib.rs` |
| `test_verify_execute_faulted_proof_rejected` | `src/lib.rs` |
| `test_verify_mock_faulted_proof_rejected` | `src/lib.rs` |
| `test_verify_rejects_unknown_proof_format_version` | `src/lib.rs` |
| `test_max_cycles_caps_effective_gas_limit` | `src/lib.rs` |
| `test_deterministic_mock_proof_timestamp_produces_stable_bytes` | `src/lib.rs` |
| `test_failed_proof_does_not_panic_on_oversized_error_message` | `src/lib.rs` |
| `test_sp1_unavailable_reason_reports_build_failed_marker` | `src/lib.rs` |
| `test_sp1_unavailable_reason_reports_toolchain_marker` | `src/lib.rs` |
| `test_hash_proof_input_does_not_panic_on_oversized_arguments` | `src/lib.rs` |

## Dependency Boundary

| Dependency | Kind |
| --- | --- |
| `bincode` | runtime |
| `neo-vm-guest` | runtime |
| `sp1-sdk` | runtime |
| `sp1-build` | build |

## Suggested Reading Path

1. Read `src/lib.rs`: crate root, public exports, and top-level documentation.
2. Read `src/elf_markers.rs`: implementation detail or helper module.
3. Read `build.rs`: implementation detail or helper module.

## Change Safety Checklist

- Keep the stated responsibility boundary intact: Hash inputs, Run guest execution, Build proof envelope.
- Update the workflow and dataflow diagrams when adding or removing major execution steps.
- Add or update tests in the files listed under Test Evidence when public API or state-transition behavior changes.
- Re-run `python tools/docs/generate_crate_visual_docs.py` from the Neo N4 repository root after source layout changes.
