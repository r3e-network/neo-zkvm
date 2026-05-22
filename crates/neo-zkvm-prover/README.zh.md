# neo-zkvm-prover

<!-- N4-CRATE-VISUAL-GUIDE-ZH:START -->

## 可视化学习指南

这些图是 `neo-zkvm-prover` 自己目录下的 crate 专属学习资料，用来说明它在 Neo N4 中的位置、自己负责的技术边界、内部工作流，以及数据如何流经它。

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

### 在 Neo N4 中的作用

- **层级:** Neo zkVM 栈
- **目的:** 将 NeoVM 执行输入转换为可验证证明产物的证明生成库。
- **主要输入:** 证明输入、guest 程序、prover 模式
- **主要输出:** NeoProof、公开输出、prover 报告
- **下游使用者:** zkVM 用户、L2 prover 服务、L1 验证适配器
- **扫描到的源码文件:** 3
- **扫描到的公开符号:** 13
- **扫描到的 Rust 测试:** 23

### 边界与职责

- **本 crate 负责:** 哈希输入、运行 guest 执行、构建证明封装
- **本 crate 消费:** 证明输入、guest 程序、prover 模式
- **本 crate 产出:** NeoProof、公开输出、prover 报告
- **主要被谁使用:** zkVM 用户、L2 prover 服务、L1 验证适配器

### 源码地图快照

| 文件 | 为什么重要 | 公开 API | 测试 |
| --- | --- | ---: | ---: |
| `src/lib.rs` | crate 根、公开导出和顶层文档 | 9 | 23 |
| `src/elf_markers.rs` | 实现细节或辅助模块 | 4 | 0 |
| `build.rs` | 实现细节或辅助模块 | 0 | 0 |

### API 快照

| 类型 | 代表符号 |
| --- | --- |
| 类型 | ProverConfig <br> NeoProver |
| 函数 | is_elf_available <br> new <br> hash_proof_input <br> prove +2 |
| Trait | 未扫描到公开符号 |
| 常量 | DUMMY_ELF_NO_PROGRAM_SOURCE <br> DUMMY_ELF_NOT_FOR_PRODUCTION <br> DUMMY_ELF_FOR_CLIPPY <br> DUMMY_ELF_BUILD_FAILED +1 |

### 学习路径

1. 先看位置图，明确这个 crate 为什么存在、上游是谁、下游是谁。
2. 再看技术原理图，理解它的核心不变量、职责边界和维护规则。
3. 然后看模块图和 API 图，确定先读哪些文件、哪些符号。
4. 最后看工作流、数据流、测试证据图和依赖图，再进入源码会更容易理解。

<!-- N4-CRATE-VISUAL-GUIDE-ZH:END -->
