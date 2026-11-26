# AI 架构入口

> 给 AI / 新协作者的入口。开始修改前先读本文件、[`working/plan.md`](../../working/plan.md)
> 与设计基线。事实来源是规范、样例与命令输出，不是本文件的文字描述。
> M0 已冻结（2026-11-26）；字段与错误码以 [`docs/spec/`](../spec/) 为准。

## 1. 项目速览

| 项 | 值 |
|---|---|
| 项目 | Composition Designer |
| 版本 | v0.1（Draft） |
| 当前阶段 | M1 完成：`composition-domain`（58）、`composition-toml`（27）、`composition-core`（8）、`composition-cli`（8）；M2 编辑事务未开始 |
| 设计基线 | [`composition-designer-v0.1-design.md`](../../composition-designer-v0.1-design.md) |
| 冻结规范 | [`docs/spec/`](../spec/)（domain / schema / error codes） |
| 测试数据 | [`examples/`](../../examples/README.md)（每个样例带 `expected.json`） |
| 执行顺序 | 设计基线 §16：冻结语义 → 冻结 Schema → 样例 → Core → 编辑事务 → GUI → Agent |
| 技术栈 | Rust Core + `toml_edit`；Tauri 2；React + TypeScript；`@xyflow/react`；CodeMirror 6（设计建议，未冻结） |
| 验证入口 | `cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check`；CLI：`cargo run -p composition-cli -- validate examples/minimal-project --json` |
| working 入口 | [`working/plan.md`](../../working/plan.md) |

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
| `composition-editor` | 待 M2 | 命令、事务、Revision、Diff、Undo | domain, core, toml |
| `composition-protocol` | 待 M4 | 机器可读命令与结果 Schema | domain |
| `composition-app` | 待 M3/M4 | 应用服务编排 | core |

005 建立 `composition-domain`，006 建立 `composition-toml`，007 建立 `composition-core`，
008 建立 `composition-cli`；resolver/validator 暂并入 core，其余在 M2+ 按依赖方向加入。

### apps/designer/

| 区域 | 职责 |
|---|---|
| `src/canvas/` | Composition Canvas：实例与结构 Binding |
| `src/explorer/` | 项目目录、composition 文件、模块来源 |
| `src/inspector/` | 实例配置、Slot、Port、依赖 |
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
| 设计基线（Rearchitect） | `composition-designer-v0.1-design.md` |
| 领域模型与术语 | `docs/spec/domain-model-v1.md`、`docs/glossary.md`（已冻结） |
| TOML Schema 与错误码 | `docs/spec/composition-schema-v1.md`、`docs/spec/error-codes-v1.md`（已冻结） |
| 示例项目 | `examples/`（26 个样例，均带 `expected.json`） |
| Core | `crates/composition-domain`（005）、`crates/composition-toml`（006）、`crates/composition-core`（007）、`crates/composition-cli`（008） |
| Designer | `apps/designer/`（M3 起） |
| Agent 协议与 Adapter | `crates/composition-protocol`、M4 的 adapter（未建立） |
| 决策 | `docs/adr/` |
| 变更声明 | `changes/` |
| 计划与任务卡 | `working/` |

## 7. 已冻结与当前进度

- 领域模型、TOML Schema、错误码与样例已在 M0 冻结；决策见
  [ADR-0001](../adr/0001-domain-and-schema-freeze.md) 与
  [ADR-0002](../adr/0002-v01-unsupported.md)；
- M1 005 已建立 `composition-domain`（58 测试）；006 已建立 `composition-toml`（27 测试）；
  007 已建立 `composition-core`（8 测试，26/26 样例匹配）；008 已建立 `composition-cli`
  （inspect/resolve/validate、JSON、退出码、4 个 golden，8 测试）；M1 完成，编辑事务未开始；
- v0.1 显式不支持项见 [`composition-schema-v1.md`](../spec/composition-schema-v1.md) §12；
- 尚未冻结的是 M2+ 实现细节：编辑器 crate 拆分、命令 Schema、Revision 持久化与协议适配；
  开始前先取对应任务卡，不要根据设计基线里的示例 TOML 猜字段语义。
