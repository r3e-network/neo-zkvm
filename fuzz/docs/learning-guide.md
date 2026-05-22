# neo-zkvm-fuzz Source-Level Learning Guide

This guide is generated from the crate's actual `Cargo.toml`, Rust source files, public symbols, and test functions. It is meant to help a reader understand what this crate owns before reading implementation details.

## What This Crate Is

| Topic | Detail |
| --- | --- |
| Layer | Neo zkVM stack |
| Purpose | Fuzzing workspace for adversarial proof and VM input exploration. |
| Inputs | random bytecode, mutated proof, seed corpus |
| Responsibilities | Generate inputs, Run no-panic checks, Capture regressions |
| Outputs | crash corpus, regression case, coverage signal |
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
| `fuzz_targets/common.rs` | fuzzing harness and adversarial input exploration | 1 | 0 |
| `fuzz_targets/fuzz_script_parser.rs` | fuzzing harness and adversarial input exploration | 0 | 0 |
| `fuzz_targets/fuzz_vm_execution.rs` | fuzzing harness and adversarial input exploration | 0 | 0 |

## Public API Surface

| Symbol | File |
| --- | --- |
| `fn append_bounded_neo_vm_sequence` | `fuzz_targets/common.rs` |

## Module and Re-Export Signals

| Signal |
| --- |
| `fuzz_targets/fuzz_script_parser.rs: mod common` |
| `fuzz_targets/fuzz_vm_execution.rs: mod common` |

## Test Evidence

No Rust `#[test]` functions were scanned in this crate.

## Dependency Boundary

| Dependency | Kind |
| --- | --- |
| `arbitrary` | runtime |
| `libfuzzer-sys` | runtime |
| `neo-vm-guest` | runtime |

## Suggested Reading Path

1. Read `fuzz_targets/common.rs`: fuzzing harness and adversarial input exploration.
2. Read `fuzz_targets/fuzz_script_parser.rs`: fuzzing harness and adversarial input exploration.
3. Read `fuzz_targets/fuzz_vm_execution.rs`: fuzzing harness and adversarial input exploration.

## Change Safety Checklist

- Keep the stated responsibility boundary intact: Generate inputs, Run no-panic checks, Capture regressions.
- Update the workflow and dataflow diagrams when adding or removing major execution steps.
- Add or update tests in the files listed under Test Evidence when public API or state-transition behavior changes.
- Re-run `python tools/docs/generate_crate_visual_docs.py` from the Neo N4 repository root after source layout changes.
