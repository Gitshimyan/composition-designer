# 未决工作登记册

> 状态：**Adopted 2026-11-27（G01）**；G03 完成 001–025 历史回溯；026 关闭 UW-025；027/G04 关闭 UW-028、UW-029、UW-010；G05 关闭 UW-030；
> 032 冻结 v2.0 语义，UW-002/UW-003 转 in_progress，新增 UW-032。
> 规则：只有实现并有证据、用户明确拒绝、被其他事项替代或用户明确不再需要，才可关闭。
> “后续任务提到过”不构成关闭证据。

## 1. 总览

| 指标 | 数量 |
|---|---:|
| 登记事项 | 32 |
| 已实现 / 已关闭 | 11 |
| 等待用户决策（pending_decision） | 5 |
| 等待授权（pending_authorization） | 0 |
| 已接受待排期（accepted） | 1 |
| 进行中（in_progress） | 2 |
| 明确延期（deferred） | 13 |

## 2. 登记表

| ID | 来源 | 功能缺口 | 类型 | 状态 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|---|
| UW-001 | 024 → 025 | Dependency Provider 的设置/清空编辑 | Extend | implemented | Extend + Modify | 已解除；Dependency 视图可编辑 | 025 已关闭，保留证据 |
| UW-002 | 021 → 032 | Interface / Contract 的独立身份 | Rearchitect | in_progress | Rearchitect | 依赖无法与实现解耦 | 032 已冻结语义（ADR-0006）；033–035 实施 |
| UW-003 | 021 → 032 | 非结构 runtime / service 容器 | Rearchitect | in_progress | Rearchitect | 运行时服务只能近似映射为结构节点 | 032 已冻结语义（ADR-0006）；033–035 实施 |
| UW-004 | 021 | 项目级 Profile / 条件组合选择 | Rearchitect | accepted | Rearchitect | 无法完整表达 profile/环境/task 变体 | ADR-0004 确认 Profile 入口；S2（039–041）实施 |
| UW-005 | 022 / 025 | ConfigValue 整数值 float 的 JSON wire | Extend（协议方案） | pending_decision | Extend；协议基线变更再评估 Rearchitect | Agent/UI 传输可能无法保留数值类型 | 单独设计 wire 语义 |
| UW-006 | 授权审计 | 权限集合、工程/文档/Compose/副作用边界 | Rearchitect | implemented | Rearchitect + Modify | 规则已生效 | G01 已关闭 |
| UW-007 | 披露评审 | 任务卡/Manifest 模板、coverage/open_items、汇报规范 | Rearchitect | implemented | Rearchitect + Modify | 规则已生效 | G01 已关闭 |
| UW-008 | 授权审计 | 001–025 历史授权审计记录与 Manifest 事实性修正 | Modify | implemented | Modify | 历史分类可追溯 | G02 已关闭 |
| UW-009 | 披露评审 | 001–025 历史未实现工作回溯与登记 | Modify | implemented | Modify | 历史遗留线索有去向 | G03 已关闭 |
| UW-010 | 授权审计 | 机器授权/覆盖门禁 | Extend + Rearchitect | implemented | Extend + Rearchitect | `composition-change-gate` report 模式已接入 CI | G04 已关闭；block 模式转 UW-030 |
| UW-011 | 014 / 015 / 路线图 | Source 编辑与草稿冲突处理（Composition 入口） | 产品能力 | implemented | Modify + Extend/Rearchitect（029） | Composition Source 已可编辑；外部冲突阻止 | 029–031 已关闭；模块文件编辑转 UW-031 |
| UW-012 | 020 | 真实 WebView2 / Tauri GUI 自动化 | 工程能力 | deferred | 待评估 | Playwright 使用 IPC fixture | M6 后评估 tauri-driver/smoke |
| UW-013 | 021 | Glide G4–G8：kind、技术元数据、Port wire、Profile-scoped task、模板 | Rearchitect / 后续 | deferred | Rearchitect | 拓扑够用，运行细节不足 | 出现真实需求时再评估 |
| UW-014 | 009 / 010 / 011 | 撤销 / 重做 | 产品能力 | deferred | Modify（事务协议变更再评估 Rearchitect） | 误操作只能手工回退 | 后续独立评估 |
| UW-015 | 009 / 010 / 011 / 016 / 019 | Revision 历史与 request 幂等跨进程持久化 | 可靠性 | deferred | Modify / Extend（存储方案） | 重启后历史/幂等丢失 | 后续独立评估 |
| UW-016 | 014 | Inspector array 属性编辑 | UI 能力 | pending_decision | Modify | array 配置只能读/手改 TOML | 可在后续小卡补齐 |
| UW-017 | 015 | 外部修改 watcher、三向合并、Review 草稿保留、layout 合并 | 产品能力 | deferred | Modify / Extend | 2s 轮询；Reload 丢弃 Review/布局 | M7 或独立评估 |
| UW-018 | 011 / 016 / 019 | 多 composition 入口 / 多文件项目 | Rearchitect | deferred | Rearchitect | 仅单入口；项目拆分受限 | M9 方向 |
| UW-019 | 011 / 013 / 019 | fsync 级崩溃恢复 | 可靠性 | deferred | Modify | 断电窗口仍存在 | 后续独立评估 |
| UW-020 | 017 / 018 / 019 | MCP subscriptions/progress/cancel/auth/多项目 | 协议能力 | deferred | Extend | Agent 长任务/多项目能力不足 | M8 方向 |
| UW-021 | 018 | Agent Skill LLM 评测、跨平台配置生成 | 工程能力 | deferred | Extend | 仅工具链 e2e，仅 Windows 脚本 | M8 方向 |
| UW-022 | 022 | 模块库/版本发现与下载 | 产品能力 | deferred | Extend / Rearchitect | 只能选已加载模块 | M9 方向 |
| UW-023 | 023 | Binding 向导/Move 的 UX 限制（child 固定、required many 只建一个、MAX_DEPTH） | UX 能力 | pending_decision | Modify | 复杂结构需多次操作/改写 child | 026 后按反馈评估 |
| UW-024 | 012 / 013 / 014 | 窗口自适应、bundle/installer、大 bundle 拆分 | 工程/UI | deferred | Modify / Extend | 仅固定 DPI/示例路径；无安装包 | 后续评估 |
| UW-025 | 010 / 014 / 024 | Diff/Review 覆盖 Dependency 与模块影响 | Change Review | implemented | Modify | 完整 Review 已展示分类/Before-After/依赖影响/全部诊断 | 026 已关闭 |
| UW-026 | 004 | edit-cases / external-change-cases 具体载荷 | 示例/测试 | pending_decision | Compose + Modify | 场景说明无具体请求载荷 | 后续补示例 |
| UW-027 | 003 / 006 / 007 | 边界测试补充（大小写路径、符号链接、级联抑制、重复 SourceMap、session 部分命令） | 测试 | pending_decision | Modify | 边界行为缺少回归保护 | 后续测试卡 |
| UW-028 | M6 总验收 | 真实项目子系统完整 M6 链路（Glide 试点上的编辑链） | 验收 | implemented | Modify | 已由 CLI/Protocol 全链路测试验证 | 027 已关闭 |
| UW-029 | 020 / M6 总验收 | CI PR 触发与分支保护/失败合并阻止的验证 | 工程流程 | implemented | Modify；仓库设置需管理员 | `main` 已启用分支保护；失败探针 PR #1 被 BLOCKED | 027/G04 已关闭 |
| UW-030 | G04 | CI 切换到 change-gate block 模式 | 治理流程 | implemented | Modify + Rearchitect | CI 声明违规会阻止合并 | G05 已关闭 |
| UW-031 | 029 / 031 | 模块文件 Source 编辑（module.toml） | 产品能力 | deferred | Rearchitect + Extend | 首版只编辑 Composition 入口，模块仍只读 | 出现模块编辑需求时评估 |
| UW-032 | 032 | 接口方法/Port 形状、service 内部组合 | Rearchitect | deferred | Rearchitect | v2.0 接口只是身份标记；service 必须是 primitive | 出现真实实现替换/服务组合需求时评估 |

## 3. 已关闭事项证据

### UW-001 — Dependency Provider 编辑

- 实现：`set_dependency_provider` / `unset_dependency_provider`、`[instances.dependencies]` 写回；
- UI：[`DependencyProviderDialog.tsx`](../../apps/designer/src/relations/DependencyProviderDialog.tsx)；
- 文档：`docs/spec/command-schema-v1.md`、ADR-0003；
- 验证：workspace、Protocol、Tauri host、Playwright；详见 025 卡；
- 关闭日期：2026-11-27。

### UW-006 — 授权规范

- 实现：`AGENTS.md` §1–§3、§5、§9；任务卡/Manifest 模板；
- 关闭日期：2026-11-27（G01）。

### UW-007 — 未实现披露制度

- 实现：`AGENTS.md` §4、§6、§9；任务卡 `coverage_status`/`open_items` 与汇报章节；
- 关闭日期：2026-11-27（G01）。

### UW-008 — 历史授权审计记录

- 实现：[`2026-11-27-authorization-audit-record.md`](../../docs/reviews/2026-11-27-authorization-audit-record.md)；
- 任务卡审计备注 13 张；025/020 Manifest 事实性修正；
- 关闭日期：2026-11-27（G02）。

### UW-009 — 历史未实现工作回溯

- 实现：[`2026-11-27-unimplemented-work-backfill.md`](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)；
- 001–025 线索去向与本登记册 UW-014–UW-027；
- 关闭日期：2026-11-27（G03）。

### UW-025 — 完整 Change Review

- 实现：[`ChangeReview.tsx`](../../apps/designer/src/review/ChangeReview.tsx)；
- 分类、Before/After、命令列表、Dependency 影响、全部诊断；
- 验证：28 UI + 155 workspace + 15 Tauri，详见 026 卡；
- 关闭日期：2026-11-27（026）。

### UW-028 — M6 真实项目全链路

- 实现：[`m6_closeout.rs`](../../crates/composition-cli/tests/m6_closeout.rs)；
- 在 Glide 试点副本上通过 CLI/Protocol 完成创建/绑定/配置/连接/Provider/写回/重开；
- 证据：实例 9、Binding 8、Connection 7，CLI validate 与 Core Digest 一致；
- 关闭日期：2026-11-27（027）。

### UW-010 — 机器授权/覆盖门禁（report）

- 实现：[`tools/change-gate`](../../tools/change-gate/README.md)、CI report 步骤；
- 检查 targets、授权、operation、敏感路径、Compose/Extend、coverage 与 unresolved_items；
- 验证：7 单元 + 1 真实仓库集成测试；block 模式退出码已测；
- 关闭日期：2026-11-27（G04）；CI 阻断切换转 UW-030。

### UW-029 — PR 分支保护/失败合并阻止

- `main` 已启用分支保护：strict required check `Windows build and UI tests`，
  enforce_admins=true，禁止 force push/删除；
- 失败探针 PR #1：CI `fail`，`mergeStateStatus=BLOCKED`；已关闭并删除分支；
- 集成 PR #2：CI 通过（run 45997141339）并合并；合并后 main push CI 通过
  （run 45997795133）；
- 关闭日期：2026-11-27（G04）。

### UW-030 — CI block 模式

- 实现：CI `Change gate` 步骤改为 `--mode block`；
- 修复重叠 Manifest 误报：任一覆盖 Manifest 满足约束即通过；
- 验证：G05 单元/仓库测试 + PR #4 CI pass（46000619083），合并后 main CI pass（46001377088）；
- 状态：`implemented`；关闭日期：2026-11-27（G05）。

### UW-011 — Composition Source 编辑

- 实现：029 `replace_source` 事务；030 CodeMirror 编辑/Review；031 Glide 验收；
- 外部冲突阻止、Reload 丢弃草稿；无三向合并（UW-017）；
- 关闭日期：2026-11-27（M7）。

### UW-031 — 模块文件 Source 编辑

- 首版只编辑 `workspace.composition` 入口；`modules/**/module.toml` 仍只读；
- 需要模块定义/契约编辑语义、命令与 UI，属 Rearchitect + Extend；
- 状态：`deferred`；出现真实模块编辑需求时评估。

## 4. 维护规则

- 新任务卡结束前必须检查：是否新增/解决/延期了 UW 事项；
- `pending_authorization` 与 `blocked_authorization` 不得自动转成 `deferred`；
- 用户未回复不等于批准、拒绝或延期；
- 已实现事项必须保留实现任务、测试/验收证据与关闭日期；
- 本文件与 `working/plan.md` 的统计必须一致。
