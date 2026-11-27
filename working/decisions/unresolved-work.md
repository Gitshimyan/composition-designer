# 未决工作登记册

> 状态：**Adopted 2026-11-27（G01）**；G03 完成 001–025 历史回溯；026 关闭 UW-025。
> 规则：只有实现并有证据、用户明确拒绝、被其他事项替代或用户明确不再需要，才可关闭。
> “后续任务提到过”不构成关闭证据。

## 1. 总览

| 指标 | 数量 |
|---|---:|
| 登记事项 | 29 |
| 已实现 / 已关闭 | 6 |
| 等待用户决策（pending_decision） | 10 |
| 等待授权（pending_authorization） | 0 |
| 已接受待排期（accepted） | 1 |
| 明确延期（deferred） | 12 |

## 2. 登记表

| ID | 来源 | 功能缺口 | 类型 | 状态 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|---|
| UW-001 | 024 → 025 | Dependency Provider 的设置/清空编辑 | Extend | implemented | Extend + Modify | 已解除；Dependency 视图可编辑 | 025 已关闭，保留证据 |
| UW-002 | 021 | Interface / Contract 的独立身份 | Rearchitect | pending_decision | Rearchitect | 依赖无法与实现解耦 | 先做设计研究；与 UW-003 联合评估 |
| UW-003 | 021 | 非结构 runtime / service 容器 | Rearchitect | pending_decision | Rearchitect | 运行时服务只能近似映射为结构节点 | 与 UW-002 联合设计 |
| UW-004 | 021 | 项目级 Profile / 条件组合选择 | Rearchitect | pending_decision | Rearchitect | 无法完整表达 profile/环境/task 变体 | 需要真实需求触发 |
| UW-005 | 022 / 025 | ConfigValue 整数值 float 的 JSON wire | Extend（协议方案） | pending_decision | Extend；协议基线变更再评估 Rearchitect | Agent/UI 传输可能无法保留数值类型 | 单独设计 wire 语义 |
| UW-006 | 授权审计 | 权限集合、工程/文档/Compose/副作用边界 | Rearchitect | implemented | Rearchitect + Modify | 规则已生效 | G01 已关闭 |
| UW-007 | 披露评审 | 任务卡/Manifest 模板、coverage/open_items、汇报规范 | Rearchitect | implemented | Rearchitect + Modify | 规则已生效 | G01 已关闭 |
| UW-008 | 授权审计 | 001–025 历史授权审计记录与 Manifest 事实性修正 | Modify | implemented | Modify | 历史分类可追溯 | G02 已关闭 |
| UW-009 | 披露评审 | 001–025 历史未实现工作回溯与登记 | Modify | implemented | Modify | 历史遗留线索有去向 | G03 已关闭 |
| UW-010 | 授权审计 | 机器授权/覆盖门禁 | Extend + Rearchitect | pending_decision | Extend + Rearchitect | Manifest 仍靠人工核对 | G04，026 之后 |
| UW-011 | 014 / 015 / 路线图 | Source 编辑与草稿冲突处理 | 产品能力 | deferred | 待 M7 评估 | Source 仍只读 | M7 排期时重新确认 |
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
| UW-028 | M6 总验收 | 真实项目子系统完整 M6 链路（Glide 试点上的编辑链） | 验收 | accepted | Modify | 目前 UI 自动化用 fixture，host 编辑测试用示例项目 | 027 承接 |
| UW-029 | 020 / M6 总验收 | CI PR 触发与分支保护/失败合并阻止的验证 | 工程流程 | pending_decision | Modify；仓库设置需管理员 | 尚未观察到 PR 运行与强制合并门禁 | 027 记录/验证 |

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
- 验证：28 UI + 154 workspace + 15 Tauri，详见 026 卡；
- 关闭日期：2026-11-27（026）。

## 4. 维护规则

- 新任务卡结束前必须检查：是否新增/解决/延期了 UW 事项；
- `pending_authorization` 与 `blocked_authorization` 不得自动转成 `deferred`；
- 用户未回复不等于批准、拒绝或延期；
- 已实现事项必须保留实现任务、测试/验收证据与关闭日期；
- 本文件与 `working/plan.md` 的统计必须一致。
