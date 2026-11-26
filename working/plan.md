# Working Plan — Composition Designer v0.1

> 本文件是**唯一的“下一步行动”入口**。开始行动前先读这里，只从行动队列取一件事做；
> 做完后回写状态、补行动日志，再规划下一步。
>
> - 设计基线：`composition-designer-v0.1-design.md`（Draft；修改需 Rearchitect 授权）
> - 冻结规范：`docs/spec/`（M0 产物，修改需 Rearchitect）
> - 任务卡（每次行动）：`working/tasks/NNN-<slug>.md`，模板 `working/tasks/000-template.md`
> - 过程文件（日志、草稿、临时产物）：`working/tmp/`（不提交）
> - 工作规则：`AGENTS.md`

---

## 1. 目标与边界

**目标**：v0.1 能**正确描述、修改、解析和验证一个完整项目**——TOML 声明 → 规范化
Composition Model → 可定位诊断 → 原子编辑事务 → Designer 与 Agent 共用同一 Core。

**边界**：

- Composition Model 是唯一权威模型；Canvas / TOML Source / Agent / CLI 只能通过 Core 读写；
- TOML 是持久化声明格式，不是执行语言；v0.1 只解析与验证，不运行项目；
- Binding（结构组合）、Dependency（依赖要求）、Connection（数据/控制）必须分离；
- 明确不做：运行/构建/部署/代码生成、在线模块市场、复杂版本求解、隐式 Adapter、
  多 Agent 调度、在 TOML 中执行脚本；
- 验收以设计基线 §12 的 A01–A18 为准，不以“画布能显示”为标准。

## 2. 当前状态（2026-11-26）

- 阶段：`M0` 完成——领域模型、TOML Schema、错误码与样例集已冻结；M1 Core 未开始
- 已有：设计基线、`working/` 编排、`AGENTS.md`、Change Manifest 模板、`docs/spec/` 三份冻结规范、
  `docs/glossary.md`、ADR-0001/0002、26 个样例数据集、本地 git（main）
- 未建：Rust workspace、Core、CLI、Designer、Agent Adapter、M1 门禁
- 门禁：**暂无**。M0 以规范 + `expected.json` 核对；M1 起为
  `cargo fmt/clippy/test` + `composition-cli validate`
- 规模：0 crate / 0 命令 / 26 个样例数据集（25 个完整项目 + 1 个入口缺失目录）

## 3. 阶段路线图

设计基线 §11 的 M0–M5 与 §16 的执行顺序。一次只推进一张卡；后续阶段的卡在进入该阶段时创建。

| 阶段 | 状态 | 目标 | 退出条件 | 任务卡 |
|---|---|---|---|---|
| M0 | ✅ 完成 | Schema 与语义冻结 | 不同实现者只依赖规范与样例也能判断项目是否合法 | 002 领域模型、003 TOML/契约 Schema、004 示例项目集 |
| M1 | 未开始 | TOML Engine + Core | Core 能独立稳定解析并给出可复现验证结果，不依赖 GUI/Agent | 005 Rust 工程与领域类型、006 TOML 加载与 Source Map、007 解析与 L1–L7 校验、008 CLI 与确定性 |
| M2 | 未开始 | 统一编辑命令与事务 | 非法变更整体失败；合法变更只改预期声明；旧 Revision 不覆盖新修改 | 009 命令 Schema 与事务、010 Revision/冲突/结构化 Diff、011 保留格式的 TOML 编辑 |
| M3 | 未开始 | 可视化 Designer | GUI 无独立模型，所有编辑经 Core，画布布局不改语义 | 012 工程打开与 Designer 外壳、013 Canvas 与 Tree、014 Inspector/Source/Diagnostics、015 外部修改同步 |
| M4 | 未开始 | Agent Interface | Agent 不直接改文件即可完成可复现的组合修改 | 016 领域协议与 CLI/JSONL、017 MCP Adapter、018 Agent Skill 与端到端 |
| M5 | 未开始 | 完整性与稳定性 | 设计基线 §12 A01–A18 全部通过 | 019 多文件/性能/崩溃恢复/迁移与验收报告 |

> 卡号为路线图预留；创建时以实际内容为准，不改已存在的编号。
> M1 的 005–008 尚未创建，进入 M1 时按 `000-template.md` 逐张创建。

## 4. 下一步行动队列

| # | 卡 | 方向 | 说明 | 所需授权 |
|---|---|---|---|---|
| 1 | 005 | Rust 工程与领域类型 | 建立 Cargo workspace（crate 拆分决策）、domain 类型、错误码枚举、测试骨架；先让 `examples` 可被加载 | Extend（新 crate/工程配置） |
| 2 | 006 | TOML 加载与 Source Map | 用 `toml_edit` 加载三类声明文件，保留位置；实现 `source_digest` 与稳定排序基础 | Modify |
| 3 | 007 | 解析与 L1–L7 校验 | 模块注册、实例/Binding/Connection/依赖解析、规范化模型与诊断；以 26 个样例为断言 | Modify |
| 4 | 008 | CLI 与确定性 | `inspect` / `resolve` / `validate`、JSON 输出、稳定性与 golden 测试 | Extend（CLI 入口） |

> 队列规则：一次只推进一张任务卡；没有用户确认的方向不写入队列，也不开工。
> 005–008 为路线图卡号，创建卡片后再开始；完成一张再创建下一张。

## 5. 验收映射（设计基线 §12）

M0 已冻结判断标准与样例；实际验证从 M1 开始。

| 验收 | 项 | 首次可验证卡 |
|---|---|---|
| A01 | 项目加载 | 006 |
| A02 | TOML 解析 | 006 |
| A03 | Schema 验证 | 007 |
| A04 | 模块解析 | 007 |
| A05 | 引用完整性 | 007 |
| A06 | 组合合法性 | 007 |
| A07 | 配置验证 | 007 |
| A08 | 依赖检查 | 007 |
| A09 | 确定性 | 008 |
| A10 | 编辑一致性 | 012 / 016（同一 Core 命令） |
| A11 | 事务安全 | 009 |
| A12 | Revision 冲突 | 010 |
| A13 | TOML 保留 | 011 |
| A14 | 外部变更 | 015 |
| A15 | Canvas 隔离 | 013 |
| A16 | Agent 闭环 | 018 |
| A17 | 错误可操作 | 007 起持续；019 汇总 |
| A18 | 无隐式兼容 | 全程；007 / 009 / 019 重点检查 |

## 6. 待冻结决策（设计基线 §15）

前 7 项已由 002 / 003 冻结（依据 ADR-0001 与 `docs/spec/`）；后 2 项留待 M2 / M3。

| 决策项 | 结论 | 冻结位置 | 冻结时需要明确 |
|---|---|---|---|
| 模块版本策略 | 精确版本，不支持范围/预发布 | 003 / ADR-0001 | 已明确 |
| 模块来源 | 项目内本地目录，`module_roots` 递归扫描 | 003 / ADR-0001 | 已明确 |
| Composition 拆分 | 单入口，不支持 include/`parts/` | 003 / ADR-0001 | 已明确 |
| Slot 基数 | `one` / `optional_one` / `many`，`required` 组合固定 | 003 / ADR-0001 | 已明确 |
| Port Connection | 简单方向/类型/协议/multi 校验；不协商 | 003 / ADR-0001 | 已明确 |
| Dependency | 模块 ID 依赖；实例映射或唯一匹配；允许环 | 003 / ADR-0001 | 已明确 |
| 默认值 | 解析时计算，不写回源文件 | 003 / schema §5.3 | 已明确 |
| Revision | `seq` + `source_digest`（声明文件字节） | 003 / schema §9；持久化实现留 010 | 010 |
| 模块定义编辑 | 初期只读 | 003 / ADR-0002；入口决策留 012 | 012 |

未冻结的能力必须显式标记 unsupported，不用隐式行为代替。

## 7. 已知限制

- M0 产物尚未被 Core 执行：规范与样例已互相核对，但真正的解析/验证要等 M1；
- 当前仍无代码与门禁：任何“验证通过”不能声称来自 Core；M1 建立 `cargo` 门禁；
- 设计基线仍是 Draft：字段以 `docs/spec/` 为准，修改设计基线需 Rearchitect 授权；
- 宿主任务表目前只有 selftest 夹具；M1 的 `cargo` 命令需要新增 dynamic host task（人工批准），
  或在宿主侧执行，不要用沙箱低完整性进程跑 Rust 构建；
- 多文件 Composition 首版只支持单入口；`object` 属性、预发布版本、能力依赖、协议协商等
  见 `docs/spec/composition-schema-v1.md` §12；
- `edit-cases/` 与 `external-change-cases/` 目前只有场景说明，M2 / M3 补请求载荷与操作序列；
- v0.1 不实现运行时、构建、部署、代码生成与在线模块市场；设计器预览只代表解析结构。

## 8. 执行规则

1. **一次只推进一张任务卡**；卡内清单勾完、验收条件满足才算 done。
2. 每张卡开始前把状态改成 `in_progress`；结束时写“结果 / 遗留问题”。
3. 过程文件一律放 `working/tmp/`，可随时删除；不要污染 `crates/`、`apps/`、`examples/`。
4. 修改范围遵循 Change Manifest：普通实现只动目标 crate 内部与测试；
   领域模型 / Schema / 命令协议 / 设计基线变化必须 `extend` / `rearchitect`。
5. 构建 / 测试通过宿主机任务执行；分析、编辑在沙箱内完成。
6. 阶段完成时更新本文件第 2、3、4 节与行动日志，保持“下一步”只有少数几项。

## 9. 快速命令

```powershell
# M0 产物（当前）
Get-Content working/plan.md
Get-Content docs/spec/composition-schema-v1.md

# M1 起（规划，包名与结构在 005 冻结前只是占位）
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p composition-cli -- validate examples/minimal-project
```

## 10. 行动日志

| 时间 | 卡 | 结果 |
|---|---|---|
| 2026-11-26 | 001 | 建立 working 工作模式（plan / tasks / tmp）、AGENTS 规则、Change Manifest 模板、AI 入口与基础目录；无代码、无门禁 |
| 2026-11-26 | git | 初始化本地仓库（main），首个提交 `58713e2`（bootstrap） |
| 2026-11-26 | 002 | 冻结 9 类对象、48 条编号不变量、Binding/Dependency/Connection 对照与对象标识；产出 `domain-model-v1.md`、`glossary.md`、ADR-0001/0002 |
| 2026-11-26 | 003 | 冻结 `project.toml` / Composition / `module.toml` / `.designer` 字段表、L1–L7 校验顺序、Revision 摘要；63 个验证代码 + 6 个预留代码 |
| 2026-11-26 | 004 | 建立 26 个数据集（25 个完整项目 + 入口缺失目录），覆盖设计基线 §12.1 的 12 类；临时校验器 26/26 与 `expected.json` 匹配；编辑/外部修改场景留 M2/M3 |
