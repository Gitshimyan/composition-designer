# AI 架构入口

> 给 AI / 新协作者的入口。开始修改前先读本文件、[`working/plan.md`](../../working/plan.md)
> 与设计基线。事实来源是规范、样例与命令输出，不是本文件的文字描述。
> M0 已冻结（2026-11-26）；字段与错误码以 [`docs/spec/`](../spec/) 为准。

## 1. 项目速览

| 项 | 值 |
|---|---|
| 项目 | Composition Designer |
| 版本 | v0.1 冻结；v0.2 M6 进行中 |
| 当前阶段 | v0.2 M6 进行中：020–025 完成；评审整改 G01 等待 Rearchitect 授权，026 Change Review 顺延 |
| 设计基线 | [`docs/design/v0.1-design.md`](../design/v0.1-design.md) |
| 外部评审 | [项目评审](../reviews/2026-11-27-project-review-and-roadmap.md) · [授权审计](../reviews/2026-11-27-authorization-audit.md) · [未实现披露](../reviews/2026-11-27-unimplemented-work-transparency.md) · [整改计划](../reviews/2026-11-27-audit-response.md) |
| 冻结规范 | [`docs/spec/`](../spec/)（domain / schema / error codes） |
| 测试数据 | [`examples/`](../../examples/README.md)（每个样例带 `expected.json`） |
| 执行顺序 | 设计基线 §16：冻结语义 → 冻结 Schema → 样例 → Core → 编辑事务 → GUI → Agent |
| 技术栈 | Rust Core + `toml_edit`；Tauri 2；React + TypeScript；`@xyflow/react`；CodeMirror 6（设计建议，未冻结） |
| 验证入口 | Core：`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check`；Pilot：`composition-cli validate working/pilots/glide-task-terminal --json`；Protocol：`cargo run -p composition-cli -- query examples/minimal-project validate`；MCP：`cargo test -p composition-mcp --test e2e`；Designer：`cd apps/designer && npm run build`、`npm run test:ui`、`npm run desktop`、`cargo test --manifest-path src-tauri/Cargo.toml`；验收：`docs/acceptance-v0.1.md` |
| working 入口 | [`working/plan.md`](../../working/plan.md)、[`working/decisions/unresolved-work.md`](../../working/decisions/unresolved-work.md) |

## 2. 目标结构与依赖方向

### crates/

| crate | 状态 | 职责 | 允许依赖 |
|---|---|---|---|
| `composition-domain` | 已建立（005） | 领域类型、不变量、错误码、serde JSON | serde |
| `composition-toml` | 已建立（006） | TOML 加载、Source Map、source_digest | domain, toml_edit, sha2 |
| `composition-core` | 已建立（007） | 解码、模块注册、L1–L7 校验、CompositionModel | domain, toml, toml_edit |
| `composition-cli` | 已建立（008） | inspect / resolve / validate、JSON、退出码 | core, domain, serde, serde_json |
| `composition-resolver` | 暂并入 core | 模块解析、规范化结果 | domain, toml |
| `composition-validator` | 暂并入 core | Schema、引用、契约校验 | domain, resolver |
| `composition-editor` | 已建立（009–014） | 命令、事务、Revision/幂等、结构化 Diff、保留格式 TOML 写回、wire 反序列化 | core, domain, toml, toml_edit, serde, serde_json |
| `composition-protocol` | 已建立（016） | Query/Command、JSONL、外部冲突守卫 | core, domain, editor, serde, serde_json |
| `composition-mcp` | 已建立（017–018） | MCP stdio、tools/resources、e2e | protocol, serde, serde_json |
| `composition-app` | 待 M3/M4 | 应用服务编排 | core |

005 建立 `composition-domain`，006 建立 `composition-toml`，007 建立 `composition-core`，
008 建立 `composition-cli`，009 建立 `composition-editor`；resolver/validator 暂并入 core，
其余在 M3/M4 按依赖方向加入。

### apps/designer/

| 区域 | 职责 |
|---|---|
| `src/canvas/` | Composition Canvas：实例与结构 Binding |
| `src/explorer/` | 项目目录、composition 文件、模块来源 |
| `src/inspector/` | 实例配置、Slot、Port、依赖 |
| `src/relations/` | Structure/Connection/Dependency 视图投影与 Connection 编辑（024） |
| `src/source/` | TOML Source（CodeMirror） |
| `src/diagnostics/` | 诊断与源码定位 |
| `src/agent-status/` | Agent 会话、Revision、操作记录 |
| `src-tauri/` | 文件访问、窗口、IPC |

**规则**：GUI 不持有权威数据；节点坐标只写 `.designer/`；所有编辑经 Core 命令。

### examples/ 与 docs/

| 路径 | 内容 |
|---|---|
| `examples/` | 合法 / 非法 / 边界项目，兼作 M1 验收数据 |
| `docs/spec/` | 已冻结的领域模型、TOML Schema、错误码 |
| `docs/glossary.md` | 术语表（中文 / English / 反例） |
| `docs/adr/` | 决策与否定方案（0001 冻结、0002 不支持项） |
| `changes/` | Change Manifest |

## 3. 权限模式

| 模式 | 允许改动 | 授权 |
|---|---|---|
| Inspect | 只读、计划、影响分析 | 默认允许 |
| Modify | 目标 crate `src/`、测试、crate 内文档；任务卡指定的 `docs/` 规范与计划 | 默认允许 |
| Compose | `examples/**` 组合声明、`.designer/**`、docs 组合样例 | 需求涉及组合时 |
| Extend | 新 crate / TOML 字段 / 命令 / Adapter / 示例类型 | 需用户明确授权 |
| Rearchitect | 领域模型、Schema 版本、命令协议、工程配置、设计基线 | 需用户明确授权 |

## 4. 标准工作流

1. 从 [`working/plan.md`](../../working/plan.md) 取一张卡；
2. 读本卡相关的规范（`docs/spec/`）与设计基线章节；
3. 复制 `changes/000-template.yaml` 声明范围（跨目录或高一级授权时必须）；
4. 实现：先失败用例，再实现；不猜字段与命令；
5. 验证：按任务卡验收条件；当前先跑 `cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check`；没有门禁就明确写“未执行”；
6. 回写任务卡与 plan。

## 5. 禁止清单

- 未经 Rearchitect 修改设计基线；
- 让 GUI / Agent / CLI 各自维护模型，或绕过 Core 直接写项目文件；
- 在 TOML 中嵌入脚本、命令或可执行逻辑；
- 用隐式 Adapter / 自动修复 / “尽量兼容”处理不兼容；
- 把 Binding、Dependency、Connection 混用；
- 把 `.designer/` 的布局或视口写进 composition；
- 为通过检查增加防御性兼容分支。

## 6. 文件地图

| 要改什么 | 去哪里 |
|---|---|
| 设计基线（Rearchitect） | `docs/design/v0.1-design.md` |
| 评审与路线图 | `docs/reviews/2026-11-27-project-review-and-roadmap.md` |
| 领域模型与术语 | `docs/spec/domain-model-v1.md`、`docs/glossary.md`（已冻结） |
| TOML Schema 与错误码 | `docs/spec/composition-schema-v1.md`、`docs/spec/error-codes-v1.md`（已冻结） |
| 编辑命令 Schema | `docs/spec/command-schema-v1.md`（M2 009） |
| Layout 格式 | `docs/spec/designer-layout-v1.md`（M3 013） |
| 示例项目 | `examples/`（26 个样例，均带 `expected.json`） |
| Core | `crates/composition-domain`（005）、`crates/composition-toml`（006）、`crates/composition-core`（007）、`crates/composition-cli`（008）、`crates/composition-editor`（009） |
| Designer | `apps/designer/src-tauri`（Tauri 2 host，012）、`apps/designer/src`（React shell，012） |
| Agent 协议与 Adapter | `crates/composition-protocol`（016）、`crates/composition-mcp`（017–018）、`agent/`（Skill/MCP 配置） |
| 验收 | `docs/acceptance-v0.1.md` |
| 决策 | `docs/adr/` |
| 变更声明 | `changes/` |
| 计划与任务卡 | `working/` |

## 7. 已冻结与当前进度

- 领域模型、TOML Schema、错误码与样例已在 M0 冻结；决策见
  [ADR-0001](../adr/0001-domain-and-schema-freeze.md) 与
  [ADR-0002](../adr/0002-v01-unsupported.md)；
- M3 012–015 完成：`apps/designer` 的项目/事务/layout/source/external 命令、React
  Explorer/Canvas/Tree/Inspector/Diagnostics、CodeMirror 只读 Source、外部变更轮询/Reload；
  5 个宿主测试；
- M5 019 完成：中断写回恢复、80 实例性能、单入口/版本边界、A01–A18 验收报告；
  v0.1 M0–M5 收口；
- v0.1 显式不支持项见 [`composition-schema-v1.md`](../spec/composition-schema-v1.md) §12；
- v0.1 已冻结；评审建议已归档到 `docs/reviews/2026-11-27-project-review-and-roadmap.md`；
- v0.2 以 M6 Complete Composition Authoring 为第一阶段：020 CI/UI 测试、021 真实项目试点，
  再做 Instance/Binding/Connection/Dependency 编辑与完整 Change Review；
- M6 020 完成：Windows CI、Playwright 1.65 + 系统 Edge、8 条 UI 链路、IPC fixture；
  远端 main 已执行并通过（run 45984567517，5m9s）；
- M6 021 完成：Glide Task + Terminal 只读试点，7 模块/7 实例/6 Binding/6 Connection/3 Dependency，
  `validate` 通过；缺口为接口身份、runtime 容器、Profile、技术元数据；
- M6 022 完成：Designer Instance 创建（Slot 下 + 首个 Binding）、删除子树、重命名、
  换模块/版本与配置调整，全部经 prepare/Review/apply；15 UI + 9 Tauri 测试；
  带必填 Slot 的复合模块子树创建由 023 补齐；
- M6 023 完成：Binding Move（parent/slot）、Detach 诊断、required Slot 递归子树创建；
  18 UI + 12 Tauri 测试；复用 009 冻结命令；
- M6 024 完成：Structure/Connections/Dependencies 三视图共享模型；Connection 创建/
  修改/删除经事务；Dependency 只读投影；23 UI + 14 Tauri 测试；
- M6 025 完成（Extend）：`set_dependency_provider` / `unset_dependency_provider`
  命令、命令 Schema、TOML `[instances.dependencies]` 写回与 Designer 编辑入口；
  154 + 24 UI + 15 Tauri 测试；设计基线未改；
- 2026-11-27 评审整改：授权审计与未实现披露评审归档；G01（Rearchitect，等待授权）
  → G02 → G03 → 026 → G04；未决事项登记 `working/decisions/unresolved-work.md`；
- M6 优先使用现有 Domain/Schema/Commands；试点发现不足再提 Extend/Rearchitect；
- 开始前先取 `working/plan.md` 中的当前卡，不要根据评审建议直接改 Schema 或 UI。
