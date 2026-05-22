# neo-zkvm-program Source-Level Learning Guide

This guide is generated from the crate's actual `Cargo.toml`, Rust source files, public symbols, and test functions. It is meant to help a reader understand what this crate owns before reading implementation details.

## What This Crate Is

| Topic | Detail |
| --- | --- |
| Layer | Neo zkVM stack |
| Purpose | SP1 guest binary entrypoint that binds proof inputs to deterministic NeoVM execution. |
| Inputs | SP1 stdin, Neo proof input, guest facade |
| Responsibilities | Deserialize stdin, Execute script, Commit public values |
| Outputs | SP1 public values, execution output, fault status |
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
| `src/main.rs` | binary or CLI entrypoint | 1 | 4 |

## Public API Surface

| Symbol | File |
| --- | --- |
| `fn zkvm_main` | `src/main.rs` |

## Module and Re-Export Signals

No `mod` or `pub use` declarations were scanned.

## Test Evidence

| Test | File |
| --- | --- |
| `test_basic_execution` | `src/main.rs` |
| `test_arithmetic` | `src/main.rs` |
| `test_hash_with_bincode_limit_matches_serialized_hash_on_success` | `src/main.rs` |
| `test_hash_with_bincode_limit_uses_error_marker_on_serialize_failure` | `src/main.rs` |

## Dependency Boundary

| Dependency | Kind |
| --- | --- |
| `bincode` | runtime |
| `neo-vm-guest` | runtime |
| `serde` | runtime |
| `sha2` | runtime |
| `sp1-zkvm` | runtime |
| `sp1-build` | build |

## Suggested Reading Path

1. Read `src/main.rs`: binary or CLI entrypoint.

## Change Safety Checklist

- Keep the stated responsibility boundary intact: Deserialize stdin, Execute script, Commit public values.
- Update the workflow and dataflow diagrams when adding or removing major execution steps.
- Add or update tests in the files listed under Test Evidence when public API or state-transition behavior changes.
- Re-run `python tools/docs/generate_crate_visual_docs.py` from the Neo N4 repository root after source layout changes.
