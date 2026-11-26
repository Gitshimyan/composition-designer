# AI 架构入口（Bootstrap）

> 给 AI / 新协作者的入口。开始修改前先读本文件、[`working/plan.md`](../../working/plan.md)
> 与设计基线。事实来源是规范、样例与命令输出，不是本文件的文字描述。
> M0 冻结后由 002 / 003 更新本文件。

## 1. 项目速览

| 项 | 值 |
|---|---|
| 项目 | Composition Designer |
| 版本 | v0.1（Draft） |
| 当前阶段 | BOOTSTRAP：无代码、无门禁 |
| 设计基线 | [`composition-designer-v0.1-design.md`](../../composition-designer-v0.1-design.md) |
| 执行顺序 | 设计基线 §16：冻结语义 → 冻结 Schema → 样例 → Core → 编辑事务 → GUI → Agent |
| 技术栈 | Rust Core + `toml_edit`；Tauri 2；React + TypeScript；`@xyflow/react`；CodeMirror 6（设计建议，未冻结） |
| 验证入口 | 暂无；M1 起为 `cargo test` + `composition-cli validate` |
| working 入口 | [`working/plan.md`](../../working/plan.md) |

## 2. 目标结构与依赖方向

### crates/

| crate | 职责 | 允许依赖 |
|---|---|---|
| `composition-domain` | 领域类型、不变量 | 无 |
| `composition-toml` | TOML 读取、Source Map、保留格式修改 | domain |
| `composition-resolver` | 模块解析、规范化结果 | domain, toml |
| `composition-validator` | Schema、引用、契约校验 | domain, resolver |
| `composition-editor` | 命令、事务、Revision、Diff、Undo | domain, resolver, validator, toml |
| `composition-protocol` | 机器可读命令与结果 Schema | domain |
| `composition-app` | 应用服务编排 | 以上全部 |

拆分不要求一次到位：可以先在单个 crate 内按模块隔离，再按依赖方向拆分（决策见 005）。

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
| `docs/spec/` | M0 冻结的领域模型、TOML Schema、错误码 |
| `docs/adr/` | 决策与否定方案 |
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
5. 验证：按任务卡验收条件；没有门禁就明确写“未执行”；
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
| 领域模型与术语 | `docs/spec/domain-model-v1.md`、`docs/glossary.md`（002 建立） |
| TOML Schema 与错误码 | `docs/spec/`（003 建立） |
| 示例项目 | `examples/`（004 建立） |
| Core | `crates/`（M1 起） |
| Designer | `apps/designer/`（M3 起） |
| Agent 协议与 Adapter | `crates/composition-protocol`、M4 的 adapter（未建立） |
| 决策 | `docs/adr/` |
| 变更声明 | `changes/` |
| 计划与任务卡 | `working/` |

## 7. 未冻结事项

见 [`working/plan.md`](../../working/plan.md) §6“待冻结决策”。在 002 / 003 冻结前，
不要根据设计基线里的示例 TOML 猜字段语义。
