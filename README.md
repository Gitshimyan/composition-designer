# Composition Designer

> v0.2 M6 与 G01–G08 完成；ADR-0004/0005 Accepted。
> M7 Source Authoring 完成（029–031：编辑 Source、文本+对象 Review、Glide 验收）。
> S1、M8、S2、H1 完成：undo/redo、跨进程状态、fsync、草稿保留、真实 WebView2 smoke；下一步生态/维护。
> 计划入口：[`working/plan.md`](./working/plan.md)。

Composition Designer 是一个面向**声明式项目组合**的设计环境。项目由一个权威的 Composition
Model 描述，通过 TOML 持久化；可视化 Canvas、TOML Source、Agent Interface 与 CLI 都只能
通过 Composition Core 读取或修改它。

- 设计基线：[docs/design/v0.1-design.md](./docs/design/v0.1-design.md)
- 评审与路线图：[docs/reviews/](./docs/reviews/README.md)
- 未决工作登记：[working/decisions/unresolved-work.md](./working/decisions/unresolved-work.md)
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

## 2. 当前状态（2026-11-28）

| 项 | 状态 |
|---|---|
| 设计基线 | 就绪（Draft） |
| working 编排 / AGENTS / Change Manifest | 就绪 |
| 领域模型与术语（M0 002） | 已冻结 |
| TOML Schema 与错误码（M0 003） | 已冻结（63 个验证代码 + 6 个预留代码） |
| 示例项目集（M0 004 / S1 035 / S2 041） | 40 个 `expected.json` 数据集（39 个完整项目 + 入口缺失目录；含 11 个 v2、3 个 v3 数据集） |
| Rust 领域层（M1 005） | 已建立：`composition-domain`，58 项测试 |
| TOML 加载与 Source Map（M1 006 / M5 019） | 已建立：`composition-toml`，29 项测试 |
| Core：解析 / 验证（M1 007 / M5 019） | 已建立：`composition-core`，9 项测试、26/26 数据集 |
| CLI（M1 008 / M4 016） | 已建立：`composition-cli`，10 项测试 |
| 编辑命令与事务（M2 009–011 / M5 019 / 025 / 029） | 已建立：`composition-editor`，44 项测试；15 条命令 |
| Revision / 幂等 / Diff | 已建立（M2 010） |
| 保留格式 TOML 写回 | 已建立（M2 011；025 增加 Dependency Provider） |
| Designer 外壳（M3 012） | 已建立：Tauri 2 + React 19 + TS + Vite 7 |
| Canvas / Tree（M3 013） | 已建立：`@xyflow/react`、结构树、布局持久化 |
| Inspector / Source（M3 014 / M7 030） | 已建立：配置编辑、Change Review、可编辑 CodeMirror Source、草稿/冲突只读 |
| 外部修改同步（M3 015） | 已建立：digest 轮询、冲突守卫、Reload |
| Agent Protocol（M4 016） | 已建立：query/transaction/JSONL、CLI query/apply |
| MCP Adapter（M4 017） | 已建立：stdio JSON-RPC、6 tools / 4 resources |
| Agent Skill / 端到端（M4 018） | 已建立：Skill、MCP 配置、真实 stdio e2e |
| 完整性与验收 | 已完成（M5 019）：A01–A18 报告 |
| v0.2 / M6 | M6 完成（020–027）：Glide 全链路、完整 Review、PR 强制门禁；G01–G05 完成，change-gate block |
| M7 Source Authoring（029–031） | 完成：`replace_source` 独占事务、可编辑 Source、文本+对象 Diff、Glide 验收；G06–G08 完成 |
| S1 Schema v2.0（032–035） | 完成：语义冻结、Core 校验、命令/UI/入口、11 个 v2 数据集与 Glide v2 试点验收；UW-002/003 关闭 |
| M8 Agent Authoring（036–038） | 完成：`context`/`impact`、只读 `batch`、Skill 评测与跨平台配置；220 workspace 测试 |
| S2 Schema v2.1（039–041） | 完成：format 3 Profile/多入口、CLI/MCP/Designer 选择、v3 数据集；238 workspace + 35 UI + 17 Tauri |
| H1 可靠性（042–046） | 完成：undo/redo、跨进程状态、fsync、草稿保留、真实 WebView2 smoke；251 workspace + 38 UI + 17 Tauri + 1 smoke |

当前已有 `composition-domain`、`composition-toml`、`composition-core`、`composition-cli`、
`composition-editor`、`composition-protocol`、`composition-mcp` 与 `apps/designer`；
Core/Protocol/MCP 通过 `cargo test` / `clippy` / `fmt`，Designer 通过 `npm run build` 与
Tauri 宿主测试。M0–M7 已完成；v0.1 验收报告见
[`docs/acceptance-v0.1.md`](./docs/acceptance-v0.1.md)。
M1–M7 的“通过”指解析/校验/CLI/编辑/Designer 可复现，不代表项目可以运行或部署。
Source 编辑首版仅覆盖 Composition 入口，模块文件仍只读（UW-031）。

## 3. 目录结构

```text
composition-designer/
├── AGENTS.md                              # AI 工作规则
├── README.md
├── .github/workflows/                     # Windows CI（020）
│
├── crates/                                # Rust Core（M1 起；职责见 crates/README.md）
│   ├── composition-domain/                # 已建立（005）
│   ├── composition-toml/                  # 已建立（006）
│   ├── composition-core/                  # 已建立（007，解析/校验）
│   ├── composition-cli/                   # 已建立（008，inspect/resolve/validate）
│   ├── composition-editor/                # 已建立（009，命令/事务）
│   ├── composition-protocol/              # 已建立（016，Query/Command/JSONL）
│   ├── composition-mcp/                   # 已建立（017，MCP stdio）
│   ├── composition-resolver/              # 暂并入 core
│   ├── composition-validator/             # 暂并入 core
│   └── composition-app/                   # 待 M3/M4
│
├── apps/
│   └── designer/                          # Tauri 2 + React 设计器（012–015、022–025）
│       ├── src/{canvas,explorer,inspector,relations,source,diagnostics,agent-status}/
│       └── src-tauri/
│
├── agent/                                 # Agent Skill + MCP 配置（018）
│
├── examples/                              # 项目样例 + expected.json（M0 完成）
│   ├── minimal-project/
│   ├── nested-composition/
│   ├── edge-valid/
│   └── invalid-projects/
│
├── docs/
│   ├── adr/                               # 架构决策（0001 冻结、0002 不支持项）
│   ├── design/                            # 设计基线（v0.1 冻结）
│   ├── reviews/                           # 外部评审与路线图
│   ├── ai/                                # AI 入口
│   ├── spec/                              # M0 冻结规范
│   └── glossary.md                        # 术语表
│
├── changes/                               # Change Manifest（改动范围声明）
└── working/                               # 计划、任务卡、过程文件
    ├── plan.md
    ├── tasks/
    ├── decisions/                         # 未决工作登记与决策历史
    ├── pilots/                            # 021 真实项目试点
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
| `working/decisions/unresolved-work.md` | 未决工作登记册（UW ID、状态、授权、影响、后续） |
| `working/decisions/decision-history.md` | 用户决策与授权历史 |
| `working/tmp/` | 过程文件（探针、日志、草稿），不提交 |

规则：开始前先读 `plan.md`；一次只取一张卡；完成后回写状态与日志，再把“下一步”收敛成
少数几项。模板是 `working/tasks/000-template.md`。

## 6. 文档索引

- [设计基线 v0.1](./docs/design/v0.1-design.md)
- [评审索引](./docs/reviews/README.md)
- [2026-11-27 项目评审与路线图](./docs/reviews/2026-11-27-project-review-and-roadmap.md)
- [2026-11-27 授权体系审计](./docs/reviews/2026-11-27-authorization-audit.md)
- [2026-11-27 未实现工作披露评审](./docs/reviews/2026-11-27-unimplemented-work-transparency.md)
- [2026-11-27 评审响应与整改计划](./docs/reviews/2026-11-27-audit-response.md)
- [001–025 历史授权审计记录](./docs/reviews/2026-11-27-authorization-audit-record.md)
- [001–025 未实现工作回溯报告](./docs/reviews/2026-11-27-unimplemented-work-backfill.md)
- [M6 收口验收](./docs/reviews/2026-11-27-m6-closeout.md)
- [机器授权/覆盖门禁](./docs/reviews/2026-11-27-machine-gate.md)
- [未决工作登记](./working/decisions/unresolved-work.md)
- [决策与授权历史](./working/decisions/decision-history.md)
- [Glide Task + Terminal 试点](./working/pilots/glide-task-terminal/README.md)
- [Glide Task + Terminal v2 试点](./working/pilots/glide-task-terminal-v2/README.md)
- [AI 架构入口](./docs/ai/architecture-entry.md)
- [AI 工作规则](./AGENTS.md)
- [术语表](./docs/glossary.md)
- [领域模型 v1](./docs/spec/domain-model-v1.md)
- [Composition Schema v1](./docs/spec/composition-schema-v1.md)
- [Composition Schema v2.0](./docs/spec/composition-schema-v2.md)
- [Composition Schema v3（v2.1）](./docs/spec/composition-schema-v3.md)
- [命令 Schema v1](./docs/spec/command-schema-v1.md)
- [Designer Layout v1](./docs/spec/designer-layout-v1.md)
- [Composition Protocol v1](./docs/spec/protocol-v1.md)
- [MCP Adapter v1](./docs/spec/mcp-adapter-v1.md)
- [v0.1 验收报告](./docs/acceptance-v0.1.md)
- [错误代码 v1](./docs/spec/error-codes-v1.md)
- [错误代码 v2.0](./docs/spec/error-codes-v2.md)
- [错误代码 v3（v2.1）](./docs/spec/error-codes-v3.md)
- [ADR-0001 领域模型与 Schema 冻结](./docs/adr/0001-domain-and-schema-freeze.md)
- [ADR-0002 v0.1 显式不支持项](./docs/adr/0002-v01-unsupported.md)
- [ADR-0003 Dependency Provider 命令](./docs/adr/0003-dependency-provider-command.md)
- [ADR-0004 Schema v2 方向](./docs/adr/0004-schema-v2-direction.md)
- [ADR-0005 Source 编辑事务](./docs/adr/0005-source-editing-transaction.md)
- [ADR-0006 Schema v2.0 冻结](./docs/adr/0006-schema-v2-freeze.md)
- [ADR-0007 Schema v2.1 Profile 冻结](./docs/adr/0007-schema-v21-profiles.md)
- [ADR-0008 外部修改与草稿策略](./docs/adr/0008-external-change-drafts.md)
- [ADR-0009 WebView2 smoke](./docs/adr/0009-webview2-smoke.md)
- [crates 职责与依赖方向](./crates/README.md)
- [Designer UI 区域](./apps/designer/README.md)
- [Agent Skill / MCP 配置](./agent/README.md)
- [示例项目约定与覆盖矩阵](./examples/README.md)
- [ADR 模板](./docs/adr/0000-template.md)

## License

MIT © 2026 Gitshimyan. See [LICENSE](./LICENSE).
