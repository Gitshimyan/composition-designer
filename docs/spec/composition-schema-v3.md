# Composition Schema v3（S2 / v2.1）

- 状态：S2 039 冻结（2026-11-28）
- 适用范围：`project.schema_version = 3` 的项目；v1/v2.0 项目继续由
  [`composition-schema-v1.md`](./composition-schema-v1.md)、
  [`composition-schema-v2.md`](./composition-schema-v2.md) 定义
- 决策：[ADR-0004](../adr/0004-schema-v2-direction.md)（方向）·
  [ADR-0007](../adr/0007-schema-v21-profiles.md)（冻结）
- 新增错误代码：[`error-codes-v3.md`](./error-codes-v3.md)
- 实现卡：040（Domain/Loader/Core/Session）、041（CLI/MCP/Designer/验收）

> v2.1 阶段只增加**项目级 Profile / 多 Composition 入口**。Schema v3 复用 v2.0 的接口、
> instance scope、结构/依赖/连接语义；`schema_version = 3` 标识新的项目格式。

## 1. 版本与文件兼容

### 1.1 `schema_version` 取值

| 文件 | v1 项目 | v2.0 项目 | v2.1 / v3 项目 |
|---|---|---|---|
| `project.toml` | `1` | `2` | `3` |
| Composition 入口 | `1` | `2` | `3` |
| `module.toml` | `1` | `1` 或 `2` | `1`、`2` 或 `3` |

- v1 项目：规则不变（Composition 1，模块 1）。
- v2.0 项目：规则不变（Composition 2，模块 1/2）。
- v3 项目：所有 Composition 入口都是 `3`；模块可为 `1`、`2` 或 `3`。
- v3 模块格式与 v2 模块相同（`contract.interfaces`、instance `scope` 等语义不变），
  仅版本号不同；v3 模块不得出现在 v1/v2.0 项目中。
- 项目版本决定规则集：`project=3` 时 Composition 必须是 `3`；`1`/`2` 报
  `schema-version-mismatch`，其他整数报 `schema-unsupported-version`。

### 1.2 兼容矩阵

| 项目 | Composition | 模块 | 结果 |
|---|---|---|---|
| 1 | 1 | 1 | v1，合法 |
| 2 | 2 | 1 或 2 | v2.0，合法 |
| 3 | 3 | 1 / 2 / 3 | v2.1，合法 |
| 3 | 1 或 2 | — | `schema-version-mismatch` |
| 3 | 其他 | — | `schema-unsupported-version` |
| 1/2 | 对应 v1/v2 | 3 | 模块 `schema-unsupported-version` |

### 1.3 语义文件集合与 Digest

对**所选 profile**：

- 语义文件 = `project.toml` + 所选 Composition 文件 + 所有已发现的 `module.toml`
  （v1/v2/v3 模块都包含）。
- 其他 profile 的 Composition 文件不读取、不参与 Digest、不参与当前诊断。
- `project.toml` 包含全部 profile 声明，因此新增/修改任何 profile 的路径都会改变 Digest。
- 拼接格式与 `source_digest` 算法不变（v1 §9）。
- `.designer/**` 仍不属于语义文件。

## 2. `project.toml`

```toml
[project]
schema_version = 3
id = "example.desktop"
name = "Example Desktop"

[workspace]
composition = "composition/main.toml"   # default profile
module_roots = ["modules"]

[profiles.desktop]
composition = "composition/desktop.toml"

[profiles.headless]
composition = "composition/headless.toml"
```

### 2.1 `[profiles]`

- 可选表；键是 Profile 名，值是表。
- Profile 名使用局部 ID 词法：`^[a-z][a-z0-9_-]*$`；不区分大小写之外的规范化，不做别名。
- `default` 是**保留名**；`[profiles.default]` 报 `profile-reserved-name`。
- 未知顶层表/键仍为 `schema-unknown-field`。

### 2.2 `[profiles.<name>]`

| 字段 | 类型 | 必需 | 默认 | 约束 | 错误代码 |
|---|---|---|---|---:|---|
| `composition` | string | 是 | — | 相对路径、`.toml`、项目根内、非空 | `schema-missing-field` / `schema-type-mismatch` / `schema-empty-value` / `file-path-*` |

- 每个 profile 表只允许 `composition`；其他键为 `schema-unknown-field`。
- 路径归一化规则与 `[workspace].composition` 相同；不得越出项目根。
- 不同 profile 可以指向同一个文件（别名允许）。
- Profile 名称非法：`schema-invalid-identifier`。

### 2.3 默认 profile

- `[workspace].composition` 必须存在，指向默认入口；
- 默认 profile 的名称在输出与选择参数中为 `"default"`；
- 不要求 `[profiles]` 非空；没有具名 profile 的 v3 项目等价于单入口项目。

## 3. 选择语义

- 选择参数 `profile`：可选字符串；省略或 `"default"` → 默认 profile。
- 具名 profile 必须存在于 `[profiles]`；否则 `profile-not-found`。
- 在 v1/v2.0 项目上显式选择 profile（包括 `"default"`）→ `profile-unsupported`。
  省略选择时行为不变。
- Profile 名非法 → `schema-invalid-identifier`。
- 选择在会话边界固定：CLI 单次命令、MCP 进程启动、Designer 切换时重开会话。
- 选择失败不加载任何 Composition，也不回退到默认 profile。

## 4. Composition 文件

v3 Composition 与 v2.0 相同：

- `[composition] schema_version = 3`；`id` 必须等于 `project.id`；`root` 必须存在；
- `[[instances]]` 支持 `scope = "structure" | "service"`；
- 接口声明/接口依赖、Connection、Binding、Slot 规则全部沿用 v2.0；
- 一个文件就是一个完整 Composition；不识别 include/import 或合并。

每个 profile 的 Composition 独立；不要求 profile 之间结构一致。模块集合通过
`module_roots` 共享。

## 5. Revision 与 Digest

- 初始 Revision：`seq = 0`，`source_digest` 覆盖 §1.3 的文件集合。
- 每个 profile 的会话独立；切换 profile 得到新的初始 Revision，不沿用旧 seq 或 digest。
- 成功事务 `seq + 1`；失败事务不改变 Revision。
- 选择失败时没有模型/Revision。
- 其他 profile 的 Composition 变化不会改变当前 profile 的 Digest，也不会触发外部冲突。

## 6. 事务与 Source

- 事务不带 `profile` 字段；会话已绑定所选 profile。
- `base_revision` 必须匹配当前 profile 的 Revision。
- 对象命令与 `replace_source` 只写所选 Composition 入口；
  其他 profile 文件不被读取或写入。
- 外部冲突检查只看当前 profile 的语义文件集合。
- 不存在跨 profile 事务；切换 profile 必须重新打开会话。

## 7. 规范化输出

v3 在 v2 输出上增加：

```json
{
  "project": {
    "schema_version": 3,
    "id": "example.desktop",
    "name": "Example Desktop",
    "composition_entry": "composition/headless.toml",
    "default_composition": "composition/main.toml",
    "active_profile": "headless",
    "profiles": [
      { "name": "desktop", "composition": "composition/desktop.toml" },
      { "name": "headless", "composition": "composition/headless.toml" }
    ],
    "module_roots": ["modules"]
  },
  "composition": { "schema_version": 3, "...": "selected composition only" }
}
```

- `profiles` 按 `name` 升序；不含默认 profile；
- `composition_entry` 是所选入口；
- `active_profile` 为 `"default"` 或具名 profile；
- 其余字段（modules/instances/bindings/connections/dependencies/diagnostics/revision）
  与 v2 相同；
- v1/v2.0 输出不增加这些字段。

## 8. 错误代码

新增：

| 代码 | 级别 | 触发 | 对象 | 提示 |
|---|---|---|---|---|
| `profile-not-found` | error | 选择的具名 profile 不存在 | `project` | 检查 `[profiles.<name>]` 或省略选择 |
| `profile-unsupported` | error | v1/v2.0 项目上显式选择 profile | `project` | 移除选择或升级为 v3 |
| `profile-reserved-name` | error | 声明 `[profiles.default]` | `project` | 删除该表；默认入口由 `workspace.composition` 提供 |

- CLI/MCP 在选择失败时以协议错误返回同一 code；Core 以诊断返回。
- 其他情况复用：`schema-invalid-identifier`、`schema-missing-field`、
  `schema-type-mismatch`、`schema-empty-value`、`schema-unknown-field`、`file-path-*`、
  `file-composition-missing`、`schema-version-mismatch`、`schema-unsupported-version`。

## 9. 校验顺序与级联

```text
L1 项目文件读取/语法
  → L2 project Schema、版本、profile 表/名称/路径、选择
  → L1 所选 Composition 读取
  → L2 Composition Schema
  → L3–L7（同 v2）
```

级联：

- 选择失败（未知/不支持/保留名/非法名）→ 停止，不读取 Composition；
- 所选 Composition 文件缺失 → `file-composition-missing`，停止；
- 其他 profile 的 Composition 不读取，不产生诊断；
- v3 项目 Composition 版本不匹配 → `schema-version-mismatch`，跳过 Composition 后续检查；
- 模块版本/接口/scope 的级联沿用 v2。

## 10. v1/v2.0 兼容与迁移

- v1/v2.0 项目的规则、诊断与输出不变；v2.0 二进制读 v3 → `schema-unsupported-version`。
- 不自动迁移：Core 不改写 `schema_version` 或新增 `[profiles]`。
- 迁移由用户显式完成：提升版本、把额外入口写入 `[profiles]`、保持默认
  `workspace.composition`。
- v3 项目可以引用 v1/v2/v3 模块；v3 模块不能在 v1/v2.0 项目中使用。

## 11. 不包含（v2.1）

| 能力 | v2.1 行为 |
|---|---|
| Profile overlay / 继承 / 条件实例 | 不实现；每个 profile 是完整 Composition |
| `[profiles.default]` | 保留名，拒绝 |
| per-profile `module_roots` | 不实现；模块共享 |
| `--all-profiles` 全量校验 | 不提供；每次选择一个 profile |
| 自动迁移 v1/v2.0 → v3 | 不做 |
| 运行时切换 / 多项目调度 | 不属于本项目 |
| 远程 Profile / 凭据 | 不实现 |

## 12. 041 验收计划

041 至少提供：

| 场景 | 预期 |
|---|---|
| v3 默认 profile | 按 `workspace.composition` 工作，`active_profile="default"` |
| v3 两个 profile | `--profile` 切换后模型/诊断/入口正确 |
| 具名 profile 缺失 | `profile-not-found` |
| v1/v2.0 上选择 profile | `profile-unsupported` |
| `[profiles.default]` | `profile-reserved-name` |
| profile 路径越界/缺失 | `file-path-*` / `file-composition-missing` |
| v3 项目 Composition 写 `2` | `schema-version-mismatch` |
| 事务只写所选入口 | 另一 profile 文件字节不变 |
| Source 只显示所选入口 | entry/text 与所选一致 |
| Designer 切换 profile | 重新加载、Revision/诊断刷新 |
| v1/v2.0 回归 | 输出与诊断不变 |
