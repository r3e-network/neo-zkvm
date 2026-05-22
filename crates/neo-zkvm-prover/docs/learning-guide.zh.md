# neo-zkvm-prover 源码级学习指南

这份文档从 crate 的真实 `Cargo.toml`、Rust 源码文件、公开符号和测试函数生成。目标是在读实现细节之前，先弄清楚这个 crate 自己负责什么、边界在哪里、应该从哪些文件开始读。

## 这个 Crate 是什么

| 主题 | 说明 |
| --- | --- |
| 层级 | Neo zkVM 栈 |
| 目的 | 将 NeoVM 执行输入转换为可验证证明产物的证明生成库。 |
| 输入 | 证明输入、guest 程序、prover 模式 |
| 职责 | 哈希输入、运行 guest 执行、构建证明封装 |
| 输出 | NeoProof、公开输出、prover 报告 |
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

## 源码文件地图

| 文件 | 作用 | 公开符号 | 测试 |
| --- | --- | ---: | ---: |
| `src/lib.rs` | crate 根、公开导出和顶层文档 | 9 | 23 |
| `src/elf_markers.rs` | 实现细节或辅助模块 | 4 | 0 |
| `build.rs` | 实现细节或辅助模块 | 0 | 0 |

## 公开 API 面

| 符号 | 文件 |
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

## 模块与重导出信号

| 信号 |
| --- |
| `build.rs: mod elf_markers` |
| `src/lib.rs: mod elf_markers` |
| `src/lib.rs: pub use neo_vm_guest::{MockProof, NeoProof, ProofMode, PublicInputs, PROOF_FORMAT_VERSION}` |

## 测试证据

| 测试 | 文件 |
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

## 依赖边界

| 依赖 | 类型 |
| --- | --- |
| `bincode` | 运行时 |
| `neo-vm-guest` | 运行时 |
| `sp1-sdk` | 运行时 |
| `sp1-build` | 构建 |

## 建议阅读路径

1. 读 `src/lib.rs`：crate 根、公开导出和顶层文档。
2. 读 `src/elf_markers.rs`：实现细节或辅助模块。
3. 读 `build.rs`：实现细节或辅助模块。

## 修改安全清单

- 保持职责边界不变：哈希输入、运行 guest 执行、构建证明封装。
- 增加或删除主要执行步骤时，同步更新工作流图和数据流图。
- 修改公开 API 或状态转换行为时，更新“测试证据”中对应的测试。
- 源码结构变化后，在 Neo N4 仓库根目录重新运行 `python tools/docs/generate_crate_visual_docs.py`。
