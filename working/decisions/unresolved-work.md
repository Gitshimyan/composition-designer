# 未决工作登记册

> 状态：初始登记（制度正式化由 G01 完成）
> 规则：只有实现并有证据、用户明确拒绝、被其他事项替代或用户明确不再需要，才可关闭。
> “后续任务提到过”不构成关闭证据。

## 1. 总览

| 指标 | 数量 |
|---|---:|
| 登记事项 | 13 |
| 已实现 / 已关闭 | 1 |
| 等待用户决策（pending_decision） | 5 |
| 等待授权（pending_authorization） | 2 |
| 已接受待排期（accepted） | 2 |
| 明确延期（deferred） | 3 |

## 2. 登记表

| ID | 来源 | 功能缺口 | 类型 | 状态 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|---|
| UW-001 | 024 → 025 | Dependency Provider 的设置/清空编辑 | Extend | implemented | Extend + Modify | 已解除；Dependency 视图可编辑 | 025 已关闭，保留证据 |
| UW-002 | 021 | Interface / Contract 的独立身份（Dependency 目标仍是具体模块 ID） | Rearchitect | pending_decision | Rearchitect | 依赖/端口无法与实现解耦，provider 只能按模块表达 | 先做设计研究；与 UW-003 联合评估 |
| UW-003 | 021 | 非结构的 runtime / service 容器 | Rearchitect | pending_decision | Rearchitect | 运行时服务只能被近似映射为结构节点，层级失真 | 与 UW-002 联合设计；不提前改 Schema |
| UW-004 | 021 | 项目级 Profile / 条件组合选择 | Rearchitect | pending_decision | Rearchitect | 无法完整表达 profile、环境、task 变体 | 需要真实 UI/项目需求触发 |
| UW-005 | 022 / 025 | ConfigValue 整数值 float 的 JSON wire 区分 | Extend（协议方案） | pending_decision | Extend；若改共享协议基线再评估 Rearchitect | Agent/UI 传输可能无法保留原始数值类型 | 单独设计 wire 语义，不做局部 hack |
| UW-006 | 授权审计 | 五种模式改为可组合权限集合；工程配置/文档/Compose 路径/副作用边界 | Rearchitect | pending_authorization | Rearchitect + Modify | 规则无法准确约束组合权限与越权判断 | G01；等待用户批准 |
| UW-007 | 披露评审 | 任务卡/Manifest 模板、`coverage_status`、`open_items`、未实现汇报规范 | Rearchitect | pending_authorization | Rearchitect + Modify | `done` 可能掩盖未实现项；登记无强制格式 | G01；等待用户批准 |
| UW-008 | 授权审计 | 001–025 历史授权审计记录与 Manifest 事实性修正 | Modify | accepted | Modify | 历史授权分类与 Manifest 覆盖不一致 | G02，G01 后执行 |
| UW-009 | 披露评审 | 001–025 历史未实现工作回溯与登记 | Modify | accepted | Modify | 部分缺口可能只停留在旧卡末尾 | G03，G01 后执行 |
| UW-010 | 授权审计 | 机器授权/覆盖门禁（targets、operations、授权等级、coverage） | Extend + Rearchitect | pending_decision | Extend + Rearchitect | Manifest 仍只是书面契约，越权靠人工发现 | G04，026 之后；先报告后阻断 |
| UW-011 | 014 / 015 / 路线图 | Source 编辑与草稿冲突处理 | 产品能力 | deferred | 待 M7 评估 | Source 仍只读；文本编辑与事务未对等 | M7 排期时重新确认，不算授权阻塞 |
| UW-012 | 020 | 真实 WebView2 / Tauri GUI 自动化 | 工程能力 | deferred | 待评估 | Playwright 使用 IPC fixture，不是真实 WebView2 | M6 后评估 tauri-driver / smoke |
| UW-013 | 021 | Glide G4–G8：kind、技术/运行时元数据、Port wire、Profile-scoped task、模板 | Rearchitect / 后续 | deferred | Rearchitect | 当前模型足以表达拓扑，但无法表达实现/运行细节 | 出现真实 UI 或导入需求时再评估 |

## 3. 已关闭事项证据

### UW-001 — Dependency Provider 编辑

- 实现：`set_dependency_provider` / `unset_dependency_provider`（`crates/composition-editor`）；
- 写回：`[instances.dependencies]`（`ProjectSession`）；
- UI：[`DependencyProviderDialog.tsx`](../../apps/designer/src/relations/DependencyProviderDialog.tsx)；
- 文档：`docs/spec/command-schema-v1.md`、ADR-0003；
- 验证：workspace、Protocol、Tauri host、Playwright 均有覆盖（详见 025 卡）；
- 关闭日期：2026-11-27。

## 4. 维护规则

- 新任务卡结束前必须检查：是否新增/解决/延期了 UW 事项；
- `pending_authorization` 与 `blocked_authorization` 不得自动转成 `deferred`；
- 用户未回复不等于批准、拒绝或延期；
- 已实现事项必须保留实现任务、测试/验收证据与关闭日期；
- 本文件与 `working/plan.md` 的统计必须一致。
