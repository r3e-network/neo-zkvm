# neo-zkvm-cli Source-Level Learning Guide

This guide is generated from the crate's actual `Cargo.toml`, Rust source files, public symbols, and test functions. It is meant to help a reader understand what this crate owns before reading implementation details.

## What This Crate Is

| Topic | Detail |
| --- | --- |
| Layer | Neo zkVM stack |
| Purpose | CLI and developer tooling for assembling, inspecting, proving, and verifying Neo zkVM programs. |
| Inputs | CLI command, script/proof files, prover options |
| Responsibilities | Parse commands, Use shared opcode metadata, Run prove/verify workflows |
| Outputs | assembled script, proof report, inspection output |
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
| `src/main.rs` | binary or CLI entrypoint | 0 | 11 |
| `tests/integration_tests.rs` | external behavior or integration test | 0 | 30 |
| `src/assembler.rs` | developer assembly and script construction | 5 | 14 |
| `src/disassembler.rs` | developer assembly and script construction | 4 | 3 |
| `tests/source_layout.rs` | external behavior or integration test | 0 | 2 |

## Public API Surface

| Symbol | File |
| --- | --- |
| `enum AssemblerError` | `src/assembler.rs` |
| `struct Assembler` | `src/assembler.rs` |
| `fn new` | `src/assembler.rs` |
| `fn warnings` | `src/assembler.rs` |
| `fn assemble` | `src/assembler.rs` |
| `struct Disassembler` | `src/disassembler.rs` |
| `fn new` | `src/disassembler.rs` |
| `fn disassemble` | `src/disassembler.rs` |
| `fn decode_instruction` | `src/disassembler.rs` |

## Module and Re-Export Signals

| Signal |
| --- |
| `src/main.rs: mod assembler` |
| `src/main.rs: mod disassembler` |

## Test Evidence

| Test | File |
| --- | --- |
| `test_pushint8_out_of_range_returns_error` | `src/assembler.rs` |
| `test_push_sugar_uses_pushint64_for_large_values` | `src/assembler.rs` |
| `test_assemble_resets_state_between_calls` | `src/assembler.rs` |
| `test_unterminated_macro_definition_returns_error` | `src/assembler.rs` |
| `test_assembles_extended_flow_control_opcodes` | `src/assembler.rs` |
| `test_crypto_aliases_emit_canonical_syscalls` | `src/assembler.rs` |
| `test_syscall_accepts_canonical_name` | `src/assembler.rs` |
| `test_reserved_opcode_names_are_rejected` | `src/assembler.rs` |
| `test_try_supports_label_operands` | `src/assembler.rs` |
| `test_try_l_supports_label_operands` | `src/assembler.rs` |
| `test_assembles_extended_slot_and_type_opcodes` | `src/assembler.rs` |
| `test_istype_requires_type_operand` | `src/assembler.rs` |
| `test_pushint128_and_pushint256_encoding` | `src/assembler.rs` |
| `test_pushint256_accepts_32_byte_literal` | `src/assembler.rs` |
| `test_decode_long_flow_control_opcodes` | `src/disassembler.rs` |
| `test_disassembles_canonical_crypto_syscall` | `src/disassembler.rs` |
| `test_disassembles_reserved_opcode_bytes_as_unknown` | `src/disassembler.rs` |
| `test_parse_proof_mode_defaults_to_sp1` | `src/main.rs` |
| `test_parse_proof_mode_accepts_all_modes` | `src/main.rs` |
| `test_parse_proof_mode_accepts_short_alias` | `src/main.rs` |
| `test_parse_proof_mode_rejects_invalid_mode` | `src/main.rs` |
| `test_parse_proof_mode_requires_value` | `src/main.rs` |
| `test_parse_proof_mode_requires_value_short_alias` | `src/main.rs` |
| `test_parse_requested_proof_mode_detects_explicit_mode` | `src/main.rs` |
| `test_parse_allow_fallback_flag` | `src/main.rs` |
| `test_should_error_on_fallback_for_crypto_modes` | `src/main.rs` |
| `test_parse_gas_limit_requires_value` | `src/main.rs` |
| `test_cmd_prove_requires_release_for_crypto_mode_without_fallback` | `src/main.rs` |
| `test_full_prove_verify_cycle` | `tests/integration_tests.rs` |
| `test_complex_arithmetic` | `tests/integration_tests.rs` |
| `test_comparison_operations` | `tests/integration_tests.rs` |
| `test_prove_verify_with_arguments` | `tests/integration_tests.rs` |
| `test_prove_verify_hash_operation` | `tests/integration_tests.rs` |
| `test_prove_verify_array_operations` | `tests/integration_tests.rs` |
| `test_prove_verify_control_flow` | `tests/integration_tests.rs` |
| `test_execute_faulted_script` | `tests/integration_tests.rs` |
| `test_gas_tracking_in_proof` | `tests/integration_tests.rs` |
| `test_script_size_limit` | `tests/integration_tests.rs` |
| `test_stack_underflow_handling` | `tests/integration_tests.rs` |
| `test_division_by_zero` | `tests/integration_tests.rs` |
| `test_gas_exhaustion` | `tests/integration_tests.rs` |
| `test_pushdata_boundary` | `tests/integration_tests.rs` |
| `test_pushdata_truncated` | `tests/integration_tests.rs` |
| `test_loop_detection_by_gas` | `tests/integration_tests.rs` |
| `test_control_flow_jump_valid` | `tests/integration_tests.rs` |
| `test_control_flow_abort` | `tests/integration_tests.rs` |
| `test_control_flow_assert` | `tests/integration_tests.rs` |
| `test_control_flow_jump_backward` | `tests/integration_tests.rs` |
| `test_bitwise_operations` | `tests/integration_tests.rs` |
| `test_shift_operations` | `tests/integration_tests.rs` |
| `test_modulo_operations` | `tests/integration_tests.rs` |
| `test_power_operations` | `tests/integration_tests.rs` |
| `test_min_max_operations` | `tests/integration_tests.rs` |
| `test_within_range_check` | `tests/integration_tests.rs` |
| `test_shared_stack_value_serialization_roundtrip` | `tests/integration_tests.rs` |
| `test_cli_uses_shared_byte_arg_helper` | `tests/integration_tests.rs` |
| `test_native_crypto_sha256` | `tests/integration_tests.rs` |
| `test_throwifnot_byte_is_rejected_as_non_canonical` | `tests/integration_tests.rs` |
| `cli_opcode_decoding_uses_shared_opcode_enum` | `tests/source_layout.rs` |
| `cli_gas_estimation_uses_shared_opcode_enum` | `tests/source_layout.rs` |

## Dependency Boundary

| Dependency | Kind |
| --- | --- |
| `hex` | runtime |
| `neo-vm-guest` | runtime |
| `neo-vm-rs` | runtime |
| `neo-zkvm-prover` | runtime |
| `neo-zkvm-verifier` | runtime |
| `sha2` | runtime |

## Suggested Reading Path

1. Read `src/main.rs`: binary or CLI entrypoint.
2. Read `tests/integration_tests.rs`: external behavior or integration test.
3. Read `src/assembler.rs`: developer assembly and script construction.
4. Read `src/disassembler.rs`: developer assembly and script construction.
5. Read `tests/source_layout.rs`: external behavior or integration test.

## Change Safety Checklist

- Keep the stated responsibility boundary intact: Parse commands, Use shared opcode metadata, Run prove/verify workflows.
- Update the workflow and dataflow diagrams when adding or removing major execution steps.
- Add or update tests in the files listed under Test Evidence when public API or state-transition behavior changes.
- Re-run `python tools/docs/generate_crate_visual_docs.py` from the Neo N4 repository root after source layout changes.
