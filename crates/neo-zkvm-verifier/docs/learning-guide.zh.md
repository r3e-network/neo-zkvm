# neo-zkvm-verifier 源码级学习指南

这份文档从 crate 的真实 `Cargo.toml`、Rust 源码文件、公开符号和测试函数生成。目标是在读实现细节之前，先弄清楚这个 crate 自己负责什么、边界在哪里、应该从哪些文件开始读。

## 这个 Crate 是什么

| 主题 | 说明 |
| --- | --- |
| 层级 | Neo zkVM 栈 |
| 目的 | 校验证明封装、公开输出和模式兼容性的 verifier 库。 |
| 输入 | 证明封装、验证键、期望公开输出 |
| 职责 | 检查证明格式、验证公开值、报告有效性 |
| 输出 | 验证结果、错误原因、审计证据 |
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
| `src/lib.rs` | crate 根、公开导出和顶层文档 | 9 | 12 |

## 公开 API 面

| 符号 | 文件 |
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

## 模块与重导出信号

未扫描到 `mod` 或 `pub use` 声明。

## 测试证据

| 测试 | 文件 |
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

## 依赖边界

| 依赖 | 类型 |
| --- | --- |
| `bincode` | 运行时 |
| `neo-vm-guest` | 运行时 |
| `neo-zkvm-prover` | 运行时 |
| `sp1-sdk` | 运行时 |

## 建议阅读路径

1. 读 `src/lib.rs`：crate 根、公开导出和顶层文档。

## 修改安全清单

- 保持职责边界不变：检查证明格式、验证公开值、报告有效性。
- 增加或删除主要执行步骤时，同步更新工作流图和数据流图。
- 修改公开 API 或状态转换行为时，更新“测试证据”中对应的测试。
- 源码结构变化后，在 Neo N4 仓库根目录重新运行 `python tools/docs/generate_crate_visual_docs.py`。
