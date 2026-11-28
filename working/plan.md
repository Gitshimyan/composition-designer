# Composition Designer — v0.3 执行计划（Post-M6）

> 上一版本计划：[`working/archive/plan-v0.2-m6.md`](archive/plan-v0.2-m6.md)（M6 完成时归档）。
> 设计基线：[`docs/design/v0.1-design.md`](../docs/design/v0.1-design.md)（v0.1 冻结）。
> 外部评审：[项目评审](../docs/reviews/2026-11-27-project-review-and-roadmap.md) ·
> [授权审计](../docs/reviews/2026-11-27-authorization-audit.md) ·
> [未实现披露](../docs/reviews/2026-11-27-unimplemented-work-transparency.md) ·
> [整改计划](../docs/reviews/2026-11-27-audit-response.md)。
> 测试基线：241 Core/Protocol/MCP/CLI + 36 UI + 17 Tauri。
> 当前阶段：S1、M8、S2 完成；**H1 042 undo/redo 完成**；下一步 043（需单独授权）。
> 推进规则：一次只推进一张任务卡；推送必须经过 PR 与 block 门禁，且只在用户明确要求时执行。

## 1. 下一阶段目标

v0.3 的目标不是继续堆叠 UI 功能，而是把 **Source 变成与 Inspector 对等的第一类编辑入口**，
随后把 Agent 的上下文/影响分析/批处理能力补齐；可靠性与生态能力按明确决策推进。

```text
Source 编辑（保留格式 + 完整事务 + 冲突处理）
  → Schema v2.0（接口身份 + service scope）
  → Agent 上下文/影响/批处理
  → Schema v2.1（Profile + 多入口）
  → 选择性可靠性加固
  → 生态能力（模块库/模板，条件承诺）
```

非目标：

- 不承诺运行时、构建、部署或模块市场；
- 不自动展开 G1–G3 等 Rearchitect 领域模型变更；
- 不把“Review 已存在”当成“Source 编辑已完成”。

## 2. 当前基线（2026-11-28）

- M6 完成（020–027）：Instance/Binding/Connection/Dependency 编辑、完整 Change Review、
  Glide 试点全链路、PR 强制门禁；证据见
  [`2026-11-27-m6-closeout.md`](../docs/reviews/2026-11-27-m6-closeout.md)。
- G01–G08 完成：权限集合、历史授权审计、未实现回溯、机器门禁 block；
  G7 已完成接口/服务/Profile 设计研究，ADR-0004 为 Accepted；
  `main` 分支保护要求 `Windows build and UI tests`。
- 当前 GUI：Canvas/Tree、Instance/Binding/Config、Connection/Dependency、
  Change Review、**可编辑** Source（草稿/Review/冲突只读）、2s 外部变更轮询 + Reload。
- G09 完成：修正 M7 后文档漂移（Source 可编辑、测试基线 30 UI / 16 Tauri、
  ADR-0004 Accepted）。
- S1 032 完成：v2.0 语义冻结（接口标记、接口依赖、实例 scope、8 个新诊断、
  `create_instance.scope`、v1 兼容矩阵）；ADR-0006 Accepted；无代码变更。
- S1 033 完成：Domain/Loader/Validation 实现接口声明/接口解析、service scope 与结构规则、
  8 个诊断、版本矩阵；v1 诊断与 JSON 输出不变；workspace 189 测试。
- S1 034 完成：`create_instance.scope` 与 TOML 写回、Services 视图/service 创建、
  Inspector scope/interface、Dependency 接口目标；CLI/MCP v2 事务；workspace 196 /
  UI 34 / Tauri 17。
- S1 035 完成：11 个 v2 数据集（37 个 expected.json）、迁移拒绝路径、Glide v2 试点
  （7 实例 / 3 Binding / 6 Connection / 3 接口依赖）与 CLI 编辑验收；UW-002/003 关闭。
- M8 036–038 完成：context/impact、只读 batch、Skill 评测与 configure.mjs；220 workspace。
- S2 039 完成：v2.1（format 3）Profile/多入口语义冻结（ADR-0007、v3 规范与 3 个诊断）；
  UW-004/UW-018 转 in_progress；无代码变更。
- S2 040 完成：Domain v3、Loader 按 profile 选择、Core 校验/Revision/Digest、
  Session/Source 作用域；核心 11 + editor 2 测试；v1/v2.0 不变。
- S2 041 完成：CLI/MCP/Designer profile 入口、v3 数据集（40 总数）、S2 验收；
  UW-004/UW-018 关闭；238 workspace + 35 UI + 17 Tauri。
- H1 042 完成：Editor undo/redo（文本快照 + 新 Revision）、Tauri IPC、Designer 按钮/快捷键；
  UW-014 关闭；241 workspace + 36 UI + 17 Tauri。
- 未决登记：32 项；16 已关闭、5 等待决策、0 已接受待排期、0 进行中、11 延期、0 等待授权。
- 关键缺口：H1 043–046（Revision 持久化、fsync、外部合并、WebView2 smoke）、
  ConfigValue float wire、模块文件编辑（UW-031）。

## 3. 路线图

| 阶段 | 目标 | 状态 | 主要交付 | 授权 |
|---|---|---|---|---|
| R0 | 计划重排（G06） | done | v0.3 队列、决策门、验收标准 | Modify |
| G7 | 领域缺口研究（UW-002/003/004） | done | [研究](../docs/reviews/2026-11-27-g7-domain-gaps.md)、[ADR-0004](../docs/adr/0004-schema-v2-direction.md) Accepted | Modify（研究） |
| R1 | M7 Source 设计决议（028） | done | [ADR-0005](../docs/adr/0005-source-editing-transaction.md) Accepted | Modify（研究） |
| M7 | Source Authoring（029–031） | done | Source 事务、UI、Review、Glide 验收 | Extend + Rearchitect（029） |
| S1 | Schema v2.0（032–035） | done | 语义冻结、Core、命令/UI/入口、v2 数据集与 Glide 验收 | Rearchitect + Extend |
| M8 | Agent Authoring（036–038） | done | 上下文/影响查询、只读 batch、Skill 评测与跨平台配置 | Extend / Modify |
| S2 | Schema v2.1（039–041） | done | Profile + 多 Composition 入口；按 profile 的 Revision/Digest | Rearchitect + Extend |
| H1 | 选择性可靠性（042–046） | in_progress | Undo、Revision 持久化、fsync、外部合并、WebView2 smoke | Modify / Extend |
| M9 | Composition Ecosystem（047+） | deferred | 模块库/版本、模板、Adapter | Rearchitect / Extend |

顺序建议：`R0 → G7 → 028 → 029–031 → S1 → M8 → S2 → H1 → M9`。
M7 已完成；S1 032 已冻结 v2.0 语义，033–035 仍需逐卡授权；M9 不能先于 S1/S2。

## 4. 队列

### G7 — 领域缺口研究（done）

- 研究接口身份、service 容器、项目 Profile；
- 结论：v2.0 = G1+G2，v2.1 = G3+多入口；v1 继续可读、不自动迁移；
- ADR-0004 为 **Accepted**；032 已冻结 v2.0 语义（ADR-0006），033–035 实施仍需逐卡授权；
- 详见 [`2026-11-27-g7-domain-gaps.md`](../docs/reviews/2026-11-27-g7-domain-gaps.md)。

### R1 — 028 M7 Source 设计决议（done）

- 研究完成：[ADR-0005](../docs/adr/0005-source-editing-transaction.md)（Accepted）；
- 推荐：事务内独占 `replace_source`，复用 Revision/幂等/Review；prepare 返回文本+对象 Diff；
- 首版仅 Composition 入口；外部冲突阻止，Reload 丢弃草稿；无三向合并；
- 029–031 已完成；首版边界见 M7 验收记录。

### M7 — Source Authoring（029–031）

| 卡 | 方向 | 授权 | 状态 |
|---|---|---|---|
| 029 | Source 编辑事务 + 写回 + 冲突守卫 | Extend + Rearchitect | done |
| 030 | CodeMirror 可编辑 Source、草稿、Diff/Review、冲突 UI | Modify | done |
| 031 | Source 验收：Glide 试点编辑、外部修改、CLI/MCP 一致 | Modify | done |

M7 验收：

- 可编辑 Source，保留注释/格式；
- prepare → 完整 Diff/诊断 → apply；取消/拒绝不写入；
- 磁盘外部修改不会被覆盖；草稿冲突有明确处理；
- 写回后 Revision/Digest/Source/模型一致，重开 valid；
- GUI/CLI/MCP 至少两个入口验证同一事务语义；
- 不建立第二套模型或写入路径。

### S1 — Schema v2.0（032–035，done）

| 卡 | 方向 | 授权 | 状态 |
|---|---|---|---|
| 032 | Schema v2.0 ADR 批准 + 接口/服务语义冻结 | Rearchitect + Extend | done |
| 033 | Domain/Loader/Validation：接口标记、service scope | Rearchitect + Modify | done |
| 034 | Commands/UI/CLI/MCP：scope 创建、interface 展示 | Extend + Modify | done |
| 035 | v1/v2 fixtures、迁移拒绝路径、Glide 试点 v2 验收 | Modify + Compose | done |

032 交付：[ADR-0006](../docs/adr/0006-schema-v2-freeze.md)、
[composition-schema-v2.md](../docs/spec/composition-schema-v2.md)、
[error-codes-v2.md](../docs/spec/error-codes-v2.md)；v2.0 新增 8 个诊断代码；
接口只做身份标记；service 必须 primitive 且不参与 Binding；命令只扩
`create_instance.scope`；v1 不自动迁移。

033 交付：接口声明/接口依赖/scope 的领域类型、TOML 解码、版本矩阵与 L2–L7 校验
（含接口解析、service 结构规则、`service-unused`）；v1 诊断与 JSON 输出不变；
`crates/composition-core/tests/v2.rs` 14 条 v2 用例；不做命令/UI/示例。

034 交付：`create_instance.scope`（v1 显式传 scope 被拒）、保留格式写回 `scope`；
Services 视图与 service 创建（仅 primitive）；Inspector scope/interface；Dependency 接口
目标与候选；CLI/MCP v2 inspect/validate/apply；Playwright 34、Tauri 17、workspace 196。

035 交付：11 个 v2 数据集（总计 37 个 expected.json）、迁移拒绝路径；Glide v2 试点
（4 structure + 3 service、3 接口依赖、6 Connection）与 CLI 编辑验收；
UW-002/UW-003 关闭。S1 收口。

### M8 — Agent Authoring（036–038，done）

目标：让 Agent 用结构化上下文/影响查询规划编辑，支持批量读取与可重复评测的 Skill；
不改变 Core 写事务语义。

| 卡 | 交付 | 授权 | 状态 |
|---|---|---|---|
| 036 | protocol/CLI/MCP `context` + `impact`（module / instance / transaction dry-run） | Extend + Rearchitect（协议文档） | done |
| 037 | protocol/CLI/MCP `batch`（只读、限流、逐项结果）；progress/cancel 不在本阶段 | Extend + Rearchitect（协议文档） | done |
| 038 | Agent eval harness（Node，跨平台）、`configure.mjs`、Skill 文档与评测记录 | Modify + Extend | done |

交付：

- 036：`context` / `impact`（module、instance、transaction prepare-only），CLI/MCP 与协议文档；
- 037：只读 `batch`（1..16、写操作拒绝、逐项 Response），CLI/MCP 与协议文档；
- 038：`composition-mcp/tests/skill_eval.rs`（minimal + Glide v2）、`agent/configure.mjs`、
  `agent/EVALUATION.md`；
- 收口：220 workspace 测试；M8 验收全部满足；UW-020 仍 deferred。

M8 验收：

- `context` 输出项目/模块索引/实例索引/诊断/Revision，不复制完整 Config；
- `impact` 覆盖 module、instance、transaction(dry-run)，与 Diff/诊断一致；
- `batch` 只接受只读 op（context/impact/inspect/resolve/validate/source），有上限与逐项错误；
- eval harness 在 `examples/minimal-project` 与 Glide v2 试点上可重复通过；
- CLI/MCP 复用 protocol，不维护第二套模型；不新增写命令。

非目标：

- 不实现 MCP progress/cancel/subscription/auth（UW-020 仍 deferred）；
- 不改变事务、Diff、Revision 或 TOML 写回语义；
- 不做远程/多项目 Agent 调度。

### S2 — Schema v2.1（039–041，done）

目标：用项目级 Profile 表达 desktop/headless/环境差异，每个 profile 是完整 Composition；
共享模块，不做 overlay/继承。格式版本与选择语义由 039 冻结。

| 卡 | 交付 | 授权 | 状态 |
|---|---|---|---|
| 039 | ADR + 规范：Profile 表、`schema_version` 矩阵、选择/Revision/Digest、错误码、CLI/MCP/Designer 语义 | Rearchitect + Extend | done |
| 040 | Domain/Loader/Core/Session：Project profiles、按 profile 加载/校验/Revision/Digest | Rearchitect + Modify | done |
| 041 | CLI/MCP/Designer profile 选择、Source/Review 按 profile；fixtures 与验收 | Extend + Modify + Rearchitect（协议文档） | done |

交付：

- 039：[ADR-0007](../docs/adr/0007-schema-v21-profiles.md)、
  [composition-schema-v3.md](../docs/spec/composition-schema-v3.md)、
  [error-codes-v3.md](../docs/spec/error-codes-v3.md)；format 3、`[profiles]`、
  默认/显式选择、Revision/Digest 作用域、3 个新代码；无代码变更。
- 040：`SchemaVersion` v3、`Project.profiles/default/active`、`load_project_with_profile`、
  `validate_project_with_profile`、`ProjectSession::open_with_profile`；
  3 个诊断；按 profile 的 Digest/Source；v1/v2.0 输出不变。
- 041：CLI `--profile`（全命令）、MCP `--profile`、Tauri `open_project(path, profile?)`、
  Designer 顶栏选择器、v3 数据集 3 组；CLI 4 + MCP 1 + Playwright 1 测试。

S2 验收（全部满足）：

- 默认 profile 保持 `workspace.composition`；显式选择缺失 profile 报稳定错误；
- 每个 profile 独立 validate；模块集合共享；
- Revision/Digest 只覆盖所选 Composition + 共享模块，切换 profile 不静默沿用旧 Revision；
- 事务/Source 只写所选 profile 入口；不做 overlay、继承或自动迁移；
- v1/v2.0 文件保持可读，v3 显式启用；
- CLI/MCP/Designer 均可选择 profile；3 组 v3 数据集与 UI 选择器验收通过。

非目标：

- Profile overlay/继承、per-profile module roots、运行时选择；
- 自动迁移 v1/v2.0 → v3；
- 远程/多项目调度。

### H1 — 选择性可靠性（042–046，in_progress）

- 042 撤销/重做（UW-014）— **done**：会话文本快照、新 Revision、按钮/快捷键；
- 043 Revision/request 跨进程持久化（UW-015）— 下一步，需单独授权；
- 044 fsync/崩溃恢复（UW-019）；
- 045 外部 watcher/三向合并/草稿保留（UW-017）；
- 046 真实 WebView2 smoke（UW-012）。

按真实使用反馈排序，不默认全部进入下一版；每张卡单独授权。

### M9 — Ecosystem（047+，deferred）

- 模块库/版本发现与下载（UW-022）；
- 模板/Adapter（UW-013 的后续部分）；
- 多入口已移入 S2，不在 M9 重复；
- 仅在 S1/S2 完成且出现明确生态需求时重新评估。

## 5. 决策门（不自动实现）

| 事项 | UW | 类型 | 计划 |
|---|---|---|---|
| Interface/Contract 独立身份 | UW-002 | Rearchitect | implemented（032–035；Glide v2 验收） |
| 非结构 runtime/service 容器 | UW-003 | Rearchitect | implemented（实例 scope + primitive；Glide v2 验收） |
| 项目级 Profile / 条件组合 | UW-004 | Rearchitect | 039 冻结 format 3；040–041 实现 |
| 多 Composition 入口 | UW-018 | Rearchitect | 039 冻结 `[profiles]`；040–041 实现 |
| 接口方法与 service 内部组合 | UW-032 | Rearchitect | v2.0 明确排除；出现真实需求时评估 |
| ConfigValue 整数值 float wire | UW-005 | Extend/协议 | M8 或独立小卡；不阻塞 Source 文本编辑 |
| Inspector array 编辑 | UW-016 | Modify | M7 UI 的 quick win 或独立小卡 |
| Binding 向导 UX | UW-023 | Modify | M8 后按反馈 |
| 示例载荷/边界测试 | UW-026/027 | Compose/Modify | 维护队列，随相关改动补 |

## 6. 验收与质量门

- 每个阶段必须有真实项目或可重复集成验收；
- 所有写操作经 Core/ProjectSession；Source 编辑也不得绕过；
- 每个任务卡记录 `coverage_status` / `open_items` / 未实现项；
- CI block 门禁要求 Manifest 与 diff 一致；所需授权缺失会失败；
- 推送只通过 PR，required check `Windows build and UI tests` 必须通过。

## 7. 风险与依赖

1. **Source 事务复杂度**：先 028 设计，避免把“可编辑文本框”当成 Source 能力；
2. **草稿/外部冲突**：整项目 digest 只能保守检测，三向合并可能超出首版；
3. **G3 Profile**：G7 已完成研究，S2 实施；G1/G2 已由 S1 交付；
4. **可靠性缺口**：undo 已实现；历史跨进程持久化/fsync 未实现，生产使用前需 H1 043–044；
5. **单入口/单文件**：S2 已支持多入口；模块文件仍单文件（UW-031 deferred）。

## 8. 当前未决总览

来源：[`working/decisions/unresolved-work.md`](decisions/unresolved-work.md)。

| 指标 | 数量 |
|---|---:|
| 登记事项 | 32 |
| 已实现 / 已关闭 | 16 |
| 等待用户决策 | 5 |
| 等待授权 | 0 |
| 已接受待排期 | 0 |
| 进行中 | 0 |
| 明确延期 | 11 |

## 9. 常用命令

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo test --manifest-path apps/designer/src-tauri/Cargo.toml
cargo test -p composition-mcp --test e2e

cd apps/designer
npm run build
npm run test:ui
npm run desktop
```

构建、测试、桌面启动通过 `host_task` 执行；分析、编辑在沙箱内完成。

## 10. 日志

| 日期 | 变更 | 结果 |
|---|---|---|
| 2026-11-27 | M6 | 020–027 完成；Glide 全链路、PR 门禁、change-gate block |
| 2026-11-27 | G05 | UW-030 关闭；CI block 模式启用 |
| 2026-11-27 | R0/G06 | v0.3 计划重排；M7 设计决议为下一卡；旧计划归档 |
| 2026-11-27 | G07 | G1/G2/G3 设计研究；ADR-0004 获用户确认，状态 Accepted；S1/S2 排入路线图 |
| 2026-11-27 | 028 | Source 编辑设计；ADR-0005 Proposed；029 需 Extend + Rearchitect 授权 |
| 2026-11-27 | 029 | replace_source 后端、source prepare/apply、文本+对象 Diff、冲突守卫；172 tests |
| 2026-11-27 | 030 | CodeMirror 可编辑 Source、Review、Discard、冲突只读；30 UI |
| 2026-11-27 | 031 | Glide 试点 Source 编辑验收；173 tests / 16 Tauri / 30 UI；M7 完成；UW-011 关闭，模块文件编辑登记 UW-031 |
| 2026-11-27 | G08 | M7 覆盖审计：UW-011 implemented、UW-031 deferred；31 项登记 |
| 2026-11-28 | G09 | 文档一致性：Source 可编辑、30 UI / 16 Tauri、ADR-0004 Accepted；无代码变更 |
| 2026-11-28 | 032 | Schema v2.0 语义冻结：ADR-0006、接口/scope/诊断/命令/兼容矩阵；8 个新代码；无代码变更 |
| 2026-11-28 | 033 | Domain/Loader/Validation：接口声明/解析、service scope、8 个诊断、版本矩阵；189 tests；v1 输出不变 |
| 2026-11-28 | 034 | create_instance.scope、Services 视图/service 创建、接口展示；CLI/MCP v2；196 workspace / 34 UI / 17 Tauri |
| 2026-11-28 | 035 | 11 个 v2 数据集（37 总数）、迁移拒绝、Glide v2 试点与 CLI 验收；UW-002/003 关闭；S1 完成 |
| 2026-11-28 | M8 | Agent Authoring 开始：036 context/impact 查询；037 batch；038 eval/跨平台配置 |
| 2026-11-28 | 036 | context/impact：protocol/CLI/MCP、协议与 Skill；210 tests；只读 |
| 2026-11-28 | 037 | 只读 batch：protocol/CLI/MCP、限流与逐项结果；218 tests；无 progress/cancel |
| 2026-11-28 | 038 | Skill 评测（minimal + Glide v2）、configure.mjs；220 tests；M8 完成 |
| 2026-11-28 | 039 | v2.1 语义冻结：format 3、Profile/多入口、选择/Revision/Digest、3 个新代码；无代码变更 |
| 2026-11-28 | 040 | Domain v3、Loader/Core profile 选择、Revision/Digest、Session/Source 作用域；233 tests |
| 2026-11-28 | 041 | CLI/MCP/Designer profile 入口、v3 数据集、S2 验收；238 tests + 35 UI + 17 Tauri；UW-004/018 关闭 |
| 2026-11-28 | 042 | Editor/Designer undo/redo、Tauri IPC、快捷键；241 tests + 36 UI + 17 Tauri；UW-014 关闭 |

## 11. 下一步

M7/S1/M8/S2 完成，H1 进行中。下一步：

1. **043 — Revision/request 跨进程持久化**（UW-015）：需单独授权（Modify + Extend）；
2. 044/045/046 按真实反馈排序；每张卡单独授权；
3. M9 生态按需；UW-020（progress/cancel）、UW-031（模块文件编辑）、
   UW-032（接口形状）仍 deferred；
4. 建议 043 之前先确认持久化位置与格式（仓库内 `.designer/` 或外部状态目录）。

035 开始前确认：是否需要把 034 的 UI 创建入口纳入试点操作脚本；若需，先写进 035 卡。
