# neo-zkvm-examples

<!-- N4-CRATE-VISUAL-GUIDE-ZH:START -->

## 可视化学习指南

这些图是 `neo-zkvm-examples` 自己目录下的 crate 专属学习资料，用来说明它在 Neo N4 中的位置、自己负责的技术边界、内部工作流，以及数据如何流经它。

完整的源码级解释见 [docs/learning-guide.zh.md](docs/learning-guide.zh.md)。

| 视图 | 图片 | 源文件 |
| --- | --- | --- |
| 在 Neo N4 中的位置 | ![位置](docs/figures/position.zh.svg) | [Mermaid](docs/figures/position.zh.mmd) |
| 技术原理 | ![技术原理](docs/figures/principles.zh.svg) | [Mermaid](docs/figures/principles.zh.mmd) |
| 架构 | ![架构](docs/figures/architecture.zh.svg) | [Mermaid](docs/figures/architecture.zh.mmd) |
| 工作流 | ![工作流](docs/figures/workflow.zh.svg) | [Mermaid](docs/figures/workflow.zh.mmd) |
| 数据流 | ![数据流](docs/figures/dataflow.zh.svg) | [Mermaid](docs/figures/dataflow.zh.mmd) |
| 模块图 | ![模块图](docs/figures/module-map.zh.svg) | [Mermaid](docs/figures/module-map.zh.mmd) |
| 公开 API 图 | ![公开 API 图](docs/figures/api-surface.zh.svg) | [Mermaid](docs/figures/api-surface.zh.mmd) |
| 测试证据图 | ![测试证据图](docs/figures/test-map.zh.svg) | [Mermaid](docs/figures/test-map.zh.mmd) |
| 依赖图 | ![依赖图](docs/figures/dependency-map.zh.svg) | [Mermaid](docs/figures/dependency-map.zh.mmd) |
| 实现全景图 | ![实现全景图](docs/figures/implementation-atlas.zh.svg) | [Mermaid](docs/figures/implementation-atlas.zh.mmd) |

### 在 Neo N4 中的作用

- **层级:** Neo zkVM 栈
- **目的:** 展示常见证明流程和应用模式的可运行示例。
- **主要输入:** 示例脚本、示例输入、本地 prover
- **主要输出:** 示例证明、教程输出、回归样本
- **下游使用者:** zkVM 用户、L2 prover 服务、L1 验证适配器
- **扫描到的源码文件:** 9
- **扫描到的公开符号:** 0
- **扫描到的 Rust 测试:** 6

### 边界与职责

- **本 crate 负责:** 演示 API、覆盖边界案例、记录期望输出
- **本 crate 消费:** 示例脚本、示例输入、本地 prover
- **本 crate 产出:** 示例证明、教程输出、回归样本
- **主要被谁使用:** zkVM 用户、L2 prover 服务、L1 验证适配器

### 源码地图快照

| 文件 | 为什么重要 | 公开 API | 测试 |
| --- | --- | ---: | ---: |
| `src/batch_verification.rs` | 实现细节或辅助模块 | 0 | 2 |
| `src/private_inputs.rs` | 实现细节或辅助模块 | 0 | 2 |
| `src/tamper_resistance.rs` | 实现细节或辅助模块 | 0 | 2 |
| `src/basic.rs` | 实现细节或辅助模块 | 0 | 0 |
| `src/proof_generation.rs` | 证明对象、布局和验证证据 | 0 | 0 |
| `src/zk_dao_voting.rs` | 实现细节或辅助模块 | 0 | 0 |
| `src/zk_dex_rollup.rs` | 实现细节或辅助模块 | 0 | 0 |
| `src/zk_preimage.rs` | 实现细节或辅助模块 | 0 | 0 |

### API 快照

| 类型 | 代表符号 |
| --- | --- |
| 类型 | 未扫描到公开符号 |
| 函数 | 未扫描到公开符号 |
| Trait | 未扫描到公开符号 |
| 常量 | 未扫描到公开符号 |

### 学习路径

1. 先看位置图，明确这个 crate 为什么存在、上游是谁、下游是谁。
2. 再看技术原理图，理解它的核心不变量、职责边界和维护规则。
3. 然后看模块图和 API 图，确定先读哪些文件、哪些符号。
4. 最后看工作流、数据流、测试证据图和依赖图，再进入源码会更容易理解。
5. 如果希望一张图看完整体，就看实现全景图；它把源码入口、API、数据流、测试、依赖和修改检查点放在一起。

<!-- N4-CRATE-VISUAL-GUIDE-ZH:END -->
