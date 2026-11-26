# apps/designer/

v0.1 可视化设计器：Tauri 2 宿主 + React/TypeScript 界面。**M3 012 已建立只读外壳**；
Canvas、Tree、Inspector 编辑与 TOML Source 在 013/014。

| 区域 | 目录 | 职责 |
|---|---|---|
| Composition Canvas | `src/canvas/` | 实例与结构 Binding 的呈现与编辑（013） |
| Project Explorer | `src/explorer/` | 项目目录、composition 文件、模块来源与搜索 |
| Inspector | `src/inspector/` | 选中实例、属性、Slot、Port、依赖（014） |
| TOML Source | `src/source/` | 权威 TOML 的查看与直接编辑（014） |
| Diagnostics | `src/diagnostics/` | 错误/警告、源码位置、修正提示 |
| Agent Status | `src/agent-status/` | Agent 会话、Revision 与操作记录 |
| Tauri Host | `src-tauri/` | `open_project` / `initial_project`、窗口与 IPC |

## 当前外壳

- 顶栏：打开项目、路径输入、项目标识；
- Explorer：项目、模块、实例、Binding/Connection 计数；
- Composition：只读实例卡片与结构摘要；
- Inspector：项目、Revision、digest；
- Diagnostics：L1–L7 代码、消息、文件:行:列；
- 空状态 / 无效项目 / 有效项目三种界面；
- 所有数据来自 `composition_core::validate_project`，React 不解析 TOML。

## 开发命令

```powershell
npm run setup       # npm install（宿主任务执行）
npm run build       # tsc --noEmit + vite build
npm run desktop     # cargo build --features custom-protocol（嵌入 dist）
npm start           # 启动已构建 exe，打开 examples/minimal-project
npm run start-invalid  # 打开 connection errors 示例
npm run dev         # 仅 Vite；需要在另一个终端启动 Tauri dev
npm run tauri dev   # Tauri 开发模式
```

- `src-tauri` 被根 Cargo workspace `exclude`，使用独立 `Cargo.lock`；
- `desktop` 构建的 exe 位于 `src-tauri/target/debug/composition-designer.exe`；
- debug 直接 `cargo build` 会使用 `devUrl`（需要 Vite）；验证嵌入资源时必须带
  `--features custom-protocol` 或使用 `npm run desktop`；
- Tauri bundle/installer 尚未启用。

## 边界

- Canvas / Tree / Inspector 不持有权威 Composition 数据，节点坐标只写 `.designer/layout.toml`；
- 所有编辑转换为 Core 命令，经事务验证后提交；
- CodeMirror 只负责编辑体验与基础语法反馈，不实现项目语义校验；
- React 不重复实现 Core 的契约规则；
- Tauri Host 不直接修改 TOML 以绕开 Core。
