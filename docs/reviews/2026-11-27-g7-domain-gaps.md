# G7 — 接口身份、服务容器与项目 Profile 设计研究

- 日期：2026-11-27
- 状态：研究完成；**决策待用户批准**
- 关联：[ADR-0004](../adr/0004-schema-v2-direction.md) · [Glide 试点](../../working/pilots/glide-task-terminal/README.md) · [G07 任务卡](../../working/tasks/G07-domain-gaps.md)
- 性质：设计研究，不修改 Schema、领域模型或产品代码

## 1. 结论摘要

G1、G2、G3 不是三个可以随手加入的独立字段，而是一次**Schema 演进**：

- **v2.0**：接口身份（G1）+ 服务实例 scope（G2）；
- **v2.1**：项目级 Profile / 多 Composition 入口（G3，依赖 UW-018）；
- v1 项目继续可用，不做自动迁移；v2 通过 `schema_version` 显式启用；
- G4–G8 仍不进入本轮，避免把 kind/技术元数据/Port wire 一起绑进 v2。

推荐的产品顺序：

```text
G7 研究（本文件）
  → M7 Source Authoring（v1 文本编辑，schema-aware 设计）
  → S1 Schema v2.0（G1+G2）
  → M8 Agent Authoring（基于 v2 模型）
  → S2 Schema v2.1（G3 + 多入口）
  → H1 可靠性
  → M9 生态（模块库/模板；被 S1/S2 解锁）
```

M7 不依赖 Schema v2；S1 需要用户对领域模型的明确 Rearchitect 授权。
M9 不能先于 S1/S2，否则模块库/模板会建立在错误的依赖/入口语义上。

## 2. 证据与约束

### 2.1 当前模型

- Module Contract 定义 Property / Slot / Port / Dependency；
- Dependency 目标是**模块 ID + 精确版本**，不是接口；
- `[instances.dependencies]` 把 requirement 映射到 provider 实例；
- 未显式映射时，只有“恰好一个匹配实例”才自动解析，多个即 `dependency-ambiguous`；
- 单根 Binding 树要求所有实例从 root 可达（BND-7）；
- 项目只有一个 `workspace.composition` 入口；
- 实例 Config 是静态值，没有 profile/environment overlay。

### 2.2 Glide 试点证据

- `IProjectTaskRunner` / `ITerminalSessionFactory` 等端口被压成模块名；
- 为了让 service 实例可达，root 下人工增加 `runtime` Slot，把服务伪装成结构子节点；
- `glide.toml` 的 profile/desktop/headless/task 列表无法完整表达；
- 试点明确记录 G1/G2/G3，并声明不在 021 修改冻结 Schema。

### 2.3 不可妥协的约束

- Binding / Dependency / Connection 三类关系继续分离；
- 单一权威模型；所有入口经 Core/ProjectSession；
- 不引入静默兼容、自动 Adapter 或隐式迁移；
- v1 项目不能被新版本破坏；
- 任何 v2 语义都必须有正反例、诊断、SourceMap 与确定性测试。

## 3. G1 — 接口/契约独立身份

### 3.1 问题

Dependency 当前回答“我需要哪个模块”，而真实项目常常回答“我需要哪个**能力/接口**，
任意实现都可以”。模块 ID 作为依赖目标会把实现和消费者绑死。

### 3.2 选项

| 选项 | 做法 | 优点 | 代价 |
|---|---|---|---|
| A | 模块契约声明 `[[contract.interfaces]]`；依赖用 `interface = "..."` | 无需新文件；模块即实现；支持多实现 | 接口形状分散在各模块；需要约定唯一性 |
| B | 独立 `interfaces/*.toml` 定义接口 | 接口有中心定义、可演进 | 新文件集合、发现/版本/迁移成本高 |
| C | 保持模块 ID，新增 `provides` 标签，依赖仍按模块 | 改动小 | 不能真正解耦；标签与模块仍绑定 |
| D | 维持现状，用命名约定模拟接口 | 零改动 | 依赖歧义、实现替换、真实项目映射失真 |

### 3.3 推荐：A（模块契约内的接口标记）

v2.0 提案（未实现）：

```toml
# provider module.toml
[[contract.interfaces]]
id = "glide.terminal.session"
version = "1.0.0"

# consumer module.toml
[[contract.dependencies]]
id = "session-factory"
interface = "glide.terminal.session"
version = "1.0.0"   # 可选精确版本
optional = false
```

规则：

- 依赖目标二选一：模块（v1 兼容）或接口（v2）；
- provider 模块必须声明同名/同版本 interface；
- 显式 `[instances.dependencies]` 仍映射到实例；
- 自动解析从“匹配模块实例”改为“匹配实现该接口的实例”；
- 接口版本精确匹配，不做范围求解；
- 接口先作为**身份标记**，不承诺完整方法/端口形状；形状可在后续单独扩展。

### 3.4 影响

- Domain：新增 Interface 标记/解析结果；Dependency target 变为 enum（module/interface）；
- Schema v2：module contract 增加 `contract.interfaces`；dependency 增加 `interface`；
- 诊断：interface 未知、provider 未实现、接口版本不符；
- 命令：`set_dependency_provider` 不变；创建/修改依赖仍属于模块定义（当前只读）；
- UI：Dependency 视图显示 interface 与实现候选；Module 只读契约展示 interfaces；
- CLI/MCP：模型 JSON 增加 interface 信息；不新增写命令；
- 测试：多实现歧义、显式选择、v1 module 依赖回归。

## 4. G2 — 非结构 runtime/service 容器

### 4.1 问题

单根 Binding 树要求所有实例从 root 可达（BND-7）。服务/运行时实例不属于 UI 树，
却被强行塞进 root 的 `runtime` Slot，制造假结构。

### 4.2 选项

| 选项 | 做法 | 优点 | 代价 |
|---|---|---|---|
| A | 实例 `scope = "service"`；service 不参与 Binding 树 | 最小扩展；保留单根结构；UI 可独立 Services 视图 | 需要验证 service 不能有父级；命令增加 scope |
| B | Composition 级 `[[services]]` 表 | 语义显式 | 与 `[[instances]]` 重复；引用/序列化复杂 |
| C | 多根 forest | 灵活 | 破坏单根/BND-7，UI/验证/迁移大 |
| D | 保留人工 runtime Slot | 零改动 | 假结构，长期误导 |

### 4.3 推荐：A（实例 scope）

v2.0 提案：

```toml
[[instances]]
id = "conpty-session"
module = "glide.platform.conpty-session"
version = "1.0.0"
scope = "service"   # 默认 "structure"
```

规则：

- `scope = "structure"`（默认）：必须遵守现有单根 Binding 树；
- `scope = "service"`：不得有 Binding 父级/子级；不要求从 root 可达；
- root 必须是 structure；
- service 实例仍可声明 config、依赖、Port/Connection；
- service 实例的删除必须检查 provider/connection 引用；
- UI 增加 Services 列表/视图，不进入 Canvas；Dependency 视图可指向 service；
- 可选诊断：未被任何依赖/连接使用的 service 实例，warning 而不是 error。

### 4.4 影响

- Domain：Instance 增加 scope；Binding 不变量调整；
- Schema v2：`instances.scope` 可选，默认 structure；
- 命令：`create_instance` 需要可选 `scope`（Extend）；`delete_instance` 级联语义区分
  structure/service；
- UI：Services 视图、Inspector、删除影响；
- Loader/Core：BND-7 只约束 structure；service 不计入 binding 树；
- 测试：service 无父级、service 被删除时有引用、root 不可为 service。

## 5. G3 — 项目级 Profile / 条件组合

### 5.1 问题

Glide 的 profile（desktop/headless、console/file logger、task 列表）无法用单份
composition + 静态实例 config 表达。当前只把 active-profile 当作一个字符串。

### 5.2 选项

| 选项 | 做法 | 优点 | 代价 |
|---|---|---|---|
| A | 每个 profile 一份完整 Composition 入口 | 语义简单；无需条件语言；复用现有模型 | 重复结构；需要多入口；漂移风险 |
| B | 单 composition + profile overlay（config/provider/enable） | 共享结构 | 合并/验证/Diff 语义复杂；容易变成隐式兼容 |
| C | 模板/生成器按 profile 展开 | 减少重复 | 新生成层；不是声明式模型 |
| D | 维持 active-profile 字符串 | 零改动 | 不能表达真实差异 |

### 5.3 推荐：A（profile-scoped composition 入口，v2.1）

```toml
[workspace]
composition = "composition/main.toml"   # default

[profiles.desktop]
composition = "composition/desktop.toml"

[profiles.headless]
composition = "composition/headless.toml"
```

规则：

- profile 是项目级选择，不是实例属性；
- 每个 profile 是完整、可独立验证的 Composition；
- CLI/MCP/Designer 显式选择 profile；默认 profile 保持 `workspace.composition`；
- Source/Revision/Digest 以当前 profile 的 Composition + 共享 modules 为语义集合；
- `--all-profiles` 可做全量验证；失败诊断带 profile 名；
- 不做 profile 继承/overlay（至少 v2.1 不做），避免隐式合并。

### 5.4 影响

- 依赖 UW-018 多入口；Profile 与多入口是同一个 Rearchitect 包；
- Project schema v2.1 增加 `[profiles.*]`；
- Loader/Model 增加 profile 选择；
- CLI/MCP/UI 增加 profile 参数/选择器；
- Source 编辑必须编辑当前 profile 的入口；
- 测试：profile 隔离、默认 profile、缺失 entry、全量验证。

## 6. 组合方案与迁移

| 版本 | 内容 | 兼容性 |
|---|---|---|
| v1 | 现状 | 永久可读；不自动迁移 |
| v2.0 | G1 接口标记 + G2 service scope | 新字段可选；v1 依赖仍用 module |
| v2.1 | G3 profile + 多入口 | 新 `[profiles]`；v2.0 仍可读 |

迁移原则：

- 不自动改写 v1 文件；
- v2 项目显式写 `schema_version = 2`；
- 提供诊断而不是“猜测升级”；
- 旧项目可继续用 v1；需要接口/服务/Profile 时新建或手动迁移；
- 迁移工具/自动升级不在本轮承诺。

## 7. 对 M7 / M9 的影响

- **M7 Source**：不阻塞；Source 编辑可先做 v1 文本事务，设计时预留 schema-aware；
- **S1 v2.0**：M7 后执行，需要用户 Rearchitect 批准；M8 Agent 应基于 v2 模型；
- **S2 v2.1**：依赖多入口，排在其后；
- **M9 生态**：模块库/模板建立在接口身份、多入口之上，不能先于 S1/S2；
- **M9 范围缩减**：多入口移入 S2，M9 主要剩模块库/模板/Adapter。

## 8. 待决策

1. 是否接受 v2.0 = G1 + G2、v2.1 = G3 的分阶段方案？
2. 接口身份采用模块契约标记（选项 A），还是独立接口文件（选项 B）？
3. service 采用实例 scope（选项 A），还是 composition 级 services 表？
4. Profile 采用完整 Composition 入口（选项 A），还是 overlay（选项 B）？
5. S1 放在 M7 之后、M8 之前，还是提前到 M7 之前？

未获明确 Rearchitect 批准前，不修改 Schema/领域模型/设计基线。
