# ADR-0009 — 046 WebView2 smoke：CDP + 真实 Tauri 窗口

- 状态：**Accepted（2026-11-28，046）**
- 授权：用户 2026-11-28 明确授权 046，并要求研究后自行决策
- 关联：[`unresolved-work.md`](../../working/decisions/unresolved-work.md) UW-012 ·
  [`working/plan.md`](../../working/plan.md) H1 046
- 实现：`apps/designer/tests/webview2/smoke.mjs`、`npm run smoke:webview2`

## 背景

现有 Playwright 测试注入 `window.__TAURI_INTERNALS__`，验证 React UI 契约，但没有启动真实
Tauri/WebView2 宿主，无法发现窗口启动、真实 IPC、WebView2 版本或前端构建产物问题。
046 需要在 Windows 上做真实宿主 smoke。

## 备选方案

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| `tauri-driver` + WebDriver | 官方 Tauri 自动化路径 | 需要额外安装 `tauri-driver`/`msedgedriver`，版本与 WebView2 强耦合；CI 配置重 | 未采用 |
| WebDriverIO + 第三方服务 | 生态成熟 | 额外依赖、配置复杂 | 未采用 |
| 原生 Win32 窗口探针 | 无浏览器依赖 | 只能证明窗口存在，不能验证 UI/IPC | 不足 |
| **CDP + Playwright** | 复用已有 Playwright；真实窗口；可截图/DOM/点击 | 仅 Windows/WebView2；需调试端口环境变量 | **采用** |

## 决策

1. 通过 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<free>` 启动真实
   `composition-designer.exe`；
2. Playwright `chromium.connectOverCDP` 连接 WebView2，等待真实项目 UI；
3. 执行“打开项目 → 编辑 title → Review → Apply → 断言 Inspector”最小闭环；
4. 截图与 JSON 报告写入 `apps/designer/.tmp/`（忽略）；
5. 由 `npm run smoke:webview2` 先 `npm run desktop` 构建真实二进制，再运行脚本；
6. 仅作为 Windows 本机 smoke；常规 CI 仍以 Playwright mock 测试为主，不替代。

## 为什么不是 `tauri-driver`

- 需要额外二进制与 WebView2 驱动版本匹配，环境准备成本高于本 smoke 的目标；
- CDP 已能访问真实 WebView2 DOM 与 IPC，足以覆盖启动、加载、编辑、事务；
- 不新增 npm 依赖（Playwright 已存在）。

## 后果

- 正面：能发现真实窗口/构建/IPC 问题；测试脚本可本地重复；截图可人工复核。
- 成本：Windows/WebView2 限定；调试端口仅测试用；构建桌面二进制增加约 20s。
- 风险：CDP 是调试接口，未来 WebView2 版本可能调整；脚本失败会明确报告端口/项目名。
- 复审条件：需要跨平台真实窗口自动化、CI 集成或完整 E2E 时，再评估 `tauri-driver`。

## 边界

- 不替代 Playwright 的 38 条 UI 契约测试；
- 不测试外部修改、多 profile、Source 等全部路径；
- 不引入 WebView2 专用生产代码或配置。
