# 001 — 建立 working 工作模式与 v0.1 基础目录

- 状态：done
- 目标：从 `E:\Work.Projects\Composition-First-clean` 取同一套 `working/` 编排方式，在本仓库建立
  v0.1 的基础目录、AI 工作规则与阶段计划。
- 授权：Rearchitect（初始化工程配置、工作规则与目录结构）
- 变更声明：无（初始化，无既有范围可声明）

## 背景

本仓库此前只有 `composition-designer-v0.1-design.md`。Composition-First 的 working 模式是：
`working/plan.md` 作为唯一行动入口，`working/tasks/NNN-*.md` 一次一张卡，`working/tmp/`
放过程文件；配合 `AGENTS.md` 的五级授权（Inspect / Modify / Compose / Extend / Rearchitect）
与 `changes/` 变更声明。v0.1 设计基线已给出 M0–M5 阶段，可直接映射成任务卡编号。

## 清单

- [x] 读取并提炼 Composition-First 的 working 模式（plan / tasks / tmp / AGENTS / changes / docs/ai）
- [x] 建立 `working/`（plan、模板、tmp）并写 v0.1 阶段路线图与验收映射
- [x] 建立 `AGENTS.md`：五级模式、默认禁止、变更声明、完成定义
- [x] 建立 `changes/`（模板 + 示例）与 `docs/adr/0000-template.md`
- [x] 建立 `docs/spec/`、`docs/ai/architecture-entry.md`
- [x] 建立 `crates/`（7 个目标 crate）、`apps/designer/`（6 个 UI 区域 + src-tauri）、`examples/`（3 类）
- [x] 建立 `README.md`、`.gitignore`、`.gitattributes`
- [x] 写 M0 任务卡 002 / 003 / 004，并回写 plan 日志

## 验收

- `working/plan.md` 是唯一行动入口，含目标、当前状态、路线图、队列、验收映射、决策、限制、规则与日志；
- `working/tasks/000-template.md` 与 `001`–`004` 齐备，编号与 plan 路线图一致；
- 目录骨架与设计基线 §10.2 的职责划分一致，且每个空目录有说明或占位；
- `AGENTS.md` 的五级授权能判定“改一个 crate 内部实现”与“改设计基线”分别需要什么授权；
- 没有代码、没有门禁：本卡不声称任何构建或测试通过。

## 结果 / 遗留问题

- 结果：working 编排、AGENTS、changes、docs（adr / ai / spec）与目录骨架就位；M0 三张卡待推进。
- 遗留：仓库未 `git init`（需要历史 / 差异检查时再建）；宿主任务表只有 selftest 夹具，
  M1 的 cargo 命令需以 dynamic host task 申请或在宿主侧执行。
- 下一步：002（领域模型冻结）。
