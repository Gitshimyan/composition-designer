# Composition Designer — 项目评审与后续发展方向

> **评审对象：** [Gitshimyan/composition-designer](https://github.com/Gitshimyan/composition-designer)  
> **评审日期：** 2026-11-27  
> **评审版本：** `main` 分支，v0.1（M0–M5 已收口）  
> **文档性质：** 架构与产品方向评审、后续阶段建议  
> **当前建议：** 保持 v0.1 冻结；下一阶段以 **M6 — Complete Composition Authoring** 为核心目标。

---

## 1. 执行摘要

Composition Designer v0.1 已经形成一套具备实际工程基础的声明式项目设计系统。它不再只是 TOML 编辑器或可视化节点画布，而是已经建立了统一的领域模型、TOML 加载与解析、组合验证、事务式编辑、结构化 Diff、桌面设计器、CLI，以及面向 Agent 的 Protocol / MCP 接口。

项目当前最重要的架构成果是：**Composition Model 是项目组合的唯一权威模型；Designer、CLI、Agent 和 MCP 作为不同入口，通过共同的 Core 与编辑事务读取或修改项目。** 这为以后接入不同 Agent Harness、增加新的交互界面和扩展组合能力提供了稳定基础。

不过，v0.1 的完成不等于完整的可视化组合设计体验已经完成。目前存在三个主要缺口：

1. **Designer 的可视化组合编辑尚不完整。** 当前界面能够展示结构、查看契约并修改实例配置，但创建/删除实例、编辑 Binding、编辑 Connection、管理 Dependency Provider 等操作尚未形成完整 GUI 链路。
2. **TOML Source 仍然只读。** 这使可视化编辑、文本编辑与 Agent 修改尚未达到完整对等；Source 编辑需要接入统一解析、Diff、验证和事务提交流程，不能另建一套写入逻辑。
3. **工程质量门禁和 UI 自动化验证需要补齐。** Core 测试较完整，但仓库当前未见 GitHub Actions workflow，且验收报告明确记录尚无前端自动化 UI 测试。

因此，下一阶段不建议立即开发运行时、代码生成、在线模块市场、多 Agent 或复杂版本求解。更合理的方向是：先使用真实项目验证当前领域模型，再补齐完整组合编辑体验，随后实现 Source 编辑与 Agent 操作体验增强。

### 1.1 总体判断

| 维度 | 评审结论 | 说明 |
|---|---|---|
| 领域模型 | 良好 | Project、Module Definition、Instance、Binding、Dependency、Port、Connection 等对象职责明确 |
| TOML 与解析 | 良好 | 具有加载、Source Map、Digest、规范化结果和确定性验证基础 |
| 组合验证 | 良好 | 已覆盖 Schema、引用、契约、结构、连接和依赖等检查 |
| 编辑事务 | 良好 | 已有候选模型、原子事务、Revision、幂等、Diff 和保留格式写回 |
| 桌面 Designer | 需要继续完成 | 当前以查看、配置编辑和部分交互为主，完整组合编辑缺失 |
| Agent 接入 | 基础闭环已建立 | CLI / JSONL / MCP / Skill 已能访问统一 Core 与事务能力 |
| 工程自动化 | 需要补齐 | 缺少可见的 CI workflow 和前端 UI 自动化测试 |
| 扩展生态 | 暂不成熟 | 真实项目验证、模块版本演进和多项目场景尚未充分验证 |

### 1.2 下一阶段目标

建议将 v0.2 的目标定义为：

> **让人类用户和 Agent 都能够通过明确、可验证、可审查的方式，完成一个项目的完整声明式组合设计。**

重点在完整的 Authoring 能力，而不是让 Composition Designer 立即承担项目构建、运行或部署职责。

---

## 2. 项目定位与边界

Composition Designer 的定位应保持为：**面向声明式项目组合的设计环境与组合模型工具链**。

它首先解决如何明确描述项目由什么组成、模块之间如何组合、模块契约如何约束使用方式，以及如何安全修改和验证这些声明。

### 2.1 核心职责

1. **Describe：** 使用 TOML 描述项目、模块定义、实例、组合关系、配置、接口和依赖。
2. **Resolve：** 将声明解析为规范化、确定性的 Composition Model。
3. **Validate：** 检查语法、Schema、引用、模块契约、结构、连接和依赖，并返回可定位诊断。
4. **Modify：** 通过统一命令与事务机制修改项目声明。
5. **Review：** 展示修改 Diff、影响范围和验证结果，再决定是否提交。
6. **Expose：** 通过 Designer、CLI、Protocol 和 MCP 向人类与 Agent 提供一致的操作能力。

### 2.2 明确不属于当前 Core 的职责

以下能力不应为了追求功能完整而提前混入 Composition Core：

- 项目实际运行与进程管理；
- 项目构建、打包、部署和发布；
- 根据声明自动生成完整业务代码；
- 在线模块市场与远程包分发；
- 自动插入 Adapter、隐式兼容层或静默修复；
- 多 Agent 调度与长期自主执行；
- 在 TOML 中执行任意脚本或命令。

这些能力即使未来需要，也应在独立的执行层、工具链或适配层中设计，通过明确接口消费 Composition Model，而不是反向污染领域模型。

### 2.3 三类关系必须保持分离

| 关系 | 表达的问题 | 不应被误解为 |
|---|---|---|
| Binding | 谁包含谁、子实例放入父实例的哪个 Slot | 普通依赖或数据连接 |
| Dependency | 一个模块需要什么，哪个实例提供所需依赖 | 父子结构关系 |
| Connection | 哪个输出 Port 与哪个输入 Port 显式连接 | 模块包含或模块依赖 |

这三类关系必须保留独立的语义、校验规则和可视化表达。不能为了简化 Canvas，把它们统一降级为没有类型语义的普通边。

---

## 3. v0.1 当前实现盘点

以下状态依据仓库 README、验收报告、设计规范、目录结构和相关源码进行归纳。

### 3.1 核心 Rust 工程

| crate | 当前职责 | 评审 |
|---|---|---|
| `composition-domain` | 领域类型、标识、契约、诊断、Revision、Composition Model | 职责清晰，应继续保持纯领域层 |
| `composition-toml` | TOML 加载、文件发现、路径处理、Source Map、Source Digest | 是持久化声明入口，应避免承担领域规则 |
| `composition-core` | 解码、模块解析、规范化、L1–L7 验证 | 作为唯一解析与验证入口应继续保持 |
| `composition-editor` | 编辑命令、候选事务、Revision、幂等、Diff、TOML 写回和恢复 | 是统一修改机制的关键，应避免 UI 或 Adapter 绕过它 |
| `composition-cli` | inspect / resolve / validate，以及 query / apply / JSONL | 有利于自动化和 Agent 集成 |
| `composition-protocol` | Query / Command 协议与 JSONL | 应保持传输协议与领域语义分离 |
| `composition-mcp` | MCP stdio、Tools、Resources | 作为薄适配器是合理方向 |

当前 `composition-resolver` 和 `composition-validator` 暂时并入 `composition-core`，`composition-app` 尚未建立。这种按实际复用需求再拆 crate 的策略是合理的，不建议仅为目录整齐而提前拆分。

### 3.2 Designer 桌面应用

当前桌面端采用：

- Tauri 2；
- React 19 + TypeScript；
- Vite 7；
- `@xyflow/react` 用于结构画布；
- CodeMirror 6 用于 TOML Source 展示。

已经具备的主要能力：

- 打开项目并展示项目身份、模块清单和组合结构；
- Structure Tree 与 Canvas 选择联动；
- 展示 Instance 和 Binding；
- 保存 Canvas 布局到 `.designer/layout.toml`，与项目语义隔离；
- Inspector 查看实例、模块契约和属性；
- 通过事务机制修改部分实例配置；
- Change Review 预览事务结果；
- 显示诊断并尝试定位 Source 行；
- 检测外部文件变化，并在冲突时阻止本地修改；
- 只读查看 Composition TOML。

当前 GUI 的主要不足不是“缺少更多面板”，而是完整的组合编辑操作尚未全部进入用户界面。

### 3.3 Agent / CLI 接口

当前 Protocol 和 MCP 已提供：

- `composition_inspect`；
- `composition_resolve`；
- `composition_validate`；
- `composition_source`；
- `composition_prepare_transaction`；
- `composition_apply_transaction`。

Agent Skill 规定了 inspect → resolve → prepare → review → apply → validate 的基本工作流，并明确禁止 Agent 直接编辑项目 TOML 绕过事务。

这是正确的基础。后续主要工作应放在提高 Agent 对项目状态、修改影响、事务结果和恢复条件的理解效率，而不是无节制增加 MCP Tools 数量。

### 3.4 v0.1 验收情况

仓库 `docs/acceptance-v0.1.md` 记录：

- A01–A18 验收项全部通过；
- 146 项 Core / Protocol / MCP / CLI 测试；
- 5 项 Tauri 宿主测试；
- 26 个示例数据集（其中包含入口缺失场景）；
- MCP stdio 端到端测试覆盖 inspect、resolve、prepare、apply、validate、source；
- 包含 80 实例 + 80 Binding 的合成项目性能测试；
- 包含写回中断备份恢复、单入口边界和不支持 Schema 版本检查。

这些结果说明 v0.1 的声明解析、验证和事务基础已经形成可重复验证的工程闭环。

需要注意：上述数字是仓库验收文档记录的结果，不代表本次评审在评审者环境中重新执行了全部测试。

---

## 4. 架构评审

### 4.1 单一权威模型：应继续作为最高原则

当前架构中最值得保留的原则是：

- Composition Model 是唯一权威模型；
- Designer、CLI、Agent、MCP 不得各自维护另一套项目模型；
- 所有项目修改都通过统一命令和事务；
- Canvas 布局、面板状态等表现数据不属于项目语义；
- Source、GUI 和 Agent 最终必须汇聚到同一套解析、验证和写回规则。

即使未来增加 Web Designer、其他桌面宿主或新的 Agent Harness，也应继续复用同一 Domain / Core / Editor，而不是复制领域规则。

### 4.2 TOML 与确定性验证：方向正确

当前设计具有以下优点：

- Project、Composition、Module Definition 有明确的声明边界；
- 模块 ID 与精确版本匹配；
- Property、Slot、Port 和 Dependency 通过契约声明；
- 默认值进入解析结果，但不因解析而擅自写回源文件；
- Binding、Connection 和 Dependency 分别检查；
- 未知字段、无效引用和不支持版本明确产生诊断；
- 规范化结果与诊断保持稳定排序；
- Source Map 让诊断可以定位到文件和范围。

目前的精确版本策略和显式失败策略应继续保持。不要为了“尽量能用”加入自动选择最新版本、隐式转换、自动 Adapter 或静默兼容。

### 4.3 编辑事务：基础正确，但后续需要跨进程边界

当前编辑机制已经覆盖候选模型、完整验证、Diff、Revision 冲突、request_id 幂等和保留格式 TOML 写回。

仍需注意：

- Revision 历史和 request 幂等记录目前只在进程内；
- 崩溃恢复处理 `.bak-*`，但没有 fsync 级断电保证；
- 外部文件变化通过定时 digest 检测，尚无文件系统 Watcher 或三向合并；
- Source 编辑目前只读，尚未纳入事务协议。

这些限制不需要在 v0.2 一次全部解决。应由实际的多进程 Agent、Source 编辑和恢复场景决定优先级。

### 4.4 MCP：保持薄适配器，不应承担业务规则

MCP 应只负责将外部调用映射到 Protocol / Editor / Core，不应：

- 在 MCP 层重复实现组合验证；
- 直接修改 TOML；
- 维护与 Core 不一致的项目模型；
- 根据工具调用习惯偷偷修正组合关系。

当前单进程单项目、无 subscriptions / progress / cancellation / authentication 等限制可以先接受。只有在长期任务、并发会话或远程服务场景成为真实需求后，再扩展对应协议能力。

### 4.5 主要架构风险

| 风险 | 可能后果 | 建议 |
|---|---|---|
| GUI 自行实现组合规则 | UI 与 Core 结果不一致 | 规则只在 Core；GUI 负责表达和交互 |
| Source 建立独立保存逻辑 | 出现第二套写入与冲突机制 | Source 候选文本必须走统一解析、验证、Diff 和事务 |
| 把所有关系画成普通边 | Binding / Dependency / Connection 语义混乱 | 保留三类关系的独立模型与视图 |
| 过早增加兼容层 | 隐式行为增加，修改影响不可预测 | 不兼容显式失败，扩展通过新契约表达 |
| 过早拆分 crate | 增加依赖和维护成本 | 先按职责边界组织，出现复用需求再拆分 |
| 过早引入多文件和版本求解 | 增加 Schema、引用和合并复杂度 | 由真实项目验证后再决定 |

---

## 5. 问题与改进建议

以下按优先级排序。优先级表示后续投入顺序，不代表所有问题都是当前代码缺陷。

### 5.1 P1 — 完整可视化组合编辑尚未完成

这是当前最重要的产品能力缺口。

Core / Editor 已经具备多种编辑命令，但当前 Designer 的主要交互路径尚未把这些能力完整暴露给用户。当前 Canvas 主要展示 Instance 和 Binding；节点连接能力关闭；Inspector 主要支持查看契约和修改部分 Config。

建议补齐以下能力：

| 操作 | Core / Editor | 当前 GUI 状态 | 后续目标 |
|---|---|---|---|
| 创建 Instance | 已支持 | 未形成完整入口 | 模块库选择后创建实例 |
| 删除 Instance | 已支持 | 未形成完整入口 | 受影响 Binding / Connection / Provider 一并预览 |
| 重命名 Instance | 已支持 | 未形成完整入口 | 显示引用重写影响 |
| 更换模块 / 版本 | 已支持 | 未形成完整入口 | 明确版本选择并重新验证契约 |
| 修改 Config | 已支持 | 已支持部分属性 | 完善类型、数组和默认值交互 |
| 创建 / 修改 / 删除 Binding | 已支持 | 未形成完整编辑流程 | 提供 Slot 感知的结构编辑 |
| 创建 / 修改 / 删除 Connection | 已支持 | 未形成完整编辑流程 | 提供 Port 感知的连接编辑 |
| 管理 Dependency Provider | 有相关模型与命令基础 | 未形成完整入口 | 展示要求、候选 Provider 和显式映射 |

**建议：** 不要先增加更多普通面板，而是围绕“完成一个组合修改任务”组织 UI。用户应能从模块选择开始，完成实例创建、关系编辑、配置修改、Diff 审查和事务提交。

### 5.2 P1 — Source 编辑仍为只读

Source 面板目前用于查看 TOML 和定位诊断，但不能直接编辑。

建议把 Source 编辑列入 v0.2，但必须明确唯一写入路径：

1. CodeMirror 中的文本作为未提交候选内容；
2. 将候选文本交给统一解析器；
3. 生成候选 Composition Model 和完整诊断；
4. 对比当前模型生成结构化 Diff 与影响范围；
5. 通过统一 Change Review 确认；
6. 通过统一事务机制提交并保留格式写回；
7. 写回后重新加载、验证并刷新 Revision。

Source 的临时文本状态可以属于编辑器会话，但不能形成第二套权威项目状态或第二套写入机制。

冲突场景必须明确：外部文件变化时，未提交 Source 文本不能静默覆盖磁盘内容；应保留用户草稿，并提供重新加载、重新基于最新 Revision 应用或放弃草稿的明确选项。

### 5.3 P1 — 自动化工程门禁与 UI 测试需要补齐

本次查看的仓库目录未发现 `.github/workflows/`。验收报告也明确记录没有前端自动化 UI 测试。因此，Core 已有测试不能完全保护 Designer 的交互行为。

建议建立 Windows 优先的 CI：

| 检查 | 建议要求 |
|---|---|
| `cargo test --workspace` | 必须通过 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 必须通过 |
| `cargo fmt --all --check` | 必须通过 |
| Tauri Host Tests | 必须通过 |
| TypeScript 类型检查 | 必须通过 |
| Vite Production Build | 必须通过 |
| UI 自动化测试 | 覆盖关键编辑链路 |
| MCP E2E | 覆盖 prepare / apply / validate |

建议优先以 `windows-latest` 作为主要构建平台，避免桌面应用的真实目标平台与 CI 环境脱节。若后续维护 macOS / Linux 构建，应分别验证平台依赖与宿主行为，不应把跨平台 CI 绿灯误认为跨平台 GUI 已验收。

UI 自动化测试第一批至少覆盖：

- 打开有效项目和无效项目；
- 选择 Instance 并查看 Inspector；
- 修改 Property 并显示 Change Review；
- 取消和确认事务；
- 事务失败时显示诊断；
- 外部修改后阻止冲突写入并正确 Reload；
- Canvas 布局保存不改变项目 Source Digest。

### 5.4 P2 — Agent 事务粒度需要更合理

当前 Agent Skill 建议“一次事务优先一个命令”。这有利于单项操作审查，但如果把它当作普遍规则，会造成过多重复调用和中间 Revision 更新。

建议改为：

> 一个完整修改意图对应一个事务；同一意图中相互依赖的命令可以批量提交。不同意图或需要独立审批的操作分开提交。

例如，创建一个实例、设置它的多个属性并将其 Binding 到父实例，通常属于同一个完整设计意图，可以在一个事务中完成。

推荐 Agent 工作流：

```text
inspect / resolve
       ↓
形成完整修改意图
       ↓
准备一个包含相关命令的 Transaction
       ↓
prepare_transaction
       ↓
检查整体 Diff、影响范围与诊断
       ↓
apply_transaction
       ↓
validate / resolve
       ↓
报告最终 Revision 与结果
```

不要为了减少事务数量而把互不相关的修改混在一起；也不要为了每次只修改一个字段而产生大量无必要的事务。

### 5.5 P2 — 跨进程 Revision 与幂等持久化

目前 Revision 历史和 request_id 幂等记录只在进程内。单进程单项目模式下可以接受，但多个 Agent 进程、MCP 会话或崩溃后恢复时会成为限制。

建议等多会话编辑成为明确需求后，再定义持久化策略：

- Revision 历史的存储位置与生命周期；
- request_id 的唯一性范围和保留期限；
- 进程重启后重复请求的重放语义；
- 外部文件变化与持久化 Revision 的一致性；
- 文件系统写入与事务日志的恢复顺序。

这可能需要单独的 Session Store / Journal，但不应为“可能以后需要”而提前引入数据库。

### 5.6 P2 — Module Definition 演进需要长期规则

目前 Module Definition 在 v0.1 中只读。未来开放修改之前，需要先定义版本化契约演进规则。

建议采用显式版本演进：

```text
shell.desktop@1.0.0
        │
        │ 契约发生变化
        ▼
shell.desktop@1.1.0
        │
        │ 项目显式选择升级
        ▼
Composition Model
```

核心原则：

- 已发布版本不可被静默覆盖；
- 修改契约应产生新版本；
- 项目明确选择要使用的版本；
- 升级前显示受影响的实例、Config、Slot、Port 和 Dependency；
- 不自动转换、不自动插入 Adapter；
- 如需迁移，必须是显式、可审查、可验证的迁移操作。

当前不建议立即开发模块编辑器，先通过真实项目确认版本演进需求。

---

## 6. 后续发展路线图

建议以 v0.2 为下一阶段总目标，分为 M6–M9。每个阶段独立验收，不要在一个阶段中同时改领域模型、重写 UI 和扩展 MCP。

## M6 — Complete Composition Authoring

<badge>最高优先级</badge>

**目标：** 人类用户可以通过 Designer 完成完整的项目组合修改，不再依赖直接调用 MCP 或手工编辑 TOML 才能完成常见组合操作。

### M6-001：建立自动化质量门禁

交付：

- Windows CI workflow；
- Rust 测试、Clippy、Rustfmt；
- Tauri Host 测试；
- TypeScript 检查与前端构建；
- 关键 UI 自动化测试框架与首批用例。

验收：核心工作流在 Pull Request 中自动执行；失败能阻止合并；测试结果可追溯。

### M6-002：真实项目试点与 Schema 复核

交付：

- 一个真实项目的代表性子系统；
- 模块、实例、Binding、Dependency、Connection 的完整声明；
- 对照现有 Schema 的表达能力检查；
- 不足项清单和明确的 Extend / Rearchitect 决策建议。

验收：在不增加隐式规则的前提下，能够清晰表达该子系统；所有无法表达的内容都有明确记录，不通过临时兼容补丁掩盖。

**顺序要求：M6-002 应先于大规模 UI 编辑功能。** 先验证当前模型是否足够，再决定是否扩展 Schema。

### M6-003：Instance 编辑

交付：

- 模块选择与精确版本选择；
- 创建 Instance；
- 删除 Instance；
- 重命名 Instance；
- 更换 Module / Version；
- 配置项编辑和默认值重置；
- 对删除、重命名、更换模块产生的级联影响进行 Diff 展示。

验收：所有操作都经过统一事务；不合法变更不会写入；GUI 结果与 CLI / MCP 结果一致。

### M6-004：Binding 编辑

交付：

- 选择父 Instance；
- 从父模块的 Slot 中选择目标 Slot；
- 选择子 Instance；
- 创建、修改和删除 Binding；
- 在交互时展示 Slot 接受范围、Cardinality 和当前占用情况。

验收：GUI 不自行复制 Slot 规则；候选关系经 Core 验证；非法操作显示对应诊断；删除和修改影响范围可审查。

### M6-005：Connection 与 Dependency 编辑视图

建议提供三个独立视图或投影：

| 视图 | 主要信息 | 操作目标 |
|---|---|---|
| Structure View | Instance、Slot、父子关系 | 编辑 Binding |
| Connection View | Port、方向、类型、协议、连接 | 编辑 Connection |
| Dependency View | 要求、Provider、解析状态 | 编辑 Provider 映射 |

三个视图共享同一 Composition Model，不维护独立语义模型。

验收：用户能够分别识别三种关系；视图之间切换不丢失选择上下文；每一种关系都有独立的创建、修改、删除和验证路径。

### M6-006：完整 Change Review

交付：

- 按对象类别展示新增、删除和修改；
- 展示 Before / After；
- 展示 Binding、Connection、Dependency 的影响；
- 展示完整验证结果；
- 支持取消和确认；
- 应用失败时保留诊断和当前项目状态。

验收：一次包含多个相关命令的事务能够被完整理解；用户确认前不写入权威文件；提交后 Revision、Diff、Source 和模型一致。

### M6 总验收链路

```text
从模块库选择模块
        ↓
创建 Instance
        ↓
建立 Binding
        ↓
配置 Property
        ↓
建立 Connection / 显式 Dependency Provider
        ↓
预览完整 Diff 与影响范围
        ↓
Composition Core 验证
        ↓
用户确认事务
        ↓
TOML 保留格式写回
        ↓
重新打开项目
        ↓
GUI / CLI / MCP 得到一致结果
```

M6 完成标准：至少使用一个真实项目子系统完成上述代表性链路；关键操作全部可通过 UI 自动化或可重复的集成测试验证。

---

## M7 — Source Authoring & Consistency

**目标：** TOML Source 编辑与可视化编辑拥有一致的语义、验证、Diff 和提交行为。

### M7-001：Source 候选编辑

- CodeMirror 可编辑当前 Composition Source；
- 编辑文本只作为未提交草稿；
- 解析与验证失败时，保留草稿并定位诊断；
- 解析成功时生成候选 Model；
- 生成结构化 Diff 和影响范围。

### M7-002：统一 Source 提交

- Source 修改使用统一 Change Review；
- 通过统一事务入口提交；
- 保留无关注释和格式；
- 写回后重新解析并验证；
- 不允许 Source 绕过 Revision / Digest 冲突守卫。

### M7-003：外部修改与草稿冲突处理

至少覆盖：

- 外部文件未变化，正常提交草稿；
- 外部文件发生变化，阻止旧 Revision 写入；
- 用户存在未提交草稿时收到外部修改；
- Reload 后保留或明确放弃草稿；
- 多种编辑入口同时修改时的冲突提示。

验收：GUI 编辑、Source 编辑、CLI 和 MCP 最终都经过相同 Core / Editor 语义；不存在第二套权威状态或保存机制。

---

## M8 — Agent Authoring Experience

**目标：** 让 Agent 更准确地理解项目、组织完整修改意图、观察执行结果并从诊断恢复，减少重复调用和局部补丁。

### M8-001：项目上下文与契约查询

建议补充或优化面向 Agent 的聚合查询，使 Agent 能高效获得：

- 项目身份、Schema 版本和 Revision；
- 模块定义及精确版本；
- Instance、Binding、Connection；
- Dependency 要求与 Provider 解析状态；
- 关键契约与相关诊断。

优先复用现有 Query / Protocol，不要为每一个字段创建独立 MCP Tool。

### M8-002：修改计划与影响分析

Agent 在 apply 之前应能够说明：

- 修改目的；
- 计划执行的命令；
- 预期新增、删除和修改对象；
- 可能受到影响的关系；
- 当前验证风险和不确定项。

这部分应优先利用已有结构化 Diff、诊断和影响集合，不应让 LLM 自己猜测项目影响范围。

### M8-003：事务批处理策略

调整 Agent Skill：

- 一个完整修改意图对应一个事务；
- 同一意图中相互依赖的命令可以批量提交；
- 不同意图或需要独立审批的操作分开提交；
- prepare 后审查整体 Diff，再 apply；
- apply 后重新验证并报告最终 Revision。

### M8-004：Agent Skill 评测集

建议建立固定任务集：

1. 从空白示例构建一个有效组合；
2. 为现有实例增加模块；
3. 修改一组相关配置；
4. 处理不符合 Slot 约束的组合；
5. 替换模块版本并检查契约影响；
6. 发现依赖歧义并显式指定 Provider；
7. 遇到外部修改后恢复到最新 Revision；
8. 处理无效声明而不引入隐式兼容层。

验收指标应包括：任务成功率、无效事务率、重复调用次数、无必要修改数量、是否绕过协议、是否正确报告失败。

### M8-005：跨进程 Revision / 幂等持久化评估

当多 Agent / 多 MCP 会话或重启恢复成为真实需求时，再设计持久化机制。需要明确 Revision 历史、request_id 唯一范围、重放语义、保留期限和文件写入恢复顺序。

---

## M9 — Composition Ecosystem（暂缓进入实现）

**目标：** 在核心 Authoring 能力稳定并经过真实项目验证后，探索可扩展模块和组合生态。

候选能力：

| 能力 | 说明 | 进入条件 |
|---|---|---|
| Module Registry | 管理本地或外部模块定义 | 模块身份、来源和版本规则明确 |
| Module Versioning | 模块契约版本演进 | 真实项目出现模块演进需求 |
| Composition Reuse | 将已有组合封装为可复用组合 | 当前单入口模型经试点证明不足 |
| Multi-file Composition | 拆分大型项目声明 | 出现明确规模瓶颈并完成 include 语义设计 |
| Project Templates | 复用常见项目骨架 | 已有稳定的组合模板和模块目录约定 |
| External Adapters | 接入其他工具链或配置系统 | 明确适配边界，不侵入 Core |

M9 不应被视为 v0.2 的承诺。它是后续演进方向，需由实际使用证据决定是否启动。

---

## 7. 建议暂缓的工作

| 暂缓事项 | 暂缓原因 | 重新评估条件 |
|---|---|---|
| 多 Agent 协作 | 单 Agent Authoring 体验尚未充分验证 | 单 Agent 任务边界和状态恢复稳定后 |
| 在线模块市场 | 本地模块契约和来源管理尚未验证 | Module Registry 与安全来源规则明确后 |
| 自动 Adapter | 容易引入隐式兼容和不可追踪行为 | 出现明确、可声明、可验证的适配契约需求后 |
| 自动代码生成 | 会混淆声明模型和模块实现 | 独立执行/生成层有明确设计后 |
| 项目运行时 | 不属于当前声明设计器职责 | 独立 Runtime 设计与接口冻结后 |
| 复杂版本求解 | 当前精确版本策略更容易确定性验证 | 真实模块生态产生明确求解需求后 |
| 多文件 Composition | 引入 include、覆盖、循环和合并语义复杂度 | 真实项目证明单入口已成为瓶颈后 |
| Module Definition 在线编辑 | 影响所有引用实例，缺少版本迁移机制 | 模块版本演进规则确定后 |
| MCP subscriptions / progress / cancellation | 当前短事务场景不一定需要 | 长任务、多会话或远程调用成为真实需求后 |

---

## 8. 实施原则与变更控制

后续继续遵守仓库现有 `AGENTS.md`、Change Manifest、ADR 和冻结规范机制。

### 8.1 领域模型与 Schema 冻结原则

M6 优先使用现有 Domain Model、TOML Schema 和 Editor Commands 完成 GUI 能力。

只有真实项目试点证明现有模型不能准确表达需求时，才提出 Extend / Rearchitect。不要因为 UI 实现不方便就随意增加字段或改变关系语义。

### 8.2 先明确语义，再实现界面

所有可编辑操作先明确：

- 操作对象；
- 前置条件；
- 允许的状态变化；
- 级联影响；
- 验证规则；
- 错误诊断；
- Revision 行为；
- TOML 写回行为。

然后才设计 GUI 交互。

### 8.3 一个修改意图，一个完整事务

避免两种极端：

- 每修改一个字段就提交一次，造成重复调用和 Revision 碎片；
- 将互不相关的修改放入同一事务，使审查和失败恢复变得困难。

以完整设计意图为事务边界，事务内部保持原子性。

### 8.4 所有入口共享结果

任何修改完成后，至少保证：

- Core 解析结果一致；
- 诊断代码与 Source Location 一致；
- Revision / Digest 一致；
- CLI / GUI / MCP 读取同一声明结果；
- Canvas 布局状态不改变项目语义。

### 8.5 每阶段独立验收

每个 M 阶段建立明确任务卡和 Change Manifest。阶段结束时：

- 回写 `working/plan.md`；
- 更新验收报告；
- 更新必要的 ADR / Spec；
- 记录未完成项和明确不支持项；
- 不把未执行的测试描述为通过。

---

## 9. 建议的近期任务顺序

建议将下一轮工作控制在以下范围，不要同时开启 M7、M8 和 M9。

| 顺序 | 建议任务 | 主要交付 | 优先级 |
|---|---|---|---|
| 1 | M6-001 自动化质量门禁 | CI + 前端测试基础 | 立即 |
| 2 | M6-002 真实项目试点 | Schema / Domain 表达能力复核 | 立即 |
| 3 | M6-003 Instance 编辑 | 完整实例操作链 | 高 |
| 4 | M6-004 Binding 编辑 | 完整结构组合链 | 高 |
| 5 | M6-005 Connection / Dependency 视图 | 三种关系完整表达 | 高 |
| 6 | M6-006 Change Review 完善 | 完整 Diff / 影响 / 确认闭环 | 高 |
| 7 | M7-001 Source 编辑 | TOML 与 Model 双向一致 | 中 |
| 8 | M8-001 Agent 评测与工作流 | 更少重复调用、更可靠恢复 | 中 |
| 9 | M8-005 Revision 持久化 | 多进程状态恢复 | 按需求 |

**执行建议：** 先创建 M6-001 任务卡并补齐质量门禁；随后进行 M6-002 真实项目试点。试点结果确认当前 Schema 足够后，再推进 Instance、Binding、Connection 和 Dependency 的 UI 编辑。

---

## 10. 最终结论

Composition Designer v0.1 的核心价值已经建立：项目组合有明确的领域模型和 TOML 声明；解析、验证和诊断具有确定性；修改通过事务完成；GUI、CLI 和 Agent 共享同一 Core。

下一阶段的主要问题不是缺少更多底层技术，而是**如何把已经建立的组合语义完整地交给人类和 Agent 使用**。

因此建议：

1. 正式收口 v0.1，不为新增 UI 需求立即修改冻结的领域模型；
2. 将 v0.2 定义为 Complete Composition Authoring；
3. 先补 CI 和 UI 测试，再用真实项目复核模型；
4. 优先完成 Instance、Binding、Connection、Dependency 的完整可视化编辑；
5. Source 编辑必须复用统一 Core、Diff、Revision 和事务机制；
6. Agent 侧重点转向完整修改意图、批量事务、影响分析和固定场景评测；
7. 运行时、代码生成、在线市场、多 Agent 和复杂版本求解继续独立规划、暂缓进入当前版本。

**核心方向：先完成组合设计，再考虑组合执行；先明确关系和契约，再考虑生态扩展。**

---

## 11. 参考文件

- [Repository README](https://github.com/Gitshimyan/composition-designer/blob/main/README.md)
- [v0.1 Design Baseline](https://github.com/Gitshimyan/composition-designer/blob/main/docs/design/v0.1-design.md)
- [v0.1 Acceptance Report](https://github.com/Gitshimyan/composition-designer/blob/main/docs/acceptance-v0.1.md)
- [Working Plan](https://github.com/Gitshimyan/composition-designer/blob/main/working/plan.md)
- [Domain Model v1](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/domain-model-v1.md)
- [Composition Schema v1](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/composition-schema-v1.md)
- [Command Schema v1](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/command-schema-v1.md)
- [Protocol v1](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/protocol-v1.md)
- [MCP Adapter v1](https://github.com/Gitshimyan/composition-designer/blob/main/docs/spec/mcp-adapter-v1.md)
- [ADR-0001: Domain Model and Schema Freeze](https://github.com/Gitshimyan/composition-designer/blob/main/docs/adr/0001-domain-and-schema-freeze.md)
- [ADR-0002: v0.1 Unsupported Items](https://github.com/Gitshimyan/composition-designer/blob/main/docs/adr/0002-v01-unsupported.md)
- [Agent Skill](https://github.com/Gitshimyan/composition-designer/blob/main/agent/SKILL.md)

## 12. 评审范围与限制

本文件基于评审时公开可访问的 `main` 分支 README、目录结构、核心规范、验收报告、Agent Skill 和部分关键实现源码形成。

本次没有在评审者本机执行完整 Rust / Tauri / npm 构建与测试，也没有进行实际桌面 UI 交互验收。因此：

- 验收数字引用仓库自身 `docs/acceptance-v0.1.md`；
- 对 UI 缺失能力的判断基于当前源码中可见的主要交互路径；
- 关于 CI 的判断仅表示本次查看的仓库树中未发现 `.github/workflows/`；
- 后续路线图属于评审建议，不代表已被项目正式采纳或排期。

在把本文转为正式实施计划前，建议由项目维护者结合实际 Windows 构建与真实项目试点结果，确认 M6 的任务边界和验收条件。
