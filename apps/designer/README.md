# apps/designer/

v0.1 可视化设计器：Tauri 2 宿主 + React/TypeScript 界面。**M3 012–015 已建立外壳、
结构 Canvas/Tree、Inspector 配置编辑/Change Review、只读 CodeMirror Source 与外部修改
同步/Reload**。

| 区域 | 目录 | 职责 |
|---|---|---|
| Composition Canvas | `src/canvas/` | 实例与结构 Binding 的 `@xyflow/react` 呈现（013） |
| Project Explorer | `src/explorer/` | 项目目录、结构 Tree、模块来源与搜索 |
| Inspector | `src/inspector/` | 选中实例只读详情；属性编辑在 014 |
| TOML Source | `src/source/` | CodeMirror 6 只读 Source、诊断定位（014） |
| Diagnostics | `src/diagnostics/` | 错误/警告、源码位置、修正提示 |
| Agent Status | `src/agent-status/` | Agent 会话、Revision 与操作记录 |
| Tauri Host | `src-tauri/` | `open_project` / 编辑事务 / layout / source IPC |

## 当前外壳

- 顶栏：打开项目、路径输入、项目标识；
- Explorer：项目、模块、结构 Tree（层级/slot/选择/聚焦）；
- Composition：`@xyflow/react` 实例节点 + Binding 边、Fit/Reset、拖动节点；
- Inspector：选中实例的模块/契约、配置编辑器、Change Review（Set/Unset Config）；
- Diagnostics：L1–L7 代码、消息、文件:行:列；点击可跳到 Source；
- TOML Source：CodeMirror 6 只读展示 composition 入口；
- 外部修改：2s digest 轮询；dirty 时禁编辑并提示 Reload；
- 空状态 / 无效项目 / 有效项目三种界面；
- 布局只写 `.designer/layout.toml`，编辑写 `composition/**` 但只经 Core 事务；
- 所有数据来自 `composition_core` / `composition_editor`，React 不解析 TOML。

## 开发命令

```powershell
npm run setup       # npm install（宿主任务执行）
npm run build       # tsc --noEmit + vite build（TEMP 指向 .tmp）
npm run desktop     # build + cargo build --features custom-protocol（嵌入 dist）
npm start           # 启动已构建 exe，打开 examples/minimal-project
npm run start-invalid  # 打开 connection errors 示例
npm run start-topmost  # 仅验收截图用：置顶 + minimal-project
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
