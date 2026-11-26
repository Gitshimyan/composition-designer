# docs/spec/

v0.1 的冻结规范。M0 产出：

| 文件 | 内容 | 任务卡 |
|---|---|---|
| `domain-model-v1.md` | 领域对象、关系语义、不变量、术语表 | 002 |
| `composition-schema-v1.md` | `project.toml`、`composition/**`、`modules/**/module.toml`、`.designer/**` 的字段、类型、默认值、约束 | 003 |
| `error-codes-v1.md` | L1–L7 稳定错误代码、级别、关联对象、修正提示 | 003 |

规范一旦冻结，修改属于 Rearchitect：必须先更新 ADR、明确 `schema_version` 策略，
再同步校验、样例与 Core。
