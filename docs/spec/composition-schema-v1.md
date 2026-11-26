# Composition Schema v1

- 状态：M0 冻结（2026-11-26）
- 格式：TOML 1.0.0，UTF-8（无 BOM）
- 适用范围：Composition Designer v0.1
- 语义定义：[`domain-model-v1.md`](./domain-model-v1.md)
- 错误代码：[`error-codes-v1.md`](./error-codes-v1.md)

本文件冻结项目声明文件的字段、类型、默认值、约束与校验顺序。Core、CLI、Designer 与 Agent
Adapter 都必须按本文件实现，不得各写一套字段规则。

## 1. 文件集合

| 文件 | 必需 | 职责 | 是否项目语义 |
|---|---:|---|---:|
| `project.toml`（项目根） | 是 | 项目身份、Composition 入口、模块来源 | 是 |
| `composition/<entry>.toml` | 是 | 实例、Binding、Connection | 是 |
| `modules/**/module.toml` | 按需 | 模块定义与契约 | 是 |
| `.designer/layout.toml` | 否 | Canvas 坐标 | 否 |
| `.designer/workspace.toml` | 否 | 视口、面板、偏好 | 否 |

- 只支持**单入口** Composition；`workspace.composition` 指向一个文件，不识别 include/import、
  `composition/parts/` 合并或隐式包含。
- 每个 `module_roots` 条目递归扫描名为 `module.toml` 的文件；目录名以 `.` 开头时跳过；
  不跟随符号链接（v0.1 显式不支持）。
- 文件编码为 UTF-8；BOM、非法 UTF-8、不可读均为 L1 错误。
- 换行 CRLF 与 LF 均可解析；计算源码行列前统一按 LF 归一化。

## 2. 通用规则

### 2.1 标识符

| 名称 | 词法 | 示例 |
|---|---|---|
| 模块 ID | `^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)*$` | `shell.desktop` |
| 类型 ID | 与模块 ID 相同 | `string`、`contract.task.state.v1` |
| 局部 ID（实例、Binding、Connection、属性、Slot、Port、依赖） | `^[a-z][a-z0-9_-]*$` | `app`、`content`、`state-1` |
| 版本 | 精确 `MAJOR.MINOR.PATCH`（无前导零，正则见 §2.1.1） | `1.0.0` |
| `schema_version` | 整数 `1` | `1` |

- 模块 ID、类型 ID 区分大小写，只允许小写。
- 局部 ID 在各自作用域内唯一；具体作用域见 §2.7。
- 版本为**精确版本**，不支持范围、通配、`latest` 或预发布后缀。
- 字符串标识符不得为空；空串是 `schema-empty-value`。

#### 2.1.1 版本正则

```text
^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$
```

### 2.2 路径

- 使用 POSIX 分隔符 `/`；反斜杠、盘符、绝对路径、空段、尾随 `/` 均非法。
- 允许 `.` / `..` 段，但归一化后必须仍在项目根内；越界为 `file-path-escape`。
- 所有路径相对于项目根解析；不得依赖进程当前工作目录。
- `workspace.composition` 必须以 `.toml` 结尾。
- 解析后的真实路径必须仍在项目根内；越界为 `file-path-escape`。
- 同一路径经归一化后重复（如 `modules` 与 `modules/`）为 `schema-duplicate-entry`。
- `module_roots` 之间不得相同或互相嵌套（归一化后），否则 `schema-duplicate-entry`；
  避免同一模块被重复发现。

### 2.3 未知字段

- `project.toml`、Composition 文件、`module.toml` 中任何未在本文档定义的键都是
  `schema-unknown-field`，**不得静默忽略**。
- 唯一例外：`.designer/**` 是表现层数据，Core 不读取，未知字段由 Designer 容错与保留。

### 2.4 默认值

- 属性默认值只在解析时进入 `effective_config`，**不写回源文件**。
- `required = true` 与 `default` 同时出现是 `schema-inconsistency`。
- 默认值必须满足其类型与约束；否则为契约错误（`schema-invalid-value`）。

### 2.5 顺序

- `module_roots`、`accepts`、`enum.values` 是集合：顺序无语义，归一化后稳定排序。
- `instances`、`bindings`、`connections` 的声明顺序无语义，归一化后按 §8.2 排序。
- 配置对象键顺序无语义，序列化时按字典序。
- 只有 Schema 明确赋予语义的列表（如 `values` 的成员资格）才影响判断，顺序本身不影响。

### 2.6 注释与格式

- Core 解析不修改文件；注释、空行、字段顺序不进入模型。
- 局部编辑必须保留未编辑部分的注释与排版（M2 的 `toml_edit` 策略）。
- 格式变化是否改变 `source_digest` 见 §9。

### 2.7 唯一性作用域

| ID | 作用域 |
|---|---|
| 模块 ID + 版本 | 整个项目（所有 `module_roots`） |
| 实例 ID | Composition 文件 |
| Binding ID | Composition 文件 |
| Connection ID | Composition 文件 |
| 属性 / Slot / Port / 依赖 ID | 单个模块定义内，按类别分别唯一 |

## 3. `project.toml`

```toml
[project]
schema_version = 1
id = "example.minimal"
name = "Minimal Project"

[workspace]
composition = "composition/main.toml"
module_roots = ["modules"]
```

### 3.1 `[project]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `schema_version` | integer | 是 | — | 必须为 `1` | `schema-missing-field` / `schema-unsupported-version` |
| `id` | string | 是 | — | 模块 ID 词法，稳定，不依赖目录名 | `schema-invalid-identifier` |
| `name` | string | 是 | — | 非空 | `schema-empty-value` |

`[project]` 表必须存在；缺少表本身按 `schema-missing-field` 报告。

### 3.2 `[workspace]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `composition` | string | 是 | — | 相对路径、`.toml` 结尾、项目根内 | `file-path-invalid` / `file-path-escape` / `file-path-absolute` |
| `module_roots` | array<string> | 是 | — | 非空、无重复、互不嵌套、每项为项目根内目录 | `schema-empty-value` / `schema-duplicate-entry` / `file-path-invalid` / `file-path-escape` |

- `[workspace]` 表必须存在。
- `module_roots` 指向的目录不存在时为 `file-module-root-missing`。
- 不允许额外的顶层表或键。

## 4. Composition 文件

路径由 `workspace.composition` 指定；推荐 `composition/main.toml`。

```toml
[composition]
schema_version = 1
id = "example.minimal"
root = "app"

[[instances]]
id = "app"
module = "shell.desktop"
version = "1.0.0"

[instances.config]
title = "Minimal"

[instances.dependencies]
state = "state-1"

[[bindings]]
id = "app-content"
parent = "app"
slot = "content"
child = "hello"

[[connections]]
id = "editor-status"

[connections.from]
instance = "editor"
port = "status"

[connections.to]
instance = "statusbar"
port = "text"
```

### 4.1 `[composition]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `schema_version` | integer | 是 | — | 必须为 `1` | `schema-missing-field` / `schema-unsupported-version` |
| `id` | string | 是 | — | 模块 ID 词法，且必须等于 `project.id` | `schema-invalid-identifier` / `composition-project-id-mismatch` |
| `root` | string | 是 | — | 局部 ID；必须指向存在的实例 | `schema-invalid-identifier` / `ref-root-not-found` |

- 顶层只允许 `[composition]`、`[[instances]]`、`[[bindings]]`、`[[connections]]`。
- 三个数组可以为空；但必须有 `root` 实例且可解析。

### 4.2 `[[instances]]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `id` | string | 是 | — | 局部 ID；Composition 内唯一 | `schema-invalid-identifier` / `id-duplicate-instance` |
| `module` | string | 是 | — | 模块 ID 词法 | `schema-invalid-identifier` / `ref-module-not-found` |
| `version` | string | 是 | — | 精确版本，必须存在 | `schema-invalid-version` / `ref-module-version-not-found` |
| `config` | table | 否 | 空表 | 键必须是模块声明的属性 | `contract-property-*` |
| `dependencies` | table<string,string> | 否 | 空表 | 键必须是模块声明的依赖 ID；值为实例 ID | `contract-dependency-unknown` / `dependency-*` |

- 未知键（如 `name`）为 `schema-unknown-field`。
- `config` 中未声明的属性为 `contract-property-unknown`；值类型/约束不符见 §6.1。
- `config` 中的嵌套表只有在对应属性类型为 `object` 时才可能合法，而 v0.1 不支持
  `object`，因此嵌套表最终表现为类型不符或未知属性。
- `dependencies` 的键不存在于模块契约时为 `contract-dependency-unknown`；值实例不存在为
  `ref-instance-not-found`；语义校验见 §7.3。

### 4.3 `[[bindings]]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `id` | string | 是 | — | 局部 ID；Composition 内唯一 | `schema-invalid-identifier` / `id-duplicate-binding` |
| `parent` | string | 是 | — | 已存在的实例 ID | `ref-instance-not-found` |
| `slot` | string | 是 | — | 父模块声明的 Slot ID | `ref-slot-not-found` |
| `child` | string | 是 | — | 已存在的实例 ID，且不是 Root | `ref-instance-not-found` / `structure-root-has-parent` |

- 未知键为 `schema-unknown-field`。
- `(parent, slot, child)` 重复（即使 ID 不同）为 `structure-duplicate-binding`。
- 重复三元组保留首个声明；重复项从 Slot 基数与父级计数中排除（见 §10）。
- 同一 child 出现在两个不同 Binding 中为 `structure-instance-multiple-parents`。

### 4.4 `[[connections]]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `id` | string | 是 | — | 局部 ID；Composition 内唯一 | `schema-invalid-identifier` / `id-duplicate-connection` |
| `from` | table | 是 | — | `instance` + `port`；必须是输出 Port | `ref-instance-not-found` / `ref-port-not-found` / `connection-direction-invalid` |
| `to` | table | 是 | — | `instance` + `port`；必须是输入 Port | 同上 |

`from` / `to` 子表：

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `instance` | string | 是 | — | 已存在的实例 ID | `ref-instance-not-found` |
| `port` | string | 是 | — | 该实例模块声明的 Port ID | `ref-port-not-found` |

- 未知键为 `schema-unknown-field`。
- 端点对重复为 `connection-duplicate`；类型/协议/方向/度数校验见 §7.2。

## 5. `module.toml`

```toml
[module]
schema_version = 1
id = "shell.desktop"
version = "1.0.0"
kind = "composite"

[description]
summary = "Desktop application shell"

[contract.properties.title]
type = "string"
required = true

[contract.properties.theme]
type = "enum"
values = ["light", "dark"]
default = "light"

[[contract.slots]]
id = "content"
cardinality = "one"
required = true
accepts = ["leaf.widget"]

[[contract.ports]]
id = "status"
direction = "output"
type = "string"
multi = true

[[contract.dependencies]]
id = "state"
module = "task.state"
version = "1.0.0"
optional = false
```

### 5.1 `[module]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `schema_version` | integer | 是 | — | 必须为 `1` | `schema-missing-field` / `schema-unsupported-version` |
| `id` | string | 是 | — | 模块 ID 词法 | `schema-invalid-identifier` |
| `version` | string | 是 | — | 精确版本；`(id, version)` 项目内唯一 | `schema-invalid-version` / `id-duplicate-module` |
| `kind` | string | 是 | — | `composite` 或 `primitive` | `schema-invalid-value` / `contract-module-invalid` |

- `primitive` 声明 `contract.slots` 为 `contract-module-invalid`。
- 模块定义只读；Designer 不提供编辑（v0.1）。

### 5.2 `[description]`

可选表。

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `summary` | string | 是（若表存在） | — | 非空 | `schema-missing-field` / `schema-empty-value` |
| `details` | string | 否 | — | 可为空 | — |

未知键为 `schema-unknown-field`。

### 5.3 `[contract.properties.<id>]`

| 字段 | 类型 | 必需 | 默认 | 适用类型 | 错误代码 |
|---|---|---:|---|---|---|
| `type` | string | 是 | — | 全部 | `schema-missing-field` / `schema-invalid-value` |
| `required` | boolean | 否 | `false` | 全部 | `schema-type-mismatch` |
| `default` | 与类型一致 | 否 | 无 | 全部 | `schema-invalid-value` |
| `description` | string | 否 | — | 全部 | `schema-type-mismatch` |
| `values` | array<string> | 是（enum） | — | `enum` | `schema-missing-field` / `schema-duplicate-entry` |
| `items` | table | 是（array） | — | `array` | `schema-missing-field` |
| `min_length` / `max_length` | integer | 否 | 无 | `string` | `schema-type-mismatch` |
| `min` / `max` | integer 或 float | 否 | 无 | `integer` / `float` | `schema-type-mismatch` |
| `min_items` / `max_items` | integer | 否 | 无 | `array` | `schema-type-mismatch` |

类型取值：`string`、`integer`、`float`、`boolean`、`enum`、`array`。`object` 及其他值为
`schema-property-type-unsupported`（v0.1 显式不支持）。

规则：

- `enum` 的 `values` 非空、无重复、元素为非空字符串；`default` 若存在必须属于 `values`。
- `array` 的 `items` 子表定义元素类型，字段为：`type`（`string`/`integer`/`float`/`boolean`/`enum`）、
  `values`、`min_length`/`max_length`、`min`/`max`。`items` 不得再含 `items`、`default`、
  `required`、`description`，也不得为 `array`/`object`。
- 数值约束：`min <= max`，`min_length <= max_length`，`min_items <= max_items`；非负。
- 约束字段若用于不适用的类型（如 boolean 上写 `min`）为 `schema-constraint-mismatch`。
- `required = true` + `default` 为 `schema-inconsistency`。
- 默认值类型必须严格匹配（integer 不接受 float，反之亦然），且满足全部约束；
  否则为 `schema-invalid-value`。
- 未列出的字段一律 `schema-unknown-field`。

### 5.4 `[[contract.slots]]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `id` | string | 是 | — | 局部 ID；模块内 Slot 唯一 | `schema-invalid-identifier` / `id-duplicate-slot` |
| `cardinality` | string | 是 | — | `one` / `optional_one` / `many` | `schema-invalid-value` |
| `required` | boolean | 是 | — | 与基数一致（见下） | `schema-inconsistency` |
| `accepts` | array<string> | 是 | — | 非空、无重复的模块 ID | `schema-empty-value` / `schema-invalid-identifier` / `schema-duplicate-entry` |

基数与 `required` 的一致性：

| cardinality | 允许的 required | 绑定数 |
|---|---|---|
| `one` | `true` | 1 |
| `optional_one` | `false` | 0 或 1 |
| `many` | `true` 或 `false` | 0..N 或 1..N |

其他组合为 `schema-inconsistency`。`accepts` 只按模块 ID 匹配，不看版本；版本仍由实例解析。

### 5.5 `[[contract.ports]]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `id` | string | 是 | — | 局部 ID；模块内 Port 唯一 | `schema-invalid-identifier` / `id-duplicate-port` |
| `direction` | string | 是 | — | `input` / `output` | `schema-invalid-value` |
| `type` | string | 是 | — | 类型 ID 词法 | `schema-invalid-identifier` |
| `multi` | boolean | 否 | `false` | — | `schema-type-mismatch` |
| `protocol` | string | 否 | 无 | 非空 | `schema-empty-value` |

- `multi=false` 时，输出 Port 最多一条出边、输入 Port 最多一条入边。
- `protocol` 只做相等比较；不做协商或转换。

### 5.6 `[[contract.dependencies]]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `id` | string | 是 | — | 局部 ID；模块内依赖唯一 | `schema-invalid-identifier` / `id-duplicate-dependency` |
| `module` | string | 是 | — | 模块 ID 词法 | `schema-invalid-identifier` |
| `version` | string | 否 | 无 | 精确版本 | `schema-invalid-version` |
| `optional` | boolean | 否 | `false` | — | `schema-type-mismatch` |

- 依赖目标是模块 ID（不是实例 ID）；提供者由实例映射或唯一匹配确定。
- 允许依赖成环；v0.1 不执行模块，环不构成结构错误。

## 6. 契约校验

### 6.1 实例配置

对每个声明属性：

1. `config` 含未知键 → `contract-property-unknown`（object_id 指向该属性）。
2. 值类型与 `type` 不符 → `contract-property-type-mismatch`。
3. 值不满足 `values` / `min` / `max` / `min_length` / `max_length` / `min_items` / `max_items`
   → `contract-property-value-invalid`。
4. 必填属性既无显式值也无默认值 → `contract-property-required-missing`。
5. 未声明但存在于配置的键不进入 `effective_config`。

TOML 类型严格匹配：`integer` 值不得用于 `float` 属性，反之亦然；`enum` 只接受字符串。

### 6.2 模块契约自身

在实例校验之前，先校验模块契约自洽（L2/L4）：

- `primitive` 有 Slot → `contract-module-invalid`。
- 属性默认值/约束无效 → `schema-invalid-value` / `schema-constraint-mismatch`。
- Slot 的 `required` 与 `cardinality` 不一致 → `schema-inconsistency`。
- 重复的 Slot/Port/属性/依赖 ID → `id-duplicate-*`。

契约自身无效时，仍可解析模块身份；但依赖该契约的实例配置检查跳过，避免级联。

## 7. 结构、连接、依赖与表现层

### 7.1 结构（L5）

按 [`domain-model-v1.md`](./domain-model-v1.md) §5 执行：

- Root 存在、是 `composite`、无父 Binding；
- 非 Root 恰好一个父 Binding；
- 无环、全部可达；
- Slot 基数满足；
- 重复三元组 / 多父 / 孤立实例分别报告。

### 7.2 Connection（L6）

1. 两端实例与 Port 存在（L3 已查；缺失则跳过本条）。
2. `from` 方向必须 `output`，`to` 必须 `input`，否则 `connection-direction-invalid`。
3. 类型 ID 必须相等，否则 `connection-type-mismatch`。
4. 双方都声明 `protocol` 时必须相等，否则 `connection-protocol-mismatch`。
5. `(from.instance, from.port, to.instance, to.port)` 重复 → `connection-duplicate`。
6. `multi=false` 的度数超限 → `connection-port-multi-violation`。

### 7.3 Dependency（L6）

对每个实例的每条依赖要求：

1. 要求 ID 必须在模块契约中声明；否则 `contract-dependency-unknown`。
2. 显式 provider：
   - 实例不存在 → `ref-instance-not-found`；
   - provider 是自己 → `dependency-self`；
   - provider 模块 ID/版本不匹配 → `dependency-provider-invalid`。
3. 未显式 provider：
   - 全项目匹配实例（模块 ID + 可选精确版本，排除自己）恰好 1 个 → 解析成功；
   - 0 个 → `dependency-unresolved`（`optional=true` 时为 `dependency-optional-unresolved` warning）；
   - 多个 → `dependency-ambiguous`。
4. 依赖解析不参考 Binding；解析结果写入规范化模型。

### 7.4 表现层 `.designer/**`

`.designer/**` 是可选的编辑器状态，Core 不读取，不产生诊断，不参与 Revision。
推荐格式（Designer 可扩展，未知字段应保留）：

`.designer/layout.toml`

```toml
schema_version = 1

[positions.app]
x = 120.0
y = 80.0
```

- `positions.<instance-id>.x/y` 为浮点坐标；实例不存在时 Designer 可忽略或清理。
- 移动节点只改本文件，不改 `composition/**`。

`.designer/workspace.toml`

```toml
schema_version = 1

[viewport]
x = 0.0
y = 0.0
zoom = 1.0

[panels]
inspector = true
diagnostics = true
source = false

[preferences]
theme = "light"
```

- 字段缺失时使用 Designer 默认值；未知字段保留。
- `preferences` 是编辑器偏好，不是模块配置。
- 这些文件损坏不应使项目声明无效；Designer 应提示并回退到默认布局。

## 8. 规范化输出

### 8.1 结构

```json
{
  "project": { "id": "example.minimal", "name": "Minimal Project", "schema_version": 1 },
  "composition": { "id": "example.minimal", "root": "app", "schema_version": 1 },
  "modules": [
    {
      "id": "shell.desktop",
      "version": "1.0.0",
      "kind": "composite",
      "description": { "summary": "Desktop shell" },
      "contract": {
        "properties": [],
        "slots": [],
        "ports": [],
        "dependencies": []
      },
      "source": { "file": "modules/shell/module.toml", "range": { "start": { "line": 1, "column": 1 }, "end": { "line": 20, "column": 1 } } }
    }
  ],
  "instances": [
    {
      "id": "app",
      "module": { "id": "shell.desktop", "version": "1.0.0" },
      "kind": "composite",
      "config": {},
      "effective_config": {},
      "source": { "file": "composition/main.toml", "range": { "start": { "line": 6, "column": 1 }, "end": { "line": 9, "column": 1 } } }
    }
  ],
  "bindings": [],
  "connections": [],
  "dependencies": [
    {
      "requirement_id": "state",
      "from_instance": "app",
      "provider_instance": null,
      "optional": false,
      "status": "unresolved",
      "source": { "file": "composition/main.toml", "range": { "start": { "line": 9, "column": 1 }, "end": { "line": 10, "column": 1 } } }
    }
  ],
  "diagnostics": [],
  "revision": { "seq": 0, "source_digest": "sha256:..." }
}
```

- `status`：`resolved` / `unresolved` / `optional_unresolved` / `ambiguous`。
- `source` 的 `range`：行、列均从 1 开始，结束位置**不包含**；列按 Unicode 码点计数，
  换行统一为 LF。
- 契约字段序列化为数组；`properties` / `slots` / `ports` / `dependencies` 按 `id` 升序。

### 8.2 排序

| 集合 | 排序键 |
|---|---|
| `modules` | (`id`, `version`) |
| `instances` | `id` |
| `bindings` | (`parent`, `slot`, `child`, `id`) |
| `connections` | (`from.instance`, `from.port`, `to.instance`, `to.port`, `id`) |
| `dependencies` | (`from_instance`, `requirement_id`) |
| `contract.properties` / `slots` / `ports` / `dependencies` | `id` |
| `accepts` / `enum.values` / `module_roots` | 字典序 |
| 对象键（config、effective_config） | 字典序 |
| `diagnostics` | 见 domain §7.3 |

- 排序按 Unicode 码点比较，不使用本地化排序。
- 相同输入必须产生字节级相同的规范化 JSON；实现不得依赖哈希表迭代顺序或目录枚举顺序。

## 9. Revision 与源摘要

- 语义文件集合：`project.toml` + Composition 入口 + 所有已发现的 `module.toml`（含未被引用者）。
- 文件按项目根相对路径（POSIX、区分大小写）升序排列。
- 摘要输入，对每个文件依次拼接：

```text
UTF8(relative_path) || 0x00 || ASCII(decimal byte_length) || 0x00 || file_bytes || 0x00
```

- `source_digest = "sha256:" + lowercase_hex(SHA-256(拼接结果))`。
- 任一必需文件不可读时 `source_digest = null`，由 L1 诊断说明原因。
- `.designer/**`、`expected.json`、README、注释以外的编辑器状态均不参与。
- 摘要关注**字节**：仅改注释/空行也会改变 `source_digest`，但不改变规范化模型。

## 10. 校验顺序与级联

```text
L1 文件与语法
  → L2 Schema（逐文件）
  → L3 标识与引用
  → L4 模块契约与实例配置
  → L5 组合结构
  → L6 连接与依赖
  → L7 整体一致性（project.id / composition.id）
```

级联规则：

1. `project.toml` 缺失、不可读或 L2 失败：停止，不解析 Composition 与模块。
2. `workspace.composition` / `module_roots` 路径错误：报告 L1 后停止对应分支。
3. 某个 `module.toml` 解析或 Schema 失败：该模块不进入注册表；引用它的实例不产生
   `ref-*` 级联错误，契约检查跳过。
4. 模块重复定义：报告 `id-duplicate-module`；引用该 `(id, version)` 的实例报
   `ref-module-ambiguous`，不自动选择。
5. Composition L2 失败：跳过 L3 及之后对 Composition 的检查。
6. Binding 的端点/Slot 缺失：报告引用错误并跳过该 Binding 的结构检查。
7. Connection 端点缺失：跳过该条的方向/类型/协议检查。
8. 结构阶段发现环：报告 `structure-cycle`；环内实例不再重复报 `structure-unreachable-instance`。
9. 重复 `(parent, slot, child)`：报告 `structure-duplicate-binding`；保留首个，重复项不参与
   Slot 基数、父级计数与可达性计算。

## 11. 示例索引

| 样例 | 位置 | 预期 |
|---|---|---|
| 最小合法 | `examples/minimal-project` | `valid: true` |
| 嵌套组合 + 多实例复用 | `examples/nested-composition` | `valid: true` |
| many Slot 合法 | `examples/edge-valid/slot-many` | `valid: true` |
| 格式保留输入 | `examples/edge-valid/format-preserved` | `valid: true` |
| Slot 基数、引用、属性、结构、版本、文件错误 | `examples/invalid-projects/**` | 见各目录 `expected.json` |
| M2 编辑错误场景 | `examples/edit-cases/README.md` | M2 实现时补命令载荷 |
| M3 外部修改场景 | `examples/external-change-cases/README.md` | M3 实现时补监听载荷 |

每个项目目录内的 `expected.json` 是机器可读数据集：

```json
{
  "valid": false,
  "diagnostics": ["structure-slot-cardinality"],
  "notes": "…"
}
```

## 12. v0.1 不支持项

以下能力显式拒绝或忽略，不使用隐式行为代替：

| 能力 | v0.1 行为 |
|---|---|
| 多文件 Composition / include / import | 只读单入口；`parts/` 不合并 |
| `object` 属性类型 | `schema-property-type-unsupported` |
| 数组嵌套数组 / 数组元素为 object | Schema 拒绝 |
| 预发布 / 范围 / 通配版本 | `schema-invalid-version` |
| 符号链接目录与文件 | 不跟随；目标在根外为 `file-path-escape` |
| 能力依赖（按 capability 而非模块 ID） | 不支持；只按模块 ID |
| Port 协议协商、自动类型转换 | 不做；只做严格相等 |
| 多父 / 共享实例 / 无根项目 | 结构错误 |
| 模块定义编辑 | 只读；M3 不提供 |
| 运行时、构建、部署、代码生成 | 不属于本项目 |

任何新增能力必须先改本文件与错误码，再实现；顺序不能反。
