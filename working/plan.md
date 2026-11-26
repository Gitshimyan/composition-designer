# Working Plan — Composition Designer v0.1

> 本文件是**唯一的“下一步行动”入口**。开始行动前先读这里，只从行动队列取一件事做；
> 做完后回写状态、补行动日志，再规划下一步。
>
> - 设计基线：`composition-designer-v0.1-design.md`（Draft；修改需 Rearchitect 授权）
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

- 阶段：`BOOTSTRAP`——设计基线就绪，只有 working 编排、工作规则与目录骨架
- 已有：设计基线（Draft）、`working/` 编排、`AGENTS.md`、Change Manifest 模板、
  `docs/ai/` 入口、`docs/spec/` 占位、`crates` / `apps` / `examples` 目录
- 未建：Rust workspace、领域模型与 TOML Schema 冻结、示例项目、Core、CLI、Designer、Agent Adapter
- 门禁：**暂无**。M0 以文档与样例核对、任务卡验收为准；M1 起为
  `cargo fmt/clippy/test` + `composition-cli validate`
- 规模：0 crate / 0 命令 / 0 示例项目

## 3. 阶段路线图

设计基线 §11 的 M0–M5 与 §16 的执行顺序。一次只推进一张卡；后续阶段的卡在进入该阶段时创建。

| 阶段 | 目标 | 退出条件 | 任务卡 |
|---|---|---|---|
| M0 | Schema 与语义冻结 | 不同实现者只依赖规范与样例也能判断项目是否合法 | 002 领域模型、003 TOML/契约 Schema、004 示例项目集 |
| M1 | TOML Engine + Core | Core 能独立稳定解析并给出可复现验证结果，不依赖 GUI/Agent | 005 Rust 工程与领域类型、006 TOML 加载与 Source Map、007 解析与 L1–L7 校验、008 CLI 与确定性 |
| M2 | 统一编辑命令与事务 | 非法变更整体失败；合法变更只改预期声明；旧 Revision 不覆盖新修改 | 009 命令 Schema 与事务、010 Revision/冲突/结构化 Diff、011 保留格式的 TOML 编辑 |
| M3 | 可视化 Designer | GUI 无独立模型，所有编辑经 Core，画布布局不改语义 | 012 工程打开与 Designer 外壳、013 Canvas 与 Tree、014 Inspector/Source/Diagnostics、015 外部修改同步 |
| M4 | Agent Interface | Agent 不直接改文件即可完成可复现的组合修改 | 016 领域协议与 CLI/JSONL、017 MCP Adapter、018 Agent Skill 与端到端 |
| M5 | 完整性与稳定性 | 设计基线 §12 A01–A18 全部通过 | 019 多文件/性能/崩溃恢复/迁移与验收报告 |

> 卡号为路线图预留；创建时以实际内容为准，不改已存在的编号。

## 4. 下一步行动队列

| # | 卡 | 方向 | 说明 | 所需授权 |
|---|---|---|---|---|
| 1 | 002 | 领域模型冻结 | 统一术语，冻结 Project / Module Definition / Instance / Binding / Dependency / Port / Connection / Diagnostic / Revision 的语义与不变量 | Modify（docs） |
| 2 | 003 | TOML 与契约 Schema | 冻结 `project.toml` / `composition/**` / `module.toml` / `.designer/**` 的字段、默认值与错误码，产出 `docs/spec/composition-schema-v1.md` | Modify（docs） |
| 3 | 004 | 示例项目集 | `examples/` 下建立最小合法、嵌套、多实例与各类非法/边界项目，覆盖设计基线 §12.1 的 12 类样例 | Extend（examples） |

> 队列规则：一次只推进一张任务卡；没有用户确认的方向不写入队列，也不开工。
> M0 三张卡完成后，再创建 M1 的 005–008。

## 5. 验收映射（设计基线 §12）

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

| 决策项 | 初始建议 | 冻结卡 | 冻结时需要明确 |
|---|---|---|---|
| 模块版本策略 | 精确版本 | 003 | 是否支持范围及范围语义 |
| 模块来源 | 项目内本地目录为主 | 003 | 是否支持外部只读模块目录 |
| Composition 拆分 | 先单入口，按明确规则扩展多文件 | 003 | include/import 语义、循环检测、覆盖规则 |
| Slot 基数 | one / optional / many | 003 | 精确枚举与数量上限 |
| Port Connection | 首版简单方向/类型校验，或明确推迟 | 003 | 类型标识、协议版本、环路规则 |
| Dependency | 显式引用，不做复杂自动求解 | 003 | 模块依赖与能力依赖的首版范围 |
| 默认值 | 解析时计算，不自动写回 | 003 | 来源、优先级、展示方式 |
| Revision | 内容摘要 + 提交序列 | 010 | 哈希范围、跨文件事务、历史持久化 |
| 模块定义编辑 | 初期只读 | 012 | 是否在同一 Designer 内编辑 |

未冻结的能力必须显式标记 unsupported，不用隐式行为代替。

## 7. 已知限制

- 当前没有任何代码与门禁：任何“验证通过”只能是文档/样例核对，不能声称构建或测试通过；
- 设计基线仍是 Draft：M0 冻结前字段与语义可能调整，但修改设计基线需 Rearchitect 授权；
- 宿主任务表目前只有 selftest 夹具；M1 的 `cargo` 命令需要新增 dynamic host task（人工批准），
  或在宿主侧执行，不要用沙箱低完整性进程跑 Rust 构建；
- 多文件 Composition 首版只支持单入口（由 003 确认是否改变）；
- v0.1 不实现运行时、构建、部署、代码生成与在线模块市场；设计器预览只代表解析结构；
- 仓库尚未 `git init`：跨文件差异、历史与 `change check` 在需要时再建立。

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
# M0（当前）：只有文档与样例
Get-Content working/plan.md

# M1 起（规划，包名与结构在 005 冻结前只是占位）
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p composition-cli -- validate examples/minimal-project
```

## 10. 行动日志

| 时间 | 卡 | 结果 |
|---|---|---|
| 2026-11-26 | 001 | 建立 working 工作模式（plan / tasks / tmp）、AGENTS 规则、Change Manifest 模板、AI 入口与基础目录；无代码、无门禁；M0 任务卡 002–004 就绪 |
