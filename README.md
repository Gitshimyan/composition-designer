# Composition Designer

> v0.1 · M3 进行中：M1/M2 Core、编辑事务与 TOML 写回已完成；012–014 建立 Tauri 2 + React Designer、Canvas/Tree、Inspector 配置编辑、Change Review 与只读 TOML Source；外部修改同步未开始。

Composition Designer 是一个面向**声明式项目组合**的设计环境。项目由一个权威的 Composition
Model 描述，通过 TOML 持久化；可视化 Canvas、TOML Source、Agent Interface 与 CLI 都只能
通过 Composition Core 读取或修改它。

- 设计基线：[composition-designer-v0.1-design.md](./composition-designer-v0.1-design.md)
- 下一步行动入口：[working/plan.md](./working/plan.md)
- AI 工作规则：[AGENTS.md](./AGENTS.md)
- AI 架构入口：[docs/ai/architecture-entry.md](./docs/ai/architecture-entry.md)
- 冻结规范：[docs/spec/](./docs/spec/README.md)
- 测试数据：[examples/](./examples/README.md)

## 1. v0.1 要解决什么

1. **Describe**：用 TOML 描述项目、模块定义、实例、组合、配置与接口关系；
2. **Modify**：可视化界面、TOML 编辑器和 Agent 接口修改同一份项目模型；
3. **Resolve**：把 TOML 解析为规范化、确定性的 Composition Model；
4. **Validate**：检查语法、Schema、引用、契约、组合与依赖，返回可定位诊断。

v0.1 **明确不做**：运行 / 构建 / 部署项目、生成业务代码、在线模块市场、复杂版本求解、
隐式 Adapter、多 Agent 调度、在 TOML 中执行脚本。边界详见设计基线 §1.2 / §14 与
[ADR-0002](./docs/adr/0002-v01-unsupported.md)。

## 2. 当前状态（2026-11-26）

| 项 | 状态 |
|---|---|
| 设计基线 | 就绪（Draft） |
| working 编排 / AGENTS / Change Manifest | 就绪 |
| 领域模型与术语（M0 002） | 已冻结 |
| TOML Schema 与错误码（M0 003） | 已冻结（63 个验证代码 + 6 个预留代码） |
| 示例项目集（M0 004） | 26 个 `expected.json` 数据集（25 个完整项目 + 入口缺失目录） |
| Rust 领域层（M1 005） | 已建立：`composition-domain`，58 项测试 |
| TOML 加载与 Source Map（M1 006） | 已建立：`composition-toml`，27 项测试 |
| Core：解析 / 验证（M1 007） | 已建立：`composition-core`，26/26 数据集 |
| CLI（M1 008） | 已建立：`composition-cli`，8 项测试 |
| 编辑命令与事务（M2 009–011） | 已建立：`composition-editor`，29 项测试 |
| Revision / 幂等 / Diff | 已建立（M2 010） |
| 保留格式 TOML 写回 | 已建立（M2 011） |
| Designer 外壳（M3 012） | 已建立：Tauri 2 + React 19 + TS + Vite 7 |
| Canvas / Tree（M3 013） | 已建立：`@xyflow/react`、结构树、布局持久化 |
| Inspector / Source（M3 014） | 已建立：配置编辑、Change Review、只读 CodeMirror Source |
| 外部修改同步 | 未开始（M3 015） |
| Agent Interface | 未开始（M4） |

当前已有 `composition-domain`、`composition-toml`、`composition-core`、`composition-cli`、
`composition-editor` 与 `apps/designer`；Core 通过 `cargo test` / `clippy` / `fmt`，Designer
通过 `npm run build` 与 Tauri 宿主测试。014 已把 Inspector 配置编辑接到 Core 事务，并提供
只读 CodeMirror Source 与诊断定位；外部修改同步和 Agent 协议尚未开始。
M1 的“通过”指只读解析/校验/CLI 可复现，不代表项目可以运行或编辑。

## 3. 目录结构

```text
composition-designer/
├── composition-designer-v0.1-design.md   # 设计基线（Draft）
├── AGENTS.md                              # AI 工作规则
├── README.md
│
├── crates/                                # Rust Core（M1 起；职责见 crates/README.md）
│   ├── composition-domain/                # 已建立（005）
│   ├── composition-toml/                  # 已建立（006）
│   ├── composition-core/                  # 已建立（007，解析/校验）
│   ├── composition-cli/                   # 已建立（008，inspect/resolve/validate）
│   ├── composition-editor/                # 已建立（009，命令/事务）
│   ├── composition-resolver/              # 暂并入 core
│   ├── composition-validator/             # 暂并入 core
│   ├── composition-editor/
│   ├── composition-protocol/
│   └── composition-app/
│
├── apps/
│   └── designer/                          # Tauri 2 + React 设计器（012–013 外壳/Canvas）
│       ├── src/{canvas,explorer,inspector,source,diagnostics,agent-status}/
│       └── src-tauri/
│
├── examples/                              # 项目样例 + expected.json（M0 完成）
│   ├── minimal-project/
│   ├── nested-composition/
│   ├── edge-valid/
│   └── invalid-projects/
│
├── docs/
│   ├── adr/                               # 架构决策（0001 冻结、0002 不支持项）
│   ├── ai/                                # AI 入口
│   ├── spec/                              # M0 冻结规范
│   └── glossary.md                        # 术语表
│
├── changes/                               # Change Manifest（改动范围声明）
└── working/                               # 计划、任务卡、过程文件
    ├── plan.md
    ├── tasks/
    └── tmp/
```

## 4. 技术栈（设计建议，M1 前不视为冻结）

| 层 | 建议 |
|---|---|
| Domain Core | Rust |
| TOML | `toml_edit` + 解析器（保留注释与格式） |
| Desktop Host | Tauri 2 |
| UI | React + TypeScript |
| Canvas | `@xyflow/react` 或等价图编辑库 |
| Source Editor | CodeMirror 6 |
| Agent Adapter | 本地协议 + MCP Adapter |

## 5. working/ 编排约定

| 路径 | 作用 |
|---|---|
| `working/plan.md` | 唯一的“下一步行动”入口：目标、状态、路线图、队列、决策、日志 |
| `working/tasks/NNN-*.md` | 每次行动一张任务卡：状态、目标、授权、清单、验收、结果 |
| `working/tmp/` | 过程文件（探针、日志、草稿），不提交 |

规则：开始前先读 `plan.md`；一次只取一张卡；完成后回写状态与日志，再把“下一步”收敛成
少数几项。模板是 `working/tasks/000-template.md`。

## 6. 文档索引

- [设计基线](./composition-designer-v0.1-design.md)
- [AI 架构入口](./docs/ai/architecture-entry.md)
- [AI 工作规则](./AGENTS.md)
- [术语表](./docs/glossary.md)
- [领域模型 v1](./docs/spec/domain-model-v1.md)
- [Composition Schema v1](./docs/spec/composition-schema-v1.md)
- [命令 Schema v1](./docs/spec/command-schema-v1.md)
- [Designer Layout v1](./docs/spec/designer-layout-v1.md)
- [错误代码 v1](./docs/spec/error-codes-v1.md)
- [ADR-0001 领域模型与 Schema 冻结](./docs/adr/0001-domain-and-schema-freeze.md)
- [ADR-0002 v0.1 显式不支持项](./docs/adr/0002-v01-unsupported.md)
- [crates 职责与依赖方向](./crates/README.md)
- [Designer UI 区域](./apps/designer/README.md)
- [示例项目约定与覆盖矩阵](./examples/README.md)
- [ADR 模板](./docs/adr/0000-template.md)
