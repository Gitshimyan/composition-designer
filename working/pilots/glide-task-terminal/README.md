# Glide Task + Terminal Pilot

- 来源项目：`E:\Work.Projects\Glide`
- 试点范围：Glide App 组合根 + Task + Terminal 垂直切片
- 性质：**只读映射**；本目录不复制 Glide 源码
- 结果：现有 v0.1 Core 验证通过；map 后的声明用于 M6-002 Schema 复核

## 1. 为什么选 Glide

Glide 是当前候选中最接近“真实产品 + 明确组合根 + 四类关系”的项目：

- `Glide.App/Composition/ServiceRegistration.cs` 是唯一抽象端口到 Windows 实现的绑定点；
- `Glide.Application` 定义 Task/Terminal 端口，`Glide.Platform.Windows` 提供 ConPTY 实现；
- `Glide.Presentation` 用 WPF 组合 Task/Terminal 面板；
- `Glide.Terminal.Web` 通过 WebView2 + xterm.js 与 ConPTY 建立真实消息连接；
- `glide.toml` 有 profile/task/shell/cwd/environment 配置；
- `main` 干净，README 记录 102 个测试（本次未重新执行 Glide 测试）。

`Composition-First-clean` 的声明化程度更高，但其 “bindings” 是依赖注入 provider，不是结构
父子关系，也没有 Port/Connection；它适合作为后续 Schema 对照，不作为主试点。

## 2. 映射

| Glide | Pilot 表达 |
|---|---|
| `Glide.App/Composition` | `glide.app` root + `runtime` slot |
| WPF Workbench | `glide.presentation.workbench` |
| TaskView / TaskViewModel | `glide.presentation.task-panel` |
| TerminalView | `glide.presentation.terminal-panel` |
| `IProjectTaskRunner` / `ProcessProjectTaskRunner` | `glide.runtime.task-runner` |
| `ITerminalSessionFactory` / ConPtySession | `glide.platform.conpty-session` |
| WebView2 + xterm bridge | `glide.terminal.web` |
| ServiceRegistration ports → implementations | instance `[instances.dependencies]` |
| Task command/output events | task-port connections |
| Terminal bridge frames | terminal-port connections |
| UI containment | `app → workbench → task/terminal` bindings |
| `glide.toml` profile/task/config | instance config（部分表达） |

## 3. 结果

```text
instances    7
bindings     6
connections  6
dependencies 3
modules      7
diagnostics  0
```

三类依赖均解析为显式 provider：

- task-panel → task-runner
- task-runner → conpty-session
- terminal-panel → terminal-web

结构树：

```text
app
├── workbench
│   ├── task-panel
│   └── terminal-panel
├── task-runner
├── terminal-web
└── conpty-session
```

## 4. 验证

```powershell
target\debug\composition-cli.exe validate working\pilots\glide-task-terminal --json
target\debug\composition-cli.exe resolve working\pilots\glide-task-terminal
```

本次结果：

- `valid=true`
- `error_count=0` / `warning_count=0`
- `source_digest=sha256:310944bf29521179698509a5c20c38d05005c3ae89be584339b7bac00f12ce48`

## 5. Schema 复核：能表达什么

- 模块定义、精确版本、模块种类（composite / primitive）；
- 结构父子 Binding 与 Slot 基数/接受类型；
- 实例配置、默认值和 effective config；
- 跨实例依赖与显式 provider；
- 输入/输出 Port、direction、type、protocol、multi；
- Connection 的方向、类型/协议相等、重复边校验；
- 单入口 Composition 与确定性解析。

## 6. 缺口清单

| # | 缺口 | 影响 | 当前处理 |
|---|---|---|---|
| G1 | Dependency 目标是具体模块 ID，不是接口/契约 ID | Glide 的 `ITerminalSessionFactory` 等端口无法与实现分离；provider 绑定被压缩成“模块 → 模块” | 试点用端口模块名近似；后续评估 Contract/Interface 是否需要独立身份 |
| G2 | 没有非结构的 runtime/service 容器 | 为让 `task-runner`/`terminal-web`/`conpty` 可达，root 增加人工 `runtime` Slot，形成“服务被父级包含”的假结构 | 保留为已知映射失真；022 UI 不应把它当真实 UI 层级 |
| G3 | 没有项目级 Profile / 条件选择 | `glide.toml` 的 profile、环境、task 列表无法完整表达；console/file logger、desktop/headless 切换只能靠不同 Composition | 当前只把 `active-profile` 作为实例 Config；后续评估 Profile/Composition 变体 |
| G4 | Module 只有 composite/primitive | Glide 的 UI / service / adapter / platform 类别与生命周期（eager/graceful）无法表达 | 不影响结构验证；避免为 UI 方便提前扩展 kind |
| G5 | 没有实现/技术/进程元数据 | WPF、WebView2、ConPTY、xterm、项目引用、入口点不在模型中 | 仅写在 README 映射；不进入权威 Composition |
| G6 | Port 只有类型/协议字符串相等 | ConPTY 原始字节流、编码边界、桥接消息 envelope、生命周期没有 Schema 表达 | `glide.terminal.frame.v1` 是抽象占位；不是 wire schema |
| G7 | Config 是实例级静态值 | profile-scoped task 列表、环境 overlay、命令模板无法表达 | 只映射少量标量配置 |
| G8 | 多终端实例只能重复声明 | 多个 terminal-panel/conpty 实例需要手工重复 config/connection；没有工厂/模板 | 当前映射单会话；多实例作为后续 UI 操作验证 |

## 7. 结论与下一步

- 现有 v0.1 模型足以表达 Glide Task + Terminal 的**拓扑**：结构、依赖、端口、连接和配置；
- 不能完整表达 Glide 的**接口身份、Profile 变体、运行时/技术元数据**；这是 Schema 复核结论，
  不在 021 直接修改冻结 Schema；
- 建议 022 先使用现有模型完成 Instance/Binding 编辑；G1/G2/G3 进入后续 Extend / Rearchitect
  候选，由 M6-003–005 的真实 UI 需求验证是否必须；
- 不复制 Glide 源码；后续如需回归，可将本目录作为 Composition Designer 项目直接打开。
