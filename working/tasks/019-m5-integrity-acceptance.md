# 019 — M5 完整性与 v0.1 验收

- 状态：done
- 目标：补齐中断写回恢复、性能/多文件/版本边界验证，汇总 A01–A18 验收报告并收口 v0.1。
- 授权：Modify（验收、恢复、测试；不改设计基线）
- 变更声明：`changes/019-integrity-acceptance.yaml`

- coverage_status：complete_with_deferred_items
- open_items：UW-011、UW-015、UW-018、UW-019、UW-020
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

M4 已有 Agent 闭环。019 做最后一轮完整性与验收：

- 恢复中断 TOML 写回留下的 `.bak-*`；
- 合成 80 实例项目验证解析/校验性能；
- 明确单入口边界与 `schema_version=2` 拒绝迁移；
- 生成 A01–A18 验收报告。

## 清单

- [x] `ProjectSession::open` 恢复缺失 composition 的最新 `.bak-*`
- [x] 恢复集成测试
- [x] 80 实例 + 80 Binding 性能测试
- [x] `composition/parts/**` 不合并测试
- [x] unsupported schema_version 测试
- [x] `docs/acceptance-v0.1.md` A01–A18
- [x] 更新 README、plan、docs/ai、crates/README、AGENTS
- [x] `cargo test` / clippy / fmt、Tauri 测试

## 验收

- `cargo test --workspace`：**146 passed**；
- Tauri `cargo test`：**5 passed**；
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check`；
- 备份恢复测试通过；
- 性能测试通过（80 实例 <5s）；
- 单入口/版本迁移边界测试通过；
- 验收报告 A01–A18 全部有证据。

## 结果 / 遗留问题

- 结果：
  - `ProjectSession` 启动时恢复缺失 composition 的最新 `.bak-*`；
  - 性能/多文件/版本测试；
  - `docs/acceptance-v0.1.md`；
  - v0.1 M0–M5 完成。
- 已知限制：
  - 无 fsync 级断电保证；
  - Source 只读；
  - Revision 历史/request 幂等仍进程内；
  - 单 composition 入口；
  - MCP 无订阅/进度/取消；
  - 无前端自动化 UI 测试。
- 结论：A01–A18 在实现范围内全部通过；未支持项与设计基线显式边界一致。
