# Composition Designer — v0.3 执行计划（Post-M6）

> 上一版本计划：[`working/archive/plan-v0.2-m6.md`](archive/plan-v0.2-m6.md)（M6 完成时归档）。
> 设计基线：[`docs/design/v0.1-design.md`](../docs/design/v0.1-design.md)（v0.1 冻结）。
> 外部评审：[项目评审](../docs/reviews/2026-11-27-project-review-and-roadmap.md) ·
> [授权审计](../docs/reviews/2026-11-27-authorization-audit.md) ·
> [未实现披露](../docs/reviews/2026-11-27-unimplemented-work-transparency.md) ·
> [整改计划](../docs/reviews/2026-11-27-audit-response.md)。
> 测试基线：164 Core/Protocol/MCP/CLI + 28 UI + 15 Tauri。
> 当前阶段：**ADR-0004 已确认；028 Source 设计完成（ADR-0005 Proposed）**；下一步 **029 实现需授权**。
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

## 2. 当前基线（2026-11-27）

- M6 完成（020–027）：Instance/Binding/Connection/Dependency 编辑、完整 Change Review、
  Glide 试点全链路、PR 强制门禁；证据见
  [`2026-11-27-m6-closeout.md`](../docs/reviews/2026-11-27-m6-closeout.md)。
- G01–G07 完成：权限集合、历史授权审计、未实现回溯、机器门禁 block；
  G7 已完成接口/服务/Profile 设计研究，ADR-0004 仍为 Proposed；
  `main` 分支保护要求 `Windows build and UI tests`。
- 当前 GUI：Canvas/Tree、Instance/Binding/Config、Connection/Dependency、
  Change Review、**只读** Source、2s 外部变更轮询 + Reload。
- 未决登记：30 项；10 已关闭、8 等待决策、12 延期、0 等待授权。
- 关键缺口：Source 编辑、草稿/外部冲突、撤销/历史持久化、fsync、
  G1 接口身份/G2 runtime 容器/G3 Profile、ConfigValue float wire。

## 3. 路线图

| 阶段 | 目标 | 状态 | 主要交付 | 授权 |
|---|---|---|---|---|
| R0 | 计划重排（G06） | done | v0.3 队列、决策门、验收标准 | Modify |
| G7 | 领域缺口研究（UW-002/003/004） | done | [研究](../docs/reviews/2026-11-27-g7-domain-gaps.md)、[ADR-0004](../docs/adr/0004-schema-v2-direction.md) Accepted | Modify（研究） |
| R1 | M7 Source 设计决议（028） | done | [ADR-0005](../docs/adr/0005-source-editing-transaction.md) Proposed | Modify（研究） |
| M7 | Source Authoring（029–031） | planned | 可编辑 Source、统一事务、草稿冲突、真实项目验收 | Extend + Rearchitect（029） |
| S1 | Schema v2.0（032–035） | pending_decision | G1 接口身份 + G2 service scope；v1 兼容 | Rearchitect + Extend |
| M8 | Agent Authoring（036–038） | later | 基于 v2 的上下文/影响、批处理、Skill 评测 | Extend / Modify |
| S2 | Schema v2.1（039–041） | pending_decision | G3 Profile + 多 Composition 入口 | Rearchitect + Extend |
| H1 | 选择性可靠性（042–046） | conditional | Undo、Revision 持久化、fsync、外部合并、WebView2 smoke | Modify / Extend |
| M9 | Composition Ecosystem（047+） | deferred | 模块库/版本、模板、Adapter | Rearchitect / Extend |

顺序建议：`R0 → G7 → 028 → 029–031 → S1 → M8 → S2 → H1 → M9`。
S1/S2 需要用户对 ADR-0004 的明确 Rearchitect 批准；M9 不能先于 S1/S2。
028 已完成；029 需用户明确批准 ADR-0005（Extend + Rearchitect）。

## 4. 队列

### G7 — 领域缺口研究（done）

- 研究接口身份、service 容器、项目 Profile；
- 结论：v2.0 = G1+G2，v2.1 = G3+多入口；v1 继续可读、不自动迁移；
- ADR-0004 为 **Proposed**，实施仍待用户 Rearchitect 批准；
- 详见 [`2026-11-27-g7-domain-gaps.md`](../docs/reviews/2026-11-27-g7-domain-gaps.md)。

### R1 — 028 M7 Source 设计决议（done）

- 研究完成：[ADR-0005](../docs/adr/0005-source-editing-transaction.md)（Proposed）；
- 推荐：事务内独占 `replace_source`，复用 Revision/幂等/Review；prepare 返回文本+对象 Diff；
- 首版仅 Composition 入口；外部冲突阻止，Reload 丢弃草稿；无三向合并；
- 029 实施需 Extend + Rearchitect 批准。

### M7 — Source Authoring（029–031）

| 卡 | 方向 | 授权 | 依赖 |
|---|---|---|---|
| 029 | Source 编辑事务 + 写回 + 冲突守卫 | Extend + Rearchitect（以 028 ADR 为准） | 028 |
| 030 | CodeMirror 可编辑 Source、草稿、Diff/Review、冲突 UI | Modify | 029 |
| 031 | Source 验收：Glide 试点编辑、外部修改、CLI/MCP 一致 | Modify | 030 |

M7 验收：

- 可编辑 Source，保留注释/格式；
- prepare → 完整 Diff/诊断 → apply；取消/拒绝不写入；
- 磁盘外部修改不会被覆盖；草稿冲突有明确处理；
- 写回后 Revision/Digest/Source/模型一致，重开 valid；
- GUI/CLI/MCP 至少两个入口验证同一事务语义；
- 不建立第二套模型或写入路径。

### S1 — Schema v2.0（032–035，pending_decision）

| 卡 | 方向 | 授权 |
|---|---|---|
| 032 | Schema v2.0 ADR 批准 + 接口/服务语义冻结 | Rearchitect |
| 033 | Domain/Loader/Validation：接口标记、service scope | Rearchitect + Modify |
| 034 | Commands/UI/CLI/MCP：scope 创建、interface 展示 | Extend + Modify |
| 035 | v1/v2 fixtures、迁移拒绝路径、Glide 试点 v2 验收 | Modify |

S1 前置：用户接受 ADR-0004 的 v2.0 方案；不自动改写 v1 项目。

### M8 — Agent Authoring（036–038）

| 卡 | 方向 | 授权 |
|---|---|---|
| 036 | 上下文/影响分析查询（模块、引用、Diff 影响） | Extend |
| 037 | 多命令批处理与流式/长任务；MCP 进度/取消按需 | Extend |
| 038 | Agent Skill 评测与跨平台配置 | Modify / Extend |

### S2 — Schema v2.1（039–041，pending_decision）

| 卡 | 方向 | 授权 |
|---|---|---|
| 039 | Profile/多入口语义冻结与 ADR | Rearchitect |
| 040 | Project/Loader/Revision/Digest 按 profile 选择 | Rearchitect + Extend |
| 041 | CLI/MCP/Designer profile 选择与验收 | Extend + Modify |

S2 前置：S1 完成 + UW-018 多入口决策。

### H1 — 选择性可靠性（042–046，conditional）

- 042 撤销/重做（UW-014）；
- 043 Revision/request 跨进程持久化（UW-015）；
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
| Interface/Contract 独立身份 | UW-002 | Rearchitect | ADR-0004 已确认接口标记；S1（032–035） |
| 非结构 runtime/service 容器 | UW-003 | Rearchitect | ADR-0004 已确认实例 scope；S1 |
| 项目级 Profile / 条件组合 | UW-004 | Rearchitect | ADR-0004 已确认 Profile 入口；S2（039–041） |
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
3. **G1–G3 领域缺口**：G7 已完成研究；真实 runtime/Profile 采用前必须完成 S1/S2；
4. **可靠性缺口**：undo/历史/fsync 未实现，生产使用前需 H1；
5. **单入口/单文件**：S2 前限制仍存在，M9 不再重复多入口。

## 8. 当前未决总览

来源：[`working/decisions/unresolved-work.md`](decisions/unresolved-work.md)。

| 指标 | 数量 |
|---|---:|
| 登记事项 | 30 |
| 已实现 / 已关闭 | 10 |
| 等待用户决策 | 5 |
| 等待授权 | 0 |
| 已接受待排期 | 3 |
| 明确延期 | 12 |

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

## 11. 下一步

028 已完成，ADR-0005 仍为 Proposed。下一步：

1. 等待用户批准 ADR-0005（`replace_source` 独占命令 + 事务复用）；
2. 批准后创建 029（Extend + Rearchitect），再顺序 030/031；
3. S1 Schema v2.0（032–035）在 M7 后启动；ADR-0004 已接受但实施仍逐卡授权。

若用户决定先做 S1，则 028 保持完成、M7 暂停，先执行 032–035。
