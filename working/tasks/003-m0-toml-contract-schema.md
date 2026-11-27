# 003 — M0 TOML 与 Module Contract Schema 冻结

- 状态：done
- 目标：把 002 的领域模型落成可实现的 TOML 规范：字段、类型、必填、默认值、约束、稳定排序、
  路径基准与错误代码，产出 `docs/spec/composition-schema-v1.md` 与 `docs/spec/error-codes-v1.md`。
- 授权：Modify（仅 `docs/`）
- 变更声明：无（仅文档）

> 授权审计（2026-11-27）：审计建议 **Extend + Rearchitect**（字段/契约 + Schema 结构/版本策略）；原始授权字段保持不变。见 [审计记录](../../docs/reviews/2026-11-27-authorization-audit-record.md)。

- coverage_status：complete_with_deferred_items
- open_items：UW-027
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

设计基线 §4 / §5 给出了草案示例和 10 条 Schema 规则，但字段表、默认值、错误代码仍是“建议”。
M1 的解析器与校验器不能建立在猜测上；004 的样例也需要确定的字段名与错误代码。本卡同时
冻结设计基线 §15 的剩余决策（Composition 拆分、模块定义编辑等）。

## 清单

- [x] `project.toml` 字段表：`project.*`、`workspace.*`（类型 / 必填 / 默认 / 约束 / 错误码）
- [x] `composition/**` 字段表：`composition.*`、`[[instances]]`、`[instances.config]`、`[[bindings]]`
- [x] `module.toml` 字段表：`[module]`、`[contract.properties.*]`、`[[contract.slots]]`、`[[contract.ports]]`、`[[contract.dependencies]]`
- [x] `.designer/**` 边界与推荐格式（Core 不读取；不参与诊断/Revision）
- [x] `schema_version` 策略：三种声明文件各自带 `schema_version = 1`
- [x] 路径规则：相对基准、`..` 归一化后不得越界、`module_roots` 互不嵌套、符号链接不跟随
- [x] 未知字段、默认值、稳定排序、注释保留的明确规则
- [x] 多文件 Composition 结论：首版单入口，不支持 include / `parts/`
- [x] Slot 基数枚举与 `required` 组合矩阵；Port Connection 的方向/类型/协议/multi 范围
- [x] 错误代码表：L1–L7，63 个验证代码 + 6 个 M2/M3 预留代码
- [x] 把 §15 九项决策逐条写成结论或 unsupported，并记 ADR-0001 / ADR-0002
- [x] 为正反例样例列出字段级清单，交给 004

## 验收

- 每个字段都有类型、必填性、默认值（或“无默认”）、约束与错误代码。
- 设计基线 §12.1 的 12 类样例都能被字段表 + 错误代码覆盖（004 已按此建立）。
- 不存在“实现时再定”的字段语义；确实推迟的（object 属性、多文件、能力依赖等）已标记 unsupported。
- 与 002 的领域模型一一对应，没有多出或缺失的对象；差异见 domain §9。
- M1 可以只按本规范实现解析与校验，不需要再问设计问题。

## 结果 / 遗留问题

- 结果：`docs/spec/composition-schema-v1.md`（文件组织、通用规则、四类文件字段表、
  L1–L7 校验顺序与级联、规范化输出与排序、Revision 摘要）、
  `docs/spec/error-codes-v1.md`（诊断结构、63 个验证代码、6 个预留代码）。
- 关键决策：`composition.id == project.id`；精确版本；`one`/`optional_one`/`many`；
  Connection 首版做严格方向/类型/协议与 `multi` 度数；`source_digest` 覆盖声明文件字节；
  `.designer/**` 由 Designer 独占。
- 遗留：`schema-invalid-identifier` 的词法规则在实现时用同一正则库；未单独冻结大小写不敏感
  文件系统上的路径行为，M1 以真实路径 + 归一化路径处理。
- 下一步：004（示例项目集）。
