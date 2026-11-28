# 041 — Profile 入口与 S2 验收

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：把 040 的 Core/Session 选择暴露到 CLI、MCP 与 Designer；补 v3 fixtures 与端到端验收；关闭 UW-004/UW-018，S2 收口。
- authorization：
  - required：modify、extend、rearchitect
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 授权 S2）
    - rearchitect：granted（用户 2026-11-28 授权 S2；更新冻结协议/MCP 文档）
- 来源：`working/plan.md` S2 041；ADR-0007
- 变更声明：`changes/041-profile-entry.yaml`

## 背景

040 已实现 Core/Session 的 profile 选择，但用户还没有入口：CLI/MCP 无法指定 profile，
Designer 没有选择器，也没有 v3 样例。041 完成入口层并验收“所选 profile 贯穿
Revision/Digest/Source/事务”。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | CLI 所有命令 `--profile` | implemented | `composition-cli` + 4 测试 |
| REQ-02 | MCP 启动 `--profile` | implemented | `composition-mcp` + 测试 |
| REQ-03 | Designer profile 选择、Source/Review 跟随 | implemented | Tauri IPC + 选择器 + Playwright |
| REQ-04 | v3 默认/多 profile/错误样例 | implemented | `examples/v3/**` 3 组 |
| REQ-05 | 写回只改所选入口 | implemented | CLI/editor 验收 |
| REQ-06 | v1/v2.0 回归 | implemented | 既有 233 + 新测试 |
| REQ-07 | 关闭 UW-004/UW-018，S2 收口 | implemented | plan/UW/D-031 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-020 | MCP progress/cancel | 协议能力 | 无长任务证据 | Extend | 无进度反馈 | 继续 deferred |
| UW-032 | 接口形状/service 内部组合 | Rearchitect | v2.1 范围外 | Rearchitect | 表达受限 | 继续 deferred |
| 无 | `--all-profiles` 全量校验 | Extend | v2.1 非目标 | Extend | 需逐个选择校验 | 出现需求再评估 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 041 入口与验收 | 保留 `--all-profiles` 不做 | v2.1 非目标 | ADR-0007 |

## 清单

- [x] CLI `--profile`（inspect/resolve/validate/context/impact/batch/query/apply/jsonl）
- [x] MCP `--profile` + context activeProfile
- [x] Tauri `open_project(path, profile)`、`initial_profile`、Designer 选择器
- [x] v3 fixtures（valid/reserved/version mismatch）
- [x] CLI/MCP/UI/Tauri 测试
- [x] 文档、plan、UW、S2 收口

## 验收

- CLI 默认/显式 profile 输出正确；未知/不支持 profile 退出 1；
- MCP 进程绑定 profile；context 返回 activeProfile；
- Designer 切换 profile 重新加载，Source/Review/写入只针对所选入口；
- v3 fixtures 的 expected 与 Core 诊断一致；v1/v2.0 不变；
- 全量测试、Tauri、UI、clippy、fmt 通过；gate 0/0。

## 结果 / 验证

- 实际执行：
  - CLI：`--profile` 全命令；`ProtocolSession::new_with_profile`；context 增加
    `activeProfile`/`profiles`；
  - MCP：`--profile` 启动参数与 `McpServer::new_with_profile`；
  - Tauri：`open_project(path, profile)`、`initial_profile`、reload/external check 绑定 profile；
  - UI：顶栏 Profile 选择器、localStorage 记忆、v3 mock fixture 与 Playwright 用例；
  - fixtures：`examples/v3/valid-profiles`、`invalid-reserved-profile`、
    `invalid-composition-version`；总数 40 个 expected.json。
- 验证：
  - `cargo test --workspace`：**238 passed**（CLI +4、MCP +1）；
  - `tauri-test`：**17 passed**；
  - `npm run test:ui`：**35 passed**；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - configure.mjs 带 `--profile` 冒烟：args 含 `--profile desktop`；
  - change-gate block（工作树临时提交）：50 个文件、48 个 Manifest，**0 error / 0 warning**。
- 未执行：Linux/macOS 实机；真实桌面窗口人工切换（Playwright mock + Tauri host 测试覆盖 IPC/session）。
- 结果：CLI/MCP/Designer 都能选择 v3 profile；Source/Review/写回限定所选入口；S2 完成。
- 遗留 / 后续：H1 可靠性、M9 生态、UW-020/031/032 仍 deferred。
