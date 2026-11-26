# 术语表（Glossary）

- 状态：M0 冻结（2026-11-26）
- 适用范围：Composition Designer v0.1
- 规则：每个术语只有一个定义；括号内为常见误用或反例。

## 1. 模型与声明

| 术语 | English | 定义 | 常见误用 |
|---|---|---|---|
| Composition Model | Composition Model | 项目结构的唯一权威模型，由项目声明解析得到 | （把 Canvas 布局或编辑器状态当成模型） |
| 项目声明 | Project Declaration | `project.toml`、`composition/**`、`modules/**/module.toml` 组成的 TOML 文件集合 | （把 `.designer/**` 也算作项目声明） |
| 规范化模型 | Normalized Model | Core 输出的确定性、可序列化模型，含来源位置与诊断 | （把 TOML 原文直接当规范化模型） |
| 表现层状态 | Presentation State | `.designer/**` 中的坐标、视口、面板与偏好 | （把节点坐标写进 `composition/**`） |
| 声明格式版本 | schema_version | 声明文件格式版本；v0.1 为 `1` | （与模块版本混用） |

## 2. 项目与模块

| 术语 | English | 定义 | 常见误用 |
|---|---|---|---|
| 项目 | Project | 一个工作区：项目身份、Composition 入口与模块来源 | （把单个模块目录当项目） |
| 项目根 | Project Root | 包含 `project.toml` 的目录；所有声明路径的基准 | （依赖进程当前目录） |
| 模块定义 | Module Definition | 可复用模块的身份、版本、形态与契约 | （把项目中的某个实例当模块定义） |
| 模块 ID | Module ID | 点分小写稳定标识，如 `shell.desktop` | （用目录名或显示名代替） |
| 模块版本 | Module Version | 精确 `MAJOR.MINOR.PATCH`，与 `schema_version` 无关 | （用“最新版”或版本范围匹配） |
| 组合模块 | composite | 通过 Slot 接纳其他实例的模块 | （认为 composite 必须有子节点） |
| 原子模块 | primitive | 不再向下组合其他模块的叶子模块 | （给 primitive 声明 Slot） |
| 模块来源 | Module Root | 扫描 `module.toml` 的目录，相对项目根 | （用绝对路径或项目外目录） |

## 3. 实例与组合

| 术语 | English | 定义 | 常见误用 |
|---|---|---|---|
| 实例 | Instance | 某个模块定义在项目中的一次实例化，拥有独立配置 | （一个模块定义只允许一个实例） |
| 实例 ID | Instance ID | Composition 内唯一的实例标识 | （与模块 ID 混用） |
| 实例配置 | Instance Config | 用户显式写入的配置，只含模块声明的属性 | （包含未声明的“扩展字段”） |
| 有效配置 | Effective Config | 显式配置叠加默认值后的结果，不写回源文件 | （解析时把默认值写回 TOML） |
| 根实例 | Composition Root | 组合树的唯一根；必须是 composite | （多个根、根作为子节点） |
| 结构绑定 | Binding | 父实例的 Slot 接纳子实例的结构关系 | （用依赖或连接代替父子关系） |
| Slot | Slot | composite 模块接纳子实例的结构单元 | （把它当数据端口） |
| 单父规则 | Single Parent | 每个非根实例恰好一个父级 Binding | （一个实例挂在多个 Slot 下） |
| 基数 | Cardinality | Slot 允许的绑定数量：`one` / `optional_one` / `many` | （用 `required` 单独表示全部数量语义） |

## 4. 依赖与连接

| 术语 | English | 定义 | 常见误用 |
|---|---|---|---|
| 依赖要求 | Requirement | 模块声明“需要哪个模块/版本”的契约 | （把依赖当成结构包含） |
| 依赖解析 | Dependency Resolution | 为要求确定提供者实例的过程 | （自动挑选多个候选之一） |
| 提供者 | Provider | 满足某个依赖要求的实例 | （提供者必须是需求方的子节点） |
| 依赖环 | Dependency Cycle | A 依赖 B、B 依赖 A 的声明；v0.1 允许 | （当成 Binding 环而报结构错误） |
| 端口 | Port | 模块对外暴露的输入或输出边界 | （把 Slot 当端口） |
| 连接 | Connection | 输出 Port 到输入 Port 的显式数据/控制关系 | （用连接推断父子或依赖） |
| 多连接 | multi | Port 是否允许多条连接；默认否 | （默认允许任意扇入/扇出） |

## 5. 结果与过程

| 术语 | English | 定义 | 常见误用 |
|---|---|---|---|
| 诊断 | Diagnostic | 解析/验证发现的问题，带稳定代码与位置 | （只返回自然语言错误） |
| 错误代码 | Error Code | 稳定的机器可判断标识，如 `ref-module-not-found` | （用 message 文本判断类型） |
| 严重级别 | Severity | `error` / `warning` / `info` | （warning 当成可忽略的 error） |
| 来源映射 | Source Map | 模型对象与 TOML 文件位置的对应 | （只给文件名不给范围） |
| Revision | Revision | 已提交模型版本：`seq` + `source_digest` | （用文件修改时间当版本） |
| 源摘要 | source_digest | 声明文件路径与内容的 SHA-256 摘要 | （摘要包含 `.designer/**` 或当前时间） |
| 校验层 | Validation Layer | L1–L7 的检查分层 | （跳过 L2 直接做结构检查） |
| 未支持项 | Unsupported | 明确拒绝、给出诊断的能力 | （用隐式行为“尽量兼容”） |
