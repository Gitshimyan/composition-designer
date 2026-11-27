# 评审响应与整改计划（2026-11-27）

- 状态：Adopted（2026-11-27 用户确认 G01 授权范围；G02 进行中）
- 来源评审：
  - [`2026-11-27-authorization-audit.md`](./2026-11-27-authorization-audit.md)
  - [`2026-11-27-unimplemented-work-transparency.md`](./2026-11-27-unimplemented-work-transparency.md)
- 执行计划：[`working/plan.md`](../../working/plan.md)
- 未决工作登记：[`working/decisions/unresolved-work.md`](../../working/decisions/unresolved-work.md)

本文件记录对两份评审的逐项处置。评审是输入，不等于已自动授权；需要 Rearchitect 的规则
变更必须由用户明确批准后，才可修改 `AGENTS.md` 或任务卡/Manifest 规则。

## 1. 总体结论

1. **接受授权审计的核心判断**：五种模式不是线性等级，应按 `required` 权限集合声明；
   审计指出的 002、003、009、010、016 等历史分类问题作为审计记录保留，不追溯伪造
   当时的授权证据。
2. **接受未实现工作显性化的方向**：任务完成状态与需求覆盖状态分离；未实现事项必须有
   稳定 `UW-*` ID、影响、所需授权和后续处理，不能只在“遗留问题”末尾一句话带过。
3. **先治理、后功能**：在 026 完整 Change Review 之前，先完成授权规范、模板和历史
   审计记录；机器门禁放在 026 之后。
4. **不自动扩大范围**：G1–G3 等 Glide 试点发现的 Schema/领域模型缺口仍是 Rearchitect
   候选，不因收到评审或启用新流程而自动进入实现。
5. **当前可立即修正的事实性记录**：025 Manifest 漏列 Protocol 测试 target；
   020 Manifest 的 `expected_command_changes` 与“生产 IPC 未改变”不一致。两项已修正，
   不涉及授权历史改写。

## 2. 逐项处置

| 评审建议 | 处置 | 承接卡 |
|---|---|---|
| 五种模式改为可组合权限集合（required / approved / evidence） | 接受；修订 AGENTS 与模板 | G01 |
| 任务卡/Manifest 模板增加 required/approval、coverage_status、open_items | 接受；修订模板 | G01 |
| 明确工程配置、文档权限、Compose `working/pilots/**`、副作用边界 | 接受；写入 AGENTS | G01 |
| 未实现事项登记册、决策历史、任务汇报规范 | 接受；建立制度与登记册 | G01 / G03 |
| 历史任务授权审计记录（001–025） | 接受；只记录审计分类，不倒填授权 | G02 |
| 修正 010/020/021/025 Manifest 问题 | 025、020 已做事实性修正；010/021 记录审计结论，不改写历史 | G02 |
| 历史未实现工作回溯 | 接受；逐卡核对并登记 UW | G03 |
| 026 完整 Change Review | 接受；在治理与历史记录之后执行 | 026 |
| 机器授权/覆盖门禁 | 接受方向；在 026 后实现，先报告模式再阻断 | G04 |

## 3. 任务卡队列

G00（评审归档与整改计划）已在本轮完成；下表为后续执行项。

| 顺序 | 任务卡 | 授权 | 状态 | 说明 |
|---|---|---|---|---|
| 1 | [G01 — 授权规范与披露制度](../../working/tasks/G01-governance-rules.md) | Rearchitect + Modify | done | AGENTS、任务卡模板、Manifest 模板、未实现披露规则生效 |
| 2 | [G02 — 历史授权审计记录与 Manifest 修正](../../working/tasks/G02-historical-audit.md) | Modify | next | 001–025 审计分类；不伪造历史授权 |
| 3 | [G03 — 历史未实现工作回溯](../../working/tasks/G03-unresolved-backfill.md) | Modify | queued | 填充未决工作登记册与覆盖状态 |
| 4 | 026 — 完整 Change Review | Modify | queued | M6-006 功能卡 |
| 5 | [G04 — 机器授权/覆盖门禁](../../working/tasks/G04-machine-gate.md) | Extend + Rearchitect | deferred | 026 后先报告、再阻断 |

## 4. 授权与执行状态

### 4.1 G01 的 Rearchitect 范围（已批准）

用户已于 2026-11-27 明确确认 G01 范围并指示开始。已修改：

- `AGENTS.md`：权限集合、工程配置边界、文档权限、Compose 路径、副作用边界、
  未实现披露规则；
- `working/tasks/000-template.md`：`authorization.required/approval`、
  `coverage_status`、`open_items`、需求覆盖与范围调整章节；
- `changes/000-template.yaml`：权限集合、`unresolved_items`、target/scope/operations 分离；
- 汇报规范：任务结束时必须包含“未实现 / 被搁置功能”独立章节。

### 4.2 执行顺序（已确认）

`G01 → G02 → G03 → 026 → G04`。G02 可立即开始；G04 仍需 Extend + Rearchitect。

### 4.3 历史分类的处理方式

历史任务卡的原始授权字段保持不变；审计结论写入 G02 的历史审计记录，并标注
“审计建议 / 当时的授权证据不足”。不追溯修改旧任务卡来假装当时已获批准。

## 5. 验收

- 两份评审文档已归档，根目录无散落评审文件；
- `working/plan.md` 有权队列、依赖、授权状态与未决工作总览；
- 每个未实现/阻塞事项有稳定 UW ID、影响、所需授权与后续处理；
- G01 已在用户明确批准后执行，G02/G03 尚未开始；
- 025/020 Manifest 的事实性修正与实际 diff 一致；
- 026 及机器门禁不被提前标记为完成。
