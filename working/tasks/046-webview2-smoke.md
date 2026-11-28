# 046 — 真实 WebView2 smoke

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：在真实 Tauri/WebView2 窗口中验证启动、打开项目、编辑、Review/Apply 与截图；不替代 Playwright UI 测试。
- authorization：
  - required：modify、extend
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 明确授权 046，并要求研究后自行决策）
- 来源：`working/plan.md` H1 046；UW-012
- 变更声明：`changes/046-webview2-smoke.yaml`

## 研究结论（ADR-0009）

| 方案 | 结论 |
|---|---|
| `tauri-driver` + WebDriver | 未采用：需额外驱动/版本匹配，环境成本高 |
| WebDriverIO | 未采用：额外依赖/配置 |
| 原生 Win32 探针 | 不足：只能证明窗口存在 |
| **Playwright + WebView2 CDP** | **采用**：复用 Playwright、真实窗口、可 DOM/点击/截图 |

实现：`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<free>` 启动
`composition-designer.exe`，Playwright `connectOverCDP` 执行最小闭环。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | 真实桌面二进制启动 | implemented | smoke 启动成功 |
| REQ-02 | 真实 IPC 打开 `--project` | implemented | 项目名 `Minimal Project` |
| REQ-03 | 真实编辑 + Review + Apply | implemented | title 改为 `WebView2 Smoke` |
| REQ-04 | 截图与 JSON 报告 | implemented | `.tmp/webview2-smoke.png/.json` |
| REQ-05 | 可重复脚本 | implemented | `npm run smoke:webview2` |
| REQ-06 | 不影响常规 UI 测试 | implemented | 38 UI 仍通过 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-017 | 原生 watcher/三向合并/layout 合并 | 产品能力 | 045 已决策 deferred | Modify / Extend | 见 045 | 按反馈评估 |

非目标：CI 中运行 Windows WebView2 smoke（当前为本地脚本；需要 CI 环境/驱动时再评估）。

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 真实 WebView2 smoke | 本地 Windows smoke，不接 CI | 先建立可重复真实宿主验证；CI 环境另行评估 | ADR-0009 |

## 清单

- [x] CDP 启动/等待/连接
- [x] 打开项目 + 编辑 + Review/Apply 断言
- [x] 截图/报告
- [x] npm 脚本与文档
- [x] 计划/UW/决策回写

## 验收

- `npm run smoke:webview2` 在 Windows 真实 WebView2 中通过；
- 脚本先构建桌面二进制，再启动；测试结束终止进程树；
- 项目副本与报告写入 `.tmp/`，不修改 `examples/`；
- 失败时非零退出并写 JSON 错误；
- 常规 Playwright UI 测试不受影响。

## 结果 / 验证

- 实际执行：
  - `apps/designer/tests/webview2/smoke.mjs`：动态空闲端口、真实 exe、CDP 连接、
    等待 `.project-name`、编辑 title、Review/Apply、截图；
  - `package.json` 新增 `smoke:webview2`（先 `npm run desktop`）；
  - ADR-0009 记录方案与边界。
- 验证：
  - `npm run smoke:webview2`：**通过**（`ok:true`, projectName `Minimal Project`,
    title `WebView2 Smoke`, cdpPort 55521）；
  - `npm run test:ui`：**38 passed**（046 未改 UI 行为）；
  - `cargo test --workspace`：**251 passed**（沿用 044 基线，046 无 Rust 变更）；
  - `tauri-test`：**17 passed**（046 无 Tauri 变更）；
  - change-gate block（工作树临时提交）：11 个文件、53 个 Manifest，**0 error / 0 warning**。
- 未执行：CI 集成、Linux/macOS 真实 WebView2（平台限定）。
- 结果：真实宿主 smoke 可重复运行，覆盖启动/IPC/编辑/事务/截图；H1 收口。
- 遗留 / 后续：M9 生态/维护；UW-017 剩余项；CI 集成按需。
