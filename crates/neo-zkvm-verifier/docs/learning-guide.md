# neo-zkvm-verifier Source-Level Learning Guide

This guide is generated from the crate's actual `Cargo.toml`, Rust source files, public symbols, and test functions. It is meant to help a reader understand what this crate owns before reading implementation details.

## What This Crate Is

| Topic | Detail |
| --- | --- |
| Layer | Neo zkVM stack |
| Purpose | Verifier library that checks proof envelopes, public outputs, and mode compatibility. |
| Inputs | proof envelope, verification key, expected public output |
| Responsibilities | Check proof format, Verify public values, Report validity |
| Outputs | verification result, error reason, audit evidence |
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
| `src/lib.rs` | crate root, public exports, and top-level documentation | 9 | 12 |

## Public API Surface

| Symbol | File |
| --- | --- |
| `struct VerificationResult` | `src/lib.rs` |
| `enum ProofType` | `src/lib.rs` |
| `fn verify` | `src/lib.rs` |
| `fn verify_for_mode` | `src/lib.rs` |
| `fn verify_detailed` | `src/lib.rs` |
| `fn verify_detailed_for_mode` | `src/lib.rs` |
| `fn verify_with_vkey` | `src/lib.rs` |
| `fn verify_with_vkey_for_mode` | `src/lib.rs` |
| `fn setup_elf` | `src/lib.rs` |

## Module and Re-Export Signals

No `mod` or `pub use` declarations were scanned.

## Test Evidence

| Test | File |
| --- | --- |
| `test_verify_mock_proof` | `src/lib.rs` |
| `test_verify_execute_only` | `src/lib.rs` |
| `test_verify_detailed` | `src/lib.rs` |
| `test_verify_tampered_mock_proof_fails` | `src/lib.rs` |
| `test_verify_rejects_mismatched_vkey_hash` | `src/lib.rs` |
| `test_verify_detailed_execute_mode` | `src/lib.rs` |
| `test_verify_faulting_execution_fails` | `src/lib.rs` |
| `test_verify_rejects_tampered_mock_output_result` | `src/lib.rs` |
| `test_verify_rejects_tampered_mock_output_gas` | `src/lib.rs` |
| `test_verify_rejects_unknown_proof_format_version` | `src/lib.rs` |
| `test_verify_for_mode_rejects_mode_mismatch` | `src/lib.rs` |
| `test_verify_for_mode_accepts_matching_mode` | `src/lib.rs` |

## Dependency Boundary

| Dependency | Kind |
| --- | --- |
| `bincode` | runtime |
| `neo-vm-guest` | runtime |
| `neo-zkvm-prover` | runtime |
| `sp1-sdk` | runtime |

## Suggested Reading Path

1. Read `src/lib.rs`: crate root, public exports, and top-level documentation.

## Change Safety Checklist

- Keep the stated responsibility boundary intact: Check proof format, Verify public values, Report validity.
- Update the workflow and dataflow diagrams when adding or removing major execution steps.
- Add or update tests in the files listed under Test Evidence when public API or state-transition behavior changes.
- Re-run `python tools/docs/generate_crate_visual_docs.py` from the Neo N4 repository root after source layout changes.
