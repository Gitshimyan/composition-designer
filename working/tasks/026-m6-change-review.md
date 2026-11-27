# 026 — M6-006 完整 Change Review

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：把 Change Review 从“对象 ID + 摘要”升级为可完整理解一次多命令事务的审阅界面：按对象类别分组、Before/After、命令列表、Dependency 影响与完整诊断。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：M6-006；`docs/design/v0.1-design.md` §7.4、§8
- 变更声明：`changes/026-change-review.yaml`

## 背景

022–025 已让 Instance/Binding/Connection/Dependency 编辑全部经事务与 Review，但当前
Change Review 只显示命令数、摘要和前 8 个对象 ID；它无法满足 M6-006：

- 不展示 Before/After；
- 不按对象类别分组；
- 不展示 Dependency Provider 映射影响；
- 只显示前 6 条诊断，且没有明确的验证通过/拒绝分区；
- 多命令事务（例如创建子树）的理解成本高。

本卡只使用现有 `EditResponse.diff` / `diagnostics` / `snapshot`，不新增 Core 命令、
不修改事务协议、不增加 Tauri IPC 字段。

## 范围与关键决策

1. **单一数据源**：Review 从 `ModelDiff`、候选诊断与当前 `ProjectSnapshot` 派生；
   不建第二套模型。
2. **分类展示**：Instance / Binding / Connection 分组；每类显示对象 ID、kind 与
   Before/After 字段。
3. **命令列表**：逐条展示事务命令的语义摘要，确保多命令事务可理解。
4. **Dependency 影响**：从 instance 变更的 before/after `dependency_providers` 推导
   受影响的 requirement 映射；当前解析状态来自 `snapshot.model.dependencies`。
   不在 UI 重新计算依赖解析规则。
5. **完整验证**：显示 accepted/rejected、全部诊断（不截断）、错误/警告数量；
   拒绝时 Apply 禁用并保留诊断。
6. **失败保持**：prepare/apply 拒绝时继续显示同一 Review 与当前快照；不写入；
   取消关闭 Review，不提交。
7. **不新增 IPC/Core 字段**：如果后续需要候选模型或依赖 after 状态，另开 Extend。

## 清单

- [x] 读 M6-006 / 设计基线 §7.4 / 现有 Diff、诊断与 Review 代码
- [x] 任务卡与 Change Manifest
- [x] `review` 组件：分类、Before/After、命令、Dependency 影响、完整诊断
- [x] App 接线与样式（宽卡片、滚动、响应式）
- [x] mock diff 补 before/after，覆盖多命令与依赖映射
- [x] UI 用例：set_config Before/After、多命令分类、Connection 更新、Provider 影响、拒绝诊断
- [x] 更新 README / architecture / plan / 未决登记
- [x] 运行 build / test:ui，确认既有 24 条链路不回归

## 验收

- 一次多命令事务的 Review 同时显示：命令列表、按类别分组的对象变更、
  Before/After、受影响 Dependency 映射、完整诊断；
- Before/After 对 Instance（模块/版本、config、providers）、Binding、Connection 可读；
- 拒绝事务显示全部诊断，Apply 禁用；取消不写入；
- 应用失败时保留当前项目状态与 Review 诊断；
- 既有 24 条 UI 链路通过，新增链路覆盖 026 的核心审阅能力；
- 不新增 Core 命令、TOML 字段、事务协议或 Tauri IPC 字段；
- 变更范围与 Manifest 一致。

## 结果 / 遗留问题

- 结果：
  - 新增 [`src/review/ChangeReview.tsx`](../../apps/designer/src/review/ChangeReview.tsx)：
    命令列表、Instance/Binding/Connection 分类、Before/After 字段、Dependency 影响、
    完整诊断与验证分区；
  - App 改为使用新组件，Review 卡片加宽、可滚动；取消/确认/失败行为保持；
  - mock diff 为 instance/binding/connection 命令补齐 before/after；
  - `tests/ui/review.spec.ts` 新增 4 条链路：config Before/After、多命令分类、
    Connection 更新、Dependency Provider 影响；
  - 既有 24 条 UI 链路全部保持通过（共 28）。
- 验证：
  - `npm run build`：通过（91 modules）；
  - `npm run test:ui`：**28 passed**；
  - `cargo test --workspace`：**154 passed**；
  - Tauri host tests：**15 passed**；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - 未执行机器 YAML 解析（环境无解析器；026 Manifest 人工核对）。
- 关键决策：
  - 复用 `ModelDiff`/诊断/当前快照，不新增 Core 命令、事务协议或 Tauri IPC 字段；
  - Dependency 影响只展示 provider 映射变化与当前解析状态，不在 UI 重算解析规则；
  - 拒绝/应用失败继续保留 Review 与诊断，不写入。
- 遗留：
  - UW-025 已由本卡关闭；UW-002–005、UW-010、UW-016、UW-023、UW-026、UW-027 仍待决策/排期。
- 下一步：M6 功能卡完成；**027 — M6 收口验收**；G04 机器门禁需另行授权 Extend + Rearchitect。
