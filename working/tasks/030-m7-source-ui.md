# 030 — Source UI（CodeMirror 编辑/草稿/Review）

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：把 029 的 `replace_source` 接到 Designer：CodeMirror 可编辑、草稿、Review 文本+对象 Diff、外部冲突只读。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：M7；029；ADR-0005 Accepted
- 变更声明：`changes/030-source-ui.yaml`

## 范围

1. `SourceView` 从只读升级为可编辑：dirty 状态、Review、Discard、外部冲突只读；
2. 通过现有 `reviewCommands` 提交 `replace_source`，不新增 Tauri/命令；
3. Change Review 增加 Source 文本 Before/After；
4. 外部 digest 冲突时阻止编辑/Review；
5. Playwright 覆盖有效 Source 编辑与非法 Source 拒绝。

## 非目标

- 不做文本级三向合并；
- 不编辑模块文件；
- 不新增命令/协议。

## 验收

- 可编辑 Source，Review 显示文本 Before/After、对象 Diff、诊断；
- Apply 后 Source/模型刷新；
- 非法文本拒绝、Apply 禁用、文件不写；
- 外部变更时 Source 只读并提示 Reload；
- 既有 UI 链路不回归。

## 结果 / 遗留问题

- 结果：
  - `SourceView` 支持 editable/dirty/Review/Discard/冲突只读；
  - App 通过 `replace_source` 事务提交 Source 编辑；
  - `ChangeReview` 显示 Source Before/After；
  - mock 支持 Source 事务与非法文本诊断；
  - 新增 2 条 Playwright Source 链路。
- 验证：
  - `npm run test:ui`：**30 passed**；
  - `npm run build`：通过。
- 下一步：031 Source 验收。