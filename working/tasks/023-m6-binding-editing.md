# 023 — M6-004 Binding 编辑

- 状态：done
- 目标：在 Designer 中完成 Slot 感知的 Binding 编辑——移动/改 Slot、可审查的 Detach、以及带必填 Slot 的复合模块子树创建——全部经统一事务与 Core 校验。
- 授权：Modify（Designer UI、Tauri host 测试、设计与协作文档）
- 变更声明：`changes/023-binding-editing.yaml`
- 依赖：022（事务/Review/影响展示基础）

- coverage_status：complete_with_deferred_items
- open_items：UW-023
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

022 已把 Instance 生命周期接入统一事务，但 Structure 仍然只能在“创建子实例”时形成首个
Binding：已有 Binding 不能换 parent/slot，也没有 Detach 入口；同时 022 只能创建不带必填
Slot 的模块，`ui.page` / `ui.panel` 这类需要子结构的复合模块无法落地。

M6-004 要求：选择父 Instance、选择 Slot、选择子 Instance，创建/修改/删除 Binding，
并展示 Slot 接受范围、Cardinality 与当前占用；GUI 不复制 Slot 规则，候选关系由 Core 验证。

## 范围与关键决策

1. **移动 Binding = `update_binding`**：子实例及其子树保持不变，编辑 parent + slot；
   child 在创建时选择。事务只发一条 `update_binding`。
2. **Slot 信息来自模块契约**：候选项按 `slot.accepts` 过滤，并展示 cardinality、required、
   当前占用与子节点；是否可绑定、是否超基数仍由 Core 最终判定。
3. **Detach 的现实语义**：v0.1 单根组合要求非 Root 实例可达，单独 `delete_binding`
   会让子实例不可达，Core 会以 `structure-unreachable-instance` 拒绝。UI 仍暴露
   Detach → Review 让用户看到真实诊断；要真正移除子结构请用 Delete（022）。
4. **必填 Slot 的子树创建**：模块的每个 required Slot 至少需要一个子实例。创建向导递归
   收集所有 required Slot 的模块/ID/配置，同一事务内按“先创建实例、再创建 Binding”提交；
   child 模块可以继续带 required Slot。可选 Slot 之后用 Add child 继续补。
5. **不做隐式修复**：不自动跳过必填 Slot，不自动替换模块，不猜测 Slot；失败由 Core 诊断。
6. **不新增命令、不改 Schema、不改设计基线**：复用 `create_instance` / `create_binding` /
   `update_binding` / `delete_binding`。

## 清单

- [x] 读 022 实现、设计基线 §7/§8、command-schema Binding 命令
- [x] 任务卡与 Change Manifest
- [x] `instanceEditing.ts`：requiredSlots、acceptedModules、parent/child binding 查询
- [x] CreateInstanceDialog：required Slot 递归子树向导（模块/ID/Binding ID/必填配置）
- [x] BindingEditorDialog：parent/slot 选择 + accepts/cardinality/occupancy 展示
- [x] InstanceActions：父 Binding 与子 Binding 列表、Move、Detach、Slot 子节点
- [x] App：binding draft 挂载、Review 后选择保持
- [x] Fixtures/mock：Slot 与 binding 编辑命令；新增 Binding/子树 UI 链路
- [x] Tauri host：Binding 移动写回、子树创建写回、Detach 拒绝不写文件
- [x] 更新 plan / README / architecture-entry，回写本卡

## 验收

本地已执行（2026-11-27）：

- `cd apps/designer && npm run build`：通过（tsc --noEmit + Vite，85 modules）；
- `cd apps/designer && npm run test:ui`：**18 passed**（原 15 条 + 新增 3 条 Binding/子树链路）；
- `cargo test --manifest-path apps/designer/src-tauri/Cargo.toml`：**12 passed**
  （原 9 条 + Binding 移动、Binding 子树创建、Detach 拒绝 3 条真实 `ProjectSession` 测试）；
- `cargo test --workspace`：**146 passed**；
- `cargo clippy --workspace --all-targets -- -D warnings`：通过；
- `cargo fmt --all --check`：通过。

逐项核对：

- Binding Move 经 `prepare_transaction` → Review → `apply_transaction`；成功后模型与 Source
  刷新，选择保持在 child；取消不写入；
- Move 对话框展示 Slot 的 accepts、cardinality、required、当前占用与子节点；非法移动在
  Review 中显示 Core 诊断且 Apply 禁用；
- Detach 在 Review 中显示 `structure-unreachable-instance`，文件不被修改；
- 必填 Slot 的复合模块可递归创建整棵必需子树；缺少模块/配置/ID 时本地阻止提交；
- Root 不作为 child 候选；Cycle 由 parent 候选排除后代 + Core 校验双重保护；
- 未修改 `docs/spec/**`、`docs/design/**`、`composition/` 语义文件或模块定义；
- 变更范围与 `changes/023-binding-editing.yaml` 一致。

## 结果 / 遗留问题

- 结果：
  - `src/inspector/BindingEditorDialog.tsx`：parent/slot 选择、Slot 元数据、Move；
  - `src/inspector/CreateInstanceDialog.tsx`：required Slot 递归草稿树，多实例多 Binding
    同一事务提交；
  - `src/inspector/instanceEditing.ts`：requiredSlots、acceptedModules、binding 查询、
    parent 候选（排除后代）、move/detach 命令；
  - `src/inspector/InstanceActions.tsx`：Binding 列表、Move/Detach、Slot 子节点；
  - `src/App.tsx`：binding draft 挂载与清理、Review 后选择跟随；
  - `tests/ui/binding.spec.ts`：3 条 UI 链路（Move、Detach 诊断、子树创建）；
  - `src-tauri/src/lib.rs`：3 条真实 `ProjectSession` 写回/拒绝测试。
- 关键决策：
  - Move 只改 parent + slot，child 固定，避免误把另一棵子树换进来；
  - Detach 不静默处理；单根模型下 Core 拒绝，UI 展示真实诊断，删除子结构走 Delete；
  - required Slot 递归展开，每个 required Slot 先创建一个子实例（cardinality many 可后续
    继续 Add child）；
  - 父级候选排除 child 后代，但 Slot 接受范围与基数仍以 Core 为准。
- 遗留：
  - Move 不支持替换 child；创建时选择 child；
  - Detach 单独提交在 v0.1 单根模型下必然被拒绝（模型语义，不是 UI 缺陷）；
  - cardinality `many` 的 required Slot 在向导中固定创建一个子实例；
  - 递归模块契约受 MAX_DEPTH=6 保护，超深结构需要用户选择更浅的模块；
  - Connection / Dependency 编辑与完整 Before/After Review 属于 024 / 025。
- 下一步：024 — Connection / Dependency 视图。
