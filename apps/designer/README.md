# apps/designer/

v0.1 可视化设计器：Tauri 2 宿主 + React/TypeScript 界面。M3 起实现。

| 区域 | 目录 | 职责 |
|---|---|---|
| Composition Canvas | `src/canvas/` | 实例与结构 Binding 的呈现与编辑 |
| Project Explorer | `src/explorer/` | 项目目录、composition 文件、模块来源与搜索 |
| Inspector | `src/inspector/` | 选中实例、属性、Slot、Port、依赖 |
| TOML Source | `src/source/` | 权威 TOML 的查看与直接编辑 |
| Diagnostics | `src/diagnostics/` | 错误/警告、源码位置、修正提示 |
| Agent Status | `src/agent-status/` | Agent 会话、Revision 与操作记录 |
| Tauri Host | `src-tauri/` | 本地文件访问、窗口与 IPC |

边界：

- Canvas / Tree / Inspector 不持有权威 Composition 数据，节点坐标只写 `.designer/layout.toml`；
- 所有编辑转换为 Core 命令，经事务验证后提交；
- CodeMirror 只负责编辑体验与基础语法反馈，不实现项目语义校验；
- React 不重复实现 Core 的契约规则；
- Tauri Host 不直接修改 TOML 以绕开 Core。
