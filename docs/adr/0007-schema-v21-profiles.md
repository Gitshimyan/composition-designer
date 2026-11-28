# ADR-0007 — Schema v2.1 冻结：项目级 Profile 与多 Composition 入口

- 状态：**Accepted（2026-11-28，039）**
- 授权：用户 2026-11-28 明确授权开始 S2（039–041）；见
  [`decision-history.md`](../../working/decisions/decision-history.md) D-029
- 关联：[ADR-0004](./0004-schema-v2-direction.md)（方向）·
  [`composition-schema-v3.md`](../spec/composition-schema-v3.md) ·
  [`error-codes-v3.md`](../spec/error-codes-v3.md) ·
  [`2026-11-27-g7-domain-gaps.md`](../reviews/2026-11-27-g7-domain-gaps.md)
- 性质：语义冻结，不改产品代码；040–041 实施

## 背景

ADR-0004 已确认 Profile 采用“每个 profile 一份完整 Composition 入口”，但只到方向层。
040/041 需要具体字段、版本号、选择语义、Revision/Digest 范围与错误码；否则 Loader、CLI、
MCP 与 Designer 会各自发明规则。本 ADR 记录 039 的冻结。

## 决策

### D1 — 新格式版本为 `schema_version = 3`

v2.1 阶段使用整数 `schema_version = 3`，而不是继续复用 `2`。理由：Profile 表是新的声明
格式；v2.0 的语义、输出与测试保持不变；旧二进制看到 v3 会以
`schema-unsupported-version` 明确拒绝，而不是把 `[profiles]` 当作未知字段。

- v3 项目：`project.toml` 与所选 Composition 为 `3`；
- v3 项目允许模块 `schema_version = 1`、`2` 或 `3`；v3 模块与 v2 模块语义相同；
- v1/v2.0 项目的版本矩阵不变。

### D2 — Profile 定义在 `project.toml`

```toml
[workspace]
composition = "composition/main.toml"   # default profile

[profiles.desktop]
composition = "composition/desktop.toml"

[profiles.headless]
composition = "composition/headless.toml"
```

- `[workspace].composition` 仍是默认入口，必须存在；
- `[profiles]` 可选；键为 Profile 名（局部 ID 词法）；
- 每个 profile 表恰好包含 `composition`；路径规则与 `workspace.composition` 相同；
- `default` 是保留名，`[profiles.default]` 报 `profile-reserved-name`；
- 不同 profile 可以指向同一文件（别名允许）。

### D3 — Profile 是完整 Composition，不是 overlay

每个 profile 指向完整、可独立解析/校验的 Composition；模块集合通过 `module_roots` 共享。
不做继承、覆盖、条件实例或运行时切换。差异通过不同 Composition 文件表达。

### D4 — 选择语义

- 未显式选择 → 默认 profile（`workspace.composition`）；
- 显式 `profile = "default"` 选择默认；
- 具名 profile 必须存在，否则 `profile-not-found`；
- 在 v1/v2.0 项目上显式选择 profile → `profile-unsupported`；
- Revision、Digest、Source、事务与诊断都只针对所选 profile；
- 选择在会话/进程边界固定：CLI 单次命令、MCP 进程启动参数、Designer 切换时重开会话。

### D5 — Revision 与 Digest 按所选 profile

语义文件集合 = `project.toml` + 所选 Composition 入口 + 所有已发现 `module.toml`。

- 其他 profile 的 Composition 文件**不参与**当前 Digest；
- `project.toml` 含全部 profile 声明，因此新增/改 profile 路径会改变 Digest；
- Digest 算法与 v1/v2 相同；切换 profile 得到新的初始 Revision（`seq = 0`），不沿用旧 seq。

### D6 — 事务与 Source 只作用于所选入口

- `base_revision` 属于所选 profile 的会话；
- 写回只修改所选 Composition；`replace_source` 文本是所选入口；
- 不存在“跨 profile 原子事务”，也不在事务内切换 profile。

### D7 — 规范化输出增量

v3 在 v2 输出上增加：

- `project.profiles`：具名 profile 数组 `[{name, composition}]`，按 `name` 升序（不含 default）；
- `project.default_composition`：`workspace.composition`；
- `project.active_profile`：`"default"` 或具名 profile；
- `composition_entry` 为**所选**入口；
- `model.composition` 只包含所选 Composition。

v1/v2.0 输出不变。

### D8 — 新增 3 个错误代码

`profile-not-found`、`profile-unsupported`、`profile-reserved-name`（均 L2，error）。
其余情况复用 `schema-invalid-identifier`、`schema-missing-field`、`schema-type-mismatch`、
`schema-unknown-field`、`file-path-*`、`file-composition-missing`、
`schema-version-mismatch`、`schema-unsupported-version`。

### D9 — v1/v2.0 兼容，不自动迁移

- v1/v2.0 文件继续可读、输出与诊断不变；
- v2.0 二进制读 v3 → `schema-unsupported-version`；
- v3 不自动改写旧文件；迁移只能由用户显式修改 `schema_version` 与 `[profiles]`；
- 未选择 profile 的 v3 项目仍按默认入口工作。

### D10 — 实现边界

039 只冻结语义；040 实现 Project/Loader/Core/Session；041 实现 CLI/MCP/Designer 选择与
fixtures/验收。不做 `--all-profiles`、overlay、per-profile module roots。

## 备选方案

| 方案 | 优点 | 缺点 | 未采用原因 |
|---|---|---|---|
| 继续用 `schema_version = 2`，把 `[profiles]` 作为可选扩展 | 不新增版本号 | 同一版本号对应两种格式；旧 v2.0 二进制只能靠未知字段报错 | 版本必须显式标识格式 |
| 独立 `profiles.toml` | project.toml 更小 | 新文件集合、发现/校验/迁移成本 | 项目级配置放 `project.toml` 足够 |
| overlay / 继承 | 减少重复 | 合并、验证、Diff 与 Source 语义复杂 | 容易变成隐式兼容 |
| 每个 profile 独立 module roots | 隔离更强 | 模块复用与 Digest 复杂 | 模块共享；差异用不同模块版本表达 |
| 自动迁移 v1/v2.0 → v3 | 用户省事 | 违反“不静默修复/升级” | 明确拒绝 |

## 后果

- 正面：多入口有唯一可实现契约；v1/v2.0 零改动；CLI/MCP/Designer 可用同一选择语义；
  Revision/Digest 范围明确。
- 成本：Project 模型与 Loader 增加 profile 选择；3 个新诊断；Designer 需要 profile 选择器；
  041 需要按 profile 的验收数据。
- 风险：profile 之间结构漂移（重复）；未选中的 profile 不参与校验，错误只在选中时暴露。
  真实使用出现“全量校验”需求时再评估 `--all-profiles`（不属 v2.1）。
- 复审条件：需要 overlay/继承、per-profile 模块、全量 profile 校验或运行时切换时重新评估。

## 关联

- 设计基线：§9 Agent Interface、§11 文件与关系、§16 实施顺序
- 相关规范：`docs/spec/composition-schema-v3.md`、`docs/spec/error-codes-v3.md`
- 任务卡：039（冻结）、040（实现）、041（入口与验收）
