# docs/spec/

v0.1 的冻结规范（M0，2026-11-26）。实现、样例与错误码以本目录为准。

| 文件 | 内容 | 状态 |
|---|---|---|
| [`domain-model-v1.md`](./domain-model-v1.md) | 领域对象、关系语义、不变量、对象标识 | 已冻结 |
| [`composition-schema-v1.md`](./composition-schema-v1.md) | `project.toml`、`composition/**`、`modules/**/module.toml`、`.designer/**` 的字段、类型、默认值、约束、校验顺序、Revision | 已冻结 |
| [`error-codes-v1.md`](./error-codes-v1.md) | L1–L7 稳定错误代码、级别、对象、提示 | 已冻结 |
| [`command-schema-v1.md`](./command-schema-v1.md) | M2 编辑命令 JSON 形状、事务规则、Revision 历史/幂等、结构化 Diff、TOML 写回 | M2 009–011 冻结 |
| [`designer-layout-v1.md`](./designer-layout-v1.md) | `.designer/layout.toml` 的节点坐标格式与表现层边界 | M3 013 冻结 |

相关：

- 术语表：[`../glossary.md`](../glossary.md)
- 冻结决策：[ADR-0001](../adr/0001-domain-and-schema-freeze.md)、[ADR-0002](../adr/0002-v01-unsupported.md)
- 测试数据：[`../../examples/README.md`](../../examples/README.md)

规范一旦冻结，修改属于 Rearchitect：必须先更新 ADR、明确 `schema_version` 策略，
再同步校验、样例与 Core。顺序不能反。
