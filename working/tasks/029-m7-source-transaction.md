# 029 — Source 编辑事务与写回

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：实现 ADR-0005 A 方案：事务内独占 `replace_source`，候选文本验证、文本+对象 Diff、原子写回、外部冲突守卫；不含 UI。
- authorization：
  - required：modify, extend, rearchitect
  - approval：
    - modify：default
    - extend：granted（新 `replace_source` 命令；用户 2026-11-27 选择 A）
    - rearchitect：granted（共享事务/命令协议增加文本命令；用户 2026-11-27 选择 A）
- 来源：ADR-0005 Accepted；028
- 变更声明：`changes/029-source-transaction.yaml`

## 范围

1. `Command::ReplaceSource`：完整 Composition 文本，必须独占事务；
2. Loader 支持“用候选文本替代磁盘 Composition”的只读加载；
3. `ProjectSession` source-aware prepare/apply：
   - prepare 验证候选并返回 `source_diff` + `ModelDiff` + 诊断；
   - apply 校验 Revision/磁盘 digest、原样写入、备份/替换/重验、失败回滚；
4. Revision/历史/幂等与对象事务一致；source request replay 不重复写；
5. Protocol/CLI apply 输出 `source_diff`；Tauri `EditResponse` 暴露 `sourceDiff`；
6. 命令 Schema 与 ADR 文档同步。

## 非目标

- 不做 CodeMirror 编辑 UI（030）；
- 不做模块文件编辑；
- 不做三向合并/自动格式化；
- 不改 Schema/领域对象。

## 验收

- `replace_source` 独占；混合对象命令拒绝；
- prepare 不写盘；非法文本拒绝且文件不变；
- apply 原样写入、保留注释/格式、Revision/Digest/历史一致；
- 外部磁盘变化会拒绝 apply；
- 相同 request 重放不二次写；
- 对象事务回归通过；
- CLI/Protocol 可执行同一命令。

## 结果 / 遗留问题

- 结果：
  - `Command::ReplaceSource`、`COMMAND_KINDS` 15；
  - `load_project_with_composition_text`；
  - `ProjectSession` source prepare/apply、source request 幂等、外部 digest 守卫；
  - `SourceDiff` 进入 TransactionOutcome/Protocol/Tauri；
  - 15 个 session tests、1 个 protocol source test、1 个 command test、1 个 CLI source test。
- 验证：
  - `cargo test --workspace`：**172 passed**；
  - Tauri host：16 passed；
  - `cargo clippy` / `cargo fmt` 通过；
  - UI build + 28 条 Playwright 回归通过。
- 下一步：030 Source UI（CodeMirror 可编辑 + 草稿/冲突/Review）。