# Composition Schema v2.0

- 状态：S1 032 冻结（2026-11-28）
- 适用范围：`project.schema_version = 2` 的项目；v1 项目继续由
  [`composition-schema-v1.md`](./composition-schema-v1.md) 定义
- 语义基线：v1 的领域对象、关系与不变量全部继续适用；本文件只定义增量与冲突时的新规则
- 决策：[ADR-0004](../adr/0004-schema-v2-direction.md)（方向）· [ADR-0006](../adr/0006-schema-v2-freeze.md)（冻结）
- 新增错误代码：[`error-codes-v2.md`](./error-codes-v2.md)
- 实现卡：033（Domain/Loader/Validation）、034（Commands/UI/CLI/MCP）、035（fixtures/迁移拒绝/试点）

> v2.1（`schema_version = 3`，项目级 Profile / 多 Composition 入口）见
> [`composition-schema-v3.md`](./composition-schema-v3.md)；v2.0 语义与输出不变。

> v2.0 只增加**接口身份**与**实例 scope**。项目 Profile、多 Composition 入口属于 v2.1（S2），
> 不在本文件范围；在 v2.0 项目中出现 `[profiles]` 仍是 `schema-unknown-field`。

## 1. 版本与文件兼容

### 1.1 `schema_version` 取值

| 文件 | v1 项目 | v2.0 项目 | 说明 |
|---|---|---|---|
| `project.toml` | `1` | `2` | 项目版本决定整项目的规则集 |
| Composition 入口 | `1` | `2` | 必须与项目版本相等 |
| `module.toml` | `1` | `1` 或 `2` | v2 模块可声明接口；v1 模块在 v2 项目中原样可用 |

规则：

- `project.schema_version = 1` 时，其余文件按 v1 规则校验：Composition 与模块都必须是 `1`。
  这保持 v1 项目的现有诊断与行为不变。
- `project.schema_version = 2` 时，Composition 必须是 `2`；`1` 报
  `schema-version-mismatch`，其他整数报 `schema-unsupported-version`。
- v2 项目允许加载 `module.schema_version = 1` 或 `2` 的模块；其他值报
  `schema-unsupported-version`（该模块按 v1 级联规则不进入注册表）。
- 任何文件都不接受 `schema_version` 缺失或非整数（沿用 v1 `schema-missing-field` /
  `schema-type-mismatch`）。

### 1.2 兼容矩阵

| 项目 | Composition | 模块 | 结果 |
|---|---|---|---|
| 1 | 1 | 1 | v1，合法 |
| 1 | 1 | 2 | 模块 `schema-unsupported-version` |
| 1 | 2 | — | v1 规则：Composition `schema-unsupported-version` |
| 2 | 2 | 1 或 2 | v2，合法 |
| 2 | 1 | — | `schema-version-mismatch` |
| 2 | 2 | 其他 | 模块 `schema-unsupported-version` |

- v1 模块在 v2 项目中按 v1 语义解释：没有接口声明，依赖只能指向模块。
- v2 模块不得出现在 v1 项目中：v1 项目要求模块版本为 `1`。

### 1.3 语义文件集合与 Revision

- 摘要集合、排序、拼接格式与 `source_digest` 算法**不变**（v1 §9）：
  仍覆盖 `project.toml`、Composition 入口与所有已发现的 `module.toml`（v1 与 v2 模块都包含）。
- `.designer/**` 仍不属于项目语义。

## 2. 接口（interface）标识与对象

- **接口 ID** 使用模块 ID 词法：`^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)*$`，例如
  `glide.terminal.session`；空串与非法字符按 `schema-invalid-identifier`。
- **接口版本**为精确 `MAJOR.MINOR.PATCH`，不支持范围、通配或预发布后缀
  （`schema-invalid-version`）。
- 接口身份是 `(interface_id, interface_version)`。同一身份可由多个模块声明（多实现）。
- v2.0 的接口只是**身份标记**：不声明方法、参数、返回类型或 Port 形状；不参与 Connection
  类型推导。
- 对象标识：

| 对象 | `object_id` |
|---|---|
| 接口声明 | `interface:<module-id>@<module-version>#<interface-id>@<interface-version>` |
| 接口依赖要求 | `dependency:<module-id>@<module-version>#<requirement-id>`（沿用 v1） |

## 3. `module.toml` 增量

v2 模块（`[module] schema_version = 2`）在 v1 契约之上增加或扩展以下两处；其余字段、
默认值与约束不变。

### 3.1 `[[contract.interfaces]]`

```toml
[module]
schema_version = 2
id = "glide.platform.conpty-session"
version = "1.0.0"
kind = "primitive"

[[contract.interfaces]]
id = "glide.terminal.session"
version = "1.0.0"
```

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `id` | string | 是 | — | 接口 ID 词法 | `schema-missing-field` / `schema-invalid-identifier` |
| `version` | string | 是 | — | 精确版本 | `schema-missing-field` / `schema-invalid-version` |

- 同一模块内 `(id, version)` 唯一；重复声明报 `id-duplicate-interface`。
- 同一模块可以声明同一 `id` 的多个版本（视为实现多个接口版本）；不同模块可以声明同一身份。
- 表可缺省或为空数组；`primitive` 与 `composite` 都可声明接口。
- 未知键为 `schema-unknown-field`。
- 该表只在模块 `schema_version = 2` 时存在；v1 模块中出现按 v1 的未知字段处理。

### 3.2 依赖目标：`module` 与 `interface` 二选一

v1：

```toml
[[contract.dependencies]]
id = "state"
module = "task.state"
version = "1.0.0"      # 可选：模块精确版本
optional = false
```

v2.0 接口目标：

```toml
[[contract.dependencies]]
id = "session-factory"
interface = "glide.terminal.session"
version = "1.0.0"      # 可选：接口精确版本
optional = false
```

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `id` | string | 是 | — | 局部 ID；模块内依赖唯一 | `schema-invalid-identifier` / `id-duplicate-dependency` |
| `module` | string | 二选一 | — | 模块 ID 词法 | `schema-invalid-identifier` |
| `interface` | string | 二选一 | — | 接口 ID 词法；仅 v2 模块 | `schema-invalid-identifier` |
| `version` | string | 否 | 无 | 精确版本；`module` 目标为模块版本，`interface` 目标为接口版本 | `schema-invalid-version` |
| `optional` | boolean | 否 | `false` | — | `schema-type-mismatch` |

规则：

- `module` 与 `interface` **恰好一个**：两个都写为 `schema-inconsistency`，都不写为
  `schema-missing-field`。
- `interface` 目标必须出现在 `schema_version = 2` 的模块中；v1 模块写 `interface` 属于
  v1 未知字段语义，按 `schema-unknown-field` 拒绝。
- `version` 在 `interface` 目标下可省略：省略表示接受该接口的任意已声明版本（仍受
  §6 的歧义规则约束）；出现时按精确版本匹配。
- `module` 目标的解析与 v1 完全一致，不因项目是 v2 而改变。

## 4. Composition 增量

### 4.1 `[[instances]] scope`

```toml
[[instances]]
id = "conpty-session"
module = "glide.platform.conpty-session"
version = "1.0.0"
scope = "service"
```

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---:|---|---|---|
| `scope` | string | 否 | `structure` | `structure` 或 `service` | `schema-invalid-value` |

- `scope` 只在 v2 Composition（`schema_version = 2`）中定义；v1 Composition 中出现按
  v1 未知字段处理（`schema-unknown-field`），v1 行为不变。
- 省略时等价于 `structure`，规范化模型中显式输出（§9）。

### 4.2 structure 与 service

| 方面 | `structure` | `service` |
|---|---|---|
| Binding 树 | 参与；沿用 v1 BND-1..BND-8 与可达性 | **不参与**；不得有父/子 Binding |
| Root | 可以是 Root（Root 必须是 structure） | 不得是 Root |
| 模块 kind | 任意（Root 必须是 composite） | 必须是 `primitive` |
| 配置 / 依赖 / Port / Connection | 支持 | 支持（Port 与 Connection 正常校验） |
| 依赖提供者 | 可以是提供者 | 可以是提供者 |
| Slot 基数 / 可达性 | 校验 | 不校验（无 Binding） |

- `service` 的删除、重命名、换模块仍走现有命令；是否仍有效由候选模型全量校验决定。
- `service` 不因「不在 Binding 树中」产生 `structure-unreachable-instance`。
- `service` 必须使用 `primitive` 模块（v1 MOD-2：primitive 不得声明 Slot），因此 service
  不存在可通过 Binding 满足的 Slot；composite 模块若作为 service，其结构无法表达，
  按 §5 INS-7 拒绝。
- `service` 可以是显式依赖 provider，也可以被自动解析选中；可以出现在 Connection 两端。

## 5. 不变量增量

以下 ID 与 v1 不变量并列；未列出的 v1 不变量继续适用。

| ID | 不变量 | 层 | 错误代码（首个） |
|---|---|---|---|
| MOD-9 | 接口声明合法，`(id, version)` 模块内唯一 | L2 / L3 | `schema-invalid-identifier` / `schema-invalid-version` / `id-duplicate-interface` |
| MOD-10 | 依赖目标 `module` 与 `interface` 恰好一个 | L2 | `schema-inconsistency` / `schema-missing-field` |
| INS-6 | `scope` 合法；省略为 `structure`；仅 v2 定义 | L2 | `schema-invalid-value` |
| INS-7 | `service` 实例的模块必须是 `primitive` | L4 | `contract-service-module-invalid` |
| BND-9 | `service` 不参与任何 Binding（父或子） | L5 | `structure-service-in-binding` |
| BND-10 | Root 实例必须是 `structure` | L5 | `structure-root-scope-invalid` |
| DEP-6 | 接口依赖按接口身份解析，不回退到模块匹配 | L6 | `ref-interface-not-found` / `dependency-unresolved` / `dependency-optional-unresolved` / `dependency-ambiguous` |
| DEP-7 | 显式 provider 必须声明所需接口（及要求的精确版本） | L6 | `dependency-interface-mismatch` |
| DEP-8 | 未被任何依赖或连接引用的 service 给出 warning | L6 | `service-unused`（warning） |

## 6. 依赖解析（L6）

对每个实例的每条依赖要求：

1. **要求声明**：`id` 必须在模块契约中声明（沿用 v1 `contract-dependency-unknown`）。
2. **目标类型**：
   - `module`：完全沿用 v1 §7.3 的显式/自动解析、版本匹配与诊断。
   - `interface`：按下述规则；**不得**回退为模块匹配，也不得按实现模块名猜测。
3. **接口存在性**（L3）：若注册表中没有任何模块声明该接口身份（带 `version` 时按
   `(interface, version)` 精确判断，省略时只按 ID），报 `ref-interface-not-found`，
   跳过该要求的后续解析（不重复报 `dependency-unresolved`）。
4. **显式 provider**（`[instances.dependencies]`）：
   - provider 实例不存在 → `ref-instance-not-found`；
   - provider 是自己 → `dependency-self`；
   - provider 的模块未声明该接口，或要求带 `version` 而声明版本不等 →
     `dependency-interface-mismatch`；
   - 通过后解析成功。
5. **自动解析**：
   - 候选 = 所有实例（structure 与 service）中，其模块声明了该接口、且（要求带版本时）
     接口版本精确相等的实例，排除自己；
   - 恰好 1 个 → 解析成功；
   - 0 个 → `optional = true` 时 `dependency-optional-unresolved`（warning），否则
     `dependency-unresolved`（error）；
   - 多个 → `dependency-ambiguous`（error），要求显式指定 provider。
6. 解析结果写入规范化模型；不参考 Binding。

- 接口版本省略时，同一实例即使声明多个版本仍只算一个候选；候选之间按实例计数。
- 依赖图仍允许成环（v1 DEP-3）。

## 7. 结构与可达性（L5）

在 v1 结构校验之上：

1. Root 的 `scope` 必须是 `structure`；否则 `structure-root-scope-invalid`。
   Root 的 `composite` 要求（v1 INS-5）独立检查，可能同时报告。
2. 任何 Binding 的 `parent` 或 `child` 是 service → `structure-service-in-binding`
   （每个 Binding 报一次），该 Binding **不参与**父级计数、Slot 基数、环与可达性计算。
3. 非 Root 的 structure 实例仍必须有且只有一个父 Binding（v1 BND-4）。
4. 可达性只针对 structure 实例；service 不因不可达报错。
5. Slot 基数只统计 structure 子实例；指向 service 的 Binding 已被上一条排除。
6. 被排除的 Binding 仍可能使其 structure 端点产生基数/可达性诊断；本规范接受这种
   独立报告，不额外抑制。

## 8. 命令

命令集合不变（15 条），只扩展一条命令的字段：

- `create_instance` 增加可选 `scope`（`structure` / `service`，默认 `structure`）。
  - v2 项目：写入 `scope`；`service` 的模块 kind 校验由候选模型给出
    `contract-service-module-invalid`。
  - v1 项目：显式传 `scope`（即使值为 `structure`）→ `edit-invalid-command`，
    因为 v1 文件无法表达该字段；省略时行为不变。
- `update_instance_module`：换成非 `primitive` 模块而实例仍为 `service` 时，候选模型报
  `contract-service-module-invalid`。
- `delete_instance`：级联规则不变；删除 service 可能使依赖/connection 失效，由候选模型
  给出真实诊断。
- `set_dependency_provider` / `unset_dependency_provider`：命令形态不变；接口依赖的
  provider 有效性由候选模型判定（`dependency-interface-mismatch`）。
- `replace_source`：文本按当前项目版本校验；v2 项目的文本必须满足本规范。
- 不新增命令；UI 的 Services 视图属于 034，不引入新事务语义。

命令 Schema 的权威增量见 [`command-schema-v1.md`](./command-schema-v1.md) §2 的 v2.0 注记。

## 9. 规范化模型

v2 项目在 v1 规范化输出上增加：

- `modules[].contract.interfaces`：`[{ "id": ..., "version": ... }]`，按 `(id, version)` 升序；
  无接口时为空数组。
- `instances[].scope`：`"structure"` 或 `"service"`，始终显式输出。
- `dependencies[].target`：`{ "kind": "module" | "interface", "id": ..., "version": string | null }`。

其余字段、排序键、`source` 范围与确定性要求不变（v1 §8）。

- v1 项目的规范化 JSON **不增加**上述字段，保持与 v1 字节级输出一致。
- v2 输出仍要求相同输入产生字节级相同 JSON，不依赖哈希表或目录顺序。

## 10. 校验顺序与级联

```text
L1 文件与语法（不变）
  → L2 Schema（含 v2 版本矩阵与新增字段）
  → L3 标识与引用（含接口注册表存在性）
  → L4 契约与配置（含 service 模块 kind）
  → L5 结构（含 scope/Binding 约束）
  → L6 连接与依赖（含接口解析、service-unused）
  → L7 整体一致性（不变）
```

级联：

1. 项目版本不是 `1`/`2` → `schema-unsupported-version`，停止。
2. v2 项目 Composition 版本不匹配 → 报 `schema-version-mismatch`，跳过 Composition 后续检查。
3. 模块 L1/L2 失败 → 不进入注册表，其接口声明不生效；引用该模块的实例沿用 v1 级联，
   不产生额外 `ref-*`；以该接口为目标的依赖报 `ref-interface-not-found`。
4. 接口声明无效（L2）→ 该模块不注册；不单独为接口声明补发下游错误。
5. 显式 provider 未实现接口 → 只报 `dependency-interface-mismatch`，不再报
   `dependency-unresolved`。
6. 指向 service 的 Binding → 只报 `structure-service-in-binding`，该 Binding 不参与后续结构计算。
7. `service-unused` 在依赖解析与 Connection 校验完成后计算，每个未使用 service 一条。

## 11. v1 兼容、迁移与拒绝

- v1 项目与 v1 模块的行为、诊断代码与规范化输出不变；`schema_version = 1` 仍是默认。
- **不自动迁移**：Core 不把 v1 文件改写成 v2，也不在读取时推断升级。
- v2 项目不会把 v1 模块升级为 v2；v1 模块保持无接口、模块目标依赖。
- v2 模块出现在 v1 项目 → 该模块 `schema-unsupported-version`；项目不会因此自动升级。
- 迁移只能由用户显式修改 `schema_version` 与字段完成；迁移工具不在 v2.0 承诺范围。
- Source 编辑（`replace_source`）必须保持当前项目版本对应的格式；把 v2 字段写进 v1 项目
  会被候选校验拒绝。

## 12. 不包含（v2.0）

| 能力 | v2.0 行为 |
|---|---|
| 项目 Profile / 多 Composition 入口 | 不实现；`[profiles]` 为 `schema-unknown-field`；属 v2.1 / S2 |
| 接口方法、参数、返回类型、Port 形状 | 不实现；接口只是身份标记（UW-032） |
| 版本范围、通配、自动选择最新版本 | 不支持；仍是精确版本 |
| 按实现模块名或命名约定推断接口实现 | 禁止；只按接口身份匹配 |
| service 内部组合（composite service） | 禁止；service 模块必须是 `primitive`（UW-032） |
| 模块文件编辑 | 仍只读（UW-031） |
| 自动迁移 / 双版本同时写入 | 不做 |
| 运行时、构建、部署、代码生成 | 不属于本项目 |

## 13. 样例计划（035）

035 需要新增的机器可读数据集（目录名与 `expected.json` 在 035 冻结）：

| 场景 | 预期 |
|---|---|
| v2 合法：接口 + service 提供者 | `valid: true` |
| 接口依赖无任何声明 | `ref-interface-not-found` |
| 同模块重复声明同一接口版本 | `id-duplicate-interface` |
| 依赖同时写 `module` 与 `interface` | `schema-inconsistency` |
| service 作为 Binding 父/子 | `structure-service-in-binding` |
| Root 为 service | `structure-root-scope-invalid` |
| service 使用 composite 模块 | `contract-service-module-invalid` |
| v2 项目 Composition 仍写 `1` | `schema-version-mismatch` |
| 显式 provider 未实现接口 | `dependency-interface-mismatch` |
| 未被引用的 service | `service-unused`（warning） |
