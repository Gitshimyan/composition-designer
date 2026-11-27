# 007 — M1 解析与 L1–L7 校验

- 状态：done
- 目标：新建 `composition-core`，在 006 的 `LoadedWorkspace` 之上解码 Composition / Module /
  Instance / Binding / Connection，执行 L1–L7 校验，产出规范化 `CompositionModel` 与稳定诊断；
  以 26 个 `examples/**/expected.json` 为断言。
- 授权：Extend（新 crate）
- 变更声明：`changes/007-composition-core.yaml`

- coverage_status：complete_with_deferred_items
- open_items：UW-027
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

005 定义了领域类型，006 提供了带 span 的 TOML 文档与 digest，但还没有任何语义解析或校验。
007 要把“声明文件 → 领域模型 → 诊断”打通，才能让 008 的 CLI 和 M2 的编辑事务有统一 Core。

crate 拆分决策：本卡先建立**一个** `composition-core`（内部按 `decode` / `validate` 模块隔离），
而不是立刻拆成 resolver + validator。两者共享解码上下文、Source Map 与级联规则，过早拆分会
增加跨 crate 传参成本；等 M2 需要独立复用时再按依赖方向拆分。

范围边界：

- 做：Composition / Module / 契约 / 实例 / Binding / Connection 解码；L2–L7 校验；
  Source Map、依赖解析、诊断排序、`CompositionModel`、`source_digest` 接入。
- 不做：CLI（008）、TOML 编辑 / 事务（M2）、GUI（M3）、Agent 协议（M4）。

## 清单

- [x] 根 workspace 加入 `crates/composition-core`（依赖 domain、toml、toml_edit）
- [x] `SourceMap`：Project / Composition / Module / Instance / Binding / Connection / 契约成员 → `SourceRef`
- [x] 解码 `module.toml`：module、description、properties、slots、ports、dependencies
- [x] 解码 Composition：composition、instances、config、dependency providers、bindings、connections
- [x] 模块注册表：`(id, version)` 唯一、重复检测、精确版本解析
- [x] L2：Schema 字段/类型/未知字段/ID/版本错误
- [x] L3：实例、Binding、Connection 的引用与唯一性
- [x] L4：模块契约自洽、实例配置、Slot accepts、依赖映射键
- [x] L5：Root、单父、重复 Binding、Slot 基数、环、可达性
- [x] L6：Connection 方向/类型/协议/重复/multi；依赖显式/唯一/歧义/可选
- [x] L7：`composition.id == project.id`
- [x] `CompositionModel` 规范化、诊断排序、`revision = seq 0 + source_digest`
- [x] 集成测试：26 个 `expected.json` 全量比对；合法项目模型可用；诊断确定性
- [x] `cargo test` / `clippy` / `fmt` 全绿；更新 plan、crates/README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**93 passed / 0 failed**
  （domain 58 + toml 单元 12 + toml 集成 15 + core 8）；
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check`：通过；
- 5 个合法项目：零诊断，`CompositionModel` 可构建且 `normalize` 稳定；
- 21 个非法项目：诊断代码集合与 `expected.json` 完全一致；
- 4 个无法构建模型的数据集（entry-missing、path-outside、toml-syntax、multiple-roots）
  仍只返回 loader 诊断；
- `effective_config` 会应用模块默认值，但不写回源文件；
- `SourceMap` 能定位实例与契约成员（模块文件）；
- 同一输入重复校验：诊断顺序、模型字段/数组顺序稳定；
- 不修改 TOML、不写文件、不提供 CLI。

## 结果 / 遗留问题

- 结果：新增 `crates/composition-core`：
  - `decode`：Module（module/description/properties/slots/ports/dependencies）与
    Composition（instances/config/dependency providers/bindings/connections）；
  - `validate`：模块注册、实例解析、L4 契约/配置、L5 结构、L6 连接/依赖、L7 ID、
    `CompositionModel`、诊断排序；
  - `SourceMap`：Project / Composition / Module / Instance / Binding / Connection / 契约成员；
  - 8 个集成测试，26/26 数据集代码集合完全匹配；workspace 共 93 项测试。
- 关键决策：
  - 单 `composition-core` 起步，内部模块隔离；resolver/validator 的拆分留到出现独立复用时；
  - 级联规则按 schema §10：loader 失败不进入解码，实例 schema 失败不产生 root/引用级联，
    环节点不重复报不可达；
  - `effective_config` 只收录已声明且通过校验的值，默认值来自模块契约；
  - 依赖解析显式优先、唯一自动匹配，provider 与 status 写入模型。
- 领域补全：`composition_domain::Instance` 增加 `effective_config` 字段（domain-model-v1
  §3.3 / schema §8.1 已要求），005 遗漏；007 一并补齐。
- 遗留：
  - 还没有 CLI（008）、TOML 编辑/事务（M2）、GUI（M3）、Agent 协议（M4）；
  - 模块语法错误与后续引用的级联抑制只做了基础处理，尚未有专门样例；
  - 重复 ID 的 SourceMap 以后出现的定义为准（模型保留首个），该边界未单独测试；
  - `composition-core` 目前未拆 resolver/validator，等 M2 视需要再拆。
- 下一步：008（CLI 与确定性），进入前创建任务卡与 CLI 入口决策。
