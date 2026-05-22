# neo-zkvm-fuzz 源码级学习指南

这份文档从 crate 的真实 `Cargo.toml`、Rust 源码文件、公开符号和测试函数生成。目标是在读实现细节之前，先弄清楚这个 crate 自己负责什么、边界在哪里、应该从哪些文件开始读。

## 这个 Crate 是什么

| 主题 | 说明 |
| --- | --- |
| 层级 | Neo zkVM 栈 |
| 目的 | 用于对证明与 VM 输入做对抗性探索的 fuzz 工作区。 |
| 输入 | 随机字节码、变异证明、种子语料 |
| 职责 | 生成输入、运行 no-panic 检查、捕获回归 |
| 输出 | 崩溃语料、回归案例、覆盖率信号 |
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
| `fuzz_targets/common.rs` | fuzz harness 与对抗输入探索 | 1 | 0 |
| `fuzz_targets/fuzz_script_parser.rs` | fuzz harness 与对抗输入探索 | 0 | 0 |
| `fuzz_targets/fuzz_vm_execution.rs` | fuzz harness 与对抗输入探索 | 0 | 0 |

## 公开 API 面

| 符号 | 文件 |
| --- | --- |
| `fn append_bounded_neo_vm_sequence` | `fuzz_targets/common.rs` |

## 模块与重导出信号

| 信号 |
| --- |
| `fuzz_targets/fuzz_script_parser.rs: mod common` |
| `fuzz_targets/fuzz_vm_execution.rs: mod common` |

## 测试证据

这个 crate 中未扫描到 Rust `#[test]` 函数。

## 依赖边界

| 依赖 | 类型 |
| --- | --- |
| `arbitrary` | 运行时 |
| `libfuzzer-sys` | 运行时 |
| `neo-vm-guest` | 运行时 |

## 建议阅读路径

1. 读 `fuzz_targets/common.rs`：fuzz harness 与对抗输入探索。
2. 读 `fuzz_targets/fuzz_script_parser.rs`：fuzz harness 与对抗输入探索。
3. 读 `fuzz_targets/fuzz_vm_execution.rs`：fuzz harness 与对抗输入探索。

## 修改安全清单

- 保持职责边界不变：生成输入、运行 no-panic 检查、捕获回归。
- 增加或删除主要执行步骤时，同步更新工作流图和数据流图。
- 修改公开 API 或状态转换行为时，更新“测试证据”中对应的测试。
- 源码结构变化后，在 Neo N4 仓库根目录重新运行 `python tools/docs/generate_crate_visual_docs.py`。
