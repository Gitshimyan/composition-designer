# Composition Designer — v0.2 执行计划

> 当前设计基线：[`docs/design/v0.1-design.md`](../docs/design/v0.1-design.md)（Draft，v0.1 冻结）  
> 外部评审：[`docs/reviews/2026-11-27-project-review-and-roadmap.md`](../docs/reviews/2026-11-27-project-review-and-roadmap.md)  
> 测试基线：146 Core/Protocol/MCP/CLI + 5 Tauri 测试  
> 当前阶段：v0.2 计划中；M6 未开工  
> 推进规则：一次只推进一张任务卡；只有用户明确要求时才推送远端，平时只做本地 commit。

## 1. 目标

v0.2 的目标是 **Complete Composition Authoring**：

> 让人类用户和 Agent 都能通过明确、可验证、可审查的方式，完成一个项目的完整声明式组合设计。

重点不是增加运行时、构建、部署或模块市场，而是把已经存在的领域命令完整、安全地交付到
Designer、Source 与 Agent 入口。

## 2. 当前状态（2026-11-27）

- v0.1 M0–M5 已冻结并通过 A01–A18 验收；
- 评审已归档；v0.1 领域模型、TOML Schema、命令协议不因 v0.2 UI 需求直接修改；
- 当前 GUI：Canvas/Tree 查看、Inspector 配置编辑、Change Review、只读 Source、外部变更 Reload；
- 缺口：Instance/Binding/Connection/Dependency 完整编辑、Source 编辑、CI/UI 自动化；
- 当前没有 M6 代码改动；020 卡处于 planned。

## 3. 路线图

| 阶段 | 目标 | 状态 | 主要交付 |
|---|---|---|---|
| M6 | Complete Composition Authoring | planned | CI、真实项目试点、Instance/Binding/Connection/Dependency、完整 Review |
| M7 | Source Authoring & Consistency | later | 可编辑 Source、统一提交、外部修改/草稿冲突 |
| M8 | Agent Authoring Experience | later | 上下文查询、影响分析、事务批处理、Skill 评测 |
| M9 | Composition Ecosystem | deferred | Module Registry/Versioning、多文件、模板、Adapter；不承诺进入 v0.2 |

## 4. M6 队列

| 顺序 | 卡 | 方向 | 依赖 | 状态 |
|---|---|---|---|---|
| 1 | 020 | 自动化质量门禁与 UI 测试基础 | 无 | planned |
| 2 | 021 | 真实项目试点与 Schema 复核 | 020；需用户提供代表性子系统 | planned |
| 3 | 022 | Instance 编辑 | 021 结论 | queued |
| 4 | 023 | Binding 编辑 | 022 | queued |
| 5 | 024 | Connection / Dependency 视图 | 022–023 | queued |
| 6 | 025 | 完整 Change Review | 022–024 | queued |

### 020 — 自动化质量门禁与 UI 测试基础

交付：

- Windows-first GitHub Actions workflow；
- `cargo test` / clippy / fmt、Tauri host tests；
- TypeScript 检查、Vite production build；
- MCP e2e；
- 前端 UI 自动化基础与首批关键链路；
- 本地可运行的 `npm run test:ui`（即使尚未推送到 GitHub，也要本地通过）。

UI 测试选型在卡内确认：优先 Playwright + 受控 IPC fixture，覆盖前端行为；Tauri 真实宿主
仍由 host tests 与手动 smoke 覆盖。若真实 WebView2 自动化成本可控，再补 tauri-driver smoke。

### 021 — 真实项目试点

需要一个真实项目的代表性子系统，完整声明 Module/Instance/Binding/Dependency/Connection；
先验证现有 Schema，不先加字段。输出表达能力不足清单与 Extend / Rearchitect 建议。
此卡在用户提供试点项目前不能开始。

### 022–025 — 完整编辑链

- Instance：创建/删除/重命名/换模块与版本/配置；
- Binding：Slot 感知的父子结构编辑；
- Connection / Dependency：三个独立视图，共享同一 Composition Model；
- Change Review：Before/After、级联影响、完整诊断、确认/取消。

所有操作必须经 `composition-editor` 事务；GUI 不复制 Slot/Port/Dependency 规则。

## 5. 决策与约束

1. **v0.1 冻结**：M6 优先用现有 Domain/Schema/Commands；试点发现不足再走 Extend/Rearchitect。
2. **单一权威模型**：Designer、Source、CLI、MCP 都经 Core/ProjectSession；不建第二模型/写入路径。
3. **三类关系分离**：Binding、Dependency、Connection 不画成同一种无类型边。
4. **一个修改意图，一个完整事务**：相关命令批量；不同意图分开；prepare 后审查整体 Diff。
5. **每阶段独立验收**：M6 未完成前不启动 M7/M8；M9 不进入 v0.2 承诺。
6. **推送规则**：默认只本地 commit；只有用户明确说“推送到远端/发布”时才 push。
   阶段性评审推送前先确认仓库与分支。

## 6. M6 总验收链路

```text
选择模块 → 创建 Instance → 建立 Binding → 配置 Property
→ 建立 Connection / 显式 Dependency Provider
→ 预览完整 Diff 与影响 → Core 验证 → 确认事务
→ TOML 保留格式写回 → 重新打开
→ GUI / CLI / MCP 结果一致
```

M6 完成标准：至少一个真实项目子系统完成该链路；关键操作可由 UI 自动化或可重复集成测试验证；
CI 在 PR 上执行并阻止失败合并（推送后生效）。

## 7. 已知限制（v0.1 遗留）

- Source 只读；
- Binding/Connection/Dependency 编辑 UI 未完成；
- 无 GitHub Actions、无前端 UI 自动化；
- 外部修改用 2s 轮询 + 整项目 digest，无 watcher/三向合并；
- Revision 历史与 request 幂等只在进程内；
- 单 composition 入口；
- MCP 无 subscriptions/progress/cancel/auth；
- 崩溃恢复仅处理 composition `.bak-*`，无 fsync 级保证。

## 8. 常用命令

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo test --manifest-path apps/designer/src-tauri/Cargo.toml
cargo test -p composition-mcp --test e2e

cd apps/designer
npm run build
npm run desktop
# 020 增加后：
# npm run test:ui
```

构建、测试、桌面启动通过 `host_task` 执行；分析、编辑在沙箱内完成。

## 9. 日志

| 日期 | 变更 | 结果 |
|---|---|---|
| 2026-11-26 | M0–M5 | v0.1 完成；146 Core + 5 Tauri 测试；A01–A18 通过 |
| 2026-11-27 | 结构迁移 | 设计基线 → `docs/design/`；评审 → `docs/reviews/`；根目录清理 |
| 2026-11-27 | v0.2 计划 | 采纳 M6 为下一阶段；020 质量门禁卡进入 planned；未开工 |

## 10. 下一步

先创建并执行 **020 — 自动化质量门禁与 UI 测试基础**。在此之前不启动 021，也不修改
v0.1 领域模型。
