# M7 Source Authoring 验收（2026-11-27）

- 任务：029 + 030 + 031
- 状态：`done`
- coverage_status：`complete`
- open_items：无
- 关联：[`working/plan.md`](../../working/plan.md)、[ADR-0005](../adr/0005-source-editing-transaction.md)

## 1. 交付

- 029：`replace_source` 独占事务；候选文本加载；source-aware prepare/apply；
  文本 `source_diff` + 对象 `ModelDiff`；Revision/幂等/外部冲突；Protocol/CLI/Tauri。
- 030：CodeMirror 可编辑 Source；draft/Review/Discard；外部冲突只读；
  Change Review 文本 Before/After；2 条 Playwright 链路。
- 031：Glide 试点副本 CLI Source 编辑验收。

## 2. 验收结果

- Glide 试点副本：`title = "Glide"` → `"Glide Source"`，通过 CLI `replace_source`；
- 重开 valid，7 实例 / 6 Binding / 6 Connection，Digest 刷新；
- 原试点目录未修改；
- 非法文本拒绝且不写；外部冲突拒绝；request 重放不二次写；
- UI 30、Tauri 16、workspace 173 测试通过。

## 3. 边界

- 仅 Composition 入口；模块文件仍只读（UW-031）；
- 无三向合并/自动格式化；Reload 丢弃草稿（UW-017）；
- Source 文本写入不做 fsync 级保证（UW-019）；
- 未开放模块定义 Source 编辑。

## 4. 结论

M7 Source Authoring 完成；下一步按 ADR-0004 启动 S1 Schema v2.0（需逐卡 Rearchitect 授权）。
