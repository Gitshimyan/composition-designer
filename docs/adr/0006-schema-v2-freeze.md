# ADR-0006 — Schema v2.0 冻结：接口身份与实例 scope

- 状态：**Accepted（2026-11-28，032）**
- 授权：用户 2026-11-28 明确授权先更新文档、随后执行 032（Rearchitect + Extend）；见
  [`decision-history.md`](../../working/decisions/decision-history.md) D-021
- 关联：[ADR-0004](./0004-schema-v2-direction.md)（方向）· [`composition-schema-v2.md`](../spec/composition-schema-v2.md) ·
  [`error-codes-v2.md`](../spec/error-codes-v2.md) · [`2026-11-27-g7-domain-gaps.md`](../reviews/2026-11-27-g7-domain-gaps.md)
- 性质：语义冻结，不改产品代码；033–035 实施

## 背景

ADR-0004 已确认 v2 的方向：v2.0 增加接口身份与 service scope，v2.1 增加项目 Profile 与多入口。
方向不等于可实现契约：字段形态、版本兼容、校验层、错误代码与命令字段仍需冻结，否则
Domain、Loader、Validation、Commands 会各自发明规则。本 ADR 记录 032 冻结的具体决策。

## 决策

### D1 — 版本按文件声明，项目版本决定规则集

三种声明文件继续各自带整数字段 `schema_version`。`project.schema_version` 为 `1` 或 `2`，
并决定整项目的规则集：v1 项目要求 Composition 与模块都是 `1`；v2 项目要求 Composition 是
`2`，允许模块为 `1` 或 `2`。v1 项目行为、诊断与规范化输出保持字节级不变。

### D2 — 接口是模块契约内的身份标记

模块用 `[[contract.interfaces]]` 声明 `(id, version)`；ID 使用模块 ID 词法，版本为精确 semver。
接口不声明方法、参数或 Port 形状。同一模块内 `(id, version)` 唯一；多个模块可声明同一身份
以表达多实现；同一模块可声明同一 ID 的多个版本。

### D3 — 依赖目标二选一

`[[contract.dependencies]]` 的 `module` 与 `interface` 恰好写一个：两个都写为
`schema-inconsistency`，都不写为 `schema-missing-field`。`version` 对接口目标是可选的精确
接口版本；省略表示接受该接口的任意已声明版本，但仍受候选唯一性约束。

### D4 — 接口解析只看实现，不回退模块匹配

显式 provider 必须声明所需接口（以及要求的精确版本）；自动解析在声明该接口的实例中按
实例计数：唯一即成功，零个为 unresolved（optional 为 warning），多个为 ambiguous。
接口要求不得回退为模块 ID 匹配，也不得按模块名或命名约定推断。

### D5 — 实例 scope 区分 structure 与 service

`[[instances]]` 增加可选 `scope = "structure" | "service"`，默认 `structure`，只在 v2
Composition 中定义。service 不参与 Binding 树：不得有父/子 Binding，不要求从 Root 可达，
但仍可声明配置、依赖、Port 与 Connection，也可以作为依赖 provider。Root 必须是 structure。

### D6 — service 必须是 primitive 模块

`scope = "service"` 的实例只能引用 `kind = "primitive"` 的模块，否则
`contract-service-module-invalid`。理由：composite 模块的结构只能由 Binding 表达，而 service
禁止 Binding；允许 composite service 会产生永远无法满足的 required Slot，属于假结构。
service 的内部组合需求应通过依赖/连接表达，或在未来版本单独设计。

### D7 — 命令只扩展 `create_instance`

命令集合保持 15 条；`create_instance` 增加可选 `scope`。v1 项目显式传 `scope` 报
`edit-invalid-command`（v1 文件无法表达该字段）；v2 项目写入并由候选模型校验。其余命令形态
不变；接口依赖的 provider 有效性由候选校验裁决。

### D8 — v2 输出增量与确定性

v2 规范化输出增加 `contract.interfaces`、`instances[].scope`、`dependencies[].target`
（`kind` / `id` / `version`）。v1 输出不增加这些字段，保持字节级兼容。排序键与确定性要求不变。

### D9 — v2.1 与接口形状不在 v2.0

项目 Profile、多 Composition 入口、接口方法与 Port 形状、service 内部组合均不进入 v2.0。
`[profiles]` 在 v2.0 项目是 `schema-unknown-field`。这些能力需要各自的 Rearchitect 决策；
其中接口形状与 service 组合登记为 UW-032。

## 备选方案

| 方案 | 优点 | 缺点 | 未采用原因 |
|---|---|---|---|
| 接口定义为独立 `interfaces/*.toml` | 中心定义、可独立演进 | 新文件集合、发现/迁移成本高；与 ADR-0004 已确认方向冲突 | 032 沿用 ADR-0004 的模块契约标记 |
| 项目版本统一约束所有模块 | 规则更简单 | 升级项目必须批量改模块；破坏「v1 继续可用」 | D1 改为按文件版本 + 兼容矩阵 |
| 依赖省略 `interface` 时回退模块匹配 | 兼容直觉 | 隐式选择实现，违反显式原则 | 禁止回退 |
| 允许 composite service | 复用现有模块 | required Slot 永远无法满足，制造假结构 | 要求 primitive，必要时单独设计组合 |
| service 使用 Composition 级 `[[services]]` 表 | 语义显式 | 与 `[[instances]]` 重复，引用/序列化复杂 | ADR-0004 已选择实例 scope |
| v2 同时支持 Profile overlay | 一次到位 | 合并/验证/Diff 语义复杂，易变隐式兼容 | 留给 S2，先冻结 v2.0 |

## 后果

- 正面：033–035 有唯一可实现的字段/校验/诊断/命令契约；v1 项目零改动；接口与 service 都能
  用现有对象表达；诊断可定位到接口、实例与 Binding。
- 成本：新增 8 个诊断代码与 9 条 v2 不变量；Loader 需要版本矩阵与接口注册表；Commands 需要
  区分 v1/v2 的 `scope`；UI 需要 Services 视图与接口展示（034）。
- 需要配套验证：035 的 v2 正反例、v1 回归、确定性输出、迁移拒绝路径；Glide 试点 v2 验收。
- 风险：接口没有形状，真实项目可能仍需在依赖语义上继续演进（UW-032）；service 必须
  primitive 可能限制既有 composite 模块的运行时用法，需真实试点确认。

## 复审条件

出现以下任一情况时重新评估：

1. 真实项目需要接口方法/参数/Port 形状，而不只是身份；
2. 需要 composite service 或 service 内部结构；
3. 需要跨模块共享接口定义（独立接口文件）；
4. v1/v2 混合项目带来无法接受的维护成本；
5. v2.1 Profile 设计需要改变 v2.0 的版本或入口语义。
