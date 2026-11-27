# Composition Designer — v0.1 设计与实现规范

> 文档状态：Draft / 设计基线  
> 目标版本：v0.1  
> 核心目标：**能够正确描述、修改、解析和验证一个完整项目。**  
> 配置格式：TOML  
> 核心原则：Composition-First

---

## 1. 项目定位

Composition Designer 是一个面向声明式项目组合的设计环境。它以 Composition Model 作为项目结构的唯一权威模型，通过 TOML 文件持久化项目声明，并提供可视化设计器、文本编辑器和 Agent 操作接口。

它不是传统 IDE，也不是单纯的可视化流程图工具。它首先解决的是：如何用明确、可验证、可修改的声明描述完整项目，以及如何让人类和 AI Agent 使用同一套模型完成项目设计。

### 1.1 v0.1 的目标

v0.1 聚焦于项目声明生命周期：

1. **描述（Describe）**：使用 TOML 描述项目、模块定义、模块实例、组合关系、配置及必要的接口关系。
2. **修改（Modify）**：通过可视化界面、TOML 文本编辑器或 Agent 接口修改同一份项目模型。
3. **解析（Parse / Resolve）**：将 TOML 与模块定义解析为规范化、确定性的 Composition Model。
4. **验证（Validate）**：检查语法、Schema、引用、模块契约、组合约束和依赖关系，并返回可定位的诊断结果。

完成 v0.1 后，系统应能够回答：

- 项目由哪些模块实例组成？
- 每个实例来自哪个模块定义、使用哪个版本？
- 模块之间如何组合？
- 每个模块实例配置了什么？
- 模块暴露和要求哪些接口、能力或依赖？
- 当前组合是否满足所有声明契约？
- 修改某个模块或关系会影响哪些对象？
- 当前解析结果是什么，是否存在错误或未解决项？

### 1.2 v0.1 暂不解决的问题

以下内容明确不属于 v0.1 的交付目标：

- 实际运行项目、启动服务或执行模块。
- 项目构建、打包、部署和发布。
- 根据 Composition 自动生成完整业务代码。
- 在线模块市场、远程包分发和复杂版本求解。
- 自动推断并插入兼容层、Adapter 或隐式依赖。
- 多 Agent 协作调度、Agent Runtime 或长时间自主执行。
- 通用脚本执行环境、在 TOML 中嵌入任意代码。

v0.1 可以提供结构预览和解析结果预览，但它们只用于理解声明，不代表项目已经能够运行。

---

## 2. 核心设计原则

### 2.1 Composition 是唯一权威模型

项目必须只有一个权威的 Composition Model。以下入口都只能读取或修改该模型：

- 可视化 Canvas、Tree 和 Inspector。
- TOML Source Editor。
- Agent Interface。
- CLI 和自动化测试入口。

它们不得各自维护一套互不一致的项目结构。

### 2.2 TOML 是持久化声明格式，不是执行语言

TOML 用于描述数据、模块引用、组合关系、参数、接口与约束。模块的具体实现仍然属于模块自身，不在 TOML 中嵌入任意脚本或业务执行逻辑。

### 2.3 模块组合必须显式

模块关系必须由项目声明明确表达。系统不得在解析过程中悄悄添加模块、替换模块、改写接口或插入兼容层。

### 2.4 先验证，再提交

所有修改都必须经过统一的解析和验证流程。非法修改不得写入权威项目文件，不得只提交事务中的一部分。

### 2.5 确定性解析

相同的项目文件、模块定义和解析规则，应产生相同的规范化解析结果和诊断结果。解析过程不得依赖随机顺序、GUI 状态或 Agent 的隐式假设。

### 2.6 表现层与项目语义分离

Canvas 节点坐标、缩放比例、面板展开状态等属于编辑器状态，不属于项目 Composition。移动画布节点不得改变项目语义。

### 2.7 显式处理不兼容

版本不匹配、属性不存在、Slot 不接受目标模块、接口类型不匹配等情况必须明确报错。系统不得采用“尽量兼容”的方式静默修复。

---

## 3. 核心领域模型

v0.1 至少区分以下对象。它们不能被混为同一种通用节点。

| 对象 | 定义 | 关键职责 |
|---|---|---|
| Project | 项目工作区及其入口 | 指定项目身份、Composition 入口和模块来源 |
| Module Definition | 可复用模块的定义与契约 | 声明模块身份、版本、属性、Slot、Port 和依赖要求 |
| Instance | 项目中某个模块的具体实例 | 将模块定义实例化，并保存本实例配置 |
| Binding | 实例之间的结构组合关系 | 声明父实例的哪个 Slot 接入哪个子实例 |
| Dependency | 模块或实例对其他模块/能力的要求 | 表达依赖要求，不等同于结构包含 |
| Port | 模块对外暴露的输入或输出接口 | 定义可连接的数据或控制边界 |
| Connection | Port 之间的显式连接 | 声明数据、事件或控制关系 |
| Diagnostic | 解析或验证发现的问题 | 提供级别、代码、位置、对象引用和说明 |
| Revision | 一次已提交的模型版本 | 支持并发检测、Diff 和历史追踪 |

### 3.1 Module Definition 与 Instance

**Module Definition** 描述一个可复用模块本身，例如 `frontend.workspace`。它定义模块可以接受哪些属性、提供哪些 Slot、暴露哪些 Port，以及需要什么依赖。

**Instance** 表示项目中实际使用该模块的一个实例，例如 `frontend`。实例引用模块 ID 和明确版本，并拥有自己的配置。

同一个 Module Definition 可以被多个 Instance 引用；每个 Instance 都必须有唯一 ID。修改一个实例的配置不得改变模块定义或其他实例。

### 3.2 Binding、Dependency 与 Connection 必须分离

三种关系表达不同语义：

- **Binding**：谁包含谁，解决结构组合。
- **Dependency**：谁需要谁或需要哪种能力，解决依赖要求。
- **Connection**：哪个输出 Port 连接到哪个输入 Port，解决显式通信或数据流关系。

不能因为 A 依赖 B 就推断 B 必须成为 A 的子节点；也不能因为 A 与 B 存在 Binding 就推断两者具有数据连接。

### 3.3 Composite 与 Primitive

建议模块初始区分两种形态：

- `composite`：通过 Slot 接纳其他模块实例，形成可嵌套的组合结构。
- `primitive`：不再向下组合其他模块，作为组合结构中的叶子模块。

Adapter 可以在后续作为明确的模块角色实现，不应在 v0.1 引入隐式 Adapter 机制。

---

## 4. TOML 项目文件组织

采用多文件 TOML。项目语义文件与编辑器表现数据分开保存。

### 4.1 建议目录结构

```text
MyProject/
├── project.toml
│
├── composition/
│   ├── main.toml
│   └── parts/
│       ├── frontend.toml
│       └── backend.toml
│
├── modules/
│   ├── shell/
│   │   └── module.toml
│   ├── frontend/
│   │   └── module.toml
│   └── backend/
│       └── module.toml
│
└── .designer/
    ├── layout.toml
    └── workspace.toml
```

目录职责：

| 路径 | 内容 | 是否影响项目语义 |
|---|---|---:|
| `project.toml` | 项目元信息、Schema 版本、Composition 入口、模块搜索路径 | 是 |
| `composition/` | 项目实际使用哪些实例及其组合关系 | 是 |
| `modules/` | 本地模块定义与契约 | 是 |
| `.designer/` | Canvas 坐标、视口、面板状态、编辑器偏好 | 否 |

`composition/parts/` 用于在项目变大时拆分声明。v0.1 必须明确文件包含/引用规则，禁止循环包含；如果首版尚未实现多文件 Composition，则先只支持 `composition/main.toml`，但目录结构不应预留含义模糊的隐式合并行为。

### 4.2 `project.toml` 示例

```toml
[project]
id = "example.application"
name = "Example Application"
schema_version = 1

[workspace]
composition = "composition/main.toml"
module_roots = ["modules"]
```

字段说明：

- `project.id`：稳定项目标识，不依赖项目目录名。
- `project.name`：用于 UI 显示的名称。
- `project.schema_version`：项目声明格式版本，与模块版本分开。
- `workspace.composition`：唯一入口 Composition 文件。
- `workspace.module_roots`：模块定义搜索目录，路径相对于项目根目录。

### 4.3 Composition 示例

`composition/main.toml`：

```toml
[composition]
id = "example.application"
schema_version = 1
root = "app"

[[instances]]
id = "app"
module = "shell.desktop"
version = "1.0.0"

[instances.config]
title = "Example Application"
theme = "light"

[[instances]]
id = "frontend"
module = "frontend.workspace"
version = "1.0.0"

[instances.config]
layout = "resizable"

[[instances]]
id = "backend"
module = "backend.service"
version = "1.0.0"

[[bindings]]
id = "app-content"
parent = "app"
slot = "content"
child = "frontend"

[[bindings]]
id = "app-service"
parent = "app"
slot = "service"
child = "backend"
```

此示例表达：

1. 项目的组合根实例为 `app`。
2. `app` 实例化 `shell.desktop@1.0.0`。
3. `frontend` 和 `backend` 分别实例化各自模块。
4. `frontend` 被绑定到 `app` 的 `content` Slot。
5. `backend` 被绑定到 `app` 的 `service` Slot。

实例配置属于实例，不属于模块定义。模块定义升级时，不能擅自覆盖用户实例配置。

### 4.4 Module Definition 示例

`modules/shell/module.toml`：

```toml
[module]
id = "shell.desktop"
version = "1.0.0"
kind = "composite"

[description]
summary = "Desktop application shell"

[contract.properties.title]
type = "string"
required = true

[contract.properties.theme]
type = "enum"
values = ["light", "dark"]
default = "light"

[[contract.slots]]
id = "content"
required = true
cardinality = "one"
accepts = ["frontend.workspace"]

[[contract.slots]]
id = "service"
required = false
cardinality = "one"
accepts = ["backend.service"]
```

其他模块可以按相同方式定义自己的属性、Slot、Port 和依赖要求。

### 4.5 TOML Schema 设计规则

1. `schema_version` 用于声明文件格式演进，不得与模块的 `version` 混用。
2. 模块使用稳定 ID 标识，版本单独声明。
3. 实例 ID 在整个 Composition 工作区内唯一。
4. Binding ID、Connection ID 等关系 ID 在所属 Composition 范围内唯一。
5. 所有路径必须有明确的相对路径基准；禁止依赖当前进程工作目录。
6. 配置字段必须由模块契约定义；不认识的字段不能静默忽略。
7. 列表顺序只有在 Schema 明确赋予语义时才有意义；无语义的集合在规范化模型中应稳定排序。
8. 默认值的处理规则必须明确：解析结果可以展开默认值，但不得在用户未编辑时擅自把默认值写回源文件。
9. 不支持隐式继承、隐式覆盖和自动兼容。
10. TOML 文件中的注释和用户格式应尽可能保留。

> 本节 TOML 是 v0.1 的设计草案。正式冻结 Schema 前，应补充字段类型、必填性、默认值、兼容规则和错误代码表，并为每个字段提供正反例测试。

---

## 5. Module Contract 契约体系

Module Contract 是 Designer 和 Agent 理解模块的唯一依据。UI 不应硬编码每个模块有哪些属性，Agent 也不应通过猜测或读取实现代码推断模块接口。

### 5.1 属性契约

每个属性至少声明：

- 类型：`string`、`integer`、`float`、`boolean`、`enum`、`array`、`object` 等。
- 是否必填。
- 默认值（如存在）。
- 约束：范围、长度、枚举值、格式等。
- 描述及适用范围。

验证时应同时检查类型和约束。未知属性应产生错误，而不是直接丢弃。

### 5.2 Slot 契约

Slot 定义 Composite 模块可以接纳的结构单元。至少包括：

- Slot ID。
- 是否必填。
- 基数：`one`、`optional_one`、`many` 等，最终枚举需统一。
- 可接受的模块类型或模块 ID。
- 是否允许重复实例（如项目模型支持）。

v0.1 建议采用严格的单父级结构：一个 Instance 最多被一个 Binding 接入，除非后续明确设计共享引用语义。

### 5.3 Port 契约

Port 用于表达模块之间显式的数据或控制连接。建议至少包含：

- Port ID。
- 方向：`input` 或 `output`。
- 类型标识。
- 基数或多连接规则。
- 可选的协议/版本信息。

Connection 必须引用存在的 Instance 和 Port，并检查方向与类型兼容性。v0.1 可以先实现简单的严格类型匹配，不应提前实现复杂的类型转换和自动适配。

### 5.4 依赖契约

依赖要求应与 Binding 分开描述。首版建议只支持明确、可解析的依赖声明：

- 被依赖的模块或能力标识。
- 版本约束。
- 是否必需。
- 依赖作用范围（如果确有必要）。

v0.1 不做复杂自动版本求解。无法唯一确定依赖来源时，应返回未解析诊断，要求用户显式指定。

---

## 6. Composition 解析与验证

解析和验证必须由独立于 GUI 的 Core 完成。Designer 和 Agent 只调用 Core，不自行实现另一套规则。

### 6.1 处理流水线

```text
Project Directory
       │
       ▼
Read TOML Sources
       │
       ▼
TOML Syntax Parse
       │
       ▼
Schema Validation
       │
       ▼
Module Resolution
       │
       ▼
Build Composition Model
       │
       ▼
Reference & Contract Validation
       │
       ▼
Dependency / Binding / Connection Validation
       │
       ▼
Normalized Composition Result
       │
       ▼
Diagnostics + Source Mapping
```

每一步应产生明确结果。语法错误时不能继续假装拥有完整模型；局部错误可在不破坏其他诊断的前提下尽量继续检查。

### 6.2 验证层次

| 层次 | 检查内容 | 典型错误 |
|---|---|---|
| L1：文件与语法 | 文件存在、可读、TOML 语法正确 | TOML 解析失败、入口文件缺失 |
| L2：Schema | 字段、类型、必填项、枚举和版本 | 未知字段、类型错误、缺少必填字段 |
| L3：标识与引用 | ID 唯一、引用存在、路径合法 | 重复 Instance ID、模块不存在 |
| L4：模块契约 | 属性、Slot、Port、依赖符合定义 | 未知属性、Slot 不接受该模块 |
| L5：组合结构 | 根节点、父子关系、基数和结构约束 | 多父节点、孤立实例、缺少必需 Slot |
| L6：连接与依赖 | Port 方向/类型、依赖版本和解析来源 | 输出连输入错误、依赖未解析 |
| L7：整体一致性 | 无悬空引用、无未处理冲突、结果可规范化 | 循环依赖、冲突或不完整组合 |

### 6.3 组合结构基本规则

v0.1 建议采用以下明确规则：

1. 必须存在且只能存在一个 Composition Root。
2. Root 必须引用一个已解析的 Composite Module Instance。
3. Root 不得作为其他实例的 child。
4. 非 Root Instance 必须且只能有一个父级 Binding。
5. Binding 的 parent 必须提供对应 Slot。
6. Binding 的 child 必须符合 Slot 的 `accepts` 规则。
7. Slot 的基数必须得到满足。
8. 不得引用不存在的 Instance、Module、Slot 或 Port。
9. 不允许重复 Binding。
10. 不允许存在无法从 Root 到达的未使用 Instance，除非未来明确引入独立资源/库节点类型。

如果后续确实需要共享节点、多个父级、外部资源或无根项目，应通过新的明确模型扩展，而不是放宽现有规则形成例外补丁。

### 6.4 解析结果

规范化结果至少应包含：

- Project 元信息。
- Composition Root。
- Instance 列表及其解析后的 Module Definition。
- 实例有效配置（包括已应用但未写回源文件的默认值）。
- Binding 结构。
- Dependency 解析结果。
- Connection 图（如项目声明了 Port 连接）。
- 模块版本和来源路径。
- 规范化后的稳定顺序。
- Diagnostics。
- Source Map：模型对象对应的文件路径和 TOML 位置。

解析结果应可序列化为机器可读数据，供 GUI、CLI、测试和 Agent 使用。

### 6.5 诊断格式

每条 Diagnostic 至少包括：

```text
code          稳定错误代码
severity      error / warning / info
message       面向人的说明
object_id     关联的 Instance、Module 或关系 ID（如果适用）
file          源文件路径（如果适用）
range         TOML 源码位置（如果适用）
related       关联对象或其他诊断（如果适用）
hint          可执行的修正方向（不能暗中代替用户修改）
```

错误代码必须稳定，不能只依赖自然语言文本供 Agent 判断。

---

## 7. 统一编辑模型与事务

所有修改入口必须转换成 Composition Core 可理解的命令。禁止 GUI 或 Agent 直接绕过 Core 修改内存模型或权威文件。

### 7.1 编辑命令

v0.1 至少支持以下命令类别：

- 创建、删除、重命名 Instance。
- 更换 Instance 的 Module Definition 或版本。
- 修改 Instance Config。
- 创建、删除、修改 Binding。
- 创建、删除、修改 Connection（如 Port Connection 纳入首版）。
- 修改模块定义及契约（仅在当前工作区允许编辑模块定义时开放）。
- 查询影响范围、生成 Diff。

每条命令必须有明确输入 Schema、前置条件和结果结构。

### 7.2 原子事务

一组有依赖关系的变更应在一个事务中提交：

1. 读取当前 Revision。
2. 基于当前模型生成候选模型。
3. 应用本次命令到候选模型。
4. 对候选模型执行完整验证。
5. 验证通过后提交 TOML 源文件和 Revision。
6. 验证失败则整体拒绝，不留下部分修改。

对于多个文件的修改，应使用临时文件、原子替换或可恢复的提交策略，避免中途失败造成项目处于半写入状态。

### 7.3 Revision 与并发保护

每次成功提交产生新的 Revision。写操作必须携带读取时的 `base_revision`：

- Revision 相同：可以进入事务验证。
- Revision 不同：拒绝直接覆盖，返回冲突和最新 Revision。
- Agent 重试：使用 `request_id` 或幂等键防止重复执行。

Revision 可由项目语义文件的内容摘要和编辑序列共同实现。具体算法在实现阶段确定，但不能仅依赖文件修改时间。

### 7.4 Diff 与影响分析

每次修改应提供结构化 Diff，而不只是文本 Diff。至少显示：

- 新增、删除和修改了哪些 Instance。
- 哪些 Binding、Dependency 或 Connection 发生变化。
- 哪些模块契约或版本发生变化。
- 哪些对象因此变为无效。
- 修改前后验证结果的变化。

删除或替换模块前必须先提供影响分析。系统可以给出建议，但不得自动添加替代模块或适配器。

### 7.5 TOML 源码保留

TOML 编辑必须考虑注释、格式和字段顺序：

- 优先采用保留语法树和格式信息的编辑库（Rust 可评估 `toml_edit`）。
- GUI 修改一个属性时，应尽量只改对应 TOML 节点。
- 不应每次保存都将整个文件重新格式化。
- 未被编辑的注释、空行和用户排版应尽可能保留。
- 如果无法安全局部修改，必须明确告知格式变化风险，不得静默重写。

---

## 8. 可视化设计器

界面是 Composition Model 的视图和操作入口，不是模型本身。

### 8.1 v0.1 主要区域

| 区域 | 职责 |
|---|---|
| Project Explorer | 项目目录、Composition 文件、模块来源和搜索 |
| Module Library | 展示已发现的模块定义及其契约摘要 |
| Composition Canvas | 展示实例和结构 Binding，支持结构编辑 |
| Tree View | 以层级方式查看组合结构 |
| Inspector | 查看和编辑选中实例、属性、Slot、Port、依赖 |
| TOML Source | 查看和直接编辑权威 TOML |
| Diagnostics | 展示错误、警告、来源位置和修正提示 |
| Change Review | 查看事务 Diff、影响分析和历史 |

### 8.2 Canvas 约束

Canvas 不是自由绘图工具。用户只能创建模型允许的结构关系。

例如：

- 将模块拖入项目时，先选择或生成 Instance，再绑定到合法 Slot。
- Slot 不接受目标模块时，禁止连接并显示契约原因。
- 删除 Instance 前显示其子节点、连接、依赖和其他影响。
- 替换模块时重新验证属性、Slot、Port 和依赖。
- 拖动节点只改变 `.designer/layout.toml`，不改变 Composition。

### 8.3 视图种类

v0.1 必须实现：

- Composition View：展示项目组合结构。
- Module Inspector：查看模块契约和实例配置。
- Source View：查看和修改 TOML。
- Diagnostics View：定位解析与验证问题。

Dependency Graph、Port Graph、Runtime View 可按数据模型成熟度后续增加。v0.1 的“预览”只表示解析后的结构，不代表运行时预览。

### 8.4 TOML 与 GUI 的双向同步

GUI 与 TOML 修改都进入同一模型更新流程：

```text
GUI Command ───────┐
                   ├──> Composition Core ──> Candidate Model
TOML File Change ──┘                              │
                                                  ▼
                                         Parse + Validate
                                                  │
                                       ┌──────────┴──────────┐
                                       ▼                     ▼
                                    Commit                Diagnostics
                                       │
                                       ▼
                              Canvas / Tree / Inspector
```

当外部编辑器修改 TOML：

1. 文件监听器检测变化。
2. 重新解析受影响文件及其依赖文件。
3. 构建候选模型并验证。
4. 有效时更新当前模型与视图。
5. 无效时保留最后一个有效模型，同时显示当前文件错误；不得自动用旧模型覆盖用户刚修改的文件。
6. 若编辑器内部存在未提交修改，必须进行冲突检测和明确处理。

---

## 9. Agent Interface

Agent 应从 v0.1 起被视为正式操作者，而不是未来附加的聊天功能。Agent 和 GUI 使用同一套领域命令与结果结构。

### 9.1 接口原则

- 能力可发现：Agent 能查询当前支持的命令、参数 Schema 和限制。
- 状态可观察：Agent 能读取项目、Revision、模块契约、诊断和变更结果。
- 修改可验证：写操作必须经过与 GUI 相同的解析和校验。
- 结果可追踪：每次操作返回 request ID、Revision、Diff 和诊断。
- Harness 无关：核心协议不依赖某个特定 Agent 产品或框架。

### 9.2 v0.1 建议操作

| 操作 | 用途 | 类型 |
|---|---|---|
| `workspace.describe` | 查询项目路径、会话和当前 Revision | Read |
| `capabilities.list` | 查询可用命令及 Schema | Read |
| `composition.inspect` | 查询完整或局部 Composition | Read |
| `registry.search` | 搜索本地可用模块 | Read |
| `module.inspect` | 读取模块定义和契约 | Read |
| `composition.validate` | 执行完整验证 | Read |
| `composition.plan` | 生成候选变更和影响分析 | Read |
| `composition.apply` | 原子提交变更 | Write |
| `composition.diff` | 查询结构化 Diff | Read |
| `history.list` | 查询已提交变更记录 | Read |
| `history.undo` | 撤销最近一次可撤销事务 | Write |
| `project.save` | 保存当前已验证状态 | Write |

`composition.plan` 不应偷偷修改项目；只有 `composition.apply` 才能提交变更。

### 9.3 Agent 标准工作流

```text
Discover
   ↓
Inspect Project / Module Contracts
   ↓
Plan Changes
   ↓
Review Impact
   ↓
Apply Transaction (base_revision + request_id)
   ↓
Validate
   ↓
Inspect Diff / Diagnostics
   ↓
Save or Continue
```

Agent 不应只得到“成功/失败”布尔值。写操作结果应返回：

- 是否提交成功。
- 新旧 Revision。
- 结构化 Diff。
- 验证摘要。
- 诊断列表。
- 失败时的冲突原因和可恢复信息。

### 9.4 协议适配

建议将领域命令与传输协议分开：

```text
Composition Core
    ├── Desktop IPC Adapter
    ├── Local CLI Adapter
    ├── Local JSON-RPC / JSONL Adapter
    └── MCP Adapter
```

MCP 只是一个适配器，不是 Composition Core 的组成部分。未来更换 Harness 时，不需要改变项目模型和核心编辑语义。

首版可以先实现进程内调用和 CLI/本地协议，再实现 MCP Adapter；但 Core 的命令 Schema 必须从开始就保持可机器读取。

---

## 10. 推荐技术架构

技术选择以单一领域核心、低耦合和可测试为优先。

| 层次 | 建议 | 职责 |
|---|---|---|
| Domain Core | Rust | 领域模型、命令、解析、验证、Revision、事务 |
| TOML | `toml_edit` + TOML parser | 语法树编辑、注释格式保留、解析 |
| Desktop Host | Tauri 2 | 本地文件访问、窗口和 IPC 宿主 |
| UI | React + TypeScript | 所有主要界面与交互 |
| Canvas | `@xyflow/react` 或等价图编辑库 | 节点和边的呈现、拖放交互 |
| Source Editor | CodeMirror 6 | TOML 文本编辑、诊断定位 |
| Agent Adapter | 本地协议 + MCP SDK | 将统一领域命令暴露给 Agent |

### 10.1 依赖边界

- Canvas 库不得持有权威 Composition 数据。
- CodeMirror 不负责项目语义验证，只负责编辑体验和基础语法反馈。
- React 不重复实现 Rust Core 中的契约规则。
- Tauri Host 不直接修改 TOML 以绕开 Core。
- Agent Adapter 不自行处理模块依赖和组合规则。
- 所有入口通过同一组 Command / Query 接口访问 Core。

### 10.2 项目内部模块建议

```text
composition-designer/
├── crates/
│   ├── composition-domain/       # 领域类型与不变量
│   ├── composition-toml/         # TOML 读取、Source Map、保留格式修改
│   ├── composition-resolver/     # 模块解析与规范化结果
│   ├── composition-validator/    # Schema、引用和契约校验
│   ├── composition-editor/       # 命令、事务、Revision、Diff、Undo
│   ├── composition-protocol/    # 机器可读命令与结果 Schema
│   └── composition-app/          # 应用服务编排
│
├── apps/
│   └── designer/
│       ├── src/
│       │   ├── canvas/
│       │   ├── explorer/
│       │   ├── inspector/
│       │   ├── source/
│       │   ├── diagnostics/
│       │   └── agent-status/
│       └── src-tauri/
│
├── examples/
│   ├── minimal-project/
│   ├── nested-composition/
│   └── invalid-projects/
│
└── docs/
```

该结构是职责建议，不要求首个提交就拆成大量 crate。若早期拆分带来不必要的边界成本，可以先在一个 Rust workspace crate 内按模块隔离。

---

## 11. 实施阶段与交付物

阶段顺序以先稳定语义、再实现界面、最后接入外部 Agent 为原则。Agent 所需的命令与 Schema 从早期就纳入设计，但不要求第一阶段完成所有传输适配器。

### M0 — Schema 与语义冻结

**目标：** 明确“什么是一个有效项目”。

工作项：

- 定义 Project、Module Definition、Instance、Binding、Dependency、Port、Connection。
- 冻结项目入口和 TOML 文件组织规则。
- 定义模块 ID、Instance ID、版本和路径规则。
- 定义属性、Slot、Port 和依赖契约。
- 定义组合根、父子关系、Slot 基数和引用约束。
- 定义 Schema 版本策略和 Diagnostic 格式。
- 准备合法、非法及边界 TOML 样例。

交付物：

- `composition-schema-v1.md`。
- TOML 示例项目。
- 字段定义表。
- 错误代码初始表。
- 正反例测试数据集。

退出条件：不同实现者只依赖规范和测试样例，也能判断同一个项目是否合法。

### M1 — TOML Engine + Composition Core

**目标：** 不依赖 GUI，完成完整项目的加载、解析、验证和规范化输出。

工作项：

- 加载项目入口及模块目录。
- 解析 TOML 并保留源文件位置信息。
- 校验 Schema 和引用。
- 解析模块定义与实例。
- 验证 Binding、Slot、属性和依赖。
- 生成规范化 Composition Model。
- 输出结构化 Diagnostics 和机器可读结果。
- 编写确定性和往返测试。

交付物：

- 可调用的 Rust Core。
- CLI：`inspect`、`resolve`、`validate`。
- 解析结果 JSON 样例。
- 自动化测试报告。

退出条件：对给定项目，Core 能稳定解析并给出可复现的验证结果；不需要 GUI 或 Agent 才能完成验证。

### M2 — 统一编辑命令与事务

**目标：** 以统一命令修改项目，并确保合法性和文件安全。

工作项：

- 定义 Command / Query Schema。
- 实现 Instance、Config、Binding 等修改命令。
- 实现候选模型、事务验证和原子提交。
- 实现 Revision、冲突检测和 request ID 幂等。
- 实现结构化 Diff、影响分析和撤销基础能力。
- 使用保留格式的 TOML 修改方式。

交付物：

- 可独立测试的编辑服务。
- 命令 Schema 文档。
- 编辑前后 TOML 与结构化 Diff 样例。
- 事务失败和冲突测试。

退出条件：非法变更整体失败；合法变更只修改预期声明；并发冲突不会覆盖新版本。

### M3 — 可视化 Designer

**目标：** 人类可以通过界面查看、修改和验证完整 Composition。

工作项：

- 项目打开、目录浏览和模块搜索。
- Composition Canvas 与 Tree View。
- Module Inspector 和属性编辑。
- Binding 创建、删除和替换。
- TOML Source View。
- Diagnostics 定位和 Change Review。
- `.designer` 视图状态保存。
- 外部 TOML 修改检测与冲突处理。

交付物：

- 可以打开示例项目的桌面设计器。
- Canvas、Inspector 与 TOML 双向同步演示。
- 外部编辑器修改文件后的刷新演示。

退出条件：GUI 不存在独立模型；所有用户编辑均经过 Core；移动画布节点不会改变项目语义。

### M4 — Agent Interface

**目标：** Agent 能通过机器接口独立理解、修改和验证项目。

工作项：

- 能力发现和命令 Schema 查询。
- 项目、模块、契约和 Revision 查询。
- Plan、Impact、Apply、Validate、Diff。
- 本地进程发现与会话管理。
- CLI / 本地 JSONL 或 JSON-RPC 接入。
- MCP Adapter 和 Agent 使用说明。
- Agent 操作记录及错误恢复。

交付物：

- Agent 工具定义。
- MCP Server 或 Adapter。
- Agent Skill / 操作指南。
- 从空项目创建组合并验证的端到端演示。

退出条件：Agent 不需要直接编辑文件或猜测契约，即可完成一项可复现的项目组合修改。

### M5 — 完整性与稳定性

**目标：** 保证 v0.1 在真实项目规模下可靠使用。

工作项：

- 多文件 Composition 管理。
- 大型模块目录性能测试。
- 崩溃恢复与未保存修改保护。
- 模块契约变更影响分析。
- 完整错误代码与用户提示。
- 端到端测试和项目迁移测试。

交付物：

- v0.1 稳定性报告。
- 完整示例项目集。
- v0.1 用户与 Agent 操作文档。

退出条件：通过本文件第 12 节的全部验收标准。

---

## 12. v0.1 验收标准

v0.1 不以“能显示画布”作为完成标准，而以项目声明的正确性和可修改性作为核心。

| 编号 | 验收项 | 通过条件 |
|---|---|---|
| A01 | 项目加载 | 能根据 `project.toml` 找到入口和模块来源 |
| A02 | TOML 解析 | 合法 TOML 可解析；非法 TOML 有准确文件和位置诊断 |
| A03 | Schema 验证 | 缺失字段、错误类型、未知字段均被识别 |
| A04 | 模块解析 | 每个 Instance 能定位唯一模块定义和指定版本 |
| A05 | 引用完整性 | 不存在悬空 Module、Instance、Slot、Port 引用 |
| A06 | 组合合法性 | Root、父子关系、Slot 基数和接受类型全部验证 |
| A07 | 配置验证 | Instance Config 与模块属性契约一致 |
| A08 | 依赖检查 | 必需依赖明确解析，不能解析时给出诊断 |
| A09 | 确定性 | 相同输入产生相同规范化结构与诊断结果 |
| A10 | 编辑一致性 | GUI、TOML、Agent 通过同一 Core 修改模型 |
| A11 | 事务安全 | 一组修改要么全部提交，要么全部不提交 |
| A12 | Revision 冲突 | 旧 Revision 不能覆盖新修改 |
| A13 | TOML 保留 | 局部修改尽量保留注释和原有排版 |
| A14 | 外部变更 | 外部 TOML 修改可重新解析；无效文件不会覆盖最后有效模型 |
| A15 | Canvas 隔离 | 画布布局变化不改变 Composition 语义 |
| A16 | Agent 闭环 | Agent 能发现、读取、计划、修改、验证并读取结果 |
| A17 | 错误可操作 | 诊断具有稳定代码、级别、定位和关联对象 |
| A18 | 无隐式兼容 | 系统不会静默插入模块、转换关系或覆盖属性 |

### 12.1 必须准备的测试项目

至少准备以下项目样例：

1. 最小合法项目：一个 Root 和一个叶子模块。
2. 嵌套组合：多层 Composite 与 Primitive。
3. 多实例复用：同一个模块定义被多个 Instance 使用。
4. Slot 基数边界：必需 Slot 缺失、单 Slot 重复接入、多 Slot 合法接入。
5. 非法引用：模块不存在、Instance 不存在、Slot 不存在。
6. 属性错误：未知属性、类型错误、枚举值错误、缺少必填属性。
7. 结构错误：多个 Root、孤立实例、多父节点、重复 Binding。
8. 版本错误：模块版本缺失或指定版本不可用。
9. 文件错误：入口缺失、路径越界、TOML 语法错误。
10. 编辑错误：事务中一条命令失败、Revision 过期、重复 request ID。
11. 外部修改：合法外部修改、非法外部修改、与未保存内容冲突。
12. 格式保留：包含注释、空行、不同字段顺序的 TOML 被局部修改。

---

## 13. 风险与约束

| 风险 | 后果 | 处理策略 |
|---|---|---|
| Schema 过早复杂化 | 解析器和 UI 难以稳定 | v0.1 仅实现明确必要的字段与关系 |
| GUI 自己维护数据 | 与 TOML 和 Agent 状态分叉 | 所有编辑经过 Composition Core |
| TOML 全量重写 | 注释、格式和用户手工整理丢失 | 使用保留语法树的局部编辑 |
| 隐式依赖解析 | 产生不可解释的项目结构 | 依赖来源必须显式、可追踪 |
| 兼容补丁累积 | 项目关系逐渐失去明确性 | 严格拒绝不兼容，Adapter 显式建模 |
| Agent 直接改文件 | 绕过验证和 Revision | 提供正式读写接口，记录每次事务 |
| 多文件写入失败 | 项目进入半更新状态 | 候选验证、临时文件和原子提交策略 |
| 过早加入 Runtime | 范围膨胀，核心语义未稳定 | v0.1 只解析和验证，不执行项目 |
| 依赖图与组合树混淆 | 关系语义不清，UI 难以理解 | Binding、Dependency、Connection 分离 |

---

## 14. v0.1 明确不做的设计

为保持边界清晰，以下能力不应以“预留扩展”为由提前进入核心：

- 不在 TOML 中执行脚本。
- 不自动生成缺失模块或兼容补丁。
- 不自动猜测模块应该连接到哪个 Slot。
- 不允许任意模块通过未声明属性接收配置。
- 不让 Canvas 自由创建没有领域含义的节点和连线。
- 不允许 UI 与 Agent 各自定义不同的编辑命令。
- 不将编辑器布局写入 Composition 文件。
- 不把模块依赖等同于结构包含。
- 不把“解析成功”表述为“项目可运行”。
- 不实现构建、部署或实际模块执行。

---

## 15. 待冻结决策

以下决策应在 M0 结束前明确，不宜在多个阶段中反复临时决定。

| 决策项 | 初始建议 | 冻结时需要明确的内容 |
|---|---|---|
| 模块版本策略 | v0.1 使用精确版本 | 是否支持版本范围，以及范围语义 |
| 模块来源 | 首版以项目内本地目录为主 | 是否支持外部只读模块目录 |
| Composition 拆分 | 先支持单入口；按明确规则扩展多文件 | include/import 语义、循环检测和覆盖规则 |
| Slot 基数 | 先支持 one / optional / many | 精确枚举及数量限制 |
| Port Connection | 首版支持简单方向和类型校验，或明确推迟 | 类型标识、协议版本、环路规则 |
| Dependency | 显式引用，不做复杂自动求解 | 模块依赖与能力依赖的首版范围 |
| 默认值 | 解析时计算，不自动写回 | 默认值来源、优先级与展示方式 |
| Revision | 内容摘要结合提交序列 | 哈希范围、跨文件事务和历史持久化方式 |
| 模块定义编辑 | 初期可只读，项目组合可编辑 | 模块定义是否在同一 Designer 内编辑 |

对于未冻结的能力，应在实现中明确标记为 unsupported，而不是以隐式行为代替尚未决定的规则。

---

## 16. 当前执行顺序

建议从以下顺序开始推进：

1. 冻结领域术语和最小对象模型。
2. 冻结 `project.toml`、Composition TOML 和 Module Contract 的字段定义。
3. 编写最小合法项目及覆盖主要错误类型的非法项目样例。
4. 实现不依赖 GUI 的 TOML Parse、Resolve 和 Validate CLI。
5. 验证规范化结果、Source Map 和确定性。
6. 实现统一编辑命令、事务、Revision 和结构化 Diff。
7. 再开始 GUI Canvas、Inspector 与 Source View。
8. 最后把相同命令通过本地协议和 MCP 暴露给 Agent。

**当前首要任务不是制作画布，而是完成 M0：冻结 Composition TOML Schema、Module Contract 和项目有效性规则。**

当 M1 的 Core 能独立证明一份项目声明是否合法、解析成什么结构时，前端和 Agent 才有可靠且统一的基础。
