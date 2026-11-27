# 031 — M7 Source 验收

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：在 Glide 试点副本上通过 CLI/Protocol 验证 Source 编辑、重开与 Digest；汇总 M7 UI/后端证据。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：M7；029/030
- 变更声明：`changes/031-source-closeout.yaml`

## 验收

- Glide 试点副本上的 Source 替换可重复执行；
- 写回后重新 validate 通过，实例/Binding/Connection 数量不变，Digest 刷新；
- 原试点目录不被修改；
- UI 30、Tauri 16、workspace 173 测试通过；
- 外部冲突/非法文本由 029 测试覆盖；
- 无新增命令/协议（031 只验收）。

## 结果 / 遗留问题

- 结果：
  - 新增 `crates/composition-cli/tests/source_closeout.rs`；
  - Glide 试点副本 `title` 通过 CLI `replace_source` 修改；
  - 重开 valid，7 实例/6 Binding/6 Connection，Digest 变化；
  - 验收记录：`docs/reviews/2026-11-27-m7-source-closeout.md`。
- 验证：
  - `cargo test --workspace`：173 passed；
  - Tauri host：16 passed；
  - Playwright：30 passed；
  - `cargo clippy` / `cargo fmt` 通过。
- 下一步：S1 Schema v2.0（032–035）需按 ADR-0004 逐卡授权。