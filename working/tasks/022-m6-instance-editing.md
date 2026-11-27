# 022 — M6-003 Instance 编辑

- 状态：done
- 目标：让 Designer 通过统一事务完成 Instance 的创建、删除、重命名、换模块/版本与配置重置，并在 Change Review 中显示级联影响。
- 授权：Modify（Designer UI、Tauri host 测试、设计与协作文档）
- 变更声明：`changes/022-instance-editing.yaml`
- 依赖：021 结论（现有 Domain/Schema/Commands 足够，不扩展 v0.1 模型）

- coverage_status：complete_with_deferred_items
- open_items：UW-005、UW-012、UW-022
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

Core/Editor 已冻结 12 条编辑命令（`create_instance`、`delete_instance`、`rename_instance`、
`update_instance_module`、`set_config`、`unset_config` 等），Tauri host 也已把
`Transaction` 原样转给 `ProjectSession`。缺口在 Designer：Inspector 只暴露
`set_config` / `unset_config`，其余 Instance 命令没有入口，也没有删除/换模块的影响预览。

本卡只做 Instance 生命周期；Binding 的重新选择/改 Slot/删除 Binding 属于 023，
Connection/Dependency 视图属于 024，完整 Before/After Review 属于 025。

## 范围与关键决策

1. **一个意图，一个事务**：用户每次操作都先 `prepare_transaction`，在 Review 中查看
   Diff/诊断，再 `apply_transaction`；GUI 不写文件。
2. **创建 Instance 必然同时建立首个 Binding**：现有模型要求非 Root 实例必须有一个父
   Binding，单独 `create_instance` 会被 L5 拒绝。因此创建入口是“在选中复合实例的
   Slot 下新增子实例”，事务为 `create_instance` + `create_binding`。
3. **创建的模块约束**：新实例必须在创建后立即合法，因此只列出自带必填 Slot 的模块之外的
   候选（模块自身 required Slot 需要先有子实例）。带必填 Slot 的复合模块整棵子树的创建
   属于 023 的 Slot 感知结构编辑。
4. **必填属性在创建时配置**：`create_instance.config` 携带模块 required 属性的值；
   没有默认值的必填属性在对话框内填写。
5. **换模块/版本时同一事务调整配置**：保留名称兼容且类型兼容的已有配置；移除目标契约
   不认识的键；为新 required 属性填值。否则候选模型会因 unknown/missing property 被拒。
6. **删除按子树**：删除一个实例会递归删除其后代实例；`DeleteInstance` 级联移除相关
   Binding/Connection/Dependency Provider。Review 前展示将移除的实例、Binding、
   Connection 与受影响的依赖解析，最终合法性仍由 Core 判定。
7. **不新增命令、不改 Schema、不改设计基线**：所有操作复用 009 冻结命令。

## 清单

- [x] 读 plan / 设计基线 §7、§8 / command-schema / Tauri host 现状
- [x] 任务卡与 Change Manifest
- [x] `types.ts`：补全 12 条 `EditCommand`、Slot/Port/Dependency 契约类型
- [x] 共享 Config 草稿解析与 Instance 影响计算模块
- [x] Inspector：重命名（引用重写影响）
- [x] Inspector：换模块/版本（版本选择 + 配置调整 + 重新校验）
- [x] Inspector：删除（子树/连接/Provider 影响 + 级联事务）
- [x] Inspector：在 Slot 下创建子实例（模块/ID/Binding ID/必填配置）
- [x] App：Review 后选择跟随、新节点布局补齐、状态栏文案
- [x] Playwright fixture/mock 支持 Instance 命令；新增 7 条 UI 链路
- [x] Tauri host：多命令事务 + 保留格式 TOML 写回测试（真实 `ProjectSession`）
- [x] 更新 plan / README / architecture-entry，回写本卡

## 验收

本地已执行（2026-11-27）：

- `cd apps/designer && npm run build`：通过（tsc --noEmit + Vite，84 modules）；
- `cd apps/designer && npm run test:ui`：**15 passed**（原 8 条 + 新增 7 条 Instance 链路）；
- `cargo test --manifest-path apps/designer/src-tauri/Cargo.toml`：**9 passed**
  （原 5 条 + 新增 4 条真实 `ProjectSession` 事务/写回测试）；
- `cargo test --workspace`：**146 passed**；
- `cargo clippy --workspace --all-targets -- -D warnings`：通过；
- `cargo fmt --all --check`：通过。

逐项核对：

- 所有 Instance 操作都经过 `prepare_transaction` + `apply_transaction`；
  取消/关闭 Review 不写入，拒绝时显示 Core 诊断且 Apply 禁用；
- 创建、重命名、换模块/版本、删除后使用 apply 返回的快照与 Source，Revision/Digest 刷新；
  重命名/删除后选择跟随父实例或新 ID，新节点补齐布局；
- 删除 Root 被禁用；删除带后代的实例时 Review 列出整棵子树及其 Binding/Connection/
  Dependency 影响；
- 未修改 `docs/spec/**`、`docs/design/**`、`composition/` 语义文件或模块定义；
- 变更范围与 `changes/022-instance-editing.yaml` 一致。

## 结果 / 遗留问题

- 结果：
  - `src/inspector/InstanceActions.tsx`：重命名、换模块/版本、删除影响与 Review；
  - `src/inspector/CreateInstanceDialog.tsx`：Slot 下创建子实例（模块/ID/Binding ID/必填配置）；
  - `src/inspector/instanceEditing.ts`、`configDraft.ts`：影响计算、ID 建议、模块切换配置计划、
    共享 Config 草稿解析；
  - `src/types.ts`：12 条 `EditCommand` 与 Slot/Port/Dependency 契约类型；
  - `App.tsx`：Review options（选择/别名/聚焦）、布局补齐、Create 对话框挂载；
  - `tests/ui/instance.spec.ts` + fixture/mock：7 条 Instance 链路；
  - `src-tauri/src/lib.rs`：创建+Binding、删除级联、拒绝不写文件、重命名 root/引用 4 条
    真实 `ProjectSession` 测试。
- 关键决策：
  - 创建 Instance 与首个 Binding 同一事务，避免 L5 瞬时非法；
  - 创建候选排除带必填 Slot 的模块（整棵子树创建留给 023）；
  - 换模块时在同一事务 Unset 不兼容/未知配置并补齐新必填属性；
  - 删除按子树先子后父，复用 `DeleteInstance` 的级联语义。
- 遗留：
  - 带必填 Slot 的复合模块无法通过本卡 UI 创建（023 的递归结构创建已补齐）；
  - 换模块只能选择已加载的模块/版本；尚无模块库/版本下载；
  - Dependency Provider 只展示受影响关系，重绑定属于 024；
  - Instance 命令的 UI 测试使用 IPC fixture，真实 WebView2 交互仍由 host tests 覆盖；
  - 既有 `ConfigValue` JSON untagged 限制未变：整数值的 float 需要小数形式输入。
- 下一步：023 — Binding 编辑（复用本卡的事务/Review/影响展示）。
