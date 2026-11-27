# 020 — M6-001 自动化质量门禁与 UI 测试基础

- 状态：planned
- 目标：建立 Windows-first 的 CI 与前端 UI 自动化基础，让关键编辑链路在本地和 PR 中可重复验证。
- 授权：Modify（workflow、测试工具、前端测试支持；不改领域模型/Schema）
- 前置：020 开工时创建 `changes/020-quality-gates.yaml`

## 范围

1. `.github/workflows/ci.yml`（windows-latest）：
   - `cargo test --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo fmt --all --check`
   - Tauri host tests
   - `npm ci` / TypeScript check / Vite production build
   - `cargo test -p composition-mcp --test e2e`
   - UI 自动化测试
2. 前端 UI 测试基础：
   - Playwright + 受控 IPC fixture（推荐首批）；
   - 不把 mock 注入生产路径；
   - 本地 `npm run test:ui` 可运行；
   - 若真实 WebView2 自动化成本可控，再补 tauri-driver smoke，不阻塞首批。
3. 首批 UI 用例（按评审）：
   - 打开有效项目；
   - 打开无效项目并显示诊断；
   - 选择 Instance 查看 Inspector；
   - 修改 Property 并显示 Change Review；
   - 取消与确认事务；
   - 事务失败显示诊断；
   - 外部修改阻止写入并 Reload；
   - Canvas 布局保存不改变 Source Digest。

## 清单

- [ ] CI workflow（Windows、最小权限、缓存）
- [ ] Rust / Tauri / TypeScript / Vite / MCP e2e 检查接入
- [ ] Playwright 测试框架与 IPC fixture
- [ ] 首批 8 条 UI 链路
- [ ] `npm run test:ui` 本地通过
- [ ] 失败日志/截图/trace 可追溯
- [ ] 文档更新（README、plan、AGENTS 命令）
- [ ] 本地验证；推送仅在用户明确要求时

## 验收

- 本地：
  - `cargo test --workspace`、clippy、fmt；
  - Tauri host tests；
  - `npm run build`；
  - `npm run test:ui`；
  - MCP e2e。
- CI（推送后）：
  - workflow 在 PR 上执行；
  - 任一检查失败阻止合并；
  - 测试结果、截图/trace 可下载。
- UI 测试不依赖真实外部网络、不写真实项目文件；fixture 只存在于测试环境。

## 风险 / 待定

- Playwright 与 Tauri IPC 的 fixture 边界需要先确认；不得为了让测试通过而改生产语义。
- WebView2/tauri-driver 在 Windows CI 的稳定性未知，首批不以真实 GUI 自动化为阻塞项。
- 引入 Playwright 会增加 node_modules/CI 时间；需缓存浏览器与 npm。
- 由于当前规则是“仅明确要求时推送”，workflow 的远端执行要等阶段评审推送后验证；
  本地必须能独立跑通。

## 结果

未开工。
