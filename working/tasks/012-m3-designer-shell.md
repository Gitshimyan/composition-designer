# 012 — M3 工程打开与 Designer 外壳

- 状态：done
- 目标：按已确认技术栈建立 Tauri 2 + React/TypeScript Designer 外壳：打开项目目录，经 Rust
  Core 读取只读 `ProjectSnapshot`，展示项目/实例/模块/诊断与只读组合预览；Canvas 与 Source
  的真实交互留到 013/014。
- 授权：Extend（新桌面应用、前端工程、Tauri 配置）
- 变更声明：`changes/012-designer-shell.yaml`

## 技术栈（用户确认）

| 层 | 选择 |
|---|---|
| Domain / Core / Editor | Rust（已有 5 个 crate） |
| TOML | `toml_edit` / `composition-toml` |
| Desktop Host | Tauri 2 |
| UI | React + TypeScript + Vite |
| Canvas | `@xyflow/react`（013 接入） |
| Source Editor | CodeMirror 6（014 接入） |
| Agent | 本地协议 + MCP SDK（M4） |

## 背景

M2 已有只读 Core 与可写 `ProjectSession`。012 只做“打开与外壳”，不引入第二套模型：

- Tauri 命令 `open_project` 调用 `composition_core::validate_project`；
- 返回 `ProjectSnapshot`（模型 + 诊断 + digest + 计数），React 只渲染；
- 打开目录用 Tauri dialog；无 Tauri 时提供路径输入用于开发调试；
- 支持 `--project <path>` 启动参数，便于自动化验收；
- 不实现编辑、Canvas 拖拽、Source 编辑。

## 清单

- [x] `apps/designer` React/TS/Vite 工程
- [x] `src-tauri` Tauri 2 宿主、dialog 插件、capability
- [x] `open_project` 命令：Core 校验 + `ProjectSnapshot`
- [x] `initial_project` 命令：`--project` 启动参数
- [x] React 外壳：顶栏、Explorer、只读 Composition 预览、Inspector、Diagnostics、状态栏
- [x] 打开项目流程：dialog + 路径输入 fallback + 错误态
- [x] 空状态 / 无效项目 / 有效项目三种界面
- [x] 基本无障碍与键盘焦点；深色桌面视觉
- [x] `npm run build`、`cargo build`、Tauri 启动验证；更新 plan、README、docs/ai、任务卡

## 验收

- `npm run build`：TypeScript + Vite 构建通过；
- `cargo build --manifest-path src-tauri/Cargo.toml --features custom-protocol` 通过；
- `cargo test`（Tauri 宿主）：2 passed；
- 窗口可启动，标题 `Composition Designer`；已打开 `examples/minimal-project` 验证：
  项目 `example.minimal`、2 实例、2 模块、0 错误；
- 已打开 `invalid-projects/connections/errors` 验证：3 errors / 0 warnings，诊断显示
  `connection-direction-invalid`、`connection-type-mismatch`、`connection-duplicate` 与文件位置；
- 路径输入在非 Tauri 环境给出明确提示；
- Rust 只调用 Core，没有第二套解析/校验；
- 不写 TOML；Canvas/Source 编辑未实现。

## 结果 / 遗留问题

- 结果：
  - 新增 `apps/designer`：Tauri 2.12 + React 19 + TypeScript + Vite 7；
  - Rust 宿主 `open_project` / `initial_project`，返回只读 `ProjectSnapshot`；
  - React 外壳包含 Explorer、只读 Composition 卡片预览、Inspector、Diagnostics、状态栏、
    空状态、错误态与路径输入 fallback；
  - 支持 `--project <path>` 启动；Tauri 窗口显式定位/尺寸适配 150% DPI；
  - 2 个 Tauri 宿主测试（有效 minimal / 无效 entry-missing）。
- 关键决策：
  - Designer 不持有第二个模型，所有数据来自 `composition_core::validate_project`；
  - app 不加入根 Cargo workspace（`exclude`），避免 `cargo test --workspace` 被 Tauri
    依赖拖慢；使用独立 `src-tauri/Cargo.lock`；
  - 用 `custom-protocol` 构建嵌入前端的 debug 可执行文件；
  - 012 只读，不接 `ProjectSession`。
- 遗留：
  - Canvas 与 Source 只是占位，013/014 接入 `@xyflow/react` / CodeMirror；
  - 没有前端自动化 UI 测试；以 `npm run build` + 真实窗口截图验证；
  - 窗口尺寸/位置按当前 150% DPI 屏幕显式设定，后续应改为按显示器工作区自适应；
  - Tauri bundle/installer 未启用（`bundle.active=false`）；
  - dev 启动脚本 `start` / `start-invalid` 固定示例路径。
- 下一步：013 Canvas 与 Tree。
