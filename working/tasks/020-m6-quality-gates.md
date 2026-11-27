# 020 — M6-001 自动化质量门禁与 UI 测试基础

- 状态：done（CI 远端执行待明确推送后验证）
- 目标：建立 Windows-first 的 CI 与前端 UI 自动化基础，让关键编辑链路在本地和 PR 中可重复验证。
- 授权：Extend（CI workflow、Playwright 测试工具、测试 fixture）
- 变更声明：`changes/020-quality-gates.yaml`

## 交付

1. `.github/workflows/ci.yml`：
   - windows-latest；
   - Rust fmt / clippy / workspace tests；
   - Tauri host tests；
   - MCP e2e；
   - npm ci、TypeScript/Vite build；
   - Playwright UI tests；
   - 失败时上传 Playwright report/trace。
2. Playwright 1.65 + 系统 Edge（`channel: "msedge"`），不下载额外浏览器；
3. 测试专用 Tauri IPC fixture：不进入生产代码，不写真实文件；
4. 8 条 UI 链路；
5. `npm run test:ui`（build + Playwright）本地通过。

## 清单

- [x] CI workflow（Windows、最小权限、Cargo/npm 缓存）
- [x] Rust / Tauri / MCP / TypeScript / Vite / Playwright 接入
- [x] Playwright 配置、fixture、IPC mock
- [x] 8 条 UI 用例
- [x] `npm run test:ui` 本地通过
- [x] Playwright 失败截图/trace/report 配置
- [x] `.gitignore` 排除 test-results / playwright-report
- [x] README / plan / AGENTS 命令更新

## 验收

本地已执行：

- `cargo test --workspace`：**146 passed**；
- Tauri host tests：**5 passed**；
- `cargo clippy --workspace --all-targets -- -D warnings`：通过；
- `cargo fmt --all --check`：通过；
- `npm run build`：80 modules，通过；
- `npm run test:ui`：**8 passed**；
  - valid project / invalid project；
  - select instance；
  - edit + review + apply；
  - cancel；
  - rejected transaction；
  - external change + reload；
  - layout save without digest change。

远端状态：

- workflow 文件已建立，但按当前规则**未推送**，因此 GitHub Actions 尚未执行；
- 远端 CI 的首次真实执行只能在用户明确要求阶段性推送后验证；
- 本卡不声称 CI 已通过。

## 结果 / 遗留问题

- 结果：
  - 8 条 UI 链路已本地自动化；
  - CI 文件覆盖 146 Rust + 5 host + 8 UI + MCP e2e；
  - Playwright 直接使用系统 Edge，避免浏览器下载。
- 关键决策：
  - UI 测试走 Vite preview + 受控 IPC fixture，覆盖 React 行为；
  - 真实 Tauri/WebView2 仍由 5 个 host tests 覆盖；
  - 首批不引入 tauri-driver，避免 CI 不稳定；后续可补 smoke。
- 遗留：
  - CI 未远端执行；
  - UI fixture 不是真实 Tauri 集成；
  - 无真实 GUI WebView2 自动化；
  - 020 的 8 条覆盖目前主要针对既有查看/配置/外部同步，Instance/Binding 编辑链路将在 022–025 扩展。
- 下一步：021 真实项目试点；需要用户提供代表性子系统。
