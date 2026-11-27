# Composition Designer — v0.2 执行计划

> 当前设计基线：[`docs/design/v0.1-design.md`](../docs/design/v0.1-design.md)（Draft，v0.1 冻结）
> 外部评审：[项目评审](../docs/reviews/2026-11-27-project-review-and-roadmap.md) ·
> [授权审计](../docs/reviews/2026-11-27-authorization-audit.md) ·
> [未实现披露](../docs/reviews/2026-11-27-unimplemented-work-transparency.md) ·
> [整改计划](../docs/reviews/2026-11-27-audit-response.md)
> 测试基线：155 Core/Protocol/MCP/CLI + 28 UI + 15 Tauri 测试
> 当前阶段：M6 功能与 Glide 全链路验收通过；PR 强制门禁（UW-029）未完成；G01–G03 完成
> 推进规则：一次只推进一张任务卡；只有用户明确要求时才推送远端，平时只做本地 commit。

## 1. 目标

v0.2 的目标是 **Complete Composition Authoring**：

> 让人类用户和 Agent 都能通过明确、可验证、可审查的方式，完成一个项目的完整声明式组合设计。

重点不是增加运行时、构建、部署或模块市场，而是把已经存在的领域命令完整、安全地交付到
Designer、Source 与 Agent 入口。

## 2. 当前状态（2026-11-27）

- v0.1 M0–M5 已冻结并通过 A01–A18 验收；
- 评审已归档；v0.1 领域模型、TOML Schema、命令协议不因 v0.2 UI 需求直接修改；
- 当前 GUI：Canvas/Tree、Inspector Instance/Binding 与配置编辑、Structure/Connections/
  Dependencies 三视图、Change Review、只读 Source、外部变更 Reload；
- 020 已完成：Windows CI 文件、Playwright 1.65 + 8 条 UI 链路、本地 `npm run test:ui`；
- 021 已完成：Glide Task + Terminal 只读试点，7 模块/7 实例/6 Binding/6 Connection/3 Dependency，Core 验证通过；
- 022 已完成：Instance 创建/删除/重命名/换模块与版本/配置调整，走统一事务与 Review；
- 023 已完成：Binding Move/Detach/required Slot 子树创建，Slot 信息与影响可审查；
- 024 已完成：Structure/Connections/Dependencies 三视图共享模型，Connection 可编辑；
- 025 已完成：Dependency Provider 编辑（Extend）：`set_dependency_provider` /
  `unset_dependency_provider`、命令 Schema、TOML 写回与 Designer 入口；
- 2026-11-27 收到两份评审：授权体系审计、未实现工作披露；已归档到 `docs/reviews/`；
- G01 已完成：权限集合、工程配置/文档/Compose/副作用边界、任务卡与 Manifest 模板、
  未实现披露与 UW 登记制度生效；
- G02 已完成：001–025 历史授权审计记录、Manifest 事实性修正与任务卡审计备注；
- G03 已完成：001–025 历史未实现工作回溯；登记册 27 项（6 关闭、9 待决策、0 accepted、12 延期）；
- 026 已完成：Change Review 分类、Before/After、命令、Dependency 影响、完整诊断；
  M6 功能卡完成，总验收 027 待执行；
- 缺口：G1–G3 等 Rearchitect 缺口；ConfigValue 整数 float 的 wire 限制；Source 编辑；
- CI 已在远端 main 执行并通过（run 45984567517，5m9s）；
- 下一步：M6 收口检查；G04 需授权；M7 为候选阶段。

## 3. 路线图

| 阶段 | 目标 | 状态 | 主要交付 |
|---|---|---|---|
| M6 | Complete Composition Authoring | complete_with_deferred_items | 功能与 Glide 全链路通过；PR 强制门禁 UW-029 待配置 |
| M7 | Source Authoring & Consistency | later | 可编辑 Source、统一提交、外部修改/草稿冲突 |
| M8 | Agent Authoring Experience | later | 上下文查询、影响分析、事务批处理、Skill 评测 |
| M9 | Composition Ecosystem | deferred | Module Registry/Versioning、多文件、模板、Adapter；不承诺进入 v0.2 |

## 4. M6 队列

| 顺序 | 卡 | 方向 | 依赖 | 状态 |
|---|---|---|---|---|
| 1 | 020 | 自动化质量门禁与 UI 测试基础 | 无 | done（远端 CI 已通过） |
| 2 | 021 | 真实项目试点与 Schema 复核 | 020；Glide Task+Terminal | done |
| 3 | 022 | Instance 编辑 | 021 结论 | done |
| 4 | 023 | Binding 编辑 | 022 | done |
| 5 | 024 | Connection / Dependency 视图 | 022–023 | done |
| 6 | 025 | Dependency Provider 编辑（Extend） | 024；用户明确授权 | done |
| 7 | 026 | 完整 Change Review | 022–025；G01–G03 | done |
| 8 | 027 | M6 收口验收 | 020–026 | done（coverage: complete_with_deferred_items；UW-029） |

### 4.1 评审整改队列（G 轨道，2026-11-27）

评审整改不占用 M6 功能卡编号；026 是完整 Change Review，027 是 M6 收口。顺序为
`G00 → G01 → G02 → G03 → 026 → 027 → G04`。

| 顺序 | 卡 | 方向 | 授权 | 状态 | 依赖 |
|---|---|---|---|---|---|
| 0 | [G00 — 评审归档与整改计划](tasks/G00-review-intake.md) | 归档评审、整改队列、UW 登记、Manifest 事实性修正 | Modify | done | 无 |
| 1 | [G01 — 授权规范与披露制度](tasks/G01-governance-rules.md) | 权限集合、工程配置/文档/Compose 边界、任务卡与 Manifest 模板、未实现汇报规则 | Rearchitect + Modify | done | 用户已批准 |
| 2 | [G02 — 历史授权审计记录与 Manifest 修正](tasks/G02-historical-audit.md) | 001–025 审计分类、Manifest 事实性修正、不追溯授权 | Modify | done | G01 |
| 3 | [G03 — 历史未实现工作回溯](tasks/G03-unresolved-backfill.md) | 全量回溯、UW 登记、覆盖状态 | Modify | done | G01 |
| 4 | 026 — 完整 Change Review | Before/After、影响、诊断、确认/取消 | Modify | done | G01–G03 |
| 5 | 027 — M6 收口验收 | Glide 试点全链路、CLI/Protocol 一致性、CI PR 门禁 | Modify | done | 020–026 |
| 6 | [G04 — 机器授权/覆盖门禁](tasks/G04-machine-gate.md) | Manifest × diff × coverage 自动检查，先报告后阻断 | Extend + Rearchitect | deferred | 027 |

**当前状态：** M6 功能与 Glide 全链路验收通过；PR 强制门禁未完成（UW-029）。
G01–G03 已完成；G04 需 Extend + Rearchitect；M7 为下一阶段候选。

### 4.2 未决工作总览

来源：[`working/decisions/unresolved-work.md`](decisions/unresolved-work.md)。

| 指标 | 数量 |
|---|---:|
| 登记事项 | 29 |
| 已实现 / 已关闭 | 7（UW-001、006、007、008、009、025、028） |
| 等待用户决策（pending_decision） | 10（G1–G3、ConfigValue、array、Binding UX、机器门禁、PR 门禁等） |
| 等待授权（pending_authorization） | 0 |
| 已接受待排期（accepted） | 0 |
| 明确延期（deferred） | 12（Source、WebView2、MCP、多入口、撤销等） |

### 020 — 自动化质量门禁与 UI 测试基础（done）

已完成：

- `.github/workflows/ci.yml`（windows-latest、最小权限、Cargo/npm 缓存）；
- 146 Rust + 5 Tauri host + MCP e2e + TypeScript/Vite + Playwright 接入；
- Playwright 1.65 + 系统 Edge；测试专用 IPC fixture；
- 8 条 UI 链路：valid/invalid/select/edit+apply/cancel/reject/external/layout；
- 远端 main 首次执行通过（run 45984567517，5m9s，2026-11-27）。

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

### 024 — Connection / Dependency 视图（done）

已完成：

- Stage 增加 Structure / Connections / Dependencies 三个视图，共享 `selectedId`；
- Connection 视图：连接列表、每实例 Port 投影（方向/type/protocol/multi/连接），
  Create/Edit/Delete 全部经 prepare → Review → apply；
- Dependency 视图：按实例分组展示要求、required/optional、候选、显式/自动 Provider
  与解析状态；
- 23 条 Playwright UI 链路（新增 5 条关系视图链路）；
- 14 个 Tauri host tests，其中 10 个真实 `ProjectSession` 写回测试
  （新增 Connection 创建/修改/删除、非法方向拒绝）；
- 复用冻结命令，不新增 Schema/命令。

明确的 Extend 缺口（025 已补齐）：v0.1 没有设置 Dependency Provider 的命令；
Provider 编辑新增了命令 Schema + Core + Adapter 一致路径，不是 UI-only 改动。

### 025 — Dependency Provider 编辑（Extend，done）

已完成（用户明确授权）：

- 新增 `set_dependency_provider` / `unset_dependency_provider`，`COMMAND_KINDS` 12 → 14；
- 事务级错误：实例不存在、Unset 无映射 → `edit-invalid-command`；
  requirement/provider 合法性、自引用、歧义仍由 L2–L7 校验；
- `ProjectSession` 写回 `[instances.dependencies]`，空表移除，保留格式；
- Designer Dependency 视图：显式 Provider 设置/清空、候选列表、状态刷新，全部经 Review；
- 154 Core/Protocol/MCP/CLI + 24 UI + 15 Tauri 测试通过；
- 命令 Schema 已在 `docs/spec/command-schema-v1.md` 标注 v0.2 025 Extend。

### 026 — 完整 Change Review（done）

已完成：

- 新增 `src/review/ChangeReview.tsx`：命令列表、Instance/Binding/Connection 分类、
  Before/After 字段、Dependency Provider 影响、完整诊断与验证分区；
- 复用现有 `ModelDiff`/诊断/快照，不新增命令、事务协议或 Tauri IPC 字段；
- 28 条 Playwright UI 链路通过；154 workspace + 15 Tauri 测试通过；
- M6 六张功能卡（020–026）全部完成，CI 远端通过；总验收 027 待执行。

### 027 — M6 收口验收（done）

已完成：

- `crates/composition-cli/tests/m6_closeout.rs`：Glide 试点副本上的 CLI/Protocol 全链路；
- 实例 9、Binding 8、Connection 7；写回后重开 valid，Digest 与 Core 一致；
- UI 28、Tauri 15、workspace 155 测试通过；
- 验收记录：`docs/reviews/2026-11-27-m6-closeout.md`；
- **未完成**：`main` 无分支保护（API 404），无 PR 运行证据；UW-029 待配置。

### 022–026 — 完整编辑链

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

当前：前两项通过（027 Glide 全链路 + 28 UI）；第三项未通过——`main` 无分支保护，
无 PR 运行证据，登记为 UW-029。

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
| 2026-11-27 | 024 | 三视图 + Connection 编辑 + Dependency 只读投影；23 UI + 14 Tauri 测试通过 |
| 2026-11-27 | 025 | Extend：Dependency Provider Set/Unset 命令、写回与 UI；154 + 24 UI + 15 Tauri 测试通过 |
| 2026-11-27 | 远端 CI | push main 后首次执行通过（Windows，run 45984567517，5m9s） |
| 2026-11-27 | 评审整改规划 | 授权审计 + 未实现披露评审归档；G00–G04 队列、UW 登记册、审计响应写入；G01 等待 Rearchitect 授权 |
| 2026-11-27 | G01 | 权限集合与边界、任务卡/Manifest 模板、未实现披露与 UW 制度生效；产品代码/Schema 未改 |
| 2026-11-27 | G02 | 001–025 历史授权审计记录、Manifest 事实性修正、13 张任务卡审计备注；G03 下一步 |
| 2026-11-27 | G03 | 001–025 未实现工作回溯；登记册 27 项；任务卡 coverage/open_items；026 下一步 |
| 2026-11-27 | 026 | 完整 Change Review：分类、Before/After、命令、依赖影响、全部诊断；28 UI 通过；027 总验收 |
| 2026-11-27 | 027 | Glide 全链路 CLI/Protocol 测试通过；PR 分支保护 404，UW-029 待配置；M6 功能验收完成 |

## 10. 下一步

M6 功能与 Glide 全链路验收通过；027 已关闭 UW-028。下一步：

1. **UW-029 — PR 强制门禁**：在 GitHub 启用 `main` 分支保护，把
   `Windows build and UI tests` 设为 required check，并用一个 PR 实际验证；
   该操作修改远端仓库设置，需用户明确批准；
2. G04 — 机器授权/覆盖门禁：需要 Extend + Rearchitect；
3. M7 — Source Authoring（Source 编辑、草稿冲突），需按新规则重新立卡与授权。

G1–G3 等 Schema/领域模型缺口、ConfigValue wire、array 编辑、Binding UX 等仍在
未决登记中等待决策，不自动进入实现。
