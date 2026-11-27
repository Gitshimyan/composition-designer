# 028 — M7 Source 设计决议

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：决定 Source 编辑的事务形状、冲突/Review/CLI/MCP 边界，产出 ADR 与 029 实施前提；不直接写代码。
- authorization：
  - required：modify
  - approval：
    - modify：default（设计研究/文档）
    - extend：pending（029 新命令）
    - rearchitect：pending（029 共享事务协议）
- 来源：M7；v0.1 设计 §7.4–§7.5；用户 2026-11-27 确认 ADR-0004 后继续
- 变更声明：`changes/028-source-design.yaml`

## 研究范围

1. 复核 `ProjectSession`、`Transaction`、`EditorWorkspace`、TOML 写回与 SourceMap；
2. 比较整文件替换、独立 Source 事务、patch 命令、直接写文件；
3. 决定 Source 编辑是否复用 `Transaction`、如何返回文本/对象 Diff、如何处理草稿/外部冲突；
4. 明确 CLI/MCP/Tauri/UI 的同一事务入口；
5. 不修改 Schema、命令实现或产品代码。

## 非目标

- 不实现 `replace_source`；
- 不修改命令协议/事务；
- 不开放模块文件编辑；
- 不实现三向合并/自动修复。

## 验收

- Source 事务形状、输入/结果、写回、冲突、Review、CLI/MCP 有明确方案；
- 明确 029 的 Extend/Rearchitect 范围与测试；
- 明确首版限制；
- 无产品代码/Schema 变更。

## 结果 / 遗留问题

- 结果：
  - 新增 [`ADR-0005`](../../docs/adr/0005-source-editing-transaction.md)（Proposed）；
  - 推荐：事务内独占 `replace_source`（完整 Composition 文本），复用 `base_revision`/
    `request_id`/Revision/Review；
  - prepare 解析+验证候选文本，返回文本 Diff + `ModelDiff` + 完整诊断；
  - apply 原样写入、备份/替换/重验，失败恢复；外部冲突阻止，Reload 丢弃草稿；
  - 首版只编辑 Composition 入口，模块仍只读；不做三向合并/自动格式化。
- 授权阻塞：
  - 029 实施需要 **Extend + Rearchitect** 明确批准；
  - 028 只完成设计，ADR-0005 仍为 Proposed。
- 验证：
  - ADR/文档链接检查、`git diff --check`；无代码测试。
- 下一步：用户批准 ADR-0005 后创建 029；否则回到 S1 或调整顺序。