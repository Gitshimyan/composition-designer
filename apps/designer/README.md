# apps/designer/

v0.1 可视化设计器：Tauri 2 宿主 + React/TypeScript 界面。**M3 012–015 已建立外壳、
结构 Canvas/Tree、Inspector 配置编辑/Change Review、CodeMirror Source 与外部修改
同步/Reload；M6 022–025 增加 Instance/Binding/Connection/Dependency 编辑，
026 完成 Change Review 的分类、Before/After、依赖影响与完整诊断；M7 029–031 把 Source
升级为可编辑（草稿/Review/冲突只读），027 在 Glide 试点副本完成 CLI/Protocol 全链路验收；
S1 034 增加 Services 视图、service 创建、scope 与接口目标展示。**

| 区域 | 目录 | 职责 |
|---|---|---|
| Composition Canvas | `src/canvas/` | 实例与结构 Binding 的 `@xyflow/react` 呈现（013） |
| Project Explorer | `src/explorer/` | 项目目录、结构 Tree、模块来源与搜索 |
| Inspector | `src/inspector/` | 实例详情、scope/接口展示、配置编辑、Instance/Binding 操作与 Change Review |
| Relation Views | `src/relations/` | Structure/Connection/Dependency/Services 投影与编辑（024、034） |
| Change Review | `src/review/` | 分类、Before/After、命令、Dependency 影响、完整诊断（026） |
| TOML Source | `src/source/` | CodeMirror 6 可编辑 Source、草稿/Review/冲突只读与诊断定位（014、030） |
| Diagnostics | `src/diagnostics/` | 错误/警告、源码位置、修正提示 |
| Agent Status | `src/agent-status/` | Agent 会话、Revision 与操作记录 |
| Tauri Host | `src-tauri/` | `open_project(path, profile?)` / 编辑事务 / layout / source IPC |

## 当前外壳

- 顶栏：打开项目、路径输入、项目标识；v3 项目显示 Profile 选择器；Undo/Redo（Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y）；
- Explorer：项目、模块、结构 Tree（层级/slot/选择/聚焦）；
- Composition：`@xyflow/react` 实例节点 + Binding 边、Fit/Reset、拖动节点；
- Inspector：
  - 选中实例的模块/契约与配置编辑器（Set/Unset Config）；
  - 显示实例 `scope`（structure/service）与模块声明的 interfaces（034）；
  - 重命名 Instance（引用重写由 Core 事务完成）；
  - 换 Module / Version（同一事务内移除不兼容配置、补齐必填属性）；
  - 删除 Instance（Review 前展示子树、Binding、Connection、Dependency 影响）；
  - 在 Slot 下创建子结构：`create_instance` + `create_binding` 同一事务，
    必填属性在对话框内填写；模块的 required Slot 会递归展开；
  - Binding 列表：父 Binding 与子 Binding、Move（改 parent/slot）、Detach；
  - Move 对话框展示 Slot 的 accepts、cardinality、required、当前占用与子节点；
- Stage 关系视图（024）：
  - `Structure`：实例 + 结构 Binding 的 Canvas；
  - `Connections`：Connection 列表、每个实例的 Port 投影（方向/type/protocol/multi/
    连接关系），Create/Edit/Delete 经 Core 事务；
  - `Dependencies`：按实例分组的依赖要求、required/optional、Provider（显式/自动）、
    候选与解析状态；模块 ID 或接口身份（v2）作为目标；可设置显式 Provider 或清空回退
    自动解析（025 Extend、034）；
  - `Services`（034）：service 实例列表（模块、接口、config/provider/connection 计数、
    被引用者）、选中与创建；v1 项目提示需要 schema v2；
  - 四个视图共享 `selectedId`，切换后保持选择并高亮；
- Diagnostics：L1–L7 代码、消息、文件:行:列；点击可跳到 Source；
- Change Review（026）：命令列表、Instance/Binding/Connection 分类、Before/After、
  Dependency Provider 影响、完整诊断与 accepted/rejected 分区；取消不写入，拒绝/失败保留诊断；
- TOML Source（030）：CodeMirror 6 可编辑 composition 入口；draft、Review 文本 + 对象
  Diff、Discard；外部变更时只读并提示 Reload；仅 Composition 入口，模块文件仍只读（UW-031）；
- 外部修改：2s digest 轮询；dirty 时禁编辑并提示 Reload；
- 空状态 / 无效项目 / 有效项目三种界面；
- 布局只写 `.designer/layout.toml`，编辑写 `composition/**` 但只经 Core 事务；
- 所有数据来自 `composition_core` / `composition_editor`，React 不解析 TOML。

### 结构编辑的边界（022 / 023）

- 非 Root Instance 必须有一条父 Binding，因此“创建 Instance”与创建它的首个 Binding
  是同一个事务；单独 `create_instance` 会被 L5 拒绝。
- 创建向导只列 Slot 契约 `accepts` 内的模块；模块的每个 required Slot 递归要求至少一个
  子实例，所有实例与 Binding 在同一事务提交。可选 Slot 之后用 Add child 继续补。
- Move Binding 只改 parent + slot，child 及其子树保持不变；parent 候选排除 child 的
  后代，Slot 候选按模块契约过滤，最终是否合法由 Core 判定。
- v0.1 单根组合要求非 Root 实例可达，单独 Detach 会使 child 不可达并被 Core 拒绝；
  UI 仍展示 Detach 的 Review 与真实诊断；移除子结构请用 Delete（022）。
- 换模块时保留名称/类型兼容的配置；目标契约不认识的键会在同一事务中 Unset，
  新必填属性必须填写。
- Connection / Dependency 视图（024 / 025）：
  - Connection 端点按 Port 方向分组；type/protocol/multi/重复端点由 Core 校验；
  - Dependency Provider 通过 `set_dependency_provider` / `unset_dependency_provider`
    编辑，写回 `[instances.dependencies]`；显式映射可清空回退自动解析；
    requirement/provider 合法性、自引用与歧义由 Core 最终判定。
- Connection / Dependency 视图与完整 Before/After Review 分别属于 024 / 025。

### Services 与 scope（034）

- v2 项目可创建 `scope = "service"` 的实例；service 不进入 Binding 树，创建时不产生 Binding。
- 只能选择 `primitive` 模块；Core 会以 `contract-service-module-invalid` 拒绝 composite service。
- Services 视图展示接口、config/provider/connection 计数与被引用者，并与 Dependency 视图共享选择。
- 依赖目标可以是接口；候选按模块声明的 interface 身份匹配，显式 Provider 仍经 Core 校验。
- v1 项目不提供可用的创建入口，并提示需要 schema v2。

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

## UI 自动化（020 / 022–026 / 030 / 034）

```powershell
npm run test:ui
```

- Playwright 使用系统 Edge（`channel: "msedge"`），不额外下载浏览器；
- 通过 Vite preview 运行生产构建；
- 测试在页面加载前注入测试专用 `window.__TAURI_INTERNALS__` fixture；
- 生产代码不包含 fixture，真实 Tauri 后端仍由 host tests 覆盖；
- 34 条链路：valid/invalid、选择、配置 Review/Apply/Cancel、拒绝、外部修改 Reload、
  布局保存，Instance 重命名/换模块/创建/子树删除/非法 ID/拒绝事务，
  Binding 移动、Detach 诊断、required Slot 子树创建，
  三视图切换/选择保持、Connection 创建/修改/删除、Dependency 投影、
  Provider 设置/清空，完整 Review 的 Before/After、多命令分类、
  Connection 字段与 Dependency 影响，Source 编辑 + 文本 Diff 与非法文本拒绝，
  以及 Services 列表/scope/接口展示、接口依赖目标、service 创建与 v1 限制。
- `src-tauri/src/lib.rs` 另有 17 个 host tests，其中 13 个直接使用真实
  `ProjectSession` 验证多命令事务与 TOML 写回（Source 文本写回、service scope 写回、
  创建 + Binding、删除级联、拒绝不写文件、重命名 root/引用、Binding 移动、
  Binding 子树创建、Detach 拒绝、Connection 创建/修改/删除、非法方向拒绝、
  Dependency Provider 设置/清空）。
