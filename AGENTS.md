# AGENTS.md — Composition Designer 工作规则

本文件告诉 AI 如何在本仓库工作。**文档不是强制边界**；真正的阻断由 `cargo` 门禁、
Schema 校验与变更检查承担。门禁建立之前，修改范围由任务卡的 `authorization.required`
与 `changes/` 声明自控，**不要声称执行过尚未存在的验证**。

## 0. 开始任何行动前

1. 读 [`working/plan.md`](./working/plan.md)，只取其中一张任务卡。
2. 读设计基线 [`docs/design/v0.1-design.md`](./docs/design/v0.1-design.md)
   的相关章节，以及 [`docs/spec/`](./docs/spec/README.md) 中与目标对象相关的冻结规范。
3. 读 [`docs/ai/architecture-entry.md`](./docs/ai/architecture-entry.md) 与目标目录的 README，
   确认当前真实状态，不要沿用记忆里的字段或命令。
4. 读 [`working/decisions/unresolved-work.md`](./working/decisions/unresolved-work.md)
   中与本任务相关的 `UW-*` 事项，避免重复发现或静默遗漏。
5. 确认任务卡声明的授权与验收条件现在是否可执行；不可执行就缩小卡的范围并登记未决事项，
   而不是伪造结果。

## 1. 工作模式（权限集合）

权限由**实际变更的影响**决定，不由文件位置、编程语言或任务名称决定。
五种模式是权限集合，不是线性等级；一个任务可以同时需要多个模式，不能用“最高一级”
替代缺失的其他权限。

| 模式 | 权限范围 | 默认规则 |
|---|---|---|
| Inspect | 只读设计基线、规范、声明、计划、源代码与影响分析 | 默认允许 |
| Modify | 在已有领域/Schema/协议契约内，修改已指定 crate、package、app 的实现、测试及任务指定文档 | 普通实现任务默认允许 |
| Compose | 修改 `examples/**` 项目组合声明、`.designer/**` 表现数据、`docs/` 组合样例，以及 `working/pilots/**` 声明式试点 | 需求涉及组合时允许 |
| Extend | 新增 crate、TOML 字段、命令/操作、Adapter、示例类型、局部依赖或既有架构内的新能力 | **需要用户明确授权** |
| Rearchitect | 修改领域模型、对象关系、不变量、Schema 版本/迁移规则、共享命令/事务协议、跨模块架构、workspace 拓扑、全局工程配置、设计基线或 `AGENTS.md` 规则 | **需要用户明确授权** |

### 1.1 组合与审批

1. 任务卡与 Change Manifest 的 `authorization.required` 必须列出实际需要的全部模式；
   `approval` 逐项记录 `granted` / `default` / `pending` 及依据。
2. 任一 required 模式仍为 `pending` 时，不得先实现再补记录；应先报告阻塞与最小方案。
3. 默认授权只覆盖 Inspect 与普通 Modify；Compose 在任务涉及组合声明时允许；
   Extend 与 Rearchitect 必须有用户明确授权或可追溯的批准记录。
4. 获得 Rearchitect **不自动**获得 Extend、Compose、删除/安装/远端调用或 push 权限。
5. 拿不准时按更高权限处理，先停下该部分并报告；不得偷偷越界。
6. 历史任务卡保持原授权记录；审计建议写入审计记录，不追溯改写成“当时已获批准”。

### 1.2 工程配置边界

| 操作 | 授权 |
|---|---|
| 为已批准的新 crate 注册 workspace member | Extend（不改变拓扑/统一构建策略） |
| 新功能所需的 crate/app 局部依赖或脚本 | Extend（不改变全局依赖治理） |
| 改变 workspace 拓扑、crate 分层或跨 crate 依赖方向 | Rearchitect |
| 改变全局工具链、编译目标或构建系统 | Rearchitect |
| 新增辅助 CI workflow 或向既有 workflow 加测试 | Extend / Modify |
| 建立全项目强制 CI、合并门禁或质量策略 | Rearchitect |

### 1.3 文档权限

| 文档操作 | 授权 |
|---|---|
| 回写当前任务卡结果、测试记录、`open_items` | 随任务授权 |
| 更新 `working/plan.md` 状态与下一步 | 随任务授权 |
| 更新 README 功能现状、`docs/ai/architecture-entry.md` 已批准架构的现状 | 随任务授权 |
| 修改 `AGENTS.md` 的授权/工作规则、设计基线 | Rearchitect |
| 新增命令 Schema 文档条目 | Extend |
| 改变领域/Schema/协议语义 | Rearchitect（必要时同时 Extend） |

### 1.4 Compose、依赖与副作用

- Compose 明确覆盖 `working/pilots/**`，但只允许放声明式试点项目及说明；
  不得借试点修改生产源码、模块实现或正式 Schema。
- 新依赖/SDK：若是已授权功能的局部实现手段，随该功能的 Extend；改变全局依赖策略、
  工具链、供应链或安全边界需 Rearchitect。
- Shell、文件删除、安装、远端服务、网络调用、`git push`、发布/部署属于独立副作用，
  需要明确指令或批准；不得从 Extend/Rearchitect 推定获得。
- `git push` 仍只在用户明确要求时执行，见 §3.1。

## 2. 默认禁止

- 未经 required 授权修改设计基线、领域模型、对象关系、Schema 版本、共享命令/事务协议
  或全局工程配置；
- 在 TOML 中嵌入脚本、命令或任意可执行逻辑；
- 让 GUI / Agent / CLI 各自维护项目模型；所有读写必须经 Composition Core；
- 让 GUI 或 Agent 直接写项目文件，绕过解析、验证与事务；
- 用“尽量兼容”、自动 Adapter 或静默修复处理不兼容；
- 把画布坐标、视口、面板状态写进 `composition/` 或任何项目语义文件；
- 把 Dependency 推断为 Binding，或把 Binding 推断为 Connection；
- 在业务模块/UI 中加入旧格式判断、静默 fallback 或“以后可能需要”的兼容分支；
- 为通过检查在下游增加防御性兼容代码；
- 修改非目标 crate / package / app / 目录；
- 把未实现、被阻塞或缩小范围的能力只写在“遗留问题”末尾，或从任务卡中静默删除；
- 用 `status: done` 单独宣称更大功能目标已经全部实现；
- 用宽泛的 `docs/**`、`src/**` target 绕过敏感路径或实际 diff 的声明要求；
- 未经明确指示推送远端、发布或部署。

需要边界变化时，**报告阻塞**并给出最小扩展方案，不要偷偷越界。

## 3. 变更声明与校验

1. 从 [`changes/000-template.yaml`](./changes/000-template.yaml) 复制一份 Change Manifest；
2. 填写 `authorization.required/approved/evidence`、`targets`、`operations`、`scope`、
   `unresolved_items`、`coverage_status`、`impact`、`validation`；
3. 实现前确认影响范围与所需权限；
4. 结卡时逐项核对实际改动文件与声明范围。

`required` 是授权需求，`operations` 是变更类型，`targets` 是文件/目录范围，
`scope` 是允许与禁止的具体行为；四者不可混为一谈。任何未声明的文件、操作或授权不足
都视为越权，不能以“已写 Manifest”代替真实校验。

### 3.1 远端推送规则

默认只在本地 `git commit` 管理历史。**只有用户明确说明“推送到远端/发布到 GitHub”时才执行
`git push`**；阶段评审需要推送时，先确认目标仓库与分支。不得在普通实现、修文档或结卡后
自动推送。

### 3.2 当前校验状态

CI 已运行 `composition-change-gate` 的 **block 模式**（G05）：它比较
`changes/*.yaml` 的 `targets`/授权与 git diff，检查敏感路径、Compose、Extend、
coverage_status 与 `unresolved_items`；声明不一致会让 CI 失败并阻止合并。

一个文件被多个 Manifest 覆盖时，只要任一覆盖 Manifest 满足该文件所需的授权/operation/
forbidden 约束即视为通过。机器门禁只检查声明与 diff 的一致性，不替代 Core 语义校验
或人工评审，也不自动授予权限。

旧 Change Manifest 保留原有单值 `authorization` 格式；新任务使用
`changes/000-template.yaml` 的权限集合格式。历史格式差异由 G02 审计记录，不追溯改写。

## 4. 实现流程

1. 在目标 crate 读 `docs/spec/` 与类型定义，不猜字段与命令；
2. 只使用 Core 暴露的 Command / Query；UI 不重复实现契约规则；
3. 新行为先写失败用例，再实现（M1 起 `cargo test`）；
4. 模块边界与依赖方向遵循 [`crates/README.md`](./crates/README.md)；
5. 若某能力需要未获授权的 Extend/Rearchitect，继续完成不受影响的工作，
   将该项登记为 `blocked_authorization` 或 `pending_authorization`，不得静默移除；
6. 结卡前运行任务卡列出的验收命令；没有命令可跑时，明确写“未执行”；
7. 回写任务卡、`working/plan.md` 与未决工作登记，再进入下一张卡。

## 5. 新增 crate / Schema 字段 / 命令（按影响授权）

- 新 crate：在 `crates/<name>` 建立并注册；在既有 workspace 内注册成员属 Extend，
  改变 workspace 拓扑/统一构建策略需 Rearchitect；同时补
  [`docs/ai/architecture-entry.md`](./docs/ai/architecture-entry.md)；
- 新 TOML 字段：先改 `docs/spec/composition-schema-v1.md` 与错误码，再改校验、样例与 Core；
  兼容的可选字段通常是 Extend，改变类型/必填语义/对象关系/`schema_version` 需 Rearchitect；
- 新命令：先定义输入 Schema、前置条件、结果结构与错误码，再实现；新增命令属 Extend；
  若同时改变共享事务/命令协议语义，需 Rearchitect；
- 新 Adapter：MCP 等只是协议适配器，不得改变 Core 语义；属 Extend；
- 新示例项目：放 `examples/`，必须带 README 与预期诊断；新增示例类型属 Extend，
  编写真实组合声明需同时 Compose。

## 6. 完成定义与需求覆盖

任务状态（`status`）只回答“本卡承诺的交付范围是否完成”，不等于相关能力全部实现。
必须同时记录 `coverage_status` 与 `open_items`：

| coverage_status | 含义 |
|---|---|
| `complete` | 原始目标与验收点均已完成 |
| `complete_with_deferred_items` | 本卡约定范围完成，但仍有明确登记/延期的后续事项 |
| `partial` | 原始目标仍有未完成部分 |
| `blocked_by_authorization` | 核心交付受授权不足阻塞 |

- 任务卡清单勾完、验收条件实际满足（不是“大部分满足”）；
- 变更范围与 Manifest 一致；受影响的规范、样例与测试同步更新；
- 未实现项已登记 `UW-*`，并说明原因、影响、所需授权、最小方案与决策选项；
- 回写 `working/tasks/<card>.md`、`working/plan.md` 与未决登记；
- 不把“没有失败”当成“验证通过”；
- 若核心验收仍受授权阻塞，不能用 `done` + `complete` 掩盖；必须标
  `partial` / `blocked_by_authorization`。
- 旧任务卡不强制重写；再次编辑或进入历史回溯时补 `coverage_status` / `open_items`，
  原始范围、验收与授权字段保持不变。

### 6.1 最终汇报结构

每次任务汇报至少包含：

1. 交付摘要与 `status` / `coverage_status`；
2. 原始目标逐项覆盖（已实现/部分/未实现/延期/拒绝/替代）；
3. **独立**的“未实现 / 被搁置功能”章节；每项写 UW ID、用户缺失能力、当前替代行为、
   原因、影响、所需授权、最小方案、决策选项；没有则明确写“无”；
4. 已执行与未执行验证；
5. 等待授权、已延期与推荐下一步。

不能用“以后再做”“需要 Extend”“暂不处理”单独作为说明。

## 7. 常用命令

```powershell
# M1（005–008）
cargo test --workspace
cargo test -p composition-cli
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p composition-cli -- validate examples/minimal-project --json

# M2（009–011）
cargo test -p composition-editor
cargo run -p composition-cli -- inspect examples/minimal-project

# M3（012–015）
cd apps/designer
npm run setup
npm run build
npm run desktop
npm start

# v0.2 M6（020–025）
cd apps/designer
npm run test:ui
cd ../..

# M4（016–018）
cargo run -p composition-cli -- query examples/minimal-project validate
cargo run -p composition-cli -- jsonl examples/minimal-project
cargo run -p composition-mcp -- --project examples/minimal-project
cargo test -p composition-mcp --test e2e

# M5（019）
cargo test --workspace
cargo test --manifest-path apps/designer/src-tauri/Cargo.toml
cargo test -p composition-mcp --test e2e
# v0.1 验收：docs/acceptance-v0.1.md
```

构建与测试通过宿主机任务（host_task）执行；分析、编辑在沙箱内完成。
`ai.cargo-test` / `ai.cargo-clippy` / `ai.cargo-fmt` 已批准可用；新增命令需再登记。

## 8. 归档与例外

- 过期或否定的方案进 `docs/adr/`，不留在代码注释里“以后再说”；
- 外部评审进 `docs/reviews/`，采纳结论写入任务卡/计划；
- 试点、旧 Schema、临时样例放 `working/tmp/` 或仓库外归档目录，不恢复旧声明文件；
- 任何例外必须在任务卡与 Change Manifest 中写明范围、责任与移除条件。

## 9. 未实现工作与未决登记

登记册：[`working/decisions/unresolved-work.md`](./working/decisions/unresolved-work.md)；
决策历史：[`working/decisions/decision-history.md`](./working/decisions/decision-history.md)。

- 每个未实现/被阻塞/延期事项必须有稳定 `UW-*` ID，不得只存在于汇报末尾；
- 至少区分 `blocked_authorization`、`deferred`、`out_of_scope`、`not_implemented`、
  `rejected`、`superseded`、`implemented`；
- 状态使用 `pending_decision`、`pending_authorization`、`blocked_authorization`、
  `deferred`、`accepted`、`in_progress`、`implemented`、`rejected`、`superseded`；
- 用户未回复不等于批准、拒绝或延期；`pending_authorization` 不得自动转成 `deferred`；
- 只有实现并有证据、用户明确拒绝、被其他事项明确替代或用户明确不再需要时才可关闭；
- 授权阻塞提案必须给出：当前能力、缺失能力、原因、授权类型、最小范围、不包含内容、
  推荐方案、替代方案、验收证据与决策选项；
- 计划页必须维护未决事项总览，阶段结项必须汇总未实现项，不能只总结已完成任务。
