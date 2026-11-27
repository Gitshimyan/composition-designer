# Composition Designer — v0.3 执行计划（Post-M6）

> 上一版本计划：[`working/archive/plan-v0.2-m6.md`](archive/plan-v0.2-m6.md)（M6 完成时归档）。
> 设计基线：[`docs/design/v0.1-design.md`](../docs/design/v0.1-design.md)（v0.1 冻结）。
> 外部评审：[项目评审](../docs/reviews/2026-11-27-project-review-and-roadmap.md) ·
> [授权审计](../docs/reviews/2026-11-27-authorization-audit.md) ·
> [未实现披露](../docs/reviews/2026-11-27-unimplemented-work-transparency.md) ·
> [整改计划](../docs/reviews/2026-11-27-audit-response.md)。
> 测试基线：164 Core/Protocol/MCP/CLI + 28 UI + 15 Tauri。
> 当前阶段：**R0 计划重排完成**；下一步 **028 — M7 Source 设计决议**。
> 推进规则：一次只推进一张任务卡；推送必须经过 PR 与 block 门禁，且只在用户明确要求时执行。

## 1. 下一阶段目标

v0.3 的目标不是继续堆叠 UI 功能，而是把 **Source 变成与 Inspector 对等的第一类编辑入口**，
随后把 Agent 的上下文/影响分析/批处理能力补齐；可靠性与生态能力按明确决策推进。

```text
Source 编辑（保留格式 + 完整事务 + 冲突处理）
  → Agent 上下文/影响/批处理
  → 选择性可靠性加固
  → 生态能力（多文件/模块库/模板，条件承诺）
```

非目标：

- 不承诺运行时、构建、部署或模块市场；
- 不自动展开 G1–G3 等 Rearchitect 领域模型变更；
- 不把“Review 已存在”当成“Source 编辑已完成”。

## 2. 当前基线（2026-11-27）

- M6 完成（020–027）：Instance/Binding/Connection/Dependency 编辑、完整 Change Review、
  Glide 试点全链路、PR 强制门禁；证据见
  [`2026-11-27-m6-closeout.md`](../docs/reviews/2026-11-27-m6-closeout.md)。
- G01–G05 完成：权限集合、历史授权审计、未实现回溯、机器门禁、
  change-gate **block** 模式；`main` 分支保护要求 `Windows build and UI tests`。
- 当前 GUI：Canvas/Tree、Instance/Binding/Config、Connection/Dependency、
  Change Review、**只读** Source、2s 外部变更轮询 + Reload。
- 未决登记：30 项；10 已关闭、8 等待决策、12 延期、0 等待授权。
- 关键缺口：Source 编辑、草稿/外部冲突、撤销/历史持久化、fsync、
  G1 接口身份/G2 runtime 容器/G3 Profile、ConfigValue float wire。

## 3. 路线图

| 阶段 | 目标 | 状态 | 主要交付 | 授权 |
|---|---|---|---|---|
| R0 | 计划重排（G06） | done | v0.3 队列、决策门、验收标准 | Modify |
| R1 | M7 Source 设计决议（028） | next | ADR：Source 事务/命令/冲突/CLI/MCP 边界 | Modify（研究） |
| M7 | Source Authoring（029–031） | planned | 可编辑 Source、统一事务、草稿冲突、真实项目验收 | Extend + Rearchitect（029） |
| M8 | Agent Authoring（032–034） | later | 上下文/影响分析、批处理/流式、Skill 评测 | Extend / Modify |
| H1 | 选择性可靠性（035–039） | conditional | Undo、Revision 持久化、fsync、外部合并、WebView2 smoke | Modify / Extend |
| M9 | Composition Ecosystem（040+） | deferred | 模块库/版本、多文件、模板、Adapter | Rearchitect / Extend |
| G7 | 领域缺口决策（G1–G3） | decision gate | 接口身份、runtime 容器、项目 Profile | Rearchitect |

顺序建议：`R0 → R1 → 029 → 030 → 031 → M8 → H1 → G7/M9`。
G7 可在 M7 验收后提前；M9 不因本计划自动进入承诺。

## 4. 队列

### R1 — 028 M7 Source 设计决议（next）

- 目标：决定 Source 编辑的事务形状，而不是直接写代码；
- 产出 ADR + 任务卡：
  - Source 编辑是整文件替换、区域 patch，还是命令化 `replace_source`；
  - 命令/协议是否需要扩展；Revision/Digest/幂等如何保持；
  - CodeMirror 草稿与磁盘冲突、外部修改、Review/Apply/Cancel 语义；
  - CLI/MCP 如何执行同一 Source 事务；
  - 是否必须修改 `command-schema-v1.md` / 设计基线；
- 完成前不创建 029 的实现。

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

### M8 — Agent Authoring（032–034）

| 卡 | 方向 | 授权 |
|---|---|---|
| 032 | 上下文/影响分析查询（模块、引用、Diff 影响） | Extend |
| 033 | 多命令批处理与流式/长任务；MCP 进度/取消按需 | Extend |
| 034 | Agent Skill 评测与跨平台配置 | Modify / Extend |

### H1 — 选择性可靠性（035–039，conditional）

- 035 撤销/重做（UW-014）；
- 036 Revision/request 跨进程持久化（UW-015）；
- 037 fsync/崩溃恢复（UW-019）；
- 038 外部 watcher/三向合并/草稿保留（UW-017）；
- 039 真实 WebView2 smoke（UW-012）。

按真实使用反馈排序，不默认全部进入下一版；每张卡单独授权。

### M9 — Ecosystem（040+，deferred）

- 模块库/版本发现与下载（UW-022）；
- 多 composition 入口/多文件（UW-018）；
- 模板/Profile-scoped task 等（UW-013）；
- 仅在出现明确生态需求时重新评估，不自动承诺。

## 5. 决策门（不自动实现）

| 事项 | UW | 类型 | 计划 |
|---|---|---|---|
| Interface/Contract 独立身份 | UW-002 | Rearchitect | G7；M7 后或真实项目提出时 |
| 非结构 runtime/service 容器 | UW-003 | Rearchitect | 与 UW-002 联合设计 |
| 项目级 Profile / 条件组合 | UW-004 | Rearchitect | G7；需真实 profile 需求 |
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
3. **G1–G3 领域缺口**：M7 不解决；真实 runtime 采用前必须走 G7；
4. **可靠性缺口**：undo/历史/fsync 未实现，生产使用前需 H1；
5. **单入口/单文件**：M9 前限制仍存在。

## 8. 当前未决总览

来源：[`working/decisions/unresolved-work.md`](decisions/unresolved-work.md)。

| 指标 | 数量 |
|---|---:|
| 登记事项 | 30 |
| 已实现 / 已关闭 | 10 |
| 等待用户决策 | 8 |
| 等待授权 | 0 |
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

## 11. 下一步

**028 — M7 Source 设计决议**：

1. 读设计基线 §7/§8、命令协议、`ProjectSession` 写回与 SourceMap；
2. 产出 ADR，明确 Source 事务/冲突/CLI/MCP 边界；
3. 经用户确认授权后，才创建 029 实现卡。

如果优先目标改为“真实 runtime/生态”而不是 Source，先执行 G7 决策门并重排 M7/M9。
