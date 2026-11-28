# Error Codes v3（S2 / v2.1）

- 状态：S2 039 冻结（2026-11-28）
- 适用范围：`project.schema_version = 3` 的项目，以及 profile 选择入口
- 关联：[`composition-schema-v3.md`](./composition-schema-v3.md)、
  [`error-codes-v2.md`](./error-codes-v2.md)、[`error-codes-v1.md`](./error-codes-v1.md)

本文件只定义 v2.1 新增的错误代码。v1/v2.0 的全部代码、级别与规则继续有效；
v1/v2.0 项目的诊断集合不因 v3 发布而改变。

## 1. 规则

- 代码是稳定契约：`message` / `hint` 可改，`code` 的语义不能改。
- 选择失败可作为 Core 诊断，也可由 CLI/MCP 以协议错误返回同一 code。
- 同一问题只报告一次；选择失败后不加载 Composition，避免级联。
- 未选中的 profile 不读取，因此不会产生其文件/内容错误。

## 2. 新增代码

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `profile-not-found` | error | 显式选择的具名 profile 不在 `[profiles]` 中 | `project` | 检查 `[profiles.<name>]` 或省略选择 |
| `profile-unsupported` | error | 在 v1/v2.0 项目上显式选择 profile | `project` | 移除选择，或把项目升级为 v3 |
| `profile-reserved-name` | error | `project.toml` 声明 `[profiles.default]` | `project` | 删除该表；默认入口由 `workspace.composition` 提供 |

- 三者均为 `error`，层为 L2（项目 Schema/选择）。
- Profile 名非法（不是局部 ID）仍用 `schema-invalid-identifier`。
- Profile 表缺字段/类型/未知键/路径错误分别复用 v1 的
  `schema-missing-field` / `schema-type-mismatch` / `schema-unknown-field` /
  `file-path-*`；所选入口缺失复用 `file-composition-missing`。
- 版本不匹配复用 `schema-version-mismatch` / `schema-unsupported-version`；
  v3 项目中的不支持值语义与 v2 相同。

## 3. 对象与来源

- 三个代码的 `object_id` 都是 `project`；
- 能定位时给出 `project.toml` 的 `file` 与字段范围；
- 选择错误（`profile-not-found` / `profile-unsupported`）由选择参数触发，
  Core 在加载前返回；CLI/MCP 可直接映射为协议错误。

## 4. 覆盖计划（041）

至少覆盖：

- `profile-not-found`：CLI `--profile missing`、MCP `--profile missing`；
- `profile-unsupported`：v1/v2.0 项目上 `--profile`；
- `profile-reserved-name`：`[profiles.default]`；
- 复用代码：profile 路径越界、入口缺失、v3 Composition 版本错误；
- v1/v2.0 回归：原有诊断不变。
