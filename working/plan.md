# Composition Designer — v0.2 执行计划

> 当前设计基线：[`docs/design/v0.1-design.md`](../docs/design/v0.1-design.md)（Draft，v0.1 冻结）
> 外部评审：[`docs/reviews/2026-11-27-project-review-and-roadmap.md`](../docs/reviews/2026-11-27-project-review-and-roadmap.md)
> 测试基线：146 Core/Protocol/MCP/CLI + 18 UI + 12 Tauri 测试
> 当前阶段：v0.2 M6 进行中；020–023 完成，024 待开始
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
- 020 已完成：Windows CI 文件、Playwright 1.65 + 8 条 UI 链路、本地 `npm run test:ui`；
- 021 已完成：Glide Task + Terminal 只读试点，7 模块/7 实例/6 Binding/6 Connection/3 Dependency，Core 验证通过；
- 022 已完成：Instance 创建/删除/重命名/换模块与版本/配置调整，走统一事务与 Review；
- 023 已完成：Binding Move/Detach/required Slot 子树创建，Slot 信息与影响可审查；
- 缺口：CI 尚未远端执行；Connection/Dependency 编辑视图、Source 编辑；
- 下一步 024：Connection / Dependency 视图。

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
| 1 | 020 | 自动化质量门禁与 UI 测试基础 | 无 | done（CI 远端待推送） |
| 2 | 021 | 真实项目试点与 Schema 复核 | 020；Glide Task+Terminal | done |
| 3 | 022 | Instance 编辑 | 021 结论 | done |
| 4 | 023 | Binding 编辑 | 022 | done |
| 5 | 024 | Connection / Dependency 视图 | 022–023 | next |
| 6 | 025 | 完整 Change Review | 022–024 | queued |

### 020 — 自动化质量门禁与 UI 测试基础（done）

已完成：

- `.github/workflows/ci.yml`（windows-latest、最小权限、Cargo/npm 缓存）；
- 146 Rust + 5 Tauri host + MCP e2e + TypeScript/Vite + Playwright 接入；
- Playwright 1.65 + 系统 Edge；测试专用 IPC fixture；
- 8 条 UI 链路：valid/invalid/select/edit+apply/cancel/reject/external/layout；
- `npm run test:ui` 本地通过（build + 8 passed）；
- CI 远端尚未执行，因为尚未推送。

### 021 — 真实项目试点（done）

已完成 Glide Task + Terminal 只读试点：

- 7 模块 / 7 实例 / 6 Binding / 6 Connection / 3 Dependency；
- `validate`：valid=true、0 诊断；
- 表达：结构、依赖、端口、连接、实例 Config；
- 缺口：接口/契约独立身份、项目级 Profile、运行时/技术元数据、Profile-scoped tasks；
- 不复制 Glide 源码，不修改 v0.1 Schema。

### 022 — Instance 编辑（done）

已完成：

- Inspector：重命名、换 Module/Version（配置 Unset/Reset/必填 Set 同一事务）、
  删除（子树/连接/依赖影响预览）、在 Slot 下创建子实例（`create_instance` +
  `create_binding`）；
- 所有操作 `prepare_transaction` → Review → `apply_transaction`；取消/拒绝不写入；
- 删除 Root 禁用；非法/重复 ID 本地拦截；
- 15 条 Playwright UI 链路（新增 7 条 Instance 链路）；
- 9 个 Tauri host tests，其中 4 个用真实 `ProjectSession` 验证创建、删除级联、
  拒绝不写文件、重命名 root/引用；
- 复用 009 冻结命令，不新增 Schema/命令。

已知边界（已由 023 补齐）：022 的创建候选只包含不带必填 Slot 的模块；023 的递归
结构创建覆盖了 required Slot 子树。

### 023 — Binding 编辑（done）

已完成：

- Inspector Binding 列表：父/子 Binding、Move、Detach；
- Move 对话框：parent 候选 + Slot 候选（accepts/cardinality/required/占用/子节点），
  只发 `update_binding`，child 子树不变；
- Detach 走 `delete_binding` + Review；单根模型下 Core 以
  `structure-unreachable-instance` 拒绝，UI 显示真实诊断而非静默处理；
- 创建向导升级为递归结构创建：模块的 required Slot 自动展开为子草稿，
  所有实例与 Binding 在同一事务提交（023 补齐 022 的边界）；
- 18 条 Playwright UI 链路（新增 3 条 Binding/子树链路）；
- 12 个 Tauri host tests，其中 7 个真实 `ProjectSession` 写回测试；
- 复用 009 冻结命令，不新增 Schema/命令。

已知边界：Move 的 child 固定（创建时选择 child）；Detach 单独提交在单根模型下必被
拒绝，移除子结构使用 Delete。

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
- GitHub Actions workflow 已建立但未远端执行；UI 自动化使用 IPC fixture，不是真实 WebView2；
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
npm run test:ui
```

构建、测试、桌面启动通过 `host_task` 执行；分析、编辑在沙箱内完成。

## 9. 日志

| 日期 | 变更 | 结果 |
|---|---|---|
| 2026-11-26 | M0–M5 | v0.1 完成；146 Core + 5 Tauri 测试；A01–A18 通过 |
| 2026-11-27 | 结构迁移 | 设计基线 → `docs/design/`；评审 → `docs/reviews/`；根目录清理 |
| 2026-11-27 | v0.2 计划 | 采纳 M6 为下一阶段；020 质量门禁卡进入 planned |
| 2026-11-27 | 020 | Windows CI + Playwright 8 条 UI 链路；本地 146 Rust + 5 Tauri + 8 UI 通过；CI 远端待推送 |
| 2026-11-27 | 021 | Glide Task + Terminal 只读试点；7 模块/7 实例/6 Binding/6 Connection/3 Dependency；valid=true |
| 2026-11-27 | 022 | Instance 创建/删除/重命名/换模块/配置调整；15 UI + 9 Tauri 测试通过；复用冻结命令 |
| 2026-11-27 | 023 | Binding Move/Detach/required Slot 递归子树创建；18 UI + 12 Tauri 测试通过 |

## 10. 下一步

023 已完成。下一步 **024 — Connection / Dependency 视图**：为 Port Connection 与
Dependency Provider 建立独立视图与编辑路径，三个视图共享同一 Composition Model，
不画成同一种无类型边；所有操作继续经 Core 事务与 Review。
