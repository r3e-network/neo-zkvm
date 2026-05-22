# neo-zkvm-cli 源码级学习指南

这份文档从 crate 的真实 `Cargo.toml`、Rust 源码文件、公开符号和测试函数生成。目标是在读实现细节之前，先弄清楚这个 crate 自己负责什么、边界在哪里、应该从哪些文件开始读。

## 这个 Crate 是什么

| 主题 | 说明 |
| --- | --- |
| 层级 | Neo zkVM 栈 |
| 目的 | 用于汇编、检查、生成证明和验证 Neo zkVM 程序的 CLI 与开发工具。 |
| 输入 | CLI 命令、脚本/证明文件、prover 选项 |
| 职责 | 解析命令、使用共享 opcode 元数据、运行 prove/verify 工作流 |
| 输出 | 汇编脚本、证明报告、检查输出 |
| 使用者 | zkVM 用户、L2 prover 服务、L1 验证适配器 |

## 可视化阅读顺序

| 步骤 | 图 | 用它学习什么 |
| ---: | --- | --- |
| 1 | [位置图](figures/position.zh.svg) | 这个 crate 为什么存在、在 Neo N4 中处于哪里。 |
| 2 | [技术原理图](figures/principles.zh.svg) | 这个 crate 必须保护的不变量和职责边界。 |
| 3 | [模块图](figures/module-map.zh.svg) | 哪些源码文件是最好的入口。 |
| 4 | [公开 API 图](figures/api-surface.zh.svg) | 哪些导出符号构成 crate 契约。 |
| 5 | [架构图](figures/architecture.zh.svg) | 输入、内部组件、依赖和输出如何连接。 |
| 6 | [工作流图](figures/workflow.zh.svg) | 正常执行路径。 |
| 7 | [数据流图](figures/dataflow.zh.svg) | 数据如何跨越 crate 边界并被转换。 |
| 8 | [测试证据图](figures/test-map.zh.svg) | 哪些测试保护行为。 |
| 9 | [依赖图](figures/dependency-map.zh.svg) | 哪些依赖是运行时、测试或构建期依赖。 |
| 10 | [实现全景图](figures/implementation-atlas.zh.svg) | 用一张高密度图同时理解用途、源码入口、API、工作流、数据流、依赖、测试和修改检查点。 |

## 源码文件地图

| 文件 | 作用 | 公开符号 | 测试 |
| --- | --- | ---: | ---: |
| `src/main.rs` | 二进制或 CLI 入口 | 0 | 11 |
| `tests/integration_tests.rs` | 外部行为或集成测试 | 0 | 30 |
| `src/assembler.rs` | 开发者汇编和脚本构造 | 5 | 14 |
| `src/disassembler.rs` | 开发者汇编和脚本构造 | 4 | 3 |
| `tests/source_layout.rs` | 外部行为或集成测试 | 0 | 2 |

## 公开 API 面

| 符号 | 文件 |
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

## 模块与重导出信号

| 信号 |
| --- |
| `src/main.rs: mod assembler` |
| `src/main.rs: mod disassembler` |

## 测试证据

| 测试 | 文件 |
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

## 依赖边界

| 依赖 | 类型 |
| --- | --- |
| `hex` | 运行时 |
| `neo-vm-guest` | 运行时 |
| `neo-vm-rs` | 运行时 |
| `neo-zkvm-prover` | 运行时 |
| `neo-zkvm-verifier` | 运行时 |
| `sha2` | 运行时 |

## 建议阅读路径

1. 读 `src/main.rs`：二进制或 CLI 入口。
2. 读 `tests/integration_tests.rs`：外部行为或集成测试。
3. 读 `src/assembler.rs`：开发者汇编和脚本构造。
4. 读 `src/disassembler.rs`：开发者汇编和脚本构造。
5. 读 `tests/source_layout.rs`：外部行为或集成测试。

## 修改安全清单

- 保持职责边界不变：解析命令、使用共享 opcode 元数据、运行 prove/verify 工作流。
- 增加或删除主要执行步骤时，同步更新工作流图和数据流图。
- 修改公开 API 或状态转换行为时，更新“测试证据”中对应的测试。
- 源码结构变化后，在 Neo N4 仓库根目录重新运行 `python tools/docs/generate_crate_visual_docs.py`。
