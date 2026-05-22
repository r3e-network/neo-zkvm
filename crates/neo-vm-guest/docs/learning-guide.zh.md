# neo-vm-guest 源码级学习指南

这份文档从 crate 的真实 `Cargo.toml`、Rust 源码文件、公开符号和测试函数生成。目标是在读实现细节之前，先弄清楚这个 crate 自己负责什么、边界在哪里、应该从哪些文件开始读。

## 这个 Crate 是什么

| 主题 | 说明 |
| --- | --- |
| 层级 | zkVM guest 外观层 |
| 目的 | 面向 zkVM guest 的适配层，以 zkVM 兼容方式暴露共享 NeoVM 执行 API。 |
| 输入 | guest 字节码、栈输入、共享 VM crate |
| 职责 | 调用共享 VM、保持 guest ABI 小、返回确定性结果 |
| 输出 | guest 执行结果、公开输出种子、fault 原因 |
| 使用者 | neo-zkvm-program、neo-zkvm-prover、示例 |

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

## 源码文件地图

| 文件 | 作用 | 公开符号 | 测试 |
| --- | --- | ---: | ---: |
| `src/lib.rs` | crate 根、公开导出和顶层文档 | 21 | 7 |
| `tests/shared_vm.rs` | 外部行为或集成测试 | 0 | 5 |

## 公开 API 面

| 符号 | 文件 |
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

## 模块与重导出信号

| 信号 |
| --- |
| `src/lib.rs: pub use neo_vm_rs::{interop_hash, pop_byte_arg, StackValue as StackItem}` |

## 测试证据

| 测试 | 文件 |
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

## 依赖边界

| 依赖 | 类型 |
| --- | --- |
| `bincode` | 运行时 |
| `neo-vm-rs` | 运行时 |
| `serde` | 运行时 |
| `sha2` | 运行时 |

## 建议阅读路径

1. 读 `src/lib.rs`：crate 根、公开导出和顶层文档。
2. 读 `tests/shared_vm.rs`：外部行为或集成测试。

## 修改安全清单

- 保持职责边界不变：调用共享 VM、保持 guest ABI 小、返回确定性结果。
- 增加或删除主要执行步骤时，同步更新工作流图和数据流图。
- 修改公开 API 或状态转换行为时，更新“测试证据”中对应的测试。
- 源码结构变化后，在 Neo N4 仓库根目录重新运行 `python tools/docs/generate_crate_visual_docs.py`。
