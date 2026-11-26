# Error Codes v1

- 状态：M0 冻结（2026-11-26）
- 适用范围：Composition Designer v0.1 的解析与验证
- 关联：[`composition-schema-v1.md`](./composition-schema-v1.md)、[`domain-model-v1.md`](./domain-model-v1.md)

## 1. 规则

- 错误代码是稳定契约：`message` 可以改，`code` 不能改语义；新语义必须新增代码。
- 代码格式：小写 kebab-case，`^[a-z][a-z0-9-]*$`。
- 机器判断只依赖 `code` + `severity` + `object_id` + `file/range`，不依赖自然语言。
- 同一问题只报告一次；上游失败导致的下游检查跳过（见 schema §10）。
- `error` 阻止提交；`warning` 不阻止提交但必须返回；`info` 在 v0.1 暂不产生。

## 2. 诊断结构

```json
{
  "code": "ref-module-not-found",
  "severity": "error",
  "message": "instance 'app' references module 'shell.desktop@1.0.0' which is not defined",
  "object_id": "instance:app",
  "file": "composition/main.toml",
  "range": { "start": { "line": 6, "column": 1 }, "end": { "line": 8, "column": 14 } },
  "related": ["module:shell.desktop@1.0.0"],
  "hint": "add the module definition under a module root or correct the module id/version"
}
```

- `object_id` 形式见 domain §7.2。
- `range` 行列从 1 开始、结束不含；列为 Unicode 码点。
- `related` 为对象 ID 数组，可省略；`hint` 可省略，但不得自动执行修改。

## 3. L1 — 文件与语法

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `file-project-missing` | error | 项目根下不存在 `project.toml` | `project` | 在项目根创建 `project.toml` |
| `file-composition-missing` | error | `workspace.composition` 指向的文件不存在 | `composition` | 创建入口文件或修正路径 |
| `file-module-root-missing` | error | `module_roots` 中的目录不存在 | `project` | 创建目录或修正 `module_roots` |
| `file-read-error` | error | 文件不可读、非 UTF-8 或含 BOM | `file:<path>` | 改为可读的 UTF-8（无 BOM）文本 |
| `file-path-absolute` | error | 路径为绝对路径或含盘符 | `project` / `file:<path>` | 改为相对项目根的路径 |
| `file-path-escape` | error | 归一化后路径越出项目根（含 `..` 越界） | `project` / `file:<path>` | 把文件移入项目根或修正路径 |
| `file-path-invalid` | error | 空路径、反斜杠、空段、尾随 `/`、入口非 `.toml` | `project` | 使用 `a/b.toml` 形式的相对路径 |
| `toml-syntax-error` | error | TOML 1.0 解析失败 | `file:<path>` + 范围 | 按 TOML 语法修正 |
| `toml-duplicate-key` | error | 同一表内重复键（含重复 `root`） | `file:<path>` + 范围 | 删除重复键 |

## 4. L2 — Schema

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `schema-missing-field` | error | 必需字段或必需表缺失 | 所属对象 | 补齐字段 |
| `schema-unknown-field` | error | 出现未定义字段 | 所属对象 | 删除或改用已声明字段 |
| `schema-type-mismatch` | error | TOML 类型与字段类型不符 | 所属对象 | 改成正确类型 |
| `schema-invalid-value` | error | 值不在允许集合、默认值不符类型/约束 | 所属对象 | 改成合法值 |
| `schema-invalid-identifier` | error | ID / 类型 ID 不符合词法 | 所属对象 | 使用小写 kebab-case（模块 ID 用点分） |
| `schema-invalid-version` | error | 版本不是精确 `MAJOR.MINOR.PATCH`（无前导零） | 所属对象 | 使用精确版本，如 `1.0.0` |
| `schema-unsupported-version` | error | `schema_version` 不是 `1` | `file:<path>` | 使用 v0.1 支持的格式版本 `1` |
| `schema-unsupported-feature` | error | 使用了 v0.1 不支持的结构（保留扩展） | 所属对象 | 查看 schema §12 的能力清单 |
| `schema-property-type-unsupported` | error | 属性 `type` 为 `object` 或未知类型 | `property:…` | 改用 v0.1 支持的类型 |
| `schema-inconsistency` | error | 字段间矛盾（required+default、Slot 基数组合、min>max） | 所属对象 | 按 schema 规则修正组合 |
| `schema-empty-value` | error | 必填字符串/数组为空 | 所属对象 | 提供非空值 |
| `schema-duplicate-entry` | error | 集合内重复（`module_roots`、`values`、`accepts`） | 所属对象 | 去重 |
| `schema-constraint-mismatch` | error | 约束用于不适用的类型（如 boolean 上写 `min`） | `property:…` | 删除约束或改用正确类型 |

## 5. L3 — 标识与引用

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `id-duplicate-module` | error | 两个 `module.toml` 声明相同 `(id, version)` | `module:<id>@<ver>` | 删除重复定义或改版本 |
| `id-duplicate-instance` | error | Composition 内实例 ID 重复 | `instance:<id>` | 重命名实例 |
| `id-duplicate-binding` | error | Binding ID 重复 | `binding:<id>` | 重命名 Binding |
| `id-duplicate-connection` | error | Connection ID 重复 | `connection:<id>` | 重命名 Connection |
| `id-duplicate-property` | error | 模块内属性 ID 重复（TOML 键重复通常先被 L1 捕获） | `property:…` | 去重 |
| `id-duplicate-slot` | error | 模块内 Slot ID 重复 | `slot:…` | 去重 |
| `id-duplicate-port` | error | 模块内 Port ID 重复 | `port:…` | 去重 |
| `id-duplicate-dependency` | error | 模块内依赖 ID 重复 | `dependency:…` | 去重 |
| `ref-module-not-found` | error | 实例引用的模块 ID 不存在 | `instance:<id>` | 添加模块定义或修正 ID |
| `ref-module-version-not-found` | error | 模块 ID 存在但精确版本不存在 | `instance:<id>` | 改用已存在的精确版本 |
| `ref-module-ambiguous` | error | 引用的 `(id, version)` 存在重复定义 | `instance:<id>` | 消除重复定义 |
| `ref-instance-not-found` | error | Binding/Connection/依赖映射引用了不存在的实例 | 引用方 | 修正实例 ID |
| `ref-slot-not-found` | error | Binding 的父模块未声明该 Slot | `binding:<id>` | 修正 Slot 或父模块契约 |
| `ref-port-not-found` | error | Connection 的实例未声明该 Port | `connection:<id>` | 修正 Port 或模块契约 |
| `ref-root-not-found` | error | `composition.root` 指向的实例不存在 | `composition` | 修正 `root` 或添加实例 |

## 6. L4 — 契约与配置

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `contract-module-invalid` | error | 模块契约与 kind 矛盾（如 primitive 有 Slot） | `module:<id>@<ver>` | 修正 kind 或移除 Slot |
| `contract-property-unknown` | error | 实例配置含未声明属性 | `instance:<id>` / 属性 | 删除该键或声明属性 |
| `contract-property-required-missing` | error | 必填属性无显式值且无默认值 | `instance:<id>` | 在 `config` 中提供值 |
| `contract-property-type-mismatch` | error | 配置值类型与属性类型不符 | `instance:<id>` | 改成正确类型（integer/float 严格区分） |
| `contract-property-value-invalid` | error | 配置值不满足枚举/范围/长度/元素数约束 | `instance:<id>` | 按契约约束修正 |
| `contract-slot-rejects-module` | error | 子实例模块不在 Slot 的 `accepts` 中 | `binding:<id>` | 换 Slot/子模块，或改模块契约（需授权） |
| `contract-dependency-unknown` | error | 实例的依赖映射键没有对应依赖声明 | `instance:<id>` | 删除映射或声明依赖 |

## 7. L5 — 组合结构

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `structure-root-not-composite` | error | Root 实例的模块不是 composite | `instance:<root>` | 改用 composite 作为根 |
| `structure-root-has-parent` | error | Root 被作为 Binding 的 child | `binding:<id>` | 解除指向 Root 的绑定 |
| `structure-instance-multiple-parents` | error | 同一实例出现在两个不同 Binding 的 child | `instance:<id>` | 每个非根实例只保留一个父级 |
| `structure-duplicate-binding` | error | 相同 `(parent, slot, child)` 重复声明 | `binding:<id>` | 删除重复 Binding |
| `structure-slot-cardinality` | error | Slot 绑定数不满足基数与 required | `slot:…` | 补齐或移除子实例 |
| `structure-cycle` | error | Binding 图存在环 | `instance:<id>` | 打破环；Root 不能是后代 |
| `structure-unreachable-instance` | error | 实例无法从 Root 经 Binding 到达 | `instance:<id>` | 接入组合树或删除实例 |

## 8. L6 — 连接与依赖

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `connection-direction-invalid` | error | `from` 不是 output 或 `to` 不是 input | `connection:<id>` | 交换方向或修正 Port 契约 |
| `connection-type-mismatch` | error | 两端 Port 类型 ID 不同 | `connection:<id>` | 连接同类型端口 |
| `connection-protocol-mismatch` | error | 两端都声明 `protocol` 且不相等 | `connection:<id>` | 使用相同协议或不声明 |
| `connection-duplicate` | error | 相同端点对重复连接 | `connection:<id>` | 删除重复 Connection |
| `connection-port-multi-violation` | error | `multi=false` 的 Port 度数超限 | `connection:<id>` | 移除多余连接或改用 `multi=true` |
| `dependency-unresolved` | error | 必需依赖找不到提供者 | `instance:<id>` | 添加提供者实例或显式映射 |
| `dependency-optional-unresolved` | warning | 可选依赖找不到提供者 | `instance:<id>` | 按需添加提供者；不阻止提交 |
| `dependency-ambiguous` | error | 依赖匹配到多个提供者且未显式指定 | `instance:<id>` | 在 `dependencies` 中指定 provider |
| `dependency-provider-invalid` | error | 显式 provider 的模块/版本不满足要求 | `instance:<id>` | 修正 provider 或依赖目标 |
| `dependency-self` | error | 依赖的提供者是需求方自己 | `instance:<id>` | 提供独立实例或改为其它模块 |

## 9. L7 — 整体一致性

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `composition-project-id-mismatch` | error | `composition.id` 与 `project.id` 不同 | `composition` | 统一两个 ID（v0.1 单组合） |
| `composition-internal-error` | error | Core 内部不一致（不应发生） | `composition` | 报告缺陷并提供输入；不要绕过 |

## 10. 预留代码（v0.1 不产生）

以下代码在 M2/M3 建立对应能力时启用；当前表先占位，避免实现时临时命名。

| 代码 | 计划层 | 触发 |
|---|---|---|
| `edit-invalid-command` | M2 | 命令输入 Schema 不合法 |
| `edit-unsupported-operation` | M2 | 当前版本未实现的编辑命令 |
| `edit-transaction-failed` | M2 | 候选模型验证失败，事务整体回滚 |
| `edit-revision-conflict` | M2 | `base_revision` 与当前 Revision 不一致 |
| `edit-duplicate-request` | M2 | `request_id` 重复提交 |
| `watch-reload-failed` | M3 | 外部 TOML 变更后重新解析失败 |

## 11. 代码覆盖检查

M0 示例集至少覆盖：

`toml-syntax-error`、`toml-duplicate-key`、`file-project-missing`、`file-path-escape`、
`schema-missing-field`、`ref-module-not-found`、`ref-module-version-not-found`、
`ref-instance-not-found`、`ref-slot-not-found`、`ref-port-not-found`、
`contract-property-unknown`、`contract-property-required-missing`、
`contract-property-type-mismatch`、`contract-property-value-invalid`、
`structure-instance-multiple-parents`、`structure-duplicate-binding`、
`structure-slot-cardinality`、`structure-cycle`、`structure-unreachable-instance`、
`connection-direction-invalid`、`connection-type-mismatch`、`connection-duplicate`。

未覆盖的代码由 M1 的单元测试补齐；本表不要求 M0 用样例覆盖全部 63 个代码。
