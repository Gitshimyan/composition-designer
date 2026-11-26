# ADR-0001：v0.1 领域模型与 TOML Schema 冻结

- 状态：Accepted
- 日期：2026-11-26
- 决策者：Composition Designer v0.1
- 关联：[`domain-model-v1.md`](../spec/domain-model-v1.md)、[`composition-schema-v1.md`](../spec/composition-schema-v1.md)

## 背景

设计基线给出了对象清单、TOML 草案与阶段顺序，但字段名、默认值、依赖解析方式、Revision
算法和若干边界仍是“建议”。M1 的解析器与校验器不能建立在猜测上；必须先冻结一套可实现、
可测试、可被 Agent 依赖的规则。冻结同时要遵守设计基线的核心原则：单一权威模型、显式组合、
确定性解析、不隐式兼容。

## 决策

1. **单入口 Composition**：`workspace.composition` 指向唯一文件；不实现 include/import、
   `composition/parts/` 合并或隐式包含。多文件能力留给后续版本。
2. **三种声明文件都带 `schema_version`**：`project.toml` 的 `[project]`、Composition 的
   `[composition]`、`module.toml` 的 `[module]` 各有一个整数字段，v0.1 固定为 `1`。
3. **`composition.id` 必须等于 `project.id`**：v0.1 一个项目只有一个 Composition；
   不引入“多组合变体”语义。
4. **精确版本**：实例以 `MAJOR.MINOR.PATCH` 精确匹配模块定义；不自动选择“最新”、
   不做范围求解、不支持预发布与通配。
5. **Slot 基数三值**：`one` / `optional_one` / `many`，`required` 必须与基数组合一致；
   `many` 不设上限。
6. **依赖显式化**：模块契约声明依赖要求；实例可用 `[instances.dependencies]` 把要求映射到
   具体提供者。未映射时仅在“恰好一个匹配实例（排除自己）”时自动解析；零个/多个都返回诊断。
   依赖图允许成环，因为 v0.1 不执行模块。
7. **Connection 纳入 v0.1**：支持 Port 方向、严格类型相等、双方都声明时的协议相等、
   端点对唯一与 `multi` 度数限制；不做协议协商、自动转换或基于连接的隐式绑定。
8. **Revision = `seq` + `source_digest`**：摘要覆盖 `project.toml`、Composition 入口与所有已发现
   `module.toml` 的路径与字节；不依赖修改时间，不包含 `.designer/**`。
9. **表现层隔离**：`.designer/**` 不属于项目语义，Core 不读取；节点坐标只影响布局文件。
10. **规范化输出稳定排序**：模块、实例、Binding、Connection、依赖、契约成员与诊断都有明确
    排序键；相同输入必须产生字节级相同结果。

## 备选方案

| 方案 | 优点 | 缺点 | 未采用原因 |
|---|---|---|---|
| 模块文件不要求 `schema_version` | 与设计基线示例逐字一致 | 模块格式无法独立演进；只能跟随项目版本 | 三种文件应由格式版本统一约束 |
| 依赖按模块 ID 自动匹配所有实例 | 字段更少 | 多实例时无法确定提供者，且可能隐式选择 | 违反“显式、可追踪”原则 |
| 首版推迟 Connection | 实现更小 | L6 与 Port 验收缺失，后续返工 | 连接是设计基线的核心对象之一 |
| Revision 用规范化模型哈希 | 与格式无关 | 仅改注释时 Revision 不变，无法保护未保存格式 | 冲突检测需要覆盖源文件字节 |
| 支持多文件 Composition | 更接近大型项目 | include 语义、循环与覆盖规则未冻结 | 先把单入口做正确 |
| 允许一个实例多父级 | 表达共享 | 与树结构和单父规则冲突 | 后续如需共享应新增显式模型 |

## 后果

- 正面：M1 可以直接按 Schema 实现，不需要再问字段语义；Agent 可以依赖稳定错误代码与排序；
  Revision 能同时覆盖语义与格式变化。
- 负面/成本：`module.toml` 多一个字段；依赖映射增加一个实例级表；单入口限制大型项目拆分。
- 需要配套的验证：`examples/**` 的 `expected.json` 覆盖错误代码；M1 的确定性与往返测试；
  M2 的 `base_revision` 冲突测试。
- 复审条件：真实项目需要多文件 Composition、能力依赖或共享实例时，重新评估 1、6、10；
  任何修改必须提升 `schema_version` 并新增 ADR。

## 关联

- 设计基线：§3、§4、§5、§7.3、§15
- 相关 ADR：[ADR-0002](./0002-v01-unsupported.md)（显式不支持项）
- 相关规范：`docs/spec/domain-model-v1.md`、`docs/spec/composition-schema-v1.md`、
  `docs/spec/error-codes-v1.md`
