# apps/designer/

v0.1 可视化设计器：Tauri 2 宿主 + React/TypeScript 界面。**M3 012–015 已建立外壳、
结构 Canvas/Tree、Inspector 配置编辑/Change Review、只读 CodeMirror Source 与外部修改
同步/Reload；M6 022 增加 Instance 创建/删除/重命名/换模块与版本。**

| 区域 | 目录 | 职责 |
|---|---|---|
| Composition Canvas | `src/canvas/` | 实例与结构 Binding 的 `@xyflow/react` 呈现（013） |
| Project Explorer | `src/explorer/` | 项目目录、结构 Tree、模块来源与搜索 |
| Inspector | `src/inspector/` | 实例详情、配置编辑、Instance 操作与 Change Review |
| TOML Source | `src/source/` | CodeMirror 6 只读 Source、诊断定位（014） |
| Diagnostics | `src/diagnostics/` | 错误/警告、源码位置、修正提示 |
| Agent Status | `src/agent-status/` | Agent 会话、Revision 与操作记录 |
| Tauri Host | `src-tauri/` | `open_project` / 编辑事务 / layout / source IPC |

## 当前外壳

- 顶栏：打开项目、路径输入、项目标识；
- Explorer：项目、模块、结构 Tree（层级/slot/选择/聚焦）；
- Composition：`@xyflow/react` 实例节点 + Binding 边、Fit/Reset、拖动节点；
- Inspector：
  - 选中实例的模块/契约与配置编辑器（Set/Unset Config）；
  - 重命名 Instance（引用重写由 Core 事务完成）；
  - 换 Module / Version（同一事务内移除不兼容配置、补齐必填属性）；
  - 删除 Instance（Review 前展示子树、Binding、Connection、Dependency 影响）；
  - 在 Slot 下创建子 Instance（`create_instance` + `create_binding` 同一事务，
    必填属性在对话框内填写）；
- Diagnostics：L1–L7 代码、消息、文件:行:列；点击可跳到 Source；
- TOML Source：CodeMirror 6 只读展示 composition 入口；
- 外部修改：2s digest 轮询；dirty 时禁编辑并提示 Reload；
- 空状态 / 无效项目 / 有效项目三种界面；
- 布局只写 `.designer/layout.toml`，编辑写 `composition/**` 但只经 Core 事务；
- 所有数据来自 `composition_core` / `composition_editor`，React 不解析 TOML。

### Instance 编辑的边界（022）

- 非 Root Instance 必须有一条父 Binding，因此“创建 Instance”与创建它的首个 Binding
  是同一个事务；单独 `create_instance` 会被 L5 拒绝。
- 创建候选只包含“创建后即可合法”的模块：模块自身不能声明必填 Slot。带必填 Slot 的
  复合模块需要连同其子树一起创建，属于 023 的 Slot 感知结构编辑。
- 换模块时保留名称/类型兼容的配置；目标契约不认识的键会在同一事务中 Unset，
  新必填属性必须填写。最终合法性由 Core 重新校验。
- 删除是整棵子树：先删后代再删目标，`delete_instance` 级联清理 Binding/Connection/
  Provider；是否仍满足 Slot/依赖约束由 Core 判定。
- 重新选择/修改/删除已有 Binding 不在本卡；023 会复用同一事务与 Review 机制。

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

## UI 自动化（020 / 022）

```powershell
npm run test:ui
```

- Playwright 使用系统 Edge（`channel: "msedge"`），不额外下载浏览器；
- 通过 Vite preview 运行生产构建；
- 测试在页面加载前注入测试专用 `window.__TAURI_INTERNALS__` fixture；
- 生产代码不包含 fixture，真实 Tauri 后端仍由 host tests 覆盖；
- 15 条链路：valid/invalid、选择、配置 Review/Apply/Cancel、拒绝、外部修改 Reload、
  布局保存，以及 Instance 重命名、换模块、Slot 下创建子实例、子树删除、
  Root 删除禁用、非法重命名本地拦截、拒绝事务不落盘。
- `src-tauri/src/lib.rs` 另有 9 个 host tests，其中 4 个直接使用真实
  `ProjectSession` 验证多命令事务与 TOML 写回（创建 + Binding、删除级联、
  拒绝不写文件、重命名重写 root/引用）。
