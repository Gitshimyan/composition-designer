# Error Codes v2.0

- 状态：S1 032 冻结（2026-11-28）
- 适用范围：`project.schema_version = 2` 的项目
- 关联：[`composition-schema-v2.md`](./composition-schema-v2.md)、[`error-codes-v1.md`](./error-codes-v1.md)、[`domain-model-v1.md`](./domain-model-v1.md)

本文件只定义 v2.0 **新增**的错误代码。v1 的全部代码、级别、对象标识与提示规则继续有效；
v1 项目的诊断集合不因 v2.0 发布而改变。

## 1. 规则

- 代码是稳定契约：`message` / `hint` 可改，`code` 的语义不能改；新语义必须新增代码。
- 格式、级别与机器判断规则同 v1：只依赖 `code` + `severity` + `object_id` + `file/range`。
- 同一问题只报告一次；上游失败导致的下游检查跳过（见 schema v2 §10）。
- 本文件的代码只在 v2 项目产生；v1 项目继续使用 v1 代码。
- v1 的 `schema-unsupported-version` 在 v2 项目中的含义扩展为「不在 `1`/`2` 中」；
  v1 项目中仍是「不是 `1`」。代码本身不变。

## 2. 新增代码

### 2.1 L2 — Schema 与版本

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `schema-version-mismatch` | error | v2 项目中 Composition 的 `schema_version` 为 `1`（与项目版本不一致） | `composition` | 把 Composition 的 `schema_version` 改为 `2`，或把项目降为 v1 |

- 其他整数（如 `3`）由 v1 既有 `schema-unsupported-version` 处理，不新增代码。

### 2.2 L3 — 标识与引用

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `id-duplicate-interface` | error | 同一模块内重复声明相同 `(id, version)` 的接口 | `interface:<module-id>@<module-version>#<interface-id>@<interface-version>` | 删除重复声明；同一接口的不同版本可并存 |
| `ref-interface-not-found` | error | 依赖要求指向的接口（含指定版本）没有任何已注册模块声明 | `dependency:<module>@<ver>#<id>` | 声明接口实现或修正 `interface`/`version` |

- `ref-interface-not-found` 在 L3 阶段检查；触发后跳过该依赖要求的解析，不再报
  `dependency-unresolved`。

### 2.3 L4 — 契约与配置

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `contract-service-module-invalid` | error | `scope = "service"` 的实例使用了 `kind = "composite"` 的模块 | `instance:<id>` | 改用 `primitive` 模块，或把 `scope` 改回 `structure` |

### 2.4 L5 — 组合结构

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `structure-service-in-binding` | error | Binding 的 `parent` 或 `child` 是 `scope = "service"` 的实例 | `binding:<id>` | 移除该 Binding，或把实例改为 `structure` |
| `structure-root-scope-invalid` | error | Root 实例的 `scope` 不是 `structure` | `instance:<root>` | Root 必须是 structure；服务不能作为根 |

### 2.5 L6 — 连接与依赖

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `dependency-interface-mismatch` | error | 显式 provider 未声明所需接口，或声明的接口版本与要求不符 | `instance:<id>` | 换 provider，或在 provider 模块声明该接口/版本 |
| `service-unused` | warning | `service` 实例既未被任何依赖引用（显式映射或解析结果），也不是任何 Connection 端点 | `instance:<id>` | 接入依赖/连接，或删除该 service；不阻止提交 |

## 3. 级别与对象

- `error` 阻止提交；`warning` 不阻止提交但必须返回并进入 Review。
- `service-unused` 是 v2.0 唯一的 v2 新增 warning；是否展示由入口决定，Core 必须返回。
- 对象标识：
  - 接口声明：`interface:<module-id>@<module-version>#<interface-id>@<interface-version>`；
  - 依赖要求：沿用 `dependency:<module-id>@<module-version>#<requirement-id>`；
  - 实例、Binding、Composition 沿用 v1。

## 4. 覆盖计划

035 的 v2 数据集至少覆盖：

`ref-interface-not-found`、`id-duplicate-interface`、`contract-service-module-invalid`、
`structure-service-in-binding`、`structure-root-scope-invalid`、`dependency-interface-mismatch`、
`schema-version-mismatch`、`service-unused`。

`schema-inconsistency`（依赖同时写 `module` 与 `interface`）与 `schema-missing-field`
（两者都不写）复用 v1 代码，必须有对应反例。
