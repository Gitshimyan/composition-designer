# Composition Designer 授权体系与任务卡审计

> 审计日期：2026-11-27  
> 审计对象：`Gitshimyan/composition-designer` 公开仓库 `main` 分支当前内容  
> 审计范围：工作模式定义、`AGENTS.md`、任务卡模板、Change Manifest、001–025 全部任务卡，以及最新完成任务卡 025 的授权回答与实现声明  
> 审计性质：设计与变更记录审查；不代表已在本地重新构建或执行仓库测试。

## 1. 执行摘要

当前五种工作模式（Inspect / Modify / Compose / Extend / Rearchitect）方向正确，但现行定义还不足以完整约束项目中的实际变更。主要问题不是所有历史任务都存在越权，而是**部分任务卡把领域、Schema 或协议基线的建立/改变归入了普通 Modify 或 Extend，另有少数任务的 Change Manifest 没有准确覆盖实际文件与变更类型**。

本次审计结论：

1. **明确需要重新分类的任务卡：002、003、009、010、016。** 它们分别涉及领域模型、TOML Schema、编辑事务契约、事务历史/幂等语义以及 Agent/CLI 通信协议。
2. **需要按“工程配置”边界作条件性分类的任务卡：005、012、020。** 如果任务改变全局 workspace 拓扑、宿主工程边界或项目级 CI 质量门禁，应加入 Rearchitect；若只是既有架构内增加成员、依赖或测试，则 Extend / Modify 足够。
3. **需要补全组合权限的任务卡：004。** 它同时新增示例类型并编写具体组合声明，应是 Extend + Compose，而不是只有 Extend。
4. **需要补充路径授权的任务卡：021。** Compose 定义应覆盖声明式试点目录 `working/pilots/**`，否则当前 Compose 权限定义与实际试点路径不一致。
5. **最新任务卡 025 的 Provider 扩展决策正确。** 它准确地将新增 Provider 命令归为 Extend，并且没有借此修改领域模型、TOML Schema 或设计基线。但 025 的 Change Manifest 漏列了实际修改的 `crates/composition-protocol/src/lib.rs` 测试文件。
6. **当前 Change Manifest 机制仍是书面契约，而不是自动门禁。** `AGENTS.md` 已明确说明当前没有自动变更校验，因此不能把“Manifest 已声明”误当作系统已阻止越权。

建议先修订授权规范、任务卡模板和 Manifest 模板，再补充历史审计记录，最后完成 026 Change Review 与机器校验门禁。不要因本次审计直接扩展 Schema、领域模型或启动 M7。

## 2. 审计依据与判断方法

### 2.1 主要依据

- [`AGENTS.md`](https://github.com/Gitshimyan/composition-designer/blob/main/AGENTS.md)：工作模式、默认禁止、变更声明、推送规则。
- [`working/plan.md`](https://github.com/Gitshimyan/composition-designer/blob/main/working/plan.md)：当前阶段、任务状态、路线图与下一步。
- [`working/tasks/000-template.md`](https://github.com/Gitshimyan/composition-designer/blob/main/working/tasks/000-template.md)：任务卡模板。
- [`docs/design/v0.1-design.md`](https://github.com/Gitshimyan/composition-designer/blob/main/docs/design/v0.1-design.md)：设计基线。
- [`docs/spec/domain-model-v1.md`](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/domain-model-v1.md)：领域对象与关系语义。
- [`docs/spec/composition-schema-v1.md`](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/composition-schema-v1.md)：TOML Schema。
- [`docs/spec/command-schema-v1.md`](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/command-schema-v1.md)：命令与事务契约。
- [`docs/spec/protocol-v1.md`](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/protocol-v1.md)：Agent/CLI 协议。
- [`docs/adr/0003-dependency-provider-command.md`](https://github.com/Gitshimyan/composition-designer/blob/main/docs/adr/0003-dependency-provider-command.md)：025 Provider Extend 决策。

审计也核对了 001–025 任务卡及对应的 `changes/*.yaml`。历史任务卡缺少 Manifest 的，不据此伪造当时的授权证据；只记录本次审计建议的分类和补正方式。

### 2.2 判定原则

授权按**实际改变的对象、语义和契约**判断，不按文件类型、目录名称或工作量判断。

例如：

- 改 `docs/` 不必然是 Modify；如果文档冻结领域模型或 Schema，仍然需要 Rearchitect / Extend。
- 新建 crate 不必然只需要 Extend；如果同时改变全局 workspace 拓扑或依赖方向，还需要 Rearchitect。
- 修改 `src/` 不必然是 Modify；若改变共享命令协议、领域对象或不变量，应升级授权。
- 一个任务可以同时需要多个授权模式。五种模式不是严格的单一阶梯，不应以“最高一级”替代权限集合。
- 本表是按现有任务内容给出的审计建议。任务发生时是否已获得用户明确授权，必须由当时的对话或审批记录证明；不能通过事后修改任务卡倒填授权历史。

## 3. 001–025 全部任务卡授权审计

### 3.1 总表

| 任务卡 | 当前任务卡授权 | 审计建议 | 结论 | 主要原因 |
|---|---|---|---|---|
| 001 Bootstrap | Rearchitect | Rearchitect | 合理 | 初始化工作规则、目录骨架、工程管理结构与权限体系。 |
| 002 Domain Model | Modify | **Rearchitect** | **等级不足** | 冻结领域对象、关系语义、对象不变量、Root/Binding 结构与 Revision 语义。 |
| 003 TOML Contract Schema | Modify | **Extend + Rearchitect** | **等级不足** | 创建 TOML 字段/契约，同时确定 Schema 版本策略、结构与校验规则。 |
| 004 Example Projects | Extend | **Extend + Compose** | **权限不完整** | 新增示例类型，也在 `examples/**` 中编写真实项目组合声明。 |
| 005 Rust Workspace / Domain | Extend | **Extend；全局 workspace 基线部分另需 Rearchitect** | **条件性遗漏** | 新 crate 属 Extend；建立 Cargo workspace、resolver、edition 等全局工程基线，触及工程配置。 |
| 006 TOML Loader | Extend | Extend | 合理 | 新 crate 与解析依赖；没有改变 Schema 语义。 |
| 007 Composition Core | Extend | Extend + Modify | 基本合理 | 新 Core crate，同时补充既有 Domain 实现；所补 `effective_config` 已由冻结规范要求，不应误判为领域重设计。 |
| 008 Composition CLI | Extend | Extend | 基本合理 | 新 CLI crate 与只读命令入口；未改变共享编辑事务协议。 |
| 009 Editor Transaction | Extend | **Extend + Rearchitect** | **等级不足** | 新增编辑命令属 Extend；首次定义 Command / Transaction 形状与原子提交语义，涉及命令协议。 |
| 010 Revision / Diff | Modify | **Modify + Rearchitect** | **等级不足** | 增加 Revision 历史、幂等、重放、冲突和 Diff 返回契约，改变事务协议语义。 |
| 011 TOML Writeback | Modify | Modify | 合理 | 在既有命令和事务语义内实现持久化、格式保留、digest 刷新与回滚。 |
| 012 Designer Shell | Extend | **Extend；workspace/宿主工程边界部分需 Rearchitect** | **条件性遗漏** | 新桌面应用属 Extend；将 Tauri app 排除出根 workspace、采用独立 lock/build 边界，触及工程架构。 |
| 013 Canvas / Tree | Extend | Extend | 合理 | 新 UI 能力、布局持久化和 Tauri 局部操作；未改变共享领域模型。 |
| 014 Inspector / Source | Extend | Extend + Modify；协议字段若属破坏性改变再评估 Rearchitect | **需明确契约边界** | 新 UI/IPC 能力属 Extend；同时修改已有 Domain/Editor 序列化实现与测试。增加 `id` 若只是兼容性增强字段，不必自动升为 Rearchitect。 |
| 015 External Sync | Extend | Extend；若改变共享事务协议则另需 Rearchitect | **需明确契约边界** | 新检测/Reload 能力属 Extend；当前记录显示主要在 Tauri Host 增加外部变更守卫，复用既有冲突错误，不足以单独证明修改了共享命令协议。 |
| 016 Protocol / CLI JSONL | Extend | **Extend + Rearchitect** | **等级不足** | 新 Protocol crate 属 Extend；建立 Agent/CLI Request、Response、Operation、错误和 JSONL 通信契约，属于命令协议。 |
| 017 MCP Adapter | Extend | Extend | 合理 | 将既有 Protocol 适配为 MCP tools/resources，没有改变 Core/Protocol 语义。 |
| 018 Agent Skill / E2E | Extend | Extend | 合理 | 新增 Agent Skill、MCP 配置与端到端验证，没有改变共享协议。 |
| 019 Integrity / Acceptance | Modify | Modify | 合理 | 在冻结 Schema 内补恢复、性能、边界测试与验收报告。 |
| 020 Quality Gates | Extend | **Extend；建立项目级强制质量门禁时另需 Rearchitect** | **条件性遗漏** | 新 CI workflow 和测试工具属 Extend；若其定义全局构建/合并门禁策略，则触及工程配置。普通增加已有 workflow 的测试项仍可为 Modify。 |
| 021 Glide Pilot | Compose + Modify | Compose + Modify | 权限类型合理，路径定义不足 | 试点声明是 Compose，分析和计划文档是 Modify；但 Compose 当前未覆盖 `working/pilots/**`。 |
| 022 Instance Editing | Modify | Modify | 合理 | 复用已冻结 12 条编辑命令、事务、Core 校验与 TOML 写回。 |
| 023 Binding Editing | Modify | Modify | 合理 | 复用 `create_binding` / `update_binding` / `delete_binding`，没有新命令或新语义。 |
| 024 Relation Views | Modify | Modify | 合理 | Connection 复用冻结命令；Dependency Provider 编辑被明确留作 Extend 缺口。 |
| 025 Dependency Provider | Extend + Modify | Extend + Modify | 正确 | 新增 Provider 命令属于 Extend，UI、事务、写回与测试属于 Modify；未修改领域模型或 TOML 字段。 |

### 3.2 必须优先修订的任务卡

#### 002 — 领域模型与术语冻结

当前标注为 `Modify（仅 docs/）`，这是最明确的授权分类遗漏。

本卡实际冻结了 Project、Module Definition、Instance、Binding、Dependency、Port、Connection、Diagnostic、Revision 等对象的身份、引用、不变量和关系语义，也确定了单父 Binding 树、依赖环及 Composite / Primitive 规则。这些内容正属于 Rearchitect 的领域模型和对象关系范围。

**建议审计记录：** `required: [rearchitect]`。由于这是初始领域定义，不应以“只改文档”为由降级。

#### 003 — TOML 与 Module Contract Schema 冻结

当前标注为 `Modify（仅 docs/）`，建议调整为 `Extend + Rearchitect`。

- Extend：建立 TOML 字段、Module Contract 字段和契约声明能力。
- Rearchitect：确定声明文件结构、Schema 版本策略、字段语义、校验层级与错误码体系。

以后应区分“增加字段”与“改变已有语义”：增加明确兼容的新可选字段通常是 Extend；改变字段类型、必填语义、对象关系或 `schema_version` / 迁移规则则是 Rearchitect。

#### 004 — 示例项目集

当前只标注 Extend，但本卡实际创建合法、非法和边界项目组合声明。建议 `Extend + Compose`。

这不是因为目录中存在 TOML 就自动需要 Compose，而是因为这些 TOML 是要交给系统解析的项目组合实例，而不是普通测试配置。

#### 009 — 编辑命令与事务

当前只标注 Extend。创建 `composition-editor` 和新增命令属于 Extend；但本卡同时首次确定 `Command`、`Transaction`、`base_revision`、原子候选模型和提交语义，属于命令/事务协议基线，建议增加 Rearchitect。

#### 010 — Revision、幂等与 Diff

当前标注 Modify，且 `changes/010-revision-diff.yaml` 声明 `expected_command_changes: true`，形成权限与影响声明不一致的信号。

Revision 历史、请求重放、重复 request、冲突处理、Diff 的 Before/After 和 TransactionOutcome 都是调用方需要依赖的契约。建议 `Modify + Rearchitect`。如果未来将 Rearchitect 的“命令协议”定义为仅限网络线协议，则必须另行明确事务语义的归属；本审计建议将它纳入命令/事务协议。

#### 016 — Agent/CLI Protocol

当前只标注 Extend，但本卡正式建立 Query/Command Request、Response、协议错误、JSONL 会话和外部冲突语义。建议 `Extend + Rearchitect`。

新增 Protocol crate 和 CLI 功能是 Extend；通信协议本身的创建与冻结是 Rearchitect。未来增加遵守既有协议的 MCP 或其他 Adapter，不应因此自动需要 Rearchitect。

### 3.3 条件性任务卡：避免过度升级

005、012、014、015、020 不建议在缺乏具体证据时一律标记为 Rearchitect。应按以下细则处理：

- **005：** 增加 `composition-domain` 是 Extend；首次建立根 Cargo workspace、resolver、edition 等全局工程基线属于 Rearchitect。若只是在已经冻结的 workspace 内注册新成员，则 Extend 足够。
- **012：** 新建 Tauri/React 应用是 Extend；改变根 Cargo workspace 纳入/排除策略、全局构建边界或宿主架构时增加 Rearchitect。
- **014：** 新 UI / IPC 操作是 Extend，既有实现和测试更新是 Modify。新增一个可选、向后兼容的序列化字段不必然构成 Rearchitect；改变已有调用方依赖的字段语义、协议版本或不兼容响应形状时才升级。
- **015：** 新增外部变更检测、Reload 和 Host 内部守卫通常是 Extend；如果它改变 Core/Protocol 层对 `prepare` / `apply` 的统一接受规则或错误契约，则需要 Rearchitect。
- **020：** 新增 workflow、Playwright 和 fixture 是 Extend；建立全项目必须遵守的构建/测试/合并门禁属于工程策略，需 Rearchitect。只向已冻结 workflow 加普通测试覆盖则 Modify 足够。

## 4. 最新任务卡 025 评审

最新已完成任务卡是 [025 — M6 Dependency Provider 编辑](https://github.com/Gitshimyan/composition-designer/blob/main/working/tasks/025-m6-dependency-provider.md)。`working/plan.md` 当前将 026 — 完整 Change Review 列为下一步；本次审计未发现 026 已有独立任务卡文件。

### 4.1 025 的核心授权判断正确

024 已明确说明：v0.1 冻结的 12 条编辑命令中没有设置 Dependency Provider 的命令。若在 UI 直接写 TOML，会绕过 Core 事务与统一校验；因此 Provider 编辑必须新增命令并通过统一链路实现。

025 新增：

```json
{ "type": "set_dependency_provider", "instance": "engine", "requirement": "logger", "provider": "logger-1" }
{ "type": "unset_dependency_provider", "instance": "engine", "requirement": "logger" }
```

并实现：

```text
Designer
   ↓
prepare_transaction / Review
   ↓
composition-editor Transaction
   ↓
ProjectSession
   ↓
TOML [instances.dependencies] writeback
   ↓
Core validation / Revision refresh
```

这一决策符合授权边界：

- 新增编辑命令和命令 Schema 条目：Extend。
- UI、事务应用、TOML 写回、测试与文档落地：Modify。
- 不修改领域对象、TOML 字段、`schema_version`、设计基线：无需 Rearchitect。

[ADR-0003](https://github.com/Gitshimyan/composition-designer/blob/main/docs/adr/0003-dependency-provider-command.md) 已记录用户授权、替代方案及影响范围，处理方式正确。

### 4.2 025 对历史缺口的审计不够完整

025 的“背景与授权审计”主要覆盖 021–024 的遗留问题，重点识别 G1–G8、ConfigValue wire、CI 和 Source 编辑边界。这对解决 Provider 缺口是足够的，但不能替代对 001–020 历史任务的全量授权审计。

因此，025 的正确结论应限定为：**它补齐了 024 明确记录的 Extend 缺口；并未完成整个项目历史授权体系审计。** 本文件第 3 节补足了这项全量审计。

### 4.3 025 的 Change Manifest 存在确定的目标文件遗漏

提交 [`e32b871`](https://github.com/Gitshimyan/composition-designer/commit/e32b871897fbd0982266867548e7582a0d1cb1a0) 实际修改文件包括：

```text
crates/composition-protocol/src/lib.rs
```

该文件增加了 Provider transaction 的 Protocol 集成测试，但 `changes/025-dependency-provider.yaml` 的 `targets` 没有登记 `crates/composition-protocol`，`scope.allowed` 也没有明确登记该 Protocol 测试。

建议补充：

```yaml
targets:
  - crates/composition-protocol

scope:
  allowed:
    - protocol_transaction_test
```

这是 Manifest 的声明覆盖遗漏，不代表 Protocol 生产实现发生了越权修改。该文件的实际改动是测试，应按 Modify 记录，不需要 Rearchitect。

### 4.4 025 当前验证记录的结论

任务卡记录的验收包括 workspace 测试、Tauri host 测试、前端 build、Playwright、clippy 和 fmt；提交信息也明确说明没有改变领域模型、TOML Schema 或设计基线。本次审计没有重新运行这些测试，因此只确认仓库记录的完整性和变更声明，不将其表述为本次独立复测结果。

## 5. 当前工作模式存在的系统性问题

### 5.1 五种模式不是严格的单一等级阶梯

目前的 Inspect / Modify / Compose / Extend / Rearchitect 更像不同的变更权限集合，而不是只能选择一个的线性等级。例如：

- Modify + Compose：修改 Designer 实现并调整项目组合声明。
- Extend + Modify：增加命令，同时修改既有 Editor、Host、UI 和测试。
- Extend + Rearchitect：新增模块能力，同时改变领域或协议基线。

因此，任务卡不应只写“取所需最低一项”，Manifest 也不应只允许一个 `authorization` 字符串。应声明任务**需要哪些权限**，并分别记录这些权限是默认授权还是由用户明确批准。

### 5.2 “工程配置”边界过于宽泛

建议明确区分：

| 操作 | 建议授权 | 边界 |
|---|---|---|
| 为已批准的新 crate 注册 workspace member | Extend | 不改变 workspace 拓扑或统一构建策略 |
| 新功能所需的 crate 局部依赖 | Extend | 不改变全局依赖治理策略 |
| 新增应用局部依赖、脚本或测试配置 | Extend | 不改变全局工程约定 |
| 改变 workspace 拓扑、crate 分层或跨 crate 依赖方向 | Rearchitect | 改变工程架构 |
| 改变全局工具链、编译目标或构建系统 | Rearchitect | 改变全局工程基线 |
| 新增 CI workflow | Extend | 若只是辅助执行既有检查 |
| 建立全项目强制 CI / 合并门禁策略 | Rearchitect | 改变全局质量与交付规则 |
| 在既有 CI 中添加普通测试用例 | Modify | 不改变门禁策略 |

### 5.3 管理性文档没有清楚的默认规则

实际任务经常需要更新：

```text
README.md
AGENTS.md
docs/ai/
crates/README.md
working/
changes/
```

建议明确：任务范围内的结果记录、计划状态、README 功能说明、架构入口现状可随任务授权更新；但这些更新不得借机改变更高权限的规则或基线。

| 文档操作 | 建议授权 |
|---|---|
| 回写当前任务卡结果、测试记录 | 随任务授权 |
| 更新 `working/plan.md` 状态和下一步 | 随任务授权 |
| 更新 README 功能现状 | 随任务授权 |
| 更新 `docs/ai/architecture-entry.md` 中已批准架构的现状 | 随任务授权 |
| 修改 `AGENTS.md` 的授权或工作规则 | Rearchitect |
| 修改设计基线 | Rearchitect |
| 新增命令 Schema 文档条目 | Extend |
| 改变领域/Schema/协议语义 | Rearchitect，必要时同时 Extend |

### 5.4 Compose 路径范围不完整

当前 Compose 定义覆盖 `examples/**`、`.designer/**` 和 `docs/` 组合样例，但 021 实际在 `working/pilots/glide-task-terminal` 建立声明式试点。

建议增加：

```text
working/pilots/**
```

该路径只允许存放声明式试点项目及其说明，不应成为修改生产源码、模块实现或正式 Schema 的通道。

### 5.5 新依赖与外部副作用应独立判定

建议补充：

- 新依赖 / SDK：若是已授权功能的局部实现手段，可随该功能的 Extend；改变全局依赖策略、工具链、供应链或安全边界需 Rearchitect。
- Shell、文件删除、安装、远端服务调用等副作用应有独立的执行边界；不能因为任务获得 Extend 或 Rearchitect，就推定获得任意外部副作用权限。
- `git push`、发布和部署仍按独立规则处理。当前 `AGENTS.md` 的“只有用户明确要求推送才执行”规则应保留。

## 6. 建议的最终授权规范

以下内容建议作为 `AGENTS.md` §1 的修订基础。

### 6.1 五种工作模式

| 模式 | 授权范围 | 默认规则 |
|---|---|---|
| Inspect | 只读设计基线、规范、声明、计划、源代码与影响分析 | 默认允许 |
| Modify | 在已有领域/Schema/协议契约内，修改已指定 crate、package、app 的实现、测试及任务指定文档 | 普通实现任务默认允许 |
| Compose | 修改 `examples/**` 项目组合声明、`.designer/**` 表现数据、`docs/` 组合样例，以及 `working/pilots/**` 声明式试点 | 需求涉及组合时允许 |
| Extend | 新增 crate、字段、命令/操作、Adapter、示例类型、依赖或既有架构内的新能力 | 需要用户明确授权 |
| Rearchitect | 修改领域模型、对象关系、不变量、Schema 版本/迁移规则、共享命令/事务协议、跨模块架构、workspace 拓扑、全局工程配置与设计基线 | 需要用户明确授权 |

### 6.2 模式组合与升级规则

1. 授权按集合声明，不将五种模式视为线性等级；一个任务可以同时需要多个模式。
2. `required` 列出实际变更所需的全部权限；`approved` 或审批记录逐项证明这些权限已获授权。
3. 同一任务既修改实现又修改组合声明时，必须同时具备 Modify 与 Compose。
4. 新增能力但不改变已有领域/协议基线时，使用 Extend；若还改变基线，必须同时获得 Rearchitect。
5. 对不确定的变更按较高权限要求处理；先停下该部分并给出最小扩展方案。
6. 不因文件位于 `docs/`、`src/`、`tests/` 或 `working/` 就自动降低权限。
7. 不因获得 Rearchitect 就自动获得所有 Extend、Compose 或副作用执行权限。

### 6.3 默认禁止

以下规则应继续保留并强化：

- 未经 Rearchitect 授权修改设计基线、领域模型、对象关系、Schema 版本或共享事务/命令协议。
- 未经 Extend 授权新增 crate、TOML 字段、命令、操作、Adapter 或示例类型。
- 让 GUI / Agent / CLI 各自维护项目模型，或绕过 Composition Core / ProjectSession 直接写项目语义文件。
- 在 TOML 中嵌入脚本、命令或任意可执行逻辑。
- 通过隐式 Adapter、静默 fallback 或防御性兼容补丁掩盖不兼容。
- 把 Binding、Dependency、Connection 混用。
- 将 `.designer/` 表现数据写入 Composition 语义文件。
- 修改非目标 crate / package / app / 目录。
- 未经明确指示推送远端、发布或部署。

## 7. 任务卡与 Change Manifest 模板建议

### 7.1 任务卡模板

当前模板的 `授权：Modify | Compose | Extend | Rearchitect（取所需最低一项）` 不支持多模式任务。建议改为：

```yaml
authorization:
  required:
    - extend
    - modify
  approval:
    extend:
      status: pending # granted | pending | not_required
      basis: "用户明确授权的对话/决策记录"
    modify:
      status: default
      basis: "普通实现任务默认允许"
```

任务卡应在开始实现前填入 `required`；需要明确授权的模式没有 `granted` 证据时，不得先实现再补记录。

### 7.2 Change Manifest 模板

建议把单值：

```yaml
authorization: modify
```

调整为权限集合：

```yaml
authorization:
  required:
    - extend
    - modify
  evidence:
    - level: extend
      basis: "用户于 YYYY-MM-DD 明确授权新增该命令"
    - level: modify
      basis: "既有实现范围内的默认修改权限"
```

并继续维护：

```yaml
operations:
  - extend
  - modify

targets:
  - crates/composition-editor
  - apps/designer/src

scope:
  allowed:
    - new_editor_command
    - editor_transaction
    - designer_ui
    - tests
  forbidden:
    - domain_object_model
    - toml_schema_fields
    - schema_version
    - design_baseline

impact:
  expected_schema_changes: false
  expected_command_changes: true
  expected_design_baseline_changes: false
```

说明：`required` 是授权需求，`operations` 是变更类型，`targets` 是文件/目录范围，`scope` 是允许与禁止的具体行为；四者不可混为一谈。

## 8. Change Manifest 的其他已发现问题

| 任务 | 发现 | 建议 |
|---|---|---|
| 010 | `authorization: modify`，但 `expected_command_changes: true`，实际增加事务历史、幂等与 Diff 契约 | 调整授权为 Modify + Rearchitect，并明确协议变更边界。 |
| 020 | 任务卡写明生产 IPC 语义未改变，但 Manifest 的 `expected_command_changes: true` | 若实际只新增 CI / 测试工具，应改为 `false`；若有实际 IPC 命令变化，应在 targets/scope 和任务卡列明。 |
| 021 | Compose 声明位于 `working/pilots/**`，当前 Compose 目录规则没有覆盖该路径 | 更新 `AGENTS.md` Compose 范围，并保留只读试点、禁止生产代码的限制。 |
| 025 | 实际修改 `crates/composition-protocol/src/lib.rs` 测试，但 Manifest 没有对应 target / scope | 补 `crates/composition-protocol` 与 `protocol_transaction_test`。 |
| 002–004 | 任务卡未配套独立 Change Manifest | 作为 M0 初始化历史保留，不伪造当时文件；在审计记录中补充分类，不追溯声称已通过 Manifest 校验。 |

此外，`AGENTS.md` 目前声明“没有自动变更校验”，所以现阶段这些差异仍需要人工核对。机器门禁建立前，不能宣称越权已经被系统自动阻断。

## 9. 后续整改计划

建议按以下顺序执行，一次只推进一张任务卡。

| 顺序 | 工作项 | 主要交付 | 建议授权 |
|---|---|---|---|
| 1 | 修订 `AGENTS.md` | 五种模式边界、多模式组合、工程配置、文档权限、Compose 路径、副作用边界 | Rearchitect |
| 2 | 修订任务卡模板与 Manifest 模板 | `required` 权限集合、审批依据、target/scope/operations 分离 | Rearchitect |
| 3 | 补充 001–025 历史授权审计记录 | 每张卡的现有授权、审计建议、授权证据状态、Manifest 差异 | Modify（记录）+ Rearchitect（规则变更，若同卡执行） |
| 4 | 修正明确的 Manifest 问题 | 010、020、021、025 声明修正；不伪造旧授权证据 | Modify；涉及规则调整则另需 Rearchitect |
| 5 | 推进 026 Change Review | Before/After、对象类别、级联影响、完整诊断与确认/取消 | Modify；如扩展共享 Diff / 事务协议则另需 Rearchitect |
| 6 | 建立机器授权门禁 | 对比 Manifest 与实际 Diff；未声明路径、操作或授权不足时失败 | Extend + Rearchitect（建立新门禁能力并改变工程门禁策略） |

### 9.1 机器门禁的最低要求

未来的授权门禁至少应检查：

1. 每个变更文件都落在 Manifest 的 `targets` 内；
2. 每种实际操作都有对应 `operations` / `scope.allowed` 声明；
3. 修改设计基线、领域模型、Schema 版本、共享命令/事务协议、全局工程配置时，Manifest 明确要求 Rearchitect；
4. 新 crate、字段、命令、Adapter、新示例类型时，Manifest 明确要求 Extend；
5. 变更组合声明或表现数据时，Manifest 明确要求 Compose；
6. 所需的 Extend / Rearchitect 审批存在可追溯依据；
7. `scope.forbidden` 中的路径或行为未发生；
8. 未声明变更不能通过“补一个宽泛的 `docs/**` 或 `src/**` target”绕过，应对敏感基线路径设置明确规则。

门禁应先以报告模式运行，验证误报和规则覆盖，再切为阻断模式；但在阻断模式启用前，必须如实标明它仍是人工审查。

## 10. 最终结论

1. 五种工作模式应保留，但要从单值等级调整为可组合的权限集合。
2. 002、003、009、010、016 是本轮最明确的授权分类遗漏，应优先补充历史审计记录。
3. 005、012、020 的 Rearchitect 需求取决于其是否改变全局工程架构或质量策略，不应不加区分地扩大权限。
4. 004 应补 Compose；021 的 `working/pilots/**` 应正式纳入 Compose 路径范围。
5. 025 的 Provider Extend 决策正确，没有发现其核心实现借机改变领域模型或 TOML Schema；但 Manifest 必须补列 Protocol 测试文件。
6. 当前授权声明尚未形成自动门禁。下一步应先修订规则和模板，再逐步建立可审计、可执行的变更校验。

**核心原则：权限由变更影响决定，而不是由文件位置、实现语言或任务名称决定；新增能力与改变既有基线是两种不同授权，必须分别声明。**

---

## 附录 A：相关仓库入口

- [仓库主页](https://github.com/Gitshimyan/composition-designer)
- [AGENTS.md](https://github.com/Gitshimyan/composition-designer/blob/main/AGENTS.md)
- [working/plan.md](https://github.com/Gitshimyan/composition-designer/blob/main/working/plan.md)
- [任务卡目录](https://github.com/Gitshimyan/composition-designer/tree/main/working/tasks)
- [Change Manifest 目录](https://github.com/Gitshimyan/composition-designer/tree/main/changes)
- [Command Schema](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/command-schema-v1.md)
- [025 Provider 提交](https://github.com/Gitshimyan/composition-designer/commit/e32b871897fbd0982266867548e7582a0d1cb1a0)
