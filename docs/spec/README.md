# docs/spec/

v0.1 的冻结规范（M0，2026-11-26）；v2.0 增量由 S1 032 冻结（2026-11-28）；
v2.1（format 3）由 S2 039 冻结（2026-11-28）。实现、样例与错误码以本目录为准。

| 文件 | 内容 | 状态 |
|---|---|---|
| [`domain-model-v1.md`](./domain-model-v1.md) | 领域对象、关系语义、不变量、对象标识 | 已冻结 |
| [`composition-schema-v1.md`](./composition-schema-v1.md) | `project.toml`、`composition/**`、`modules/**/module.toml`、`.designer/**` 的字段、类型、默认值、约束、校验顺序、Revision | 已冻结 |
| [`error-codes-v1.md`](./error-codes-v1.md) | L1–L7 稳定错误代码、级别、对象、提示 | 已冻结 |
| [`composition-schema-v2.md`](./composition-schema-v2.md) | v2.0 增量：接口标记、接口依赖、实例 scope、版本矩阵、诊断与规范化增量 | 已冻结（S1 032）；033–035 实现 |
| [`error-codes-v2.md`](./error-codes-v2.md) | v2.0 新增错误代码（8 个） | 已冻结（S1 032） |
| [`composition-schema-v3.md`](./composition-schema-v3.md) | v2.1 增量：项目级 Profile、多 Composition 入口、选择/Revision/Digest | 已冻结（S2 039）；040–041 实现 |
| [`error-codes-v3.md`](./error-codes-v3.md) | v2.1 新增错误代码（3 个） | 已冻结（S2 039） |
| [`command-schema-v1.md`](./command-schema-v1.md) | M2 编辑命令 JSON 形状、事务规则、Revision 历史/幂等、结构化 Diff、TOML 写回 | M2 009–011 冻结；v0.2 025 Extend（Dependency Provider） |
| [`designer-layout-v1.md`](./designer-layout-v1.md) | `.designer/layout.toml` 的节点坐标格式与表现层边界 | M3 013 冻结 |
| [`protocol-v1.md`](./protocol-v1.md) | 机器可读 Query/Command 协议、JSONL 与 CLI 单次命令 | M4 016 冻结 |
| [`mcp-adapter-v1.md`](./mcp-adapter-v1.md) | MCP stdio 生命周期、tools/resources 与 JSON-RPC 错误 | M4 017 冻结 |

相关：

- 术语表：[`../glossary.md`](../glossary.md)
- 冻结决策：[ADR-0001](../adr/0001-domain-and-schema-freeze.md)、[ADR-0002](../adr/0002-v01-unsupported.md)、
  [ADR-0006](../adr/0006-schema-v2-freeze.md)、[ADR-0007](../adr/0007-schema-v21-profiles.md)
- 测试数据：[`../../examples/README.md`](../../examples/README.md)

规范一旦冻结，修改属于 Rearchitect：必须先更新 ADR、明确 `schema_version` 策略，
再同步校验、样例与 Core。顺序不能反。
